//! App state and core application logic
//!
//! Manages the application state machine, navigation stack,
//! and coordinates between UI and backend services.

use crate::config::save_settings_sync;
use crate::models::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

// =============================================================================
// Async Message Types
// =============================================================================

/// Commands sent from UI to async task spawner
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Fetch trending content
    FetchTrending,
    /// Search for content
    Search(String),
    /// Fetch movie detail
    FetchMovieDetail(u64),
    /// Fetch TV detail
    FetchTvDetail(u64),
    /// Fetch episodes for a TV season
    FetchEpisodes { tv_id: u64, season: u8 },
    /// Fetch streams for content
    FetchStreams { imdb_id: String, season: Option<u8>, episode: Option<u8> },
    /// Fetch subtitles (for TV: season/episode needed)
    FetchSubtitles { imdb_id: String, season: Option<u16>, episode: Option<u16>, lang: String },
    /// Discover Chromecast devices
    DiscoverDevices,
    /// Start playback (webtorrent + cast)
    StartPlayback {
        magnet: String,
        title: String,
        device: String,
        subtitle_url: Option<String>,
        file_idx: Option<u32>,
    },
    /// Stop playback
    StopPlayback,
    /// Restart playback with subtitles at position
    RestartWithSubtitles {
        magnet: String,
        title: String,
        device: String,
        subtitle_url: String,
        seek_seconds: u32,
        file_idx: Option<u32>,
    },
    /// Playback control (pause, volume, seek)
    PlaybackControl {
        action: String,
        device: String,
    },
    /// Save settings to config file
    SaveSettings {
        subtitle_lang: String,
        device_name: Option<String>,
    },
}

/// Results sent from async tasks back to UI
#[derive(Debug)]
pub enum AppMessage {
    /// Trending results loaded
    TrendingLoaded(Vec<SearchResult>),
    /// Search results loaded
    SearchResults(Vec<SearchResult>),
    /// Movie detail loaded
    MovieDetailLoaded(MovieDetail),
    /// TV detail loaded
    TvDetailLoaded(TvDetail),
    /// Episodes loaded for a season
    EpisodesLoaded { season: u8, episodes: Vec<Episode> },
    /// Streams loaded
    StreamsLoaded(Vec<StreamSource>),
    /// Subtitles loaded
    SubtitlesLoaded(Vec<SubtitleResult>),
    /// Chromecast devices discovered
    DevicesLoaded(Vec<CastDevice>),
    /// Playback started with stream URL
    PlaybackStarted { stream_url: String },
    /// Playback stopped
    PlaybackStopped,
    /// Error occurred
    Error(String),
}

// =============================================================================
// App State Enum
// =============================================================================

/// Application state enum representing current screen
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppState {
    /// Home screen with search box and trending content
    #[default]
    Home,
    /// Search results view
    Search,
    /// Detail view for a movie or TV show
    Detail,
    /// Source/quality selection screen
    Sources,
    /// Subtitle selection screen
    Subtitles,
    /// Now playing overlay (cast in progress)
    Playing,
}

// =============================================================================
// Input Mode
// =============================================================================

/// Current input mode for keyboard handling
#[derive(Debug, Clone, PartialEq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Text input mode (search box focused)
    Editing,
}

// =============================================================================
// Loading State
// =============================================================================

/// Loading state for async operations
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LoadingState {
    /// Idle - no loading in progress
    #[default]
    Idle,
    /// Loading with optional message
    Loading(Option<String>),
    /// Error with message
    Error(String),
}

impl LoadingState {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, LoadingState::Error(_))
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            LoadingState::Loading(Some(msg)) => Some(msg),
            LoadingState::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

// =============================================================================
// Selection State (per-view)
// =============================================================================

/// Selection state for list views
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset for viewport
    pub offset: usize,
    /// Total number of items
    pub len: usize,
}

impl ListState {
    pub fn new(len: usize) -> Self {
        Self {
            selected: 0,
            offset: 0,
            len,
        }
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Adjust offset if needed
            if self.selected < self.offset {
                self.offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        if self.len > 0 && self.selected < self.len - 1 {
            self.selected += 1;
        }
    }

    /// Move selection up by a page
    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    /// Move selection down by a page
    pub fn page_down(&mut self, page_size: usize) {
        if self.len > 0 {
            self.selected = (self.selected + page_size).min(self.len - 1);
        }
    }

    /// Jump to first item
    pub fn first(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    /// Jump to last item
    pub fn last(&mut self) {
        if self.len > 0 {
            self.selected = self.len - 1;
        }
    }

    /// Update offset to keep selected item visible
    pub fn scroll_into_view(&mut self, visible_height: usize) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_height {
            self.offset = self.selected - visible_height + 1;
        }
    }

    /// Reset selection
    pub fn reset(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    /// Update length (e.g., when new results come in)
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        // Clamp selected to valid range
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }
}

// =============================================================================
// View-Specific State
// =============================================================================

/// Home view state
#[derive(Debug, Clone, Default)]
pub struct HomeState {
    /// Trending content results
    pub results: Vec<SearchResult>,
    /// Trending content list state
    pub list: ListState,
    /// Loading state
    pub loading: LoadingState,
}

impl HomeState {
    /// Get currently selected trending item
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.list.selected)
    }
}

/// Search view state
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Search query
    pub query: String,
    /// Cursor position in query
    pub cursor: usize,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Results list state
    pub list: ListState,
    /// Loading state
    pub loading: LoadingState,
}

