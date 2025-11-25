//! TUI rendering with ratatui

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode};

/// Theme colors
pub struct Theme {
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,
    pub text: Color,
    pub text_dim: Color,
    pub highlight: Color,
    pub highlight_bg: Color,
    pub error: Color,
    pub success: Color,
    pub status_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            title: Color::White,
            text: Color::Gray,
            text_dim: Color::DarkGray,
            highlight: Color::Cyan,
            highlight_bg: Color::DarkGray,
            error: Color::Red,
            success: Color::Green,
            status_bg: Color::DarkGray,
        }
    }
}

/// Render the entire application
pub fn render(frame: &mut Frame, app: &App) {
    let theme = Theme::default();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Render header
    render_header(frame, chunks[0], app, &theme);

    // Content area - split into modules and cables panels
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Render panels based on mode
    match app.mode {
        AppMode::Normal | AppMode::Input => {
            render_modules_panel(frame, content_chunks[0], app, &theme);
            render_cables_panel(frame, content_chunks[1], app, &theme);
        }
        AppMode::Help => {
            render_help(frame, chunks[1], &theme);
        }
        AppMode::ModuleDetail => {
            render_module_detail(frame, chunks[1], app, &theme);
        }
    }

    // Render input area
    render_input(frame, chunks[2], app, &theme);

    // Render status bar
    render_status_bar(frame, chunks[3], app, &theme);

    // Render completion popup if visible
    if app.completion.visible && !app.completion.suggestions.is_empty() {
        render_completion_popup(frame, chunks[2], app, &theme);
    }
}

/// Render the header bar
fn render_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let connection_status = if app.connected {
        Span::styled(" Connected ", Style::default().fg(theme.success))
    } else {
        Span::styled(" Disconnected ", Style::default().fg(theme.error))
    };

    let header = Line::from(vec![
        Span::styled(
            " VCV Rack Terminal Client ",
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("                    "),
        connection_status,
        Span::styled(
            format!(" {} ", app.server_url),
            Style::default().fg(theme.text_dim),
        ),
    ]);

    let header_widget = Paragraph::new(header).style(Style::default().bg(theme.status_bg));
    frame.render_widget(header_widget, area);
}

