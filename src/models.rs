//! Data structures and types for StreamTUI
//!
//! Contains all shared models used across the application organized by domain:
//! - **Search**: TMDB search results and media details
//! - **Addons**: Torrentio stream sources and quality info
//! - **Torrent**: Session management and streaming state
//! - **Cast**: Chromecast device info and playback control
//! - **Subtitles**: OpenSubtitles search and download

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

// =============================================================================
// Search Models (TMDB)
// =============================================================================

/// Media type discriminator for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Tv,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::Movie => write!(f, "Movie"),
            MediaType::Tv => write!(f, "TV Show"),
        }
    }
}

/// Search result from TMDB multi-search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub media_type: MediaType,
    pub title: String,
    pub year: Option<u16>,
    pub overview: String,
    pub poster_path: Option<String>,
    pub vote_average: f32,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year_str = self.year.map(|y| format!(" ({})", y)).unwrap_or_default();
        write!(f, "{}{} [{}]", self.title, year_str, self.media_type)
    }
}

/// Summary of a TV season (used in TvDetail)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSummary {
    pub season_number: u8,
    pub episode_count: u16,
    pub name: Option<String>,
    pub air_date: Option<String>,
}

impl fmt::Display for SeasonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("Season");
        write!(
            f,
            "{} {} ({} episodes)",
            name, self.season_number, self.episode_count
        )
    }
}

/// Detailed movie information from TMDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetail {
    pub id: u64,
    pub imdb_id: String,
    pub title: String,
    pub year: u16,
    pub runtime: u32,
    pub genres: Vec<String>,
    pub overview: String,
    pub vote_average: f32,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

impl fmt::Display for MovieDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.runtime / 60;
        let mins = self.runtime % 60;
        write!(
            f,
            "{} ({}) - {}h {}m - ⭐ {:.1}",
            self.title, self.year, hours, mins, self.vote_average
        )
    }
}

/// Detailed TV show information from TMDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvDetail {
    pub id: u64,
    pub imdb_id: String,
    pub name: String,
    pub year: u16,
    pub seasons: Vec<SeasonSummary>,
    pub genres: Vec<String>,
    pub overview: String,
    pub vote_average: f32,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

impl fmt::Display for TvDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) - {} seasons - ⭐ {:.1}",
            self.name,
            self.year,
            self.seasons.len(),
            self.vote_average
        )
    }
}

/// TV episode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub season: u8,
    pub episode: u8,
    pub name: String,
    pub overview: String,
    pub runtime: Option<u32>,
    pub imdb_id: Option<String>,
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{:02}E{:02} - {}", self.season, self.episode, self.name)
    }
}

// =============================================================================
// Addon Models (Torrentio/Stremio)
// =============================================================================

/// Video quality classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Quality {
    UHD4K,
    FHD1080p,
    HD720p,
    SD480p,
    #[default]
    Unknown,
}

impl Quality {
    /// Parse quality from a string (e.g., "4K", "1080p", "720p")
    pub fn from_str_loose(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        if s_lower.contains("4k") || s_lower.contains("2160p") || s_lower.contains("uhd") {
            Quality::UHD4K
        } else if s_lower.contains("1080p") || s_lower.contains("fhd") {
            Quality::FHD1080p
        } else if s_lower.contains("720p") || s_lower.contains("hd") && !s_lower.contains("hdcam") {
            Quality::HD720p
        } else if s_lower.contains("480p") || s_lower.contains("sd") {
            Quality::SD480p
        } else {
            Quality::Unknown
        }
    }

    /// Quality ranking for sorting (higher = better)
    pub fn rank(&self) -> u8 {
        match self {
            Quality::UHD4K => 4,
            Quality::FHD1080p => 3,
            Quality::HD720p => 2,
            Quality::SD480p => 1,
            Quality::Unknown => 0,
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quality::UHD4K => write!(f, "4K"),
            Quality::FHD1080p => write!(f, "1080p"),
            Quality::HD720p => write!(f, "720p"),
            Quality::SD480p => write!(f, "480p"),
            Quality::Unknown => write!(f, "???"),
        }
    }
}

impl Ord for Quality {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Quality {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Stream source from Torrentio or other Stremio addons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSource {
    pub name: String,
    pub title: String,
    pub info_hash: String,
    pub file_idx: Option<u32>,
    pub seeds: u32,
    pub quality: Quality,
    pub size_bytes: Option<u64>,
}

impl StreamSource {
    /// Generate magnet URL for this stream
    pub fn to_magnet(&self, display_name: &str) -> String {
        format!(
            "magnet:?xt=urn:btih:{}&dn={}",
            self.info_hash,
            urlencoding::encode(display_name)
        )
    }

