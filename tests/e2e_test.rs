//! End-to-end flow tests for StreamTUI
//!
//! Tests the complete user journey from search to casting,
//! including both TUI state transitions and CLI command flows.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mockito::{Matcher, Server};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use streamtui::api::{TmdbClient, TorrentioClient};
use streamtui::app::{App, AppState, DetailState, InputMode, LoadingState, SourcesState};
use streamtui::models::{
    CastDevice, CastState, MediaType, MovieDetail, PlaybackStatus, Quality, SearchResult,
    SeasonSummary, StreamSource, SubFormat, TvDetail,
};
use streamtui::stream::SubtitleClient;

// =============================================================================
// Mock Response Fixtures
// =============================================================================

fn mock_tmdb_search_response() -> &'static str {
    r#"{
        "page": 1,
        "results": [
            {
                "id": 414906,
                "media_type": "movie",
                "title": "The Batman",
                "release_date": "2022-03-01",
                "overview": "When a sadistic serial killer begins murdering key political figures in Gotham, Batman is forced to investigate the city's hidden corruption.",
                "poster_path": "/74xTEgt7R36Fpooo50r9T25onhq.jpg",
                "vote_average": 7.8
            },
            {
                "id": 1396,
                "media_type": "tv",
                "name": "Breaking Bad",
                "first_air_date": "2008-01-20",
                "overview": "A chemistry teacher diagnosed with lung cancer teams up with a former student to cook and sell methamphetamine.",
                "poster_path": "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
                "vote_average": 9.5
            }
        ],
        "total_results": 2,
        "total_pages": 1
    }"#
}

fn mock_tmdb_movie_detail_response() -> &'static str {
    r#"{
        "id": 414906,
        "imdb_id": "tt1877830",
        "title": "The Batman",
        "release_date": "2022-03-01",
        "runtime": 176,
        "genres": [{"id": 80, "name": "Crime"}, {"id": 9648, "name": "Mystery"}, {"id": 53, "name": "Thriller"}],
        "overview": "When a sadistic serial killer begins murdering key political figures in Gotham, Batman is forced to investigate the city's hidden corruption.",
        "vote_average": 7.8,
        "poster_path": "/74xTEgt7R36Fpooo50r9T25onhq.jpg",
        "backdrop_path": "/5P8SmMzSNYikXpxil6BYzJ16611.jpg"
    }"#
}

fn mock_tmdb_tv_detail_response() -> &'static str {
    r#"{
        "id": 1396,
        "name": "Breaking Bad",
        "first_air_date": "2008-01-20",
        "external_ids": {"imdb_id": "tt0903747"},
        "seasons": [
            {"season_number": 1, "episode_count": 7, "name": "Season 1", "air_date": "2008-01-20"},
            {"season_number": 2, "episode_count": 13, "name": "Season 2", "air_date": "2009-03-08"}
        ],
        "genres": [{"id": 18, "name": "Drama"}, {"id": 80, "name": "Crime"}],
        "overview": "A chemistry teacher diagnosed with lung cancer teams up with a former student to cook and sell methamphetamine.",
        "vote_average": 9.5,
        "poster_path": "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
        "backdrop_path": "/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg"
    }"#
}

fn mock_torrentio_streams_response() -> &'static str {
    r#"{
        "streams": [
            {
                "name": "Torrentio\n4K",
                "title": "The.Batman.2022.2160p.WEB-DL.DDP5.1.Atmos.HDR.x265 👤 2.1k\n💾 15.2 GB",
                "infoHash": "abc123def456789012345678901234567890abcd",
                "fileIdx": 0
            },
            {
                "name": "Torrentio\n1080p",
                "title": "The.Batman.2022.1080p.BluRay.x264-SPARKS 👤 1.5k\n💾 8.5 GB",
                "infoHash": "def456abc789012345678901234567890abcdef",
                "fileIdx": 0
            },
            {
                "name": "Torrentio\n720p",
                "title": "The.Batman.2022.720p.BluRay.x264-GECKOS 👤 890\n💾 4.2 GB",
                "infoHash": "789abc123def456789012345678901234567890",
                "fileIdx": 0
            }
        ]
    }"#
}

fn mock_stremio_subtitles_response() -> &'static str {
    r#"{
        "subtitles": [
            {
                "id": "12345",
                "url": "https://subs.strem.io/download/12345",
                "lang": "eng"
            },
            {
                "id": "12346",
                "url": "https://subs.strem.io/download/12346",
                "lang": "eng"
            }
        ]
    }"#
}

// =============================================================================
// TUI Flow Tests: Search -> Detail -> Sources
// =============================================================================

