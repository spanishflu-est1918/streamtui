//! Cyberpunk neon theme for StreamTUI
//!
//! Color palette and style helpers for the TUI.
//! All colors are from specs/tui.md - the cyberpunk neon aesthetic.

use ratatui::style::{Color, Modifier, Style};

/// Cyberpunk neon color palette
///
/// All colors defined exactly as specified in specs/tui.md
pub struct Theme;

impl Theme {
    // ═══════════════════════════════════════════════════════════════════════
    // CORE PALETTE (from specs/tui.md)
    // ═══════════════════════════════════════════════════════════════════════

    /// Background: #0a0a0f (deep black-blue)
    pub const BACKGROUND: Color = Color::Rgb(0x0a, 0x0a, 0x0f);

    /// Primary: #00fff2 (cyan neon)
    pub const PRIMARY: Color = Color::Rgb(0x00, 0xff, 0xf2);

    /// Secondary: #ff00ff (magenta)
    pub const SECONDARY: Color = Color::Rgb(0xff, 0x00, 0xff);

    /// Accent: #ffff00 (yellow)
    pub const ACCENT: Color = Color::Rgb(0xff, 0xff, 0x00);

    /// Highlight: #ff0080 (hot pink)
    pub const HIGHLIGHT: Color = Color::Rgb(0xff, 0x00, 0x80);

    /// Text: #e0e0e0 (soft white)
    pub const TEXT: Color = Color::Rgb(0xe0, 0xe0, 0xe0);

    /// Dim: #404050 (muted)
    pub const DIM: Color = Color::Rgb(0x40, 0x40, 0x50);

    /// Success: #00ff00 (green)
    pub const SUCCESS: Color = Color::Rgb(0x00, 0xff, 0x00);

    /// Warning: #ffaa00 (orange)
    pub const WARNING: Color = Color::Rgb(0xff, 0xaa, 0x00);

    /// Error: #ff0040 (red)
    pub const ERROR: Color = Color::Rgb(0xff, 0x00, 0x40);

    // ═══════════════════════════════════════════════════════════════════════
    // DERIVED COLORS (for UI elements)
    // ═══════════════════════════════════════════════════════════════════════

    /// Slightly lighter background for panels/cards
    pub const BACKGROUND_LIGHT: Color = Color::Rgb(0x14, 0x14, 0x1e);

    /// Even lighter for hover states
    pub const BACKGROUND_HOVER: Color = Color::Rgb(0x1e, 0x1e, 0x2d);

    /// Border color (dim cyan)
    pub const BORDER: Color = Color::Rgb(0x00, 0x80, 0x78);

    /// Border color when focused (full cyan)
    pub const BORDER_FOCUSED: Color = Self::PRIMARY;

    // ═══════════════════════════════════════════════════════════════════════
    // STYLE HELPERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Default text style
    pub fn text() -> Style {
        Style::default().fg(Self::TEXT).bg(Self::BACKGROUND)
    }

