//! Content browser view
//!
//! Displays search results or trending content in a selectable list.
//! Cyberpunk neon aesthetic with keyboard navigation.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::models::{MediaType, Quality, SearchResult, StreamSource};
use crate::ui::Theme;

/// Trait for items that can be displayed in the browser
pub trait BrowserItem {
    fn title(&self) -> &str;
    fn year(&self) -> Option<u16>;
    fn quality_str(&self) -> Option<String>;
    fn size_str(&self) -> Option<String>;
    fn extra_info(&self) -> Option<String>;
}

impl BrowserItem for SearchResult {
    fn title(&self) -> &str {
        &self.title
    }

    fn year(&self) -> Option<u16> {
        self.year
    }

    fn quality_str(&self) -> Option<String> {
        // SearchResult doesn't have quality, show media type instead
        Some(match self.media_type {
            MediaType::Movie => "MOVIE".to_string(),
            MediaType::Tv => "TV".to_string(),
        })
    }

    fn size_str(&self) -> Option<String> {
        // Show rating instead of size
        Some(format!("★ {:.1}", self.vote_average))
    }

    fn extra_info(&self) -> Option<String> {
        None
    }
}

impl BrowserItem for StreamSource {
    fn title(&self) -> &str {
        &self.title
    }

    fn year(&self) -> Option<u16> {
        None
    }

    fn quality_str(&self) -> Option<String> {
        Some(self.quality.to_string())
    }

    fn size_str(&self) -> Option<String> {
        Some(self.format_size())
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("👤 {}", self.seeds))
    }
}

/// Browser view state
#[derive(Debug, Default)]
pub struct BrowserView {
    /// List of items to display
    pub items: Vec<SearchResult>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset for viewport
    pub offset: usize,
    /// Whether the view is focused
    pub focused: bool,
    /// Title to display in the border
    pub title: String,
}

impl BrowserView {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            offset: 0,
            focused: true,
            title: "BROWSE".to_string(),
        }
    }

    /// Create with custom title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Set items to display
    pub fn set_items(&mut self, items: Vec<SearchResult>) {
        self.items = items;
        self.selected = 0;
        self.offset = 0;
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Adjust offset if selection goes above visible area
            if self.selected < self.offset {
                self.offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Page up - move selection by page
    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    /// Page down - move selection by page
    pub fn page_down(&mut self, page_size: usize) {
        let max_idx = self.items.len().saturating_sub(1);
        self.selected = (self.selected + page_size).min(max_idx);
    }

    /// Jump to start
    pub fn home(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    /// Jump to end
    pub fn end(&mut self) {
        self.selected = self.items.len().saturating_sub(1);
    }

    /// Get currently selected item
    pub fn current(&self) -> Option<&SearchResult> {
        self.items.get(self.selected)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get total item count
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Adjust offset for viewport scrolling
    fn adjust_offset(&mut self, visible_height: usize) {
        // Ensure selected item is visible
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_height {
            self.offset = self.selected.saturating_sub(visible_height.saturating_sub(1));
        }
    }

    /// Render the browser view
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate visible height (minus borders)
        let visible_height = area.height.saturating_sub(2) as usize;
        
        // Adjust offset for scroll
        self.adjust_offset(visible_height);

        // Handle empty state
        if self.items.is_empty() {
            self.render_empty(frame, area);
            return;
        }

        // Build list items with cyberpunk styling
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(self.offset)
            .take(visible_height)
            .map(|(i, item)| self.render_item(i, item))
            .collect();

        // Select border style based on focus
        let border_style = if self.focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let title = format!(" {} ({}/{}) ", self.title, self.selected + 1, self.items.len());

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(Span::styled(title, Theme::title()))
                    .title_alignment(Alignment::Left),
            )
            .style(Theme::text());

        frame.render_widget(list, area);
    }

    /// Render a single item
    fn render_item(&self, index: usize, item: &SearchResult) -> ListItem<'static> {
        let is_selected = index == self.selected;

        // Format: ▸ Title (Year)                    [TYPE] ★ 8.5
        let marker = if is_selected { "▸ " } else { "  " };
        
        let year_str = item.year
            .map(|y| format!(" ({})", y))
            .unwrap_or_default();
        
        let type_str = match item.media_type {
            MediaType::Movie => "MOVIE",
            MediaType::Tv => "TV",
        };

        // Build styled line
        let line = Line::from(vec![
            Span::styled(
                marker.to_string(),
                if is_selected { Theme::accent() } else { Theme::dimmed() }
            ),
            Span::styled(
                item.title.clone(),
                if is_selected { Theme::list_item_selected() } else { Theme::text() }
            ),
            Span::styled(
                year_str,
                if is_selected { Theme::accent() } else { Theme::year() }
            ),
            Span::raw(" "),
            Span::styled(
                format!("[{}]", type_str),
                if is_selected { Theme::accent() } else { Theme::secondary() }
            ),
            Span::raw(" "),
            Span::styled(
                format!("★ {:.1}", item.vote_average),
                Self::rating_style(item.vote_average, is_selected)
            ),
        ]);

        ListItem::new(line)
    }

    /// Get style for rating based on value
    fn rating_style(rating: f32, is_selected: bool) -> Style {
        if is_selected {
            Theme::accent()
        } else if rating >= 7.5 {
            Theme::success()
        } else if rating >= 6.0 {
            Theme::warning()
        } else if rating >= 4.0 {
            Theme::dimmed()
        } else {
            Theme::error()
        }
    }

    /// Render empty state
    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let empty_text = Paragraph::new("No content to display")
            .style(Theme::dimmed())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(Span::styled(format!(" {} ", self.title), Theme::title())),
            );

        frame.render_widget(empty_text, area);
    }
}

