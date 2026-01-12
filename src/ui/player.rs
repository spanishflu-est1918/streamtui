//! Now Playing overlay
//!
//! Shows casting status, progress bar, and playback controls.
//! Centered overlay with cyberpunk neon aesthetic.

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph},
};

use crate::models::{CastDevice, CastState, PlaybackStatus};
use crate::ui::Theme;

/// Player overlay state
#[derive(Debug, Default)]
pub struct PlayerView {
    /// Current playback status
    pub status: Option<PlaybackStatus>,
    /// Title of what's playing
    pub title: String,
    /// Year of the media (optional)
    pub year: Option<u16>,
    /// Target cast device
    pub device: Option<CastDevice>,
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

    /// Set the year
    pub fn set_year(&mut self, year: Option<u16>) {
        self.year = year;
    }

    /// Set target device
    pub fn set_device(&mut self, device: CastDevice) {
        self.device = Some(device);
    }

    /// Set subtitle state
    pub fn set_subtitles(&mut self, active: bool, language: Option<String>) {
        self.subtitles_active = active;
        self.subtitle_language = language;
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

    /// Get current playback state
    pub fn state(&self) -> CastState {
        self.status
            .as_ref()
            .map(|s| s.state.clone())
            .unwrap_or(CastState::Idle)
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.state(), CastState::Playing)
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        matches!(self.state(), CastState::Paused)
    }

    /// Get state indicator symbol
    fn state_indicator(&self) -> (&str, Style) {
        match self.state() {
            CastState::Playing => ("▶", Theme::success()),
            CastState::Paused => ("⏸", Theme::warning()),
            CastState::Buffering => ("◌", Theme::loading()),
            CastState::Connecting => ("⟳", Theme::loading()),
            CastState::Idle | CastState::Stopped => ("⏹", Theme::dimmed()),
            CastState::Error(_) => ("✖", Theme::error()),
        }
    }

    /// Get device name for display
    fn device_name(&self) -> &str {
        self.device
            .as_ref()
            .map(|d| d.name.as_str())
            .unwrap_or("Unknown Device")
    }

    /// Calculate centered overlay area
    fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
        let popup_width = width.min(area.width.saturating_sub(4));
        let popup_height = height.min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;

