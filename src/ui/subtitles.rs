//! Subtitle selection view
//!
//! Display available subtitles grouped by language with trust indicators.
//! Cyberpunk neon aesthetic with keyboard-first navigation.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
};

use crate::models::SubtitleResult;
use crate::ui::Theme;
use std::collections::BTreeMap;

// =============================================================================
// SubtitlesView
// =============================================================================

/// Subtitle selection view state
#[derive(Debug, Default)]
pub struct SubtitlesView {
    /// Available subtitles
    pub subtitles: Vec<SubtitleResult>,
    /// Selected index (flat index across all languages)
    pub selected: usize,
    /// Filter by language (None = show all)
    pub language_filter: Option<String>,
    /// Title being displayed (for header)
    pub title: String,
    /// Year (for header)
    pub year: Option<u16>,
    /// Scroll offset for long lists
    pub scroll_offset: usize,
}

/// A grouped subtitle entry for rendering
#[derive(Debug)]
pub enum SubtitleRow {
    /// Language header row (not selectable)
    LanguageHeader(String),
    /// Subtitle entry (selectable)
    Subtitle(SubtitleResult),
}

impl SubtitlesView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set media info for display
    pub fn set_media(&mut self, title: String, year: Option<u16>) {
        self.title = title;
        self.year = year;
    }

    /// Set available subtitles
    pub fn set_subtitles(&mut self, subtitles: Vec<SubtitleResult>) {
        self.subtitles = subtitles;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Clear subtitles
    pub fn clear(&mut self) {
        self.subtitles.clear();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Get filtered subtitles
    pub fn filtered(&self) -> Vec<&SubtitleResult> {
        match &self.language_filter {
            Some(lang) => self
                .subtitles
                .iter()
                .filter(|s| s.language == *lang)
                .collect(),
            None => self.subtitles.iter().collect(),
        }
    }

    /// Group subtitles by language for display
    /// Returns a flat list with language headers interspersed
    pub fn grouped_rows(&self) -> Vec<SubtitleRow> {
        let filtered = self.filtered();
        
        // Group by language using BTreeMap for consistent ordering
        let mut by_language: BTreeMap<String, Vec<&SubtitleResult>> = BTreeMap::new();
        for sub in filtered {
            by_language
                .entry(sub.language_name.clone())
                .or_default()
                .push(sub);
        }

        // Sort each language group by trust score (descending)
        for subs in by_language.values_mut() {
            subs.sort_by(|a, b| b.trust_score().cmp(&a.trust_score()));
        }

        // Flatten into rows with headers
        let mut rows = Vec::new();
        for (lang_name, subs) in by_language {
            rows.push(SubtitleRow::LanguageHeader(lang_name));
            for sub in subs {
                rows.push(SubtitleRow::Subtitle(sub.clone()));
            }
        }

        rows
    }

    /// Get selectable items (subtitles only, no headers)
    pub fn selectable(&self) -> Vec<&SubtitleResult> {
        self.filtered()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .collect()
    }

    /// Get count of selectable items
    pub fn selectable_count(&self) -> usize {
        self.filtered().len()
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        let max = self.selectable_count().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
    }

    /// Page down
    pub fn page_down(&mut self, page_size: usize) {
        let max = self.selectable_count().saturating_sub(1);
        self.selected = (self.selected + page_size).min(max);
    }

    /// Jump to first
    pub fn first(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Jump to last
    pub fn last(&mut self) {
        let count = self.selectable_count();
        if count > 0 {
            self.selected = count - 1;
        }
    }

    /// Get currently selected subtitle
    pub fn current(&self) -> Option<&SubtitleResult> {
        let selectables = self.selectable();
        selectables.get(self.selected).copied()
    }

    /// Set language filter
    pub fn set_language_filter(&mut self, lang: Option<String>) {
        self.language_filter = lang;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Get available languages
    pub fn available_languages(&self) -> Vec<String> {
        let mut langs: Vec<_> = self
            .subtitles
            .iter()
            .map(|s| s.language_name.clone())
            .collect();
        langs.sort();
        langs.dedup();
        langs
    }

    /// Render the subtitles view
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Main layout: header, content, keybinds
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Content
                Constraint::Length(3),  // Keybinds
            ])
            .split(area);

        self.render_header(frame, chunks[0]);
        self.render_content(frame, chunks[1]);
        self.render_keybinds(frame, chunks[2]);
    }

    /// Render the header with title
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let year_str = self.year.map(|y| format!(" ({})", y)).unwrap_or_default();
        let header_text = format!("📝 SUBTITLES - {}{}", self.title, year_str);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Theme::border_focused())
            .title(Span::styled(" StreamTUI ", Theme::title()));

        let header = Paragraph::new(Line::from(vec![
            Span::styled(header_text, Theme::title()),
        ]))
        .block(block)
        .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }

    /// Render the main content area with grouped subtitles
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border_focused());

        if self.subtitles.is_empty() {
            let empty = Paragraph::new(Line::from(vec![
                Span::styled("No subtitles found", Theme::dimmed()),
            ]))
            .block(block)
            .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let inner = block.inner(area);
        let visible_height = inner.height as usize;

        // Build grouped rows for display
        let rows = self.grouped_rows();
        
        // Map selected index to row index (accounting for headers)
        let selected_row_idx = self.selected_to_row_index(&rows);
        
        // Calculate scroll offset
        let scroll_offset = if selected_row_idx >= visible_height {
            selected_row_idx.saturating_sub(visible_height) + 1
        } else {
            0
        };

        // Track which selectable item we're on
        let mut selectable_idx = 0;

        let items: Vec<ListItem> = rows
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(_, row)| match row {
                SubtitleRow::LanguageHeader(lang) => {
                    self.render_language_header(lang)
                }
                SubtitleRow::Subtitle(sub) => {
                    let is_selected = selectable_idx == self.selected;
                    let item = self.render_subtitle_item(sub, is_selected);
                    selectable_idx += 1;
                    item
                }
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Map selected index to actual row index (accounting for headers)
    fn selected_to_row_index(&self, rows: &[SubtitleRow]) -> usize {
        let mut selectable_count = 0;
        for (i, row) in rows.iter().enumerate() {
            if matches!(row, SubtitleRow::Subtitle(_)) {
                if selectable_count == self.selected {
                    return i;
                }
                selectable_count += 1;
            }
        }
        0
    }

    /// Render a language header row
    fn render_language_header(&self, language: &str) -> ListItem<'static> {
        let line = Line::from(vec![
            Span::styled("  🌐 ", Theme::accent()),
            Span::styled(language.to_string(), Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD)),
        ]);
        ListItem::new(line)
    }

    /// Render a subtitle item row
    fn render_subtitle_item(&self, sub: &SubtitleResult, is_selected: bool) -> ListItem<'static> {
        // Selection marker
        let marker = if is_selected { "  ▸ " } else { "    " };
        let marker_style = if is_selected { Theme::accent() } else { Theme::dimmed() };

        // Trust indicator: ✓ for trusted, empty otherwise
        let trust_indicator = if sub.from_trusted {
            "[✓] "
        } else {
            "[ ] "
        };
        let trust_style = if sub.from_trusted {
            Theme::success()
        } else {
            Theme::dimmed()
        };

        // Release name (truncated)
        let release: String = sub.release.chars().take(32).collect();
        let release_style = if is_selected {
            Theme::list_item_selected()
        } else {
            Theme::text()
        };

        // Download count (formatted)
        let downloads = Self::format_downloads(sub.downloads);
        let downloads_style = if is_selected { Theme::accent() } else { Theme::dimmed() };

        // Status indicators
        let mut status_spans = Vec::new();
        
        if sub.from_trusted {
            status_spans.push(Span::styled(" Trusted", Theme::success()));
        }
        if sub.ai_translated {
            status_spans.push(Span::styled(" ⚠️AI", Theme::warning()));
        }
        if sub.hearing_impaired {
            status_spans.push(Span::styled(" 👂SDH", Theme::secondary()));
        }

        let mut spans = vec![
            Span::styled(marker.to_string(), marker_style),
            Span::styled(trust_indicator.to_string(), trust_style),
            Span::styled(release, release_style),
            Span::styled(format!("  {:>5}⬇", downloads), downloads_style),
        ];
        spans.extend(status_spans);

        ListItem::new(Line::from(spans))
    }

    /// Format download count (e.g., 50000 -> "50k")
    fn format_downloads(count: u32) -> String {
        if count >= 1_000_000 {
            format!("{:.1}M", count as f64 / 1_000_000.0)
        } else if count >= 1_000 {
            format!("{:.0}k", count as f64 / 1_000.0)
        } else {
            count.to_string()
        }
    }

    /// Render keybinding hints
    fn render_keybinds(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border());

        let keybinds = Line::from(vec![
            Span::styled("[Enter] ", Theme::keybind()),
            Span::styled("Select  ", Theme::keybind_desc()),
            Span::styled("[c] ", Theme::keybind()),
            Span::styled("Cast with selected  ", Theme::keybind_desc()),
            Span::styled("[n] ", Theme::keybind()),
            Span::styled("No subtitles  ", Theme::keybind_desc()),
            Span::styled("[Esc] ", Theme::keybind()),
            Span::styled("Back", Theme::keybind_desc()),
        ]);

        let para = Paragraph::new(keybinds)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(para, area);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SubFormat;

    fn make_subtitle(
        id: &str,
        lang: &str,
        lang_name: &str,
        release: &str,
        downloads: u32,
        trusted: bool,
        ai: bool,
    ) -> SubtitleResult {
        SubtitleResult {
            id: id.to_string(),
            file_id: id.parse().unwrap_or(1),
            language: lang.to_string(),
            language_name: lang_name.to_string(),
            release: release.to_string(),
            fps: None,
            format: SubFormat::Srt,
            downloads,
            from_trusted: trusted,
            hearing_impaired: false,
            ai_translated: ai,
        }
    }

    #[test]
    fn test_set_subtitles() {
        let mut view = SubtitlesView::new();
        let subs = vec![
            make_subtitle("1", "en", "English", "Test.1080p", 1000, true, false),
            make_subtitle("2", "es", "Spanish", "Test.720p", 500, false, false),
        ];
        view.set_subtitles(subs);

        assert_eq!(view.subtitles.len(), 2);
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_navigation_up_down() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "Test1", 1000, true, false),
            make_subtitle("2", "en", "English", "Test2", 500, false, false),
            make_subtitle("3", "en", "English", "Test3", 200, false, false),
        ]);

        assert_eq!(view.selected, 0);
        view.down();
        assert_eq!(view.selected, 1);
        view.down();
        assert_eq!(view.selected, 2);
        view.down(); // Should stay at max
        assert_eq!(view.selected, 2);
        view.up();
        assert_eq!(view.selected, 1);
        view.up();
        assert_eq!(view.selected, 0);
        view.up(); // Should stay at 0
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_current_selection() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "First", 1000, true, false),
            make_subtitle("2", "en", "English", "Second", 500, false, false),
        ]);

        assert_eq!(view.current().unwrap().release, "First");
        view.down();
        assert_eq!(view.current().unwrap().release, "Second");
    }

    #[test]
    fn test_empty_subtitles() {
        let view = SubtitlesView::new();
        assert!(view.current().is_none());
        assert_eq!(view.selectable_count(), 0);
    }

    #[test]
    fn test_grouped_rows() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "English1", 1000, true, false),
            make_subtitle("2", "es", "Spanish", "Spanish1", 500, false, false),
            make_subtitle("3", "en", "English", "English2", 200, false, false),
        ]);

        let rows = view.grouped_rows();
        
        // Should have: English header, 2 English subs, Spanish header, 1 Spanish sub
        assert_eq!(rows.len(), 5);
        
        // First should be English header (BTreeMap orders alphabetically)
        assert!(matches!(&rows[0], SubtitleRow::LanguageHeader(l) if l == "English"));
        // Then English subs (sorted by trust score - trusted first)
        assert!(matches!(&rows[1], SubtitleRow::Subtitle(s) if s.from_trusted));
        assert!(matches!(&rows[2], SubtitleRow::Subtitle(s) if s.release == "English2"));
        // Spanish header
        assert!(matches!(&rows[3], SubtitleRow::LanguageHeader(l) if l == "Spanish"));
        // Spanish sub
        assert!(matches!(&rows[4], SubtitleRow::Subtitle(_)));
    }

    #[test]
    fn test_language_filter() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "English1", 1000, true, false),
            make_subtitle("2", "es", "Spanish", "Spanish1", 500, false, false),
            make_subtitle("3", "en", "English", "English2", 200, false, false),
        ]);

        // No filter - should have 3 items
        assert_eq!(view.selectable_count(), 3);

        // Filter to English only
        view.set_language_filter(Some("en".to_string()));
        assert_eq!(view.selectable_count(), 2);

        // Filter to Spanish
        view.set_language_filter(Some("es".to_string()));
        assert_eq!(view.selectable_count(), 1);

        // Clear filter
        view.set_language_filter(None);
        assert_eq!(view.selectable_count(), 3);
    }

    #[test]
    fn test_available_languages() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "E1", 1000, true, false),
            make_subtitle("2", "es", "Spanish", "S1", 500, false, false),
            make_subtitle("3", "en", "English", "E2", 200, false, false),
            make_subtitle("4", "fr", "French", "F1", 100, false, false),
        ]);

        let langs = view.available_languages();
        assert_eq!(langs.len(), 3);
        assert!(langs.contains(&"English".to_string()));
        assert!(langs.contains(&"Spanish".to_string()));
        assert!(langs.contains(&"French".to_string()));
    }

    #[test]
    fn test_format_downloads() {
        assert_eq!(SubtitlesView::format_downloads(500), "500");
        assert_eq!(SubtitlesView::format_downloads(1000), "1k");
        assert_eq!(SubtitlesView::format_downloads(50000), "50k");
        assert_eq!(SubtitlesView::format_downloads(1500000), "1.5M");
    }

    #[test]
    fn test_page_navigation() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "E1", 1000, true, false),
            make_subtitle("2", "en", "English", "E2", 900, false, false),
            make_subtitle("3", "en", "English", "E3", 800, false, false),
            make_subtitle("4", "en", "English", "E4", 700, false, false),
            make_subtitle("5", "en", "English", "E5", 600, false, false),
        ]);

        view.page_down(3);
        assert_eq!(view.selected, 3);
        
        view.page_down(3); // Should cap at max
        assert_eq!(view.selected, 4);
        
        view.page_up(2);
        assert_eq!(view.selected, 2);
        
        view.first();
        assert_eq!(view.selected, 0);
        
        view.last();
        assert_eq!(view.selected, 4);
    }

    #[test]
    fn test_set_media() {
        let mut view = SubtitlesView::new();
        view.set_media("The Batman".to_string(), Some(2022));

        assert_eq!(view.title, "The Batman");
        assert_eq!(view.year, Some(2022));
    }

    #[test]
    fn test_trust_score_sorting() {
        let mut view = SubtitlesView::new();
        view.set_subtitles(vec![
            make_subtitle("1", "en", "English", "Untrusted", 5000, false, false),
            make_subtitle("2", "en", "English", "Trusted", 1000, true, false),
            make_subtitle("3", "en", "English", "AI", 8000, false, true),
        ]);

        let rows = view.grouped_rows();
        
        // After English header, should be: Trusted (11000), Untrusted (5000), AI (3000)
        if let SubtitleRow::Subtitle(first) = &rows[1] {
            assert!(first.from_trusted, "Trusted should be first");
        }
        if let SubtitleRow::Subtitle(second) = &rows[2] {
            assert!(!second.from_trusted && !second.ai_translated, "Untrusted non-AI should be second");
        }
        if let SubtitleRow::Subtitle(third) = &rows[3] {
            assert!(third.ai_translated, "AI should be last");
        }
    }
}
