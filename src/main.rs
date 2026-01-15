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
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use tokio::sync::mpsc;

use crate::api::{TmdbClient, TorrentioClient};
use crate::stream::SubtitleClient;
use crate::app::{App, AppCommand, AppMessage, AppState, DetailState, InputMode, ListState, LoadingState, TvFocus};
use crate::cli::{Cli, Command, ExitCode, Output};
use crate::config::Config;
use crate::models::{CastDevice, CastState, Episode, TorrentState};
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

    // Create app state with command channel
    let (mut app, cmd_rx) = App::with_channels();

    // Load config and apply saved settings
    let config = Config::load();
    if let Some(ref lang) = config.default_subtitle_lang {
        app.default_subtitle_lang = lang.clone();
    }
    // Store default device name for later matching when devices are discovered
    app.default_device_name = config.default_device.clone();

    // Create message channel for async results
    let (msg_tx, msg_rx) = mpsc::unbounded_channel();

    // Spawn the async task handler
    let task_handle = tokio::spawn(handle_async_commands(cmd_rx, msg_tx.clone()));

    // Trigger initial trending fetch
    app.home.loading = LoadingState::Loading(Some("Loading trending...".into()));
    app.send_command(AppCommand::FetchTrending);

    // Discover devices at startup (for auto-selecting saved default)
    app.send_command(AppCommand::DiscoverDevices);

    // Run the main event loop
    let result = run_event_loop(&mut terminal, &mut app, msg_rx).await;

    // Clean up
    drop(app); // Drop app to close cmd_tx, which will end the task handler
    let _ = task_handle.await;

    // Kill any orphaned webtorrent processes
    cleanup_torrent_processes();

    // Always restore terminal, even on error
    restore_terminal(&mut terminal)?;

    result
}

/// Handle async commands from the UI
async fn handle_async_commands(
    mut cmd_rx: mpsc::UnboundedReceiver<AppCommand>,
    msg_tx: mpsc::UnboundedSender<AppMessage>,
) {
    let mut config = Config::load();

    while let Some(cmd) = cmd_rx.recv().await {
        let msg_tx = msg_tx.clone();
        let api_key = config.get_tmdb_api_key();

        // Spawn each command as a separate task for concurrency
        tokio::spawn(async move {
            let client = TmdbClient::new(api_key);
            let result = match cmd {
                AppCommand::FetchTrending => {
                    match client.trending().await {
                        Ok(results) => AppMessage::TrendingLoaded(results),
                        Err(e) => AppMessage::Error(format!("Failed to fetch trending: {}", e)),
                    }
                }
                AppCommand::Search(query) => {
                    match client.search(&query).await {
                        Ok(results) => AppMessage::SearchResults(results),
                        Err(e) => AppMessage::Error(format!("Search failed: {}", e)),
                    }
                }
                AppCommand::FetchMovieDetail(id) => {
                    match client.movie_detail(id).await {
                        Ok(detail) => AppMessage::MovieDetailLoaded(detail),
                        Err(e) => AppMessage::Error(format!("Failed to fetch movie: {}", e)),
                    }
                }
                AppCommand::FetchTvDetail(id) => {
                    match client.tv_detail(id).await {
                        Ok(detail) => AppMessage::TvDetailLoaded(detail),
                        Err(e) => AppMessage::Error(format!("Failed to fetch TV show: {}", e)),
                    }
                }
                AppCommand::FetchEpisodes { tv_id, season } => {
                    match client.tv_season(tv_id, season).await {
                        Ok(episodes) => AppMessage::EpisodesLoaded { season, episodes },
                        Err(e) => AppMessage::Error(format!("Failed to fetch episodes: {}", e)),
                    }
                }
                AppCommand::FetchStreams { imdb_id, season, episode } => {
                    let torrentio = TorrentioClient::new();
                    let result = if let (Some(s), Some(e)) = (season, episode) {
                        // TV episode
                        torrentio.episode_streams(&imdb_id, s as u16, e as u16).await
                    } else {
                        // Movie
                        torrentio.movie_streams(&imdb_id).await
                    };
                    match result {
                        Ok(streams) => AppMessage::StreamsLoaded(streams),
                        Err(e) => AppMessage::Error(format!("Failed to fetch streams: {}", e)),
                    }
                }
                AppCommand::FetchSubtitles { imdb_id, season, episode, lang } => {
                    // Stremio client is free - no API key needed
                    let subtitle_client = SubtitleClient::new();
                    let lang_opt = if lang.is_empty() { None } else { Some(lang.as_str()) };
                    // Use episode-specific search for TV, movie search otherwise
                    let result = match (season, episode) {
                        (Some(s), Some(e)) => subtitle_client.search_episode(&imdb_id, s, e, lang_opt).await,
                        _ => subtitle_client.search(&imdb_id, lang_opt).await,
                    };
                    match result {
                        Ok(subs) => AppMessage::SubtitlesLoaded(subs),
                        Err(e) => AppMessage::Error(format!("Failed to fetch subtitles: {}", e)),
                    }
                }
                AppCommand::DiscoverDevices => {
                    // Discover Chromecast devices using catt scan
                    match discover_cast_devices().await {
                        Ok(devices) => AppMessage::DevicesLoaded(devices),
                        Err(e) => AppMessage::Error(format!("Device discovery failed: {}", e)),
                    }
                }
                AppCommand::StartPlayback { magnet, title, device, subtitle_url, file_idx } => {
                    // Start webtorrent + cast flow
                    match start_playback(&magnet, &title, &device, subtitle_url.as_deref(), file_idx).await {
                        Ok(stream_url) => AppMessage::PlaybackStarted { stream_url },
                        Err(e) => AppMessage::Error(format!("Playback failed: {}", e)),
                    }
                }
                AppCommand::StopPlayback => {
                    // Stop webtorrent and cast
                    let _ = stop_playback().await;
                    AppMessage::PlaybackStopped
                }
                AppCommand::RestartWithSubtitles { magnet, title, device, subtitle_url, seek_seconds, file_idx } => {
                    // Restart playback with subtitles at saved position
                    match restart_with_subtitles(&magnet, &title, &device, &subtitle_url, seek_seconds, file_idx).await {
                        Ok(msg) => AppMessage::PlaybackStarted { stream_url: msg },
                        Err(e) => AppMessage::Error(format!("Restart failed: {}", e)),
                    }
                }
                AppCommand::PlaybackControl { action, device } => {
                    // Send control command to catt
                    let _ = playback_control(&action, &device).await;
                    // No message needed - fire and forget
                    return;
                }
                AppCommand::SaveSettings { subtitle_lang, device_name } => {
                    // Save settings to config file
                    let mut cfg = Config::load();
                    cfg.default_subtitle_lang = Some(subtitle_lang);
                    cfg.default_device = device_name;
                    let _ = cfg.save();
                    return;
                }
            };
            let _ = msg_tx.send(result);
        });
    }
}

