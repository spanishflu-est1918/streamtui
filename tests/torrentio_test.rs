//! Torrentio Client Tests (TDD)
//!
//! Tests for the Torrentio Stremio addon client.
//! Following specs/addons.md test specifications.

use mockito::Server;
use streamtui::api::torrentio::TorrentioClient;
use streamtui::models::{Quality, StreamSource};

// =============================================================================
// Quality Parsing Tests
// =============================================================================

/// Test: Parse quality from "Torrentio\n4K" name
#[test]
fn test_parse_quality_4k() {
    // Test various 4K representations
    assert_eq!(Quality::from_str_loose("Torrentio\n4K"), Quality::UHD4K);
    assert_eq!(Quality::from_str_loose("4K"), Quality::UHD4K);
    assert_eq!(Quality::from_str_loose("2160p"), Quality::UHD4K);
    assert_eq!(Quality::from_str_loose("UHD"), Quality::UHD4K);
    assert_eq!(
        Quality::from_str_loose("Torrentio\n2160p HDR"),
        Quality::UHD4K
    );
}

/// Test: Parse quality from "Torrentio\n1080p" name
#[test]
fn test_parse_quality_1080p() {
    assert_eq!(
        Quality::from_str_loose("Torrentio\n1080p"),
        Quality::FHD1080p
    );
    assert_eq!(Quality::from_str_loose("1080p"), Quality::FHD1080p);
    assert_eq!(Quality::from_str_loose("FHD"), Quality::FHD1080p);
    assert_eq!(
        Quality::from_str_loose("Torrentio\n1080p BluRay"),
        Quality::FHD1080p
    );
}

// =============================================================================
// Seeds Parsing Tests
// =============================================================================

/// Test: Parse seeds from title containing "👤 142"
#[test]
fn test_parse_seeds() {
    // Standard format: 👤 142
    assert_eq!(
        StreamSource::parse_seeds("The.Batman.2022.2160p.WEB-DL 👤 142"),
        142
    );

    // With space variations
    assert_eq!(StreamSource::parse_seeds("Some.Movie 👤142"), 142);
    assert_eq!(StreamSource::parse_seeds("Title 👤  89"), 89);

    // With k suffix: 👤 1.2k → 1200
    assert_eq!(StreamSource::parse_seeds("Popular.Movie 👤 1.2k"), 1200);
    assert_eq!(StreamSource::parse_seeds("Very.Popular 👤 2k"), 2000);
    assert_eq!(StreamSource::parse_seeds("Mega.Popular 👤 10.5k"), 10500);

    // Fallback text format
    assert_eq!(StreamSource::parse_seeds("Movie seeds: 500"), 500);
    assert_eq!(StreamSource::parse_seeds("Movie seed: 123"), 123);

    // No seeds info → 0
    assert_eq!(StreamSource::parse_seeds("Movie.Without.Seeds.Info"), 0);
}

// =============================================================================
// Size Parsing Tests
// =============================================================================

/// Test: Parse size from title containing "4.2 GB" or "890 MB"
#[test]
fn test_parse_size() {
    // GB format: 4.2 GB = 4.2 * 1024^3 = 4509715660 bytes (approx)
    let size_gb = StreamSource::parse_size("The.Batman.2022.4.2 GB.mkv");
    assert!(size_gb.is_some());
    let bytes = size_gb.unwrap();
    // 4.2 * 1024 * 1024 * 1024 = 4509715660.8
    assert!(
        bytes >= 4_509_000_000 && bytes <= 4_510_000_000,
        "Expected ~4509715660, got {}",
        bytes
    );

    // MB format: 890 MB = 890 * 1024^2 = 933232640 bytes
    let size_mb = StreamSource::parse_size("Small.File.890 MB");
    assert!(size_mb.is_some());
    let bytes_mb = size_mb.unwrap();
    // 890 * 1024 * 1024 = 933232640
    assert!(
        bytes_mb >= 933_000_000 && bytes_mb <= 934_000_000,
        "Expected ~933232640, got {}",
        bytes_mb
    );

    // Case insensitive
    assert!(StreamSource::parse_size("Movie.2.5 gb").is_some());
    assert!(StreamSource::parse_size("Movie.500 mb").is_some());

    // No size → None
    assert!(StreamSource::parse_size("No.Size.Info.Here").is_none());
}

// =============================================================================
// Magnet URL Generation Tests
// =============================================================================