#[tokio::test]
async fn test_tui_search_to_detail_flow() {
    // Test the TUI state machine: Home -> Search -> Detail

    let mut app = App::new();

    // Initial state: Home
    assert_eq!(app.state, AppState::Home);
    assert_eq!(app.input_mode, InputMode::Normal);

    // Press '/' to focus search - should navigate to Search and enter editing mode
    app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Search);
    assert_eq!(app.input_mode, InputMode::Editing);

    // Type search query
    for c in "batman".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
    }
    assert_eq!(app.search.query, "batman");
    assert_eq!(app.search.cursor, 6);

    // Press Enter to submit search (exits editing mode)
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(app.input_mode, InputMode::Normal);

    // Simulate search results being populated
    let results = vec![
        SearchResult {
            id: 414906,
            media_type: MediaType::Movie,
            title: "The Batman".to_string(),
            year: Some(2022),
            overview: "Batman investigates...".to_string(),
            poster_path: Some("/poster.jpg".to_string()),
            vote_average: 7.8,
        },
        SearchResult {
            id: 1396,
            media_type: MediaType::Tv,
            title: "Breaking Bad".to_string(),
            year: Some(2008),
            overview: "Chemistry teacher...".to_string(),
            poster_path: None,
            vote_average: 9.5,
        },
    ];
    app.search.set_results(results);

    // Verify results are set
    assert_eq!(app.search.results.len(), 2);
    assert_eq!(app.search.list.selected, 0);

    // Navigate down to Breaking Bad
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    assert_eq!(app.search.list.selected, 1);

    // Navigate back up
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
    assert_eq!(app.search.list.selected, 0);

    // Verify selected result
    let selected = app.search.selected_result().unwrap();
    assert_eq!(selected.title, "The Batman");
    assert_eq!(selected.media_type, MediaType::Movie);

    // Navigate to detail view
    app.navigate(AppState::Detail);
    assert_eq!(app.state, AppState::Detail);
    assert_eq!(app.nav_stack.len(), 2); // Home -> Search -> Detail

    // Set up movie detail state
    let movie_detail = MovieDetail {
        id: 414906,
        imdb_id: "tt1877830".to_string(),
        title: "The Batman".to_string(),
        year: 2022,
        runtime: 176,
        genres: vec!["Crime".to_string(), "Mystery".to_string()],
        overview: "Batman investigates...".to_string(),
        vote_average: 7.8,
        poster_path: Some("/poster.jpg".to_string()),
        backdrop_path: None,
    };
    app.detail = Some(DetailState::movie(movie_detail));

    // Verify detail state
    if let Some(DetailState::Movie { detail, .. }) = &app.detail {
        assert_eq!(detail.title, "The Batman");
        assert_eq!(detail.imdb_id, "tt1877830");
        assert_eq!(detail.runtime, 176);
    } else {
        panic!("Expected movie detail state");
    }

    // Press 'c' or Enter to go to sources
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Sources);

    // Go back to detail
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Detail);

    // Go back to search
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Search);

    // Go back to home
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Home);
}

#[tokio::test]
async fn test_tui_detail_to_sources_flow() {
    // Test: Detail -> Sources -> Playing

    let mut app = App::new();
    app.state = AppState::Detail;

    // Set up movie detail
    let movie = MovieDetail {
        id: 414906,
        imdb_id: "tt1877830".to_string(),
        title: "The Batman".to_string(),
        year: 2022,
        runtime: 176,
        genres: vec!["Crime".to_string()],
        overview: "Dark knight...".to_string(),
        vote_average: 7.8,
        poster_path: None,
        backdrop_path: None,
    };
    app.detail = Some(DetailState::movie(movie));
    app.sources = SourcesState::new("The Batman".to_string());

    // Navigate to sources
    app.navigate(AppState::Sources);
    assert_eq!(app.state, AppState::Sources);

    // Simulate sources loading
    let sources = vec![
        StreamSource {
            name: "Torrentio\n4K".to_string(),
            title: "The.Batman.2022.2160p 👤 2.1k\n💾 15.2 GB".to_string(),
            info_hash: "abc123def456789012345678901234567890abcd".to_string(),
            file_idx: Some(0),
            seeds: 2100,
            quality: Quality::UHD4K,
            size_bytes: Some(16_323_727_360),
        },
        StreamSource {
            name: "Torrentio\n1080p".to_string(),
            title: "The.Batman.2022.1080p 👤 1.5k\n💾 8.5 GB".to_string(),
            info_hash: "def456abc789012345678901234567890abcdef".to_string(),
            file_idx: Some(0),
            seeds: 1500,
            quality: Quality::FHD1080p,
            size_bytes: Some(9_126_805_504),
        },
        StreamSource {
            name: "Torrentio\n720p".to_string(),
            title: "The.Batman.2022.720p 👤 890\n💾 4.2 GB".to_string(),
            info_hash: "789abc123def456789012345678901234567890".to_string(),
            file_idx: Some(0),
            seeds: 890,
            quality: Quality::HD720p,
            size_bytes: Some(4_509_715_660),
        },
    ];
    app.sources.set_sources(sources);

    // Verify sources are loaded
    assert_eq!(app.sources.sources.len(), 3);
    assert_eq!(app.sources.loading, LoadingState::Idle);

    // Select first source (4K)
    assert_eq!(app.sources.list.selected, 0);
    let selected = app.sources.selected_source().unwrap();
    assert_eq!(selected.quality, Quality::UHD4K);

    // Navigate down to 1080p
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    assert_eq!(app.sources.list.selected, 1);
    let selected = app.sources.selected_source().unwrap();
    assert_eq!(selected.quality, Quality::FHD1080p);

    // Quick select with number key '3' (720p)
    app.handle_key(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty()));
    assert_eq!(app.sources.list.selected, 2);
    let selected = app.sources.selected_source().unwrap();
    assert_eq!(selected.quality, Quality::HD720p);

    // Press 'u' to go to subtitles
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Subtitles);

    // Go back to sources
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Sources);

    // Set up a Chromecast device (required for playback)
    app.cast_devices = vec![CastDevice {
        id: "192.168.1.50".to_string(),
        name: "Living Room TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    }];
    app.selected_device = Some(0);

    // Press Enter to start playing
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(app.state, AppState::Playing);
}

#[tokio::test]
async fn test_tui_tv_show_flow() {
    // Test TV show flow with seasons/episodes

    let mut app = App::new();
    app.state = AppState::Detail;

    // Set up TV detail
    let tv = TvDetail {
        id: 1396,
        imdb_id: "tt0903747".to_string(),
        name: "Breaking Bad".to_string(),
        year: 2008,
        seasons: vec![
            SeasonSummary {
                season_number: 1,
                episode_count: 7,
                name: Some("Season 1".to_string()),
                air_date: Some("2008-01-20".to_string()),
            },
            SeasonSummary {
                season_number: 2,
                episode_count: 13,
                name: Some("Season 2".to_string()),
                air_date: Some("2009-03-08".to_string()),
            },
        ],
        genres: vec!["Drama".to_string(), "Crime".to_string()],
        overview: "Chemistry teacher...".to_string(),
        vote_average: 9.5,
        poster_path: None,
        backdrop_path: None,
    };
    app.detail = Some(DetailState::tv(tv));

    // Verify TV detail state
    if let Some(DetailState::Tv {
        detail,
        season_list,
        ..
    }) = &app.detail
    {
        assert_eq!(detail.name, "Breaking Bad");
        assert_eq!(detail.seasons.len(), 2);
        assert_eq!(season_list.len, 2);
    } else {
        panic!("Expected TV detail state");
    }

    // Navigate seasons with j/k
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()));
    if let Some(DetailState::Tv { season_list, .. }) = &app.detail {
        assert_eq!(season_list.selected, 1);
    }

    app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()));
    if let Some(DetailState::Tv { season_list, .. }) = &app.detail {
        assert_eq!(season_list.selected, 0);
    }
}

