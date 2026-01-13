//! TMDB API client tests
//!
//! Tests search, metadata retrieval, and error handling.

use mockito::{Matcher, Server};
use streamtui::api::TmdbClient;
use streamtui::models::MediaType;

// =============================================================================
// Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_parses_results() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "page": 1,
        "results": [
            {
                "id": 414906,
                "media_type": "movie",
                "title": "The Batman",
                "release_date": "2022-03-01",
                "overview": "Batman ventures into Gotham",
                "poster_path": "/74xTEgt7R36Fpooo50r9T25onhq.jpg",
                "vote_average": 7.8
            },
            {
                "id": 157336,
                "media_type": "movie",
                "title": "Interstellar",
                "release_date": "2014-11-05",
                "overview": "Space epic",
                "poster_path": "/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
                "vote_average": 8.4
            },
            {
                "id": 1396,
                "media_type": "tv",
                "name": "Breaking Bad",
                "first_air_date": "2008-01-20",
                "overview": "A chemistry teacher",
                "poster_path": "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
                "vote_average": 9.5
            }
        ],
        "total_results": 3,
        "total_pages": 1
    }"#;

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("query".into(), "batman".into()),
            Matcher::UrlEncoded("page".into(), "1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let results = client.search("batman").await.unwrap();

    mock.assert_async().await;

    assert_eq!(results.len(), 3);

    // Check first movie
    assert_eq!(results[0].id, 414906);
    assert_eq!(results[0].media_type, MediaType::Movie);
    assert_eq!(results[0].title, "The Batman");

    // Check TV show (name vs title)
    assert_eq!(results[2].id, 1396);
    assert_eq!(results[2].media_type, MediaType::Tv);
    assert_eq!(results[2].title, "Breaking Bad");
}

#[tokio::test]
async fn test_search_extracts_year() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "page": 1,
        "results": [
            {
                "id": 1,
                "media_type": "movie",
                "title": "Movie With Date",
                "release_date": "2022-03-04",
                "overview": "",
                "poster_path": null,
                "vote_average": 5.0
            },
            {
                "id": 2,
                "media_type": "tv",
                "name": "TV With Date",
                "first_air_date": "2019-11-12",
                "overview": "",
                "poster_path": null,
                "vote_average": 6.0
            },
            {
                "id": 3,
                "media_type": "movie",
                "title": "Movie No Date",
                "release_date": null,
                "overview": "",
                "poster_path": null,
                "vote_average": 4.0
            },
            {
                "id": 4,
                "media_type": "tv",
                "name": "TV Empty Date",
                "first_air_date": "",
                "overview": "",
                "poster_path": null,
                "vote_average": 3.0
            }
        ],
        "total_results": 4,
        "total_pages": 1
    }"#;

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let results = client.search("test").await.unwrap();

    mock.assert_async().await;

    assert_eq!(results[0].year, Some(2022));
    assert_eq!(results[1].year, Some(2019));
    assert_eq!(results[2].year, None);
    assert_eq!(results[3].year, None);
}

#[tokio::test]
async fn test_search_filters_person_results() {
    let mut server = Server::new_async().await;

    // TMDB multi-search also returns 'person' results, we should filter them out
    let mock_response = r#"{
        "page": 1,
        "results": [
            {
                "id": 1,
                "media_type": "movie",
                "title": "Some Movie",
                "release_date": "2020-01-01",
                "overview": "",
                "poster_path": null,
                "vote_average": 5.0
            },
            {
                "id": 999,
                "media_type": "person",
                "name": "Some Actor",
                "known_for_department": "Acting"
            },
            {
                "id": 2,
                "media_type": "tv",
                "name": "Some Show",
                "first_air_date": "2021-05-15",
                "overview": "",
                "poster_path": null,
                "vote_average": 7.0
            }
        ],
        "total_results": 3,
        "total_pages": 1
    }"#;

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let results = client.search("test").await.unwrap();

    mock.assert_async().await;

    // Should only have movie and tv, not person
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].media_type, MediaType::Movie);
    assert_eq!(results[1].media_type, MediaType::Tv);
}

// =============================================================================
// Trending Tests
// =============================================================================

#[tokio::test]
async fn test_trending_returns_results() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "page": 1,
        "results": [
            {
                "id": 100,
                "media_type": "movie",
                "title": "Trending Movie",
                "release_date": "2024-01-15",
                "overview": "Hot new movie",
                "poster_path": "/path.jpg",
                "vote_average": 8.0
            },
            {
                "id": 200,
                "media_type": "tv",
                "name": "Trending Show",
                "first_air_date": "2024-02-20",
                "overview": "Popular series",
                "poster_path": "/path2.jpg",
                "vote_average": 8.5
            }
        ],
        "total_results": 2,
        "total_pages": 1
    }"#;

    let mock = server
        .mock("GET", "/trending/all/week")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let results = client.trending().await.unwrap();

    mock.assert_async().await;

    assert!(!results.is_empty());
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Trending Movie");
    assert_eq!(results[1].title, "Trending Show");
}

