//! StreamTUI - Cyberpunk TUI for streaming to Chromecast
//!
//! A neon-soaked terminal interface for searching content, selecting quality,
//! and casting to your TV. Simple. Fast. Beautiful.
//!
//! # Usage
//!
//! ```bash
//! # Launch interactive TUI
//! streamtui
//!
//! # CLI mode (for automation)
//! streamtui search "blade runner"
//! streamtui cast tt1856101 --device "Living Room TV"
//! streamtui status --json
//! ```

// Allow dead code for TUI components and models prepared for future interactive mode
#![allow(dead_code)]

mod app;
mod cli;
mod commands;
mod config;
mod models;

mod api;
mod stream;
mod ui;

use std::io::{stdout, Stdout};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::app::{App, AppState, InputMode};
use crate::cli::{Cli, Command, ExitCode, Output};
use crate::ui::Theme;

/// Terminal type alias for convenience
type Tui = Terminal<CrosstermBackend<Stdout>>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.is_cli_mode() {
        // CLI mode: execute command and exit
        let exit_code = run_cli(cli).await;
        std::process::exit(exit_code.into());
    } else {
        // TUI mode: launch interactive interface
        run_tui().await
    }
}

/// Run CLI command and return exit code
async fn run_cli(cli: Cli) -> ExitCode {
    let output = Output::new(&cli);
    let device = cli.device.as_deref();

    match cli.command {
        Some(Command::Search(cmd)) => commands::search_cmd(cmd, &output).await,

        Some(Command::Trending(cmd)) => commands::trending_cmd(cmd, &output).await,

        Some(Command::Info(cmd)) => commands::info_cmd(cmd, &output).await,

        Some(Command::Streams(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::streams_cmd(cmd, &output).await
        }

        Some(Command::Subtitles(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::subtitles_cmd(cmd, &output).await
        }

        Some(Command::Devices(cmd)) => commands::devices_cmd(cmd, &output).await,

        Some(Command::Cast(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::cast_cmd(cmd, device, &output).await
        }

        Some(Command::CastMagnet(cmd)) => commands::cast_magnet_cmd(cmd, device, &output).await,

        Some(Command::PlayLocal(cmd)) => commands::play_local_cmd(cmd, &output).await,

        Some(Command::Status(cmd)) => commands::status_cmd(cmd, device, &output).await,

        Some(Command::Play(cmd)) => commands::play_cmd(cmd, device, &output).await,

        Some(Command::Pause(cmd)) => commands::pause_cmd(cmd, device, &output).await,

        Some(Command::Stop(cmd)) => commands::stop_cmd(cmd, device, &output).await,

        Some(Command::Seek(cmd)) => commands::seek_cmd(cmd, device, &output).await,

        Some(Command::Volume(cmd)) => commands::volume_cmd(cmd, device, &output).await,

        None => {
            // This shouldn't happen (handled by is_cli_mode check)
            ExitCode::Success
        }
    }
}

// =============================================================================
// TUI Mode
// =============================================================================

/// Initialize the terminal for TUI mode
fn init_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal state
fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run interactive TUI
async fn run_tui() -> Result<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create app state
    let mut app = App::new();

    // Run the main event loop
    let result = run_event_loop(&mut terminal, &mut app).await;

    // Always restore terminal, even on error
    restore_terminal(&mut terminal)?;

    result
}

/// Main event loop - handles input, updates state, renders UI
async fn run_event_loop(terminal: &mut Tui, app: &mut App) -> Result<()> {
    const TICK_RATE: Duration = Duration::from_millis(100);

    while app.running {
        // Render current state
        terminal.draw(|frame| render_ui(frame, app))?;

        // Poll for events with timeout (allows for async updates later)
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (ignore releases on Windows)
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        // Future: Handle async operations (search, fetch streams, etc.)
        // This is where we'd check channels for completed async tasks in TUI mode
    }

    Ok(())
}

// =============================================================================
// UI Rendering
// =============================================================================

/// Main render function - dispatches to view-specific renderers
fn render_ui(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Clear with background color
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(ratatui::style::Style::default().bg(Theme::BACKGROUND)),
        area,
    );

    // Main layout: header, content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Render components
    render_header(frame, chunks[0], app);
    render_content(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);

    // Render error overlay if present
    if let Some(ref error) = app.error {
        render_error_popup(frame, area, error);
    }
}

/// Render the header with title and search box
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Logo
            Constraint::Min(1),     // Search box
        ])
        .split(area);

    // Logo
    let logo = Paragraph::new(Line::from(vec![
        Span::styled(
            "STREAM",
            ratatui::style::Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "TUI",
            ratatui::style::Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(ratatui::style::Style::default().fg(Theme::BORDER)),
    );
    frame.render_widget(logo, header_chunks[0]);

    // Search box
    let search_style = if app.input_mode == InputMode::Editing {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let search_text = if app.input_mode == InputMode::Editing {
        let query = &app.search.query;
        let cursor = app.search.cursor.min(query.len());
        let (before, after) = query.split_at(cursor);
        format!("⌕ {}│{}", before, after)
    } else if app.search.query.is_empty() {
        "⌕ Type / to search...".to_string()
    } else {
        format!("⌕ {}", app.search.query)
    };

    let search_box = Paragraph::new(search_text)
        .style(if app.input_mode == InputMode::Editing {
            Theme::input().fg(Theme::PRIMARY)
        } else {
            Theme::input()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(search_style)
                .title(Span::styled(" SEARCH ", Theme::title())),
        );
    frame.render_widget(search_box, header_chunks[1]);
}

/// Render the main content area based on current state
fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.state {
        AppState::Home => render_home(frame, area, app),
        AppState::Search => render_search_results(frame, area, app),
        AppState::Detail => render_detail(frame, area, app),
        AppState::Sources => render_sources(frame, area, app),
        AppState::Subtitles => render_subtitles(frame, area, app),
        AppState::Playing => render_playing(frame, area, app),
    }
}

/// Render home screen with trending content
fn render_home(frame: &mut Frame, area: Rect, _app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(" ⚡ TRENDING ", Theme::title()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Placeholder for trending content
    let help = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Welcome to ", Theme::text()),
            Span::styled(
                "StreamTUI",
                ratatui::style::Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Your neon-soaked streaming companion",
            Theme::dimmed(),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Quick Start:", Theme::accent())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  /  ", Theme::keybind()),
            Span::styled("Search for movies & shows", Theme::dimmed()),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓ ", Theme::keybind()),
            Span::styled("Navigate lists", Theme::dimmed()),
        ]),
        Line::from(vec![
            Span::styled("  ↵  ", Theme::keybind()),
            Span::styled("Select item", Theme::dimmed()),
        ]),
        Line::from(vec![
            Span::styled("  q  ", Theme::keybind()),
            Span::styled("Quit", Theme::dimmed()),
        ]),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(help, inner);
}

/// Render search results
fn render_search_results(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(
            format!(" RESULTS ({}) ", app.search.results.len()),
            Theme::title(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.search.loading.is_loading() {
        let loading = Paragraph::new("⟳ Searching...")
            .style(Theme::loading())
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner);
        return;
    }

    if app.search.results.is_empty() {
        let empty = Paragraph::new(if app.search.query.is_empty() {
            "Type to search for movies and TV shows..."
        } else {
            "No results found"
        })
        .style(Theme::dimmed())
        .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Build result list
    let items: Vec<ListItem> = app
        .search
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == app.search.list.selected;
            let marker = if is_selected { "▸ " } else { "  " };
            let year_str = result.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            let type_str = match result.media_type {
                crate::models::MediaType::Movie => "MOVIE",
                crate::models::MediaType::Tv => "TV",
            };

            let line = Line::from(vec![
                Span::styled(
                    marker,
                    if is_selected {
                        Theme::accent()
                    } else {
                        Theme::dimmed()
                    },
                ),
                Span::styled(
                    &result.title,
                    if is_selected {
                        Theme::highlighted()
                    } else {
                        Theme::text()
                    },
                ),
                Span::styled(year_str, Theme::year()),
                Span::raw(" "),
                Span::styled(format!("[{}]", type_str), Theme::secondary()),
                Span::raw(" "),
                Span::styled(
                    format!("★ {:.1}", result.vote_average),
                    if result.vote_average >= 7.0 {
                        Theme::success()
                    } else if result.vote_average >= 5.0 {
                        Theme::warning()
                    } else {
                        Theme::dimmed()
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(Theme::text());
    frame.render_widget(list, inner);
}

/// Render detail view (movie or TV show)
fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let title = app.detail.as_ref().map(|d| d.title()).unwrap_or("DETAIL");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(format!(" {} ", title), Theme::title()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("Detail view", Theme::text())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  c  ", Theme::keybind()),
            Span::styled("View sources", Theme::dimmed()),
        ]),
        Line::from(vec![
            Span::styled("  u  ", Theme::keybind()),
            Span::styled("Select subtitles", Theme::dimmed()),
        ]),
        Line::from(vec![
            Span::styled(" ESC ", Theme::keybind()),
            Span::styled("Go back", Theme::dimmed()),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(content, inner);
}

/// Render sources view
fn render_sources(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(
            format!(" SOURCES ({}) ", app.sources.sources.len()),
            Theme::title(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.sources.loading.is_loading() {
        let loading = Paragraph::new("⟳ Fetching sources...")
            .style(Theme::loading())
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner);
        return;
    }

    if app.sources.sources.is_empty() {
        let empty = Paragraph::new("No sources available")
            .style(Theme::dimmed())
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Build source list
    let items: Vec<ListItem> = app
        .sources
        .sources
        .iter()
        .enumerate()
        .map(|(i, source)| {
            let is_selected = i == app.sources.list.selected;
            let marker = if is_selected { "▸ " } else { "  " };

            let quality_style = match source.quality {
                crate::models::Quality::UHD4K => Theme::quality_4k(),
                crate::models::Quality::FHD1080p => Theme::quality_1080p(),
                crate::models::Quality::HD720p => Theme::quality_720p(),
                _ => Theme::quality_sd(),
            };

            let seeds_style = if source.seeds >= 100 {
                Theme::seeds_high()
            } else if source.seeds >= 10 {
                Theme::seeds_medium()
            } else {
                Theme::seeds_low()
            };

            let line = Line::from(vec![
                Span::styled(
                    marker,
                    if is_selected {
                        Theme::accent()
                    } else {
                        Theme::dimmed()
                    },
                ),
                Span::styled(format!("{:6}", source.quality), quality_style),
                Span::raw(" "),
                Span::styled(
                    &source.title,
                    if is_selected {
                        Theme::highlighted()
                    } else {
                        Theme::text()
                    },
                ),
                Span::raw(" "),
                Span::styled(source.format_size(), Theme::file_size()),
                Span::raw(" "),
                Span::styled(format!("👤 {}", source.seeds), seeds_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(Theme::text());
    frame.render_widget(list, inner);
}

/// Render subtitles view
fn render_subtitles(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(
            format!(" SUBTITLES ({}) ", app.subtitles.subtitles.len()),
            Theme::title(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.subtitles.loading.is_loading() {
        let loading = Paragraph::new("⟳ Fetching subtitles...")
            .style(Theme::loading())
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner);
        return;
    }

    if app.subtitles.subtitles.is_empty() {
        let empty = Paragraph::new("No subtitles available")
            .style(Theme::dimmed())
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Build subtitle list
    let items: Vec<ListItem> = app
        .subtitles
        .subtitles
        .iter()
        .enumerate()
        .map(|(i, sub)| {
            let is_selected = i == app.subtitles.list.selected;
            let marker = if is_selected { "▸ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(
                    marker,
                    if is_selected {
                        Theme::accent()
                    } else {
                        Theme::dimmed()
                    },
                ),
                Span::styled(
                    &sub.language,
                    if is_selected {
                        Theme::highlighted()
                    } else {
                        Theme::text()
                    },
                ),
                Span::raw(" "),
                Span::styled(&sub.release, Theme::dimmed()),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(Theme::text());
    frame.render_widget(list, inner);
}

/// Render now playing view
fn render_playing(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(Span::styled(" ▶ NOW PLAYING ", Theme::success()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let playback = app.playing.playback.as_ref();

    let content: Vec<Line> = if let Some(status) = playback {
        let pos = status.position.as_secs();
        let dur = status.duration.as_secs();
        let progress = if dur > 0 {
            pos as f64 / dur as f64
        } else {
            0.0
        };
        let filled = (progress * 40.0) as usize;
        let empty = 40 - filled;

        vec![
            Line::from(""),
            Line::from(Span::styled(
                app.playing.title.clone(),
                ratatui::style::Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("{}{}", "█".repeat(filled), "░".repeat(empty))),
            Line::from(Span::styled(
                format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    pos / 60,
                    pos % 60,
                    dur / 60,
                    dur % 60
                ),
                Theme::dimmed(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Volume: {:.0}%", status.volume * 100.0),
                Theme::text(),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(" SPACE ", Theme::keybind()),
                Span::styled("Play/Pause  ", Theme::dimmed()),
                Span::styled(" ←→ ", Theme::keybind()),
                Span::styled("Seek  ", Theme::dimmed()),
                Span::styled(" ↑↓ ", Theme::keybind()),
                Span::styled("Volume", Theme::dimmed()),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled("Connecting...", Theme::loading())),
        ]
    };

    let para = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(para, inner);
}

/// Render status bar at bottom
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            ratatui::style::Style::default()
                .fg(Theme::BACKGROUND)
                .bg(Theme::PRIMARY),
        ),
        InputMode::Editing => Span::styled(
            " INSERT ",
            ratatui::style::Style::default()
                .fg(Theme::BACKGROUND)
                .bg(Theme::ACCENT),
        ),
    };

    let state_indicator = Span::styled(
        format!(" {} ", format!("{:?}", app.state).to_uppercase()),
        ratatui::style::Style::default().fg(Theme::DIM),
    );

    let device_indicator = if let Some(device) = app.selected_cast_device() {
        Span::styled(format!(" 📺 {} ", device.name), Theme::cast_target())
    } else {
        Span::styled(" No device ", Theme::dimmed())
    };

    let help = Span::styled(" q:quit  /:search  ESC:back ", Theme::dimmed());

    let status_line = Line::from(vec![
        mode_indicator,
        state_indicator,
        Span::raw(" "),
        device_indicator,
        Span::raw(" │ "),
        help,
    ]);

    let status = Paragraph::new(status_line).style(Theme::status_bar());
    frame.render_widget(status, area);
}

/// Render error popup overlay
fn render_error_popup(frame: &mut Frame, area: Rect, error: &str) {
    // Calculate centered popup
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 5;

    let popup_area = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let error_block = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(error, Theme::error())),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Theme::error())
            .title(Span::styled(" ✗ ERROR ", Theme::error()))
            .style(ratatui::style::Style::default().bg(Theme::BACKGROUND)),
    );

    frame.render_widget(error_block, popup_area);
}
