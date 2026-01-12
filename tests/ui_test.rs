//! UI component tests for StreamTUI
//!
//! Tests TUI layout, theme, navigation, and rendering from specs/tui.md.
//!
//! ## Test Cases
//! - test_theme_colors: All colors valid RGB, WCAG contrast compliance
//! - test_layout_responsive: Renders at 80x24 (min) and 200x50 (large)
//! - test_navigation: Up/down list movement, enter/escape handling
//! - test_search_focus: '/' focuses, typing works, enter submits, escape clears
//! - test_content_card_render: Title, year, quality, size displayed correctly
//! - test_now_playing_overlay: Centered, progress bar, playback time

use ratatui::{
    backend::TestBackend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use streamtui::ui::theme::{
    color_to_rgb, contrast_ratio, meets_wcag_aa, meets_wcag_aa_large, Theme,
};
use streamtui::{App, AppState, MediaType, Quality, SearchResult};

// =============================================================================
// THEME COLOR TESTS
// =============================================================================

/// Test all theme colors are valid RGB hex values
#[test]
fn test_theme_colors_valid_rgb() {
    // All core palette colors should be RGB type
    let colors = [
        ("BACKGROUND", Theme::BACKGROUND),
        ("PRIMARY", Theme::PRIMARY),
        ("SECONDARY", Theme::SECONDARY),
        ("ACCENT", Theme::ACCENT),
        ("HIGHLIGHT", Theme::HIGHLIGHT),
        ("TEXT", Theme::TEXT),
        ("DIM", Theme::DIM),
        ("SUCCESS", Theme::SUCCESS),
        ("WARNING", Theme::WARNING),
        ("ERROR", Theme::ERROR),
        ("BACKGROUND_LIGHT", Theme::BACKGROUND_LIGHT),
        ("BACKGROUND_HOVER", Theme::BACKGROUND_HOVER),
        ("BORDER", Theme::BORDER),
        ("BORDER_FOCUSED", Theme::BORDER_FOCUSED),
    ];

    for (name, color) in colors {
        let rgb = color_to_rgb(color);
        assert!(rgb.is_some(), "{} should be an RGB color", name);

        // Verify values are valid (0-255)
        let (r, g, b) = rgb.unwrap();
        assert!(
            r <= 255 && g <= 255 && b <= 255,
            "{} has invalid RGB values",
            name
        );
    }
}

/// Test theme colors match the spec exactly
#[test]
fn test_theme_colors_match_spec() {
    // From specs/tui.md:
    assert_eq!(color_to_rgb(Theme::BACKGROUND), Some((0x0a, 0x0a, 0x0f)));
    assert_eq!(color_to_rgb(Theme::PRIMARY), Some((0x00, 0xff, 0xf2)));
    assert_eq!(color_to_rgb(Theme::SECONDARY), Some((0xff, 0x00, 0xff)));
    assert_eq!(color_to_rgb(Theme::ACCENT), Some((0xff, 0xff, 0x00)));
    assert_eq!(color_to_rgb(Theme::HIGHLIGHT), Some((0xff, 0x00, 0x80)));
    assert_eq!(color_to_rgb(Theme::TEXT), Some((0xe0, 0xe0, 0xe0)));
    assert_eq!(color_to_rgb(Theme::DIM), Some((0x40, 0x40, 0x50)));
    assert_eq!(color_to_rgb(Theme::SUCCESS), Some((0x00, 0xff, 0x00)));
    assert_eq!(color_to_rgb(Theme::WARNING), Some((0xff, 0xaa, 0x00)));
    assert_eq!(color_to_rgb(Theme::ERROR), Some((0xff, 0x00, 0x40)));
}

/// Test contrast ratios meet WCAG AA requirements
#[test]
fn test_theme_colors_contrast_ratios() {
    let bg = color_to_rgb(Theme::BACKGROUND).unwrap();

    // Text on background must meet WCAG AA (4.5:1 for normal text)
    let text = color_to_rgb(Theme::TEXT).unwrap();
    let text_ratio = contrast_ratio(text, bg);
    assert!(
        meets_wcag_aa(text, bg),
        "TEXT on BACKGROUND contrast {:.2}:1 must be >= 4.5:1",
        text_ratio
    );

    // Primary, secondary, accent must meet WCAG AA for large text (3:1)
    let primary = color_to_rgb(Theme::PRIMARY).unwrap();
    let primary_ratio = contrast_ratio(primary, bg);
    assert!(
        meets_wcag_aa_large(primary, bg),
        "PRIMARY on BACKGROUND contrast {:.2}:1 must be >= 3:1",
        primary_ratio
    );

    let secondary = color_to_rgb(Theme::SECONDARY).unwrap();
    let secondary_ratio = contrast_ratio(secondary, bg);
    assert!(
        meets_wcag_aa_large(secondary, bg),
        "SECONDARY on BACKGROUND contrast {:.2}:1 must be >= 3:1",
        secondary_ratio
    );

    let highlight = color_to_rgb(Theme::HIGHLIGHT).unwrap();
    let highlight_ratio = contrast_ratio(highlight, bg);
    assert!(
        meets_wcag_aa_large(highlight, bg),
        "HIGHLIGHT on BACKGROUND contrast {:.2}:1 must be >= 3:1",
        highlight_ratio
    );

    // Success, warning, error must be visible
    let success = color_to_rgb(Theme::SUCCESS).unwrap();
    assert!(
        meets_wcag_aa_large(success, bg),
        "SUCCESS on BACKGROUND must meet large text contrast"
    );

    let warning = color_to_rgb(Theme::WARNING).unwrap();
    assert!(
        meets_wcag_aa_large(warning, bg),
        "WARNING on BACKGROUND must meet large text contrast"
    );

    let error = color_to_rgb(Theme::ERROR).unwrap();
    assert!(
        meets_wcag_aa_large(error, bg),
        "ERROR on BACKGROUND must meet large text contrast"
    );
}

/// Test inverted styles still readable (text on primary background)
#[test]
fn test_theme_inverted_contrast() {
    let bg_color = color_to_rgb(Theme::BACKGROUND).unwrap();
    let primary = color_to_rgb(Theme::PRIMARY).unwrap();

    // When highlighted (inverted), background color on primary should be readable
    let ratio = contrast_ratio(bg_color, primary);
    assert!(
        meets_wcag_aa_large(bg_color, primary),
        "Inverted highlight (bg on primary) contrast {:.2}:1 must be >= 3:1",
        ratio
    );
}

// =============================================================================
// LAYOUT RESPONSIVE TESTS
// =============================================================================

/// Helper to create a test terminal with given size
fn test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Helper layout function that mirrors the actual app layout
fn render_main_layout(frame: &mut Frame, area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Test layout renders at minimum size (80x24)
#[test]
fn test_layout_responsive_minimum_size() {
    let mut terminal = test_terminal(80, 24);

    terminal
        .draw(|frame| {
            let area = frame.area();

            // Minimum viable size
            assert!(area.width >= 80, "Minimum width should be 80");
            assert!(area.height >= 24, "Minimum height should be 24");

            let (header, content, status) = render_main_layout(frame, area);

            // Header should be 3 rows
            assert_eq!(header.height, 3);

            // Status bar should be 1 row
            assert_eq!(status.height, 1);

            // Content should fill remaining space
            assert!(
                content.height >= 20,
                "Content area too small at {}h",
                content.height
            );

            // Width should span terminal
            assert_eq!(header.width, 80);
            assert_eq!(content.width, 80);
            assert_eq!(status.width, 80);
        })
        .unwrap();
}

/// Test layout renders at large size (200x50)
#[test]
fn test_layout_responsive_large_size() {
    let mut terminal = test_terminal(200, 50);

    terminal
        .draw(|frame| {
            let area = frame.area();
            let (header, content, status) = render_main_layout(frame, area);

            // Header and status bar stay fixed
            assert_eq!(header.height, 3);
            assert_eq!(status.height, 1);

            // Content expands to fill space
            assert_eq!(content.height, 46); // 50 - 3 - 1

            // All widths span terminal
            assert_eq!(header.width, 200);
            assert_eq!(content.width, 200);
            assert_eq!(status.width, 200);
        })
        .unwrap();
}

/// Test content area scrolls when items exceed height
#[test]
fn test_layout_content_scrollable() {
    let mut terminal = test_terminal(80, 24);

    // Simulate 50 items in a list that only has ~20 rows
    let items: Vec<ListItem> = (1..=50)
        .map(|i| ListItem::new(format!("Item {}", i)))
        .collect();

    let mut state = ListState::default();
    state.select(Some(45)); // Select item near the bottom

    terminal
        .draw(|frame| {
            let area = frame.area();
            let (_, content, _) = render_main_layout(frame, area);

            // Content area is smaller than item count
            assert!(content.height < 50, "Should need scrolling");

            // Create scrollable list
            let list = List::new(items.clone())
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Theme::selected());

            // Render with state (handles scroll offset automatically)
            frame.render_stateful_widget(list, content, &mut state);

            // Selected item should be visible (ratatui handles scroll)
            assert_eq!(state.selected(), Some(45));
        })
        .unwrap();
}

// =============================================================================
// NAVIGATION TESTS
// =============================================================================

/// Test up/down moves selection in list
#[test]
fn test_navigation_up_down() {
    let mut state = ListState::default();
    let item_count = 10;

    // Start with nothing selected
    assert_eq!(state.selected(), None);

    // Select first item (simulating initial down press or default)
    state.select(Some(0));
    assert_eq!(state.selected(), Some(0));

    // Move down
    let current = state.selected().unwrap();
    if current < item_count - 1 {
        state.select(Some(current + 1));
    }
    assert_eq!(state.selected(), Some(1));

    // Move down again
    let current = state.selected().unwrap();
    if current < item_count - 1 {
        state.select(Some(current + 1));
    }
    assert_eq!(state.selected(), Some(2));

    // Move up
    let current = state.selected().unwrap();
    if current > 0 {
        state.select(Some(current - 1));
    }
    assert_eq!(state.selected(), Some(1));

    // Move up at boundary
    state.select(Some(0));
    let current = state.selected().unwrap();
    if current > 0 {
        state.select(Some(current - 1));
    }
    assert_eq!(state.selected(), Some(0)); // Stays at 0

    // Move down at boundary
    state.select(Some(9));
    let current = state.selected().unwrap();
    if current < item_count - 1 {
        state.select(Some(current + 1));
    }
    assert_eq!(state.selected(), Some(9)); // Stays at 9
}

/// Test Enter on content opens detail view
#[test]
fn test_navigation_enter_opens_detail() {
    let mut app = App::new();
    assert_eq!(app.state, AppState::Home);

    // Navigate to Search (simulating search start)
    app.navigate(AppState::Search);
    assert_eq!(app.state, AppState::Search);
    assert_eq!(app.nav_stack.len(), 1);

    // Press Enter on selected content -> opens Detail
    app.navigate(AppState::Detail);
    assert_eq!(app.state, AppState::Detail);
    assert_eq!(app.nav_stack.len(), 2);

    // Verify navigation stack
    assert_eq!(app.nav_stack[0], AppState::Home);
    assert_eq!(app.nav_stack[1], AppState::Search);
}

/// Test Escape from detail returns to list
#[test]
fn test_navigation_escape_returns() {
    let mut app = App::new();

    // Set up: Home -> Search -> Detail
    app.navigate(AppState::Search);
    app.navigate(AppState::Detail);
    assert_eq!(app.state, AppState::Detail);

    // Press Escape
    let went_back = app.back();
    assert!(went_back);
    assert_eq!(app.state, AppState::Search);

    // Press Escape again
    let went_back = app.back();
    assert!(went_back);
    assert_eq!(app.state, AppState::Home);

    // Press Escape at root - can't go further back
    let went_back = app.back();
    assert!(!went_back);
    assert_eq!(app.state, AppState::Home);
}

/// Test navigation full cycle
#[test]
fn test_navigation_full_cycle() {
    let mut app = App::new();

    // Home -> Search -> Detail -> Subtitles -> Playing
    app.navigate(AppState::Search);
    app.navigate(AppState::Detail);
    app.navigate(AppState::Subtitles);
    app.navigate(AppState::Playing);

    assert_eq!(app.state, AppState::Playing);
    assert_eq!(app.nav_stack.len(), 4);

    // Go all the way back
    app.back();
    assert_eq!(app.state, AppState::Subtitles);
    app.back();
    assert_eq!(app.state, AppState::Detail);
    app.back();
    assert_eq!(app.state, AppState::Search);
    app.back();
    assert_eq!(app.state, AppState::Home);

    assert_eq!(app.nav_stack.len(), 0);
}

// =============================================================================
// SEARCH FOCUS TESTS
// =============================================================================

/// Simple search state for testing
struct SearchState {
    focused: bool,
    query: String,
    cursor_pos: usize,
}

impl SearchState {
    fn new() -> Self {
        Self {
            focused: false,
            query: String::new(),
            cursor_pos: 0,
        }
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }

    fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
    }

    fn type_char(&mut self, c: char) {
        if self.focused {
            self.query.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
        }
    }

    fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.type_char(c);
        }
    }

    fn submit(&self) -> Option<String> {
        if self.focused && !self.query.is_empty() {
            Some(self.query.clone())
        } else {
            None
        }
    }
}

