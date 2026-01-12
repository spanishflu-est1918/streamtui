//! App state and core application logic
//!
//! Manages the application state machine, navigation stack,
//! and coordinates between UI and backend services.

use crate::models::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// =============================================================================
// App State Enum
// =============================================================================

/// Application state enum representing current screen
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Home screen with search box and trending content
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

impl Default for AppState {
    fn default() -> Self {
        AppState::Home
    }
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
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    /// Idle - no loading in progress
    Idle,
    /// Loading with optional message
    Loading(Option<String>),
    /// Error with message
    Error(String),
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState::Idle
    }
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
    /// Trending content list
    pub list: ListState,
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
        self.cursor += 1;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.query.remove(self.cursor);
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
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor < self.query.len() {
            self.cursor += 1;
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
}

// =============================================================================
// Main Application State
// =============================================================================

/// Main application state
#[derive(Debug)]
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
}

impl Default for App {
    fn default() -> Self {
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

            cast_devices: Vec::new(),
            selected_device: None,
        }
    }
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        Self::default()
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
            self.state = prev;
            true
        } else {
            false
        }
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
            self.input_mode = InputMode::Editing;
            if self.state == AppState::Home {
                self.navigate(AppState::Search);
            }
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

        // Route to appropriate handler based on mode and state
        if self.input_mode == InputMode::Editing {
            self.handle_editing_key(key)
        } else {
            self.handle_normal_key(key)
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
                // Signal that search should be triggered
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
            KeyCode::Char('/') | KeyCode::Char('s') => {
                self.focus_search();
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
            KeyCode::Enter => {
                // Open detail view for selected trending item
                true
            }
            KeyCode::Char('i') => {
                // Show info/detail
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
            KeyCode::Enter => {
                // Open detail view for selected result
                true
            }
            KeyCode::Char('i') => {
                // Show info/detail
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
                // Navigate within detail view (e.g., seasons/episodes)
                if let Some(DetailState::Tv { season_list, .. }) = &mut self.detail {
                    season_list.up();
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(DetailState::Tv { season_list, .. }) = &mut self.detail {
                    season_list.down();
                }
                true
            }
            KeyCode::Enter | KeyCode::Char('c') => {
                // Go to sources or start cast
                self.navigate(AppState::Sources);
                true
            }
            KeyCode::Char('u') => {
                // Go to subtitles
                self.navigate(AppState::Subtitles);
                true
            }
            KeyCode::Tab => {
                // Switch between seasons/episodes panel
                true
            }
            _ => false,
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
                self.navigate(AppState::Playing);
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
                // Go to subtitles first
                self.navigate(AppState::Subtitles);
                true
            }
            _ => false,
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
            KeyCode::Enter => {
                // Select subtitle and continue
                if let Some(sub) = self.subtitles.selected_subtitle() {
                    self.subtitles.selected = Some(sub.clone());
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
        match key.code {
            KeyCode::Char(' ') => {
                // Toggle pause
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
                if let Some(ref mut playback) = self.playing.playback {
                    playback.state = CastState::Stopped;
                }
                true
            }
            KeyCode::Left => {
                // Seek backward
                true
            }
            KeyCode::Right => {
                // Seek forward
                true
            }
            KeyCode::Up => {
                // Volume up
                if let Some(ref mut playback) = self.playing.playback {
                    playback.volume = (playback.volume + 0.1).min(1.0);
                }
                true
            }
            KeyCode::Down => {
                // Volume down
                if let Some(ref mut playback) = self.playing.playback {
                    playback.volume = (playback.volume - 0.1).max(0.0);
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
        app.playing.playback = Some(PlaybackStatus {
            state: CastState::Playing,
            position: std::time::Duration::from_secs(100),
            duration: std::time::Duration::from_secs(3600),
            volume: 0.8,
            title: Some("Test".into()),
        });

        // Space toggles pause
        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert_eq!(app.playing.playback.as_ref().unwrap().state, CastState::Paused);

        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert_eq!(app.playing.playback.as_ref().unwrap().state, CastState::Playing);
    }

    #[test]
    fn test_playing_volume() {
        let mut app = App::new();
        app.state = AppState::Playing;
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
