//! Chromecast control via catt CLI
//!
//! Discovers Chromecast devices and controls playback using catt.
//! catt provides a simpler interface than native Cast protocol.

use crate::models::{CastDevice, PlaybackStatus};
use anyhow::Result;

/// Chromecast manager using catt CLI
pub struct CastManager {
    /// Path to catt binary
    catt_path: String,
    /// Currently selected device
    selected_device: Option<CastDevice>,
}

impl CastManager {
    /// Create a new cast manager
    pub fn new() -> Self {
        Self {
            catt_path: "catt".to_string(),
            selected_device: None,
        }
    }

    /// Create with custom catt path
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            catt_path: path.into(),
            selected_device: None,
        }
    }

    /// Discover available Chromecast devices
    /// Note: CLI commands use direct subprocess calls in commands.rs
    pub async fn discover(&self) -> Result<Vec<CastDevice>> {
        // Stub - actual implementation in commands::devices_cmd
        Ok(vec![])
    }

    /// Select a device for casting
    pub fn select_device(&mut self, device: CastDevice) {
        self.selected_device = Some(device);
    }

    /// Get currently selected device
    pub fn selected(&self) -> Option<&CastDevice> {
        self.selected_device.as_ref()
    }

    /// Cast a URL to the selected device
    /// Note: CLI commands use direct subprocess calls in commands.rs
    pub async fn cast(&self, _url: &str, _subtitle_url: Option<&str>) -> Result<()> {
        // Stub - actual implementation in commands::cast_cmd
        anyhow::bail!("No device selected")
    }

    /// Get playback status
    /// Note: CLI commands use direct subprocess calls in commands.rs
    pub async fn status(&self) -> Result<PlaybackStatus> {
        // Stub - actual implementation in commands::status_cmd
        anyhow::bail!("Not implemented")
    }

    /// Play/resume
    pub async fn play(&self) -> Result<()> {
        // Stub - actual implementation in commands::playback_control
        Ok(())
    }

    /// Pause
    pub async fn pause(&self) -> Result<()> {
        // Stub - actual implementation in commands::playback_control
        Ok(())
    }

    /// Stop
    pub async fn stop(&self) -> Result<()> {
        // Stub - actual implementation in commands::playback_control
        Ok(())
    }

    /// Seek to position (seconds)
    pub async fn seek(&self, _position: f64) -> Result<()> {
        // Stub - actual implementation in commands::seek_cmd
        Ok(())
    }

    /// Set volume (0.0 - 1.0)
    pub async fn volume(&self, _level: f32) -> Result<()> {
        // Stub - actual implementation in commands::volume_cmd
        Ok(())
    }
}

impl Default for CastManager {
    fn default() -> Self {
        Self::new()
    }
}