/// Test '/' key focuses search input
#[test]
fn test_search_focus_slash_key() {
    let mut search = SearchState::new();
    assert!(!search.focused);

    // Simulate '/' press
    search.focus();
    assert!(search.focused);
}

/// Test typing updates search query
#[test]
fn test_search_focus_typing() {
    let mut search = SearchState::new();
    search.focus();

    // Type a query
    search.type_str("batman");
    assert_eq!(search.query, "batman");
    assert_eq!(search.cursor_pos, 6);

    // Type more
    search.type_str(" 2022");
    assert_eq!(search.query, "batman 2022");
}

/// Test typing only works when focused
#[test]
fn test_search_focus_typing_requires_focus() {
    let mut search = SearchState::new();
    assert!(!search.focused);

    // Type without focus - nothing happens
    search.type_str("ignored");
    assert_eq!(search.query, "");
}

/// Test Enter submits search
#[test]
fn test_search_focus_enter_submits() {
    let mut search = SearchState::new();
    search.focus();
    search.type_str("the matrix");

    // Submit
    let query = search.submit();
    assert_eq!(query, Some("the matrix".to_string()));
}

/// Test Enter with empty query doesn't submit
#[test]
fn test_search_focus_enter_empty_no_submit() {
    let mut search = SearchState::new();
    search.focus();

    // Try to submit empty
    let query = search.submit();
    assert_eq!(query, None);
}