/// Main event loop - handles input, updates state, renders UI
async fn run_event_loop(
    terminal: &mut Tui,
    app: &mut App,
    mut msg_rx: mpsc::UnboundedReceiver<AppMessage>,
) -> Result<()> {
    const TICK_RATE: Duration = Duration::from_millis(50);

    while app.running {
        // Render current state
        terminal.draw(|frame| render_ui(frame, app))?;

        // Check for async messages (non-blocking)
        while let Ok(msg) = msg_rx.try_recv() {
            app.handle_message(msg);
        }

        // Poll for keyboard events with timeout
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (ignore releases on Windows)
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
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

    // Render device selection modal if open
    if app.show_device_modal {
        render_device_modal(frame, area, app);
    }

    // Render settings modal if open
    if app.show_settings_modal {
        render_settings_modal(frame, area, app);
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
fn render_home(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(
            format!(" ⚡ TRENDING ({}) ", app.home.results.len()),
            Theme::title(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show loading state
    if app.home.loading.is_loading() {
        let loading = Paragraph::new("⟳ Loading trending content...")
            .style(Theme::loading())
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner);
        return;
    }

    // Show empty state with help
    if app.home.results.is_empty() {
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
            Line::from(Span::styled("Quick Start:", Theme::accent())),
            Line::from(""),
            Line::from(vec![
                Span::styled("  /  ", Theme::keybind()),
                Span::styled("Search for movies & shows", Theme::dimmed()),
            ]),
            Line::from(vec![
                Span::styled("  q  ", Theme::keybind()),
                Span::styled("Quit", Theme::dimmed()),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(help, inner);
        return;
    }

    // Show trending results list
    let items: Vec<ListItem> = app
        .home
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == app.home.list.selected;
            let marker = if is_selected { "▸ " } else { "  " };
            let year_str = result.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            let type_str = match result.media_type {
                crate::models::MediaType::Movie => "MOVIE",
                crate::models::MediaType::Tv => "TV",
            };

            let line = Line::from(vec![
                Span::styled(
                    marker,
                    if is_selected { Theme::accent() } else { Theme::dimmed() },
                ),
                Span::styled(
                    &result.title,
                    if is_selected { Theme::highlighted() } else { Theme::text() },
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
    let Some(detail) = &app.detail else {
        // No detail loaded yet
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border())
            .title(Span::styled(" DETAIL ", Theme::title()));
        frame.render_widget(block, area);
        return;
    };

    match detail {
        DetailState::Movie { detail, .. } => render_movie_detail(frame, area, detail),
        DetailState::Tv { detail, season_list, episode_list, episodes, selected_season, focus, .. } => {
            render_tv_detail(frame, area, detail, season_list, episode_list, episodes, *selected_season, *focus);
        }
    }
}

/// Render movie detail view
fn render_movie_detail(frame: &mut Frame, area: Rect, movie: &crate::models::MovieDetail) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(format!(" {} ", movie.title), Theme::title()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Format runtime
    let hours = movie.runtime / 60;
    let mins = movie.runtime % 60;
    let runtime_str = if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };

    // Format genres
    let genres_str = movie.genres.join(" · ");

    // Rating color based on score
    let rating_style = if movie.vote_average >= 7.0 {
        Theme::success()
    } else if movie.vote_average >= 5.0 {
        Theme::warning()
    } else {
        Theme::dimmed()
    };

    // Build content lines
    let mut lines = vec![
        // Title line with year and rating
        Line::from(vec![
            Span::styled(format!("{} ", movie.title), Theme::highlighted()),
            Span::styled(format!("({}) ", movie.year), Theme::year()),
            Span::styled(format!("★ {:.1}", movie.vote_average), rating_style),
        ]),
        Line::from(""),
        // Runtime and genres
        Line::from(vec![
            Span::styled(runtime_str, Theme::accent()),
            Span::styled(" │ ", Theme::dimmed()),
            Span::styled(genres_str, Theme::secondary()),
        ]),
        Line::from(""),
    ];

    // Add overview with word wrapping
    let overview_width = inner.width.saturating_sub(4) as usize;
    if !movie.overview.is_empty() {
        for line in wrap_text(&movie.overview, overview_width) {
            lines.push(Line::from(Span::styled(line, Theme::text())));
        }
    } else {
        lines.push(Line::from(Span::styled("No overview available.", Theme::dimmed())));
    }

    // Add spacing before keybinds
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Keybind hints
    lines.push(Line::from(vec![
        Span::styled("  c  ", Theme::keybind()),
        Span::styled("View sources    ", Theme::dimmed()),
        Span::styled("  u  ", Theme::keybind()),
        Span::styled("Subtitles    ", Theme::dimmed()),
        Span::styled(" ESC ", Theme::keybind()),
        Span::styled("Back", Theme::dimmed()),
    ]));

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}

/// Render TV show detail view with season/episode selection
fn render_tv_detail(
    frame: &mut Frame,
    area: Rect,
    tv: &crate::models::TvDetail,
    season_list: &ListState,
    episode_list: &ListState,
    episodes: &[Episode],
    selected_season: u8,
    focus: TvFocus,
) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(format!(" {} ", tv.name), Theme::title()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into header (info) and body (seasons/episodes)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(5), Constraint::Length(2)])
        .split(inner);

    // Header: Title, year, rating, genres
    let rating_style = if tv.vote_average >= 7.0 {
        Theme::success()
    } else if tv.vote_average >= 5.0 {
        Theme::warning()
    } else {
        Theme::dimmed()
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled(format!("{} ", tv.name), Theme::highlighted()),
            Span::styled(format!("({}) ", tv.year), Theme::year()),
            Span::styled(format!("★ {:.1}", tv.vote_average), rating_style),
        ]),
        Line::from(Span::styled(tv.genres.join(" · "), Theme::secondary())),
    ];
    let header = Paragraph::new(header_lines);
    frame.render_widget(header, chunks[0]);

    // Body: Split into seasons (left) and episodes (right)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(30)])
        .split(chunks[1]);

    // Seasons list
    let season_items: Vec<ListItem> = tv
        .seasons
        .iter()
        .map(|s| {
            let style = if s.season_number == selected_season {
                Theme::selected()
            } else {
                Theme::text()
            };
            ListItem::new(format!("Season {} ({} ep)", s.season_number, s.episode_count))
                .style(style)
        })
        .collect();

    let seasons_focused = focus == TvFocus::Seasons;
    let seasons_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if seasons_focused { BorderType::Double } else { BorderType::Rounded })
        .border_style(if seasons_focused { Theme::accent() } else { Theme::border() })
        .title(Span::styled(" Seasons ", if seasons_focused { Theme::highlighted() } else { Theme::accent() }));

    let seasons_widget = List::new(season_items)
        .block(seasons_block)
        .highlight_style(Theme::highlighted())
        .highlight_symbol("▸ ");

    // Convert our ListState to ratatui's ListState
    let mut ratatui_season_state = ratatui::widgets::ListState::default();
    ratatui_season_state.select(Some(season_list.selected));
    frame.render_stateful_widget(seasons_widget, body_chunks[0], &mut ratatui_season_state);

    // Episodes list
    let episode_items: Vec<ListItem> = episodes
        .iter()
        .map(|ep| {
            ListItem::new(format!("E{:02} {}", ep.episode, ep.name))
                .style(Theme::text())
        })
        .collect();

    let episodes_title = if episodes.is_empty() {
        " Episodes (loading...) ".to_string()
    } else {
        format!(" Episodes ({}) ", episodes.len())
    };

    let episodes_focused = focus == TvFocus::Episodes;
    let episodes_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if episodes_focused { BorderType::Double } else { BorderType::Rounded })
        .border_style(if episodes_focused { Theme::accent() } else { Theme::border() })
        .title(Span::styled(episodes_title, if episodes_focused { Theme::highlighted() } else { Theme::accent() }));

    let episodes_widget = List::new(episode_items)
        .block(episodes_block)
        .highlight_style(Theme::highlighted())
        .highlight_symbol("▸ ");

    // Convert our ListState to ratatui's ListState
    let mut ratatui_episode_state = ratatui::widgets::ListState::default();
    ratatui_episode_state.select(Some(episode_list.selected));
    frame.render_stateful_widget(episodes_widget, body_chunks[1], &mut ratatui_episode_state);

    // Footer: Keybind hints
    let footer = Line::from(vec![
        Span::styled("Tab", Theme::keybind()),
        Span::styled(":switch  ", Theme::dimmed()),
        Span::styled("↑↓", Theme::keybind()),
        Span::styled(":select  ", Theme::dimmed()),
        Span::styled("Enter", Theme::keybind()),
        Span::styled(":sources  ", Theme::dimmed()),
        Span::styled("u", Theme::keybind()),
        Span::styled(":subtitles  ", Theme::dimmed()),
        Span::styled("ESC", Theme::keybind()),
        Span::styled(":back", Theme::dimmed()),
    ]);
    frame.render_widget(Paragraph::new(footer), chunks[2]);
}

/// Wrap text to fit within a given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
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

    // Split into list (left) and detail panel (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(inner);

    // Build compact source list for left panel
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

            // Truncate title for compact display (use chars to handle UTF-8 properly)
            let max_title_len = chunks[0].width.saturating_sub(22) as usize;
            let title_chars: Vec<char> = source.title.chars().collect();
            let truncated_title = if title_chars.len() > max_title_len {
                format!("{}…", title_chars[..max_title_len.saturating_sub(1)].iter().collect::<String>())
            } else {
                source.title.clone()
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
                    truncated_title,
                    if is_selected {
                        Theme::highlighted()
                    } else {
                        Theme::text()
                    },
                ),
                Span::raw(" "),
                Span::styled(format!("👤{:>4}", source.seeds), seeds_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(Theme::text());
    frame.render_widget(list, chunks[0]);

    // Render detail panel for selected source
    render_source_detail(frame, chunks[1], app);
}

/// Render the detail panel for the selected source
fn render_source_detail(frame: &mut Frame, area: Rect, app: &App) {
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(" DETAILS ", Theme::title()));

    let detail_inner = detail_block.inner(area);
    frame.render_widget(detail_block, area);

    let selected_source = app.sources.sources.get(app.sources.list.selected);
    let Some(source) = selected_source else {
        let empty = Paragraph::new("No source selected")
            .style(Theme::dimmed())
            .alignment(Alignment::Center);
        frame.render_widget(empty, detail_inner);
        return;
    };

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

    // Build detail lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Quality: ", Theme::dimmed()),
            Span::styled(format!("{}", source.quality), quality_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Size: ", Theme::dimmed()),
            Span::styled(source.format_size(), Theme::file_size()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Seeds: ", Theme::dimmed()),
            Span::styled(format!("{}", source.seeds), seeds_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Title:", Theme::dimmed())]),
    ];

    // Wrap the title text to fit the panel width
    let title_width = detail_inner.width.saturating_sub(2) as usize;
    let title = &source.title;
    for chunk in title.chars().collect::<Vec<_>>().chunks(title_width) {
        lines.push(Line::from(Span::styled(
            chunk.iter().collect::<String>(),
            Theme::text(),
        )));
    }

    // Add provider info if available
    if !source.name.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Provider: ", Theme::dimmed()),
            Span::styled(&source.name, Theme::text()),
        ]));
    }

    let detail_paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(detail_paragraph, detail_inner);
}

