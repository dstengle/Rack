//! Main application state and logic

use anyhow::{Context, Result};
use std::collections::VecDeque;

use crate::api::{Cable, CreateCableRequest, CreateModuleRequest, ModuleDetails, PatchModule, Position, RackApiClient};
use crate::config::AppSettings;
use crate::state::{CableManager, LocalModule, ModuleManager};
use crate::ui::CompletionEngine;

/// Updates from the background task
#[derive(Debug)]
pub enum StateUpdate {
    /// List of modules updated
    Modules(Vec<PatchModule>),
    /// List of cables updated
    Cables(Vec<Cable>),
    /// Module details updated
    ModuleDetails(ModuleDetails),
    /// Connection status changed
    ConnectionStatus(bool),
    /// Error occurred
    Error(String),
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal browsing mode
    Normal,
    /// Input mode (typing command)
    Input,
    /// Showing help screen
    Help,
    /// Showing module detail view
    ModuleDetail,
}

/// Commands that can be executed
#[derive(Debug, Clone)]
pub enum Command {
    /// Add a module to the patch
    Add(String),
    /// Remove a module from the patch
    Remove(String),
    /// Create a cable connection
    Connect(String),
    /// Remove a cable connection
    Disconnect(String),
    /// List modules or cables
    List(Option<String>),
    /// Show module details
    Show(String),
    /// Show help
    Help(Option<String>),
    /// Refresh from server
    Refresh,
    /// Quit the application
    Quit,
}

/// Main application state
pub struct App {
    /// API client
    pub api: RackApiClient,
    /// Application settings
    pub settings: AppSettings,
    /// Module manager
    pub modules: ModuleManager,
    /// Cable manager
    pub cables: CableManager,
    /// Completion engine
    pub completion: CompletionEngine,
    /// Current input text
    pub input: String,
    /// Command history
    pub history: VecDeque<String>,
    /// Current history index (for navigation)
    pub history_index: Option<usize>,
    /// Current application mode
    pub mode: AppMode,
    /// Whether connected to server
    pub connected: bool,
    /// Server URL for display
    pub server_url: String,
    /// Currently selected module index in list
    pub selected_module: Option<usize>,
    /// Module being shown in detail view
    pub detail_module: Option<LocalModule>,
    /// Current status/error message
    pub message: Option<String>,
    /// Whether current message is an error
    pub message_is_error: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(api: RackApiClient, settings: AppSettings) -> Self {
        let server_url = api.base_url().to_string();
        Self {
            api,
            settings: settings.clone(),
            modules: ModuleManager::new(),
            cables: CableManager::new(),
            completion: CompletionEngine::new(settings.max_suggestions),
            input: String::new(),
            history: VecDeque::with_capacity(100),
            history_index: None,
            mode: AppMode::Normal,
            connected: false,
            server_url,
            selected_module: None,
            detail_module: None,
            message: None,
            message_is_error: false,
        }
    }

    /// Initialize the application (fetch initial data)
    pub async fn init(&mut self) -> Result<()> {
        // Check connection
        self.connected = self.api.health_check().await.unwrap_or(false);

        if self.connected {
            // Fetch available models
            let models = self.api.get_models().await?;
            self.modules.set_available_models(models);
        }

        Ok(())
    }

    /// Apply an update from the background task
    pub fn apply_update(&mut self, update: StateUpdate) {
        match update {
            StateUpdate::Modules(modules) => {
                self.modules.sync_modules(modules);
                self.connected = true;
            }
            StateUpdate::Cables(cables) => {
                self.cables.sync_cables(cables, &self.modules);
                // Update port connections based on cables
                // This avoids fetching details for every module constantly
                self.modules.update_port_connections(self.cables.all_cables());
                self.connected = true;
            }
            StateUpdate::ModuleDetails(details) => {
                self.modules.update_module_details(details);
            }
            StateUpdate::ConnectionStatus(status) => {
                self.connected = status;
            }
            StateUpdate::Error(msg) => {
                // Only show error if we were previously connected or it's a new error
                if self.connected {
                     self.show_error(&msg);
                }
            }
        }
    }