#[tokio::test]
async fn test_tui_playing_controls() {
    // Test playback controls in Playing state

    let mut app = App::new();
    app.state = AppState::Playing;
    app.playing.title = "The Batman".to_string();
    app.playing.device = Some(CastDevice {
        id: "192.168.1.50".to_string(),
        name: "Living Room TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: Some("Chromecast Ultra".to_string()),
    });
    app.playing.playback = Some(PlaybackStatus {
        state: CastState::Playing,
        position: Duration::from_secs(300),
        duration: Duration::from_secs(10560), // 2h56m
        volume: 0.8,
        title: Some("The Batman".to_string()),
    });

    // Verify initial state
    assert!(matches!(
        app.playing.playback.as_ref().unwrap().state,
        CastState::Playing
    ));

    // Space toggles pause
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
    assert!(matches!(
        app.playing.playback.as_ref().unwrap().state,
        CastState::Paused
    ));

    // Space again resumes
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
    assert!(matches!(
        app.playing.playback.as_ref().unwrap().state,
        CastState::Playing
    ));

    // Up increases volume
    let initial_vol = app.playing.playback.as_ref().unwrap().volume;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
    let new_vol = app.playing.playback.as_ref().unwrap().volume;
    assert!(new_vol > initial_vol);

    // Down decreases volume
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    let final_vol = app.playing.playback.as_ref().unwrap().volume;
    assert!((final_vol - initial_vol).abs() < 0.01);

    // 's' stops playback
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
    assert!(matches!(
        app.playing.playback.as_ref().unwrap().state,
        CastState::Stopped
    ));
}

// =============================================================================
// API Integration Tests (Mocked)
// =============================================================================