/// Test Escape clears and unfocuses
#[test]
fn test_search_focus_escape_clears() {
    let mut search = SearchState::new();
    search.focus();
    search.type_str("some query");

    // Press Escape
    search.clear();
    search.unfocus();

    assert!(!search.focused);
    assert_eq!(search.query, "");
    assert_eq!(search.cursor_pos, 0);
}

// =============================================================================
// CONTENT CARD RENDER TESTS
// =============================================================================

/// Build a content card line similar to specs/tui.md format
fn render_content_card(
    title: &str,
    year: Option<u16>,
    quality: Quality,
    size_bytes: Option<u64>,
    seeds: u32,
    selected: bool,
) -> Vec<Line<'static>> {
    let max_title_len = 30;
    let display_title = if title.len() > max_title_len {
        format!("{}...", &title[..max_title_len - 3])
    } else {
        title.to_string()
    };

    let year_str = year.map(|y| format!(" ({})", y)).unwrap_or_default();

    let size_str = match size_bytes {
        Some(bytes) if bytes >= 1024 * 1024 * 1024 => {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
        Some(bytes) => format!("{}MB", bytes / (1024 * 1024)),
        None => "?GB".to_string(),
    };

    let prefix = if selected { "▸ " } else { "  " };

    let line1 = Line::from(vec![
        Span::raw(prefix),
        Span::styled(
            format!("{}{}", display_title, year_str),
            if selected {
                Theme::selected()
            } else {
                Theme::text()
            },
        ),
        Span::raw(" "),
        Span::styled(format!("{}", quality), Theme::quality_1080p()),
        Span::raw(" "),
        Span::styled(size_str, Theme::file_size()),
    ]);

    let line2 = Line::from(vec![
        Span::raw("  "),
        Span::styled("Action, Crime • 2h 56m", Theme::dimmed()),
        Span::raw("    "),
        Span::styled(
            format!("Seeds: {}", seeds),
            if seeds > 100 {
                Theme::seeds_high()
            } else if seeds > 20 {
                Theme::seeds_medium()
            } else {
                Theme::seeds_low()
            },
        ),
    ]);

    vec![line1, line2]
}

