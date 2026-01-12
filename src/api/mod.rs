//! API clients for external services
//!
//! - TMDB: Movie/TV metadata and search
//! - Torrentio: Stream sources via Stremio addon protocol

pub mod tmdb;
pub mod torrentio;

pub use tmdb::{TmdbClient, TmdbError};
pub use torrentio::TorrentioClient;