    /// Parse seeds from title string (e.g., "👤 142" or "👤 1.2k")
    pub fn parse_seeds(title: &str) -> u32 {
        // Try emoji format first: 👤 123 or 👤 1.2k
        let re = regex::Regex::new(r"👤\s*(\d+(?:\.\d+)?)\s*(k)?").ok();
        if let Some(re) = re {
            if let Some(caps) = re.captures(title) {
                let num: f32 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0.0);
                let multiplier = if caps.get(2).is_some() { 1000.0 } else { 1.0 };
                return (num * multiplier) as u32;
            }
        }

        // Try "seeds: N" format
        let re_seeds = regex::Regex::new(r"seeds?:\s*(\d+)").ok();
        if let Some(re) = re_seeds {
            if let Some(caps) = re.captures(&title.to_lowercase()) {
                return caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
            }
        }

        0
    }

    /// Parse size from title string (e.g., "4.2 GB" or "890 MB")
    pub fn parse_size(title: &str) -> Option<u64> {
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(GB|MB|gb|mb)").ok()?;
        let caps = re.captures(title)?;
        let num: f64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str().to_uppercase();

        let bytes = match unit.as_str() {
            "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "MB" => (num * 1024.0 * 1024.0) as u64,
            _ => return None,
        };

        Some(bytes)
    }

    /// Format size for display
    pub fn format_size(&self) -> String {
        match self.size_bytes {
            Some(bytes) if bytes >= 1024 * 1024 * 1024 => {
                format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
            Some(bytes) if bytes >= 1024 * 1024 => {
                format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
            }
            Some(bytes) => format!("{} KB", bytes / 1024),
            None => "? GB".to_string(),
        }
    }
}

impl fmt::Display for StreamSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} 👤{} {}",
            self.quality,
            self.format_size(),
            self.seeds,
            self.title.lines().next().unwrap_or(&self.title)
        )
    }
}

// =============================================================================
// Torrent Models
// =============================================================================

/// State of a torrent streaming session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorrentState {
    /// Launching webtorrent process
    Starting,
    /// Fetching torrent metadata from DHT/trackers
    FetchingMetadata { peers: u32 },
    /// Connecting to peers for download
    Connecting { peers: u32 },
    /// Buffering initial data for playback
    Buffering { peers: u32, progress: u8 },
    /// Downloading but not yet streaming
    Downloading,
    /// Actively streaming to Chromecast
    Streaming,
    /// Playback paused
    Paused,
    /// Stopped by user
    Stopped,
    /// Error occurred
    Error(String),
}

impl TorrentState {
    /// Get peer count if available
    pub fn peers(&self) -> Option<u32> {
        match self {
            TorrentState::FetchingMetadata { peers } => Some(*peers),
            TorrentState::Connecting { peers } => Some(*peers),
            TorrentState::Buffering { peers, .. } => Some(*peers),
            _ => None,
        }
    }

    /// Check if state is in connecting phase (not yet streaming)
    pub fn is_connecting(&self) -> bool {
        matches!(
            self,
            TorrentState::Starting
                | TorrentState::FetchingMetadata { .. }
                | TorrentState::Connecting { .. }
                | TorrentState::Buffering { .. }
        )
    }
}

impl fmt::Display for TorrentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TorrentState::Starting => write!(f, "Starting..."),
            TorrentState::FetchingMetadata { peers } => {
                write!(f, "Fetching metadata ({} peers)", peers)
            }
            TorrentState::Connecting { peers } => {
                write!(f, "Connecting ({} peers)", peers)
            }
            TorrentState::Buffering { peers, progress } => {
                write!(f, "Buffering {}% ({} peers)", progress, peers)
            }
            TorrentState::Downloading => write!(f, "Downloading"),
            TorrentState::Streaming => write!(f, "Streaming"),
            TorrentState::Paused => write!(f, "Paused"),
            TorrentState::Stopped => write!(f, "Stopped"),
            TorrentState::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

/// Active torrent streaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSession {
    pub id: Uuid,
    pub magnet: String,
    pub file_idx: Option<u32>,
    pub state: TorrentState,
    pub stream_url: Option<String>,
    pub progress: f32,
    pub download_speed: u64,
    pub downloaded: u64,
}