/// Render subtitles view
fn render_subtitles(frame: &mut Frame, area: Rect, app: &App) {
    let filter_display = app.subtitles.lang_filter.display();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(Span::styled(
            format!(" SUBTITLES ({}) ", app.subtitles.subtitles.len()),
            Theme::title(),
        ))
        .title_bottom(Line::from(vec![
            Span::styled(" Tab:", Theme::dimmed()),
            Span::styled(filter_display, Theme::accent()),
            Span::styled("  ↑↓:select  Enter:use  n:none  ESC:back ", Theme::dimmed()),
        ]));

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
        let empty = Paragraph::new("No subtitles available\n\nPress Tab to change language filter")
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
                    &sub.language_name,
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

    let list = List::new(items)
        .style(Theme::text())
        .highlight_style(Theme::highlighted());

    // Convert app ListState to ratatui ListState for scrolling
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.subtitles.list.selected));

    frame.render_stateful_widget(list, inner, &mut list_state);
}

/// Render now playing view - a nice centered video player interface
fn render_playing(frame: &mut Frame, area: Rect, app: &App) {
    // Main container
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Theme::accent())
        .style(ratatui::style::Style::default().bg(Theme::BACKGROUND));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: top spacer, player card, bottom controls
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // Top spacer
            Constraint::Length(16),  // Player card
            Constraint::Length(3),   // Controls
            Constraint::Min(1),      // Bottom spacer
        ])
        .split(inner);

    // Center the player card horizontally
    let card_width = 60.min(layout[1].width);
    let card_area = Rect {
        x: layout[1].x + (layout[1].width.saturating_sub(card_width)) / 2,
        y: layout[1].y,
        width: card_width,
        height: layout[1].height,
    };

    // Player card with double border
    let card_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(Span::styled(" ▶ NOW PLAYING ", Theme::success()))
        .title_alignment(Alignment::Center);

    let card_inner = card_block.inner(card_area);
    frame.render_widget(card_block, card_area);

    // Get device name for display
    let device_name = app
        .playing
        .device
        .as_ref()
        .map(|d| d.name.as_str())
        .or_else(|| {
            app.selected_device
                .and_then(|i| app.cast_devices.get(i))
                .map(|d| d.name.as_str())
        })
        .unwrap_or("Unknown");

    // Animated spinner for loading states
    let spinner_frame = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 150) % 8;
    let spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"][spinner_frame as usize];

    // Build player content based on state
    let mut lines: Vec<Line> = Vec::new();

    // Title (truncate if needed)
    let title = if app.playing.title.len() > card_inner.width as usize - 4 {
        format!("{}...", &app.playing.title[..card_inner.width as usize - 7])
    } else {
        app.playing.title.clone()
    };

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        title,
        ratatui::style::Style::default()
            .fg(Theme::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Device indicator
    lines.push(Line::from(vec![
        Span::styled("📺 ", Theme::text()),
        Span::styled(device_name, Theme::accent()),
    ]));
    lines.push(Line::from(""));

    if let Some(ref status) = app.playing.playback {
        // Active playback - show progress
        let pos = status.position.as_secs();
        let dur = status.duration.as_secs();
        let progress = if dur > 0 { pos as f64 / dur as f64 } else { 0.0 };

        // Progress bar
        let bar_width = (card_inner.width as usize).saturating_sub(8).min(40);
        let filled = (progress * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);

        let state_icon = match status.state {
            CastState::Playing => "▶",
            CastState::Paused => "⏸",
            CastState::Buffering => spinner,
            _ => "●",
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", state_icon), Theme::success()),
            Span::styled("▓".repeat(filled), Theme::accent()),
            Span::styled("░".repeat(empty), Theme::dimmed()),
        ]));

        // Time display
        lines.push(Line::from(Span::styled(
            format!(
                "{:02}:{:02} / {:02}:{:02}",
                pos / 60, pos % 60, dur / 60, dur % 60
            ),
            Theme::dimmed(),
        )));

        // Volume
        lines.push(Line::from(""));
        let vol_bars = (status.volume * 10.0) as usize;
        lines.push(Line::from(vec![
            Span::styled("🔊 ", Theme::text()),
            Span::styled("█".repeat(vol_bars), Theme::accent()),
            Span::styled("░".repeat(10 - vol_bars), Theme::dimmed()),
            Span::styled(format!(" {:.0}%", status.volume * 100.0), Theme::dimmed()),
        ]));
    } else if let Some(ref torrent) = app.playing.torrent {
        // Connecting/buffering state
        let state_text = torrent.state.to_string();

        // Visual streaming animation
        let anim_frame = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 100) % 20;

        let wave: String = (0..20)
            .map(|i| {
                let dist = ((i as i64 - anim_frame as i64).abs() % 20) as usize;
                if dist < 3 { '█' } else if dist < 5 { '▓' } else if dist < 7 { '▒' } else { '░' }
            })
            .collect();

        lines.push(Line::from(Span::styled(wave, Theme::accent())));
        lines.push(Line::from(""));

        // Status with spinner
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", spinner), Theme::accent()),
            Span::styled(state_text, Theme::loading()),
        ]));

        // Hint for 0 peers
        if torrent.state.peers() == Some(0) {
            lines.push(Line::from(Span::styled(
                "Searching DHT for peers...",
                Theme::dimmed(),
            )));
        }

        // Buffering progress
        if let TorrentState::Buffering { progress, .. } = torrent.state {
            lines.push(Line::from(""));
            let bar_width = 30;
            let filled = (progress as usize * bar_width) / 100;
            lines.push(Line::from(vec![
                Span::styled("▓".repeat(filled), Theme::success()),
                Span::styled("░".repeat(bar_width - filled), Theme::dimmed()),
                Span::styled(format!(" {}%", progress), Theme::text()),
            ]));
        }
    } else {
        // Fallback - initializing
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", spinner), Theme::accent()),
            Span::styled("Initializing...", Theme::loading()),
        ]));
    }

    let para = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(para, card_inner);

    // Bottom controls bar
    let controls_area = Rect {
        x: inner.x + (inner.width.saturating_sub(card_width)) / 2,
        y: layout[2].y,
        width: card_width,
        height: 3,
    };

    let controls = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" SPACE ", Theme::keybind()),
            Span::styled("Play/Pause ", Theme::dimmed()),
            Span::styled(" ←→ ", Theme::keybind()),
            Span::styled("Seek ", Theme::dimmed()),
            Span::styled(" ↑↓ ", Theme::keybind()),
            Span::styled("Vol ", Theme::dimmed()),
            Span::styled(" u ", Theme::keybind()),
            Span::styled("Subs ", Theme::dimmed()),
            Span::styled(" s ", Theme::keybind()),
            Span::styled("Stop", Theme::dimmed()),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(controls, controls_area);
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

    let help = Span::styled(" q:quit  /:search  d:device  o:settings  ESC:back ", Theme::dimmed());

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