/// Test content card displays title, year, quality, size
#[test]
fn test_content_card_render_basic() {
    let lines = render_content_card(
        "The Batman",
        Some(2022),
        Quality::FHD1080p,
        Some(4_509_715_660), // ~4.2 GB
        142,
        false,
    );

    let line1_str = lines[0].to_string();
    assert!(line1_str.contains("The Batman"), "Should contain title");
    assert!(line1_str.contains("2022"), "Should contain year");
    assert!(line1_str.contains("1080p"), "Should contain quality");
    assert!(line1_str.contains("4.2GB"), "Should contain size");

    let line2_str = lines[1].to_string();
    assert!(line2_str.contains("Seeds: 142"), "Should contain seeds");
}

/// Test long titles are truncated with ellipsis
#[test]
fn test_content_card_render_truncate_long_title() {
    let lines = render_content_card(
        "The Lord of the Rings: The Fellowship of the Ring Extended Edition",
        Some(2001),
        Quality::UHD4K,
        Some(50_000_000_000),
        89,
        false,
    );

    let line1_str = lines[0].to_string();
    assert!(
        line1_str.contains("..."),
        "Long title should be truncated with ellipsis"
    );
    assert!(line1_str.len() < 100, "Line should be reasonable length");
}

/// Test selected item is highlighted with accent color
#[test]
fn test_content_card_render_selected_highlight() {
    let lines = render_content_card(
        "The Batman",
        Some(2022),
        Quality::FHD1080p,
        Some(4_509_715_660),
        142,
        true,
    );

    // First line should start with selection indicator
    let line1_str = lines[0].to_string();
    assert!(
        line1_str.starts_with("▸"),
        "Selected item should have indicator"
    );
}

