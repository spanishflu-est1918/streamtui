//! Torrent streaming via webtorrent-cli
//!
//! Manages webtorrent subprocess for streaming magnet links.
//! Provides progress updates and stream URL for casting.

use anyhow::Result;

/// Torrent streaming manager
pub struct TorrentManager {
    /// Optional path to webtorrent binary
    webtorrent_path: String,
}

/// Torrent download progress
#[derive(Debug, Clone)]
pub struct TorrentProgress {
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers: u32,
    pub stream_url: Option<String>,
}

impl TorrentManager {
    /// Create a new torrent manager
    pub fn new() -> Self {
        Self {
            webtorrent_path: "webtorrent".to_string(),
        }
    }

    /// Create with custom webtorrent path
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            webtorrent_path: path.into(),
        }
    }

    /// Start streaming a magnet link, returns stream URL
    /// Note: CLI commands use direct subprocess calls in commands.rs
    pub async fn stream(&self, _magnet: &str) -> Result<String> {
        // Stub - actual implementation in commands::cast_cmd spawns webtorrent directly
        anyhow::bail!("Not implemented")
    }

    /// Get current progress
    pub fn progress(&self) -> Option<TorrentProgress> {
        // Stub - progress parsed from webtorrent stdout in commands.rs
        None
    }

    /// Stop the current stream
    pub async fn stop(&self) -> Result<()> {
        // Stub - process killed directly in commands.rs
        Ok(())
    }
}

impl Default for TorrentManager {
    fn default() -> Self {
        Self::new()
    }
}
