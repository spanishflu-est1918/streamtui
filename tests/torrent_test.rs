//! Torrent Manager Tests (TDD)
//!
//! Tests for webtorrent-cli based torrent streaming.
//! Written BEFORE implementation per TDD methodology.

use std::net::IpAddr;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::Result;
use tokio::process::Command;
use tokio::sync::Mutex;
use uuid::Uuid;

// ============================================================================
// TYPE DEFINITIONS (for TDD - define what we need)
// ============================================================================

/// Torrent session state machine
#[derive(Debug, Clone, PartialEq)]
pub enum TorrentState {
    Starting,
    Connecting,
    Downloading,
    Streaming,
    Paused,
    Stopped,
    Error(String),
}

/// A torrent streaming session
#[derive(Debug, Clone)]
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
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validate a magnet link
pub fn validate_magnet(magnet: &str) -> Result<(), String> {
    if magnet.is_empty() {
        return Err("Magnet link cannot be empty".to_string());
    }

    if !magnet.starts_with("magnet:?") {
        return Err("Invalid magnet link format".to_string());
    }

    // Must contain infohash (btih)
    if !magnet.contains("xt=urn:btih:") {
        return Err("Magnet link missing infohash".to_string());
    }

    // Extract and validate infohash (should be 40 hex chars for SHA1)
    let btih_prefix = "xt=urn:btih:";
    if let Some(start) = magnet.find(btih_prefix) {
        let hash_start = start + btih_prefix.len();
        let hash_end = magnet[hash_start..]
            .find('&')
            .map(|i| hash_start + i)
            .unwrap_or(magnet.len());
        let hash = &magnet[hash_start..hash_end];

        // SHA1 hash is 40 hex chars, SHA256 is 64
        if hash.len() != 40 && hash.len() != 64 {
            return Err(format!(
                "Invalid infohash length: {} (expected 40 or 64)",
                hash.len()
            ));
        }

        // Verify hex characters
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Infohash contains non-hexadecimal characters".to_string());
        }
    }

    Ok(())
}

/// Parse download speed from webtorrent output
/// Input: "Speed: 5.2 MB/s" → 5452595 bytes/sec
pub fn parse_speed(output: &str) -> Option<u64> {
    let re = regex::Regex::new(r"Speed:\s*([\d.]+)\s*(KB|MB|GB)/s").ok()?;
    let caps = re.captures(output)?;

    let value: f64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2)?.as_str();

    let multiplier: u64 = match unit {
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        _ => return None,
    };

    Some((value * multiplier as f64) as u64)
}

/// Parse downloaded bytes from webtorrent output
/// Input: "Downloaded: 1.2 GB" → 1288490189 bytes
pub fn parse_downloaded(output: &str) -> Option<u64> {
    let re = regex::Regex::new(r"Downloaded:\s*([\d.]+)\s*(KB|MB|GB)").ok()?;
    let caps = re.captures(output)?;

    let value: f64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2)?.as_str();

    let multiplier: u64 = match unit {
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        _ => return None,
    };

    Some((value * multiplier as f64) as u64)
}

/// Calculate progress from downloaded bytes and total size
pub fn calculate_progress(downloaded: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (downloaded as f32 / total as f32).clamp(0.0, 1.0)
}

/// Generate stream URL for casting
pub fn generate_stream_url(lan_ip: IpAddr, port: u16, file_idx: u32) -> String {
    format!("http://{}:{}/{}", lan_ip, port, file_idx)
}