#[tokio::test]
async fn test_api_search_to_detail_to_streams() {
    // Test full API flow: TMDB search -> TMDB detail -> Torrentio streams

    let mut tmdb_server = Server::new_async().await;
    let mut torrentio_server = Server::new_async().await;

    // Mock TMDB search
    let search_mock = tmdb_server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "query".into(),
            "batman".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_tmdb_search_response())
        .create_async()
        .await;

    // Mock TMDB movie detail (client appends ?append_to_response=external_ids)
    let detail_mock = tmdb_server
        .mock("GET", "/movie/414906")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_tmdb_movie_detail_response())
        .create_async()
        .await;

    // Mock Torrentio streams
    let streams_mock = torrentio_server
        .mock("GET", "/stream/movie/tt1877830.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_torrentio_streams_response())
        .create_async()
        .await;

    // Step 1: Search
    let tmdb = TmdbClient::with_base_url("test_key", tmdb_server.url());
    let results = tmdb.search("batman").await.unwrap();

    assert!(!results.is_empty());
    let movie = results
        .iter()
        .find(|r| r.media_type == MediaType::Movie)
        .unwrap();
    assert_eq!(movie.title, "The Batman");
    assert_eq!(movie.id, 414906);

    search_mock.assert_async().await;

    // Step 2: Get detail
    let detail = tmdb.movie_detail(414906).await.unwrap();
    assert_eq!(detail.imdb_id, "tt1877830");
    assert_eq!(detail.title, "The Batman");
    assert_eq!(detail.runtime, 176);

    detail_mock.assert_async().await;

    // Step 3: Get streams
    let torrentio = TorrentioClient::with_base_url(torrentio_server.url());
    let streams = torrentio.movie_streams(&detail.imdb_id).await.unwrap();

    assert_eq!(streams.len(), 3);

    // Verify stream qualities and sorting
    let qualities: Vec<_> = streams.iter().map(|s| s.quality).collect();
    assert!(qualities.contains(&Quality::UHD4K));
    assert!(qualities.contains(&Quality::FHD1080p));
    assert!(qualities.contains(&Quality::HD720p));

    // Verify best stream (4K)
    let best = &streams[0];
    assert_eq!(best.quality, Quality::UHD4K);
    assert!(best.seeds > 2000);

    streams_mock.assert_async().await;
}

