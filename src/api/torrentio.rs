//! Torrentio addon client
//!
//! Fetches stream sources from the Torrentio Stremio addon.
//! Provides magnet links with quality, size, and seed info.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::models::{Quality, StreamSource};

/// Torrentio API response
#[derive(Debug, Deserialize)]
struct TorrentioResponse {
    streams: Vec<TorrentioStream>,
}

/// Individual stream from Torrentio
#[derive(Debug, Deserialize)]
struct TorrentioStream {
    name: String,
    title: String,
    #[serde(rename = "infoHash")]
    info_hash: String,
    #[serde(rename = "fileIdx")]
    file_idx: Option<u32>,
}

impl TorrentioStream {
    /// Convert API response to our StreamSource model
    fn into_stream_source(self) -> StreamSource {
        let quality = Quality::from_str_loose(&self.name);
        let seeds = StreamSource::parse_seeds(&self.title);
        let size_bytes = StreamSource::parse_size(&self.title);

        StreamSource {
            name: self.name,
            title: self.title,
            info_hash: self.info_hash,
            file_idx: self.file_idx,
            seeds,
            quality,
            size_bytes,
        }
    }
}

/// Torrentio addon client
pub struct TorrentioClient {
    base_url: String,
    client: reqwest::Client,
}

impl TorrentioClient {
    /// Create a new Torrentio client with default settings
    pub fn new() -> Self {
        Self {
            base_url: "https://torrentio.strem.fun".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Get streams for a movie by IMDB ID
    pub async fn movie_streams(&self, imdb_id: &str) -> Result<Vec<StreamSource>> {
        let url = format!("{}/stream/movie/{}.json", self.base_url, imdb_id);
        self.fetch_streams(&url).await
    }

    /// Get streams for a TV episode by IMDB ID and episode info
    pub async fn episode_streams(
        &self,
        imdb_id: &str,
        season: u16,
        episode: u16,
    ) -> Result<Vec<StreamSource>> {
        let url = format!(
            "{}/stream/series/{}:{}:{}.json",
            self.base_url, imdb_id, season, episode
        );
        self.fetch_streams(&url).await
    }

    /// Fetch and parse streams from a Torrentio URL
    async fn fetch_streams(&self, url: &str) -> Result<Vec<StreamSource>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch from Torrentio")?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("Torrentio returned HTTP {}", status);
        }

        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        let data: TorrentioResponse =
            serde_json::from_str(&text).context("Failed to parse JSON response")?;

        // Convert to StreamSource and sort by quality then seeds
        let mut streams: Vec<StreamSource> = data
            .streams
            .into_iter()
            .map(|s| s.into_stream_source())
            .collect();

        // Sort: quality descending, then seeds descending within same quality
        streams.sort_by(|a, b| match b.quality.cmp(&a.quality) {
            std::cmp::Ordering::Equal => b.seeds.cmp(&a.seeds),
            other => other,
        });

        Ok(streams)
    }
}

impl Default for TorrentioClient {
    fn default() -> Self {
        Self::new()
    }
}