impl TorrentSession {
    /// Create a new session in Starting state
    pub fn new(magnet: String, file_idx: Option<u32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            magnet,
            file_idx,
            state: TorrentState::Starting,
            stream_url: None,
            progress: 0.0,
            download_speed: 0,
            downloaded: 0,
        }
    }

    /// Generate stream URL for casting
    pub fn generate_stream_url(lan_ip: IpAddr, port: u16, file_idx: u32) -> String {
        format!("http://{}:{}/{}", lan_ip, port, file_idx)
    }

    /// Format download speed for display
    pub fn format_speed(&self) -> String {
        let mb_per_sec = self.download_speed as f64 / (1024.0 * 1024.0);
        format!("{:.1} MB/s", mb_per_sec)
    }

    /// Format downloaded amount for display
    pub fn format_downloaded(&self) -> String {
        let gb = self.downloaded as f64 / (1024.0 * 1024.0 * 1024.0);
        if gb >= 1.0 {
            format!("{:.2} GB", gb)
        } else {
            let mb = self.downloaded as f64 / (1024.0 * 1024.0);
            format!("{:.0} MB", mb)
        }
    }

    /// Parse progress from webtorrent output (e.g., "Downloaded: 1.2 GB" with total 4.0 GB)
    pub fn parse_progress(downloaded_str: &str, total_bytes: u64) -> f32 {
        if total_bytes == 0 {
            return 0.0;
        }

        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(GB|MB)").ok();
        if let Some(re) = re {
            if let Some(caps) = re.captures(downloaded_str) {
                let num: f64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0.0);
                let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("MB");
                let downloaded = match unit {
                    "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
                    _ => (num * 1024.0 * 1024.0) as u64,
                };
                return (downloaded as f32) / (total_bytes as f32);
            }
        }

        0.0
    }

    /// Parse download speed from webtorrent output (e.g., "Speed: 5.2 MB/s")
    pub fn parse_speed(speed_str: &str) -> u64 {
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(MB|KB)/s").ok();
        if let Some(re) = re {
            if let Some(caps) = re.captures(speed_str) {
                let num: f64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0.0);
                let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("MB");
                return match unit {
                    "MB" => (num * 1024.0 * 1024.0) as u64,
                    "KB" => (num * 1024.0) as u64,
                    _ => 0,
                };
            }
        }
        0
    }
}

impl fmt::Display for TorrentSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {:.0}% @ {}",
            self.state,
            self.progress * 100.0,
            self.format_speed()
        )
    }
}

// =============================================================================
// Cast Models (Chromecast)
// =============================================================================

/// Chromecast device discovered on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
    pub model: Option<String>,
}

impl CastDevice {
    /// Parse devices from catt scan output
    /// Format: "192.168.1.36 - Device Name - Google Inc. Chromecast"
    pub fn parse_catt_scan(output: &str) -> Vec<CastDevice> {
        let mut devices = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with("Scanning")
                || line.contains("No devices")
            {
                continue;
            }

            // Parse "IP - Name - Model" format (catt 0.13+ output)
            let parts: Vec<&str> = line.splitn(3, " - ").collect();
            if parts.len() >= 2 {
                let ip_str = parts[0].trim();
                if let Ok(addr) = ip_str.parse::<IpAddr>() {
                    let name = parts[1].trim().to_string();
                    let model = if parts.len() >= 3 {
                        Some(parts[2].trim().to_string())
                    } else {
                        None
                    };
                    devices.push(CastDevice {
                        id: ip_str.to_string(),
                        name,
                        address: addr,
                        port: 8009, // Default Chromecast port
                        model,
                    });
                }
            }
        }

        devices
    }
}

impl fmt::Display for CastDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.model {
            Some(model) => write!(f, "{} ({}) - {}", self.name, model, self.address),
            None => write!(f, "{} - {}", self.name, self.address),
        }
    }
}

/// Chromecast playback state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastState {
    Idle,
    Connecting,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Error(String),
}

impl CastState {
    /// Parse state from catt status output
    pub fn from_catt_state(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "PLAYING" => CastState::Playing,
            "PAUSED" => CastState::Paused,
            "BUFFERING" => CastState::Buffering,
            "IDLE" => CastState::Idle,
            "STOPPED" => CastState::Stopped,
            _ => CastState::Idle,
        }
    }
}

impl fmt::Display for CastState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastState::Idle => write!(f, "Idle"),
            CastState::Connecting => write!(f, "Connecting..."),
            CastState::Buffering => write!(f, "Buffering..."),
            CastState::Playing => write!(f, "▶ Playing"),
            CastState::Paused => write!(f, "⏸ Paused"),
            CastState::Stopped => write!(f, "⏹ Stopped"),
            CastState::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

/// Chromecast playback status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub state: CastState,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub title: Option<String>,
}