#[tokio::test]
async fn test_api_tv_show_episode_flow() {
    // Test TV show flow: TMDB detail -> Torrentio episode streams

    let mut tmdb_server = Server::new_async().await;
    let mut torrentio_server = Server::new_async().await;

    // Mock TMDB TV detail (client appends ?append_to_response=external_ids)
    let detail_mock = tmdb_server
        .mock("GET", "/tv/1396")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_tmdb_tv_detail_response())
        .create_async()
        .await;

    // Mock Torrentio episode streams
    let streams_mock = torrentio_server
        .mock("GET", "/stream/series/tt0903747:1:1.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "streams": [
                {
                    "name": "Torrentio\n1080p",
                    "title": "Breaking.Bad.S01E01.1080p 👤 500\n💾 1.2 GB",
                    "infoHash": "bb1111111111111111111111111111111111bbbb",
                    "fileIdx": 0
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    // Get TV detail
    let tmdb = TmdbClient::with_base_url("test_key", tmdb_server.url());
    let detail = tmdb.tv_detail(1396).await.unwrap();

    assert_eq!(detail.name, "Breaking Bad");
    assert_eq!(detail.imdb_id, "tt0903747");
    assert_eq!(detail.seasons.len(), 2);
    assert_eq!(detail.seasons[0].episode_count, 7);

    detail_mock.assert_async().await;

    // Get episode streams
    let torrentio = TorrentioClient::with_base_url(torrentio_server.url());
    let streams = torrentio
        .episode_streams(&detail.imdb_id, 1, 1)
        .await
        .unwrap();

    assert!(!streams.is_empty());
    assert_eq!(streams[0].quality, Quality::FHD1080p);

    streams_mock.assert_async().await;
}

#[tokio::test]
async fn test_api_search_with_subtitles() {
    // Test search flow with subtitle lookup (Stremio endpoint)

    let mut tmdb_server = Server::new_async().await;
    let mut stremio_server = Server::new_async().await;

    // Mock TMDB search
    let _search_mock = tmdb_server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "query".into(),
            "batman".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_tmdb_search_response())
        .create_async()
        .await;

    // Mock Stremio subtitles endpoint
    let subs_mock = stremio_server
        .mock("GET", "/subtitles/movie/tt1877830.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_stremio_subtitles_response())
        .create_async()
        .await;

    // Search
    let tmdb = TmdbClient::with_base_url("test_key", tmdb_server.url());
    let results = tmdb.search("batman").await.unwrap();
    assert!(!results.is_empty());

    // Get subtitles (Stremio uses 3-letter codes like "eng")
    let sub_client = SubtitleClient::with_base_url(stremio_server.url());
    let subtitles = sub_client.search("tt1877830", Some("eng")).await.unwrap();

    assert_eq!(subtitles.len(), 2);

    // Verify subtitle attributes
    let best_sub = &subtitles[0];
    assert_eq!(best_sub.language, "eng");
    assert!(best_sub.from_trusted); // Stremio subs default to trusted
    assert_eq!(best_sub.format, SubFormat::Srt);

    subs_mock.assert_async().await;
}

// =============================================================================
// CLI Command Flow Tests (Mocked)
// =============================================================================

#[tokio::test]
async fn test_cli_search_streams_flow() {
    // Test CLI: search -> streams (the typical automation flow)

    let mut tmdb_server = Server::new_async().await;
    let mut torrentio_server = Server::new_async().await;

    // Mock TMDB search (UrlEncoded matcher expects decoded value)
    let _search_mock = tmdb_server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "query".into(),
            "the batman".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_tmdb_search_response())
        .create_async()
        .await;

    // Mock Torrentio streams
    let _streams_mock = torrentio_server
        .mock("GET", "/stream/movie/tt1877830.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_torrentio_streams_response())
        .create_async()
        .await;

    // Simulate CLI flow
    let tmdb = TmdbClient::with_base_url("test_key", tmdb_server.url());
    let results = tmdb.search("the batman").await.unwrap();

    // Filter movies only
    let movies: Vec<_> = results
        .iter()
        .filter(|r| r.media_type == MediaType::Movie)
        .collect();
    assert!(!movies.is_empty());

    // Get IMDB ID from first movie result (would need detail call for real IMDB ID)
    let movie_id = movies[0].id;
    assert_eq!(movie_id, 414906);

    // Get streams
    let torrentio = TorrentioClient::with_base_url(torrentio_server.url());
    let streams = torrentio.movie_streams("tt1877830").await.unwrap();

    // Apply quality filter (1080p minimum)
    let filtered: Vec<_> = streams
        .iter()
        .filter(|s| s.quality.rank() >= Quality::FHD1080p.rank())
        .collect();

    assert_eq!(filtered.len(), 2); // 4K and 1080p

    // Sort by seeds
    let mut sorted = filtered;
    sorted.sort_by(|a, b| b.seeds.cmp(&a.seeds));

    // Best stream should be 4K with most seeds
    assert_eq!(sorted[0].quality, Quality::UHD4K);
}

#[tokio::test]
async fn test_cli_cast_flow_simulation() {
    // Test the cast flow simulation (without actual casting)

    let mut torrentio_server = Server::new_async().await;

    // Mock Torrentio streams
    let _streams_mock = torrentio_server
        .mock("GET", "/stream/movie/tt1877830.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_torrentio_streams_response())
        .create_async()
        .await;

    // Simulate cast command flow
    let imdb_id = "tt1877830";
    let target_device = "Living Room TV";
    let quality_preference = Quality::FHD1080p;

    // Get streams
    let torrentio = TorrentioClient::with_base_url(torrentio_server.url());
    let mut streams = torrentio.movie_streams(imdb_id).await.unwrap();

    assert!(!streams.is_empty());

    // Select stream by quality preference (closest match)
    streams.sort_by(|a, b| {
        let a_diff = (a.quality.rank() as i8 - quality_preference.rank() as i8).abs();
        let b_diff = (b.quality.rank() as i8 - quality_preference.rank() as i8).abs();
        a_diff.cmp(&b_diff).then_with(|| b.seeds.cmp(&a.seeds))
    });

    let selected = &streams[0];
    assert_eq!(selected.quality, Quality::FHD1080p);

    // Generate magnet
    let magnet = selected.to_magnet("The Batman");
    assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
    assert!(magnet.contains(&selected.info_hash));

    // Verify we'd cast to correct device
    assert_eq!(target_device, "Living Room TV");
}

// =============================================================================
// Full E2E Flow Test (Mocked)
// =============================================================================

#[tokio::test]
async fn test_full_e2e_flow_mocked() {
    // Complete end-to-end test: Search -> Detail -> Sources -> Cast simulation

    let mut tmdb_server = Server::new_async().await;
    let mut torrentio_server = Server::new_async().await;
    let mut stremio_sub_server = Server::new_async().await;

    // Setup all mocks (UrlEncoded matcher expects decoded value)
    let _search = tmdb_server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "query".into(),
            "the batman".into(),
        )]))
        .with_status(200)
        .with_body(mock_tmdb_search_response())
        .create_async()
        .await;

    let _detail = tmdb_server
        .mock("GET", "/movie/414906")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(mock_tmdb_movie_detail_response())
        .create_async()
        .await;

    let _streams = torrentio_server
        .mock("GET", "/stream/movie/tt1877830.json")
        .with_status(200)
        .with_body(mock_torrentio_streams_response())
        .create_async()
        .await;

    let _subs = stremio_sub_server
        .mock("GET", "/subtitles/movie/tt1877830.json")
        .with_status(200)
        .with_body(mock_stremio_subtitles_response())
        .create_async()
        .await;

    // Initialize clients
    let tmdb = TmdbClient::with_base_url("test_key", tmdb_server.url());
    let torrentio = TorrentioClient::with_base_url(torrentio_server.url());
    let sub_client = SubtitleClient::with_base_url(stremio_sub_server.url());

    // === STEP 1: Search ===
    let results = tmdb.search("the batman").await.unwrap();
    assert!(!results.is_empty());

    let movie_result = results
        .iter()
        .find(|r| r.media_type == MediaType::Movie && r.title.contains("Batman"))
        .expect("Should find The Batman");

    // === STEP 2: Get Detail ===
    let detail = tmdb.movie_detail(movie_result.id).await.unwrap();
    assert_eq!(detail.title, "The Batman");
    assert_eq!(detail.imdb_id, "tt1877830");

    // Verify runtime (2h56m = 176 minutes)
    let hours = detail.runtime / 60;
    let mins = detail.runtime % 60;
    assert_eq!(hours, 2);
    assert_eq!(mins, 56);

    // === STEP 3: Get Streams ===
    let streams = torrentio.movie_streams(&detail.imdb_id).await.unwrap();
    assert!(!streams.is_empty());

    // Find best quality stream
    let mut sorted_streams = streams.clone();
    sorted_streams.sort_by(|a, b| {
        b.quality
            .rank()
            .cmp(&a.quality.rank())
            .then_with(|| b.seeds.cmp(&a.seeds))
    });

    let best_stream = &sorted_streams[0];
    assert_eq!(best_stream.quality, Quality::UHD4K);

    // === STEP 4: Get Subtitles ===
    let subtitles = sub_client
        .search(&detail.imdb_id, Some("eng"))
        .await
        .unwrap();
    assert!(!subtitles.is_empty());

    // Select best subtitle (most downloads, trusted, not AI)
    let mut sorted_subs = subtitles.clone();
    sorted_subs.sort_by(|a, b| b.trust_score().cmp(&a.trust_score()));

    let best_sub = &sorted_subs[0];
    assert!(best_sub.from_trusted);

    // === STEP 5: Prepare Cast (simulation) ===
    let magnet = best_stream.to_magnet(&detail.title);
    assert!(magnet.contains("magnet:?xt=urn:btih:"));

    // Simulate device selection
    let devices = vec![CastDevice {
        id: "192.168.1.50".to_string(),
        name: "Living Room TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: Some("Chromecast Ultra".to_string()),
    }];
    let selected_device = &devices[0];
    assert_eq!(selected_device.name, "Living Room TV");

    // Verify all data is ready for casting
    assert!(!magnet.is_empty());
    assert!(!detail.imdb_id.is_empty());
    assert!(!selected_device.name.is_empty());

    // Success! All components ready for cast
    println!("E2E Test Success:");
    println!("  Movie: {} ({})", detail.title, detail.year);
    println!(
        "  Stream: {} ({} seeds)",
        best_stream.quality, best_stream.seeds
    );
    println!(
        "  Subtitle: {} ({} downloads)",
        best_sub.language, best_sub.downloads
    );
    println!("  Device: {}", selected_device.name);
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_api_error_handling() {
    // Test graceful handling of API errors

    let mut server = Server::new_async().await;

    // Mock 404 response
    let _mock = server
        .mock("GET", "/search/multi")
        .with_status(404)
        .with_body(r#"{"success": false, "status_message": "Not found"}"#)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let result = client.search("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_no_streams_handling() {
    // Test handling of content with no available streams

    let mut server = Server::new_async().await;

    let _mock = server
        .mock("GET", "/stream/movie/tt9999999.json")
        .with_status(200)
        .with_body(r#"{"streams": []}"#)
        .create_async()
        .await;

    let client = TorrentioClient::with_base_url(server.url());
    let streams = client.movie_streams("tt9999999").await.unwrap();

    assert!(streams.is_empty());
}

#[tokio::test]
async fn test_no_subtitles_handling() {
    // Test handling when no subtitles are available (Stremio endpoint)

    let mut server = Server::new_async().await;

    let _mock = server
        .mock("GET", "/subtitles/movie/tt9999999.json")
        .with_status(200)
        .with_body(r#"{"subtitles": []}"#)
        .create_async()
        .await;

    let client = SubtitleClient::with_base_url(server.url());
    let subs = client.search("tt9999999", Some("eng")).await.unwrap();

    assert!(subs.is_empty());
}

// =============================================================================
// Stress / Edge Case Tests
// =============================================================================

#[test]
fn test_app_rapid_navigation() {
    // Test rapid state transitions don't corrupt state

    let mut app = App::new();

    // Rapid navigation sequence
    for _ in 0..100 {
        app.navigate(AppState::Search);
        app.navigate(AppState::Detail);
        app.navigate(AppState::Sources);
        app.navigate(AppState::Playing);

        while app.back() {}

        assert_eq!(app.state, AppState::Home);
        assert!(app.nav_stack.is_empty());
    }

    // App should still be in valid state
    assert!(app.running);
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_search_with_special_characters() {
    // Test search input handling with special characters

    let mut app = App::new();
    app.input_mode = InputMode::Editing;

    let special_chars = "Batman: Arkham Knight (2015) — Director's Cut!";
    for c in special_chars.chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
    }

    assert_eq!(app.search.query, special_chars);
    assert_eq!(app.search.cursor, special_chars.len());
}

#[test]
fn test_empty_results_handling() {
    // Test UI behavior with empty results

    let mut app = App::new();
    app.state = AppState::Search;
    app.search.set_results(vec![]);

    // Navigation on empty list should be safe
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
    app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty()));
    app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty()));

    assert_eq!(app.search.list.selected, 0);
    assert!(app.search.selected_result().is_none());
}

