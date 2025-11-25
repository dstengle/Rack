//! Completion system for commands and arguments

use crate::state::ModuleManager;

/// Types of completion context
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// Completing a command name
    Command,
    /// Completing an available module to add
    AvailableModule,
    /// Completing a module currently in the patch
    PatchModule,
    /// Completing a port on a specific module
    Port {
        module_name: String,
        is_output: bool,
    },
    /// No completion available
    None,
}

/// A completion suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The text to insert
    pub text: String,
    /// Display text (may include additional info)
    pub display: String,
    /// Optional description
    pub description: Option<String>,
}

/// Available commands
pub const COMMANDS: &[(&str, &str)] = &[
    ("add", "Add a module to the patch"),
    ("remove", "Remove a module from the patch"),
    ("connect", "Create a cable connection"),
    ("disconnect", "Remove a cable connection"),
    ("list", "List modules or cables"),
    ("show", "Show module details"),
    ("help", "Show help"),
    ("refresh", "Refresh state from server"),
    ("quit", "Exit the application"),
    ("exit", "Exit the application"),
];

/// Completion engine
#[derive(Debug)]
pub struct CompletionEngine {
    /// Current suggestions
    pub suggestions: Vec<Suggestion>,
    /// Currently selected suggestion index
    pub selected: usize,
    /// Whether the completion popup is visible
    pub visible: bool,
    /// Maximum number of suggestions to show
    max_suggestions: usize,
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self {
            suggestions: Vec::new(),
            selected: 0,
            visible: false,
            max_suggestions: 10,
        }
    }
}

impl CompletionEngine {
    /// Create a new completion engine
    pub fn new(max_suggestions: usize) -> Self {
        Self {
            max_suggestions,
            ..Default::default()
        }
    }

    /// Parse arguments respecting quoted strings
    /// Returns a vector of arguments, with quotes removed
    fn parse_quoted_args(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            args.push(current);
        }

