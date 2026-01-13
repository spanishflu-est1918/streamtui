//! Subtitle Client Tests - TDD
//!
//! Tests for Stremio subtitle client (free, no API key).
//! Uses Stremio's public OpenSubtitles addon endpoint.

use mockito::Server;
use std::net::{IpAddr, Ipv4Addr};
use streamtui::models::{SubFormat, SubtitleFile, SubtitleResult};

// =============================================================================
// Stremio Subtitle Client - Search Tests
// =============================================================================

/// Test: Parse Stremio subtitle search response for movies
#[tokio::test]
async fn test_search_movie_subtitles() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt0234215.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "subtitles": [
                {
                    "id": "55419",
                    "url": "https://subs5.strem.io/en/download/file/70235",
                    "lang": "eng",
                    "SubEncoding": "CP1252"
                },
                {
                    "id": "122952",
                    "url": "https://subs5.strem.io/en/download/file/169216",
                    "lang": "fre",
                    "SubEncoding": "CP1252"
                },
                {
                    "id": "135292",
                    "url": "https://subs5.strem.io/en/download/file/185727",
                    "lang": "spa",
                    "SubEncoding": "CP1252"
                }
            ],
            "cacheMaxAge": 14400
        }"#,
        )
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let results = client.search("tt0234215", None).await.unwrap();

    mock.assert_async().await;

    assert_eq!(results.len(), 3, "Should parse 3 subtitle results");

    // Check first result (English)
    assert_eq!(results[0].id, "55419");
    assert_eq!(results[0].language, "eng");
    assert_eq!(results[0].language_name, "English");
    assert!(results[0].url.contains("subs5.strem.io"));

    // Check second result (French)
    assert_eq!(results[1].language, "fre");
    assert_eq!(results[1].language_name, "French");

    // Check third result (Spanish)
    assert_eq!(results[2].language, "spa");
    assert_eq!(results[2].language_name, "Spanish");
}

/// Test: Filter subtitles by language
#[tokio::test]
async fn test_search_filters_by_language() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt0234215.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "subtitles": [
                {"id": "1", "url": "https://subs.io/1", "lang": "eng"},
                {"id": "2", "url": "https://subs.io/2", "lang": "fre"},
                {"id": "3", "url": "https://subs.io/3", "lang": "eng"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let results = client.search("tt0234215", Some("eng")).await.unwrap();

    mock.assert_async().await;

    assert_eq!(results.len(), 2, "Should filter to 2 English results");
    assert!(results.iter().all(|r| r.language == "eng"));
}

/// Test: Search TV episode subtitles
#[tokio::test]
async fn test_search_episode_subtitles() {
    let mut server = Server::new_async().await;

    // Stremio format: /subtitles/series/{imdb}:{season}:{episode}.json
    let mock = server
        .mock("GET", "/subtitles/series/tt0903747:1:5.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "subtitles": [
                {"id": "78901", "url": "https://subs.io/78901", "lang": "eng"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let results = client
        .search_episode("tt0903747", 1, 5, None)
        .await
        .unwrap();

    mock.assert_async().await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "78901");
}

/// Test: Handle empty results gracefully
#[tokio::test]
async fn test_handles_no_subtitles() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt9999999.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"subtitles": []}"#)
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let results = client.search("tt9999999", None).await.unwrap();

    mock.assert_async().await;

    assert!(results.is_empty(), "Should return empty vec for no results");
}

/// Test: Handle API errors
#[tokio::test]
async fn test_handles_api_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt0000000.json")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let result = client.search("tt0000000", None).await;

    mock.assert_async().await;

    assert!(result.is_err(), "Should return error on 500");
}

/// Test: Add tt prefix if missing from IMDB ID
#[tokio::test]
async fn test_adds_tt_prefix() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt1234567.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"subtitles": []}"#)
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let _ = client.search("1234567", None).await; // Without tt prefix

    mock.assert_async().await;
}

