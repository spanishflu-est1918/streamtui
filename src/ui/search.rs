//! Search view component
//!
//! Search input with live results and trending section.

use ratatui::prelude::*;

/// Search view state
#[derive(Debug, Default)]
pub struct SearchView {
    /// Current search query
    pub query: String,
    /// Cursor position in query
    pub cursor: usize,
    /// Whether search input is focused
    pub focused: bool,
}

impl SearchView {
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Clear the search query
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
    }

    /// Render the search view
    pub fn render(&self, _frame: &mut Frame, _area: Rect) {
        // TODO: Implement search UI rendering
    }
}