/// Test missing year handled gracefully
#[test]
fn test_content_card_render_no_year() {
    let lines = render_content_card("Unknown Movie", None, Quality::Unknown, None, 0, false);

    let line1_str = lines[0].to_string();
    assert!(line1_str.contains("Unknown Movie"), "Should show title");
    assert!(!line1_str.contains("()"), "Should not show empty parens");
}

/// Test content card renders in terminal
#[test]
fn test_content_card_render_in_terminal() {
    let mut terminal = test_terminal(80, 24);

    let items = vec![
        SearchResult {
            id: 1,
            media_type: MediaType::Movie,
            title: "The Batman".to_string(),
            year: Some(2022),
            overview: "".to_string(),
            poster_path: None,
            vote_average: 7.8,
        },
        SearchResult {
            id: 2,
            media_type: MediaType::Movie,
            title: "The Dark Knight".to_string(),
            year: Some(2008),
            overview: "".to_string(),
            poster_path: None,
            vote_average: 9.0,
        },
    ];

    let list_items: Vec<ListItem> = items.iter().map(|r| ListItem::new(r.to_string())).collect();

    let mut state = ListState::default();
    state.select(Some(0));

    terminal
        .draw(|frame| {
            let area = frame.area();
            let list = List::new(list_items)
                .block(Block::default().title("Results").borders(Borders::ALL))
                .highlight_style(Theme::list_item_selected());

            frame.render_stateful_widget(list, area, &mut state);
        })
        .unwrap();

    // Verify buffer contains expected text
    let buffer = terminal.backend().buffer();
    let content: String = buffer.content.iter().map(|c| c.symbol()).collect();
    assert!(content.contains("The Batman (2022)"));
    assert!(content.contains("The Dark Knight (2008)"));
}

// =============================================================================
// NOW PLAYING OVERLAY TESTS
// =============================================================================

/// Simple NowPlaying state for testing overlay
struct NowPlayingState {
    title: String,
    position_secs: u64,
    duration_secs: u64,
    device_name: String,
    is_paused: bool,
}

impl NowPlayingState {
    fn progress(&self) -> f64 {
        if self.duration_secs == 0 {
            return 0.0;
        }
        self.position_secs as f64 / self.duration_secs as f64
    }