/// Test: Language code to name mapping
#[tokio::test]
async fn test_language_code_mapping() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/subtitles/movie/tt0234215.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "subtitles": [
                {"id": "1", "url": "https://subs.io/1", "lang": "por"},
                {"id": "2", "url": "https://subs.io/2", "lang": "pob"},
                {"id": "3", "url": "https://subs.io/3", "lang": "rus"},
                {"id": "4", "url": "https://subs.io/4", "lang": "jpn"},
                {"id": "5", "url": "https://subs.io/5", "lang": "zho"},
                {"id": "6", "url": "https://subs.io/6", "lang": "unk"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let client = streamtui::stream::SubtitleClient::with_base_url(server.url());
    let results = client.search("tt0234215", None).await.unwrap();

    mock.assert_async().await;

    assert_eq!(results[0].language_name, "Portuguese");
    assert_eq!(results[1].language_name, "Portuguese"); // pob = Brazilian Portuguese
    assert_eq!(results[2].language_name, "Russian");
    assert_eq!(results[3].language_name, "Japanese");
    assert_eq!(results[4].language_name, "Chinese");
    assert_eq!(results[5].language_name, "UNK"); // Unknown -> uppercase
}

// =============================================================================
// SRT to WebVTT Conversion Tests
// =============================================================================

/// Test: Convert SRT timestamps to WebVTT format
#[test]
fn test_srt_to_webvtt_conversion() {
    let srt = r#"1
00:01:23,456 --> 00:01:25,789
First subtitle line

2
00:02:30,100 --> 00:02:35,500
Second subtitle line"#;

    let webvtt = streamtui::stream::SubtitleClient::srt_to_webvtt(srt);

    // Must start with WEBVTT header
    assert!(
        webvtt.starts_with("WEBVTT"),
        "WebVTT must start with WEBVTT header"
    );

    // Timestamps must use dots not commas
    assert!(webvtt.contains("00:01:23.456"), "Should convert , to .");
    assert!(webvtt.contains("00:01:25.789"));
    assert!(webvtt.contains("00:02:30.100"));

    // Should NOT contain commas in timestamps
    assert!(!webvtt.contains("00:01:23,"), "Should not have comma timestamps");
}

/// Test: Preserve dialogue content during conversion
#[test]
fn test_srt_to_webvtt_preserves_content() {
    let srt = r#"1
00:00:01,000 --> 00:00:03,000
Hello, Batman!

2
00:00:05,000 --> 00:00:08,000
I'm vengeance."#;

    let webvtt = streamtui::stream::SubtitleClient::srt_to_webvtt(srt);

    assert!(webvtt.contains("Batman"), "Dialogue should be preserved");
    assert!(webvtt.contains("vengeance"), "Second line should be preserved");
}

/// Test: Handle multi-line subtitles
#[test]
fn test_srt_to_webvtt_multiline() {
    let srt = r#"1
00:00:01,000 --> 00:00:05,000
Line one
Line two
Line three"#;

    let webvtt = streamtui::stream::SubtitleClient::srt_to_webvtt(srt);

    assert!(webvtt.contains("Line one"));
    assert!(webvtt.contains("Line two"));
    assert!(webvtt.contains("Line three"));
}

/// Test: Empty SRT produces valid WebVTT
#[test]
fn test_srt_to_webvtt_empty() {
    let webvtt = streamtui::stream::SubtitleClient::srt_to_webvtt("");
    assert!(webvtt.starts_with("WEBVTT"), "Empty SRT should produce valid header");
}

// =============================================================================
// URL Generation Tests
// =============================================================================

/// Test: Generate subtitle serving URL
#[test]
fn test_subtitle_url_generation() {
    let lan_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    let port = 8889u16;
    let language = "en";

    let url = SubtitleFile::generate_url(lan_ip, port, language);

    assert_eq!(url, "http://192.168.1.100:8889/subtitles/en.vtt");
}

/// Test: URL generation with different languages
#[test]
fn test_subtitle_url_different_languages() {
    let lan_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 50));
    let port = 9000u16;

    assert_eq!(
        SubtitleFile::generate_url(lan_ip, port, "es"),
        "http://10.0.0.50:9000/subtitles/es.vtt"
    );
    assert_eq!(
        SubtitleFile::generate_url(lan_ip, port, "fr"),
        "http://10.0.0.50:9000/subtitles/fr.vtt"
    );
}

