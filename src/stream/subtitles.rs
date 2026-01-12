//! OpenSubtitles API client
//!
//! Search and download subtitles from OpenSubtitles.
//! Handles SRT to WebVTT conversion for Chromecast.
//! Caches downloaded subtitles in ~/.cache/streamtui/subtitles/

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::{SubtitleResult, SubFormat};

/// OpenSubtitles API client
pub struct SubtitleClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
    cache_dir: PathBuf,
}

/// OpenSubtitles API response wrapper
#[derive(Debug, Deserialize)]
struct OpenSubtitlesResponse {
    data: Vec<OpenSubtitlesEntry>,
    #[allow(dead_code)]
    total_count: Option<u32>,
}

/// Single subtitle entry from OpenSubtitles API
#[derive(Debug, Deserialize)]
struct OpenSubtitlesEntry {
    id: String,
    attributes: OpenSubtitlesAttributes,
}

/// Subtitle attributes from OpenSubtitles API
#[derive(Debug, Deserialize)]
struct OpenSubtitlesAttributes {
    language: String,
    hearing_impaired: bool,
    ai_translated: bool,
    #[allow(dead_code)]
    machine_translated: bool,
    from_trusted: bool,
    download_count: u32,
    release: String,
    fps: Option<f32>,
    files: Vec<OpenSubtitlesFile>,
}

/// File info from OpenSubtitles API
#[derive(Debug, Deserialize)]
struct OpenSubtitlesFile {
    file_id: u64,
    #[allow(dead_code)]
    file_name: String,
}

/// Download response from OpenSubtitles API
#[derive(Debug, Deserialize)]
struct DownloadResponse {
    link: String,
    #[allow(dead_code)]
    file_name: String,
    #[allow(dead_code)]
    requests: Option<u32>,
    #[allow(dead_code)]
    remaining: Option<u32>,
}

impl SubtitleClient {
    /// Create a new subtitle client
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("streamtui")
            .join("subtitles");
        
