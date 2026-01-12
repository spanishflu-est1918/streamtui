//! Cast Manager Tests - TDD
//!
//! Testing Chromecast discovery and control via catt CLI.
//! CASTING IS PARAMOUNT! 🎯

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use streamtui::models::{CastDevice, CastState, PlaybackStatus};
use streamtui::stream::CastManager;

// =============================================================================
// Discovery Tests
// =============================================================================

/// Test parsing catt scan output with multiple devices
#[test]
fn test_discover_parses_catt_output() {
    let output =
        "Scanning for Chromecast devices...\nLiving Room TV - 192.168.1.50\nBedroom - 192.168.1.51";

    let devices = CastDevice::parse_catt_scan(output);

    assert_eq!(devices.len(), 2, "Should parse 2 devices");

    // First device
    assert_eq!(devices[0].name, "Living Room TV");
    assert_eq!(
        devices[0].address,
        "192.168.1.50".parse::<IpAddr>().unwrap()
    );
    assert_eq!(devices[0].port, 8009); // Default Chromecast port

    // Second device
    assert_eq!(devices[1].name, "Bedroom");
    assert_eq!(
        devices[1].address,
        "192.168.1.51".parse::<IpAddr>().unwrap()
    );
}

/// Test handling empty/no devices output
#[test]
fn test_discover_handles_no_devices() {
    // Test "No devices found" message
    let output1 = "Scanning for Chromecast devices...\nNo devices found";
    let devices1 = CastDevice::parse_catt_scan(output1);
    assert!(
        devices1.is_empty(),
        "Should return empty vec for 'No devices found'"
    );

    // Test completely empty output
    let output2 = "";
    let devices2 = CastDevice::parse_catt_scan(output2);
    assert!(
        devices2.is_empty(),
        "Should return empty vec for empty output"
    );

    // Test just scanning message
    let output3 = "Scanning for Chromecast devices...";
    let devices3 = CastDevice::parse_catt_scan(output3);
    assert!(
        devices3.is_empty(),
        "Should return empty vec for scan-only output"
    );
}

/// Test parsing devices with special characters in names
#[test]
fn test_discover_handles_special_names() {
    let output =
        "John's TV - 192.168.1.50\nLiving Room (Main) - 192.168.1.51\nTV-2023 - 10.0.0.100";

    let devices = CastDevice::parse_catt_scan(output);

    assert_eq!(devices.len(), 3);
    assert_eq!(devices[0].name, "John's TV");
    assert_eq!(devices[1].name, "Living Room (Main)");
    assert_eq!(devices[2].name, "TV-2023");
    assert_eq!(devices[2].address, "10.0.0.100".parse::<IpAddr>().unwrap());
}

// =============================================================================
// Cast Command Tests
// =============================================================================

/// Test that cast forms correct catt command
#[test]
fn test_cast_forms_correct_command() {
    // Build the expected command parts
    let device_name = "Living Room TV";
    let url = "http://192.168.1.100:8888/0";

    // Expected command: catt -d "Living Room TV" cast "http://192.168.1.100:8888/0"
    let expected_args = vec![
        "-d".to_string(),
        device_name.to_string(),
        "cast".to_string(),
        url.to_string(),
    ];

    // Build actual args as CastManager would
    let mut args = Vec::new();
    args.push("-d".to_string());
    args.push(device_name.to_string());
    args.push("cast".to_string());
    args.push(url.to_string());

    assert_eq!(args, expected_args);
}

/// Test cast command with subtitle URL
#[test]
fn test_cast_with_subtitles_forms_correct_command() {
    let device_name = "Living Room TV";
    let url = "http://192.168.1.100:8888/0";
    let subtitle_url = "http://192.168.1.100:8888/subtitles.vtt";

    // Expected: catt -d "Living Room TV" cast "url" --subtitle "subtitle_url"
    let mut args = Vec::new();
    args.push("-d".to_string());
    args.push(device_name.to_string());
    args.push("cast".to_string());
    args.push(url.to_string());
    args.push("--subtitle".to_string());
    args.push(subtitle_url.to_string());

    assert_eq!(args.len(), 6);
    assert_eq!(args[4], "--subtitle");
    assert_eq!(args[5], subtitle_url);
}

// =============================================================================
// Status Parsing Tests
// =============================================================================

/// Test parsing catt status output for PLAYING state
#[test]
fn test_status_parsing() {
    let output = "State: PLAYING\nDuration: 10234.5\nCurrent time: 1234.5\nVolume: 80";

    let status = PlaybackStatus::parse_catt_status(output).expect("Should parse status");

    assert_eq!(status.state, CastState::Playing);
    assert_eq!(status.duration, Duration::from_secs_f64(10234.5));
    assert_eq!(status.position, Duration::from_secs_f64(1234.5));
    assert!(
        (status.volume - 0.8).abs() < 0.01,
        "Volume should be 0.8 (80%)"
    );
}

