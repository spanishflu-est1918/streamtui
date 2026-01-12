//! CLI - Command Line Interface for StreamTUI
//!
//! Designed for automation and Claude Code integration.
//! Every TUI action is scriptable. All output is JSON-parseable.
//!
//! # Examples
//!
//! ```bash
//! # Search for content
//! streamtui search "the batman" --json
//!
//! # Get streams and cast
//! streamtui streams tt1877830
//! streamtui cast tt1877830 --device "Living Room TV" --quality 1080p
//!
//! # Playback control
//! streamtui status
//! streamtui pause
//! streamtui seek 3600
//! ```

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::path::PathBuf;

// =============================================================================
// Exit Codes
// =============================================================================

/// Exit codes for CLI operations (semantic for scripting)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Success
    Success = 0,
    /// General error
    Error = 1,
    /// Invalid arguments
    InvalidArgs = 2,
    /// Network error
    NetworkError = 3,
    /// Device not found
    DeviceNotFound = 4,
    /// No streams available
    NoStreams = 5,
    /// Cast failed
    CastFailed = 6,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> std::process::ExitCode {
        std::process::ExitCode::from(code as u8)
    }
}

// =============================================================================
// Main CLI Structure
// =============================================================================

/// StreamTUI - Cyberpunk TUI for streaming to Chromecast
///
/// Run without arguments to launch interactive TUI.
/// Use subcommands for scriptable automation.
#[derive(Parser, Debug)]
#[command(
    name = "streamtui",
    version,
    author = "Gorka & Hermes",
    about = "Cyberpunk TUI for streaming to Chromecast",
    long_about = "A neon-soaked terminal interface for searching content, \
                  selecting quality, and casting to your TV.\n\n\
                  Run without arguments to launch the interactive TUI.\n\
                  Use subcommands for automation and scripting.",
    after_help = "EXAMPLES:\n\
                  streamtui                           Launch interactive TUI\n\
                  streamtui search \"blade runner\"     Search for content\n\
                  streamtui cast tt1877830 -d TV      Cast to device\n\
                  streamtui status --json             Check playback status"
)]
pub struct Cli {
    /// Output format as JSON (default for non-TTY)
    #[arg(long, short = 'j', global = true)]
    pub json: bool,

    /// Target Chromecast device name
    #[arg(long, short = 'd', global = true)]
    pub device: Option<String>,

    /// Suppress non-essential output
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Path to config file
    #[arg(long, short = 'c', global = true)]
    pub config: Option<PathBuf>,

    /// Subcommand to run (omit for TUI mode)
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    /// Check if running in CLI mode (has subcommand)
    pub fn is_cli_mode(&self) -> bool {
        self.command.is_some()
    }

    /// Check if JSON output should be used
    pub fn should_json(&self) -> bool {
        self.json || !std::io::stdout().is_terminal()
    }
}

// =============================================================================
// Subcommands
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search for movies and TV shows
    #[command(visible_alias = "s")]
    Search(SearchCmd),

    /// Get trending content
    #[command(visible_alias = "tr")]
    Trending(TrendingCmd),

    /// Get details for a movie or show
    #[command(visible_alias = "i")]
    Info(InfoCmd),

    /// Get available streams for content
    #[command(visible_alias = "st")]
    Streams(StreamsCmd),

    /// Search for subtitles
    #[command(visible_alias = "sub")]
    Subtitles(SubtitlesCmd),

    /// List available Chromecast devices
    #[command(visible_alias = "dev")]
    Devices(DevicesCmd),

    /// Start casting content to a device
    Cast(CastCmd),

    /// Get current playback status
    Status(StatusCmd),

    /// Resume playback
    Play(PlayCmd),

    /// Pause playback
    Pause(PauseCmd),

    /// Stop playback and disconnect
    Stop(StopCmd),

    /// Seek to a position
    Seek(SeekCmd),

    /// Set volume level
    #[command(visible_alias = "vol")]
    Volume(VolumeCmd),

    /// Cast a raw magnet link directly
    #[command(visible_alias = "cm")]
    CastMagnet(CastMagnetCmd),

    /// Play locally in VLC or mpv (no Chromecast)
    #[command(visible_alias = "pl")]
    PlayLocal(PlayLocalCmd),
}