/// Browser view for stream sources (quality/size selection)
#[derive(Debug, Default)]
pub struct SourceBrowserView {
    /// List of stream sources
    pub items: Vec<StreamSource>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset
    pub offset: usize,
    /// Whether focused
    pub focused: bool,
}

impl SourceBrowserView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set items to display
    pub fn set_items(&mut self, items: Vec<StreamSource>) {
        self.items = items;
        self.selected = 0;
        self.offset = 0;
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.offset {
                self.offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Get currently selected
    pub fn current(&self) -> Option<&StreamSource> {
        self.items.get(self.selected)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Render the source browser
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let visible_height = area.height.saturating_sub(2) as usize;
        
        // Adjust offset
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_height {
            self.offset = self.selected.saturating_sub(visible_height.saturating_sub(1));
        }

        if self.items.is_empty() {
            let empty = Paragraph::new("No sources available")
                .style(Theme::dimmed())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Theme::border())
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .title(Span::styled(" SOURCES ", Theme::title())),
                );
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(self.offset)
            .take(visible_height)
            .map(|(i, source)| self.render_source_item(i, source))
            .collect();

        let border_style = if self.focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let title = format!(" SOURCES ({}/{}) ", self.selected + 1, self.items.len());

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(Span::styled(title, Theme::title())),
            );

        frame.render_widget(list, area);
    }

    /// Render a single source item
    /// Format: [1] 1080p BluRay x264      4.2 GB  👤142
    fn render_source_item(&self, index: usize, source: &StreamSource) -> ListItem<'static> {
        let is_selected = index == self.selected;
        let hotkey = if index < 9 {
            format!("[{}] ", index + 1)
        } else {
            "    ".to_string()
        };

        let quality_style = Self::quality_style(&source.quality, is_selected);
        let seeds_style = Self::seeds_style(source.seeds, is_selected);

        // Truncate title to first line/reasonable length
        let title_display: String = source.title
            .lines()
            .next()
            .unwrap_or(&source.title)
            .chars()
            .take(40)
            .collect();

        let line = Line::from(vec![
            Span::styled(
                hotkey,
                if is_selected { Theme::accent() } else { Theme::keybind() }
            ),
            Span::styled(
                format!("{:<6}", source.quality.to_string()),
                quality_style
            ),
            Span::styled(
                title_display,
                if is_selected { Theme::list_item_selected() } else { Theme::text() }
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:>8}", source.format_size()),
                if is_selected { Theme::accent() } else { Theme::file_size() }
            ),
            Span::raw(" "),
            Span::styled(
                format!("👤{}", source.seeds),
                seeds_style
            ),
        ]);

        ListItem::new(line)
    }

    /// Style for quality badge
    fn quality_style(quality: &Quality, is_selected: bool) -> Style {
        if is_selected {
            return Theme::accent();
        }
        match quality {
            Quality::UHD4K => Theme::quality_4k(),
            Quality::FHD1080p => Theme::quality_1080p(),
            Quality::HD720p => Theme::quality_720p(),
            Quality::SD480p | Quality::Unknown => Theme::quality_sd(),
        }
    }

    /// Style for seed count
    fn seeds_style(seeds: u32, is_selected: bool) -> Style {
        if is_selected {
            return Theme::accent();
        }
        if seeds >= 100 {
            Theme::seeds_high()
        } else if seeds >= 20 {
            Theme::seeds_medium()
        } else {
            Theme::seeds_low()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn sample_sources() -> Vec<StreamSource> {
        vec![
            StreamSource {
                name: "Torrentio".to_string(),
                title: "The.Batman.2022.1080p.BluRay.x264".to_string(),
                info_hash: "abc123".to_string(),
                file_idx: Some(0),
                seeds: 142,
                quality: Quality::FHD1080p,
                size_bytes: Some(4_500_000_000),
            },
            StreamSource {
                name: "Torrentio".to_string(),
                title: "The.Batman.2022.2160p.WEB-DL.HDR".to_string(),
                info_hash: "def456".to_string(),
                file_idx: Some(0),
                seeds: 89,
                quality: Quality::UHD4K,
                size_bytes: Some(12_000_000_000),
            },
            StreamSource {
                name: "Torrentio".to_string(),
                title: "The.Batman.2022.720p.WEB".to_string(),
                info_hash: "ghi789".to_string(),
                file_idx: Some(0),
                seeds: 203,
                quality: Quality::HD720p,
                size_bytes: Some(1_800_000_000),
            },
        ]
    }

    // =========================================================================
    // BrowserView Tests
    // =========================================================================

    #[test]
    fn test_browser_view_new() {
        let view = BrowserView::new();
        assert!(view.items.is_empty());
        assert_eq!(view.selected, 0);
        assert_eq!(view.offset, 0);
        assert!(view.focused);
    }

    #[test]
    fn test_browser_view_with_title() {
        let view = BrowserView::with_title("RESULTS");
        assert_eq!(view.title, "RESULTS");
    }

    #[test]
    fn test_set_items_resets_selection() {
        let mut view = BrowserView::new();
        view.selected = 5;
        view.offset = 3;
        view.set_items(sample_results());
        assert_eq!(view.selected, 0);
        assert_eq!(view.offset, 0);
        assert_eq!(view.items.len(), 3);
    }

    #[test]
    fn test_navigation_up_down() {
        let mut view = BrowserView::new();
        view.set_items(sample_results());

        assert_eq!(view.selected, 0);

        view.down();
        assert_eq!(view.selected, 1);

        view.down();
        assert_eq!(view.selected, 2);

        view.down(); // At end, stays
        assert_eq!(view.selected, 2);

        view.up();
        assert_eq!(view.selected, 1);

        view.up();
        assert_eq!(view.selected, 0);

        view.up(); // At start, stays
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_home_and_end() {
        let mut view = BrowserView::new();
        view.set_items(sample_results());

        view.end();
        assert_eq!(view.selected, 2);

        view.home();
        assert_eq!(view.selected, 0);
        assert_eq!(view.offset, 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut view = BrowserView::new();
        let mut items = Vec::new();
        for i in 0..20 {
            items.push(SearchResult {
                id: i,
                media_type: MediaType::Movie,
                title: format!("Movie {}", i),
                year: Some(2020),
                overview: String::new(),
                poster_path: None,
                vote_average: 5.0,
            });
        }
        view.set_items(items);

        view.page_down(5);
        assert_eq!(view.selected, 5);

        view.page_down(5);
        assert_eq!(view.selected, 10);

        view.page_up(3);
        assert_eq!(view.selected, 7);

        view.page_up(10);
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_current_returns_selected() {
        let mut view = BrowserView::new();
        view.set_items(sample_results());

        let current = view.current().unwrap();
        assert_eq!(current.title, "The Batman");

        view.down();
        let current = view.current().unwrap();
        assert_eq!(current.title, "Breaking Bad");
    }

    #[test]
    fn test_current_empty_returns_none() {
        let view = BrowserView::new();
        assert!(view.current().is_none());
    }

    #[test]
    fn test_is_empty() {
        let mut view = BrowserView::new();
        assert!(view.is_empty());

        view.set_items(sample_results());
        assert!(!view.is_empty());
    }

    #[test]
    fn test_len() {
        let mut view = BrowserView::new();
        assert_eq!(view.len(), 0);

        view.set_items(sample_results());
        assert_eq!(view.len(), 3);
    }

    #[test]
    fn test_navigation_empty_list() {
        let mut view = BrowserView::new();
        // Should not panic on empty list
        view.up();
        view.down();
        view.home();
        view.end();
        view.page_up(5);
        view.page_down(5);
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_offset_adjustment_scrolls_up() {
        let mut view = BrowserView::new();
        view.set_items(sample_results());
        view.selected = 2;
        view.offset = 2;
        
        view.up();
        // Should adjust offset when going above visible
        assert!(view.offset <= view.selected);
    }

    // =========================================================================
    // SourceBrowserView Tests
    // =========================================================================

    #[test]
    fn test_source_browser_new() {
        let view = SourceBrowserView::new();
        assert!(view.items.is_empty());
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_source_browser_set_items() {
        let mut view = SourceBrowserView::new();
        view.selected = 5;
        view.set_items(sample_sources());
        assert_eq!(view.selected, 0);
        assert_eq!(view.items.len(), 3);
    }

    #[test]
    fn test_source_browser_navigation() {
        let mut view = SourceBrowserView::new();
        view.set_items(sample_sources());

        assert_eq!(view.selected, 0);
        view.down();
        assert_eq!(view.selected, 1);
        view.up();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_source_browser_current() {
        let mut view = SourceBrowserView::new();
        view.set_items(sample_sources());

        let current = view.current().unwrap();
        assert_eq!(current.quality, Quality::FHD1080p);

        view.down();
        let current = view.current().unwrap();
        assert_eq!(current.quality, Quality::UHD4K);
    }

    #[test]
    fn test_source_browser_is_empty() {
        let mut view = SourceBrowserView::new();
        assert!(view.is_empty());

        view.set_items(sample_sources());
        assert!(!view.is_empty());
    }

    // =========================================================================
    // BrowserItem Trait Tests
    // =========================================================================

    #[test]
    fn test_search_result_browser_item() {
        let result = SearchResult {
            id: 1,
            media_type: MediaType::Movie,
            title: "Test Movie".to_string(),
            year: Some(2022),
            overview: String::new(),
            poster_path: None,
            vote_average: 8.5,
        };

        assert_eq!(result.title(), "Test Movie");
        assert_eq!(result.year(), Some(2022));
        assert_eq!(result.quality_str(), Some("MOVIE".to_string()));
        assert_eq!(result.size_str(), Some("★ 8.5".to_string()));
    }

    #[test]
    fn test_stream_source_browser_item() {
        let source = StreamSource {
            name: "Test".to_string(),
            title: "Test.File.1080p".to_string(),
            info_hash: "abc".to_string(),
            file_idx: None,
            seeds: 100,
            quality: Quality::FHD1080p,
            size_bytes: Some(2_000_000_000),
        };

        assert_eq!(source.title(), "Test.File.1080p");
        assert_eq!(source.year(), None);
        assert_eq!(source.quality_str(), Some("1080p".to_string()));
        assert!(source.size_str().unwrap().contains("GB"));
        assert_eq!(source.extra_info(), Some("👤 100".to_string()));
    }

    // =========================================================================
    // Rating Style Tests
    // =========================================================================

    #[test]
    fn test_rating_style_thresholds() {
        // High rating (>= 7.5)
        let style = BrowserView::rating_style(8.0, false);
        assert_eq!(style.fg, Some(Theme::SUCCESS));

        // Medium rating (6.0-7.5)
        let style = BrowserView::rating_style(6.5, false);
        assert_eq!(style.fg, Some(Theme::WARNING));

        // Low rating (4.0-6.0)
        let style = BrowserView::rating_style(5.0, false);
        assert_eq!(style.fg, Some(Theme::DIM));

        // Very low rating (< 4.0)
        let style = BrowserView::rating_style(3.0, false);
        assert_eq!(style.fg, Some(Theme::ERROR));

        // Selected always uses accent
        let style = BrowserView::rating_style(5.0, true);
        assert_eq!(style.fg, Some(Theme::ACCENT));
    }

    // =========================================================================
    // Quality/Seeds Style Tests
    // =========================================================================

    #[test]
    fn test_quality_style() {
        let style = SourceBrowserView::quality_style(&Quality::UHD4K, false);
        assert_eq!(style.fg, Some(Theme::SECONDARY)); // Magenta for 4K

        let style = SourceBrowserView::quality_style(&Quality::FHD1080p, false);
        assert_eq!(style.fg, Some(Theme::PRIMARY)); // Cyan for 1080p

        // Selected overrides
        let style = SourceBrowserView::quality_style(&Quality::UHD4K, true);
        assert_eq!(style.fg, Some(Theme::ACCENT));
    }

    #[test]
    fn test_seeds_style() {
        let style = SourceBrowserView::seeds_style(150, false);
        assert_eq!(style.fg, Some(Theme::SUCCESS)); // High seeds

        let style = SourceBrowserView::seeds_style(50, false);
        assert_eq!(style.fg, Some(Theme::WARNING)); // Medium seeds

        let style = SourceBrowserView::seeds_style(10, false);
        assert_eq!(style.fg, Some(Theme::ERROR)); // Low seeds

        // Selected overrides
        let style = SourceBrowserView::seeds_style(10, true);
        assert_eq!(style.fg, Some(Theme::ACCENT));
    }
}