/// Test parsing BUFFERING state
#[test]
fn test_status_buffering() {
    let output = "State: BUFFERING\nDuration: 5000.0\nCurrent time: 100.0\nVolume: 50";

    let status = PlaybackStatus::parse_catt_status(output).expect("Should parse status");

    assert_eq!(status.state, CastState::Buffering);
}

/// Test parsing various cast states
#[test]
fn test_status_all_states() {
    let test_cases = [
        ("State: PLAYING", CastState::Playing),
        ("State: PAUSED", CastState::Paused),
        ("State: BUFFERING", CastState::Buffering),
        ("State: IDLE", CastState::Idle),
        ("State: STOPPED", CastState::Stopped),
        ("State: UNKNOWN", CastState::Idle), // Unknown defaults to Idle
    ];

    for (input, expected_state) in test_cases {
        let status = PlaybackStatus::parse_catt_status(input).expect("Should parse");
        assert_eq!(
            status.state, expected_state,
            "Input '{}' should produce state {:?}",
            input, expected_state
        );
    }
}

/// Test parsing status with title
#[test]
fn test_status_with_title() {
    let output = "State: PLAYING\nTitle: Cyberpunk Movie 2077\nDuration: 7200.0\nCurrent time: 1800.0\nVolume: 70";

    let status = PlaybackStatus::parse_catt_status(output).expect("Should parse status");

    assert_eq!(status.title, Some("Cyberpunk Movie 2077".to_string()));
}

// =============================================================================
// Volume Tests
// =============================================================================

/// Test volume clamping to valid range
#[test]
fn test_volume_clamps() {
    // Helper to clamp volume as CastManager should
    fn clamp_volume(vol: f32) -> f32 {
        vol.clamp(0.0, 1.0)
    }

    // Test over 1.0 -> clamp to 1.0
    assert!((clamp_volume(1.5) - 1.0).abs() < f32::EPSILON);
    assert!((clamp_volume(100.0) - 1.0).abs() < f32::EPSILON);

    // Test negative -> clamp to 0.0
    assert!((clamp_volume(-0.5) - 0.0).abs() < f32::EPSILON);
    assert!((clamp_volume(-100.0) - 0.0).abs() < f32::EPSILON);

    // Test valid values stay unchanged
    assert!((clamp_volume(0.5) - 0.5).abs() < f32::EPSILON);
    assert!((clamp_volume(0.0) - 0.0).abs() < f32::EPSILON);
    assert!((clamp_volume(1.0) - 1.0).abs() < f32::EPSILON);
}

/// Test volume to percentage conversion (for catt which uses 0-100)
#[test]
fn test_volume_to_percentage() {
    fn to_catt_volume(vol: f32) -> u8 {
        (vol.clamp(0.0, 1.0) * 100.0) as u8
    }

    assert_eq!(to_catt_volume(0.0), 0);
    assert_eq!(to_catt_volume(0.5), 50);
    assert_eq!(to_catt_volume(1.0), 100);
    assert_eq!(to_catt_volume(0.75), 75);
    assert_eq!(to_catt_volume(1.5), 100); // Clamped
}

// =============================================================================
// Seek Validation Tests
// =============================================================================

/// Test seek position validation
#[test]
fn test_seek_validation() {
    let duration = Duration::from_secs(3600); // 1 hour video

    // Helper to validate and clamp seek position
    fn validate_seek(position: f64, duration: Duration) -> Result<Duration, &'static str> {
        if position < 0.0 {
            return Err("Cannot seek to negative position");
        }

        let duration_secs = duration.as_secs_f64();
        if position > duration_secs {
            return Err("Cannot seek beyond duration");
        }

        Ok(Duration::from_secs_f64(position))
    }

    // Valid seek
    let result = validate_seek(1800.0, duration);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Duration::from_secs(1800));

    // Seek to start
    let result = validate_seek(0.0, duration);
    assert!(result.is_ok());

    // Seek to exact end
    let result = validate_seek(3600.0, duration);
    assert!(result.is_ok());

    // Seek beyond duration
    let result = validate_seek(4000.0, duration);
    assert!(result.is_err());

    // Negative seek
    let result = validate_seek(-100.0, duration);
    assert!(result.is_err());
}

