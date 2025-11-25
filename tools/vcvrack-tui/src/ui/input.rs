//! Input handling for the TUI

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppMode, Command};

/// Result of handling an input event
pub enum InputResult {
    /// Continue running, optionally with a command to execute
    Continue(Option<Command>),
    /// Exit the application
    Exit,
}

/// Handle a terminal event
pub fn handle_event(app: &mut App, event: Event) -> InputResult {
    match event {
        Event::Key(key_event) => handle_key(app, key_event),
        Event::Resize(_, _) => InputResult::Continue(None),
        _ => InputResult::Continue(None),
    }
}

/// Handle a key event
fn handle_key(app: &mut App, key: KeyEvent) -> InputResult {
    // Clear message on any key press
    if app.message.is_some() {
        app.message = None;
        // If it was just showing a message, continue with the key handling
    }

    // Handle mode-specific keys
    match app.mode {
        AppMode::Help => {
            // Any key returns from help
            app.mode = AppMode::Normal;
            return InputResult::Continue(None);
        }
        AppMode::ModuleDetail => {
            // Escape returns from module detail
            if key.code == KeyCode::Esc {
                app.mode = AppMode::Normal;
                app.detail_module = None;
            }
            return InputResult::Continue(None);
        }
        AppMode::Normal | AppMode::Input => {
            // Handle normal/input mode keys below
        }
    }

    // Handle global keys
    match (key.modifiers, key.code) {
        // Ctrl+C - Exit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            return InputResult::Exit;
        }
        // Ctrl+R - Refresh
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            return InputResult::Continue(Some(Command::Refresh));
        }
        // Ctrl+U - Clear input
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
            app.input.clear();
            app.completion.hide();
            return InputResult::Continue(None);
        }
        _ => {}
    }

    // Handle input keys
    match key.code {
        // ? - Show help (only when input is empty)
        KeyCode::Char('?') if app.input.is_empty() => {
            app.mode = AppMode::Help;
            InputResult::Continue(None)
        }

        // F1 - Show help
        KeyCode::F(1) => {
            app.mode = AppMode::Help;
            InputResult::Continue(None)
        }

        // Escape - Close completion or clear input
        KeyCode::Esc => {
            if app.completion.visible {
                app.completion.hide();
            } else if !app.input.is_empty() {
                app.input.clear();
            }
            InputResult::Continue(None)
        }

        // Tab - Trigger/cycle completion
        KeyCode::Tab => {
            if app.completion.visible {
                // Apply current selection and cycle to next
                if let Some(completed) = app.completion.apply(&app.input) {
                    app.input = completed;
                }
                app.completion.next();
            } else {
                // Trigger completion
                app.completion.update(&app.input, &app.modules);
            }
            InputResult::Continue(None)
        }

        // Backtab (Shift+Tab) - Previous completion
        KeyCode::BackTab => {
            if app.completion.visible {
                app.completion.prev();
            }
            InputResult::Continue(None)
        }

        // Up arrow - Previous completion or history
        KeyCode::Up => {
            if app.completion.visible {
                app.completion.prev();
            } else {
                // Navigate command history
                app.history_prev();
            }
            InputResult::Continue(None)
        }

        // Down arrow - Next completion or history
        KeyCode::Down => {
            if app.completion.visible {
                app.completion.next();
            } else {
                // Navigate command history
                app.history_next();
            }
            InputResult::Continue(None)
        }

        // Enter - Execute command or apply completion
        KeyCode::Enter => {
            if app.completion.visible {
                // Apply selected completion
                if let Some(completed) = app.completion.apply(&app.input) {
                    app.input = completed;
                }
                app.completion.hide();
                InputResult::Continue(None)
            } else if !app.input.is_empty() {
                // Execute command
                let input = app.input.clone();
                app.add_to_history(&input);
                app.input.clear();
                app.completion.hide();

                // Parse and return command
                match parse_command(&input) {
                    Some(cmd) => InputResult::Continue(Some(cmd)),
                    None => {
                        app.show_error("Unknown command. Type 'help' for available commands.");
                        InputResult::Continue(None)
                    }
                }
            } else {
                InputResult::Continue(None)
            }
        }

        // Backspace - Delete character
        KeyCode::Backspace => {
            app.input.pop();
            // Update completion on input change
            if app.completion.visible || !app.input.is_empty() {
                app.completion.update(&app.input, &app.modules);
            }
            InputResult::Continue(None)
        }

        // Delete - Delete character
        KeyCode::Delete => {
            // For now, same as backspace
            app.input.pop();
            if app.completion.visible || !app.input.is_empty() {
                app.completion.update(&app.input, &app.modules);
            }
            InputResult::Continue(None)
        }

        // Regular character input
        KeyCode::Char(c) => {
            app.input.push(c);
            // Auto-update completion
            app.completion.update(&app.input, &app.modules);
            InputResult::Continue(None)
        }

        _ => InputResult::Continue(None),
    }
}

/// Parse a command from user input
fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    if parts.is_empty() {
        return None;
    }

    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| s.to_string());

    match cmd.as_str() {
        "add" => args.map(Command::Add),
        "remove" => args.map(Command::Remove),
        "connect" => args.map(Command::Connect),
        "disconnect" => args.map(Command::Disconnect),
        "list" => Some(Command::List(args)),
        "show" => args.map(Command::Show),
        "help" => Some(Command::Help(args)),
        "refresh" => Some(Command::Refresh),
        "quit" | "exit" => Some(Command::Quit),
        _ => None,
    }
}