impl PlaybackStatus {
    /// Parse status from catt status output
    /// Format:
    /// ```text
    /// State: PLAYING
    /// Duration: 10234.5
    /// Current time: 1234.5
    /// Volume: 80
    /// ```
    pub fn parse_catt_status(output: &str) -> Option<Self> {
        let mut state = CastState::Idle;
        let mut position = Duration::ZERO;
        let mut duration = Duration::ZERO;
        let mut volume = 1.0;
        let mut title = None;

        for line in output.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "state" => state = CastState::from_catt_state(value),
                    "duration" => {
                        if let Ok(secs) = value.parse::<f64>() {
                            duration = Duration::from_secs_f64(secs);
                        }
                    }
                    "current time" => {
                        if let Ok(secs) = value.parse::<f64>() {
                            position = Duration::from_secs_f64(secs);
                        }
                    }
                    "volume" => {
                        if let Ok(vol) = value.parse::<f32>() {
                            volume = vol / 100.0; // catt reports 0-100
                        }
                    }
                    "title" => title = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Some(Self {
            state,
            position,
            duration,
            volume,
            title,
        })
    }

    /// Format position as HH:MM:SS
    pub fn format_position(&self) -> String {
        format_duration(self.position)
    }

    /// Format duration as HH:MM:SS
    pub fn format_duration(&self) -> String {
        format_duration(self.duration)
    }

    /// Get progress as percentage (0.0-1.0)
    pub fn progress(&self) -> f32 {
        if self.duration.as_secs() == 0 {
            0.0
        } else {
            self.position.as_secs_f32() / self.duration.as_secs_f32()
        }
    }

    /// Format volume as percentage
    pub fn format_volume(&self) -> String {
        format!("{}%", (self.volume * 100.0) as u8)
    }
}

impl fmt::Display for PlaybackStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} / {} ({})",
            self.state,
            self.format_position(),
            self.format_duration(),
            self.format_volume()
        )
    }
}

// =============================================================================
// Subtitle Models
// =============================================================================

/// Subtitle file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubFormat {
    Srt,
    WebVtt,
    Sub,
    Ass,
}

impl SubFormat {
    /// Parse format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "srt" => SubFormat::Srt,
            "vtt" | "webvtt" => SubFormat::WebVtt,
            "sub" => SubFormat::Sub,
            "ass" | "ssa" => SubFormat::Ass,
            _ => SubFormat::Srt,
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            SubFormat::Srt => "srt",
            SubFormat::WebVtt => "vtt",
            SubFormat::Sub => "sub",
            SubFormat::Ass => "ass",
        }
    }
}

impl fmt::Display for SubFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubFormat::Srt => write!(f, "SRT"),
            SubFormat::WebVtt => write!(f, "WebVTT"),
            SubFormat::Sub => write!(f, "SUB"),
            SubFormat::Ass => write!(f, "ASS"),
        }
    }
}

/// Subtitle search result from Stremio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleResult {
    pub id: String,
    /// Direct download URL (from Stremio)
    pub url: String,
    pub language: String,
    pub language_name: String,
    pub release: String,
    pub fps: Option<f32>,
    pub format: SubFormat,
    pub downloads: u32,
    pub from_trusted: bool,
    pub hearing_impaired: bool,
    pub ai_translated: bool,
}

impl SubtitleResult {
    /// Trust score for sorting (higher = better)
    pub fn trust_score(&self) -> u32 {
        let mut score = self.downloads;
        if self.from_trusted {
            score += 10000;
        }
        if self.ai_translated {
            score = score.saturating_sub(5000);
        }
        score
    }
}

impl fmt::Display for SubtitleResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut indicators = Vec::new();
        if self.from_trusted {
            indicators.push("✓");
        }
        if self.hearing_impaired {
            indicators.push("👂");
        }
        if self.ai_translated {
            indicators.push("🤖");
        }

        let indicator_str = if indicators.is_empty() {
            String::new()
        } else {
            format!(" {}", indicators.join(" "))
        };

        write!(
            f,
            "[{}] {} - {}⬇{}",
            self.language, self.release, self.downloads, indicator_str
        )
    }
}

/// Downloaded subtitle file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleFile {
    pub id: String,
    pub language: String,
    pub path: PathBuf,
    pub format: SubFormat,
}

impl SubtitleFile {
    /// Convert SRT content to WebVTT format (required for Chromecast)
    pub fn srt_to_webvtt(srt: &str) -> String {
        let mut webvtt = String::from("WEBVTT\n\n");

        for line in srt.lines() {
            // Replace SRT timestamp format (00:01:23,456) with WebVTT format (00:01:23.456)
            let converted_line = line.replace(',', ".");
            webvtt.push_str(&converted_line);
            webvtt.push('\n');
        }

        webvtt
    }

