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
    pub async fn discover(&self) -> Result<Vec<CastDevice>> {
        // TODO: Implement catt scan
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
    pub async fn cast(&self, _url: &str, _subtitle_url: Option<&str>) -> Result<()> {
        // TODO: Implement catt cast
        anyhow::bail!("No device selected")
    }

    /// Get playback status
    pub async fn status(&self) -> Result<PlaybackStatus> {
        // TODO: Implement catt status
        anyhow::bail!("Not implemented")
    }

    /// Play/resume
    pub async fn play(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    /// Pause
    pub async fn pause(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    /// Stop
    pub async fn stop(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    /// Seek to position (seconds)
    pub async fn seek(&self, _position: f64) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    /// Set volume (0.0 - 1.0)
    pub async fn volume(&self, _level: f32) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
}

impl Default for CastManager {
    fn default() -> Self {
        Self::new()
    }
}