impl SearchState {
    /// Insert character at cursor
    pub fn insert(&mut self, c: char) {
        self.query.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            // Find the start of the previous character
            let prev_boundary = self.query[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.query.remove(prev_boundary);
            self.cursor = prev_boundary;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.query.len() {
            self.query.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            // Find the start of the previous character
            self.cursor = self.query[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor < self.query.len() {
            // Find the start of the next character
            if let Some(c) = self.query[self.cursor..].chars().next() {
                self.cursor += c.len_utf8();
            }
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor = self.query.len();
    }

    /// Clear query
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
    }

    /// Set results and update list state
    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.list.set_len(results.len());
        self.results = results;
        self.loading = LoadingState::Idle;
    }

    /// Get currently selected result
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.list.selected)
    }
}

/// Focus state for TV detail view (which panel is focused)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TvFocus {
    #[default]
    Seasons,
    Episodes,
}

/// Detail view state (movie or TV show)
#[derive(Debug, Clone)]
pub enum DetailState {
    /// Movie detail
    Movie {
        detail: MovieDetail,
        loading: LoadingState,
    },
    /// TV show detail
    Tv {
        detail: TvDetail,
        season_list: ListState,
        episode_list: ListState,
        episodes: Vec<Episode>,
        selected_season: u8,
        focus: TvFocus,
        loading: LoadingState,
    },
}

impl DetailState {
    pub fn movie(detail: MovieDetail) -> Self {
        DetailState::Movie {
            detail,
            loading: LoadingState::Idle,
        }
    }

    pub fn tv(detail: TvDetail) -> Self {
        let season_count = detail.seasons.len();
        DetailState::Tv {
            detail,
            season_list: ListState::new(season_count),
            episode_list: ListState::new(0),
            episodes: Vec::new(),
            selected_season: 1,
            focus: TvFocus::Seasons,
            loading: LoadingState::Idle,
        }
    }

    /// Get IMDB ID for the current content
    pub fn imdb_id(&self) -> &str {
        match self {
            DetailState::Movie { detail, .. } => &detail.imdb_id,
            DetailState::Tv { detail, .. } => &detail.imdb_id,
        }
    }

    /// Get title for the current content
    pub fn title(&self) -> &str {
        match self {
            DetailState::Movie { detail, .. } => &detail.title,
            DetailState::Tv { detail, .. } => &detail.name,
        }
    }
}

/// Sources view state
#[derive(Debug, Clone, Default)]
pub struct SourcesState {
    /// Available stream sources
    pub sources: Vec<StreamSource>,
    /// List state
    pub list: ListState,
    /// Loading state
    pub loading: LoadingState,
    /// Content title (for display)
    pub title: String,
}

impl SourcesState {
    pub fn new(title: String) -> Self {
        Self {
            sources: Vec::new(),
            list: ListState::new(0),
            loading: LoadingState::Loading(Some("Fetching sources...".into())),
            title,
        }
    }

    pub fn set_sources(&mut self, sources: Vec<StreamSource>) {
        self.list.set_len(sources.len());
        self.sources = sources;
        self.loading = LoadingState::Idle;
    }

    pub fn selected_source(&self) -> Option<&StreamSource> {
        self.sources.get(self.list.selected)
    }
}

/// Subtitle language filter options
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SubLangFilter {
    #[default]
    EngSpa,  // English + Spanish (default)
    English, // English only
    Spanish, // Spanish only
    All,     // All languages
}

impl SubLangFilter {
    /// Cycle to next filter
    pub fn next(self) -> Self {
        match self {
            Self::EngSpa => Self::English,
            Self::English => Self::Spanish,
            Self::Spanish => Self::All,
            Self::All => Self::EngSpa,
        }
    }

    /// Get the language code(s) for API call
    pub fn lang_code(&self) -> &'static str {
        match self {
            Self::EngSpa => "eng,spa",
            Self::English => "eng",
            Self::Spanish => "spa",
            Self::All => "",
        }
    }

    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::EngSpa => "🇬🇧+🇪🇸",
            Self::English => "🇬🇧 English",
            Self::Spanish => "🇪🇸 Spanish",
            Self::All => "🌍 All",
        }
    }
}

/// Subtitles view state
#[derive(Debug, Clone, Default)]
pub struct SubtitlesState {
    /// Available subtitles
    pub subtitles: Vec<SubtitleResult>,
    /// List state
    pub list: ListState,
    /// Loading state
    pub loading: LoadingState,
    /// Selected subtitle (for playback)
    pub selected: Option<SubtitleResult>,
    /// Language filter
    pub lang_filter: SubLangFilter,
}

impl SubtitlesState {
    pub fn set_subtitles(&mut self, subtitles: Vec<SubtitleResult>) {
        self.list.set_len(subtitles.len());
        self.subtitles = subtitles;
        self.loading = LoadingState::Idle;
    }

    pub fn selected_subtitle(&self) -> Option<&SubtitleResult> {
        self.subtitles.get(self.list.selected)
    }
}

/// Playing view state
#[derive(Debug, Clone, Default)]
pub struct PlayingState {
    /// Current torrent session
    pub torrent: Option<TorrentSession>,
    /// Cast target device
    pub device: Option<CastDevice>,
    /// Playback status
    pub playback: Option<PlaybackStatus>,
    /// Content title
    pub title: String,
    /// Selected subtitle file
    pub subtitle: Option<SubtitleFile>,
    /// Current magnet URL (for restart with subtitles)
    pub magnet: Option<String>,
    /// Pending subtitle change (triggers restart on return to Playing)
    pub pending_subtitle_url: Option<String>,
}

// =============================================================================
// Main Application State
// =============================================================================

/// Main application state
pub struct App {
    /// Current state/screen
    pub state: AppState,
    /// Navigation history stack
    pub nav_stack: Vec<AppState>,
    /// Whether the app is running
    pub running: bool,
    /// Current input mode
    pub input_mode: InputMode,
    /// Global error message
    pub error: Option<String>,