#[test]
fn test_volume_bounds() {
    // Test volume stays within bounds

    let mut app = App::new();
    app.state = AppState::Playing;
    app.playing.playback = Some(PlaybackStatus {
        state: CastState::Playing,
        position: Duration::ZERO,
        duration: Duration::from_secs(3600),
        volume: 1.0,
        title: None,
    });

    // Try to increase beyond 100%
    for _ in 0..20 {
        app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
    }
    assert!(app.playing.playback.as_ref().unwrap().volume <= 1.0);

    // Set to 0% and try to decrease further
    app.playing.playback.as_mut().unwrap().volume = 0.0;
    for _ in 0..20 {
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    }
    assert!(app.playing.playback.as_ref().unwrap().volume >= 0.0);
}

// =============================================================================
// Playback Flow Tests
// =============================================================================

#[test]
fn test_devices_loaded_message_updates_state() {
    use streamtui::app::AppMessage;

    let mut app = App::new();
    // VLC is initialized as default device
    assert_eq!(app.cast_devices.len(), 1);
    assert_eq!(app.cast_devices[0].name, "VLC (Local)");
    assert_eq!(app.selected_device, Some(0));

    // Simulate receiving devices from discovery
    let devices = vec![
        CastDevice {
            id: "192.168.1.50".to_string(),
            name: "Living Room TV".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: Some("Chromecast Ultra".to_string()),
        },
        CastDevice {
            id: "192.168.1.51".to_string(),
            name: "Bedroom".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 51)),
            port: 8009,
            model: None,
        },
    ];

    app.handle_message(AppMessage::DevicesLoaded(devices.clone()));

    // VLC (Local) is injected as first device, then discovered devices
    assert_eq!(app.cast_devices.len(), 3);
    assert_eq!(app.cast_devices[0].name, "VLC (Local)");
    assert_eq!(app.cast_devices[1].name, "Living Room TV");
    assert_eq!(app.selected_device, Some(0)); // Auto-selects first device (VLC)
}

