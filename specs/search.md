# Search Specification

## Overview
TMDB (The Movie Database) integration for search, metadata, and trending content.

## API
Base URL: `https://api.themoviedb.org/3`
Auth: Bearer token in header

## Endpoints Used

### Search Multi
```
GET /search/multi?query={query}&page=1
```
Returns movies and TV shows matching query.

### Trending
```
GET /trending/all/week
```
Returns trending movies and TV shows.

### Movie Details
```
GET /movie/{id}?append_to_response=external_ids
```
Returns movie details including IMDB ID.

### TV Details
```
GET /tv/{id}?append_to_response=external_ids
```
Returns TV show details including IMDB ID.

### TV Season
```
GET /tv/{id}/season/{season_number}
```
Returns episodes for a season.

## Data Models

### SearchResult
```rust
struct SearchResult {
    id: u64,
    media_type: MediaType,  // Movie | Tv
    title: String,          // "title" for movie, "name" for tv
    year: Option<u16>,      // extracted from release_date/first_air_date
    overview: String,
    poster_path: Option<String>,
    vote_average: f32,
}
```

### MovieDetail
```rust
struct MovieDetail {
    id: u64,
    imdb_id: String,
    title: String,
    year: u16,
    runtime: u32,           // minutes
    genres: Vec<String>,
    overview: String,
    vote_average: f32,
}
```

### TvDetail
```rust
struct TvDetail {
    id: u64,
    imdb_id: String,
    name: String,
    year: u16,
    seasons: Vec<SeasonSummary>,
    genres: Vec<String>,
    overview: String,
    vote_average: f32,
}
```

### Episode
```rust
struct Episode {
    season: u8,
    episode: u8,
    name: String,
    overview: String,
    runtime: Option<u32>,
    imdb_id: Option<String>,  // for episode-level streams
}
```

## Client Interface

```rust
impl TmdbClient {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
    async fn trending(&self) -> Result<Vec<SearchResult>>;
    async fn movie_detail(&self, id: u64) -> Result<MovieDetail>;
    async fn tv_detail(&self, id: u64) -> Result<TvDetail>;
    async fn tv_season(&self, id: u64, season: u8) -> Result<Vec<Episode>>;
}
```

## Environment
- `TMDB_API_KEY` — API key (v4 bearer token)

## Tests (TDD)

### test_search_parses_results
- Input: Mock JSON response with 3 results (2 movies, 1 TV)
- Expect: 3 SearchResult items with correct media_type
- Expect: Titles extracted correctly (title vs name)

### test_search_extracts_year
- Input: release_date "2022-03-04"
- Expect: year = 2022
- Input: first_air_date "2019-11-12"
- Expect: year = 2019
- Input: null date
- Expect: year = None

### test_trending_returns_results
- Integration test (can be mocked)
- Expect: Non-empty Vec<SearchResult>

### test_movie_detail_gets_imdb
- Input: Movie ID 414906 (The Batman)
- Expect: imdb_id = "tt1877830"

### test_tv_detail_gets_seasons
- Input: TV ID 1396 (Breaking Bad)
- Expect: 5 seasons in seasons vec

### test_tv_season_gets_episodes
- Input: TV ID 1396, Season 1
- Expect: 7 episodes with names

### test_handles_rate_limit
- On 429 response, retry with backoff
- Max 3 retries before error

### test_handles_not_found
- On 404, return specific NotFound error
- Don't panic or crash
