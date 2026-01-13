//! Search view component
//!
//! Search input with live results and neon cyberpunk styling.
//! Keyboard-first: typing updates query, Enter submits, Esc clears.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::models::SearchResult;
use crate::ui::Theme;

/// Search view state
#[derive(Debug, Default)]
pub struct SearchView {
    /// Current search query
    pub query: String,
    /// Cursor position in query
    pub cursor: usize,
    /// Whether search input is focused
    pub focused: bool,
    /// Search results to display
    pub results: Vec<SearchResult>,
    /// Trending content to display when search is empty
    pub trending: Vec<SearchResult>,
    /// Currently selected result index
    pub selected: usize,
    /// Whether a search is in progress
    pub loading: bool,
    /// Whether trending is loading
    pub trending_loading: bool,
    /// Error message if search failed
    pub error: Option<String>,
}

impl SearchView {
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Input Handling
    // =========================================================================

    /// Handle character input
    pub fn input(&mut self, c: char) {
        self.query.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Handle backspace
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.query.remove(self.cursor);
        }
    }

    /// Delete character at cursor position
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

    /// Clear the search query
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.error = None;
    }

    /// Get the current query for submission
    pub fn submit(&self) -> Option<&str> {
        if self.query.trim().is_empty() {
            None
        } else {
            Some(&self.query)
        }
    }

    // =========================================================================
    // Results Navigation
    // =========================================================================

    /// Set search results
    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        self.selected = 0;
        self.loading = false;
        self.error = None;
    }

    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    /// Set trending results
    pub fn set_trending(&mut self, trending: Vec<SearchResult>) {
        self.trending = trending;
        self.trending_loading = false;
        // If showing trending (no search results), reset selection
        if self.results.is_empty() {
            self.selected = 0;
        }
    }

    /// Check if we're showing trending (no search active)
    pub fn showing_trending(&self) -> bool {
        self.query.is_empty() && self.results.is_empty() && !self.trending.is_empty()
    }

    /// Get the currently active list (search results or trending)
    pub fn active_list(&self) -> &[SearchResult] {
        if self.results.is_empty() && self.query.is_empty() {
            &self.trending
        } else {
            &self.results
        }
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        let list_len = self.active_list().len();
        if self.selected < list_len.saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Get currently selected result
    pub fn current(&self) -> Option<&SearchResult> {
        self.active_list().get(self.selected)
    }

    /// Check if we have results to navigate
    pub fn has_results(&self) -> bool {
        !self.active_list().is_empty()
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Render the search view
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Split into search box (top) and results (bottom)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search box
                Constraint::Min(1),    // Results
            ])
            .split(area);

        self.render_search_box(frame, chunks[0]);
        self.render_results(frame, chunks[1]);
    }

    /// Render the search input box with neon styling
    fn render_search_box(&self, frame: &mut Frame, area: Rect) {
        // Build the query text with cursor indicator
        let display_query = if self.focused {
            // Insert cursor character at position
            let (before, after) = self.query.split_at(self.cursor.min(self.query.len()));
            format!("{}│{}", before, after)
        } else {
            self.query.clone()
        };

        // Neon-styled search prompt
        let prompt = if self.loading { "⟳ " } else { "⌕ " };

        let text = format!("{}{}", prompt, display_query);

        // Select border style based on focus
        let border_style = if self.focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let input_style = if self.focused {
            Theme::input().fg(Theme::PRIMARY)
        } else {
            Theme::input()
        };

        let search_box = Paragraph::new(text).style(input_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Span::styled(" SEARCH ", Theme::title()))
                .title_alignment(Alignment::Left),
        );

        frame.render_widget(search_box, area);
    }

    /// Render the results list with neon styling
    fn render_results(&self, frame: &mut Frame, area: Rect) {
        // Handle error state
        if let Some(ref error) = self.error {
            let error_block = Paragraph::new(format!("✗ {}", error))
                .style(Theme::error())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Theme::border())
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .title(Span::styled(" ERROR ", Theme::error())),
                );
            frame.render_widget(error_block, area);
            return;
        }

        // Handle loading state
        if self.loading {
            let loading_block = Paragraph::new("⟳ Searching...")
                .style(Theme::loading())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Theme::border())
                        .border_type(ratatui::widgets::BorderType::Rounded),
                );
            frame.render_widget(loading_block, area);
            return;
        }

        // Handle empty state with trending fallback
        if self.results.is_empty() {
            if self.query.is_empty() && !self.trending.is_empty() {
                // Show trending content
                self.render_list(frame, area, &self.trending, " 🔥 TRENDING ", self.selected);
                return;
            }

            // Show trending loading state
            if self.trending_loading {
                let loading_block = Paragraph::new("⟳ Loading trending...")
                    .style(Theme::loading())
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Theme::border())
                            .border_type(ratatui::widgets::BorderType::Rounded),
                    );
                frame.render_widget(loading_block, area);
                return;
            }

            let hint = if self.query.is_empty() {
                "Type to search for movies and TV shows..."
            } else {
                "No results found"
            };

            let empty_block = Paragraph::new(hint)
                .style(Theme::dimmed())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Theme::border())
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .title(Span::styled(" RESULTS ", Theme::title())),
                );
            frame.render_widget(empty_block, area);
            return;
        }

        // Show search results
        self.render_list(
            frame,
            area,
            &self.results,
            &format!(" RESULTS ({}) ", self.results.len()),
            self.selected,
        );
    }

    /// Render a list of search results with cyberpunk styling
    fn render_list(
        &self,
        frame: &mut Frame,
        area: Rect,
        items: &[SearchResult],
        title: &str,
        selected_idx: usize,
    ) {
        // Build result items with cyberpunk styling
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let is_selected = i == selected_idx;

                // Format: ▸ Title (Year) [Type] ★ Rating
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

        let results_list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Theme::border())
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(Span::styled(title, Theme::title()))
                    .title_alignment(Alignment::Left),
            )
            .style(Theme::text());

        frame.render_widget(results_list, area);
    }

    /// Render as a popup/overlay (centered on screen)
    pub fn render_popup(&self, frame: &mut Frame, area: Rect) {
        // Calculate centered popup area (60% width, 80% height)
        let popup_width = (area.width as f32 * 0.6).min(80.0) as u16;
        let popup_height = (area.height as f32 * 0.8).min(30.0) as u16;

        let popup_area = Rect {
            x: area.x + (area.width.saturating_sub(popup_width)) / 2,
            y: area.y + (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area
        frame.render_widget(Clear, popup_area);

        // Render the search view inside
        self.render(frame, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MediaType;

    fn sample_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                id: 1,
                media_type: MediaType::Movie,
                title: "The Batman".to_string(),
                year: Some(2022),
                overview: "Dark Knight returns".to_string(),
                poster_path: None,
                vote_average: 7.8,
            },
            SearchResult {
                id: 2,
                media_type: MediaType::Tv,
                title: "Breaking Bad".to_string(),
                year: Some(2008),
                overview: "Chemistry teacher".to_string(),
                poster_path: None,
                vote_average: 9.5,
            },
            SearchResult {
                id: 3,
                media_type: MediaType::Movie,
                title: "Dune".to_string(),
                year: Some(2021),
                overview: "Spice must flow".to_string(),
                poster_path: None,
                vote_average: 8.0,
            },
        ]
    }

    #[test]
    fn test_new_search_view() {
        let view = SearchView::new();
        assert!(view.query.is_empty());
        assert_eq!(view.cursor, 0);
        assert!(!view.focused);
        assert!(view.results.is_empty());
        assert!(view.trending.is_empty());
        assert_eq!(view.selected, 0);
        assert!(!view.loading);
        assert!(!view.trending_loading);
        assert!(view.error.is_none());
    }

    #[test]
    fn test_input_adds_characters() {
        let mut view = SearchView::new();
        view.input('h');
        view.input('e');
        view.input('l');
        view.input('l');
        view.input('o');
        assert_eq!(view.query, "hello");
        assert_eq!(view.cursor, 5);
    }

    #[test]
    fn test_backspace_removes_character() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 5;
        view.backspace();
        assert_eq!(view.query, "hell");
        assert_eq!(view.cursor, 4);
    }

    #[test]
    fn test_backspace_at_start_does_nothing() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 0;
        view.backspace();
        assert_eq!(view.query, "hello");
        assert_eq!(view.cursor, 0);
    }

    #[test]
    fn test_delete_removes_character_at_cursor() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 2;
        view.delete();
        assert_eq!(view.query, "helo");
        assert_eq!(view.cursor, 2);
    }

    #[test]
    fn test_delete_at_end_does_nothing() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 5;
        view.delete();
        assert_eq!(view.query, "hello");
    }

    #[test]
    fn test_cursor_movement() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 2;

        view.cursor_left();
        assert_eq!(view.cursor, 1);

        view.cursor_right();
        assert_eq!(view.cursor, 2);

        view.cursor_home();
        assert_eq!(view.cursor, 0);

        view.cursor_end();
        assert_eq!(view.cursor, 5);
    }

    #[test]
    fn test_cursor_left_at_start() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 0;
        view.cursor_left();
        assert_eq!(view.cursor, 0);
    }

    #[test]
    fn test_cursor_right_at_end() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 5;
        view.cursor_right();
        assert_eq!(view.cursor, 5);
    }

    #[test]
    fn test_clear() {
        let mut view = SearchView::new();
        view.query = "hello".to_string();
        view.cursor = 3;
        view.error = Some("test error".to_string());
        view.clear();
        assert!(view.query.is_empty());
        assert_eq!(view.cursor, 0);
        assert!(view.error.is_none());
    }

    #[test]
    fn test_submit_with_query() {
        let mut view = SearchView::new();
        view.query = "batman".to_string();
        assert_eq!(view.submit(), Some("batman"));
    }

    #[test]
    fn test_submit_empty_returns_none() {
        let view = SearchView::new();
        assert!(view.submit().is_none());
    }

    #[test]
    fn test_submit_whitespace_returns_none() {
        let mut view = SearchView::new();
        view.query = "   ".to_string();
        assert!(view.submit().is_none());
    }

    #[test]
    fn test_set_results() {
        let mut view = SearchView::new();
        view.selected = 5; // Set to something non-zero
        view.loading = true;
        view.error = Some("old error".to_string());

        view.set_results(sample_results());

        assert_eq!(view.results.len(), 3);
        assert_eq!(view.selected, 0); // Reset to 0
        assert!(!view.loading);
        assert!(view.error.is_none());
    }

    #[test]
    fn test_set_error() {
        let mut view = SearchView::new();
        view.loading = true;

        view.set_error("Network error".to_string());

        assert_eq!(view.error.as_deref(), Some("Network error"));
        assert!(!view.loading);
    }

    #[test]
    fn test_navigation_up_down() {
        let mut view = SearchView::new();
        view.set_results(sample_results());

        assert_eq!(view.selected, 0);

        view.down();
        assert_eq!(view.selected, 1);

        view.down();
        assert_eq!(view.selected, 2);

        view.down(); // At end, should stay
        assert_eq!(view.selected, 2);

        view.up();
        assert_eq!(view.selected, 1);

        view.up();
        assert_eq!(view.selected, 0);

        view.up(); // At start, should stay
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_current_selected() {
        let mut view = SearchView::new();
        view.set_results(sample_results());

        let current = view.current().unwrap();
        assert_eq!(current.title, "The Batman");

        view.down();
        let current = view.current().unwrap();
        assert_eq!(current.title, "Breaking Bad");
    }

    #[test]
    fn test_current_empty_results() {
        let view = SearchView::new();
        assert!(view.current().is_none());
    }

    #[test]
    fn test_has_results() {
        let mut view = SearchView::new();
        assert!(!view.has_results());

        view.set_results(sample_results());
        assert!(view.has_results());

        view.set_results(vec![]);
        assert!(!view.has_results());
    }

    #[test]
    fn test_input_insert_at_cursor() {
        let mut view = SearchView::new();
        view.query = "hllo".to_string();
        view.cursor = 1;
        view.input('e');
        assert_eq!(view.query, "hello");
        assert_eq!(view.cursor, 2);
    }

    #[test]
    fn test_focus_state() {
        let mut view = SearchView::new();
        assert!(!view.focused);

        view.focused = true;
        assert!(view.focused);
    }

    #[test]
    fn test_loading_state() {
        let mut view = SearchView::new();
        assert!(!view.loading);

        view.loading = true;
        assert!(view.loading);
    }

    // =========================================================================
    // Trending Tests
    // =========================================================================

    #[test]
    fn test_set_trending() {
        let mut view = SearchView::new();
        view.trending_loading = true;
        view.selected = 5;

        view.set_trending(sample_results());

        assert_eq!(view.trending.len(), 3);
        assert!(!view.trending_loading);
        assert_eq!(view.selected, 0); // Reset because showing trending
    }

    #[test]
    fn test_set_trending_with_search_results() {
        let mut view = SearchView::new();
        view.set_results(sample_results());
        view.selected = 2;

        // Set trending while we have search results - selection should not reset
        view.set_trending(sample_results());

        assert_eq!(view.trending.len(), 3);
        assert_eq!(view.selected, 2); // Not reset because we have search results
    }

    #[test]
    fn test_showing_trending() {
        let mut view = SearchView::new();

        // Empty state - no trending
        assert!(!view.showing_trending());

        // With trending, no query, no results
        view.set_trending(sample_results());
        assert!(view.showing_trending());

        // With query - not showing trending
        view.query = "batman".to_string();
        assert!(!view.showing_trending());

        // With results - not showing trending
        view.query.clear();
        view.set_results(sample_results());
        assert!(!view.showing_trending());
    }

    #[test]
    fn test_active_list_shows_trending() {
        let mut view = SearchView::new();
        view.set_trending(sample_results());

        // No query, no results -> show trending
        let active = view.active_list();
        assert_eq!(active.len(), 3);
        assert_eq!(active[0].title, "The Batman");
    }

    #[test]
    fn test_active_list_shows_results() {
        let mut view = SearchView::new();
        view.set_trending(sample_results());

        // Set results - should show results not trending
        view.set_results(vec![SearchResult {
            id: 100,
            media_type: MediaType::Movie,
            title: "Search Result".to_string(),
            year: Some(2023),
            overview: "".to_string(),
            poster_path: None,
            vote_average: 8.0,
        }]);

        let active = view.active_list();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Search Result");
    }

    #[test]
    fn test_active_list_with_query() {
        let mut view = SearchView::new();
        view.set_trending(sample_results());
        view.query = "test".to_string();

        // Query set, but no results yet - show empty (not trending)
        let active = view.active_list();
        assert!(active.is_empty());
    }

    #[test]
    fn test_navigation_with_trending() {
        let mut view = SearchView::new();
        view.set_trending(sample_results());

        assert_eq!(view.selected, 0);
        assert_eq!(view.current().unwrap().title, "The Batman");

        view.down();
        assert_eq!(view.current().unwrap().title, "Breaking Bad");

        view.down();
        assert_eq!(view.current().unwrap().title, "Dune");

        view.down(); // At end
        assert_eq!(view.selected, 2);
    }

    #[test]
    fn test_has_results_with_trending() {
        let mut view = SearchView::new();
        assert!(!view.has_results());

        view.set_trending(sample_results());
        assert!(view.has_results()); // Trending counts as having results
    }

    #[test]
    fn test_trending_loading_state() {
        let mut view = SearchView::new();
        assert!(!view.trending_loading);

        view.trending_loading = true;
        assert!(view.trending_loading);
    }
}