// =============================================================================
// Search Command
// =============================================================================

/// Search for movies and TV shows by query
#[derive(Args, Debug)]
pub struct SearchCmd {
    /// Search query (title, keywords)
    #[arg(required = true)]
    pub query: String,

    /// Maximum number of results
    #[arg(long, short = 'l', default_value = "20")]
    pub limit: usize,

    /// Filter by media type
    #[arg(long, short = 't', value_enum)]
    pub media_type: Option<MediaTypeFilter>,

    /// Minimum year
    #[arg(long)]
    pub year_from: Option<u16>,

    /// Maximum year
    #[arg(long)]
    pub year_to: Option<u16>,
}

/// Media type filter for search
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaTypeFilter {
    /// Movies only
    Movie,
    /// TV shows only
    Tv,
}

// =============================================================================
// Trending Command
// =============================================================================

/// Get trending movies and TV shows
#[derive(Args, Debug)]
pub struct TrendingCmd {
    /// Time window for trending
    #[arg(long, short = 'w', value_enum, default_value = "day")]
    pub window: TrendingWindow,

    /// Maximum number of results
    #[arg(long, short = 'l', default_value = "20")]
    pub limit: usize,

    /// Filter by media type
    #[arg(long, short = 't', value_enum)]
    pub media_type: Option<MediaTypeFilter>,
}

/// Time window for trending content
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrendingWindow {
    /// Today's trending
    #[default]
    Day,
    /// This week's trending
    Week,
}

// =============================================================================
// Info Command
// =============================================================================

/// Get detailed information about a movie or TV show
#[derive(Args, Debug)]
pub struct InfoCmd {
    /// IMDB ID (e.g., tt1877830) or TMDB ID
    #[arg(required = true)]
    pub id: String,

    /// ID type if using TMDB ID
    #[arg(long, short = 't', value_enum)]
    pub media_type: Option<MediaTypeFilter>,
}

// =============================================================================
// Streams Command
// =============================================================================

/// Get available streams for a movie or TV episode
#[derive(Args, Debug)]
pub struct StreamsCmd {
    /// IMDB ID (e.g., tt1877830)
    #[arg(required = true)]
    pub imdb_id: String,

    /// Season number (for TV shows)
    #[arg(long, short = 's')]
    pub season: Option<u8>,

    /// Episode number (for TV shows)
    #[arg(long, short = 'e')]
    pub episode: Option<u16>,

    /// Filter by minimum quality
    #[arg(long, short = 'Q', value_enum)]
    pub quality: Option<QualityFilter>,

    /// Maximum number of results
    #[arg(long, short = 'l', default_value = "20")]
    pub limit: usize,

    /// Sort by criterion
    #[arg(long, value_enum, default_value = "seeds")]
    pub sort: StreamSort,
}

/// Quality filter for streams
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityFilter {
    /// 4K / 2160p
    #[value(name = "4k", alias = "2160p")]
    Q4k,
    /// 1080p Full HD
    #[value(name = "1080p")]
    Q1080p,
    /// 720p HD
    #[value(name = "720p")]
    Q720p,
    /// 480p SD
    #[value(name = "480p")]
    Q480p,
}

impl std::fmt::Display for QualityFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityFilter::Q4k => write!(f, "4K"),
            QualityFilter::Q1080p => write!(f, "1080p"),
            QualityFilter::Q720p => write!(f, "720p"),
            QualityFilter::Q480p => write!(f, "480p"),
        }
    }
}

/// Sort criterion for streams
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StreamSort {
    /// Sort by seed count (default)
    #[default]
    Seeds,
    /// Sort by quality
    Quality,
    /// Sort by file size
    Size,
}

/// Local player selection
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerChoice {
    /// VLC media player (default)
    #[default]
    Vlc,
    /// mpv media player
    Mpv,
}

