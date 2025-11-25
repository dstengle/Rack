//! VCV Rack Terminal Client
//!
//! A terminal user interface for controlling VCV Rack patches via the HTTP API.

mod api;
mod app;
mod config;
mod state;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::api::RackApiClient;
use crate::app::{App, StateUpdate};
use crate::config::{AppSettings, CliArgs, Config};
use crate::ui::{handle_event, render, InputResult};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = CliArgs::parse();

    // Load configuration
    let mut config = Config::load(args.config.as_ref())?;
    config.merge_cli_args(&args);

    // Create settings
    let settings = AppSettings::from_config_and_args(&config, &args);

    // Create API client
    let api = RackApiClient::new(&settings.host, settings.port, settings.timeout_ms)
        .context("Failed to create API client")?;

    // Create application
    let mut app = App::new(api.clone(), settings.clone());

    // Initialize (fetch initial data)
    if let Err(e) = app.init().await {
        eprintln!("Warning: Failed to connect to VCV Rack: {}", e);
        eprintln!("Make sure VCV Rack is running with --httpapi flag");
        app.show_error(&format!("Cannot connect: {}", e));
    }

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create channel for background updates
    let (tx, rx) = mpsc::channel(100);

    // Spawn background polling task
    let api_clone = api.clone();
    let settings_clone = settings.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(settings_clone.refresh_interval_ms));
        let mut known_modules = std::collections::HashSet::new();

        loop {
            interval.tick().await;

            // Try to fetch modules directly
            match api_clone.get_modules().await {
                Ok(modules) => {
                    let _ = tx.send(StateUpdate::ConnectionStatus(true)).await;
                    
                    // Check for new modules and fetch details
                    for module in &modules {
                        if !known_modules.contains(&module.id) {
                            if let Ok(details) = api_clone.get_module_details(module.id).await {
                                let _ = tx.send(StateUpdate::ModuleDetails(details)).await;
                                known_modules.insert(module.id);
                            }
                        }
                    }
                    let _ = tx.send(StateUpdate::Modules(modules)).await;

                    // Fetch cables
                    match api_clone.get_cables().await {
                        Ok(cables) => {
                            let _ = tx.send(StateUpdate::Cables(cables)).await;
                        }
                        Err(_) => {
                            // Ignore cable fetch errors if modules succeeded, 
                            // or maybe just log it but don't disconnect yet.
                        }
                    }
                }
                Err(_) => {
                    let _ = tx.send(StateUpdate::ConnectionStatus(false)).await;
                }
            }
        }
    });

    // Run the main loop
    let result = run_app(&mut terminal, &mut app, rx).await;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    result
}

/// Main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut rx: mpsc::Receiver<StateUpdate>,
) -> Result<()> {

    loop {
        // Render UI
        terminal.draw(|f| render(f, app))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            match handle_event(app, event) {
                InputResult::Exit => break,
                InputResult::Continue(Some(cmd)) => {
                    // Execute command
                    match app.execute(cmd).await {
                        Ok(should_quit) => {
                            if should_quit {
                                break;
                            }
                        }
                        Err(e) => {
                            app.show_error(&e.to_string());
                        }
                    }
                }
                InputResult::Continue(None) => {}
            }
        }

        // Check for updates from background task
        while let Ok(update) = rx.try_recv() {
            app.apply_update(update);
        }
    }

    Ok(())
}