    /// Execute a command
    pub async fn execute(&mut self, cmd: Command) -> Result<bool> {
        match cmd {
            Command::Add(search) => {
                self.cmd_add(&search).await?;
            }
            Command::Remove(name) => {
                self.cmd_remove(&name).await?;
            }
            Command::Connect(args) => {
                self.cmd_connect(&args).await?;
            }
            Command::Disconnect(args) => {
                self.cmd_disconnect(&args).await?;
            }
            Command::List(what) => {
                self.cmd_list(what.as_deref());
            }
            Command::Show(name) => {
                self.cmd_show(&name).await?;
            }
            Command::Help(topic) => {
                self.cmd_help(topic.as_deref());
            }
            Command::Refresh => {
                // Trigger a manual refresh (handled by background task mostly, but we can force status)
                self.show_message("Refreshed from server");
            }
            Command::Quit => {
                return Ok(true); // Signal to quit
            }
        }
        Ok(false)
    }

    /// Add a module
    async fn cmd_add(&mut self, search: &str) -> Result<()> {
        // Find matching model
        let model = self
            .modules
            .find_available_model(search)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", search))?
            .clone();

        // Calculate position
        let x = self.modules.next_x_position(self.settings.module_spacing);

        // Create module
        let request = CreateModuleRequest {
            plugin_slug: model.plugin_slug.clone(),
            model_slug: model.slug.clone(),
            pos: Position { x, y: 0.0 },
        };

        let response = self
            .api
            .create_module(request)
            .await
            .context("Failed to create module")?;

        // Add to local state
        self.modules
            .add_module(response.id, &model.plugin_slug, &model.slug);

        // Fetch details for the new module
        if let Ok(details) = self.api.get_module_details(response.id).await {
            self.modules.update_module_details(details);
        }

        let friendly_name = self
            .modules
            .get_by_id(response.id)
            .map(|m| m.friendly_name.clone())
            .unwrap_or_else(|| format!("Module {}", response.id));

        self.show_message(&format!("Added {} ({})", friendly_name, model.full_name));
        Ok(())
    }

    /// Remove a module
    async fn cmd_remove(&mut self, name: &str) -> Result<()> {
        let module = self
            .modules
            .get_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", name))?;

        let id = module.id;
        let friendly_name = module.friendly_name.clone();

        self.api
            .delete_module(id)
            .await
            .context("Failed to delete module")?;

        self.modules.remove_module(id);

        // Refresh cables since they may have been disconnected
        let cables = self.api.get_cables().await?;
        self.cables.sync_cables(cables, &self.modules);

        self.show_message(&format!("Removed {}", friendly_name));
        Ok(())
    }

    /// Create a cable connection
    async fn cmd_connect(&mut self, args: &str) -> Result<()> {
        let (src_module, src_port, dst_module, dst_port) = Self::parse_connect_args(args)?;

        // Resolve source module and port
        let src = self
            .modules
            .get_by_name(&src_module)
            .ok_or_else(|| anyhow::anyhow!("Source module not found: {}", src_module))?;

        let src_port_obj = src
            .find_output(&src_port)
            .ok_or_else(|| anyhow::anyhow!("Output port not found: {}", src_port))?;

        let src_id = src.id;
        let src_port_id = src_port_obj.id;

        // Resolve destination module and port
        let dst = self
            .modules
            .get_by_name(&dst_module)
            .ok_or_else(|| anyhow::anyhow!("Target module not found: {}", dst_module))?;

        let dst_port_obj = dst
            .find_input(&dst_port)
            .ok_or_else(|| anyhow::anyhow!("Input port not found: {}", dst_port))?;

        let dst_id = dst.id;
        let dst_port_id = dst_port_obj.id;

        // Create cable
        let request = CreateCableRequest {
            output_module_id: src_id,
            output_id: src_port_id,
            input_module_id: dst_id,
            input_id: dst_port_id,
        };

        let response = self
            .api
            .create_cable(request)
            .await
            .context("Failed to create cable")?;

        // Add to local state
        let cable = crate::api::Cable {
            id: response.id,
            output_module_id: response.output_module_id,
            output_id: response.output_id,
            input_module_id: response.input_module_id,
            input_id: response.input_id,
        };
        self.cables.add_cable(&cable, &self.modules);

        // Update port connected status
        if let Ok(details) = self.api.get_module_details(src_id).await {
            self.modules.update_module_details(details);
        }
        if let Ok(details) = self.api.get_module_details(dst_id).await {
            self.modules.update_module_details(details);
        }

        self.show_message(&format!(
            "Connected {}:{} -> {}:{}",
            src_module, src_port, dst_module, dst_port
        ));
        Ok(())
    }

