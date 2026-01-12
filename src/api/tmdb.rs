//! TMDB (The Movie Database) API client
//!
//! Provides search and metadata for movies and TV shows.
//! API docs: https://developer.themoviedb.org/docs

use anyhow::Result;
use crate::models::{SearchResult, SeasonSummary, Episode};

/// TMDB API client
pub struct TmdbClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl TmdbClient {
    /// Create a new TMDB client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.themoviedb.org/3".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Search for movies and TV shows
    pub async fn search(&self, _query: &str) -> Result<Vec<SearchResult>> {
        // TODO: Implement
        Ok(vec![])
    }

    /// Get trending content
    pub async fn trending(&self) -> Result<Vec<SearchResult>> {
        // TODO: Implement
        Ok(vec![])
    }

    /// Get movie details by ID
    pub async fn movie(&self, _id: &str) -> Result<SearchResult> {
        // TODO: Implement
        anyhow::bail!("Not implemented")
    }

    /// Get TV show details by ID
    pub async fn tv_show(&self, _id: &str) -> Result<SearchResult> {
        // TODO: Implement
        anyhow::bail!("Not implemented")
    }

    /// Get TV show seasons
    pub async fn seasons(&self, _id: &str) -> Result<Vec<SeasonSummary>> {
        // TODO: Implement
        Ok(vec![])
    }

    /// Get episodes for a season
    pub async fn episodes(&self, _id: &str, _season: u16) -> Result<Vec<Episode>> {
        // TODO: Implement
        Ok(vec![])
    }
}
