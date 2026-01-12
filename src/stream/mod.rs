//! Streaming infrastructure
//!
//! - Torrent: webtorrent-cli integration for streaming torrents
//! - Cast: Chromecast discovery and control via catt
//! - Subtitles: OpenSubtitles API integration

pub mod torrent;
pub mod cast;
pub mod subtitles;

pub use torrent::TorrentManager;
pub use cast::CastManager;
pub use subtitles::SubtitleClient;