    // View-specific states
    pub home: HomeState,
    pub search: SearchState,
    pub detail: Option<DetailState>,
    pub sources: SourcesState,
    pub subtitles: SubtitlesState,
    pub playing: PlayingState,

    // Shared state
    /// Available cast devices
    pub cast_devices: Vec<CastDevice>,
    /// Selected cast device index
    pub selected_device: Option<usize>,
    /// Show device selection modal
    pub show_device_modal: bool,
    /// Device modal selection index (separate from selected_device until confirmed)
    pub device_modal_index: usize,

    // Settings
    /// Default subtitle language (ISO 639-1 code, e.g., "en", "es", "fr")
    pub default_subtitle_lang: String,
    /// Default device name (from config, used to auto-select on discovery)
    pub default_device_name: Option<String>,
    /// Show settings modal
    pub show_settings_modal: bool,
    /// Settings modal field index (0=language, 1=device)
    pub settings_field_index: usize,
    /// Temporary language input while editing
    pub settings_lang_input: String,

    // Async communication
    /// Channel to send commands to async task spawner
    pub cmd_tx: mpsc::UnboundedSender<AppCommand>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new App instance (for tests - commands are no-op)
    pub fn new() -> Self {
        let (cmd_tx, _) = mpsc::unbounded_channel();
        Self {
            state: AppState::Home,
            nav_stack: Vec::new(),
            running: true,
            input_mode: InputMode::Normal,
            error: None,

            home: HomeState::default(),
            search: SearchState::default(),
            detail: None,
            sources: SourcesState::default(),
            subtitles: SubtitlesState::default(),
            playing: PlayingState::default(),

            // Initialize with VLC as default device (always available)
            cast_devices: vec![CastDevice {
                id: "vlc-local".to_string(),
                name: "VLC (Local)".to_string(),
                address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                port: 0,
                model: Some("Local Playback".to_string()),
            }],
            selected_device: Some(0), // VLC selected by default
            show_device_modal: false,
            device_modal_index: 0,

            default_subtitle_lang: "eng,spa".to_string(), // English + Spanish by default
            default_device_name: None,
            show_settings_modal: false,
            settings_field_index: 0,
            settings_lang_input: String::new(),

            cmd_tx,
        }
    }

    /// Create a new App instance with async channel for TUI
    /// Returns (App, command_receiver) - caller must handle commands
    pub fn with_channels() -> (Self, mpsc::UnboundedReceiver<AppCommand>) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let app = Self {
            state: AppState::Home,
            nav_stack: Vec::new(),
            running: true,
            input_mode: InputMode::Normal,
            error: None,

            home: HomeState::default(),
            search: SearchState::default(),
            detail: None,
            sources: SourcesState::default(),
            subtitles: SubtitlesState::default(),
            playing: PlayingState::default(),

            // Initialize with VLC as default device (always available)
            cast_devices: vec![CastDevice {
                id: "vlc-local".to_string(),
                name: "VLC (Local)".to_string(),
                address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                port: 0,
                model: Some("Local Playback".to_string()),
            }],
            selected_device: Some(0), // VLC selected by default
            show_device_modal: false,
            device_modal_index: 0,

            default_subtitle_lang: "eng,spa".to_string(), // English + Spanish by default
            default_device_name: None,
            show_settings_modal: false,
            settings_field_index: 0,
            settings_lang_input: String::new(),

            cmd_tx,
        };

