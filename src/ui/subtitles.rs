//! SubtitleResult selection view
//!
//! Display available subtitles grouped by language with trust indicators.

use ratatui::prelude::*;
use crate::models::SubtitleResult;

/// SubtitleResult selection view state
#[derive(Debug, Default)]
pub struct SubtitlesView {
    /// Available subtitles
    pub subtitles: Vec<SubtitleResult>,
    /// Selected index
    pub selected: usize,
    /// Filter by language
    pub language_filter: Option<String>,
}

impl SubtitlesView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set available subtitles
    pub fn set_subtitles(&mut self, subtitles: Vec<SubtitleResult>) {
        self.subtitles = subtitles;
        self.selected = 0;
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

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        let max = self.filtered().len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    /// Get currently selected subtitle
    pub fn current(&self) -> Option<&SubtitleResult> {
        self.filtered().get(self.selected).copied()
    }

    /// Render the subtitles view
    pub fn render(&self, _frame: &mut Frame, _area: Rect) {
        // TODO: Implement subtitles UI rendering
    }
}
