//! App state and core application logic
//!
//! Manages the application state machine, navigation stack,
//! and coordinates between UI and backend services.

use crate::models::*;

/// Application state enum representing current screen
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    /// Search screen with optional query
    Search,
    /// Content browser showing search results or trending
    Browser,
    /// Detail view for a movie or TV show
    Detail,
    /// Subtitle selection screen
    Subtitles,
    /// Now playing overlay
    NowPlaying,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current screen
    pub screen: AppScreen,
    /// Navigation history stack
    pub nav_stack: Vec<AppScreen>,
    /// Whether the app is running
    pub running: bool,
    /// Current loading state
    pub loading: bool,
    /// Error message to display
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: AppScreen::Search,
            nav_stack: Vec::new(),
            running: true,
            loading: false,
            error: None,
        }
    }
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigate to a new screen, pushing current to stack
    pub fn navigate(&mut self, screen: AppScreen) {
        self.nav_stack.push(self.screen.clone());
        self.screen = screen;
    }

    /// Go back to previous screen
    pub fn back(&mut self) -> bool {
        if let Some(prev) = self.nav_stack.pop() {
            self.screen = prev;
            true
        } else {
            false
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