// =============================================================================
// Subtitles Command
// =============================================================================

/// Search for subtitles
#[derive(Args, Debug)]
pub struct SubtitlesCmd {
    /// IMDB ID (e.g., tt1877830)
    #[arg(required = true)]
    pub imdb_id: String,

    /// Language codes, comma-separated (default: en)
    #[arg(long, short = 'l', default_value = "en")]
    pub lang: String,

    /// Season number (for TV shows)
    #[arg(long, short = 's')]
    pub season: Option<u8>,

    /// Episode number (for TV shows)
    #[arg(long, short = 'e')]
    pub episode: Option<u16>,

    /// Only show hearing-impaired subtitles
    #[arg(long)]
    pub hearing_impaired: bool,

    /// Only show trusted/verified subtitles
    #[arg(long)]
    pub trusted: bool,

    /// Maximum number of results
    #[arg(long, default_value = "20")]
    pub limit: usize,
}

impl SubtitlesCmd {
    /// Parse language codes into a vector
    pub fn languages(&self) -> Vec<&str> {
        self.lang.split(',').map(|s| s.trim()).collect()
    }
}

// =============================================================================
// Devices Command
// =============================================================================

/// List available Chromecast devices on the network
#[derive(Args, Debug)]
pub struct DevicesCmd {
    /// Scan timeout in seconds
    #[arg(long, short = 't', default_value = "5")]
    pub timeout: u64,

    /// Refresh device cache
    #[arg(long, short = 'r')]
    pub refresh: bool,
}

// =============================================================================
// Cast Command
// =============================================================================

/// Start casting content to a Chromecast device
#[derive(Args, Debug)]
pub struct CastCmd {
    /// IMDB ID (e.g., tt1877830)
    #[arg(required = true)]
    pub imdb_id: String,

    /// Target device name (overrides --device global flag)
    #[arg(long, short = 'd')]
    pub device: Option<String>,

    /// Preferred quality
    #[arg(long, short = 'Q', value_enum)]
    pub quality: Option<QualityFilter>,

    /// Season number (for TV shows)
    #[arg(long, short = 's')]
    pub season: Option<u8>,

    /// Episode number (for TV shows)
    #[arg(long, short = 'e')]
    pub episode: Option<u16>,

    /// Stream index from `streams` output
    #[arg(long, short = 'i')]
    pub index: Option<usize>,

    /// Subtitle language code (e.g., "en")
    #[arg(long)]
    pub subtitle: Option<String>,

    /// Specific subtitle ID from `subtitles` output
    #[arg(long)]
    pub subtitle_id: Option<String>,

    /// Explicitly disable subtitles
    #[arg(long)]
    pub no_subtitle: bool,

    /// Start position in seconds
    #[arg(long)]
    pub start: Option<u64>,

    /// Play locally in VLC instead of casting
    #[arg(long)]
    pub vlc: bool,
}

impl CastCmd {
    /// Get effective device name (command-specific or global)
    pub fn effective_device<'a>(&'a self, global: &'a Option<String>) -> Option<&'a str> {
        self.device.as_deref().or(global.as_deref())
    }
}

// =============================================================================
// Cast Magnet Command
// =============================================================================

/// Cast a raw magnet link directly to a Chromecast device
#[derive(Args, Debug)]
pub struct CastMagnetCmd {
    /// Magnet link URL
    #[arg(required = true)]
    pub magnet: String,

    /// Target device name (overrides --device global flag)
    #[arg(long, short = 'd')]
    pub device: Option<String>,

    /// Subtitle language code (e.g., "en") - searches OpenSubtitles
    #[arg(long)]
    pub subtitle: Option<String>,

    /// Path to a local subtitle file (.srt, .vtt)
    #[arg(long)]
    pub subtitle_file: Option<PathBuf>,

    /// File index within the torrent (default: largest video file)
    #[arg(long, short = 'i')]
    pub file_idx: Option<u32>,

    /// Start position in seconds
    #[arg(long)]
    pub start: Option<u64>,

