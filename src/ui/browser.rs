//! Content browser view
//!
//! Displays search results or trending content in a selectable list.

use ratatui::prelude::*;
use crate::models::SearchResult;

/// Browser view state
#[derive(Debug, Default)]
pub struct BrowserView {
    /// List of items to display
    pub items: Vec<SearchResult>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset
    pub offset: usize,
}

impl BrowserView {
    pub fn new() -> Self {
        Self::default()
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
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Get currently selected item
    pub fn current(&self) -> Option<&SearchResult> {
        self.items.get(self.selected)
    }

    /// Render the browser view
    pub fn render(&self, _frame: &mut Frame, _area: Rect) {
        // TODO: Implement browser UI rendering
    }
}