        (app, cmd_rx)
    }

    /// Send a command to the async task spawner
    pub fn send_command(&self, cmd: AppCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// Handle an incoming async message
    pub fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::TrendingLoaded(results) => {
                self.home.results = results;
                self.home.list.set_len(self.home.results.len());
                self.home.loading = LoadingState::Idle;
            }
            AppMessage::SearchResults(results) => {
                self.search.set_results(results);
            }
            AppMessage::MovieDetailLoaded(detail) => {
                self.detail = Some(DetailState::movie(detail));
                self.navigate(AppState::Detail);
            }
            AppMessage::TvDetailLoaded(detail) => {
                // Store the TV ID for episode fetching
                let tv_id = detail.id;
                self.detail = Some(DetailState::tv(detail));
                self.navigate(AppState::Detail);
                // Auto-fetch first season's episodes
                self.send_command(AppCommand::FetchEpisodes { tv_id, season: 1 });
            }
            AppMessage::EpisodesLoaded { season, episodes } => {
                // Update the TV detail state with loaded episodes
                if let Some(DetailState::Tv { episodes: eps, episode_list, selected_season, .. }) = &mut self.detail {
                    *eps = episodes;
                    episode_list.set_len(eps.len());
                    *selected_season = season;
                }
            }
            AppMessage::StreamsLoaded(streams) => {
                self.sources.set_sources(streams);
            }
            AppMessage::SubtitlesLoaded(subs) => {
                self.subtitles.set_subtitles(subs);
            }
            AppMessage::DevicesLoaded(devices) => {
                // Start with VLC as first option (always available for local playback)
                let mut all_devices = vec![CastDevice {
                    id: "vlc-local".to_string(),
                    name: "VLC (Local)".to_string(),
                    address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                    port: 0,
                    model: Some("Local Playback".to_string()),
                }];
                all_devices.extend(devices);
                self.cast_devices = all_devices;
                // Always try to match default device when devices are loaded
                if !self.cast_devices.is_empty() {
                    if let Some(ref default_name) = self.default_device_name {
                        // Try to find and select the saved default device
                        if let Some(idx) = self.cast_devices.iter().position(|d| &d.name == default_name) {
                            self.selected_device = Some(idx);
                        } else if self.selected_device.is_none() {
                            // Fallback to first device if default not found
                            self.selected_device = Some(0);
                        }
                    } else if self.selected_device.is_none() {
                        // No default configured, select first
                        self.selected_device = Some(0);
                    }
                }
            }
            AppMessage::PlaybackStarted { stream_url } => {
                // Update torrent session with stream URL
                if let Some(ref mut session) = self.playing.torrent {
                    session.stream_url = Some(stream_url);
                    session.state = TorrentState::Streaming;
                }
            }
            AppMessage::PlaybackStopped => {
                self.playing = PlayingState::default();
                self.back();
            }
            AppMessage::Error(msg) => {
                self.set_error(msg);
                // Reset loading states
                self.home.loading = LoadingState::Idle;
                self.search.loading = LoadingState::Idle;
                self.sources.loading = LoadingState::Idle;
                self.subtitles.loading = LoadingState::Idle;
            }
        }
    }

    /// Navigate to a new state, pushing current to stack
    pub fn navigate(&mut self, state: AppState) {
        // Don't push if going to same state
        if self.state != state {
            self.nav_stack.push(self.state.clone());
            self.state = state;
        }
        // Reset input mode when navigating
        self.input_mode = InputMode::Normal;
    }

    /// Go back to previous state
    pub fn back(&mut self) -> bool {
        // If in editing mode, exit editing first
        if self.input_mode == InputMode::Editing {
            self.input_mode = InputMode::Normal;
            return true;
        }

        if let Some(prev) = self.nav_stack.pop() {
            self.state = prev.clone();

            // If returning to Playing with a pending subtitle, trigger restart
            if prev == AppState::Playing {
                if let Some(subtitle_url) = self.playing.pending_subtitle_url.take() {
                    self.trigger_subtitle_restart(subtitle_url);
                }
            }

            true
        } else {
            false
        }
    }

    /// Trigger playback restart with new subtitle at current position
    fn trigger_subtitle_restart(&mut self, subtitle_url: String) {
        let Some(magnet) = self.playing.magnet.clone() else { return };
        let Some(device) = self.playing.device.clone() else { return };

        // Get current position (default to 0 if unknown)
        let seek_seconds = self.playing.playback
            .as_ref()
            .map(|p| p.position.as_secs() as u32)
            .unwrap_or(0);

        // Get file_idx from torrent session
        let file_idx = self.playing.torrent.as_ref().and_then(|t| t.file_idx);

        self.send_command(AppCommand::RestartWithSubtitles {
            magnet,
            title: self.playing.title.clone(),
            device: device.name,
            subtitle_url,
            seek_seconds,
            file_idx,
        });
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Set error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(msg.into());
    }

    /// Focus search input
    pub fn focus_search(&mut self) {
        if self.state == AppState::Home || self.state == AppState::Search {
            if self.state == AppState::Home {
                self.navigate(AppState::Search);
            }
            // Set editing mode AFTER navigate (navigate resets to Normal)
            self.input_mode = InputMode::Editing;
        }
    }

    /// Get currently selected cast device
    pub fn selected_cast_device(&self) -> Option<&CastDevice> {
        self.selected_device.and_then(|i| self.cast_devices.get(i))
    }

    // -------------------------------------------------------------------------
    // Keyboard Event Handling
    // -------------------------------------------------------------------------

    /// Handle keyboard event, returns true if event was consumed
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Clear error on any keypress
        self.error = None;

        // Global quit shortcut (Ctrl+C or q in normal mode)
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.quit();
            return true;
        }

        // Handle device modal if open
        if self.show_device_modal {
            return self.handle_device_modal_key(key);
        }

        // Handle settings modal if open
        if self.show_settings_modal {
            return self.handle_settings_modal_key(key);
        }

        // Route to appropriate handler based on mode and state
        if self.input_mode == InputMode::Editing {
            self.handle_editing_key(key)
        } else {
            self.handle_normal_key(key)
        }
    }

    /// Handle keys when device selection modal is open
    fn handle_device_modal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('d') => {
                self.show_device_modal = false;
                true
            }
            KeyCode::Enter => {
                // Confirm selection and save as default
                if !self.cast_devices.is_empty() {
                    self.selected_device = Some(self.device_modal_index);

                    // Save as default device
                    let device_name = self.cast_devices
                        .get(self.device_modal_index)
                        .map(|d| d.name.clone());
                    self.default_device_name = device_name.clone();

                    // Persist to config synchronously (don't rely on async)
                    save_settings_sync(&self.default_subtitle_lang, device_name.as_deref());
                }
                self.show_device_modal = false;
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.cast_devices.is_empty() {
                    if self.device_modal_index > 0 {
                        self.device_modal_index -= 1;
                    } else {
                        self.device_modal_index = self.cast_devices.len() - 1;
                    }
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.cast_devices.is_empty() {
                    self.device_modal_index = (self.device_modal_index + 1) % self.cast_devices.len();
                }
                true
            }
            KeyCode::Char('r') => {
                // Refresh devices
                self.send_command(AppCommand::DiscoverDevices);
                true
            }
            _ => true, // Consume all other keys when modal is open
        }
    }

    /// Handle keys when settings modal is open
    fn handle_settings_modal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('o') => {
                // Close without saving changes to temp input
                self.show_settings_modal = false;
                true
            }
            KeyCode::Enter => {
                // Confirm changes - save language if valid
                let lang = self.settings_lang_input.trim().to_lowercase();
                if lang.len() >= 2 && lang.len() <= 3 {
                    self.default_subtitle_lang = lang.clone();
                }

                // Get current device name for saving
                let device_name = self
                    .selected_device
                    .and_then(|i| self.cast_devices.get(i))
                    .map(|d| d.name.clone());

                // Update local cache
                self.default_device_name = device_name.clone();

                // Persist to config synchronously
                save_settings_sync(&self.default_subtitle_lang, device_name.as_deref());

                self.show_settings_modal = false;
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_field_index > 0 {
                    self.settings_field_index -= 1;
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Only 2 fields: language (0) and device info (1)
                if self.settings_field_index < 1 {
                    self.settings_field_index += 1;
                }
                true
            }
            KeyCode::Char(c) => {
                // Only language field (index 0) accepts text input
                if self.settings_field_index == 0 && self.settings_lang_input.len() < 3 {
                    self.settings_lang_input.push(c);
                }
                true
            }
            KeyCode::Backspace => {
                if self.settings_field_index == 0 {
                    self.settings_lang_input.pop();
                }
                true
            }
            _ => true,
        }
    }

    /// Handle keys in editing (text input) mode
    fn handle_editing_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                true
            }
            KeyCode::Enter => {
                // Submit search
                self.input_mode = InputMode::Normal;
                if !self.search.query.is_empty() {
                    self.search.loading = LoadingState::Loading(Some("Searching...".into()));
                    self.send_command(AppCommand::Search(self.search.query.clone()));
                }
                true
            }
            KeyCode::Char(c) => {
                self.search.insert(c);
                true
            }
            KeyCode::Backspace => {
                self.search.backspace();
                true
            }
            KeyCode::Delete => {
                self.search.delete();
                true
            }
            KeyCode::Left => {
                self.search.cursor_left();
                true
            }
            KeyCode::Right => {
                self.search.cursor_right();
                true
            }
            KeyCode::Home => {
                self.search.cursor_home();
                true
            }
            KeyCode::End => {
                self.search.cursor_end();
                true
            }
            _ => false,
        }
    }

    /// Handle keys in normal navigation mode
    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        // Global shortcuts
        match key.code {
            KeyCode::Char('q') => {
                self.quit();
                return true;
            }
            KeyCode::Char('/') => {
                self.focus_search();
                return true;
            }
            // 's' focuses search except in Playing state where it stops playback
            KeyCode::Char('s') if self.state != AppState::Playing => {
                self.focus_search();
                return true;
            }
            // 'd' opens device selection modal globally
            KeyCode::Char('d') => {
                self.show_device_modal = true;
                self.device_modal_index = self.selected_device.unwrap_or(0);
                self.send_command(AppCommand::DiscoverDevices);
                return true;
            }
            // 'o' opens settings modal
            KeyCode::Char('o') => {
                self.show_settings_modal = true;
                self.settings_field_index = 0;
                self.settings_lang_input = self.default_subtitle_lang.clone();
                return true;
            }
            KeyCode::Esc => {
                return self.back();
            }
            _ => {}
        }

        // State-specific handling
        match &self.state {
            AppState::Home => self.handle_home_key(key),
            AppState::Search => self.handle_search_key(key),
            AppState::Detail => self.handle_detail_key(key),
            AppState::Sources => self.handle_sources_key(key),
            AppState::Subtitles => self.handle_subtitles_key(key),
            AppState::Playing => self.handle_playing_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.home.list.up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.home.list.down();
                true
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                // Open detail view for selected trending item
                if let Some(result) = self.home.selected_result() {
                    let id = result.id;
                    match result.media_type {
                        crate::models::MediaType::Movie => {
                            self.send_command(AppCommand::FetchMovieDetail(id));
                        }
                        crate::models::MediaType::Tv => {
                            self.send_command(AppCommand::FetchTvDetail(id));
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.search.list.up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.search.list.down();
                true
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                // Open detail view for selected result
                if let Some(result) = self.search.selected_result() {
                    let id = result.id;
                    match result.media_type {
                        crate::models::MediaType::Movie => {
                            self.send_command(AppCommand::FetchMovieDetail(id));
                        }
                        crate::models::MediaType::Tv => {
                            self.send_command(AppCommand::FetchTvDetail(id));
                        }
                    }
                }
                true
            }
            KeyCode::PageUp => {
                self.search.list.page_up(10);
                true
            }
            KeyCode::PageDown => {
                self.search.list.page_down(10);
                true
            }
            KeyCode::Home => {
                self.search.list.first();
                true
            }
            KeyCode::End => {
                self.search.list.last();
                true
            }
            _ => false,
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(DetailState::Tv { season_list, episode_list, focus, .. }) = &mut self.detail {
                    match focus {
                        TvFocus::Seasons => season_list.up(),
                        TvFocus::Episodes => episode_list.up(),
                    }
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(DetailState::Tv { season_list, episode_list, focus, .. }) = &mut self.detail {
                    match focus {
                        TvFocus::Seasons => season_list.down(),
                        TvFocus::Episodes => episode_list.down(),
                    }
                }
                true
            }
            KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                // Switch between seasons/episodes panel
                if let Some(DetailState::Tv { focus, .. }) = &mut self.detail {
                    *focus = match focus {
                        TvFocus::Seasons => TvFocus::Episodes,
                        TvFocus::Episodes => TvFocus::Seasons,
                    };
                }
                true
            }
            KeyCode::Enter => {
                // For TV: if on seasons panel, load episodes for selected season
                // If on episodes panel (or movie), go to sources
                if let Some(DetailState::Tv { detail, season_list, focus, .. }) = &self.detail {
                    if *focus == TvFocus::Seasons {
                        // Load episodes for selected season
                        let tv_id = detail.id;
                        let selected_season_idx = season_list.selected;
                        if let Some(season) = detail.seasons.get(selected_season_idx) {
                            let season_num = season.season_number;
                            self.send_command(AppCommand::FetchEpisodes { tv_id, season: season_num });
                            // Switch focus to episodes
                            if let Some(DetailState::Tv { focus, .. }) = &mut self.detail {
                                *focus = TvFocus::Episodes;
                            }
                        }
                        return true;
                    }
                }
                // Fetch sources (movie or TV episode)
                self.fetch_sources_for_current();
                true
            }
            KeyCode::Char('c') => {
                // Always go to sources (skip season selection)
                self.fetch_sources_for_current();
                true
            }
            KeyCode::Char('u') => {
                // Fetch subtitles and navigate
                if let Some(detail) = &self.detail {
                    let (imdb_id, season, episode) = match detail {
                        DetailState::Movie { detail, .. } => (detail.imdb_id.clone(), None, None),
                        DetailState::Tv { detail, selected_season, episode_list, episodes, .. } => {
                            // Get selected episode number for TV
                            let season_num = detail.seasons.get(*selected_season as usize)
                                .map(|s| s.season_number as u16);
                            let episode_num = episodes.get(episode_list.selected)
                                .map(|e| e.episode as u16);
                            (detail.imdb_id.clone(), season_num, episode_num)
                        }
                    };
                    self.subtitles.loading = LoadingState::Loading(Some("Fetching subtitles...".into()));
                    self.send_command(AppCommand::FetchSubtitles { imdb_id, season, episode, lang: self.subtitles.lang_filter.lang_code().to_string() });
                    self.navigate(AppState::Subtitles);
                }
                true
            }
            _ => false,
        }
    }

    /// Fetch sources for current selection (movie or TV episode)
    fn fetch_sources_for_current(&mut self) {
        if let Some(detail) = &self.detail {
            let (imdb_id, season, episode, title) = match detail {
                DetailState::Movie { detail, .. } => {
                    (detail.imdb_id.clone(), None, None, detail.title.clone())
                }
                DetailState::Tv { detail, selected_season, episode_list, episodes, .. } => {
                    let ep = episodes.get(episode_list.selected);
                    let ep_num = ep.map(|e| e.episode);
                    let title = ep.map(|e| format!("{} S{}E{}", detail.name, selected_season, e.episode))
                        .unwrap_or_else(|| detail.name.clone());
                    (detail.imdb_id.clone(), Some(*selected_season), ep_num, title)
                }
            };
            self.sources.title = title;
            self.sources.loading = LoadingState::Loading(Some("Fetching streams...".into()));
            self.send_command(AppCommand::FetchStreams { imdb_id, season, episode });
            self.navigate(AppState::Sources);
        }
    }

    fn handle_sources_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.sources.list.up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.sources.list.down();
                true
            }
            KeyCode::Enter | KeyCode::Char('c') => {
                // Start streaming selected source
                self.start_playback();
                true
            }
            KeyCode::Char(c @ '1'..='9') => {
                // Quick select source by number
                let idx = (c as usize) - ('1' as usize);
                if idx < self.sources.sources.len() {
                    self.sources.list.selected = idx;
                }
                true
            }
            KeyCode::Char('u') => {
                // Go to subtitles and trigger fetch
                self.navigate(AppState::Subtitles);
                // Auto-fetch subtitles if we have an IMDB ID
                if let Some(imdb_id) = self.get_imdb_id() {
                    let (season, episode) = self.get_season_episode();
                    self.subtitles.loading = LoadingState::Loading(Some("Fetching subtitles...".into()));
                    self.send_command(AppCommand::FetchSubtitles { imdb_id, season, episode, lang: self.subtitles.lang_filter.lang_code().to_string() });
                }
                true
            }
            KeyCode::Tab => {
                // Cycle to next device
                if !self.cast_devices.is_empty() {
                    let current = self.selected_device.unwrap_or(0);
                    self.selected_device = Some((current + 1) % self.cast_devices.len());
                }
                true
            }
            KeyCode::BackTab => {
                // Cycle to previous device
                if !self.cast_devices.is_empty() {
                    let current = self.selected_device.unwrap_or(0);
                    let len = self.cast_devices.len();
                    self.selected_device = Some((current + len - 1) % len);
                }
                true
            }
            _ => false,
        }
    }

    /// Start playback of selected source on selected device
    fn start_playback(&mut self) {
        // Check we have a device selected
        let device = match self.selected_device {
            Some(idx) if idx < self.cast_devices.len() => &self.cast_devices[idx],
            _ => {
                self.set_error("No Chromecast device selected. Press 'd' to discover devices.");
                return;
            }
        };

        // Check we have a source selected
        let source = match self.sources.selected_source() {
            Some(s) => s.clone(),
            None => {
                self.set_error("No stream source selected.");
                return;
            }
        };

        // Generate magnet link
        let magnet = source.to_magnet(&self.sources.title);

        // Get subtitle URL if selected
        let subtitle_url = self.subtitles.selected.as_ref().map(|s| s.url.clone());

        // Set up playing state
        self.playing.title = self.sources.title.clone();
        self.playing.device = Some(device.clone());
        self.playing.torrent = Some(TorrentSession::new(magnet.clone(), source.file_idx));
        self.playing.magnet = Some(magnet.clone()); // Store for subtitle restart
        self.playing.pending_subtitle_url = None;

        // Send command to start playback
        self.send_command(AppCommand::StartPlayback {
            magnet,
            title: self.sources.title.clone(),
            device: device.name.clone(),
            subtitle_url,
            file_idx: source.file_idx,
        });

        // Navigate to Playing state
        self.navigate(AppState::Playing);
    }

    /// Get IMDB ID from current detail
    fn get_imdb_id(&self) -> Option<String> {
        self.detail.as_ref().map(|d| match d {
            DetailState::Movie { detail, .. } => detail.imdb_id.clone(),
            DetailState::Tv { detail, .. } => detail.imdb_id.clone(),
        })
    }

    /// Get season/episode from current TV detail (if applicable)
    fn get_season_episode(&self) -> (Option<u16>, Option<u16>) {
        match &self.detail {
            Some(DetailState::Tv { detail, selected_season, episode_list, episodes, .. }) => {
                let season_num = detail.seasons.get(*selected_season as usize)
                    .map(|s| s.season_number as u16);
                let episode_num = episodes.get(episode_list.selected)
                    .map(|e| e.episode as u16);
                (season_num, episode_num)
            }
            _ => (None, None),
        }
    }

    fn handle_subtitles_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.subtitles.list.up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.subtitles.list.down();
                true
            }
            KeyCode::Tab => {
                // Toggle language filter and refetch
                self.subtitles.lang_filter = self.subtitles.lang_filter.next();
                if let Some(imdb_id) = self.get_imdb_id() {
                    let (season, episode) = self.get_season_episode();
                    self.subtitles.loading = LoadingState::Loading(Some("Fetching subtitles...".into()));
                    let lang = self.subtitles.lang_filter.lang_code().to_string();
                    self.send_command(AppCommand::FetchSubtitles { imdb_id, season, episode, lang });
                }
                true
            }
            KeyCode::Enter => {
                // Select subtitle (clone first to avoid borrow issues)
                if let Some(sub) = self.subtitles.selected_subtitle().cloned() {
                    // If coming from Playing state, set pending subtitle for restart
                    if self.nav_stack.last() == Some(&AppState::Playing) {
                        self.playing.pending_subtitle_url = Some(sub.url.clone());
                    }
                    self.subtitles.selected = Some(sub);
                }
                self.back();
                true
            }
            KeyCode::Char('n') => {
                // No subtitles
                self.subtitles.selected = None;
                self.back();
                true
            }
            _ => false,
        }
    }

    fn handle_playing_key(&mut self, key: KeyEvent) -> bool {
        // Get device name for commands
        let device_name = match &self.playing.device {
            Some(d) => d.name.clone(),
            None => return false,
        };

        match key.code {
            KeyCode::Char(' ') => {
                // Toggle pause - send command to catt
                self.send_command(AppCommand::PlaybackControl {
                    action: "play_toggle".into(),
                    device: device_name,
                });
                // Also update local state
                if let Some(ref mut playback) = self.playing.playback {
                    playback.state = match playback.state {
                        CastState::Playing => CastState::Paused,
                        CastState::Paused => CastState::Playing,
                        _ => playback.state.clone(),
                    };
                }
                true
            }
            KeyCode::Char('s') => {
                // Stop playback
                self.send_command(AppCommand::StopPlayback);
                // Update local state
                if let Some(ref mut playback) = self.playing.playback {
                    playback.state = CastState::Stopped;
                }
                true
            }
            KeyCode::Left => {
                // Seek backward 30 seconds
                self.send_command(AppCommand::PlaybackControl {
                    action: "rewind".into(),
                    device: device_name,
                });
                true
            }
            KeyCode::Right => {
                // Seek forward 30 seconds
                self.send_command(AppCommand::PlaybackControl {
                    action: "ffwd".into(),
                    device: device_name,
                });
                true
            }
            KeyCode::Up => {
                // Volume up
                self.send_command(AppCommand::PlaybackControl {
                    action: "volumeup".into(),
                    device: device_name,
                });
                if let Some(ref mut playback) = self.playing.playback {
                    playback.volume = (playback.volume + 0.1).min(1.0);
                }
                true
            }
            KeyCode::Down => {
                // Volume down
                self.send_command(AppCommand::PlaybackControl {
                    action: "volumedown".into(),
                    device: device_name,
                });
                if let Some(ref mut playback) = self.playing.playback {
                    playback.volume = (playback.volume - 0.1).max(0.0);
                }
                true
            }
            KeyCode::Char('u') => {
                // Open subtitle selector
                if let Some(imdb_id) = self.get_imdb_id() {
                    let (season, episode) = self.get_season_episode();
                    self.subtitles.loading = LoadingState::Loading(Some("Fetching subtitles...".into()));
                    self.send_command(AppCommand::FetchSubtitles {
                        imdb_id,
                        season,
                        episode,
                        lang: self.subtitles.lang_filter.lang_code().to_string()
                    });
                    self.navigate(AppState::Subtitles);
                }
                true
            }
            _ => false,
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ListState Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_list_state_navigation() {
        let mut list = ListState::new(5);
        assert_eq!(list.selected, 0);

        list.down();
        assert_eq!(list.selected, 1);

        list.down();
        list.down();
        list.down();
        assert_eq!(list.selected, 4);

        // Can't go past end
        list.down();
        assert_eq!(list.selected, 4);

        list.up();
        assert_eq!(list.selected, 3);

        list.first();
        assert_eq!(list.selected, 0);

        list.last();
        assert_eq!(list.selected, 4);
    }

    #[test]
    fn test_list_state_empty() {
        let mut list = ListState::new(0);
        list.down();
        assert_eq!(list.selected, 0);
        list.up();
        assert_eq!(list.selected, 0);
    }

    #[test]
    fn test_list_state_set_len() {
        let mut list = ListState::new(10);
        list.selected = 8;

        // Shrinking should clamp selection
        list.set_len(5);
        assert_eq!(list.selected, 4);

        // Growing shouldn't change selection
        list.set_len(10);
        assert_eq!(list.selected, 4);
    }

    // -------------------------------------------------------------------------
    // SearchState Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_search_state_editing() {
        let mut search = SearchState::default();

        search.insert('h');
        search.insert('e');
        search.insert('l');
        search.insert('l');
        search.insert('o');
        assert_eq!(search.query, "hello");
        assert_eq!(search.cursor, 5);

        search.cursor_left();
        search.cursor_left();
        assert_eq!(search.cursor, 3);

        search.insert('X');
        assert_eq!(search.query, "helXlo");
        assert_eq!(search.cursor, 4);

        search.backspace();
        assert_eq!(search.query, "hello");

        search.cursor_home();
        assert_eq!(search.cursor, 0);

        search.cursor_end();
        assert_eq!(search.cursor, 5);
    }

    #[test]
    fn test_search_state_clear() {
        let mut search = SearchState::default();
        search.query = "test".into();
        search.cursor = 4;

        search.clear();
        assert_eq!(search.query, "");
        assert_eq!(search.cursor, 0);
    }

    // -------------------------------------------------------------------------
    // App Navigation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_navigation() {
        let mut app = App::new();
        assert_eq!(app.state, AppState::Home);
        assert!(app.nav_stack.is_empty());

        app.navigate(AppState::Search);
        assert_eq!(app.state, AppState::Search);
        assert_eq!(app.nav_stack.len(), 1);

        app.navigate(AppState::Detail);
        assert_eq!(app.state, AppState::Detail);
        assert_eq!(app.nav_stack.len(), 2);

        assert!(app.back());
        assert_eq!(app.state, AppState::Search);

        assert!(app.back());
        assert_eq!(app.state, AppState::Home);

        // Can't go back from home
        assert!(!app.back());
        assert_eq!(app.state, AppState::Home);
    }

    #[test]
    fn test_app_navigate_same_state() {
        let mut app = App::new();
        app.navigate(AppState::Search);

        // Navigating to same state shouldn't push to stack
        app.navigate(AppState::Search);
        assert_eq!(app.nav_stack.len(), 1);
    }

    // -------------------------------------------------------------------------
    // App Key Handling Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_quit_key() {
        let mut app = App::new();
        assert!(app.running);

        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
        assert!(!app.running);
    }

    #[test]
    fn test_app_quit_ctrl_c() {
        let mut app = App::new();
        assert!(app.running);

        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!app.running);
    }

    #[test]
    fn test_app_focus_search() {
        let mut app = App::new();
        assert_eq!(app.input_mode, InputMode::Normal);

        app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
        assert_eq!(app.input_mode, InputMode::Editing);
        assert_eq!(app.state, AppState::Search);
    }

    #[test]
    fn test_app_editing_mode() {
        let mut app = App::new();
        app.focus_search();

        // Type some text
        app.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
        app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
        app.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
        assert_eq!(app.search.query, "test");

        // Escape exits editing mode
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_app_escape_from_editing_first() {
        let mut app = App::new();
        app.navigate(AppState::Search);
        app.input_mode = InputMode::Editing;

        // First escape exits editing mode
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.state, AppState::Search); // Still on search

        // Second escape goes back
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        assert_eq!(app.state, AppState::Home);
    }

    // -------------------------------------------------------------------------
    // LoadingState Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_loading_state() {
        let idle = LoadingState::Idle;
        assert!(!idle.is_loading());
        assert!(!idle.is_error());

        let loading = LoadingState::Loading(Some("Loading...".into()));
        assert!(loading.is_loading());
        assert_eq!(loading.message(), Some("Loading..."));

        let error = LoadingState::Error("Failed".into());
        assert!(error.is_error());
        assert_eq!(error.message(), Some("Failed"));
    }

    // -------------------------------------------------------------------------
    // Sources Quick Select Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sources_quick_select() {
        let mut app = App::new();
        app.state = AppState::Sources;
        app.sources.sources = vec![
            StreamSource {
                name: "1".into(),
                title: "Source 1".into(),
                info_hash: "hash1".into(),
                file_idx: None,
                seeds: 100,
                quality: Quality::FHD1080p,
                size_bytes: None,
            },
            StreamSource {
                name: "2".into(),
                title: "Source 2".into(),
                info_hash: "hash2".into(),
                file_idx: None,
                seeds: 200,
                quality: Quality::UHD4K,
                size_bytes: None,
            },
        ];
        app.sources.list.set_len(2);

        // Press '2' to select second source
        app.handle_key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()));
        assert_eq!(app.sources.list.selected, 1);
    }

    // -------------------------------------------------------------------------
    // Playing Controls Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_playing_pause_toggle() {
        let mut app = App::new();
        app.state = AppState::Playing;
        // Must have a device for playing controls to work
        app.playing.device = Some(CastDevice {
            id: "test".into(),
            name: "Test TV".into(),
            address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: None,
        });
        app.playing.playback = Some(PlaybackStatus {
            state: CastState::Playing,
            position: std::time::Duration::from_secs(100),
            duration: std::time::Duration::from_secs(3600),
            volume: 0.8,
            title: Some("Test".into()),
        });

        // Space toggles pause
        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert_eq!(
            app.playing.playback.as_ref().unwrap().state,
            CastState::Paused
        );

        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert_eq!(
            app.playing.playback.as_ref().unwrap().state,
            CastState::Playing
        );
    }

    #[test]
    fn test_playing_volume() {
        let mut app = App::new();
        app.state = AppState::Playing;
        // Must have a device for playing controls to work
        app.playing.device = Some(CastDevice {
            id: "test".into(),
            name: "Test TV".into(),
            address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: None,
        });
        app.playing.playback = Some(PlaybackStatus {
            state: CastState::Playing,
            position: std::time::Duration::ZERO,
            duration: std::time::Duration::from_secs(3600),
            volume: 0.5,
            title: None,
        });

        // Up increases volume
        app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        assert!((app.playing.playback.as_ref().unwrap().volume - 0.6).abs() < 0.01);

        // Down decreases volume
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        assert!((app.playing.playback.as_ref().unwrap().volume - 0.5).abs() < 0.01);
    }
}