        Rect::new(x, y, popup_width, popup_height)
    }

    /// Render the player overlay
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Overlay dimensions: 50 chars wide, 9 lines tall
        let overlay_width = 50;
        let overlay_height = 9;
        let overlay_area = Self::centered_rect(overlay_width, overlay_height, area);

        // Clear background (makes it a true overlay)
        frame.render_widget(Clear, overlay_area);

        // Build the overlay block with double border for cyberpunk feel
        let (state_symbol, state_style) = self.state_indicator();
        let header_title = format!(" {} NOW CASTING ", state_symbol);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Theme::border_focused())
            .title(Span::styled(header_title, state_style))
            .title_alignment(Alignment::Left)
            .style(Style::default().bg(Theme::BACKGROUND));

        let inner = block.inner(overlay_area);
        frame.render_widget(block, overlay_area);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Progress bar
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Device + subtitles
                Constraint::Length(1), // Controls
            ])
            .split(inner);

        // Title line with year
        self.render_title(frame, chunks[0]);

        // Progress bar with times
        self.render_progress(frame, chunks[2]);

        // Device name + subtitle indicator
        self.render_device_line(frame, chunks[4]);

        // Playback controls hint
        self.render_controls(frame, chunks[5]);
    }

    /// Render title line
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let year_str = self.year.map(|y| format!(" ({})", y)).unwrap_or_default();

        let title_line = Line::from(vec![
            Span::styled(&self.title, Theme::title()),
            Span::styled(year_str, Theme::secondary()),
        ]);

        let paragraph = Paragraph::new(title_line).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    /// Render progress bar with time positions
    fn render_progress(&self, frame: &mut Frame, area: Rect) {
        let (position_secs, duration_secs) = self
            .status
            .as_ref()
            .map(|s| (s.position.as_secs_f64(), s.duration.as_secs_f64()))
            .unwrap_or((0.0, 0.0));

        let progress_ratio = self.progress_percent();
        let position_str = Self::format_duration(position_secs);
        let duration_str = Self::format_duration(duration_secs);

        // Layout: [time] [==progress==] [time]
        let time_width = 9u16; // "HH:MM:SS" + padding
        let bar_width = area.width.saturating_sub(time_width * 2);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(time_width),
                Constraint::Min(bar_width),
                Constraint::Length(time_width),
            ])
            .split(area);

        // Current position (left)
        let pos_text = Paragraph::new(Span::styled(&position_str, Theme::accent()))
            .alignment(Alignment::Right);
        frame.render_widget(pos_text, chunks[0]);

        // Progress bar (center)
        let gauge = Gauge::default()
            .gauge_style(Theme::progress_bar())
            .ratio(progress_ratio.clamp(0.0, 1.0))
            .label(""); // No label on the gauge itself

        frame.render_widget(gauge, chunks[1]);

        // Duration (right)
        let dur_text =
            Paragraph::new(Span::styled(&duration_str, Theme::dimmed())).alignment(Alignment::Left);
        frame.render_widget(dur_text, chunks[2]);
    }

    /// Render device name and subtitle indicator
    fn render_device_line(&self, frame: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled("📺 ", Theme::secondary()),
            Span::styled(self.device_name(), Theme::cast_target()),
        ];

        // Add subtitle indicator if active
        if self.subtitles_active {
            let lang = self.subtitle_language.as_deref().unwrap_or("??");
            spans.push(Span::styled("  │  ", Theme::dimmed()));
            spans.push(Span::styled("CC ", Theme::accent()));
            spans.push(Span::styled(lang.to_uppercase(), Theme::secondary()));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    /// Render playback controls hint
    fn render_controls(&self, frame: &mut Frame, area: Rect) {
        // Show pause/play depending on current state
        let play_pause = if self.is_playing() {
            ("[Space] Pause", Theme::keybind())
        } else {
            ("[Space] Play", Theme::keybind())
        };

        let line = Line::from(vec![
            Span::styled(play_pause.0, play_pause.1),
            Span::styled("  ", Theme::dimmed()),
            Span::styled("[s]", Theme::keybind()),
            Span::styled(" Stop  ", Theme::keybind_desc()),
            Span::styled("[←→]", Theme::keybind()),
            Span::styled(" Seek  ", Theme::keybind_desc()),
            Span::styled("[Esc]", Theme::keybind()),
            Span::styled(" Close", Theme::keybind_desc()),
        ]);

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_status_playing() -> PlaybackStatus {
        PlaybackStatus {
            state: CastState::Playing,
            position: Duration::from_secs(2723),  // 45:23
            duration: Duration::from_secs(10560), // 2:56:00
            volume: 0.8,
            title: Some("The Batman".to_string()),
        }
    }

    fn sample_status_paused() -> PlaybackStatus {
        PlaybackStatus {
            state: CastState::Paused,
            position: Duration::from_secs(1000),
            duration: Duration::from_secs(5000),
            volume: 1.0,
            title: None,
        }
    }

    fn sample_device() -> CastDevice {
        CastDevice {
            id: "device-1".to_string(),
            name: "Living Room TV".to_string(),
            address: "192.168.1.100".parse().unwrap(),
            port: 8009,
            model: Some("Chromecast".to_string()),
        }
    }

    // =========================================================================
    // Duration Formatting Tests
    // =========================================================================

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(PlayerView::format_duration(45.0), "00:45");
    }

    #[test]
    fn test_format_duration_minutes_seconds() {
        assert_eq!(PlayerView::format_duration(125.0), "02:05");
    }

    #[test]
    fn test_format_duration_hours() {
        // 2h 56m = 10560 seconds
        assert_eq!(PlayerView::format_duration(10560.0), "02:56:00");
    }

    #[test]
    fn test_format_duration_complex() {
        // 1h 23m 45s = 5025 seconds
        assert_eq!(PlayerView::format_duration(5025.0), "01:23:45");
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(PlayerView::format_duration(0.0), "00:00");
    }

    // =========================================================================
    // Progress Calculation Tests
    // =========================================================================

    #[test]
    fn test_progress_percent_playing() {
        let mut view = PlayerView::new();
        view.update_status(sample_status_playing());

        let progress = view.progress_percent();
        // 2723 / 10560 ≈ 0.2578
        assert!((progress - 0.2578).abs() < 0.01);
    }

    #[test]
    fn test_progress_percent_no_status() {
        let view = PlayerView::new();
        assert_eq!(view.progress_percent(), 0.0);
    }

    #[test]
    fn test_progress_percent_zero_duration() {
        let mut view = PlayerView::new();
        view.update_status(PlaybackStatus {
            state: CastState::Playing,
            position: Duration::from_secs(100),
            duration: Duration::ZERO,
            volume: 1.0,
            title: None,
        });
        assert_eq!(view.progress_percent(), 0.0);
    }

    // =========================================================================
    // State Tests
    // =========================================================================

    #[test]
    fn test_state_playing() {
        let mut view = PlayerView::new();
        view.update_status(sample_status_playing());
        assert_eq!(view.state(), CastState::Playing);
        assert!(view.is_playing());
        assert!(!view.is_paused());
    }

    #[test]
    fn test_state_paused() {
        let mut view = PlayerView::new();
        view.update_status(sample_status_paused());
        assert_eq!(view.state(), CastState::Paused);
        assert!(!view.is_playing());
        assert!(view.is_paused());
    }

    #[test]
    fn test_state_default_idle() {
        let view = PlayerView::new();
        assert_eq!(view.state(), CastState::Idle);
    }

    // =========================================================================
    // State Indicator Tests
    // =========================================================================

    #[test]
    fn test_state_indicator_playing() {
        let mut view = PlayerView::new();
        view.update_status(sample_status_playing());
        let (symbol, _) = view.state_indicator();
        assert_eq!(symbol, "▶");
    }

    #[test]
    fn test_state_indicator_paused() {
        let mut view = PlayerView::new();
        view.update_status(sample_status_paused());
        let (symbol, _) = view.state_indicator();
        assert_eq!(symbol, "⏸");
    }

    #[test]
    fn test_state_indicator_buffering() {
        let mut view = PlayerView::new();
        view.update_status(PlaybackStatus {
            state: CastState::Buffering,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 1.0,
            title: None,
        });
        let (symbol, _) = view.state_indicator();
        assert_eq!(symbol, "◌");
    }

    // =========================================================================
    // Device & Subtitle Tests
    // =========================================================================

    #[test]
    fn test_device_name_set() {
        let mut view = PlayerView::new();
        view.set_device(sample_device());
        assert_eq!(view.device_name(), "Living Room TV");
    }

    #[test]
    fn test_device_name_default() {
        let view = PlayerView::new();
        assert_eq!(view.device_name(), "Unknown Device");
    }

    #[test]
    fn test_subtitles_state() {
        let mut view = PlayerView::new();
        assert!(!view.subtitles_active);
        assert!(view.subtitle_language.is_none());

        view.set_subtitles(true, Some("en".to_string()));
        assert!(view.subtitles_active);
        assert_eq!(view.subtitle_language.as_deref(), Some("en"));
    }

    // =========================================================================
    // Title & Year Tests
    // =========================================================================

    #[test]
    fn test_set_title() {
        let mut view = PlayerView::new();
        view.set_title("The Batman");
        assert_eq!(view.title, "The Batman");
    }

    #[test]
    fn test_set_year() {
        let mut view = PlayerView::new();
        view.set_year(Some(2022));
        assert_eq!(view.year, Some(2022));
    }

    // =========================================================================
    // Centered Rect Tests
    // =========================================================================

    #[test]
    fn test_centered_rect_normal() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = PlayerView::centered_rect(50, 10, area);

        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 10);
        assert_eq!(centered.x, 25); // (100 - 50) / 2
        assert_eq!(centered.y, 20); // (50 - 10) / 2
    }

    #[test]
    fn test_centered_rect_small_terminal() {
        let area = Rect::new(0, 0, 40, 15);
        let centered = PlayerView::centered_rect(50, 10, area);

        // Width capped to 36 (40 - 4)
        assert_eq!(centered.width, 36);
        // Height capped to 13 (15 - 2)
        assert_eq!(centered.height, 10);
    }

    // =========================================================================
    // View Creation Tests
    // =========================================================================

    #[test]
    fn test_player_view_new() {
        let view = PlayerView::new();
        assert!(view.status.is_none());
        assert!(view.title.is_empty());
        assert!(view.year.is_none());
        assert!(view.device.is_none());
        assert!(!view.subtitles_active);
    }

    #[test]
    fn test_player_view_full_setup() {
        let mut view = PlayerView::new();
        view.set_title("The Batman");
        view.set_year(Some(2022));
        view.set_device(sample_device());
        view.update_status(sample_status_playing());
        view.set_subtitles(true, Some("en".to_string()));

        assert_eq!(view.title, "The Batman");
        assert_eq!(view.year, Some(2022));
        assert_eq!(view.device_name(), "Living Room TV");
        assert!(view.is_playing());
        assert!(view.subtitles_active);
    }
}
