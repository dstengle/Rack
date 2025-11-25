//! Configuration loading and CLI argument parsing

use anyhow::{Context, Result};
use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// VCV Rack Terminal Client
#[derive(Parser, Debug)]
#[command(name = "vcvrack-tui")]
#[command(author, version, about = "A terminal user interface client for VCV Rack", long_about = None)]
pub struct CliArgs {
    /// Server hostname
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Config file path (default: ~/.config/vcvrack-tui/config.toml)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Disable colors
    #[arg(long)]
    pub no_color: bool,
}

/// Server configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout_ms: default_timeout(),
        }
    }
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    5000
}

/// UI configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_ms: u64,
    #[serde(default = "default_color_theme")]
    pub color_theme: String,
    #[serde(default = "default_true")]
    pub show_help_hints: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: default_refresh_interval(),
            color_theme: default_color_theme(),
            show_help_hints: true,
        }
    }
}

fn default_refresh_interval() -> u64 {
    1000
}

fn default_color_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

/// Completion configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfig {
    #[serde(default = "default_true")]
    pub auto_suggest: bool,
    #[serde(default = "default_true")]
    pub fuzzy_match: bool,
    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: usize,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            auto_suggest: true,
            fuzzy_match: true,
            max_suggestions: default_max_suggestions(),
        }
    }
}

fn default_max_suggestions() -> usize {
    10
}

/// Layout configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default = "default_module_spacing")]
    pub module_spacing: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            module_spacing: default_module_spacing(),
        }
    }
}

fn default_module_spacing() -> f64 {
    100.0
}

/// Full configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub completion: CompletionConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
}

impl Config {
    /// Get the default config file path
    pub fn default_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "vcvrack", "vcvrack-tui")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load configuration from file
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        let config_path = match path {
            Some(p) => Some(p.clone()),
            None => Self::default_config_path(),
        };

        if let Some(path) = config_path {
            if path.exists() {
                let content =
                    fs::read_to_string(&path).context("Failed to read configuration file")?;
                let config: Config =
                    toml::from_str(&content).context("Failed to parse configuration file")?;
                return Ok(config);
            }
        }

        Ok(Config::default())
    }

    /// Save default configuration to file
    pub fn save_default() -> Result<PathBuf> {
        let path = Self::default_config_path().context("Could not determine config path")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let config = Config::default();
        let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;
        fs::write(&path, content).context("Failed to write config file")?;

        Ok(path)
    }

    /// Merge CLI arguments into configuration
    pub fn merge_cli_args(&mut self, args: &CliArgs) {
        // CLI args override config file values
        self.server.host = args.host.clone();
        self.server.port = args.port;
    }
}

/// Application settings (merged from config and CLI)
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub refresh_interval_ms: u64,
    pub color_enabled: bool,
    pub show_help_hints: bool,
    pub auto_suggest: bool,
    pub fuzzy_match: bool,
    pub max_suggestions: usize,
    pub module_spacing: f64,
}

impl AppSettings {
    /// Create settings from config and CLI args
    pub fn from_config_and_args(config: &Config, args: &CliArgs) -> Self {
        Self {
            host: args.host.clone(),
            port: args.port,
            timeout_ms: config.server.timeout_ms,
            refresh_interval_ms: config.ui.refresh_interval_ms,
            color_enabled: !args.no_color,
            show_help_hints: config.ui.show_help_hints,
            auto_suggest: config.completion.auto_suggest,
            fuzzy_match: config.completion.fuzzy_match,
            max_suggestions: config.completion.max_suggestions,
            module_spacing: config.layout.module_spacing,
        }
    }
}