    /// Remove a cable connection
    async fn cmd_disconnect(&mut self, args: &str) -> Result<()> {
        let (src_module, src_port, dst_module, dst_port) = Self::parse_connect_args(args)?;

        // Find the cable
        let cable = self
            .cables
            .find_cable(&src_module, &src_port, &dst_module, &dst_port)
            .ok_or_else(|| anyhow::anyhow!("Cable not found"))?;

        let cable_id = cable.id;
        let src_id = cable.output_module_id;
        let dst_id = cable.input_module_id;

        self.api
            .delete_cable(cable_id)
            .await
            .context("Failed to delete cable")?;

        self.cables.remove_cable(cable_id);

        // Update port connected status
        if let Ok(details) = self.api.get_module_details(src_id).await {
            self.modules.update_module_details(details);
        }
        if let Ok(details) = self.api.get_module_details(dst_id).await {
            self.modules.update_module_details(details);
        }

        self.show_message(&format!(
            "Disconnected {}:{} from {}:{}",
            src_module, src_port, dst_module, dst_port
        ));
        Ok(())
    }

    /// Parse connect/disconnect arguments
    fn parse_connect_args(args: &str) -> Result<(String, String, String, String)> {
        // Parse arguments respecting quotes
        let mut parsed_args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in args.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        parsed_args.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        if !current.is_empty() {
            parsed_args.push(current);
        }

        if parsed_args.len() != 2 {
            anyhow::bail!("Usage: connect <module:port> <module:port>");
        }

        let src_parts: Vec<&str> = parsed_args[0].splitn(2, ':').collect();
        let dst_parts: Vec<&str> = parsed_args[1].splitn(2, ':').collect();

        if src_parts.len() != 2 || dst_parts.len() != 2 {
            anyhow::bail!("Invalid format. Use: module:port or \"module name\":port");
        }

        Ok((
            src_parts[0].to_string(),
            src_parts[1].to_string(),
            dst_parts[0].to_string(),
            dst_parts[1].to_string(),
        ))
    }

    /// List modules or cables
    fn cmd_list(&mut self, what: Option<&str>) {
        match what {
            Some("cables") => {
                let count = self.cables.count();
                self.show_message(&format!("Listing {} cables", count));
            }
            Some("modules") | None => {
                let count = self.modules.count();
                self.show_message(&format!("Listing {} modules", count));
            }
            Some(other) => {
                self.show_error(&format!("Unknown list type: {}. Use 'modules' or 'cables'", other));
            }
        }
    }

    /// Show module details
    async fn cmd_show(&mut self, name: &str) -> Result<()> {
        let module = self
            .modules
            .get_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", name))?;

        // Fetch fresh details
        let details = self.api.get_module_details(module.id).await?;
        self.modules.update_module_details(details);

        // Get updated module and clone for display
        let module = self
            .modules
            .get_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", name))?
            .clone();

        self.detail_module = Some(module);
        self.mode = AppMode::ModuleDetail;
        Ok(())
    }

    /// Show help
    fn cmd_help(&mut self, _topic: Option<&str>) {
        self.mode = AppMode::Help;
    }

    /// Show a status message
    pub fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
        self.message_is_error = false;
    }

    /// Show an error message
    pub fn show_error(&mut self, msg: &str) {
        self.message = Some(format!("Error: {}", msg));
        self.message_is_error = true;
    }

    /// Add command to history
    pub fn add_to_history(&mut self, cmd: &str) {
        if !cmd.is_empty() {
            // Don't add duplicates of the last command
            if self.history.front() != Some(&cmd.to_string()) {
                self.history.push_front(cmd.to_string());
                if self.history.len() > 100 {
                    self.history.pop_back();
                }
            }
        }
        self.history_index = None;
    }

    /// Navigate to previous history entry
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(0);
                if let Some(cmd) = self.history.get(0) {
                    self.input = cmd.clone();
                }
            }
            Some(idx) if idx + 1 < self.history.len() => {
                self.history_index = Some(idx + 1);
                if let Some(cmd) = self.history.get(idx + 1) {
                    self.input = cmd.clone();
                }
            }
            _ => {}
        }
    }

    /// Navigate to next history entry
    pub fn history_next(&mut self) {
        match self.history_index {
            Some(0) => {
                self.history_index = None;
                self.input.clear();
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
                if let Some(cmd) = self.history.get(idx - 1) {
                    self.input = cmd.clone();
                }
            }
            None => {}
        }
    }
}