    /// Play locally in VLC instead of casting
    #[arg(long)]
    pub vlc: bool,
}

impl CastMagnetCmd {
    /// Get effective device name (command-specific or global)
    pub fn effective_device<'a>(&'a self, global: &'a Option<String>) -> Option<&'a str> {
        self.device.as_deref().or(global.as_deref())
    }
}

// =============================================================================
// Play Local Command
// =============================================================================

/// Play a magnet link locally in VLC or mpv
#[derive(Args, Debug)]
pub struct PlayLocalCmd {
    /// Magnet link URL
    #[arg(required = true)]
    pub magnet: String,

    /// Player to use (vlc or mpv)
    #[arg(long, short = 'p', value_enum, default_value = "vlc")]
    pub player: PlayerChoice,

    /// Path to a local subtitle file (.srt, .vtt)
    #[arg(long)]
    pub subtitle_file: Option<PathBuf>,

    /// File index within the torrent (default: largest video file)
    #[arg(long, short = 'i')]
    pub file_idx: Option<u32>,
}

// =============================================================================
// Playback Control Commands
// =============================================================================

/// Get current playback status
#[derive(Args, Debug)]
pub struct StatusCmd {
    /// Watch mode: continuously update status
    #[arg(long, short = 'w')]
    pub watch: bool,

    /// Update interval in seconds (for watch mode)
    #[arg(long, short = 'i', default_value = "1")]
    pub interval: u64,
}

/// Resume playback
#[derive(Args, Debug)]
pub struct PlayCmd {}

/// Pause playback
#[derive(Args, Debug)]
pub struct PauseCmd {}

/// Stop playback and disconnect
#[derive(Args, Debug)]
pub struct StopCmd {
    /// Also stop the torrent stream
    #[arg(long)]
    pub kill_stream: bool,
}

/// Seek to a position in playback
#[derive(Args, Debug)]
pub struct SeekCmd {
    /// Target position in seconds, or relative (+/-) seconds
    #[arg(required = true)]
    pub position: String,
}

impl SeekCmd {
    /// Parse the position argument
    pub fn parse_position(&self) -> SeekPosition {
        let s = self.position.trim();
        if let Some(stripped) = s.strip_prefix('+') {
            if let Ok(secs) = stripped.parse::<i64>() {
                return SeekPosition::Forward(secs);
            }
        } else if let Some(stripped) = s.strip_prefix('-') {
            if let Ok(secs) = stripped.parse::<i64>() {
                return SeekPosition::Backward(secs);
            }
        } else if let Ok(secs) = s.parse::<u64>() {
            return SeekPosition::Absolute(secs);
        }
        // Try parsing as timestamp (HH:MM:SS or MM:SS)
        if let Some(secs) = parse_timestamp(s) {
            return SeekPosition::Absolute(secs);
        }
        SeekPosition::Invalid(self.position.clone())
    }
}

/// Parsed seek position
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeekPosition {
    /// Absolute position in seconds
    Absolute(u64),
    /// Seek forward by seconds
    Forward(i64),
    /// Seek backward by seconds
    Backward(i64),
    /// Invalid position string
    Invalid(String),
}

/// Parse timestamp string (HH:MM:SS or MM:SS) to seconds
fn parse_timestamp(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let mins: u64 = parts[0].parse().ok()?;
            let secs: u64 = parts[1].parse().ok()?;
            Some(mins * 60 + secs)
        }
        3 => {
            let hours: u64 = parts[0].parse().ok()?;
            let mins: u64 = parts[1].parse().ok()?;
            let secs: u64 = parts[2].parse().ok()?;
            Some(hours * 3600 + mins * 60 + secs)
        }
        _ => None,
    }
}

/// Set volume level
#[derive(Args, Debug)]
pub struct VolumeCmd {
    /// Volume level (0-100) or relative (+/- N)
    #[arg(required = true)]
    pub level: String,
}

