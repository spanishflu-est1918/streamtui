//! CLI Command Tests
//!
//! Tests for all CLI commands with mocked backends.
//! Covers JSON output format, exit codes, and input validation.

// =============================================================================
// CLI Argument Parsing Tests
// =============================================================================

mod cli_parsing {
    use clap::Parser;
    use streamtui::cli::{
        Cli, Command, ExitCode as CliExitCode, MediaTypeFilter, QualityFilter, SeekCmd,
        SeekPosition, StreamSort, TrendingWindow, VolumeCmd, VolumeLevel,
    };

    #[test]
    fn test_no_args_is_tui_mode() {
        let cli = Cli::parse_from::<_, &str>([]);
        assert!(!cli.is_cli_mode());
    }

    #[test]
    fn test_search_command_basic() {
        let cli = Cli::parse_from(["streamtui", "search", "batman"]);
        assert!(cli.is_cli_mode());
        match cli.command {
            Some(Command::Search(cmd)) => {
                assert_eq!(cmd.query, "batman");
                assert_eq!(cmd.limit, 20); // default
                assert!(cmd.media_type.is_none());
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_search_with_filters() {
        let cli = Cli::parse_from([
            "streamtui",
            "search",
            "batman",
            "--limit",
            "10",
            "-t",
            "movie",
            "--year-from",
            "2020",
            "--year-to",
            "2024",
        ]);
        match cli.command {
            Some(Command::Search(cmd)) => {
                assert_eq!(cmd.query, "batman");
                assert_eq!(cmd.limit, 10);
                assert_eq!(cmd.media_type, Some(MediaTypeFilter::Movie));
                assert_eq!(cmd.year_from, Some(2020));
                assert_eq!(cmd.year_to, Some(2024));
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_trending_command() {
        let cli = Cli::parse_from(["streamtui", "trending", "-w", "week", "-l", "5"]);
        match cli.command {
            Some(Command::Trending(cmd)) => {
                assert_eq!(cmd.window, TrendingWindow::Week);
                assert_eq!(cmd.limit, 5);
            }
            _ => panic!("Expected Trending command"),
        }
    }

    #[test]
    fn test_info_command() {
        let cli = Cli::parse_from(["streamtui", "info", "12345", "-t", "movie"]);
        match cli.command {
            Some(Command::Info(cmd)) => {
                assert_eq!(cmd.id, "12345");
                assert_eq!(cmd.media_type, Some(MediaTypeFilter::Movie));
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn test_streams_command_movie() {
        let cli = Cli::parse_from(["streamtui", "streams", "tt1877830"]);
        match cli.command {
            Some(Command::Streams(cmd)) => {
                assert_eq!(cmd.imdb_id, "tt1877830");
                assert!(cmd.season.is_none());
                assert!(cmd.episode.is_none());
            }
            _ => panic!("Expected Streams command"),
        }
    }

    #[test]
    fn test_streams_command_tv() {
        let cli = Cli::parse_from([
            "streamtui",
            "streams",
            "tt0903747",
            "-s",
            "1",
            "-e",
            "5",
            "-Q",
            "1080p",
            "--sort",
            "quality",
        ]);
        match cli.command {
            Some(Command::Streams(cmd)) => {
                assert_eq!(cmd.imdb_id, "tt0903747");
                assert_eq!(cmd.season, Some(1));
                assert_eq!(cmd.episode, Some(5));
                assert_eq!(cmd.quality, Some(QualityFilter::Q1080p));
                assert_eq!(cmd.sort, StreamSort::Quality);
            }
            _ => panic!("Expected Streams command"),
        }
    }

    #[test]
    fn test_subtitles_command() {
        let cli = Cli::parse_from([
            "streamtui",
            "subtitles",
            "tt1877830",
            "--lang",
            "en,es,fr",
            "--trusted",
        ]);
        match cli.command {
            Some(Command::Subtitles(cmd)) => {
                assert_eq!(cmd.imdb_id, "tt1877830");
                assert_eq!(cmd.languages(), vec!["en", "es", "fr"]);
                assert!(cmd.trusted);
                assert!(!cmd.hearing_impaired);
            }
            _ => panic!("Expected Subtitles command"),
        }
    }

    #[test]
    fn test_devices_command() {
        let cli = Cli::parse_from(["streamtui", "devices", "-t", "10", "--refresh"]);
        match cli.command {
            Some(Command::Devices(cmd)) => {
                assert_eq!(cmd.timeout, 10);
                assert!(cmd.refresh);
            }
            _ => panic!("Expected Devices command"),
        }
    }

    #[test]
    fn test_cast_command() {
        let cli = Cli::parse_from([
            "streamtui",
            "cast",
            "tt1877830",
            "-d",
            "Living Room TV",
            "-Q",
            "1080p",
            "--subtitle",
            "en",
        ]);
        match cli.command {
            Some(Command::Cast(cmd)) => {
                assert_eq!(cmd.imdb_id, "tt1877830");
                assert_eq!(cmd.device, Some("Living Room TV".to_string()));
                assert_eq!(cmd.quality, Some(QualityFilter::Q1080p));
                assert_eq!(cmd.subtitle, Some("en".to_string()));
                assert!(!cmd.no_subtitle);
            }
            _ => panic!("Expected Cast command"),
        }
    }

    #[test]
    fn test_cast_tv_episode() {
        let cli = Cli::parse_from([
            "streamtui",
            "cast",
            "tt0903747",
            "-d",
            "TV",
            "-s",
            "3",
            "-e",
            "7",
            "--index",
            "2",
        ]);
        match cli.command {
            Some(Command::Cast(cmd)) => {
                assert_eq!(cmd.imdb_id, "tt0903747");
                assert_eq!(cmd.season, Some(3));
                assert_eq!(cmd.episode, Some(7));
                assert_eq!(cmd.index, Some(2));
            }
            _ => panic!("Expected Cast command"),
        }
    }

    #[test]
    fn test_status_command() {
        let cli = Cli::parse_from(["streamtui", "status", "--watch", "-i", "2"]);
        match cli.command {
            Some(Command::Status(cmd)) => {
                assert!(cmd.watch);
                assert_eq!(cmd.interval, 2);
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_playback_commands() {
        // Play
        let cli = Cli::parse_from(["streamtui", "play"]);
        assert!(matches!(cli.command, Some(Command::Play(_))));

        // Pause
        let cli = Cli::parse_from(["streamtui", "pause"]);
        assert!(matches!(cli.command, Some(Command::Pause(_))));

        // Stop
        let cli = Cli::parse_from(["streamtui", "stop", "--kill-stream"]);
        match cli.command {
            Some(Command::Stop(cmd)) => assert!(cmd.kill_stream),
            _ => panic!("Expected Stop command"),
        }
    }

    #[test]
    fn test_seek_position_parsing_absolute() {
        let cmd = SeekCmd {
            position: "3600".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(3600));
    }

    #[test]
    fn test_seek_position_parsing_forward() {
        let cmd = SeekCmd {
            position: "+30".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Forward(30));
    }

    #[test]
    fn test_seek_position_parsing_backward() {
        let cmd = SeekCmd {
            position: "-15".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Backward(15));
    }

    #[test]
    fn test_seek_position_parsing_timestamp() {
        // MM:SS
        let cmd = SeekCmd {
            position: "5:30".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(330)); // 5*60+30

        // HH:MM:SS
        let cmd = SeekCmd {
            position: "1:30:00".to_string(),
        };
        assert_eq!(cmd.parse_position(), SeekPosition::Absolute(5400)); // 1*3600+30*60
    }

    #[test]
    fn test_seek_position_parsing_invalid() {
        let cmd = SeekCmd {
            position: "invalid".to_string(),
        };
        assert!(matches!(cmd.parse_position(), SeekPosition::Invalid(_)));
    }

    #[test]
    fn test_volume_parsing_absolute() {
        let cmd = VolumeCmd {
            level: "50".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Absolute(50));
    }

    #[test]
    fn test_volume_parsing_capped() {
        // Values over 100 should be capped
        let cmd = VolumeCmd {
            level: "150".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Absolute(100));
    }

    #[test]
    fn test_volume_parsing_relative() {
        let cmd = VolumeCmd {
            level: "+10".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Relative(10));

        let cmd = VolumeCmd {
            level: "-5".to_string(),
        };
        assert_eq!(cmd.parse_level(), VolumeLevel::Relative(-5));
    }

    #[test]
    fn test_volume_parsing_invalid() {
        let cmd = VolumeCmd {
            level: "loud".to_string(),
        };
        assert!(matches!(cmd.parse_level(), VolumeLevel::Invalid(_)));
    }

    #[test]
    fn test_global_flags() {
        let cli = Cli::parse_from([
            "streamtui",
            "--json",
            "--device",
            "My TV",
            "--quiet",
            "--config",
            "/path/to/config.toml",
            "search",
            "test",
        ]);
        assert!(cli.json);
        assert!(cli.quiet);
        assert_eq!(cli.device, Some("My TV".to_string()));
        assert_eq!(
            cli.config,
            Some(std::path::PathBuf::from("/path/to/config.toml"))
        );
    }

    #[test]
    fn test_command_aliases() {
        // Search alias: s
        let cli = Cli::parse_from(["streamtui", "s", "test"]);
        assert!(matches!(cli.command, Some(Command::Search(_))));

        // Trending alias: tr
        let cli = Cli::parse_from(["streamtui", "tr"]);
        assert!(matches!(cli.command, Some(Command::Trending(_))));

        // Info alias: i
        let cli = Cli::parse_from(["streamtui", "i", "123"]);
        assert!(matches!(cli.command, Some(Command::Info(_))));

        // Streams alias: st
        let cli = Cli::parse_from(["streamtui", "st", "tt1234567"]);
        assert!(matches!(cli.command, Some(Command::Streams(_))));

        // Subtitles alias: sub
        let cli = Cli::parse_from(["streamtui", "sub", "tt1234567"]);
        assert!(matches!(cli.command, Some(Command::Subtitles(_))));

        // Devices alias: dev
        let cli = Cli::parse_from(["streamtui", "dev"]);
        assert!(matches!(cli.command, Some(Command::Devices(_))));

        // Volume alias: vol
        let cli = Cli::parse_from(["streamtui", "vol", "50"]);
        assert!(matches!(cli.command, Some(Command::Volume(_))));
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(i32::from(CliExitCode::Success), 0);
        assert_eq!(i32::from(CliExitCode::Error), 1);
        assert_eq!(i32::from(CliExitCode::InvalidArgs), 2);
        assert_eq!(i32::from(CliExitCode::NetworkError), 3);
        assert_eq!(i32::from(CliExitCode::DeviceNotFound), 4);
        assert_eq!(i32::from(CliExitCode::NoStreams), 5);
        assert_eq!(i32::from(CliExitCode::CastFailed), 6);
    }
}

// =============================================================================
// IMDB ID Validation Tests
// =============================================================================

mod imdb_validation {
    use streamtui::cli::validate_imdb_id;

    #[test]
    fn test_valid_imdb_ids() {
        assert!(validate_imdb_id("tt1877830").is_ok());
        assert!(validate_imdb_id("tt0903747").is_ok());
        assert!(validate_imdb_id("tt12345678").is_ok());
        assert!(validate_imdb_id("tt1234567890").is_ok());
    }

    #[test]
    fn test_invalid_imdb_ids() {
        // Too short (less than 7 digits)
        assert!(validate_imdb_id("tt123456").is_err());
        assert!(validate_imdb_id("tt12345").is_err());

        // Wrong prefix
        assert!(validate_imdb_id("nm1234567").is_err());
        assert!(validate_imdb_id("co1234567").is_err());

        // No prefix
        assert!(validate_imdb_id("1234567").is_err());

        // Letters in numeric part
        assert!(validate_imdb_id("tt123abc7").is_err());

        // Empty
        assert!(validate_imdb_id("").is_err());
        assert!(validate_imdb_id("tt").is_err());
    }
}

// =============================================================================
// JSON Output Format Tests
// =============================================================================

mod json_output {
    use serde_json;
    use streamtui::cli::{ExitCode, JsonOutput, PlaybackState, PlaybackStatus, StatusOk};

    #[test]
    fn test_json_output_success() {
        let output = JsonOutput::success("test data");
        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("\"data\":\"test data\""));
        assert!(!json.contains("error"));
        assert!(!json.contains("exit_code")); // Should be omitted when 0
    }

    #[test]
    fn test_json_output_error() {
        let output = JsonOutput::<()>::error_msg("Something went wrong", ExitCode::NetworkError);
        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("\"error\":\"Something went wrong\""));
        assert!(json.contains("\"exit_code\":3"));
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_status_ok_format() {
        let status = StatusOk::default();
        let json = serde_json::to_string(&status).unwrap();

        assert_eq!(json, r#"{"status":"ok"}"#);
    }

    #[test]
    fn test_playback_status_idle() {
        let status = PlaybackStatus::default();
        let json = serde_json::to_string(&status).unwrap();

        assert!(json.contains("\"state\":\"idle\""));
        // Optional fields should be omitted when None
        assert!(!json.contains("\"title\""));
    }

    #[test]
    fn test_playback_status_playing() {
        let status = PlaybackStatus {
            state: PlaybackState::Playing,
            title: Some("The Batman".to_string()),
            device: Some("Living Room TV".to_string()),
            position: Some(1234),
            duration: Some(10560),
            progress: Some(0.117),
            volume: Some(80),
        };
        let json = serde_json::to_string(&status).unwrap();

        assert!(json.contains("\"state\":\"playing\""));
        assert!(json.contains("\"title\":\"The Batman\""));
        assert!(json.contains("\"device\":\"Living Room TV\""));
        assert!(json.contains("\"position\":1234"));
        assert!(json.contains("\"duration\":10560"));
    }

    #[test]
    fn test_playback_states_serialization() {
        let states = vec![
            (PlaybackState::Idle, "idle"),
            (PlaybackState::Buffering, "buffering"),
            (PlaybackState::Playing, "playing"),
            (PlaybackState::Paused, "paused"),
            (PlaybackState::Stopped, "stopped"),
            (PlaybackState::Error, "error"),
        ];

        for (state, expected) in states {
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }
}

// =============================================================================
// Cast Device Parsing Tests
// =============================================================================

mod device_parsing {
    use streamtui::models::CastDevice;

    #[test]
    fn test_parse_catt_scan_single_device() {
        let output = "Living Room TV - 192.168.1.50";
        let devices = CastDevice::parse_catt_scan(output);

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Living Room TV");
        assert_eq!(devices[0].address.to_string(), "192.168.1.50");
    }

    #[test]
    fn test_parse_catt_scan_multiple_devices() {
        let output = r#"
Living Room TV - 192.168.1.50
Bedroom TV - 192.168.1.51
Kitchen Speaker - 192.168.1.52
"#;
        let devices = CastDevice::parse_catt_scan(output);

        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].name, "Living Room TV");
        assert_eq!(devices[1].name, "Bedroom TV");
        assert_eq!(devices[2].name, "Kitchen Speaker");
    }

    #[test]
    fn test_parse_catt_scan_with_scanning_message() {
        let output = r#"
Scanning for Chromecast devices...
Living Room TV - 192.168.1.50
1 device found
"#;
        let devices = CastDevice::parse_catt_scan(output);

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Living Room TV");
    }

    #[test]
    fn test_parse_catt_scan_no_devices() {
        let output = "No devices found";
        let devices = CastDevice::parse_catt_scan(output);
        assert!(devices.is_empty());
    }

    #[test]
    fn test_parse_catt_scan_empty() {
        let devices = CastDevice::parse_catt_scan("");
        assert!(devices.is_empty());
    }

    #[test]
    fn test_parse_catt_scan_invalid_lines() {
        let output = r#"
Some random text
Living Room TV - 192.168.1.50
Invalid line without IP
Another Device - not.an.ip.address
"#;
        let devices = CastDevice::parse_catt_scan(output);

        // Only the valid device should be parsed
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Living Room TV");
    }
}

// =============================================================================
// Playback Status Parsing Tests
// =============================================================================

mod status_parsing {
    use std::time::Duration;
    use streamtui::models::{CastState, PlaybackStatus};

    #[test]
    fn test_parse_catt_status_playing() {
        let output = r#"
State: PLAYING
Duration: 10560.5
Current time: 1234.5
Volume: 80
Title: The Batman
"#;
        let status = PlaybackStatus::parse_catt_status(output).unwrap();

        assert_eq!(status.state, CastState::Playing);
        assert_eq!(status.duration.as_secs(), 10560);
        assert_eq!(status.position.as_secs(), 1234);
        assert_eq!((status.volume * 100.0) as u8, 80);
        assert_eq!(status.title, Some("The Batman".to_string()));
    }

    #[test]
    fn test_parse_catt_status_paused() {
        let output = r#"
State: PAUSED
Duration: 5400.0
Current time: 2700.0
Volume: 50
"#;
        let status = PlaybackStatus::parse_catt_status(output).unwrap();

        assert_eq!(status.state, CastState::Paused);
        assert_eq!(status.duration, Duration::from_secs(5400));
        assert_eq!(status.position, Duration::from_secs(2700));
    }

    #[test]
    fn test_parse_catt_status_idle() {
        let output = "State: IDLE";
        let status = PlaybackStatus::parse_catt_status(output).unwrap();

        assert_eq!(status.state, CastState::Idle);
        assert_eq!(status.duration, Duration::ZERO);
        assert_eq!(status.position, Duration::ZERO);
    }

    #[test]
    fn test_parse_catt_status_buffering() {
        let output = "State: BUFFERING";
        let status = PlaybackStatus::parse_catt_status(output).unwrap();
        assert_eq!(status.state, CastState::Buffering);
    }

    #[test]
    fn test_parse_catt_status_progress() {
        let output = r#"
State: PLAYING
Duration: 1000.0
Current time: 500.0
"#;
        let status = PlaybackStatus::parse_catt_status(output).unwrap();
        assert!((status.progress() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_parse_catt_status_format_position() {
        let output = r#"
State: PLAYING
Duration: 7384.0
Current time: 3692.0
"#;
        let status = PlaybackStatus::parse_catt_status(output).unwrap();
        // 3692 seconds = 1h 1m 32s = "01:01:32"
        assert_eq!(status.format_position(), "01:01:32");
    }
}

// =============================================================================
// Stream Source Tests
// =============================================================================

mod stream_source {
    use streamtui::models::{Quality, StreamSource};

    #[test]
    fn test_quality_ranking() {
        assert!(Quality::UHD4K.rank() > Quality::FHD1080p.rank());
        assert!(Quality::FHD1080p.rank() > Quality::HD720p.rank());
        assert!(Quality::HD720p.rank() > Quality::SD480p.rank());
        assert!(Quality::SD480p.rank() > Quality::Unknown.rank());
    }

    #[test]
    fn test_quality_from_string() {
        assert_eq!(Quality::from_str_loose("4K"), Quality::UHD4K);
        assert_eq!(Quality::from_str_loose("2160p"), Quality::UHD4K);
        assert_eq!(Quality::from_str_loose("1080p"), Quality::FHD1080p);
        assert_eq!(Quality::from_str_loose("720p"), Quality::HD720p);
        assert_eq!(Quality::from_str_loose("480p"), Quality::SD480p);
        assert_eq!(Quality::from_str_loose("random"), Quality::Unknown);
    }

    #[test]
    fn test_parse_seeds() {
        // Format: "👤 142" or "Seeds: 142" or just numbers
        assert_eq!(StreamSource::parse_seeds("👤 142"), 142);
        assert_eq!(StreamSource::parse_seeds("Seeds: 50"), 50);
        assert_eq!(StreamSource::parse_seeds("No seeds info"), 0);
    }

    #[test]
    fn test_parse_size() {
        // Format: "4.2 GB" or "800 MB"
        let gb_size = StreamSource::parse_size("4.2 GB");
        assert!(gb_size.is_some());
        let gb = gb_size.unwrap();
        // 4.2 GB in bytes (approximately)
        assert!(gb > 4_000_000_000 && gb < 5_000_000_000);

        let mb_size = StreamSource::parse_size("800 MB");
        assert!(mb_size.is_some());
        let mb = mb_size.unwrap();
        // 800 MB in bytes
        assert!(mb > 700_000_000 && mb < 900_000_000);
    }

    #[test]
    fn test_format_size() {
        let source = StreamSource {
            name: "Test".to_string(),
            title: "Test".to_string(),
            info_hash: "abc123".to_string(),
            file_idx: None,
            seeds: 100,
            quality: Quality::FHD1080p,
            size_bytes: Some(4_500_000_000), // 4.5 GB
        };
        let formatted = source.format_size();
        assert!(formatted.contains("4.") || formatted.contains("GB"));
    }

    #[test]
    fn test_to_magnet() {
        let source = StreamSource {
            name: "The.Batman.2022.1080p.BluRay".to_string(),
            title: "Test".to_string(),
            info_hash: "abc123def456".to_string(),
            file_idx: Some(0),
            seeds: 100,
            quality: Quality::FHD1080p,
            size_bytes: None,
        };
        let magnet = source.to_magnet("tt1877830");

        assert!(magnet.starts_with("magnet:?"));
        assert!(magnet.contains("xt=urn:btih:abc123def456"));
        assert!(magnet.contains("dn="));
    }
}

// =============================================================================
// Subtitle Result Tests
// =============================================================================

mod subtitle_result {
    use streamtui::models::SubtitleResult;

    fn make_subtitle(
        downloads: u32,
        from_trusted: bool,
        ai_translated: bool,
        hearing_impaired: bool,
    ) -> SubtitleResult {
        SubtitleResult {
            id: "123".to_string(),
            file_id: 123,
            language: "en".to_string(),
            language_name: "English".to_string(),
            release: "The.Batman.2022.1080p".to_string(),
            fps: Some(23.976),
            format: streamtui::models::SubFormat::Srt,
            downloads,
            from_trusted,
            hearing_impaired,
            ai_translated,
        }
    }

    #[test]
    fn test_trust_score_trusted() {
        let sub = make_subtitle(50000, true, false, false);
        // Trusted subs should have higher score (50000 + 10000 bonus)
        assert!(sub.trust_score() >= 60000);
    }

    #[test]
    fn test_trust_score_ai_translated() {
        let sub = make_subtitle(1000, false, true, false);
        // AI translated should have lower score (1000 - 5000 penalty, but saturates at 0)
        assert!(sub.trust_score() == 0);
    }

    #[test]
    fn test_trust_score_high_downloads_ai() {
        let sub = make_subtitle(10000, false, true, false);
        // 10000 - 5000 = 5000
        assert_eq!(sub.trust_score(), 5000);
    }

    #[test]
    fn test_trust_score_trusted_beats_ai() {
        let trusted_sub = make_subtitle(1000, true, false, false);
        let ai_sub = make_subtitle(10000, false, true, false);

        // Trusted should beat AI even with fewer downloads
        // trusted: 1000 + 10000 = 11000
        // ai: 10000 - 5000 = 5000
        assert!(trusted_sub.trust_score() > ai_sub.trust_score());
    }

    #[test]
    fn test_subtitle_display() {
        let sub = make_subtitle(50000, true, false, false);
        let display = sub.to_string();

        // Should contain language, release, downloads, and trust indicator
        assert!(display.contains("en") || display.contains("English"));
        assert!(display.contains("50000") || display.contains("⬇"));
    }

    #[test]
    fn test_subtitle_display_hearing_impaired() {
        let sub = make_subtitle(1000, false, false, true);
        let display = sub.to_string();

        // Should contain hearing impaired indicator
        assert!(display.contains("👂"));
    }

    #[test]
    fn test_subtitle_display_ai_translated() {
        let sub = make_subtitle(1000, false, true, false);
        let display = sub.to_string();

        // Should contain AI indicator
        assert!(display.contains("🤖"));
    }
}

// =============================================================================
// Torrent Session Tests
// =============================================================================

mod torrent_session {
    use std::net::IpAddr;
    use streamtui::models::TorrentSession;

    #[test]
    fn test_generate_stream_url() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let url = TorrentSession::generate_stream_url(ip, 8888, 0);
        assert_eq!(url, "http://192.168.1.100:8888/0");
    }

    #[test]
    fn test_generate_stream_url_different_index() {
        let ip: IpAddr = "10.0.0.5".parse().unwrap();
        let url = TorrentSession::generate_stream_url(ip, 9000, 3);
        assert_eq!(url, "http://10.0.0.5:9000/3");
    }

    #[test]
    fn test_parse_speed_mb() {
        let speed = TorrentSession::parse_speed("Speed: 5.2 MB/s");
        // 5.2 MB/s in bytes
        assert!(speed > 5_000_000 && speed < 6_000_000);
    }

    #[test]
    fn test_parse_speed_kb() {
        let speed = TorrentSession::parse_speed("Speed: 512 KB/s");
        // 512 KB/s in bytes
        assert!(speed > 500_000 && speed < 550_000);
    }

    #[test]
    fn test_parse_progress() {
        // 2 GB downloaded out of 4 GB total = 50%
        let total = 4 * 1024 * 1024 * 1024u64; // 4 GB
        let progress = TorrentSession::parse_progress("Downloaded: 2.0 GB", total);
        assert!((progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_format_speed() {
        let session = TorrentSession::new("magnet:?xt=...".to_string(), None);
        let mut session = session;
        session.download_speed = 5 * 1024 * 1024; // 5 MB/s

        let formatted = session.format_speed();
        assert!(formatted.contains("5") && formatted.contains("MB/s"));
    }

    #[test]
    fn test_format_downloaded() {
        let mut session = TorrentSession::new("magnet:?xt=...".to_string(), None);

        // Less than 1 GB: show MB
        session.downloaded = 500 * 1024 * 1024; // 500 MB
        assert!(session.format_downloaded().contains("MB"));

        // 1 GB+: show GB
        session.downloaded = 2 * 1024 * 1024 * 1024; // 2 GB
        assert!(session.format_downloaded().contains("GB"));
    }
}

// =============================================================================
// Quality Filter Tests
// =============================================================================

mod quality_filter {
    use streamtui::cli::QualityFilter;

    #[test]
    fn test_quality_filter_display() {
        assert_eq!(QualityFilter::Q4k.to_string(), "4K");
        assert_eq!(QualityFilter::Q1080p.to_string(), "1080p");
        assert_eq!(QualityFilter::Q720p.to_string(), "720p");
        assert_eq!(QualityFilter::Q480p.to_string(), "480p");
    }
}

// =============================================================================
// Output Helper Tests
// =============================================================================

mod output_helpers {
    use clap::Parser;
    use streamtui::cli::{Cli, Output};

    #[test]
    fn test_output_json_mode() {
        // With --json flag
        let cli = Cli::parse_from(["streamtui", "--json", "status"]);
        let output = Output::new(&cli);
        assert!(output.json);
    }

    #[test]
    fn test_output_quiet_mode() {
        let cli = Cli::parse_from(["streamtui", "--quiet", "status"]);
        let output = Output::new(&cli);
        assert!(output.quiet);
    }

    #[test]
    fn test_should_json_with_flag() {
        let cli = Cli::parse_from(["streamtui", "--json", "search", "test"]);
        assert!(cli.should_json());
    }

    #[test]
    fn test_should_json_without_flag() {
        // When stdout is a TTY, should_json returns false without --json
        // This test doesn't actually check TTY (hard to test), just the flag
        let cli = Cli::parse_from(["streamtui", "search", "test"]);
        // Can't easily test TTY detection, but we can verify the flag isn't set
        assert!(!cli.json);
    }
}

// =============================================================================
// Cast Command Helper Tests
// =============================================================================

mod cast_helpers {
    use streamtui::cli::{CastCmd, QualityFilter};

    fn make_cast_cmd(device: Option<String>) -> CastCmd {
        CastCmd {
            imdb_id: "tt1877830".to_string(),
            device,
            quality: Some(QualityFilter::Q1080p),
            season: None,
            episode: None,
            index: None,
            subtitle: None,
            subtitle_id: None,
            no_subtitle: false,
            start: None,
            vlc: false,
        }
    }

    #[test]
    fn test_effective_device_command_specific() {
        let cmd = make_cast_cmd(Some("Command TV".to_string()));
        let global = Some("Global TV".to_string());

        // Command-specific should take precedence
        assert_eq!(cmd.effective_device(&global), Some("Command TV"));
    }

    #[test]
    fn test_effective_device_fallback_to_global() {
        let cmd = make_cast_cmd(None);
        let global = Some("Global TV".to_string());

        assert_eq!(cmd.effective_device(&global), Some("Global TV"));
    }

    #[test]
    fn test_effective_device_none() {
        let cmd = make_cast_cmd(None);
        let global: Option<String> = None;

        assert_eq!(cmd.effective_device(&global), None);
    }
}

// =============================================================================
// Integration Test Helpers
// =============================================================================

mod integration_helpers {
    use streamtui::models::CastState;

    #[test]
    fn test_cast_state_from_string() {
        assert_eq!(CastState::from_catt_state("PLAYING"), CastState::Playing);
        assert_eq!(CastState::from_catt_state("playing"), CastState::Playing);
        assert_eq!(CastState::from_catt_state("PAUSED"), CastState::Paused);
        assert_eq!(
            CastState::from_catt_state("BUFFERING"),
            CastState::Buffering
        );
        assert_eq!(CastState::from_catt_state("IDLE"), CastState::Idle);
        assert_eq!(CastState::from_catt_state("STOPPED"), CastState::Stopped);
        assert_eq!(CastState::from_catt_state("unknown"), CastState::Idle);
    }

    #[test]
    fn test_cast_state_display() {
        assert!(CastState::Playing.to_string().contains("Playing"));
        assert!(CastState::Paused.to_string().contains("Paused"));
        assert!(CastState::Buffering.to_string().contains("Buffering"));
        assert!(CastState::Idle.to_string().contains("Idle"));
    }
}
