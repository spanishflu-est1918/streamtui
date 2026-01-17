//! Stremio Subtitle Client
//!
//! Free subtitle search using Stremio's public addon endpoint.
//! No API key required - uses Stremio's OpenSubtitles v3 addon.
//!
//! Handles SRT to WebVTT conversion for Chromecast.
//! Caches downloaded subtitles in ~/.cache/streamtui/subtitles/

use crate::models::{SubFormat, SubtitleResult};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Subtitle client using Stremio's free public endpoint
///
/// Uses Stremio's OpenSubtitles v3 addon - no API key required!
pub struct SubtitleClient {
    base_url: String,
    client: reqwest::Client,
    cache_dir: PathBuf,
}

/// Stremio subtitle response
#[derive(Debug, Deserialize)]
struct StremioResponse {
    subtitles: Vec<StremioSubtitle>,
}

/// Single subtitle from Stremio
#[derive(Debug, Deserialize)]
struct StremioSubtitle {
    id: String,
    url: String,
    lang: String,
    #[serde(rename = "SubEncoding")]
    #[allow(dead_code)]
    sub_encoding: Option<String>,
}

impl SubtitleClient {
    /// Create a new subtitle client (free, no API key)
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("streamtui")
            .join("subtitles");

        Self {
            base_url: "https://opensubtitles-v3.strem.io".to_string(),
            client: reqwest::Client::new(),
            cache_dir,
        }
    }

    /// Create with custom base URL (for testing)
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("streamtui")
            .join("subtitles");

        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            cache_dir,
        }
    }

    /// Search for movie subtitles by IMDB ID
    ///
    /// # Arguments
    /// * `imdb_id` - IMDB ID (with or without "tt" prefix)
    /// * `language` - Optional 3-letter language code filter (e.g., "eng", "spa")
    pub async fn search(
        &self,
        imdb_id: &str,
        language: Option<&str>,
    ) -> Result<Vec<SubtitleResult>> {
        let imdb = normalize_imdb_id(imdb_id);
        let url = format!("{}/subtitles/movie/{}.json", self.base_url, imdb);
        self.fetch_subtitles(&url, language).await
    }

    /// Search for TV episode subtitles
    ///
    /// # Arguments
    /// * `imdb_id` - IMDB ID of the TV show
    /// * `season` - Season number
    /// * `episode` - Episode number
    /// * `language` - Optional 3-letter language code filter
    pub async fn search_episode(
        &self,
        imdb_id: &str,
        season: u16,
        episode: u16,
        language: Option<&str>,
    ) -> Result<Vec<SubtitleResult>> {
        let imdb = normalize_imdb_id(imdb_id);
        let url = format!(
            "{}/subtitles/series/{}:{}:{}.json",
            self.base_url, imdb, season, episode
        );
        self.fetch_subtitles(&url, language).await
    }

    /// Fetch and parse subtitles from Stremio endpoint
    async fn fetch_subtitles(
        &self,
        url: &str,
        language: Option<&str>,
    ) -> Result<Vec<SubtitleResult>> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Stremio API error: {}", response.status()));
        }

        let api_response: StremioResponse = response.json().await?;

        // Convert and optionally filter by language(s)
        // Supports comma-separated languages like "eng,spa"
        let langs: Vec<&str> = language
            .map(|l| l.split(',').map(|s| s.trim()).collect())
            .unwrap_or_default();

        let results: Vec<SubtitleResult> = api_response
            .subtitles
            .into_iter()
            .filter(|s| {
                langs.is_empty() || langs.iter().any(|lang| {
                    s.lang.eq_ignore_ascii_case(lang)
                        || s.lang.starts_with(lang)
                        || lang.starts_with(&s.lang)
                })
            })
            .map(|s| {
                // Extract release name from subtitle ID (format: "id|release_name" or just use ID)
                let release = extract_release_from_id(&s.id);
                SubtitleResult {
                    id: s.id.clone(),
                    url: s.url,
                    language: s.lang.clone(),
                    language_name: lang_code_to_name(&s.lang),
                    release,
                    fps: None,
                    format: SubFormat::Srt,
                    downloads: 0, // Stremio API doesn't provide download counts
                    from_trusted: true,
                    hearing_impaired: false,
                    ai_translated: false,
                }
            })
            .collect();

        Ok(results)
    }

    /// Download subtitle by ID - searches for the subtitle and downloads it
    ///
    /// Returns the path to the cached WebVTT file
    pub async fn download_by_id(
        &self,
        imdb_id: &str,
        subtitle_id: &str,
        season: Option<u16>,
        episode: Option<u16>,
    ) -> Result<PathBuf> {
        // Search for subtitles to find the one with matching ID
        let subtitles = if let (Some(s), Some(e)) = (season, episode) {
            self.search_episode(imdb_id, s, e, None).await?
        } else {
            self.search(imdb_id, None).await?
        };

        let subtitle = subtitles
            .into_iter()
            .find(|s| s.id == subtitle_id)
            .ok_or_else(|| anyhow!("Subtitle ID {} not found", subtitle_id))?;

        // Download and get the content (caches automatically)
        self.download(&subtitle).await?;

        // Return the cache path
        Ok(self.get_cache_path(&subtitle))
    }

    /// Download subtitle from URL and convert to WebVTT
    pub async fn download(&self, subtitle: &SubtitleResult) -> Result<String> {
        // Check cache first
        let cache_path = self.get_cache_path(subtitle);
        if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)?;
            return Ok(content);
        }

        // Download from Stremio
        let response = self.client.get(&subtitle.url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download subtitle: {}",
                response.status()
            ));
        }

        let srt_content = response.text().await?;

        // Convert to WebVTT
        let webvtt_content = Self::srt_to_webvtt(&srt_content);

        // Cache the result
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&cache_path, &webvtt_content)?;

        Ok(webvtt_content)
    }

    /// Get the cache path for a subtitle
    fn get_cache_path(&self, subtitle: &SubtitleResult) -> PathBuf {
        self.cache_dir
            .join(format!("{}_{}.vtt", subtitle.language, subtitle.id))
    }

    /// Convert SRT content to WebVTT format
    ///
    /// WebVTT is required for Chromecast subtitle playback.
    /// This converts SRT timestamps (00:00:00,000) to WebVTT format (00:00:00.000)
    /// and adds the required WEBVTT header.
    pub fn srt_to_webvtt(srt: &str) -> String {
        let mut webvtt = String::from("WEBVTT\n\n");

        // Process line by line, only converting timestamps (not dialogue text)
        for line in srt.lines() {
            let converted = if line.contains(" --> ") {
                // This is a timestamp line - convert commas to dots
                line.replace(',', ".")
            } else {
                // This is dialogue or cue number - preserve as-is
                line.to_string()
            };
            webvtt.push_str(&converted);
            webvtt.push('\n');
        }

        webvtt
    }
}

