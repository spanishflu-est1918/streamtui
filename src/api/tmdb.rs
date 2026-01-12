//! TMDB (The Movie Database) API client
//!
//! Provides search and metadata for movies and TV shows.
//! API docs: https://developer.themoviedb.org/docs

use anyhow::Result;
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

use crate::models::{Episode, MediaType, MovieDetail, SearchResult, SeasonSummary, TvDetail};

/// TMDB API error types
#[derive(Error, Debug)]
pub enum TmdbError {
    #[error("Resource not found (404)")]
    NotFound,

    #[error("Rate limited (429), retries exhausted")]
    RateLimited,

    #[error("Server error: {0}")]
    ServerError(u16),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}

/// TMDB API client
pub struct TmdbClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    max_retries: u32,
}

impl TmdbClient {
    /// Create a new TMDB client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.themoviedb.org/3".to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            max_retries: 3,
        }
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            max_retries: 3,
        }
    }

    /// Make an authenticated GET request with retry logic for rate limits
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut retries = 0;

        loop {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Accept", "application/json")
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let body = response.text().await?;
                    let parsed: T = serde_json::from_str(&body).map_err(|e| {
                        TmdbError::InvalidResponse(format!("JSON parse error: {}", e))
                    })?;
                    return Ok(parsed);
                }
                StatusCode::NOT_FOUND => {
                    return Err(TmdbError::NotFound.into());
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    retries += 1;
                    if retries >= self.max_retries {
                        return Err(TmdbError::RateLimited.into());
                    }

                    // Get Retry-After header or default to exponential backoff
                    let wait_secs = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(2u64.pow(retries));

                    tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                    continue;
                }
                status if status.is_server_error() => {
                    return Err(TmdbError::ServerError(status.as_u16()).into());
                }
                status => {
                    return Err(TmdbError::ServerError(status.as_u16()).into());
                }
            }
        }
    }

    /// Search for movies and TV shows
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let endpoint = format!(
            "/search/multi?query={}&page=1",
            urlencoding::encode(query)
        );

        let response: SearchResponse = self.get(&endpoint).await?;
        Ok(response.into_results())
    }

    /// Get trending content
    pub async fn trending(&self) -> Result<Vec<SearchResult>> {
        let response: SearchResponse = self.get("/trending/all/week").await?;
        Ok(response.into_results())
    }

    /// Get movie details by ID
    pub async fn movie_detail(&self, id: u64) -> Result<MovieDetail> {
        let endpoint = format!("/movie/{}?append_to_response=external_ids", id);
        let response: MovieResponse = self.get(&endpoint).await?;
        Ok(response.into_detail())
    }

    /// Get TV show details by ID
    pub async fn tv_detail(&self, id: u64) -> Result<TvDetail> {
        let endpoint = format!("/tv/{}?append_to_response=external_ids", id);
        let response: TvResponse = self.get(&endpoint).await?;
        Ok(response.into_detail())
    }

    /// Get episodes for a TV season
    pub async fn tv_season(&self, id: u64, season: u8) -> Result<Vec<Episode>> {
        let endpoint = format!("/tv/{}/season/{}", id, season);
        let response: SeasonResponse = self.get(&endpoint).await?;
        Ok(response.into_episodes(season))
    }

    // Legacy method names for backwards compatibility
    
    /// Get movie details (legacy name)
    pub async fn movie(&self, id: &str) -> Result<SearchResult> {
        let id: u64 = id.parse().map_err(|_| TmdbError::InvalidResponse("Invalid movie ID".into()))?;
        let detail = self.movie_detail(id).await?;
        Ok(SearchResult {
            id: detail.id,
            media_type: MediaType::Movie,
            title: detail.title,
            year: Some(detail.year),
            overview: detail.overview,
            poster_path: detail.poster_path,
            vote_average: detail.vote_average,
        })
    }

    /// Get TV show details (legacy name)
    pub async fn tv_show(&self, id: &str) -> Result<SearchResult> {
        let id: u64 = id.parse().map_err(|_| TmdbError::InvalidResponse("Invalid TV ID".into()))?;
        let detail = self.tv_detail(id).await?;
        Ok(SearchResult {
            id: detail.id,
            media_type: MediaType::Tv,
            title: detail.name,
            year: Some(detail.year),
            overview: detail.overview,
            poster_path: detail.poster_path,
            vote_average: detail.vote_average,
        })
    }

    /// Get TV show seasons (legacy name)
    pub async fn seasons(&self, id: &str) -> Result<Vec<SeasonSummary>> {
        let id: u64 = id.parse().map_err(|_| TmdbError::InvalidResponse("Invalid TV ID".into()))?;
        let detail = self.tv_detail(id).await?;
        Ok(detail.seasons)
    }

    /// Get episodes for a season (legacy name)
    pub async fn episodes(&self, id: &str, season: u16) -> Result<Vec<Episode>> {
        let id: u64 = id.parse().map_err(|_| TmdbError::InvalidResponse("Invalid TV ID".into()))?;
        self.tv_season(id, season as u8).await
    }
}

// =============================================================================
// Response Structures (internal deserialization)
// =============================================================================

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResultRaw>,
}

