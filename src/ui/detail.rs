//! Detail view for movies and TV shows
//!
//! Shows full info, seasons/episodes for TV, and stream sources.

use ratatui::prelude::*;
use crate::models::{SearchResult, StreamSource, SeasonSummary, Episode};

/// Detail view state
#[derive(Debug, Default)]
pub struct DetailView {
    /// The media being displayed
    pub media: Option<SearchResult>,
    /// Available stream sources
    pub sources: Vec<StreamSource>,
    /// Selected source index
    pub selected_source: usize,
    /// For TV: available seasons
    pub seasons: Vec<SeasonSummary>,
    /// For TV: selected season
    pub selected_season: usize,
    /// For TV: episodes in selected season
    pub episodes: Vec<Episode>,
    /// For TV: selected episode
    pub selected_episode: usize,
    /// Current focus area
    pub focus: DetailFocus,
}

/// Focus areas in detail view
#[derive(Debug, Default, Clone, PartialEq)]
pub enum DetailFocus {
    #[default]
    Info,
    Seasons,
    Episodes,
    Sources,
}

impl DetailView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set media to display
    pub fn set_media(&mut self, media: SearchResult) {
        self.media = Some(media);
        self.sources.clear();
        self.selected_source = 0;
    }

    /// Set available sources
    pub fn set_sources(&mut self, sources: Vec<StreamSource>) {
        self.sources = sources;
        self.selected_source = 0;
    }

    /// Get selected source
    pub fn current_source(&self) -> Option<&StreamSource> {
        self.sources.get(self.selected_source)
    }

    /// Move source selection up
    pub fn source_up(&mut self) {
        if self.selected_source > 0 {
            self.selected_source -= 1;
        }
    }

    /// Move source selection down
    pub fn source_down(&mut self) {
        if self.selected_source < self.sources.len().saturating_sub(1) {
            self.selected_source += 1;
        }
    }

    /// Render the detail view
    pub fn render(&self, _frame: &mut Frame, _area: Rect) {
        // TODO: Implement detail UI rendering
    }
}