        args
    }

    /// Determine completion context from current input
    pub fn get_context(input: &str) -> CompletionContext {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let trailing_space = input.ends_with(' ');

        if parts.is_empty() {
            return CompletionContext::Command;
        }

        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "add" => {
                if parts.len() == 1 && !trailing_space {
                    CompletionContext::Command
                } else {
                    CompletionContext::AvailableModule
                }
            }
            "remove" | "show" => {
                if parts.len() == 1 && !trailing_space {
                    CompletionContext::Command
                } else {
                    CompletionContext::PatchModule
                }
            }
            "connect" | "disconnect" => {
                if parts.len() == 1 && !trailing_space {
                    return CompletionContext::Command;
                }

                // Parse with quote support
                let args = Self::parse_quoted_args(&input[cmd.len()..].trim());
                
                // Determine if we're in a quote
                let in_quote = input.chars().filter(|&c| c == '"').count() % 2 == 1;
                
                if args.is_empty() {
                    // No arguments yet, complete source module
                    CompletionContext::PatchModule
                } else if args.len() == 1 {
                    if trailing_space && !in_quote {
                        // First argument complete with trailing space -> start destination module
                        CompletionContext::PatchModule
                    } else {
                        // Still typing first argument
                        let arg = args[0].as_str();
                        Self::parse_module_port_context(arg, true)
                    }
                } else if args.len() == 2 {
                    if trailing_space && !in_quote {
                        // Both arguments complete, no more completion
                        CompletionContext::None
                    } else {
                        // Still typing second argument
                        let arg = args[1].as_str();
                        Self::parse_module_port_context(arg, false)
                    }
                } else {
                    CompletionContext::None
                }
            }
            "list" => {
                if parts.len() == 1 && !trailing_space {
                    CompletionContext::Command
                } else {
                    CompletionContext::None // Could add "modules" / "cables" completion
                }
            }
            "help" => {
                if parts.len() == 1 && !trailing_space {
                    CompletionContext::Command
                } else {
                    CompletionContext::Command // Complete command for help
                }
            }
            _ => {
                if parts.len() == 1 && !trailing_space {
                    CompletionContext::Command
                } else {
                    CompletionContext::None
                }
            }
        }
    }

    /// Parse module:port context
    fn parse_module_port_context(arg: &str, is_output: bool) -> CompletionContext {
        if arg.contains(':') {
            // We have a colon, complete the port
            let module_name = arg.split(':').next().unwrap_or("").to_string();
            CompletionContext::Port {
                module_name,
                is_output,
            }
        } else {
            // No colon, complete module name
            CompletionContext::PatchModule
        }
    }

    /// Update suggestions based on input and context
    pub fn update(&mut self, input: &str, modules: &ModuleManager) {
        let context = Self::get_context(input);
        let query = Self::get_query(input, &context);

        self.suggestions = match context {
            CompletionContext::Command => self.complete_command(&query),
            CompletionContext::AvailableModule => self.complete_available_module(&query, modules),
            CompletionContext::PatchModule => self.complete_patch_module(&query, modules),
            CompletionContext::Port {
                module_name,
                is_output,
            } => self.complete_port(&query, &module_name, is_output, modules),
            CompletionContext::None => Vec::new(),
        };

        // Ensure selected index is valid
        if self.selected >= self.suggestions.len() {
            self.selected = 0;
        }

        self.visible = !self.suggestions.is_empty();
    }

    /// Extract the query portion from input
    fn get_query(input: &str, context: &CompletionContext) -> String {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let trailing_space = input.ends_with(' ');

        match context {
            CompletionContext::Command => {
                if trailing_space || parts.is_empty() {
                    String::new()
                } else {
                    parts.last().unwrap_or(&"").to_string()
                }
            }
            CompletionContext::AvailableModule | CompletionContext::PatchModule => {
                if trailing_space {
                    String::new()
                } else if parts.len() > 1 {
                    // Use quote-aware parsing to get the current argument
                    let cmd = parts[0];
                    let args = Self::parse_quoted_args(&input[cmd.len()..].trim());
                    // Return the last (incomplete) argument
                    args.last().map(|s| s.to_string()).unwrap_or_default()
                } else {
                    String::new()
                }
            }
            CompletionContext::Port { .. } => {
                // For port completion, we need to find the text after the last colon
                // Use quote-aware parsing to get the current argument
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.is_empty() {
                    return String::new();
                }
                let cmd = parts[0];
                let args = Self::parse_quoted_args(&input[cmd.len()..].trim());
                
                // Get the last argument (the one we're currently editing)
                let current_arg = if trailing_space {
                    ""
                } else {
                    args.last().map(|s| s.as_str()).unwrap_or("")
                };
                
                // Extract the part after the colon
                if let Some(colon_pos) = current_arg.rfind(':') {
                    current_arg[colon_pos + 1..].to_string()
                } else {
                    String::new()
                }
            }
            CompletionContext::None => String::new(),
        }
    }

    /// Complete command names
    fn complete_command(&self, query: &str) -> Vec<Suggestion> {
        let query_lower = query.to_lowercase();
        COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(&query_lower))
            .take(self.max_suggestions)
            .map(|(cmd, desc)| Suggestion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                description: Some(desc.to_string()),
            })
            .collect()
    }

    /// Complete available modules (for add command)
    fn complete_available_module(&self, query: &str, modules: &ModuleManager) -> Vec<Suggestion> {
        modules
            .search_available_models(query, self.max_suggestions)
            .into_iter()
            .map(|m| Suggestion {
                text: m.full_name.clone(),
                display: m.full_name.clone(),
                description: if m.description.is_empty() {
                    None
                } else {
                    Some(m.description.clone())
                },
            })
            .collect()
    }

    /// Complete patch modules (for remove, show, connect, etc.)
    fn complete_patch_module(&self, query: &str, modules: &ModuleManager) -> Vec<Suggestion> {
        modules
            .search_modules(query)
            .into_iter()
            .take(self.max_suggestions)
            .map(|m| Suggestion {
                text: m.friendly_name.clone(),
                display: format!("{} ({})", m.friendly_name, m.full_name),
                description: Some(format!(
                    "in:{} out:{}",
                    m.inputs.len(),
                    m.outputs.len()
                )),
            })
            .collect()
    }

    /// Complete port names
    fn complete_port(
        &self,
        query: &str,
        module_name: &str,
        is_output: bool,
        modules: &ModuleManager,
    ) -> Vec<Suggestion> {
        let Some(module) = modules.get_by_name(module_name) else {
            return Vec::new();
        };

        let ports = if is_output {
            &module.outputs
        } else {
            &module.inputs
        };

        let query_lower = query.to_lowercase();

        ports
            .iter()
            .filter(|p| query_lower.is_empty() || p.name.to_lowercase().contains(&query_lower))
            .take(self.max_suggestions)
            .map(|p| {
                let status = if p.connected {
                    " (connected)"
                } else {
                    ""
                };
                Suggestion {
                    text: p.name.clone(),
                    display: format!("{}{}", p.name, status),
                    description: None,
                }
            })
            .collect()
    }

    /// Select next suggestion
    pub fn next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
        }
    }

    /// Select previous suggestion
    pub fn prev(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected == 0 {
                self.selected = self.suggestions.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    /// Get the currently selected suggestion
    pub fn selected_suggestion(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected)
    }

    /// Apply the selected suggestion to the input
    pub fn apply(&self, input: &str, modules: &ModuleManager) -> Option<String> {
        let suggestion = self.selected_suggestion()?;
        let context = Self::get_context(input);

        Some(match context {
            CompletionContext::Command => suggestion.text.clone(),
            CompletionContext::AvailableModule => {
                format!("add {}", suggestion.text)
            }
            CompletionContext::PatchModule => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                let cmd = parts[0];
                // Add quotes if module name contains spaces
                let module_text = if suggestion.text.contains(' ') {
                    format!("\"{}\"", suggestion.text)
                } else {
                    suggestion.text.clone()
                };
                format!("{} {}", cmd, module_text)
            }
            CompletionContext::Port { module_name, is_output } => {
                // Parse with quote support to get arguments
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                let cmd = parts[0];
                let args = Self::parse_quoted_args(&input[cmd.len()..].trim());
                
                // Look up the actual module to get its friendly name
                // This ensures we use the friendly name consistently, not whatever the user typed
                let module = modules.get_by_name(&module_name)?;
                let friendly_name = &module.friendly_name;
                
                // Format module name with quotes if needed
                let format_module = |name: &str| {
                    if name.contains(' ') {
                        format!("\"{}\"", name)
                    } else {
                        name.to_string()
                    }
                };

                if args.is_empty() || args.len() == 1 {
                    // Completing first argument (source)
                    format!("{} {}:{} ", cmd, format_module(friendly_name), suggestion.text)
                } else {
                    // Completing second argument (destination)
                    // Look up the first argument's module to get its friendly name
                    let first_arg = &args[0];
                    let first_module = modules.get_by_name(first_arg)?;
                    let first_friendly_name = &first_module.friendly_name;
                    let first_arg_formatted = format_module(first_friendly_name);
                    format!("{} {} {}:{}", cmd, first_arg_formatted, format_module(friendly_name), suggestion.text)
                }
            }
            CompletionContext::None => return None,
        })
    }

    /// Hide the completion popup
    pub fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.selected = 0;
    }
}