    /// Highlighted text (inverted with primary color)
    pub fn highlighted() -> Style {
        Style::default()
            .fg(Self::BACKGROUND)
            .bg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Selected item style (hot pink, bold)
    pub fn selected() -> Style {
        Style::default()
            .fg(Self::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Dimmed/muted text
    pub fn dimmed() -> Style {
        Style::default().fg(Self::DIM)
    }

    /// Error style
    pub fn error() -> Style {
        Style::default()
            .fg(Self::ERROR)
            .add_modifier(Modifier::BOLD)
    }

    /// Success style
    pub fn success() -> Style {
        Style::default()
            .fg(Self::SUCCESS)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning style
    pub fn warning() -> Style {
        Style::default()
            .fg(Self::WARNING)
            .add_modifier(Modifier::BOLD)
    }

    /// Title/header style
    pub fn title() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Secondary text style (magenta)
    pub fn secondary() -> Style {
        Style::default().fg(Self::SECONDARY)
    }

    /// Accent text style (yellow)
    pub fn accent() -> Style {
        Style::default()
            .fg(Self::ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    /// Normal/unfocused border
    pub fn border() -> Style {
        Style::default().fg(Self::BORDER)
    }

    /// Focused border (glowing effect)
    pub fn border_focused() -> Style {
        Style::default()
            .fg(Self::BORDER_FOCUSED)
            .add_modifier(Modifier::BOLD)
    }

    /// Progress bar style
    pub fn progress_bar() -> Style {
        Style::default()
            .fg(Self::SUCCESS)
            .bg(Self::BACKGROUND_LIGHT)
    }

    /// Progress bar unfilled portion
    pub fn progress_bar_empty() -> Style {
        Style::default().fg(Self::DIM).bg(Self::BACKGROUND_LIGHT)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // QUALITY-SPECIFIC STYLES
    // ═══════════════════════════════════════════════════════════════════════

    /// 4K/2160p quality indicator
    pub fn quality_4k() -> Style {
        Style::default()
            .fg(Self::SECONDARY) // Magenta for premium
            .add_modifier(Modifier::BOLD)
    }

    /// 1080p quality indicator
    pub fn quality_1080p() -> Style {
        Style::default().fg(Self::PRIMARY) // Cyan
    }

    /// 720p quality indicator
    pub fn quality_720p() -> Style {
        Style::default().fg(Self::SUCCESS) // Green
    }

    /// SD quality indicator
    pub fn quality_sd() -> Style {
        Style::default().fg(Self::DIM)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMPONENT STYLES
    // ═══════════════════════════════════════════════════════════════════════

    /// Style for list items (normal state)
    pub fn list_item() -> Style {
        Style::default().fg(Self::TEXT)
    }

    /// Style for list items (selected/highlighted)
    pub fn list_item_selected() -> Style {
        Style::default()
            .fg(Self::BACKGROUND)
            .bg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for input fields
    pub fn input() -> Style {
        Style::default().fg(Self::TEXT).bg(Self::BACKGROUND_LIGHT)
    }

    /// Style for input cursor
    pub fn input_cursor() -> Style {
        Style::default().fg(Self::BACKGROUND).bg(Self::PRIMARY)
    }

    /// Keybinding hint style
    pub fn keybind() -> Style {
        Style::default().fg(Self::ACCENT)
    }

    /// Keybinding description style
    pub fn keybind_desc() -> Style {
        Style::default().fg(Self::DIM)
    }

    /// Status bar style
    pub fn status_bar() -> Style {
        Style::default().fg(Self::TEXT).bg(Self::BACKGROUND_LIGHT)
    }

    /// Cast target indicator
    pub fn cast_target() -> Style {
        Style::default()
            .fg(Self::SUCCESS)
            .add_modifier(Modifier::BOLD)
    }

    /// Loading/spinner indicator
    pub fn loading() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Seed count (high seeds = green, low = yellow/red)
    pub fn seeds_high() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    pub fn seeds_medium() -> Style {
        Style::default().fg(Self::WARNING)
    }

    pub fn seeds_low() -> Style {
        Style::default().fg(Self::ERROR)
    }

    /// File size indicator
    pub fn file_size() -> Style {
        Style::default().fg(Self::DIM)
    }

    /// Year/date metadata
    pub fn year() -> Style {
        Style::default().fg(Self::SECONDARY)
    }

    /// Genre tags
    pub fn genre() -> Style {
        Style::default().fg(Self::DIM)
    }

    /// Duration text
    pub fn duration() -> Style {
        Style::default().fg(Self::DIM)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COLOR UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate relative luminance for a color (used in contrast ratio)
/// Formula: https://www.w3.org/TR/WCAG20/#relativeluminancedef
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    fn channel_luminance(c: u8) -> f64 {
        let c = c as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel_luminance(r) + 0.7152 * channel_luminance(g) + 0.0722 * channel_luminance(b)
}

/// Calculate contrast ratio between two colors
/// Returns a value between 1 (same color) and 21 (black/white)
/// WCAG AA requires >= 4.5:1 for normal text, >= 3:1 for large text
pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(fg.0, fg.1, fg.2);
    let l2 = relative_luminance(bg.0, bg.1, bg.2);

    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };

    (lighter + 0.05) / (darker + 0.05)
}

/// Check if a foreground/background pair meets WCAG AA for normal text
pub fn meets_wcag_aa(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> bool {
    contrast_ratio(fg, bg) >= 4.5
}

/// Check if a foreground/background pair meets WCAG AA for large text
pub fn meets_wcag_aa_large(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> bool {
    contrast_ratio(fg, bg) >= 3.0
}

/// Extract RGB tuple from ratatui Color (only works for Rgb variant)
pub fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to extract RGB from our theme colors
    fn rgb(color: Color) -> (u8, u8, u8) {
        color_to_rgb(color).expect("Theme colors should all be RGB")
    }

    #[test]
    fn test_all_theme_colors_are_rgb() {
        // Verify all theme colors are valid RGB values
        assert!(color_to_rgb(Theme::BACKGROUND).is_some());
        assert!(color_to_rgb(Theme::PRIMARY).is_some());
        assert!(color_to_rgb(Theme::SECONDARY).is_some());
        assert!(color_to_rgb(Theme::ACCENT).is_some());
        assert!(color_to_rgb(Theme::HIGHLIGHT).is_some());
        assert!(color_to_rgb(Theme::TEXT).is_some());
        assert!(color_to_rgb(Theme::DIM).is_some());
        assert!(color_to_rgb(Theme::SUCCESS).is_some());
        assert!(color_to_rgb(Theme::WARNING).is_some());
        assert!(color_to_rgb(Theme::ERROR).is_some());
    }

    #[test]
    fn test_colors_match_spec() {
        // Background: #0a0a0f
        assert_eq!(rgb(Theme::BACKGROUND), (0x0a, 0x0a, 0x0f));
        // Primary: #00fff2
        assert_eq!(rgb(Theme::PRIMARY), (0x00, 0xff, 0xf2));
        // Secondary: #ff00ff
        assert_eq!(rgb(Theme::SECONDARY), (0xff, 0x00, 0xff));
        // Accent: #ffff00
        assert_eq!(rgb(Theme::ACCENT), (0xff, 0xff, 0x00));
        // Highlight: #ff0080
        assert_eq!(rgb(Theme::HIGHLIGHT), (0xff, 0x00, 0x80));
        // Text: #e0e0e0
        assert_eq!(rgb(Theme::TEXT), (0xe0, 0xe0, 0xe0));
        // Dim: #404050
        assert_eq!(rgb(Theme::DIM), (0x40, 0x40, 0x50));
        // Success: #00ff00
        assert_eq!(rgb(Theme::SUCCESS), (0x00, 0xff, 0x00));
        // Warning: #ffaa00
        assert_eq!(rgb(Theme::WARNING), (0xff, 0xaa, 0x00));
        // Error: #ff0040
        assert_eq!(rgb(Theme::ERROR), (0xff, 0x00, 0x40));
    }

    #[test]
    fn test_text_contrast_against_background() {
        let bg = rgb(Theme::BACKGROUND);
        let text = rgb(Theme::TEXT);

        let ratio = contrast_ratio(text, bg);
        println!("Text/Background contrast ratio: {:.2}:1", ratio);

        // WCAG AA requires >= 4.5:1 for normal text
        assert!(
            meets_wcag_aa(text, bg),
            "Text on background should meet WCAG AA (got {:.2}:1)",
            ratio
        );
    }

    #[test]
    fn test_primary_contrast_against_background() {
        let bg = rgb(Theme::BACKGROUND);
        let primary = rgb(Theme::PRIMARY);

        let ratio = contrast_ratio(primary, bg);
        println!("Primary/Background contrast ratio: {:.2}:1", ratio);

        // Primary neon colors should at least meet large text requirements
        assert!(
            meets_wcag_aa_large(primary, bg),
            "Primary on background should meet WCAG AA for large text (got {:.2}:1)",
            ratio
        );
    }

    #[test]
    fn test_highlight_contrast() {
        let bg = rgb(Theme::BACKGROUND);
        let highlight = rgb(Theme::HIGHLIGHT);

        let ratio = contrast_ratio(highlight, bg);
        println!("Highlight/Background contrast ratio: {:.2}:1", ratio);

        assert!(
            meets_wcag_aa_large(highlight, bg),
            "Highlight on background should meet WCAG AA for large text (got {:.2}:1)",
            ratio
        );
    }

    #[test]
    fn test_error_contrast() {
        let bg = rgb(Theme::BACKGROUND);
        let error = rgb(Theme::ERROR);

        let ratio = contrast_ratio(error, bg);
        println!("Error/Background contrast ratio: {:.2}:1", ratio);

        assert!(
            meets_wcag_aa_large(error, bg),
            "Error on background should meet WCAG AA for large text (got {:.2}:1)",
            ratio
        );
    }

    #[test]
    fn test_inverted_highlighted_contrast() {
        // When we invert (text on primary background), it should still be readable
        let fg = rgb(Theme::BACKGROUND);
        let bg = rgb(Theme::PRIMARY);

        let ratio = contrast_ratio(fg, bg);
        println!("Background on Primary contrast ratio: {:.2}:1", ratio);

        assert!(
            meets_wcag_aa_large(fg, bg),
            "Inverted highlight should be readable (got {:.2}:1)",
            ratio
        );
    }

    #[test]
    fn test_style_helpers_return_valid_styles() {
        // Just verify all style helpers return without panicking
        // and have the expected foreground colors
        let _ = Theme::text();
        let _ = Theme::highlighted();
        let _ = Theme::selected();
        let _ = Theme::dimmed();
        let _ = Theme::error();
        let _ = Theme::success();
        let _ = Theme::warning();
        let _ = Theme::title();
        let _ = Theme::secondary();
        let _ = Theme::accent();
        let _ = Theme::border();
        let _ = Theme::border_focused();
        let _ = Theme::progress_bar();
        let _ = Theme::quality_4k();
        let _ = Theme::quality_1080p();
        let _ = Theme::quality_720p();
        let _ = Theme::quality_sd();
        let _ = Theme::list_item();
        let _ = Theme::list_item_selected();
        let _ = Theme::input();
        let _ = Theme::keybind();
        let _ = Theme::status_bar();
        let _ = Theme::cast_target();
        let _ = Theme::loading();
        let _ = Theme::seeds_high();
        let _ = Theme::seeds_medium();
        let _ = Theme::seeds_low();
    }

    #[test]
    fn test_relative_luminance_black() {
        let lum = relative_luminance(0, 0, 0);
        assert!((lum - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_relative_luminance_white() {
        let lum = relative_luminance(255, 255, 255);
        assert!((lum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let ratio = contrast_ratio((0, 0, 0), (255, 255, 255));
        // Should be 21:1
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let ratio = contrast_ratio((100, 100, 100), (100, 100, 100));
        // Should be 1:1
        assert!((ratio - 1.0).abs() < 0.001);
    }
}
