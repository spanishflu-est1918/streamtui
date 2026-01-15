//! Local Player - VLC/mpv playback support
//!
//! Opens streams directly in VLC or mpv instead of casting to Chromecast.

use std::path::Path;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::{Child, Command};

/// Supported local players
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerType {
    /// VLC media player (default)
    #[default]
    Vlc,
    /// mpv media player
    Mpv,
}

impl PlayerType {
    /// Get the command name for this player
    pub fn command(&self) -> &'static str {
        match self {
            PlayerType::Vlc => {
                // On macOS, VLC is an app bundle - check for it
                #[cfg(target_os = "macos")]
                if std::path::Path::new("/Applications/VLC.app").exists() {
                    return "/Applications/VLC.app/Contents/MacOS/VLC";
                }
                "vlc"
            }
            PlayerType::Mpv => "mpv",
        }
    }

    /// Get a display name for this player
    pub fn display_name(&self) -> &'static str {
        match self {
            PlayerType::Vlc => "VLC",
            PlayerType::Mpv => "mpv",
        }
    }
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Errors from local player operations
#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("Player '{0}' not found. Install it first.")]
    NotFound(String),
    #[error("Failed to start player: {0}")]
    StartFailed(#[from] std::io::Error),
    #[error("Subtitle file not found: {0}")]
    SubtitleNotFound(String),
}

/// Local player for streaming content
pub struct LocalPlayer {
    player_type: PlayerType,
}

impl LocalPlayer {
    /// Create a new local player with the specified type
    pub fn new(player_type: PlayerType) -> Self {
        Self { player_type }
    }

    /// Create a VLC player
    pub fn vlc() -> Self {
        Self::new(PlayerType::Vlc)
    }

    /// Create an mpv player
    pub fn mpv() -> Self {
        Self::new(PlayerType::Mpv)
    }

    /// Get the player type
    pub fn player_type(&self) -> PlayerType {
        self.player_type
    }

    /// Check if the player is available on the system
    pub async fn is_available(&self) -> bool {
        let cmd = self.player_type.command();

        // If it's a full path (macOS app bundle), check if it exists
        if cmd.starts_with('/') {
            return std::path::Path::new(cmd).exists();
        }

        // Otherwise use 'which' to find in PATH
        Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Play a stream URL with optional subtitles
    ///
    /// # Arguments
    /// * `stream_url` - The URL to stream (http, file, etc.)
    /// * `subtitle_path` - Optional path to a subtitle file
    ///
    /// # Returns
    /// The spawned child process
    pub async fn play(
        &self,
        stream_url: &str,
        subtitle_path: Option<&Path>,
    ) -> Result<Child, PlayerError> {
        // Validate subtitle path if provided
        if let Some(sub_path) = subtitle_path {
            if !sub_path.exists() {
                return Err(PlayerError::SubtitleNotFound(
                    sub_path.display().to_string(),
                ));
            }
        }

        let mut cmd = Command::new(self.player_type.command());

        match self.player_type {
            PlayerType::Vlc => {
                cmd.arg(stream_url);
                if let Some(sub_path) = subtitle_path {
                    cmd.arg("--sub-file").arg(sub_path);
                }
                // VLC-specific options for better streaming
                cmd.arg("--no-video-title-show"); // Don't show filename overlay
            }
            PlayerType::Mpv => {
                cmd.arg(stream_url);
                if let Some(sub_path) = subtitle_path {
                    cmd.arg(format!("--sub-file={}", sub_path.display()));
                }
                // mpv-specific options
                cmd.arg("--force-window=immediate"); // Show window immediately
            }
        }

        // Don't capture output - let it display normally
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PlayerError::NotFound(self.player_type.command().to_string())
            } else {
                PlayerError::StartFailed(e)
            }
        })
    }

    /// Play a stream and wait for the player to close
    pub async fn play_and_wait(
        &self,
        stream_url: &str,
        subtitle_path: Option<&Path>,
    ) -> Result<(), PlayerError> {
        let mut child = self.play(stream_url, subtitle_path).await?;
        let _ = child.wait().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_type_command() {
        // On macOS with VLC installed, returns full path; otherwise "vlc"
        let vlc_cmd = PlayerType::Vlc.command();
        assert!(vlc_cmd == "vlc" || vlc_cmd == "/Applications/VLC.app/Contents/MacOS/VLC");
        assert_eq!(PlayerType::Mpv.command(), "mpv");
    }

    #[test]
    fn test_player_type_display() {
        assert_eq!(PlayerType::Vlc.to_string(), "VLC");
        assert_eq!(PlayerType::Mpv.to_string(), "mpv");
    }

    #[test]
    fn test_default_player() {
        assert_eq!(PlayerType::default(), PlayerType::Vlc);
    }
}