/// Test: Generate magnet URL from info_hash and name
#[test]
fn test_magnet_generation() {
    let source = StreamSource {
        name: "Torrentio\n4K".to_string(),
        title: "The.Batman.2022.2160p".to_string(),
        info_hash: "abc123def456789".to_string(),
        file_idx: Some(0),
        seeds: 100,
        quality: Quality::UHD4K,
        size_bytes: Some(4_500_000_000),
    };

    // Basic magnet generation
    let magnet = source.to_magnet("Movie Name");
    assert_eq!(
        magnet,
        "magnet:?xt=urn:btih:abc123def456789&dn=Movie%20Name"
    );

    // URL encoding special characters
    let magnet_special = source.to_magnet("The Batman (2022)");
    assert!(magnet_special.contains("xt=urn:btih:abc123def456789"));
    assert!(magnet_special.contains("dn=The%20Batman%20%282022%29"));

    // Ampersand encoding
    let magnet_amp = source.to_magnet("Tom & Jerry");
    assert!(magnet_amp.contains("dn=Tom%20%26%20Jerry"));
}

// =============================================================================
// HTTP Request Tests (with mockito)
// =============================================================================

/// Test: Movie streams request forms correct URL
#[tokio::test]
async fn test_movie_streams_request() {
    let mut server = Server::new_async().await;

    // Mock the movie streams endpoint
    let mock = server
        .mock("GET", "/stream/movie/tt1877830.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "streams": [
                {
                    "name": "Torrentio\n4K",
                    "title": "The.Batman.2022.2160p.WEB-DL.DDP5.1.Atmos 👤 89",
                    "infoHash": "abc123def456",
                    "fileIdx": 0
                },
                {
                    "name": "Torrentio\n1080p",
                    "title": "The.Batman.2022.1080p.BluRay 4.2 GB 👤 234",
                    "infoHash": "def789ghi012",
                    "fileIdx": 0
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let streams = client.movie_streams("tt1877830").await.unwrap();

    mock.assert_async().await;

    // Verify parsed results
    assert_eq!(streams.len(), 2);

    // First stream should be 4K
    assert_eq!(streams[0].quality, Quality::UHD4K);
    assert_eq!(streams[0].info_hash, "abc123def456");
    assert_eq!(streams[0].seeds, 89);

    // Second stream should be 1080p with size
    assert_eq!(streams[1].quality, Quality::FHD1080p);
    assert_eq!(streams[1].seeds, 234);
    assert!(streams[1].size_bytes.is_some());
}

/// Test: Series streams request forms correct URL format
#[tokio::test]
async fn test_series_streams_format() {
    let mut server = Server::new_async().await;

    // Mock for Breaking Bad S01E01: /stream/series/tt0903747:1:1.json
    let mock = server
        .mock("GET", "/stream/series/tt0903747:1:1.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "streams": [
                {
                    "name": "Torrentio\n1080p",
                    "title": "Breaking.Bad.S01E01.1080p 👤 456",
                    "infoHash": "series123hash",
                    "fileIdx": 2
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let streams = client.episode_streams("tt0903747", 1, 1).await.unwrap();

    mock.assert_async().await;

    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].info_hash, "series123hash");
    assert_eq!(streams[0].file_idx, Some(2));
}

// =============================================================================
// Sorting Tests
// =============================================================================

/// Test: Results sorted by quality (4K first) then by seeds descending
#[test]
fn test_sorts_by_quality_and_seeds() {
    let mut streams = vec![
        StreamSource {
            name: "720p".to_string(),
            title: "720p release".to_string(),
            info_hash: "hash1".to_string(),
            file_idx: None,
            seeds: 1000, // Many seeds but low quality
            quality: Quality::HD720p,
            size_bytes: None,
        },
        StreamSource {
            name: "4K".to_string(),
            title: "4K release low seeds".to_string(),
            info_hash: "hash2".to_string(),
            file_idx: None,
            seeds: 50, // Few seeds but best quality
            quality: Quality::UHD4K,
            size_bytes: None,
        },
        StreamSource {
            name: "1080p".to_string(),
            title: "1080p release".to_string(),
            info_hash: "hash3".to_string(),
            file_idx: None,
            seeds: 500,
            quality: Quality::FHD1080p,
            size_bytes: None,
        },
        StreamSource {
            name: "4K".to_string(),
            title: "4K release high seeds".to_string(),
            info_hash: "hash4".to_string(),
            file_idx: None,
            seeds: 200, // More seeds, same quality as hash2
            quality: Quality::UHD4K,
            size_bytes: None,
        },
    ];

    // Sort: quality descending, then seeds descending within same quality
    streams.sort_by(|a, b| match b.quality.cmp(&a.quality) {
        std::cmp::Ordering::Equal => b.seeds.cmp(&a.seeds),
        other => other,
    });

    // Verify order: 4K (200 seeds), 4K (50 seeds), 1080p (500), 720p (1000)
    assert_eq!(streams[0].info_hash, "hash4"); // 4K, 200 seeds
    assert_eq!(streams[1].info_hash, "hash2"); // 4K, 50 seeds
    assert_eq!(streams[2].info_hash, "hash3"); // 1080p, 500 seeds
    assert_eq!(streams[3].info_hash, "hash1"); // 720p, 1000 seeds

    // Quality ordering takes precedence
    assert!(streams[0].quality > streams[2].quality);
    assert!(streams[2].quality > streams[3].quality);

    // Within same quality, seeds take precedence
    assert_eq!(streams[0].quality, streams[1].quality);
    assert!(streams[0].seeds > streams[1].seeds);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test: Empty streams array returns empty Vec, no error
#[tokio::test]
async fn test_handles_empty_streams() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/stream/movie/tt0000000.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"streams": []}"#)
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let streams = client.movie_streams("tt0000000").await.unwrap();

    mock.assert_async().await;

    assert!(streams.is_empty());
}

/// Test: Malformed JSON returns ParseError, not panic
#[tokio::test]
async fn test_handles_malformed_response() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/stream/movie/tt9999999.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"streams": not valid json"#)
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let result = client.movie_streams("tt9999999").await;

    mock.assert_async().await;

    // Should return an error, not panic
    assert!(result.is_err());
    let err = result.unwrap_err();
    // Error should indicate JSON parsing issue
    assert!(
        err.to_string().to_lowercase().contains("json")
            || err.to_string().to_lowercase().contains("parse")
            || err.to_string().to_lowercase().contains("expected"),
        "Expected JSON parse error, got: {}",
        err
    );
}