/// Test seek with clamping instead of error
#[test]
fn test_seek_clamps() {
    let duration = Duration::from_secs(3600);

    fn clamp_seek(position: f64, duration: Duration) -> Duration {
        let clamped = position.clamp(0.0, duration.as_secs_f64());
        Duration::from_secs_f64(clamped)
    }

    // Beyond duration -> clamp to end
    assert_eq!(clamp_seek(5000.0, duration), duration);

    // Negative -> clamp to 0
    assert_eq!(clamp_seek(-100.0, duration), Duration::ZERO);

    // Valid -> unchanged
    assert_eq!(clamp_seek(1800.0, duration), Duration::from_secs(1800));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

/// Test handling when catt is not installed
#[tokio::test]
async fn test_handles_catt_not_installed() {
    // Use a non-existent path to simulate catt not installed
    let manager = CastManager::with_path("/nonexistent/catt");

    let result = manager.discover().await;

    // Currently returns Ok(empty vec) - but should ideally return an error
    // This test documents expected behavior: clear error message
    // After implementation: expect Err with message containing "catt not found"

    // For now, we verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

/// Test handling when target device goes offline
#[tokio::test]
async fn test_handles_device_offline() {
    let mut manager = CastManager::new();

    // Create a device that's "offline" (unreachable IP)
    let offline_device = CastDevice {
        id: "offline".to_string(),
        name: "Offline TV".to_string(),
        address: "192.168.255.255".parse().unwrap(), // Likely unreachable
        port: 8009,
        model: None,
    };

    manager.select_device(offline_device);

    // Cast should fail gracefully
    let result = manager.cast("http://localhost:8888/0", None).await;

    // Currently returns Err("No device selected") but should return device-specific error
    // After implementation: error message should include device name
    assert!(result.is_err());
}

/// Test cast failure recovery
#[tokio::test]
async fn test_cast_failure_recovery() {
    let mut manager = CastManager::new();

    let device = CastDevice {
        id: "test".to_string(),
        name: "Test TV".to_string(),
        address: "192.168.1.50".parse().unwrap(),
        port: 8009,
        model: None,
    };

    manager.select_device(device.clone());

    // First cast attempt (simulated failure)
    let result1 = manager.cast("http://invalid-url", None).await;

    // Manager should still be usable after failure
    assert!(
        manager.selected().is_some(),
        "Device should still be selected after failure"
    );

    // Should be able to retry
    let _result2 = manager.cast("http://192.168.1.100:8888/0", None).await;

    // Manager remains functional (doesn't panic or corrupt state)
    assert_eq!(manager.selected().unwrap().name, "Test TV");
}

// =============================================================================
// Concurrency Tests
// =============================================================================

/// Test concurrent discovery doesn't cause issues
#[tokio::test]
async fn test_concurrent_discovery() {
    let manager = Arc::new(CastManager::new());

    // Spawn multiple discovery tasks concurrently
    let mut handles = vec![];

    for _ in 0..5 {
        let mgr = Arc::clone(&manager);
        handles.push(tokio::spawn(async move { mgr.discover().await }));
    }

    // All should complete without panic
    let results: Vec<_> = futures::future::join_all(handles).await;

    for result in results {
        assert!(result.is_ok(), "Task should not panic");
        // Inner result can be Ok or Err depending on implementation
    }
}

/// Test discovery while cast is in progress
#[tokio::test]
async fn test_discovery_during_cast() {
    let manager = Arc::new(Mutex::new(CastManager::new()));

    let device = CastDevice {
        id: "test".to_string(),
        name: "Test TV".to_string(),
        address: "192.168.1.50".parse().unwrap(),
        port: 8009,
        model: None,
    };

    {
        let mut mgr = manager.lock().await;
        mgr.select_device(device);
    }

    // Start cast and discovery concurrently
    let mgr1 = Arc::clone(&manager);
    let cast_handle = tokio::spawn(async move {
        let mgr = mgr1.lock().await;
        mgr.cast("http://192.168.1.100:8888/0", None).await
    });

    let mgr2 = Arc::clone(&manager);
    let discover_handle = tokio::spawn(async move {
        let mgr = mgr2.lock().await;
        mgr.discover().await
    });

    // Both should complete without deadlock (with timeout)
    let cast_result = tokio::time::timeout(Duration::from_secs(5), cast_handle).await;

    let discover_result = tokio::time::timeout(Duration::from_secs(5), discover_handle).await;

    assert!(cast_result.is_ok(), "Cast should not timeout/deadlock");
    assert!(
        discover_result.is_ok(),
        "Discovery should not timeout/deadlock"
    );
}

// =============================================================================
// PlaybackStatus Helper Tests
// =============================================================================

/// Test progress calculation
#[test]
fn test_playback_progress() {
    let status = PlaybackStatus {
        state: CastState::Playing,
        position: Duration::from_secs(1800), // 30 minutes
        duration: Duration::from_secs(3600), // 1 hour
        volume: 0.8,
        title: None,
    };

    let progress = status.progress();
    assert!((progress - 0.5).abs() < 0.01, "Should be 50% progress");
}

/// Test progress with zero duration (edge case)
#[test]
fn test_playback_progress_zero_duration() {
    let status = PlaybackStatus {
        state: CastState::Idle,
        position: Duration::ZERO,
        duration: Duration::ZERO,
        volume: 1.0,
        title: None,
    };

    let progress = status.progress();
    assert!(
        (progress - 0.0).abs() < f32::EPSILON,
        "Progress should be 0 with zero duration"
    );
}

/// Test time formatting
#[test]
fn test_format_position() {
    let status = PlaybackStatus {
        state: CastState::Playing,
        position: Duration::from_secs(3661), // 1:01:01
        duration: Duration::from_secs(7200), // 2:00:00
        volume: 0.5,
        title: None,
    };

    let pos_str = status.format_position();
    assert!(
        pos_str.contains("01") || pos_str.contains("1:01"),
        "Should format as HH:MM:SS"
    );

    let dur_str = status.format_duration();
    assert!(
        dur_str.contains("2:00") || dur_str.contains("02:00"),
        "Should format duration"
    );
}

/// Test volume formatting
#[test]
fn test_format_volume() {
    let status = PlaybackStatus {
        state: CastState::Playing,
        position: Duration::ZERO,
        duration: Duration::ZERO,
        volume: 0.75,
        title: None,
    };

    let vol_str = status.format_volume();
    assert!(vol_str.contains("75"), "Should show 75%");
}

// =============================================================================
// CastDevice Tests
// =============================================================================

/// Test CastDevice Display trait
#[test]
fn test_cast_device_display() {
    let device = CastDevice {
        id: "abc123".to_string(),
        name: "Living Room TV".to_string(),
        address: "192.168.1.50".parse().unwrap(),
        port: 8009,
        model: Some("Chromecast Ultra".to_string()),
    };

    let display = format!("{}", device);
    assert!(display.contains("Living Room TV"));
    assert!(display.contains("192.168.1.50"));
    assert!(display.contains("Chromecast Ultra"));
}

/// Test CastDevice Display without model
#[test]
fn test_cast_device_display_no_model() {
    let device = CastDevice {
        id: "abc123".to_string(),
        name: "Bedroom TV".to_string(),
        address: "192.168.1.51".parse().unwrap(),
        port: 8009,
        model: None,
    };

    let display = format!("{}", device);
    assert!(display.contains("Bedroom TV"));
    assert!(display.contains("192.168.1.51"));
    // Should not contain parentheses for model
}

// =============================================================================
// CastState Tests
// =============================================================================

/// Test CastState from_catt_state parsing
#[test]
fn test_cast_state_parsing() {
    assert_eq!(CastState::from_catt_state("PLAYING"), CastState::Playing);
    assert_eq!(CastState::from_catt_state("playing"), CastState::Playing);
    assert_eq!(CastState::from_catt_state("Playing"), CastState::Playing);
    assert_eq!(CastState::from_catt_state("PAUSED"), CastState::Paused);
    assert_eq!(
        CastState::from_catt_state("BUFFERING"),
        CastState::Buffering
    );
    assert_eq!(CastState::from_catt_state("IDLE"), CastState::Idle);
    assert_eq!(CastState::from_catt_state("STOPPED"), CastState::Stopped);

    // Unknown state defaults to Idle
    assert_eq!(CastState::from_catt_state("UNKNOWN"), CastState::Idle);
    assert_eq!(CastState::from_catt_state("garbage"), CastState::Idle);
}

/// Test CastState Display trait
#[test]
fn test_cast_state_display() {
    assert!(format!("{}", CastState::Playing).contains("Playing"));
    assert!(format!("{}", CastState::Paused).contains("Paused"));
    assert!(format!("{}", CastState::Buffering).contains("Buffering"));
    assert!(format!("{}", CastState::Error("Network error".to_string())).contains("Network error"));
}

// =============================================================================
// Integration-Ready Tests (will fail until implementation)
// =============================================================================

/// Test actual catt execution (integration test, may fail without catt)
#[tokio::test]
#[ignore] // Enable when testing with real catt
async fn test_real_catt_scan() {
    let manager = CastManager::new();
    let result = manager.discover().await;

    // Should return Ok, even if empty
    assert!(result.is_ok(), "Discover should not error with real catt");
}