impl VolumeCmd {
    /// Parse the volume argument
    pub fn parse_level(&self) -> VolumeLevel {
        let s = self.level.trim();
        if let Some(stripped) = s.strip_prefix('+') {
            if let Ok(delta) = stripped.parse::<i8>() {
                return VolumeLevel::Relative(delta);
            }
        } else if let Some(stripped) = s.strip_prefix('-') {
            if let Ok(delta) = stripped.parse::<i8>() {
                return VolumeLevel::Relative(-delta);
            }
        } else if let Ok(vol) = s.parse::<u8>() {
            return VolumeLevel::Absolute(vol.min(100));
        }
        VolumeLevel::Invalid(self.level.clone())
    }
}

/// Parsed volume level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeLevel {
    /// Absolute volume (0-100)
    Absolute(u8),
    /// Relative volume change
    Relative(i8),
    /// Invalid level string
    Invalid(String),
}

// =============================================================================
// JSON Output Types
// =============================================================================

/// Generic JSON output wrapper with status
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "is_zero")]
    pub exit_code: i32,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

impl<T: Serialize> JsonOutput<T> {
    /// Create success output with data
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            exit_code: 0,
        }
    }

    /// Create error output (no data)
    pub fn error_msg(msg: impl Into<String>, code: ExitCode) -> JsonOutput<()> {
        JsonOutput::<()> {
            data: None,
            error: Some(msg.into()),
            exit_code: code.into(),
        }
    }
}

/// Status OK response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusOk {
    pub status: &'static str,
}

impl Default for StatusOk {
    fn default() -> Self {
        Self { status: "ok" }
    }
}

/// Playback status response
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub state: PlaybackState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
}

/// Playback state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Error,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self {
            state: PlaybackState::Idle,
            title: None,
            device: None,
            position: None,
            duration: None,
            progress: None,
            volume: None,
        }
    }
}

/// Cast success response
#[derive(Debug, Serialize, Deserialize)]
pub struct CastResponse {
    pub status: &'static str,
    pub device: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
}

// =============================================================================
// Output Helpers
// =============================================================================

/// Output handler for consistent formatting
pub struct Output {
    pub json: bool,
    pub quiet: bool,
}

impl Output {
    pub fn new(cli: &Cli) -> Self {
        Self {
            json: cli.should_json(),
            quiet: cli.quiet,
        }
    }

    /// Print success data
    pub fn print<T: Serialize>(&self, data: T) -> anyhow::Result<()> {
        if self.json {
            let output = JsonOutput::success(data);
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            // For non-JSON, caller should handle formatting
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Ok(())
    }

    /// Print raw JSON (already formatted)
    pub fn print_json<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        println!("{}", serde_json::to_string_pretty(data)?);
        Ok(())
    }

    /// Print error and return exit code
    pub fn error(&self, msg: impl Into<String>, code: ExitCode) -> ExitCode {
        let msg = msg.into();
        if self.json {
            let output = JsonOutput::<()>::error_msg(&msg, code);
            if let Ok(json) = serde_json::to_string_pretty(&output) {
                eprintln!("{}", json);
            }
        } else if !self.quiet {
            eprintln!("Error: {}", msg);
        }
        code
    }

    /// Print info message (suppressed in quiet mode)
    pub fn info(&self, msg: impl std::fmt::Display) {
        if !self.quiet && !self.json {
            eprintln!("{}", msg);
        }
    }
}

// =============================================================================
// IMDB ID Validation
// =============================================================================