        Self {
            base_url: "https://api.opensubtitles.com/api/v1".to_string(),
            api_key: None,
            client: reqwest::Client::new(),
            cache_dir,
        }
    }

    /// Create with API key for authenticated requests
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut client = Self::new();
        client.api_key = Some(api_key.into());
        client
    }

    /// Create with custom base URL (for testing)
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("streamtui")
            .join("subtitles");
        
        Self {
            base_url: base_url.into(),
            api_key: None,
            client: reqwest::Client::new(),
            cache_dir,
        }
    }

    /// Search for subtitles by IMDB ID
    /// 
    /// # Arguments
    /// * `imdb_id` - IMDB ID (with or without "tt" prefix)
    /// * `language` - Optional language code (e.g., "en", "es")
    pub async fn search(&self, imdb_id: &str, language: Option<&str>) -> Result<Vec<SubtitleResult>> {
        // Strip "tt" prefix and convert to number to remove leading zeros
        let imdb_num = imdb_id
            .trim_start_matches("tt")
            .parse::<u64>()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| imdb_id.trim_start_matches("tt").to_string());
        
        let mut url = format!("{}/subtitles?imdb_id={}", self.base_url, imdb_num);
        
        if let Some(lang) = language {
            url.push_str(&format!("&languages={}", lang));
        }
        
        let mut request = self.client.get(&url);
        
        // Add API key header if present
        if let Some(ref api_key) = self.api_key {
            request = request.header("Api-Key", api_key);
        }
        
        let response = request.send().await?;
        
        // Handle rate limiting (429)
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("Rate limit exceeded (429 Too Many Requests)"));
        }
        
        // Handle 404
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("Not found (404)"));
        }
        
        // Handle other errors
        if !response.status().is_success() {
            return Err(anyhow!("API error: {}", response.status()));
        }
        
        let api_response: OpenSubtitlesResponse = response.json().await?;
        
        Ok(self.convert_response(api_response))
    }

    /// Search for TV episode subtitles
    /// 
    /// # Arguments
    /// * `imdb_id` - IMDB ID of the TV show
    /// * `season` - Season number
    /// * `episode` - Episode number  
    /// * `language` - Optional language code
    pub async fn search_episode(
        &self,
        imdb_id: &str,
        season: u16,
        episode: u16,
        language: Option<&str>,
    ) -> Result<Vec<SubtitleResult>> {
        // Strip "tt" prefix and convert to number to remove leading zeros
        let imdb_num = imdb_id
            .trim_start_matches("tt")
            .parse::<u64>()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| imdb_id.trim_start_matches("tt").to_string());
        
        let mut url = format!(
            "{}/subtitles?imdb_id={}&season_number={}&episode_number={}",
            self.base_url, imdb_num, season, episode
        );
        
        if let Some(lang) = language {
            url.push_str(&format!("&languages={}", lang));
        }
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.api_key {
            request = request.header("Api-Key", api_key);
        }
        
        let response = request.send().await?;
        
        // Handle rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("Rate limit exceeded (429 Too Many Requests)"));
        }
        
        if !response.status().is_success() {
            return Err(anyhow!("API error: {}", response.status()));
        }
        
        let api_response: OpenSubtitlesResponse = response.json().await?;
        
        Ok(self.convert_response(api_response))
    }

    /// Convert OpenSubtitles API response to our model
    fn convert_response(&self, response: OpenSubtitlesResponse) -> Vec<SubtitleResult> {
        response.data.into_iter().filter_map(|entry| {
            // Get the first file (main subtitle file)
            let file = entry.attributes.files.first()?;
            
            // Determine format from filename
            let format = entry.attributes.files.first()
                .map(|f| {
                    let ext = f.file_name.rsplit('.').next().unwrap_or("srt");
                    SubFormat::from_extension(ext)
                })
                .unwrap_or(SubFormat::Srt);
            
            Some(SubtitleResult {
                id: entry.id,
                file_id: file.file_id,
                language: entry.attributes.language.clone(),
                language_name: language_code_to_name(&entry.attributes.language),
                release: entry.attributes.release,
                fps: entry.attributes.fps,
                format,
                downloads: entry.attributes.download_count,
                from_trusted: entry.attributes.from_trusted,
                hearing_impaired: entry.attributes.hearing_impaired,
                ai_translated: entry.attributes.ai_translated,
            })
        }).collect()
    }

    /// Download subtitle and return WebVTT content
    /// 
    /// Downloads the subtitle file, converts to WebVTT if needed,
    /// and caches the result.
    pub async fn download(&self, subtitle: &SubtitleResult) -> Result<String> {
        // Check cache first
        let cache_path = self.get_cache_path(subtitle);
        if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)?;
            return Ok(content);
        }
        
        // Request download link from OpenSubtitles
        let url = format!("{}/download", self.base_url);
        let body = serde_json::json!({
            "file_id": subtitle.file_id
        });
        
        let mut request = self.client.post(&url)
            .json(&body);
        
        if let Some(ref api_key) = self.api_key {
            request = request.header("Api-Key", api_key);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Download API error: {}", response.status()));
        }
        
        let download_info: DownloadResponse = response.json().await?;
        
        // Download the actual subtitle file
        let subtitle_response = self.client.get(&download_info.link).send().await?;
        
        if !subtitle_response.status().is_success() {
            return Err(anyhow!("Failed to download subtitle file"));
        }
        
        let srt_content = subtitle_response.text().await?;
        
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

    /// Get cached subtitle if exists
    pub fn get_cached(&self, subtitle: &SubtitleResult) -> Option<String> {
        let cache_path = self.get_cache_path(subtitle);
        std::fs::read_to_string(&cache_path).ok()
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

/// Convert language code to full name
fn language_code_to_name(code: &str) -> String {
    match code {
        "en" => "English".to_string(),
        "es" => "Spanish".to_string(),
        "fr" => "French".to_string(),
        "de" => "German".to_string(),
        "it" => "Italian".to_string(),
        "pt" => "Portuguese".to_string(),
        "ru" => "Russian".to_string(),
        "ja" => "Japanese".to_string(),
        "ko" => "Korean".to_string(),
        "zh" => "Chinese".to_string(),
        "ar" => "Arabic".to_string(),
        "hi" => "Hindi".to_string(),
        "nl" => "Dutch".to_string(),
        "pl" => "Polish".to_string(),
        "tr" => "Turkish".to_string(),
        "sv" => "Swedish".to_string(),
        "no" => "Norwegian".to_string(),
        "da" => "Danish".to_string(),
        "fi" => "Finnish".to_string(),
        "el" => "Greek".to_string(),
        _ => code.to_uppercase(),
    }
}