impl SearchResponse {
    fn into_results(self) -> Vec<SearchResult> {
        self.results
            .into_iter()
            .filter_map(|r| r.into_search_result())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct SearchResultRaw {
    id: u64,
    media_type: String,
    // Movies use "title", TV uses "name"
    title: Option<String>,
    name: Option<String>,
    // Movies use "release_date", TV uses "first_air_date"
    release_date: Option<String>,
    first_air_date: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    vote_average: Option<f32>,
}

impl SearchResultRaw {
    fn into_search_result(self) -> Option<SearchResult> {
        let media_type = match self.media_type.as_str() {
            "movie" => MediaType::Movie,
            "tv" => MediaType::Tv,
            _ => return None, // Filter out "person" and other types
        };

        let title = self.title.or(self.name).unwrap_or_default();
        let date_str = self.release_date.or(self.first_air_date);
        let year = date_str.and_then(|d| extract_year(&d));

        Some(SearchResult {
            id: self.id,
            media_type,
            title,
            year,
            overview: self.overview.unwrap_or_default(),
            poster_path: self.poster_path,
            vote_average: self.vote_average.unwrap_or(0.0),
        })
    }
}

#[derive(Debug, Deserialize)]
struct MovieResponse {
    id: u64,
    imdb_id: Option<String>,
    title: String,
    release_date: Option<String>,
    runtime: Option<u32>,
    genres: Vec<GenreRaw>,
    overview: Option<String>,
    vote_average: Option<f32>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

impl MovieResponse {
    fn into_detail(self) -> MovieDetail {
        let year = self
            .release_date
            .as_ref()
            .and_then(|d| extract_year(d))
            .unwrap_or(0);

        MovieDetail {
            id: self.id,
            imdb_id: self.imdb_id.unwrap_or_default(),
            title: self.title,
            year,
            runtime: self.runtime.unwrap_or(0),
            genres: self.genres.into_iter().map(|g| g.name).collect(),
            overview: self.overview.unwrap_or_default(),
            vote_average: self.vote_average.unwrap_or(0.0),
            poster_path: self.poster_path,
            backdrop_path: self.backdrop_path,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TvResponse {
    id: u64,
    name: String,
    first_air_date: Option<String>,
    seasons: Vec<SeasonRaw>,
    genres: Vec<GenreRaw>,
    overview: Option<String>,
    vote_average: Option<f32>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    external_ids: Option<ExternalIds>,
}

impl TvResponse {
    fn into_detail(self) -> TvDetail {
        let year = self
            .first_air_date
            .as_ref()
            .and_then(|d| extract_year(d))
            .unwrap_or(0);

        let imdb_id = self
            .external_ids
            .and_then(|e| e.imdb_id)
            .unwrap_or_default();

        // Filter out specials (season 0)
        let seasons: Vec<SeasonSummary> = self
            .seasons
            .into_iter()
            .filter(|s| s.season_number > 0)
            .map(|s| s.into_summary())
            .collect();

        TvDetail {
            id: self.id,
            imdb_id,
            name: self.name,
            year,
            seasons,
            genres: self.genres.into_iter().map(|g| g.name).collect(),
            overview: self.overview.unwrap_or_default(),
            vote_average: self.vote_average.unwrap_or(0.0),
            poster_path: self.poster_path,
            backdrop_path: self.backdrop_path,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SeasonResponse {
    episodes: Vec<EpisodeRaw>,
}

impl SeasonResponse {
    fn into_episodes(self, season: u8) -> Vec<Episode> {
        self.episodes
            .into_iter()
            .map(|e| e.into_episode(season))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct GenreRaw {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SeasonRaw {
    season_number: u8,
    episode_count: u16,
    name: Option<String>,
    air_date: Option<String>,
}

impl SeasonRaw {
    fn into_summary(self) -> SeasonSummary {
        SeasonSummary {
            season_number: self.season_number,
            episode_count: self.episode_count,
            name: self.name,
            air_date: self.air_date,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EpisodeRaw {
    episode_number: u8,
    name: String,
    overview: Option<String>,
    runtime: Option<u32>,
}

impl EpisodeRaw {
    fn into_episode(self, season: u8) -> Episode {
        Episode {
            season,
            episode: self.episode_number,
            name: self.name,
            overview: self.overview.unwrap_or_default(),
            runtime: self.runtime,
            imdb_id: None, // TMDB doesn't provide episode-level IMDB IDs in season endpoint
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExternalIds {
    imdb_id: Option<String>,
}

/// Extract year from a date string like "2022-03-04"
fn extract_year(date: &str) -> Option<u16> {
    if date.len() >= 4 {
        date[..4].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_year() {
        assert_eq!(extract_year("2022-03-04"), Some(2022));
        assert_eq!(extract_year("2019-11-12"), Some(2019));
        assert_eq!(extract_year(""), None);
        assert_eq!(extract_year("abc"), None);
    }

    #[test]
    fn test_media_type_filter() {
        let movie = SearchResultRaw {
            id: 1,
            media_type: "movie".to_string(),
            title: Some("Test".to_string()),
            name: None,
            release_date: Some("2022-01-01".to_string()),
            first_air_date: None,
            overview: None,
            poster_path: None,
            vote_average: None,
        };

        let person = SearchResultRaw {
            id: 2,
            media_type: "person".to_string(),
            title: None,
            name: Some("Actor".to_string()),
            release_date: None,
            first_air_date: None,
            overview: None,
            poster_path: None,
            vote_average: None,
        };

        assert!(movie.into_search_result().is_some());
        assert!(person.into_search_result().is_none());
    }
}