// =============================================================================
// Movie Detail Tests
// =============================================================================

#[tokio::test]
async fn test_movie_detail_gets_imdb() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": 414906,
        "imdb_id": "tt1877830",
        "title": "The Batman",
        "release_date": "2022-03-01",
        "runtime": 176,
        "genres": [
            {"id": 80, "name": "Crime"},
            {"id": 9648, "name": "Mystery"},
            {"id": 53, "name": "Thriller"}
        ],
        "overview": "Batman ventures into Gotham City's underworld",
        "vote_average": 7.8,
        "poster_path": "/74xTEgt7R36Fpooo50r9T25onhq.jpg",
        "backdrop_path": "/b0PlSFdDwbyK0cf5RxwDpaOJQvQ.jpg",
        "external_ids": {
            "imdb_id": "tt1877830"
        }
    }"#;

    let mock = server
        .mock("GET", "/movie/414906")
        .match_query(Matcher::UrlEncoded(
            "append_to_response".into(),
            "external_ids".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let detail = client.movie_detail(414906).await.unwrap();

    mock.assert_async().await;

    assert_eq!(detail.id, 414906);
    assert_eq!(detail.imdb_id, "tt1877830");
    assert_eq!(detail.title, "The Batman");
    assert_eq!(detail.year, 2022);
    assert_eq!(detail.runtime, 176);
    assert!(detail.genres.contains(&"Crime".to_string()));
    assert!((detail.vote_average - 7.8).abs() < 0.01);
}

#[tokio::test]
async fn test_movie_detail_handles_missing_imdb() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": 12345,
        "imdb_id": null,
        "title": "Some Movie",
        "release_date": "2023-06-15",
        "runtime": 120,
        "genres": [],
        "overview": "A movie without IMDB",
        "vote_average": 5.0,
        "poster_path": null,
        "backdrop_path": null
    }"#;

    let mock = server
        .mock("GET", "/movie/12345")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let detail = client.movie_detail(12345).await.unwrap();

    mock.assert_async().await;

    // Should have empty string for missing IMDB
    assert!(detail.imdb_id.is_empty());
}

// =============================================================================
// TV Detail Tests
// =============================================================================

