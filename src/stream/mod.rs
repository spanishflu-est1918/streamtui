//! Streaming infrastructure
//!
//! - Torrent: webtorrent-cli integration for streaming torrents
//! - Cast: Chromecast discovery and control via catt
//! - Subtitles: OpenSubtitles API integration

pub mod cast;
pub mod subtitles;
pub mod torrent;

pub use subtitles::SubtitleClient;

// Re-export for TUI use (currently unused in CLI)
#[allow(unused_imports)]
pub use cast::CastManager;
#[allow(unused_imports)]
pub use torrent::TorrentManager;