#[test]
fn test_devices_loaded_empty_still_has_vlc() {
    use streamtui::app::AppMessage;

    let mut app = App::new();
    app.handle_message(AppMessage::DevicesLoaded(vec![]));

    // VLC (Local) is always available even with no Chromecasts discovered
    assert_eq!(app.cast_devices.len(), 1);
    assert_eq!(app.cast_devices[0].name, "VLC (Local)");
    assert_eq!(app.selected_device, Some(0)); // Auto-selects VLC
}

#[test]
fn test_playback_started_updates_torrent_session() {
    use streamtui::app::AppMessage;
    use streamtui::models::{TorrentSession, TorrentState};

    let mut app = App::new();
    app.state = AppState::Playing;

    // Set up a torrent session in Starting state
    let session = TorrentSession::new("magnet:?xt=urn:btih:abc".to_string(), Some(0));
    assert_eq!(session.state, TorrentState::Starting);
    assert!(session.stream_url.is_none());

    app.playing.torrent = Some(session);

    // Simulate playback started
    app.handle_message(AppMessage::PlaybackStarted {
        stream_url: "http://localhost:8888/0".to_string(),
    });

    let session = app.playing.torrent.as_ref().unwrap();
    assert_eq!(session.state, TorrentState::Streaming);
    assert_eq!(
        session.stream_url,
        Some("http://localhost:8888/0".to_string())
    );
}

#[test]
fn test_playback_stopped_clears_state_and_navigates_back() {
    use streamtui::app::AppMessage;
    use streamtui::models::TorrentSession;

    let mut app = App::new();

    // Set up: navigate to Playing state
    app.navigate(AppState::Sources);
    app.navigate(AppState::Playing);
    assert_eq!(app.state, AppState::Playing);

    // Set up playing state with data
    app.playing.title = "The Batman".to_string();
    app.playing.torrent = Some(TorrentSession::new("magnet:?xt=...".to_string(), None));
    app.playing.device = Some(CastDevice {
        id: "192.168.1.50".to_string(),
        name: "TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    });

    // Simulate playback stopped
    app.handle_message(AppMessage::PlaybackStopped);

    // Should clear playing state
    assert!(app.playing.torrent.is_none());
    assert!(app.playing.device.is_none());
    assert!(app.playing.title.is_empty());

    // Should navigate back
    assert_eq!(app.state, AppState::Sources);
}

#[tokio::test]
async fn test_enter_in_sources_sends_start_playback_command() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();

    // Set up: we're in Sources with a selected source and device
    app.state = AppState::Sources;
    app.sources.title = "The Batman".to_string();
    app.sources.set_sources(vec![StreamSource {
        name: "Torrentio\n1080p".to_string(),
        title: "The.Batman.2022.1080p".to_string(),
        info_hash: "abc123def456789012345678901234567890abcd".to_string(),
        file_idx: Some(0),
        seeds: 1500,
        quality: Quality::FHD1080p,
        size_bytes: Some(8_000_000_000),
    }]);

    // Set up a device
    app.cast_devices = vec![CastDevice {
        id: "192.168.1.50".to_string(),
        name: "Living Room TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    }];
    app.selected_device = Some(0);

    // Press Enter to start playback
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

    // Should navigate to Playing
    assert_eq!(app.state, AppState::Playing);

    // Should have sent StartPlayback command
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    match cmd {
        AppCommand::StartPlayback { magnet, title, device, .. } => {
            assert!(magnet.contains("abc123def456789012345678901234567890abcd"));
            assert_eq!(title, "The Batman");
            assert_eq!(device, "Living Room TV");
        }
        other => panic!("Expected StartPlayback, got {:?}", other),
    }
}

#[tokio::test]
async fn test_enter_in_sources_with_vlc_default_starts_playback() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();

    // VLC is now default device
    assert_eq!(app.selected_device, Some(0));
    assert_eq!(app.cast_devices[0].name, "VLC (Local)");

    // Set up: Sources with source, VLC already selected
    app.state = AppState::Sources;
    app.sources.set_sources(vec![StreamSource {
        name: "Torrentio\n1080p".to_string(),
        title: "Test".to_string(),
        info_hash: "abc123".to_string(),
        file_idx: Some(0),
        seeds: 100,
        quality: Quality::FHD1080p,
        size_bytes: None,
    }]);

    // Press Enter
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

    // Should navigate to Playing and send StartPlayback command
    assert_eq!(app.state, AppState::Playing);

    // Verify command was sent
    let cmd = cmd_rx.try_recv().expect("Should receive command");
    match cmd {
        AppCommand::StartPlayback { device, .. } => {
            assert_eq!(device, "VLC (Local)");
        }
        other => panic!("Expected StartPlayback, got {:?}", other),
    }
}

#[tokio::test]
async fn test_d_key_triggers_device_discovery() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Sources;

    // Press 'd' to discover devices
    app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()));

    // Should have sent DiscoverDevices command
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::DiscoverDevices));
}