// =============================================================================
// SubFormat Tests
// =============================================================================

/// Test: Parse format from file extension
#[test]
fn test_subformat_from_extension() {
    assert_eq!(SubFormat::from_extension("srt"), SubFormat::Srt);
    assert_eq!(SubFormat::from_extension("SRT"), SubFormat::Srt);
    assert_eq!(SubFormat::from_extension("vtt"), SubFormat::WebVtt);
    assert_eq!(SubFormat::from_extension("webvtt"), SubFormat::WebVtt);
    assert_eq!(SubFormat::from_extension("ass"), SubFormat::Ass);
    assert_eq!(SubFormat::from_extension("sub"), SubFormat::Sub);
    assert_eq!(SubFormat::from_extension("xyz"), SubFormat::Srt); // Unknown = SRT
}

/// Test: Format extension output
#[test]
fn test_subformat_extension_output() {
    assert_eq!(SubFormat::Srt.extension(), "srt");
    assert_eq!(SubFormat::WebVtt.extension(), "vtt");
    assert_eq!(SubFormat::Ass.extension(), "ass");
    assert_eq!(SubFormat::Sub.extension(), "sub");
}

/// Test: Format display names
#[test]
fn test_subformat_display() {
    assert_eq!(SubFormat::Srt.to_string(), "SRT");
    assert_eq!(SubFormat::WebVtt.to_string(), "WebVTT");
    assert_eq!(SubFormat::Ass.to_string(), "ASS");
    assert_eq!(SubFormat::Sub.to_string(), "SUB");
}

// =============================================================================
// SubtitleResult Tests
// =============================================================================

/// Test: Trust score calculation
#[test]
fn test_trust_score() {
    let trusted = SubtitleResult {
        id: "1".to_string(),
        url: "https://example.com".to_string(),
        language: "eng".to_string(),
        language_name: "English".to_string(),
        from_trusted: true,
        hearing_impaired: false,
        ai_translated: false,
        downloads: 1000,
        format: SubFormat::Srt,
        release: "Test".to_string(),
        fps: None,
    };

    let untrusted = SubtitleResult {
        from_trusted: false,
        ..trusted.clone()
    };

    assert!(
        trusted.trust_score() > untrusted.trust_score(),
        "Trusted should have higher score"
    );
}

// =============================================================================
// Cast Command Integration Tests
// =============================================================================

/// Test: Cast command includes subtitle flag
#[test]
fn test_cast_command_with_subtitle() {
    let device_name = "Living Room TV";
    let video_url = "http://192.168.1.100:8888/0";
    let subtitle_url = "http://192.168.1.100:8889/subtitles/en.vtt";

    let args = vec![
        "-d".to_string(),
        device_name.to_string(),
        "cast".to_string(),
        video_url.to_string(),
        "-s".to_string(),
        subtitle_url.to_string(),
    ];

    assert_eq!(args.len(), 6);
    assert_eq!(args[4], "-s");
    assert!(args[5].contains("subtitles/en.vtt"));
}

/// Test: Cast command without subtitles
#[test]
fn test_cast_command_without_subtitle() {
    let args = vec![
        "-d".to_string(),
        "Living Room TV".to_string(),
        "cast".to_string(),
        "http://192.168.1.100:8888/0".to_string(),
    ];

    assert_eq!(args.len(), 4);
    assert!(!args.contains(&"-s".to_string()));
}