impl Default for SubtitleClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize IMDB ID to have "tt" prefix
fn normalize_imdb_id(imdb_id: &str) -> String {
    if imdb_id.starts_with("tt") {
        imdb_id.to_string()
    } else {
        format!("tt{}", imdb_id)
    }
}

/// Extract release name from Stremio subtitle ID
/// Stremio IDs can be: "12345678" (numeric) or contain embedded release info
fn extract_release_from_id(id: &str) -> String {
    // Check for pipe-separated format: "id|release_name"
    if let Some(idx) = id.find('|') {
        let release = &id[idx + 1..];
        if !release.is_empty() {
            // Clean up the release name (replace dots with spaces, truncate)
            return release
                .replace('.', " ")
                .trim()
                .chars()
                .take(50)
                .collect();
        }
    }

    // If ID looks like a release name (contains dots or dashes), use it
    if id.contains('.') || (id.contains('-') && !id.chars().all(|c| c.is_ascii_digit() || c == '-'))
    {
        return id.replace('.', " ").trim().chars().take(50).collect();
    }

    // Fallback to "OpenSubtitles" for pure numeric IDs
    "OpenSubtitles".to_string()
}

/// Convert 3-letter language code to full name
fn lang_code_to_name(code: &str) -> String {
    match code.to_lowercase().as_str() {
        "eng" => "English".to_string(),
        "spa" => "Spanish".to_string(),
        "fre" => "French".to_string(),
        "ger" => "German".to_string(),
        "ita" => "Italian".to_string(),
        "por" | "pob" => "Portuguese".to_string(),
        "rus" => "Russian".to_string(),
        "jpn" => "Japanese".to_string(),
        "kor" => "Korean".to_string(),
        "chi" | "zho" => "Chinese".to_string(),
        "ara" => "Arabic".to_string(),
        "hin" => "Hindi".to_string(),
        "dut" | "nld" => "Dutch".to_string(),
        "pol" => "Polish".to_string(),
        "tur" => "Turkish".to_string(),
        "swe" => "Swedish".to_string(),
        "nor" => "Norwegian".to_string(),
        "dan" => "Danish".to_string(),
        "fin" => "Finnish".to_string(),
        "gre" | "ell" => "Greek".to_string(),
        "heb" => "Hebrew".to_string(),
        "hun" => "Hungarian".to_string(),
        "cze" | "ces" => "Czech".to_string(),
        "rum" | "ron" => "Romanian".to_string(),
        "bul" => "Bulgarian".to_string(),
        "hrv" => "Croatian".to_string(),
        "slv" => "Slovenian".to_string(),
        "srp" => "Serbian".to_string(),
        "ukr" => "Ukrainian".to_string(),
        "vie" => "Vietnamese".to_string(),
        "tha" => "Thai".to_string(),
        "ind" => "Indonesian".to_string(),
        "may" | "msa" => "Malay".to_string(),
        "ice" | "isl" => "Icelandic".to_string(),
        _ => code.to_uppercase(),
    }
}
