//! Integration tests for StreamTUI
//!
//! Tests are organized by component:
//! - tmdb_test: TMDB API client tests
//! - torrentio_test: Torrentio addon tests
//! - torrent_test: Torrent manager tests
//! - cast_test: Chromecast manager tests
//! - subtitles_test: Subtitle client tests (OpenSubtitles API + conversion)
//! - ui_test: UI component tests
//! - e2e_test: End-to-end flow tests (Search -> Detail -> Sources -> Cast)

// Note: Each test file is a separate integration test crate
// Tests are run individually by cargo, not via mod.rs
