//! StreamTUI - Cyberpunk TUI for streaming to Chromecast
//!
//! A neon-soaked terminal interface for searching content, selecting quality,
//! and casting to your TV. Simple. Fast. Beautiful.
//!
//! # Modules
//!
//! - `models` - Data structures for search results, streams, playback
//! - `api` - API clients (TMDB, Torrentio)
//! - `stream` - Torrent and cast managers
//! - `ui` - TUI components

pub mod models;
pub mod api;
pub mod stream;
pub mod ui;

// Re-export commonly used types
pub use models::{
    Quality, StreamSource, SearchResult, MediaType,
    MovieDetail, TvDetail, Episode,
    TorrentSession, TorrentState,
    CastDevice, CastState, PlaybackStatus,
    SubtitleResult, SubtitleFile, SubFormat,
};

pub use api::{TmdbClient, TorrentioClient};
