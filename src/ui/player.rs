//! Now Playing overlay
//!
//! Shows casting status, progress bar, and playback controls.

use ratatui::prelude::*;
use crate::models::PlaybackStatus;

/// Player overlay state
#[derive(Debug, Default)]
pub struct PlayerView {
    /// Current playback status
    pub status: Option<PlaybackStatus>,
    /// Title of what's playing
    pub title: String,
    /// Whether subtitles are active
    pub subtitles_active: bool,
    /// Subtitle language if active
    pub subtitle_language: Option<String>,
}

impl PlayerView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update playback status
    pub fn update_status(&mut self, status: PlaybackStatus) {
        self.status = Some(status);
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Format duration as HH:MM:SS or MM:SS
    pub fn format_duration(seconds: f64) -> String {
        let total_secs = seconds as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{:02}:{:02}", minutes, secs)
        }
    }

    /// Get progress as percentage (0.0 - 1.0)
    pub fn progress_percent(&self) -> f64 {
        self.status
            .as_ref()
            .map(|s| {
                let dur = s.duration.as_secs_f64();
                if dur > 0.0 {
                    s.position.as_secs_f64() / dur
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }

    /// Render the player overlay
    pub fn render(&self, _frame: &mut Frame, _area: Rect) {
        // TODO: Implement player UI rendering
    }
}