/// Render device selection modal
fn render_device_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate centered popup
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = (app.cast_devices.len() as u16 + 4).min(15).max(6);

    let popup_area = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Theme::accent())
        .title(Span::styled(" 📺 SELECT DEVICE ", Theme::title()))
        .style(ratatui::style::Style::default().bg(Theme::BACKGROUND));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if app.cast_devices.is_empty() {
        let msg = Paragraph::new("Scanning for devices...")
            .style(Theme::loading())
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
    } else {
        let items: Vec<ListItem> = app
            .cast_devices
            .iter()
            .enumerate()
            .map(|(i, device)| {
                let is_selected = i == app.device_modal_index;
                let marker = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Theme::highlighted()
                } else {
                    Theme::text()
                };
                let model = device.model.as_deref().unwrap_or("Chromecast");
                ListItem::new(Line::from(vec![
                    Span::styled(marker, if is_selected { Theme::accent() } else { Theme::dimmed() }),
                    Span::styled(&device.name, style),
                    Span::styled(format!(" ({})", model), Theme::dimmed()),
                ]))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // Help text at bottom
    let help_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + popup_area.height - 2,
        width: popup_area.width - 2,
        height: 1,
    };
    let help = Paragraph::new("↑↓:select  Enter:confirm  r:refresh  Esc:close")
        .style(Theme::dimmed())
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

fn render_settings_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate centered popup
    let popup_width = 45.min(area.width.saturating_sub(4));
    let popup_height = 10;

    let popup_area = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Theme::accent())
        .title(Span::styled(" SETTINGS ", Theme::title()))
        .style(ratatui::style::Style::default().bg(Theme::BACKGROUND));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Language field
    let lang_label_style = if app.settings_field_index == 0 {
        Theme::highlighted()
    } else {
        Theme::text()
    };
    let lang_marker = if app.settings_field_index == 0 { "▸ " } else { "  " };
    let lang_value = if app.settings_field_index == 0 {
        format!("[{}]", app.settings_lang_input)
    } else {
        app.default_subtitle_lang.clone()
    };

    let lang_line = Line::from(vec![
        Span::styled(lang_marker, if app.settings_field_index == 0 { Theme::accent() } else { Theme::dimmed() }),
        Span::styled("Subtitle language: ", lang_label_style),
        Span::styled(lang_value, if app.settings_field_index == 0 { Theme::accent() } else { Theme::dimmed() }),
    ]);

    // Device field (read-only info, use 'd' to change)
    let device_label_style = if app.settings_field_index == 1 {
        Theme::highlighted()
    } else {
        Theme::text()
    };
    let device_marker = if app.settings_field_index == 1 { "▸ " } else { "  " };
    let device_name = app
        .selected_device
        .and_then(|i| app.cast_devices.get(i))
        .map(|d| d.name.as_str())
        .unwrap_or("None (press 'd' to select)");

    let device_line = Line::from(vec![
        Span::styled(device_marker, if app.settings_field_index == 1 { Theme::accent() } else { Theme::dimmed() }),
        Span::styled("Default device: ", device_label_style),
        Span::styled(device_name, Theme::dimmed()),
    ]);

    let content = Paragraph::new(vec![
        Line::from(""),
        lang_line,
        Line::from(""),
        device_line,
        Line::from(""),
    ]);
    frame.render_widget(content, inner);

    // Help text at bottom
    let help_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + popup_area.height - 2,
        width: popup_area.width - 2,
        height: 1,
    };
    let help = Paragraph::new("↑↓:navigate  Enter:save  Esc:close")
        .style(Theme::dimmed())
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