#[tokio::test]
async fn test_tv_detail_gets_seasons() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": 1396,
        "name": "Breaking Bad",
        "first_air_date": "2008-01-20",
        "seasons": [
            {"season_number": 0, "episode_count": 9, "name": "Specials", "air_date": null},
            {"season_number": 1, "episode_count": 7, "name": "Season 1", "air_date": "2008-01-20"},
            {"season_number": 2, "episode_count": 13, "name": "Season 2", "air_date": "2009-03-08"},
            {"season_number": 3, "episode_count": 13, "name": "Season 3", "air_date": "2010-03-21"},
            {"season_number": 4, "episode_count": 13, "name": "Season 4", "air_date": "2011-07-17"},
            {"season_number": 5, "episode_count": 16, "name": "Season 5", "air_date": "2012-07-15"}
        ],
        "genres": [
            {"id": 18, "name": "Drama"},
            {"id": 80, "name": "Crime"}
        ],
        "overview": "A chemistry teacher diagnosed with cancer",
        "vote_average": 9.5,
        "poster_path": "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
        "backdrop_path": "/tsRy63Mu5cu8etL1X7ZLyf7pLTE.jpg",
        "external_ids": {
            "imdb_id": "tt0903747"
        }
    }"#;

    let mock = server
        .mock("GET", "/tv/1396")
        .match_query(Matcher::UrlEncoded(
            "append_to_response".into(),
            "external_ids".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let detail = client.tv_detail(1396).await.unwrap();

    mock.assert_async().await;

    assert_eq!(detail.id, 1396);
    assert_eq!(detail.imdb_id, "tt0903747");
    assert_eq!(detail.name, "Breaking Bad");
    assert_eq!(detail.year, 2008);
    // 5 regular seasons (excluding specials with season_number 0)
    assert_eq!(detail.seasons.len(), 5);
    assert!(detail.genres.contains(&"Drama".to_string()));
}

// =============================================================================
// TV Season Tests
// =============================================================================

#[tokio::test]
async fn test_tv_season_gets_episodes() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": 3572,
        "season_number": 1,
        "episodes": [
            {
                "episode_number": 1,
                "name": "Pilot",
                "overview": "Walter White joins forces with Jesse",
                "runtime": 58,
                "air_date": "2008-01-20"
            },
            {
                "episode_number": 2,
                "name": "Cat's in the Bag...",
                "overview": "Walt and Jesse clean up",
                "runtime": 48,
                "air_date": "2008-01-27"
            },
            {
                "episode_number": 3,
                "name": "...And the Bag's in the River",
                "overview": "Walt struggles with a decision",
                "runtime": 48,
                "air_date": "2008-02-10"
            },
            {
                "episode_number": 4,
                "name": "Cancer Man",
                "overview": "Walt tells his family about his cancer",
                "runtime": 48,
                "air_date": "2008-02-17"
            },
            {
                "episode_number": 5,
                "name": "Gray Matter",
                "overview": "Walt's former colleagues offer help",
                "runtime": 48,
                "air_date": "2008-02-24"
            },
            {
                "episode_number": 6,
                "name": "Crazy Handful of Nothin'",
                "overview": "Walt takes action against Tuco",
                "runtime": 48,
                "air_date": "2008-03-02"
            },
            {
                "episode_number": 7,
                "name": "A No-Rough-Stuff-Type Deal",
                "overview": "Walt and Jesse cook for Tuco",
                "runtime": 48,
                "air_date": "2008-03-09"
            }
        ]
    }"#;

    let mock = server
        .mock("GET", "/tv/1396/season/1")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let episodes = client.tv_season(1396, 1).await.unwrap();

    mock.assert_async().await;

    assert_eq!(episodes.len(), 7);
    assert_eq!(episodes[0].episode, 1);
    assert_eq!(episodes[0].name, "Pilot");
    assert_eq!(episodes[0].season, 1);
    assert_eq!(episodes[0].runtime, Some(58));
    assert_eq!(episodes[6].episode, 7);
    assert_eq!(episodes[6].name, "A No-Rough-Stuff-Type Deal");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_handles_rate_limit() {
    let mut server = Server::new_async().await;

    // First request returns 429, second succeeds
    let mock_429 = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "1")
        .expect(1)
        .create_async()
        .await;

    let mock_200 = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page": 1, "results": [], "total_results": 0, "total_pages": 0}"#)
        .expect(1)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let result = client.search("test").await;

    // Should succeed after retry
    assert!(result.is_ok());
    mock_429.assert_async().await;
    mock_200.assert_async().await;
}

#[tokio::test]
async fn test_handles_not_found() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/movie/99999999")
        .match_query(Matcher::Any)
        .with_status(404)
        .with_body(r#"{"success": false, "status_code": 34, "status_message": "The resource could not be found."}"#)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let result = client.movie_detail(99999999).await;

    mock.assert_async().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Should be a NotFound error, not a panic
    assert!(
        err.to_string().to_lowercase().contains("not found")
            || err.to_string().contains("404")
            || err
                .downcast_ref::<streamtui::api::tmdb::TmdbError>()
                .is_some()
    );
}

#[tokio::test]
async fn test_handles_server_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/trending/all/week")
        .match_query(Matcher::Any)
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let result = client.trending().await;

    mock.assert_async().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handles_invalid_json() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let client = TmdbClient::with_base_url("test_key", server.url());
    let result = client.search("test").await;

    mock.assert_async().await;

    assert!(result.is_err());
}

// =============================================================================
// Authentication Tests
// =============================================================================

#[tokio::test]
async fn test_sends_bearer_token() {
    let mut server = Server::new_async().await;

    // Use a long token (64+ chars) to trigger Bearer auth instead of query param auth
    let long_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlRlc3QifQ";

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::Any)
        .match_header("Authorization", format!("Bearer {}", long_token).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page": 1, "results": [], "total_results": 0, "total_pages": 0}"#)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url(long_token, server.url());
    let _ = client.search("test").await;

    mock.assert_async().await;
}

#[tokio::test]
async fn test_sends_legacy_api_key() {
    let mut server = Server::new_async().await;

    // Short keys (< 64 chars) are sent as query params, not Bearer tokens
    let short_key = "abc123def456";

    let mock = server
        .mock("GET", "/search/multi")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("api_key".into(), short_key.into()),
            Matcher::UrlEncoded("query".into(), "test".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page": 1, "results": [], "total_results": 0, "total_pages": 0}"#)
        .create_async()
        .await;

    let client = TmdbClient::with_base_url(short_key, server.url());
    let _ = client.search("test").await;

    mock.assert_async().await;
}
