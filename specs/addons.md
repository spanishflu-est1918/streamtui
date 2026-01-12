# Stremio Addons Specification

## Overview
Client for Stremio addons, primarily Torrentio for stream sources.

## Torrentio

Base URL: `https://torrentio.strem.fun`

### Manifest
```
GET /manifest.json
```
Returns addon capabilities and configuration.

### Movie Streams
```
GET /stream/movie/{imdb_id}.json
```
Returns available torrent streams for a movie.

### Series Streams
```
GET /stream/series/{imdb_id}:{season}:{episode}.json
```
Returns available torrent streams for a TV episode.

## Response Format

```json
{
  "streams": [
    {
      "name": "Torrentio\n4K",
      "title": "The.Batman.2022.2160p.WEB-DL.DDP5.1.Atmos.HDR.H.265\n👤 89",
      "infoHash": "abc123...",
      "fileIdx": 0,
      "behaviorHints": {
        "bingeGroup": "torrentio|4K"
      }
    }
  ]
}
```

## Data Models

### StreamSource
```rust
struct StreamSource {
    name: String,           // "Torrentio\n4K" -> quality
    title: String,          // Full release name + seeds
    info_hash: String,      // Torrent hash
    file_idx: Option<u32>,  // File index in torrent
    seeds: u32,             // Parsed from title
    quality: Quality,       // 4K, 1080p, 720p, etc.
    size_bytes: Option<u64>,// Parsed if available
}
```

### Quality (enum)
```rust
enum Quality {
    UHD4K,
    FHD1080p,
    HD720p,
    SD480p,
    Unknown,
}
```

## Parsing Logic

### Quality from name
- Contains "4K" or "2160p" → UHD4K
- Contains "1080p" → FHD1080p
- Contains "720p" → HD720p
- Contains "480p" → SD480p
- Else → Unknown

### Seeds from title
- Regex: `👤\s*(\d+)` or `seeds:\s*(\d+)`
- Default to 0 if not found

### Size from title
- Regex: `(\d+(?:\.\d+)?)\s*(GB|MB)`
- Convert to bytes

## Client Interface

```rust
impl TorrentioClient {
    async fn movie_streams(&self, imdb_id: &str) -> Result<Vec<StreamSource>>;
    async fn series_streams(&self, imdb_id: &str, season: u8, episode: u8) -> Result<Vec<StreamSource>>;
}
```

## Magnet URL Generation

```rust
fn to_magnet(source: &StreamSource, name: &str) -> String {
    format!(
        "magnet:?xt=urn:btih:{}&dn={}",
        source.info_hash,
        urlencoding::encode(name)
    )
}
```

## Tests (TDD)

### test_parse_quality_4k
- Input: name = "Torrentio\n4K"
- Expect: Quality::UHD4K

### test_parse_quality_1080p
- Input: name = "Torrentio\n1080p"
- Expect: Quality::FHD1080p

### test_parse_seeds
- Input: title contains "👤 142"
- Expect: seeds = 142
- Input: title contains "👤 1.2k"
- Expect: seeds = 1200

### test_parse_size
- Input: title contains "4.2 GB"
- Expect: size_bytes = 4509715660
- Input: title contains "890 MB"
- Expect: size_bytes = 933232640

### test_movie_streams_request
- Mock HTTP call to /stream/movie/tt1877830.json
- Expect: Correct URL formed
- Expect: Parsed StreamSource vec

### test_series_streams_format
- Input: imdb_id=tt0903747, season=1, episode=1
- Expect: URL = /stream/series/tt0903747:1:1.json

### test_magnet_generation
- Input: info_hash = "abc123", name = "Movie Name"
- Expect: "magnet:?xt=urn:btih:abc123&dn=Movie%20Name"

### test_sorts_by_quality_and_seeds
- Given mixed quality results
- Sort: 4K first, then by seeds descending within quality
- Expect: Best quality + most seeds at top

### test_handles_empty_streams
- Input: {"streams": []}
- Expect: Empty Vec, no error

### test_handles_malformed_response
- Input: Invalid JSON
- Expect: ParseError, not panic