// =============================================================================
// Playback Functions
// =============================================================================

/// Discover Chromecast devices using catt scan
async fn discover_cast_devices() -> anyhow::Result<Vec<CastDevice>> {
    let output = tokio::process::Command::new("catt")
        .arg("scan")
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file") {
            anyhow::bail!("catt not found. Install with: pip install catt");
        }
        anyhow::bail!("catt scan failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = CastDevice::parse_catt_scan(&stdout);

    // Also try stderr (catt sometimes outputs there)
    if devices.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(CastDevice::parse_catt_scan(&stderr));
    }

    Ok(devices)
}

/// Kill any running webtorrent processes on TUI exit
fn cleanup_torrent_processes() {
    // Kill webtorrent processes spawned by this app
    let _ = std::process::Command::new("pkill")
        .args(["-f", "webtorrent"])
        .output();
}

/// Start playback: webtorrent with native chromecast support
async fn start_playback(
    magnet: &str,
    _title: &str,
    device: &str,
    subtitle_url: Option<&str>,
    file_idx: Option<u32>,
) -> anyhow::Result<String> {
    // Download subtitle file if URL provided
    let subtitle_path = if let Some(url) = subtitle_url {
        download_subtitle(url).await.ok()
    } else {
        None
    };

    // Use our own CLI tool for casting
    let exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get executable path: {}", e))?;
    let exe_path = exe.to_string_lossy();

    // Build command: streamtui cast-magnet <magnet> [-d <device> | --vlc] [--subtitle-file <path>] -q
    let is_vlc = device == "VLC (Local)";
    let mut args = if is_vlc {
        format!(
            "nohup '{}' cast-magnet '{}' --vlc -q",
            exe_path.replace('\'', "'\\''"),
            magnet.replace('\'', "'\\''")
        )
    } else {
        format!(
            "nohup '{}' cast-magnet '{}' -d '{}' -q",
            exe_path.replace('\'', "'\\''"),
            magnet.replace('\'', "'\\''"),
            device.replace('\'', "'\\''")
        )
    };

    if let Some(ref sub_path) = subtitle_path {
        args.push_str(&format!(" --subtitle-file '{}'", sub_path.replace('\'', "'\\''")));
    }

    // Add file index if specified (to select correct file in multi-file torrents)
    if let Some(idx) = file_idx {
        args.push_str(&format!(" -i {}", idx));
    }

    // Use a log file instead of /dev/null - webtorrent/VLC need somewhere to output
    let log_path = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("streamtui")
        .join("playback.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let log_path_str = log_path.to_string_lossy();

    args.push_str(&format!(" </dev/null >>'{}' 2>&1 &", log_path_str));

    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start cast: {}", e))?;

    std::mem::forget(child);

    let msg = if is_vlc {
        if subtitle_path.is_some() {
            "Playing in VLC (with subtitles)".to_string()
        } else {
            "Playing in VLC".to_string()
        }
    } else if subtitle_path.is_some() {
        format!("Casting to {} (with subtitles)", device)
    } else {
        format!("Casting to {}", device)
    };
    Ok(msg)
}

/// Download subtitle file to temp directory
async fn download_subtitle(url: &str) -> anyhow::Result<String> {
    use std::io::Write;

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download subtitle: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // Create temp file with .srt extension
    let temp_dir = std::env::temp_dir();
    let filename = format!("streamtui_sub_{}.srt", std::process::id());
    let path = temp_dir.join(filename);

    let mut file = std::fs::File::create(&path)?;
    file.write_all(&bytes)?;

    Ok(path.to_string_lossy().to_string())
}

/// Stop playback - kill webtorrent processes
async fn stop_playback() -> anyhow::Result<()> {
    // Kill any running webtorrent processes
    let _ = tokio::process::Command::new("pkill")
        .arg("-f")
        .arg("webtorrent")
        .output()
        .await;

    // Stop catt playback
    let _ = tokio::process::Command::new("catt")
        .arg("stop")
        .output()
        .await;

    Ok(())
}

/// Restart playback with subtitles at a specific position
async fn restart_with_subtitles(
    magnet: &str,
    _title: &str,
    device: &str,
    subtitle_url: &str,
    seek_seconds: u32,
    file_idx: Option<u32>,
) -> anyhow::Result<String> {
    // 1. Stop current playback
    stop_playback().await?;

    // Small delay to ensure clean stop
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 2. Download subtitle file
    let subtitle_path = download_subtitle(subtitle_url).await?;

    // 3. Use our own CLI tool with --start for seeking
    let exe = std::env::current_exe()?;
    let exe_path = exe.to_string_lossy();

    // Build command: streamtui cast-magnet <magnet> [-d <device> | --vlc] --subtitle-file <path> --start <pos> -q
    let is_vlc = device == "VLC (Local)";
    let mut args = if is_vlc {
        format!(
            "nohup '{}' cast-magnet '{}' --vlc --subtitle-file '{}' -q",
            exe_path.replace('\'', "'\\''"),
            magnet.replace('\'', "'\\''"),
            subtitle_path.replace('\'', "'\\''")
        )
    } else {
        format!(
            "nohup '{}' cast-magnet '{}' -d '{}' --subtitle-file '{}' -q",
            exe_path.replace('\'', "'\\''"),
            magnet.replace('\'', "'\\''"),
            device.replace('\'', "'\\''"),
            subtitle_path.replace('\'', "'\\''")
        )
    };

    if seek_seconds > 0 {
        args.push_str(&format!(" --start {}", seek_seconds));
    }

    // Add file index if specified
    if let Some(idx) = file_idx {
        args.push_str(&format!(" -i {}", idx));
    }

    // Use a log file instead of /dev/null - webtorrent/VLC need somewhere to output
    let log_path = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("streamtui")
        .join("playback.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let log_path_str = log_path.to_string_lossy();

    args.push_str(&format!(" </dev/null >>'{}' 2>&1 &", log_path_str));

    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    std::mem::forget(child);

    Ok(format!("Restarted with subtitles at {}s", seek_seconds))
}

/// Send playback control command using our own CLI
async fn playback_control(action: &str, device: &str) -> anyhow::Result<()> {
    // VLC controls from TUI not supported - users control VLC directly
    if device == "VLC (Local)" {
        return Ok(());
    }

    let exe = std::env::current_exe()?;

    // Map action to our CLI command
    let (cmd, extra_arg): (&str, Option<&str>) = match action {
        "play_toggle" | "play" => ("play", None),
        "pause" => ("pause", None),
        "stop" => ("stop", None),
        "volumeup" => ("volume", Some("+10")),
        "volumedown" => ("volume", Some("-10")),
        "ffwd" => ("seek", Some("+30")),
        "rewind" => ("seek", Some("-30")),
        _ => anyhow::bail!("Unknown action: {}", action),
    };

    let mut command = tokio::process::Command::new(&exe);
    command.arg(cmd).arg("-d").arg(device).arg("-q");

    if let Some(arg) = extra_arg {
        command.arg(arg);
    }

    command.output().await?;

    Ok(())
}