/// Test: Network error is handled gracefully
#[tokio::test]
async fn test_handles_network_error() {
    // Use a non-existent server URL
    let client = TorrentioClient::with_base_url("http://localhost:59999");
    let result = client.movie_streams("tt1234567").await;

    assert!(result.is_err());
}

/// Test: 404 response is handled
#[tokio::test]
async fn test_handles_404_response() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/stream/movie/ttinvalid.json")
        .with_status(404)
        .with_body("Not Found")
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let result = client.movie_streams("ttinvalid").await;

    mock.assert_async().await;

    // Should handle 404 gracefully (either error or empty streams)
    // The implementation decides the behavior
    assert!(result.is_err() || result.unwrap().is_empty());
}

// =============================================================================
// Additional Parsing Edge Cases
// =============================================================================

/// Test: Quality parsing edge cases
#[test]
fn test_quality_edge_cases() {
    // Mixed case
    assert_eq!(Quality::from_str_loose("torrentio\n4k"), Quality::UHD4K);
    assert_eq!(
        Quality::from_str_loose("TORRENTIO\n1080P"),
        Quality::FHD1080p
    );

    // Multiple quality indicators (first match wins based on priority)
    // Contains both 4K and 1080p - 4K should win
    let q = Quality::from_str_loose("4K 1080p version");
    assert_eq!(q, Quality::UHD4K);

    // HDCAM should NOT be detected as HD
    assert_eq!(Quality::from_str_loose("HDCAM"), Quality::Unknown);

    // Empty string
    assert_eq!(Quality::from_str_loose(""), Quality::Unknown);
}

/// Test: Seeds parsing edge cases
#[test]
fn test_seeds_edge_cases() {
    // Zero seeds
    assert_eq!(StreamSource::parse_seeds("Movie 👤 0"), 0);

    // Large numbers
    assert_eq!(StreamSource::parse_seeds("Popular 👤 99999"), 99999);

    // Decimal k values
    assert_eq!(StreamSource::parse_seeds("Title 👤 0.5k"), 500);

    // Multiple seed indicators (first match)
    let seeds = StreamSource::parse_seeds("Title 👤 100 👤 200");
    assert_eq!(seeds, 100);
}

/// Test: Size parsing edge cases  
#[test]
fn test_size_edge_cases() {
    // Decimal MB
    let size = StreamSource::parse_size("File.1.5 MB").unwrap();
    assert!(size > 1_500_000 && size < 1_600_000);

    // Large GB
    let size_large = StreamSource::parse_size("Huge.File.50 GB").unwrap();
    assert!(size_large > 50_000_000_000);

    // Very small
    let size_small = StreamSource::parse_size("Small.1 MB").unwrap();
    assert!(size_small > 1_000_000 && size_small < 1_100_000);
}