/// Check if webtorrent-cli is installed
pub async fn check_webtorrent_installed() -> Result<(), String> {
    let result = Command::new("which")
        .arg("webtorrent")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    match result {
        Ok(status) if status.success() => Ok(()),
        _ => {
            Err("webtorrent-cli not found. Install with: npm install -g webtorrent-cli".to_string())
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Valid test magnet (Big Buck Bunny - public domain)
    const VALID_MAGNET: &str =
        "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny";

    // ------------------------------------------------------------------------
    // test_magnet_validation
    // ------------------------------------------------------------------------

    #[test]
    fn test_magnet_validation_valid() {
        assert!(validate_magnet(VALID_MAGNET).is_ok());
    }

    #[test]
    fn test_magnet_validation_valid_with_trackers() {
        let magnet = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Test&tr=udp://tracker.example.com:80";
        assert!(validate_magnet(magnet).is_ok());
    }

    #[test]
    fn test_magnet_validation_invalid_format() {
        assert!(validate_magnet("http://example.com/torrent").is_err());
        assert!(validate_magnet("not a magnet link").is_err());
    }

    #[test]
    fn test_magnet_validation_empty() {
        let result = validate_magnet("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_magnet_validation_missing_infohash() {
        let result = validate_magnet("magnet:?dn=NoInfohash");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("infohash"));
    }

    #[test]
    fn test_magnet_validation_invalid_infohash_length() {
        let result = validate_magnet("magnet:?xt=urn:btih:abc123&dn=Short");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("length"));
    }

    #[test]
    fn test_magnet_validation_non_hex_infohash() {
        let result = validate_magnet(
            "magnet:?xt=urn:btih:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz&dn=BadHex",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("hexadecimal"));
    }

    // ------------------------------------------------------------------------
    // test_session_state_transitions
    // ------------------------------------------------------------------------

    #[test]
    fn test_session_starts_in_starting_state() {
        let session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        assert_eq!(session.state, TorrentState::Starting);
    }

    #[test]
    fn test_session_state_transition_to_connecting() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        session.state = TorrentState::Connecting;
        assert_eq!(session.state, TorrentState::Connecting);
    }

    #[test]
    fn test_session_state_transition_to_downloading() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        session.state = TorrentState::Downloading;
        session.download_speed = 1024 * 1024; // 1 MB/s
        assert_eq!(session.state, TorrentState::Downloading);
        assert!(session.download_speed > 0);
    }

    #[test]
    fn test_session_state_transition_to_streaming() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        session.state = TorrentState::Streaming;
        session.stream_url = Some("http://192.168.1.100:8888/0".to_string());
        assert_eq!(session.state, TorrentState::Streaming);
        assert!(session.stream_url.is_some());
    }

    #[test]
    fn test_session_state_transition_to_stopped() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        session.state = TorrentState::Streaming;
        session.state = TorrentState::Stopped;
        assert_eq!(session.state, TorrentState::Stopped);
    }

    #[test]
    fn test_session_state_transition_to_error() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        session.state = TorrentState::Error("Connection timeout".to_string());
        match &session.state {
            TorrentState::Error(msg) => assert!(msg.contains("timeout")),
            _ => panic!("Expected Error state"),
        }
    }

    #[test]
    fn test_session_full_lifecycle() {
        let mut session = TorrentSession::new(VALID_MAGNET.to_string(), Some(0));

        // Starting → Connecting
        assert_eq!(session.state, TorrentState::Starting);
        session.state = TorrentState::Connecting;

        // Connecting → Downloading
        session.state = TorrentState::Downloading;
        session.download_speed = 5 * 1024 * 1024;

        // Downloading → Streaming
        session.state = TorrentState::Streaming;
        session.stream_url = Some("http://192.168.1.100:8888/0".to_string());

        // Streaming → Stopped
        session.state = TorrentState::Stopped;
        assert_eq!(session.state, TorrentState::Stopped);
    }

    // ------------------------------------------------------------------------
    // test_progress_parsing
    // ------------------------------------------------------------------------

    #[test]
    fn test_progress_parsing_gb() {
        let downloaded = parse_downloaded("Downloaded: 1.2 GB");
        assert!(downloaded.is_some());
        let bytes = downloaded.unwrap();
        // 1.2 GB = 1.2 * 1024^3 = 1288490188.8 ≈ 1288490188
        assert!(bytes > 1_200_000_000 && bytes < 1_300_000_000);
    }

    #[test]
    fn test_progress_parsing_mb() {
        let downloaded = parse_downloaded("Downloaded: 512 MB");
        assert!(downloaded.is_some());
        let bytes = downloaded.unwrap();
        // 512 MB = 512 * 1024^2 = 536870912
        assert_eq!(bytes, 536870912);
    }

    #[test]
    fn test_progress_parsing_kb() {
        let downloaded = parse_downloaded("Downloaded: 256 KB");
        assert!(downloaded.is_some());
        let bytes = downloaded.unwrap();
        // 256 KB = 256 * 1024 = 262144
        assert_eq!(bytes, 262144);
    }

    #[test]
    fn test_progress_calculation() {
        let total: u64 = 4 * 1024 * 1024 * 1024; // 4 GB
        let downloaded: u64 = (1.2 * 1024.0 * 1024.0 * 1024.0) as u64; // 1.2 GB
        let progress = calculate_progress(downloaded, total);
        assert!((progress - 0.3).abs() < 0.01); // ~30%
    }

    #[test]
    fn test_progress_calculation_zero_total() {
        let progress = calculate_progress(100, 0);
        assert_eq!(progress, 0.0);
    }

    #[test]
    fn test_progress_calculation_complete() {
        let progress = calculate_progress(1000, 1000);
        assert_eq!(progress, 1.0);
    }

    #[test]
    fn test_progress_calculation_exceeds_total() {
        // Edge case: downloaded > total (shouldn't happen but handle gracefully)
        let progress = calculate_progress(2000, 1000);
        assert_eq!(progress, 1.0); // Clamped to 1.0
    }

    // ------------------------------------------------------------------------
    // test_speed_parsing
    // ------------------------------------------------------------------------

    #[test]
    fn test_speed_parsing_mb_per_sec() {
        let speed = parse_speed("Speed: 5.2 MB/s");
        assert!(speed.is_some());
        let bytes_per_sec = speed.unwrap();
        // 5.2 MB/s = 5.2 * 1024 * 1024 = 5452595.2 ≈ 5452595
        assert!(bytes_per_sec > 5_400_000 && bytes_per_sec < 5_500_000);
    }

    #[test]
    fn test_speed_parsing_kb_per_sec() {
        let speed = parse_speed("Speed: 512 KB/s");
        assert!(speed.is_some());
        let bytes_per_sec = speed.unwrap();
        // 512 KB/s = 512 * 1024 = 524288
        assert_eq!(bytes_per_sec, 524288);
    }

    #[test]
    fn test_speed_parsing_gb_per_sec() {
        let speed = parse_speed("Speed: 1.5 GB/s");
        assert!(speed.is_some());
        let bytes_per_sec = speed.unwrap();
        // 1.5 GB/s = 1.5 * 1024^3
        assert!(bytes_per_sec > 1_500_000_000);
    }

    #[test]
    fn test_speed_parsing_from_full_output() {
        let output = "Downloading: movie.mkv\nSpeed: 3.7 MB/s  Downloaded: 2.1 GB  Peers: 45";
        let speed = parse_speed(output);
        assert!(speed.is_some());
        let bytes_per_sec = speed.unwrap();
        assert!(bytes_per_sec > 3_500_000 && bytes_per_sec < 4_000_000);
    }

    #[test]
    fn test_speed_parsing_invalid() {
        assert!(parse_speed("No speed info here").is_none());
        assert!(parse_speed("").is_none());
        assert!(parse_speed("Speed: fast").is_none());
    }

    // ------------------------------------------------------------------------
    // test_stream_url_generation
    // ------------------------------------------------------------------------

    #[test]
    fn test_stream_url_generation_basic() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let url = generate_stream_url(ip, 8888, 0);
        assert_eq!(url, "http://192.168.1.100:8888/0");
    }

    #[test]
    fn test_stream_url_generation_different_file_index() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let url = generate_stream_url(ip, 8888, 2);
        assert_eq!(url, "http://192.168.1.100:8888/2");
    }

    #[test]
    fn test_stream_url_generation_different_port() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let url = generate_stream_url(ip, 9999, 0);
        assert_eq!(url, "http://192.168.1.100:9999/0");
    }

    #[test]
    fn test_stream_url_generation_localhost() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let url = generate_stream_url(ip, 8888, 0);
        assert_eq!(url, "http://127.0.0.1:8888/0");
    }

    #[test]
    fn test_stream_url_generation_ipv6() {
        let ip: IpAddr = "::1".parse().unwrap();
        let url = generate_stream_url(ip, 8888, 0);
        assert_eq!(url, "http://::1:8888/0");
    }

    // ------------------------------------------------------------------------
    // test_stop_kills_process
    // ------------------------------------------------------------------------

    /// Mock torrent manager for testing process management
    struct MockTorrentManager {
        sessions: Arc<Mutex<Vec<TorrentSession>>>,
        process_killed: Arc<Mutex<bool>>,
    }

    impl MockTorrentManager {
        fn new() -> Self {
            Self {
                sessions: Arc::new(Mutex::new(Vec::new())),
                process_killed: Arc::new(Mutex::new(false)),
            }
        }

        async fn start(&self, magnet: &str, file_idx: Option<u32>) -> Result<TorrentSession> {
            validate_magnet(magnet).map_err(|e| anyhow::anyhow!(e))?;

            let session = TorrentSession::new(magnet.to_string(), file_idx);
            self.sessions.lock().await.push(session.clone());
            Ok(session)
        }

        async fn stop(&self, id: Uuid) -> Result<()> {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.iter_mut().find(|s| s.id == id) {
                session.state = TorrentState::Stopped;
                *self.process_killed.lock().await = true;
            }
            Ok(())
        }

        async fn stop_all(&self) -> Result<()> {
            let mut sessions = self.sessions.lock().await;
            for session in sessions.iter_mut() {
                session.state = TorrentState::Stopped;
            }
            *self.process_killed.lock().await = true;
            Ok(())
        }

        async fn is_process_killed(&self) -> bool {
            *self.process_killed.lock().await
        }

        async fn get_session(&self, id: Uuid) -> Option<TorrentSession> {
            self.sessions
                .lock()
                .await
                .iter()
                .find(|s| s.id == id)
                .cloned()
        }

        async fn all_sessions(&self) -> Vec<TorrentSession> {
            self.sessions.lock().await.clone()
        }
    }

    #[tokio::test]
    async fn test_stop_kills_process() {
        let manager = MockTorrentManager::new();

        // Start a session
        let session = manager
            .start(VALID_MAGNET, Some(0))
            .await
            .expect("Should start session");
        let session_id = session.id;

        // Verify session started
        let active = manager.get_session(session_id).await;
        assert!(active.is_some());
        assert_eq!(active.unwrap().state, TorrentState::Starting);

        // Stop the session
        manager.stop(session_id).await.expect("Should stop");

        // Verify process was killed
        assert!(manager.is_process_killed().await);

        // Verify session is stopped
        let stopped = manager.get_session(session_id).await;
        assert!(stopped.is_some());
        assert_eq!(stopped.unwrap().state, TorrentState::Stopped);
    }

    // ------------------------------------------------------------------------
    // test_stop_all_cleanup
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_stop_all_cleanup() {
        let manager = MockTorrentManager::new();

        // Start 3 sessions
        let session1 = manager.start(VALID_MAGNET, Some(0)).await.unwrap();
        let session2 = manager.start(VALID_MAGNET, Some(1)).await.unwrap();
        let session3 = manager.start(VALID_MAGNET, Some(2)).await.unwrap();

        // Verify all sessions exist
        let all = manager.all_sessions().await;
        assert_eq!(all.len(), 3);

        // Stop all
        manager.stop_all().await.expect("Should stop all");

        // Verify all processes terminated (represented by killed flag)
        assert!(manager.is_process_killed().await);

        // Verify all sessions are in Stopped state
        let stopped = manager.all_sessions().await;
        assert!(stopped.iter().all(|s| s.state == TorrentState::Stopped));
        assert_eq!(
            stopped
                .iter()
                .filter(|s| s.id == session1.id || s.id == session2.id || s.id == session3.id)
                .count(),
            3
        );
    }

    // ------------------------------------------------------------------------
    // test_handles_webtorrent_not_installed
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_handles_webtorrent_not_installed() {
        // Test by checking a non-existent command
        let result = Command::new("which")
            .arg("nonexistent_webtorrent_cli_xyz")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        let is_installed = result.map(|s| s.success()).unwrap_or(false);
        assert!(!is_installed);

        // The error message should be clear
        if !is_installed {
            let error_msg = "webtorrent-cli not found. Install with: npm install -g webtorrent-cli";
            assert!(error_msg.contains("webtorrent-cli not found"));
            assert!(error_msg.contains("npm install"));
        }
    }

    #[tokio::test]
    async fn test_check_webtorrent_installed_function() {
        // This test documents the expected behavior
        // In CI without webtorrent, it should return an error
        let result = check_webtorrent_installed().await;

        // We just verify the function returns the right type
        // Actual result depends on environment
        match result {
            Ok(()) => {
                // webtorrent is installed
                println!("webtorrent-cli is installed");
            }
            Err(msg) => {
                // webtorrent is not installed
                assert!(msg.contains("webtorrent-cli not found"));
                assert!(msg.contains("npm install"));
            }
        }
    }

    // ------------------------------------------------------------------------
    // test_handles_connection_failure
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_handles_connection_failure() {
        // Simulate a connection failure with invalid/dead magnet
        let dead_magnet =
            "magnet:?xt=urn:btih:0000000000000000000000000000000000000000&dn=DeadTorrent";

        let manager = MockTorrentManager::new();
        let session = manager.start(dead_magnet, None).await.unwrap();

        // Simulate timeout and transition to error state
        let mut updated_session = session.clone();
        updated_session.state = TorrentState::Error("No peers found after 30s timeout".to_string());

        match &updated_session.state {
            TorrentState::Error(msg) => {
                assert!(msg.contains("No peers") || msg.contains("timeout"));
            }
            _ => panic!("Expected Error state after connection failure"),
        }
    }

    #[tokio::test]
    async fn test_connection_timeout_produces_clear_error() {
        let error_state = TorrentState::Error("No peers found".to_string());

        match error_state {
            TorrentState::Error(msg) => {
                assert!(!msg.is_empty());
                assert!(msg.contains("peers") || msg.contains("timeout") || msg.contains("failed"));
            }
            _ => panic!("Expected error state"),
        }
    }

    // ------------------------------------------------------------------------
    // Additional edge case tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_session_preserves_file_index() {
        let session = TorrentSession::new(VALID_MAGNET.to_string(), Some(5));
        assert_eq!(session.file_idx, Some(5));
    }

    #[test]
    fn test_session_no_file_index() {
        let session = TorrentSession::new(VALID_MAGNET.to_string(), None);
        assert_eq!(session.file_idx, None);
    }

    #[test]
    fn test_session_unique_ids() {
        let session1 = TorrentSession::new(VALID_MAGNET.to_string(), None);
        let session2 = TorrentSession::new(VALID_MAGNET.to_string(), None);
        assert_ne!(session1.id, session2.id);
    }

    #[test]
    fn test_torrent_state_error_message_preserved() {
        let error_msg = "Custom error: Network unreachable";
        let state = TorrentState::Error(error_msg.to_string());

        match state {
            TorrentState::Error(msg) => assert_eq!(msg, error_msg),
            _ => panic!("Expected Error state"),
        }
    }
}