// =============================================================================
// Device Selector Tests
// =============================================================================

#[test]
fn test_tab_cycles_through_devices() {
    let mut app = App::new();
    app.state = AppState::Sources;

    // Set up multiple devices
    app.cast_devices = vec![
        CastDevice {
            id: "1".to_string(),
            name: "Living Room".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: None,
        },
        CastDevice {
            id: "2".to_string(),
            name: "Bedroom".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 51)),
            port: 8009,
            model: None,
        },
        CastDevice {
            id: "3".to_string(),
            name: "Kitchen".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 52)),
            port: 8009,
            model: None,
        },
    ];
    app.selected_device = Some(0);

    // Tab should cycle to next device
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));
    assert_eq!(app.selected_device, Some(1));

    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));
    assert_eq!(app.selected_device, Some(2));

    // Should wrap around
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));
    assert_eq!(app.selected_device, Some(0));
}

#[test]
fn test_shift_tab_cycles_backwards() {
    let mut app = App::new();
    app.state = AppState::Sources;

    app.cast_devices = vec![
        CastDevice {
            id: "1".to_string(),
            name: "Living Room".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
            port: 8009,
            model: None,
        },
        CastDevice {
            id: "2".to_string(),
            name: "Bedroom".to_string(),
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 51)),
            port: 8009,
            model: None,
        },
    ];
    app.selected_device = Some(0);

    // Shift+Tab should go backwards (wrap to end)
    app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(app.selected_device, Some(1));

    app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(app.selected_device, Some(0));
}

// =============================================================================
// Playback Control Tests
// =============================================================================

#[tokio::test]
async fn test_space_sends_pause_command() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Playing;
    app.playing.device = Some(CastDevice {
        id: "1".to_string(),
        name: "TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    });

    // Press space to toggle pause
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));

    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::PlaybackControl { action, .. } if action == "play_toggle"));
}

#[tokio::test]
async fn test_s_sends_stop_command() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Playing;
    app.playing.device = Some(CastDevice {
        id: "1".to_string(),
        name: "TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    });

    // Press 's' to stop
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));

    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::StopPlayback));
}

#[tokio::test]
async fn test_arrow_keys_send_volume_commands() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Playing;
    app.playing.device = Some(CastDevice {
        id: "1".to_string(),
        name: "TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    });

    // Press Up for volume up
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::PlaybackControl { action, .. } if action == "volumeup"));

    // Press Down for volume down
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::PlaybackControl { action, .. } if action == "volumedown"));
}

#[tokio::test]
async fn test_left_right_send_seek_commands() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Playing;
    app.playing.device = Some(CastDevice {
        id: "1".to_string(),
        name: "TV".to_string(),
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        port: 8009,
        model: None,
    });

    // Press Left for rewind
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::PlaybackControl { action, .. } if action == "rewind"));

    // Press Right for ffwd
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    assert!(matches!(cmd, AppCommand::PlaybackControl { action, .. } if action == "ffwd"));
}

// =============================================================================
// Auto-fetch Subtitles Tests
// =============================================================================

#[tokio::test]
async fn test_entering_subtitles_triggers_fetch() {
    use streamtui::app::AppCommand;

    let (mut app, mut cmd_rx) = App::with_channels();
    app.state = AppState::Sources;

    // Set up detail with IMDB ID
    app.detail = Some(DetailState::movie(MovieDetail {
        id: 414906,
        imdb_id: "tt1877830".to_string(),
        title: "The Batman".to_string(),
        year: 2022,
        runtime: 176,
        genres: vec![],
        overview: "".to_string(),
        vote_average: 7.8,
        poster_path: None,
        backdrop_path: None,
    }));

    // Press 'u' to go to subtitles
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty()));

    // Should navigate to Subtitles
    assert_eq!(app.state, AppState::Subtitles);

    // Should have sent FetchSubtitles command
    let cmd = cmd_rx.try_recv().expect("Should have sent a command");
    match cmd {
        AppCommand::FetchSubtitles { imdb_id, .. } => {
            assert_eq!(imdb_id, "tt1877830");
        }
        other => panic!("Expected FetchSubtitles, got {:?}", other),
    }
}