/// Validate IMDB ID format (tt followed by digits)
pub fn validate_imdb_id(id: &str) -> Result<&str, &'static str> {
    if id.starts_with("tt") && id.len() >= 9 && id[2..].chars().all(|c| c.is_ascii_digit()) {
        Ok(id)
    } else {
        Err("Invalid IMDB ID format (expected tt followed by 7+ digits)")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_no_args_is_tui_mode() {
        let cli = Cli::parse_from::<_, &str>([]);
        assert!(!cli.is_cli_mode());
    }

    #[test]
    fn test_search_command() {
        let cli = Cli::parse_from(["streamtui", "search", "batman"]);
        assert!(cli.is_cli_mode());
        if let Some(Command::Search(cmd)) = cli.command {
            assert_eq!(cmd.query, "batman");
        } else {
            panic!("Expected Search command");
        }
    }

    #[test]
    fn test_global_flags() {
        let cli = Cli::parse_from([
            "streamtui",
            "--json",
            "--device",
            "Living Room TV",
            "--quiet",
            "search",
            "test",
        ]);
        assert!(cli.json);
        assert!(cli.quiet);
        assert_eq!(cli.device.as_deref(), Some("Living Room TV"));
    }

    #[test]
    fn test_cast_with_options() {
        let cli = Cli::parse_from([
            "streamtui",
            "cast",
            "tt1877830",
            "-d",
            "TV",
            "-Q",
            "1080p",
            "-s",
            "1",
            "-e",
            "3",
        ]);
        if let Some(Command::Cast(cmd)) = cli.command {
            assert_eq!(cmd.imdb_id, "tt1877830");
            assert_eq!(cmd.device.as_deref(), Some("TV"));
            assert_eq!(cmd.quality, Some(QualityFilter::Q1080p));
            assert_eq!(cmd.season, Some(1));
            assert_eq!(cmd.episode, Some(3));
        } else {
            panic!("Expected Cast command");
        }
    }

    #[test]
    fn test_seek_position_parsing() {
        let cmd = SeekCmd {
            position: "3600".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(3600));

        let cmd = SeekCmd {
            position: "+30".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Forward(30));

        let cmd = SeekCmd {
            position: "-10".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Backward(10));

        let cmd = SeekCmd {
            position: "1:30:00".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(5400));

        let cmd = SeekCmd {
            position: "5:30".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(330));
    }

    #[test]
    fn test_volume_parsing() {
        let cmd = VolumeCmd {
            level: "50".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Absolute(50));

        let cmd = VolumeCmd {
            level: "+10".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Relative(10));

        let cmd = VolumeCmd {
            level: "-5".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Relative(-5));

        // Cap at 100
        let cmd = VolumeCmd {
            level: "150".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Absolute(100));
    }

    #[test]
    fn test_validate_imdb_id() {
        assert!(validate_imdb_id("tt1877830").is_ok());
        assert!(validate_imdb_id("tt0903747").is_ok());
        assert!(validate_imdb_id("tt12345678").is_ok());
        assert!(validate_imdb_id("tt123456").is_err()); // too short
        assert!(validate_imdb_id("nm1234567").is_err()); // wrong prefix
        assert!(validate_imdb_id("1234567").is_err()); // no prefix
    }

    #[test]
    fn test_subtitles_languages() {
        let cmd = SubtitlesCmd {
            imdb_id: "tt1877830".to_string(),
            lang: "en,es,fr".to_string(),
            season: None,
            episode: None,
            hearing_impaired: false,
            trusted: false,
            limit: 20,
        };
        assert_eq!(cmd.languages(), vec!["en", "es", "fr"]);
    }

    #[test]
    fn test_streams_command() {
        let cli = Cli::parse_from([
            "streamtui",
            "streams",
            "tt0903747",
            "-s",
            "1",
            "-e",
            "1",
            "-Q",
            "1080p",
        ]);
        if let Some(Command::Streams(cmd)) = cli.command {
            assert_eq!(cmd.imdb_id, "tt0903747");
            assert_eq!(cmd.season, Some(1));
            assert_eq!(cmd.episode, Some(1));
            assert_eq!(cmd.quality, Some(QualityFilter::Q1080p));
        } else {
            panic!("Expected Streams command");
        }
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(i32::from(ExitCode::Success), 0);
        assert_eq!(i32::from(ExitCode::Error), 1);
        assert_eq!(i32::from(ExitCode::InvalidArgs), 2);
        assert_eq!(i32::from(ExitCode::NetworkError), 3);
        assert_eq!(i32::from(ExitCode::DeviceNotFound), 4);
        assert_eq!(i32::from(ExitCode::NoStreams), 5);
        assert_eq!(i32::from(ExitCode::CastFailed), 6);
    }
}