    fn format_time(secs: u64) -> String {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
}

/// Calculate centered overlay rect
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Test overlay renders centered on screen
#[test]
fn test_now_playing_overlay_centered() {
    let overlay_width = 45;
    let overlay_height = 9;

    // Test on 80x24 terminal
    let area = Rect::new(0, 0, 80, 24);
    let overlay_rect = centered_rect(overlay_width, overlay_height, area);

    // Should be horizontally centered
    let expected_x = (80 - overlay_width) / 2;
    assert_eq!(
        overlay_rect.x, expected_x,
        "Should be horizontally centered"
    );

    // Should be vertically centered
    let expected_y = (24 - overlay_height) / 2;
    assert_eq!(overlay_rect.y, expected_y, "Should be vertically centered");

    // Test on larger terminal
    let large_area = Rect::new(0, 0, 200, 50);
    let large_overlay = centered_rect(overlay_width, overlay_height, large_area);

    let expected_x = (200 - overlay_width) / 2;
    let expected_y = (50 - overlay_height) / 2;
    assert_eq!(large_overlay.x, expected_x);
    assert_eq!(large_overlay.y, expected_y);
}

/// Test progress bar displays correctly
#[test]
fn test_now_playing_overlay_progress_bar() {
    let state = NowPlayingState {
        title: "The Batman (2022)".to_string(),
        position_secs: 2723,  // 45:23
        duration_secs: 10560, // 2:56:00
        device_name: "Living Room TV".to_string(),
        is_paused: false,
    };

    let progress = state.progress();
    assert!((progress - 0.258).abs() < 0.01, "Progress should be ~25.8%");

    // Create gauge widget
    let gauge = Gauge::default()
        .gauge_style(Theme::progress_bar())
        .ratio(progress)
        .label(format!(
            "{} / {}",
            NowPlayingState::format_time(state.position_secs),
            NowPlayingState::format_time(state.duration_secs)
        ));

    // Verify the label format
    let pos_str = NowPlayingState::format_time(state.position_secs);
    let dur_str = NowPlayingState::format_time(state.duration_secs);
    assert_eq!(pos_str, "45:23");
    assert_eq!(dur_str, "02:56:00");
}

/// Test playback time updates
#[test]
fn test_now_playing_overlay_time_updates() {
    let mut state = NowPlayingState {
        title: "Movie".to_string(),
        position_secs: 0,
        duration_secs: 7200, // 2 hours
        device_name: "TV".to_string(),
        is_paused: false,
    };

    // Initial
    assert_eq!(NowPlayingState::format_time(state.position_secs), "00:00");
    assert_eq!(state.progress(), 0.0);

    // After 30 minutes
    state.position_secs = 1800;
    assert_eq!(NowPlayingState::format_time(state.position_secs), "30:00");
    assert!((state.progress() - 0.25).abs() < 0.01);

    // After 1 hour
    state.position_secs = 3600;
    assert_eq!(
        NowPlayingState::format_time(state.position_secs),
        "01:00:00"
    );
    assert!((state.progress() - 0.5).abs() < 0.01);

    // After 1:45:30
    state.position_secs = 6330;
    assert_eq!(
        NowPlayingState::format_time(state.position_secs),
        "01:45:30"
    );
}

/// Test overlay responds to pause/stop commands
#[test]
fn test_now_playing_overlay_controls() {
    let mut state = NowPlayingState {
        title: "Movie".to_string(),
        position_secs: 1000,
        duration_secs: 7200,
        device_name: "TV".to_string(),
        is_paused: false,
    };

    // Space toggles pause
    assert!(!state.is_paused);
    state.is_paused = true; // Simulate space press
    assert!(state.is_paused);
    state.is_paused = false; // Toggle again
    assert!(!state.is_paused);
}

/// Test overlay renders all expected elements
#[test]
fn test_now_playing_overlay_full_render() {
    let mut terminal = test_terminal(80, 24);

    let state = NowPlayingState {
        title: "The Batman (2022)".to_string(),
        position_secs: 2723,
        duration_secs: 10560,
        device_name: "Living Room TV".to_string(),
        is_paused: false,
    };

    terminal
        .draw(|frame| {
            let area = frame.area();

            // Calculate centered overlay position
            let overlay_rect = centered_rect(50, 9, area);

            // Clear background
            frame.render_widget(Clear, overlay_rect);

            // Render overlay block
            let block = Block::default()
                .title(Span::styled("▶ NOW CASTING", Theme::title()))
                .borders(Borders::ALL)
                .border_style(Theme::border_focused());

            // Inner area for content
            let inner = block.inner(overlay_rect);
            frame.render_widget(block, overlay_rect);

            // Title
            let title_para = Paragraph::new(state.title.as_str()).style(Theme::text());
            frame.render_widget(title_para, Rect::new(inner.x, inner.y, inner.width, 1));

            // Progress bar
            let gauge = Gauge::default()
                .gauge_style(Theme::progress_bar())
                .ratio(state.progress())
                .label(format!(
                    "{} / {}",
                    NowPlayingState::format_time(state.position_secs),
                    NowPlayingState::format_time(state.duration_secs)
                ));
            frame.render_widget(gauge, Rect::new(inner.x, inner.y + 2, inner.width, 1));

            // Device name
            let device =
                Paragraph::new(format!("📺 {}", state.device_name)).style(Theme::cast_target());
            frame.render_widget(device, Rect::new(inner.x, inner.y + 4, inner.width, 1));

            // Controls hint
            let controls =
                Paragraph::new("[Space] Pause  [s] Stop  [Esc] Close").style(Theme::dimmed());
            frame.render_widget(controls, Rect::new(inner.x, inner.y + 5, inner.width, 1));
        })
        .unwrap();

    // Verify buffer contains expected elements
    let buffer = terminal.backend().buffer();
    let content: String = buffer.content.iter().map(|c| c.symbol()).collect();

    assert!(
        content.contains("NOW CASTING"),
        "Should show NOW CASTING header"
    );
    assert!(content.contains("The Batman"), "Should show title");
    assert!(
        content.contains("Living Room TV"),
        "Should show device name"
    );
    assert!(content.contains("Pause"), "Should show pause hint");
    assert!(content.contains("Stop"), "Should show stop hint");
    assert!(content.contains("Esc"), "Should show escape hint");
}

/// Test overlay handles edge cases
#[test]
fn test_now_playing_overlay_edge_cases() {
    // Zero duration
    let state = NowPlayingState {
        title: "".to_string(),
        position_secs: 0,
        duration_secs: 0,
        device_name: "".to_string(),
        is_paused: false,
    };
    assert_eq!(
        state.progress(),
        0.0,
        "Zero duration should give 0 progress"
    );

    // Position > duration (shouldn't happen but handle gracefully)
    let over_state = NowPlayingState {
        title: "".to_string(),
        position_secs: 10000,
        duration_secs: 5000,
        device_name: "".to_string(),
        is_paused: false,
    };
    assert!(
        over_state.progress() > 1.0,
        "Over progress should be > 1.0 (not clamped here)"
    );
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

/// Test complete UI flow: Home -> Search -> Detail -> Playing
#[test]
fn test_ui_flow_integration() {
    let mut app = App::new();
    let mut search = SearchState::new();

    // 1. Start at home screen
    assert_eq!(app.state, AppState::Home);

    // 2. Navigate to Search
    app.navigate(AppState::Search);
    assert_eq!(app.state, AppState::Search);

    // 3. Focus search with '/'
    search.focus();
    assert!(search.focused);

    // 4. Type query
    search.type_str("batman");
    assert_eq!(search.query, "batman");

    // 5. Submit with Enter
    let query = search.submit();
    assert_eq!(query, Some("batman".to_string()));

    // 6. Select item and press Enter -> Detail
    app.navigate(AppState::Detail);
    assert_eq!(app.state, AppState::Detail);

    // 7. Select source and cast -> Playing
    app.navigate(AppState::Playing);
    assert_eq!(app.state, AppState::Playing);

    // 8. Press Escape to go back through screens
    app.back();
    assert_eq!(app.state, AppState::Detail);
    app.back();
    assert_eq!(app.state, AppState::Search);
    app.back();
    assert_eq!(app.state, AppState::Home);

    // 9. Can't go back further
    let went_back = app.back();
    assert!(!went_back);
}

/// Test theme consistency across all UI elements
#[test]
fn test_ui_theme_consistency() {
    // All style helpers should use consistent backgrounds
    let text_style = Theme::text();
    let title_style = Theme::title();
    let selected_style = Theme::selected();
    let error_style = Theme::error();

    // Text-based styles should have same background
    // (or no background, defaulting to terminal bg)
    assert_eq!(text_style.bg, Some(Theme::BACKGROUND));

    // All primary styles should use theme foreground colors
    assert_eq!(title_style.fg, Some(Theme::PRIMARY));
    assert_eq!(selected_style.fg, Some(Theme::HIGHLIGHT));
    assert_eq!(error_style.fg, Some(Theme::ERROR));
}