/// Render the modules panel
fn render_modules_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let modules = app.modules.all_modules();

    let items: Vec<ListItem> = modules
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if Some(i) == app.selected_module {
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let line = Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), Style::default().fg(theme.text_dim)),
                Span::styled(&m.friendly_name, style),
                Span::styled(
                    format!(" ({})", m.full_name),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(
                    format!(" [in:{} out:{}]", m.inputs.len(), m.outputs.len()),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = format!(" MODULES IN PATCH ({}) ", modules.len());
    let modules_list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(modules_list, area);
}

/// Render the cables panel
fn render_cables_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let cables = app.cables.all_cables();

    let items: Vec<ListItem> = cables
        .iter()
        .map(|c| {
            let line = Line::from(vec![
                Span::styled(&c.source_module, Style::default().fg(theme.highlight)),
                Span::raw(":"),
                Span::styled(&c.source_port, Style::default().fg(theme.text)),
                Span::styled(" -> ", Style::default().fg(theme.text_dim)),
                Span::styled(&c.target_module, Style::default().fg(theme.highlight)),
                Span::raw(":"),
                Span::styled(&c.target_port, Style::default().fg(theme.text)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!(" CABLES ({}) ", cables.len());
    let cables_list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(cables_list, area);
}

/// Render the input area
fn render_input(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let input_style = Style::default().fg(theme.text);

    // Show message if present, otherwise show input prompt
    let content = if let Some(ref msg) = app.message {
        let color = if app.message_is_error {
            theme.error
        } else {
            theme.success
        };
        Line::from(vec![Span::styled(msg, Style::default().fg(color))])
    } else {
        Line::from(vec![
            Span::styled("> ", Style::default().fg(theme.highlight)),
            Span::styled(&app.input, input_style),
            Span::styled("_", Style::default().fg(theme.highlight)),
        ])
    };

    let input_widget = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if app.mode == AppMode::Input {
                theme.border_focused
            } else {
                theme.border
            })),
    );

    frame.render_widget(input_widget, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let modules_count = app.modules.count();
    let cables_count = app.cables.count();

    let status = Line::from(vec![
        Span::styled(
            format!(" {} modules ", modules_count),
            Style::default().fg(theme.text),
        ),
        Span::styled("|", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!(" {} cables ", cables_count),
            Style::default().fg(theme.text),
        ),
        Span::styled("|", Style::default().fg(theme.text_dim)),
        Span::styled(" ? help ", Style::default().fg(theme.text_dim)),
        Span::styled("|", Style::default().fg(theme.text_dim)),
        Span::styled(" Tab complete ", Style::default().fg(theme.text_dim)),
    ]);

    let status_widget = Paragraph::new(status).style(Style::default().bg(theme.status_bg));
    frame.render_widget(status_widget, area);
}

/// Render the completion popup
fn render_completion_popup(frame: &mut Frame, input_area: Rect, app: &App, theme: &Theme) {
    let suggestions = &app.completion.suggestions;
    let selected = app.completion.selected;

    // Calculate popup position and size
    let popup_height = (suggestions.len() + 2).min(12) as u16;
    let popup_width = suggestions
        .iter()
        .map(|s| s.display.len() + s.description.as_ref().map_or(0, |d| d.len() + 3))
        .max()
        .unwrap_or(20)
        .min(60) as u16
        + 4;

    // Position above the input area
    let popup_area = Rect {
        x: input_area.x + 2,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width.min(frame.area().width - 4),
        height: popup_height,
    };

    // Clear the area first
    frame.render_widget(Clear, popup_area);

    // Render suggestions
    let items: Vec<ListItem> = suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.highlight)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let mut spans = vec![Span::styled(&s.display, style)];
            if let Some(ref desc) = s.description {
                spans.push(Span::styled(
                    format!(" - {}", desc),
                    Style::default().fg(theme.text_dim),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let popup = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_focused))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(popup, popup_area);
}

/// Render help screen
fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "VCV Rack Terminal Client - Help",
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Commands:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  add <module>              Add a module to the patch"),
        Line::from("  remove <module>           Remove a module from the patch"),
        Line::from("  connect <src:port> <dst:port>   Create a cable"),
        Line::from("  disconnect <src:port> <dst:port> Remove a cable"),
        Line::from("  list [modules|cables]     List modules or cables"),
        Line::from("  show <module>             Show module details"),
        Line::from("  help [command]            Show help"),
        Line::from("  refresh                   Refresh from server"),
        Line::from("  quit / exit               Exit the application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?            Show this help"),
        Line::from("  Tab          Complete / Cycle completions"),
        Line::from("  Up/Down      Navigate completions or history"),
        Line::from("  Enter        Execute command or select completion"),
        Line::from("  Esc          Close popup / Clear input"),
        Line::from("  Ctrl+C       Exit"),
        Line::from("  Ctrl+R       Refresh from server"),
        Line::from("  Ctrl+U       Clear input line"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Module Naming:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Modules get auto-assigned names like VCO-1-1, VCO-1-2, etc."),
        Line::from("  Use these names in commands instead of numeric IDs."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Examples:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  add VCV VCO-1"),
        Line::from("  connect VCO-1-1:Sine VCF-1:In"),
        Line::from("  show VCO-1-1"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to return...",
            Style::default().fg(theme.text_dim),
        )]),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help_widget, area);
}

/// Render module detail view
fn render_module_detail(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(ref module) = app.detail_module else {
        return;
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Plugin: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&module.plugin_slug, Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Model:  ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&module.model_name, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
    ];

    // Inputs section
    lines.push(Line::from(vec![Span::styled(
        "INPUTS",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "------",
        Style::default().fg(theme.text_dim),
    )]));

    for input in &module.inputs {
        let connected = if input.connected {
            Span::styled(" (connected)", Style::default().fg(theme.success))
        } else {
            Span::raw("")
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", input.id),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(&input.name, Style::default().fg(theme.text)),
            connected,
        ]));
    }

    lines.push(Line::from(""));

    // Outputs section
    lines.push(Line::from(vec![Span::styled(
        "OUTPUTS",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "-------",
        Style::default().fg(theme.text_dim),
    )]));

    for output in &module.outputs {
        let connected = if output.connected {
            Span::styled(" (connected)", Style::default().fg(theme.success))
        } else {
            Span::raw("")
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", output.id),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(&output.name, Style::default().fg(theme.text)),
            connected,
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Press Esc to return...",
        Style::default().fg(theme.text_dim),
    )]));

    let title = format!(" {} ", module.friendly_name);
    let detail_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(detail_widget, area);
}