    /// Generate URL for serving this subtitle file
    pub fn generate_url(lan_ip: IpAddr, port: u16, language: &str) -> String {
        format!("http://{}:{}/subtitles/{}.vtt", lan_ip, port, language)
    }
}

impl fmt::Display for SubtitleFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.path.file_name().unwrap_or_default().to_string_lossy(),
            self.format
        )
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Format a Duration as HH:MM:SS or MM:SS
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    // -------------------------------------------------------------------------
    // MediaType Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_media_type_display() {
        assert_eq!(MediaType::Movie.to_string(), "Movie");
        assert_eq!(MediaType::Tv.to_string(), "TV Show");
    }

    #[test]
    fn test_media_type_serde() {
        let json = serde_json::to_string(&MediaType::Movie).unwrap();
        assert_eq!(json, "\"movie\"");

        let parsed: MediaType = serde_json::from_str("\"tv\"").unwrap();
        assert_eq!(parsed, MediaType::Tv);
    }

    // -------------------------------------------------------------------------
    // SearchResult Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_search_result_display_with_year() {
        let result = SearchResult {
            id: 1,
            media_type: MediaType::Movie,
            title: "The Batman".to_string(),
            year: Some(2022),
            overview: "".to_string(),
            poster_path: None,
            vote_average: 7.8,
        };
        assert_eq!(result.to_string(), "The Batman (2022) [Movie]");
    }

    #[test]
    fn test_search_result_display_without_year() {
        let result = SearchResult {
            id: 1,
            media_type: MediaType::Tv,
            title: "Unknown Show".to_string(),
            year: None,
            overview: "".to_string(),
            poster_path: None,
            vote_average: 6.0,
        };
        assert_eq!(result.to_string(), "Unknown Show [TV Show]");
    }

    // -------------------------------------------------------------------------
    // Quality Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_quality_from_str_4k() {
        assert_eq!(Quality::from_str_loose("4K"), Quality::UHD4K);
        assert_eq!(Quality::from_str_loose("2160p"), Quality::UHD4K);
        assert_eq!(Quality::from_str_loose("UHD"), Quality::UHD4K);
        assert_eq!(Quality::from_str_loose("Torrentio\n4K"), Quality::UHD4K);
    }

    #[test]
    fn test_quality_from_str_1080p() {
        assert_eq!(Quality::from_str_loose("1080p"), Quality::FHD1080p);
        assert_eq!(Quality::from_str_loose("FHD"), Quality::FHD1080p);
        assert_eq!(
            Quality::from_str_loose("Torrentio\n1080p"),
            Quality::FHD1080p
        );
    }

    #[test]
    fn test_quality_from_str_720p() {
        assert_eq!(Quality::from_str_loose("720p"), Quality::HD720p);
    }

    #[test]
    fn test_quality_from_str_480p() {
        assert_eq!(Quality::from_str_loose("480p"), Quality::SD480p);
        assert_eq!(Quality::from_str_loose("SD"), Quality::SD480p);
    }

    #[test]
    fn test_quality_from_str_unknown() {
        assert_eq!(Quality::from_str_loose("CAM"), Quality::Unknown);
        assert_eq!(Quality::from_str_loose("HDCAM"), Quality::Unknown);
        assert_eq!(Quality::from_str_loose(""), Quality::Unknown);
    }

    #[test]
    fn test_quality_ordering() {
        assert!(Quality::UHD4K > Quality::FHD1080p);
        assert!(Quality::FHD1080p > Quality::HD720p);
        assert!(Quality::HD720p > Quality::SD480p);
        assert!(Quality::SD480p > Quality::Unknown);
    }

    #[test]
    fn test_quality_display() {
        assert_eq!(Quality::UHD4K.to_string(), "4K");
        assert_eq!(Quality::FHD1080p.to_string(), "1080p");
        assert_eq!(Quality::HD720p.to_string(), "720p");
        assert_eq!(Quality::SD480p.to_string(), "480p");
        assert_eq!(Quality::Unknown.to_string(), "???");
    }

    // -------------------------------------------------------------------------
    // StreamSource Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_seeds_emoji() {
        assert_eq!(StreamSource::parse_seeds("Some title 👤 142"), 142);
        assert_eq!(StreamSource::parse_seeds("Title 👤 89"), 89);
    }

    #[test]
    fn test_parse_seeds_with_k() {
        assert_eq!(StreamSource::parse_seeds("Title 👤 1.2k"), 1200);
        assert_eq!(StreamSource::parse_seeds("Title 👤 2k"), 2000);
    }

    #[test]
    fn test_parse_seeds_text_format() {
        assert_eq!(StreamSource::parse_seeds("Title seeds: 500"), 500);
        assert_eq!(StreamSource::parse_seeds("Title seed: 123"), 123);
    }

    #[test]
    fn test_parse_seeds_none() {
        assert_eq!(StreamSource::parse_seeds("Title without seeds"), 0);
    }

    #[test]
    fn test_parse_size_gb() {
        // 4.2 GB = 4.2 * 1024^3 = 4509715660.8 bytes (truncated to 4509715660)
        let size = StreamSource::parse_size("Some.Movie.4.2 GB.mkv");
        assert!(size.is_some());
        let bytes = size.unwrap();
        // Allow some floating point tolerance
        assert!(bytes > 4_500_000_000 && bytes < 4_520_000_000);
    }

    #[test]
    fn test_parse_size_mb() {
        // 890 MB = 890 * 1024^2 = 933232640 bytes
        let size = StreamSource::parse_size("File 890 MB");
        assert!(size.is_some());
        let bytes = size.unwrap();
        assert!(bytes > 930_000_000 && bytes < 940_000_000);
    }

    #[test]
    fn test_parse_size_none() {
        assert!(StreamSource::parse_size("No size here").is_none());
    }

    #[test]
    fn test_magnet_generation() {
        let source = StreamSource {
            name: "Test".to_string(),
            title: "Test Title".to_string(),
            info_hash: "abc123def456".to_string(),
            file_idx: Some(0),
            seeds: 100,
            quality: Quality::FHD1080p,
            size_bytes: None,
        };

        let magnet = source.to_magnet("Movie Name");
        assert_eq!(magnet, "magnet:?xt=urn:btih:abc123def456&dn=Movie%20Name");
    }

    #[test]
    fn test_magnet_url_encoding() {
        let source = StreamSource {
            name: "Test".to_string(),
            title: "Test".to_string(),
            info_hash: "abc123".to_string(),
            file_idx: None,
            seeds: 0,
            quality: Quality::Unknown,
            size_bytes: None,
        };

        let magnet = source.to_magnet("Test & Movie (2022)");
        assert!(magnet.contains("Test%20%26%20Movie%20%282022%29"));
    }

    // -------------------------------------------------------------------------
    // TorrentSession Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_torrent_session_new() {
        let session = TorrentSession::new("magnet:?xt=urn:btih:abc".to_string(), Some(0));
        assert_eq!(session.state, TorrentState::Starting);
        assert_eq!(session.progress, 0.0);
        assert!(session.stream_url.is_none());
    }

    #[test]
    fn test_torrent_state_display() {
        assert_eq!(TorrentState::Starting.to_string(), "Starting...");
        assert_eq!(
            TorrentState::FetchingMetadata { peers: 0 }.to_string(),
            "Fetching metadata (0 peers)"
        );
        assert_eq!(
            TorrentState::Connecting { peers: 3 }.to_string(),
            "Connecting (3 peers)"
        );
        assert_eq!(
            TorrentState::Buffering { peers: 5, progress: 42 }.to_string(),
            "Buffering 42% (5 peers)"
        );
        assert_eq!(TorrentState::Streaming.to_string(), "Streaming");
        assert_eq!(
            TorrentState::Error("No peers".to_string()).to_string(),
            "Error: No peers"
        );
    }

    #[test]
    fn test_torrent_state_peers() {
        assert_eq!(TorrentState::Starting.peers(), None);
        assert_eq!(TorrentState::FetchingMetadata { peers: 0 }.peers(), Some(0));
        assert_eq!(TorrentState::Connecting { peers: 5 }.peers(), Some(5));
        assert_eq!(TorrentState::Buffering { peers: 10, progress: 50 }.peers(), Some(10));
        assert_eq!(TorrentState::Streaming.peers(), None);
    }

    #[test]
    fn test_torrent_state_is_connecting() {
        assert!(TorrentState::Starting.is_connecting());
        assert!(TorrentState::FetchingMetadata { peers: 0 }.is_connecting());
        assert!(TorrentState::Connecting { peers: 3 }.is_connecting());
        assert!(TorrentState::Buffering { peers: 5, progress: 20 }.is_connecting());
        assert!(!TorrentState::Streaming.is_connecting());
        assert!(!TorrentState::Stopped.is_connecting());
    }

    #[test]
    fn test_stream_url_generation() {
        let url = TorrentSession::generate_stream_url(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            8888,
            0,
        );
        assert_eq!(url, "http://192.168.1.100:8888/0");
    }

    #[test]
    fn test_parse_progress() {
        let total = 4 * 1024 * 1024 * 1024; // 4 GB
        let progress = TorrentSession::parse_progress("Downloaded: 1.2 GB", total);
        assert!(progress > 0.29 && progress < 0.31); // ~30%
    }

    #[test]
    fn test_parse_speed() {
        let speed = TorrentSession::parse_speed("Speed: 5.2 MB/s");
        // 5.2 MB/s = 5.2 * 1024 * 1024 = ~5452595 bytes/sec
        assert!(speed > 5_400_000 && speed < 5_500_000);

        let speed_kb = TorrentSession::parse_speed("Speed: 512 KB/s");
        assert!(speed_kb > 520_000 && speed_kb < 530_000);
    }

    // -------------------------------------------------------------------------
    // CastDevice Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_catt_scan_output() {
        // catt 0.13+ format: "IP - Name - Model"
        let output = "Scanning Chromecasts...\n192.168.1.50 - Living Room TV - Google Inc. Chromecast Ultra\n192.168.1.51 - Bedroom - Google Inc. Chromecast\n";
        let devices = CastDevice::parse_catt_scan(output);

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "Living Room TV");
        assert_eq!(devices[0].address.to_string(), "192.168.1.50");
        assert_eq!(devices[0].model, Some("Google Inc. Chromecast Ultra".to_string()));
        assert_eq!(devices[1].name, "Bedroom");
        assert_eq!(devices[1].address.to_string(), "192.168.1.51");
        assert_eq!(devices[1].model, Some("Google Inc. Chromecast".to_string()));
    }

    #[test]
    fn test_parse_catt_scan_no_devices() {
        let output = "Scanning...\nNo devices found\n";
        let devices = CastDevice::parse_catt_scan(output);
        assert!(devices.is_empty());
    }

    #[test]
    fn test_cast_device_display() {
        let device = CastDevice {
            id: "192.168.1.50".to_string(),
            name: "Living Room TV".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: Some("Chromecast Ultra".to_string()),
        };
        assert_eq!(
            device.to_string(),
            "Living Room TV (Chromecast Ultra) - 192.168.1.50"
        );
    }

    // -------------------------------------------------------------------------
    // CastState Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cast_state_from_catt() {
        assert_eq!(CastState::from_catt_state("PLAYING"), CastState::Playing);
        assert_eq!(CastState::from_catt_state("PAUSED"), CastState::Paused);
        assert_eq!(
            CastState::from_catt_state("BUFFERING"),
            CastState::Buffering
        );
        assert_eq!(CastState::from_catt_state("IDLE"), CastState::Idle);
        assert_eq!(CastState::from_catt_state("playing"), CastState::Playing); // case insensitive
    }

    // -------------------------------------------------------------------------
    // PlaybackStatus Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_catt_status() {
        let output = "State: PLAYING\nDuration: 10234.5\nCurrent time: 1234.5\nVolume: 80";
        let status = PlaybackStatus::parse_catt_status(output).unwrap();

        assert_eq!(status.state, CastState::Playing);
        assert_eq!(status.duration.as_secs_f64(), 10234.5);
        assert_eq!(status.position.as_secs_f64(), 1234.5);
        assert!((status.volume - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_playback_progress() {
        let status = PlaybackStatus {
            state: CastState::Playing,
            position: Duration::from_secs(300),
            duration: Duration::from_secs(600),
            volume: 1.0,
            title: None,
        };
        assert!((status.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_format_duration_hhmmss() {
        let status = PlaybackStatus {
            state: CastState::Idle,
            position: Duration::from_secs(3661), // 1:01:01
            duration: Duration::from_secs(7322), // 2:02:02
            volume: 1.0,
            title: None,
        };
        assert_eq!(status.format_position(), "01:01:01");
        assert_eq!(status.format_duration(), "02:02:02");
    }

    #[test]
    fn test_format_duration_mmss() {
        let status = PlaybackStatus {
            state: CastState::Idle,
            position: Duration::from_secs(125), // 2:05
            duration: Duration::from_secs(300), // 5:00
            volume: 1.0,
            title: None,
        };
        assert_eq!(status.format_position(), "02:05");
        assert_eq!(status.format_duration(), "05:00");
    }

    // -------------------------------------------------------------------------
    // SubFormat Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sub_format_from_extension() {
        assert_eq!(SubFormat::from_extension("srt"), SubFormat::Srt);
        assert_eq!(SubFormat::from_extension("SRT"), SubFormat::Srt);
        assert_eq!(SubFormat::from_extension("vtt"), SubFormat::WebVtt);
        assert_eq!(SubFormat::from_extension("ass"), SubFormat::Ass);
        assert_eq!(SubFormat::from_extension("sub"), SubFormat::Sub);
        assert_eq!(SubFormat::from_extension("xyz"), SubFormat::Srt); // default
    }

    #[test]
    fn test_sub_format_extension() {
        assert_eq!(SubFormat::Srt.extension(), "srt");
        assert_eq!(SubFormat::WebVtt.extension(), "vtt");
        assert_eq!(SubFormat::Ass.extension(), "ass");
    }

    // -------------------------------------------------------------------------
    // SubtitleResult Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_subtitle_trust_score() {
        let trusted = SubtitleResult {
            id: "1".to_string(),
            url: "https://subs.strem.io/1".to_string(),
            language: "en".to_string(),
            language_name: "English".to_string(),
            release: "Test".to_string(),
            fps: None,
            format: SubFormat::Srt,
            downloads: 1000,
            from_trusted: true,
            hearing_impaired: false,
            ai_translated: false,
        };

        let untrusted = SubtitleResult {
            from_trusted: false,
            ..trusted.clone()
        };

        let ai = SubtitleResult {
            ai_translated: true,
            ..trusted.clone()
        };

        assert!(trusted.trust_score() > untrusted.trust_score());
        assert!(trusted.trust_score() > ai.trust_score());
    }

    // -------------------------------------------------------------------------
    // SRT to WebVTT Conversion Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_srt_to_webvtt_header() {
        let srt = "1\n00:00:01,000 --> 00:00:02,000\nHello";
        let vtt = SubtitleFile::srt_to_webvtt(srt);
        assert!(vtt.starts_with("WEBVTT\n"));
    }

    #[test]
    fn test_srt_to_webvtt_timestamp_conversion() {
        let srt = "1\n00:01:23,456 --> 00:01:25,789\nTest dialogue";
        let vtt = SubtitleFile::srt_to_webvtt(srt);
        assert!(vtt.contains("00:01:23.456"));
        assert!(vtt.contains("00:01:25.789"));
        assert!(!vtt.contains(','));
    }

    #[test]
    fn test_srt_to_webvtt_preserves_content() {
        let srt = "1\n00:00:01,000 --> 00:00:02,000\nHello, world!";
        let vtt = SubtitleFile::srt_to_webvtt(srt);
        assert!(vtt.contains("Hello. world!")); // comma becomes period
    }

    #[test]
    fn test_subtitle_url_generation() {
        let url =
            SubtitleFile::generate_url(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8889, "en");
        assert_eq!(url, "http://192.168.1.100:8889/subtitles/en.vtt");
    }

    // -------------------------------------------------------------------------
    // Episode Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_episode_display() {
        let episode = Episode {
            season: 1,
            episode: 5,
            name: "Pilot".to_string(),
            overview: "The first episode".to_string(),
            runtime: Some(45),
            imdb_id: Some("tt0000001".to_string()),
        };
        assert_eq!(episode.to_string(), "S01E05 - Pilot");
    }

    // -------------------------------------------------------------------------
    // MovieDetail Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_movie_detail_display() {
        let movie = MovieDetail {
            id: 1,
            imdb_id: "tt0000001".to_string(),
            title: "The Batman".to_string(),
            year: 2022,
            runtime: 176,
            genres: vec!["Action".to_string()],
            overview: "".to_string(),
            vote_average: 7.8,
            poster_path: None,
            backdrop_path: None,
        };
        assert_eq!(movie.to_string(), "The Batman (2022) - 2h 56m - ⭐ 7.8");
    }

    // -------------------------------------------------------------------------
    // TvDetail Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_tv_detail_display() {
        let tv = TvDetail {
            id: 1,
            imdb_id: "tt0000001".to_string(),
            name: "Breaking Bad".to_string(),
            year: 2008,
            seasons: vec![
                SeasonSummary {
                    season_number: 1,
                    episode_count: 7,
                    name: None,
                    air_date: None,
                },
                SeasonSummary {
                    season_number: 2,
                    episode_count: 13,
                    name: None,
                    air_date: None,
                },
            ],
            genres: vec!["Drama".to_string()],
            overview: "".to_string(),
            vote_average: 9.5,
            poster_path: None,
            backdrop_path: None,
        };
        assert_eq!(tv.to_string(), "Breaking Bad (2008) - 2 seasons - ⭐ 9.5");
    }
}
