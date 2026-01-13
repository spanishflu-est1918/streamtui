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
//! - `app` - Application state and navigation
//! - `cli` - Command-line interface for automation

// Allow dead code for TUI components and models prepared for future interactive mode
#![allow(dead_code)]

pub mod api;
pub mod app;
pub mod cli;
pub mod models;
pub mod stream;
pub mod ui;

// Re-export commonly used types
pub use models::{
    CastDevice, CastState, Episode, MediaType, MovieDetail, PlaybackStatus, Quality, SearchResult,
    StreamSource, SubFormat, SubtitleFile, SubtitleResult, TorrentSession, TorrentState, TvDetail,
};

pub use api::{TmdbClient, TorrentioClient};
pub use app::{App, AppState};
