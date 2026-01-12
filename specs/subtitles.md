# Subtitles Specification

## Overview
Subtitle search, download, and casting integration.

## Sources

### OpenSubtitles.com API
Primary source. REST API with good coverage.

Base URL: `https://api.opensubtitles.com/api/v1`
Auth: API key in header

### Stremio Subtitle Addons (Alternative)
- `opensubtitles-v3` addon
- Query same as other Stremio addons

## API Endpoints

### Search Subtitles
```
GET /subtitles
?imdb_id={imdb_id}
&languages=en,es
&type=movie|episode
&season_number=1
&episode_number=1
```

### Download Subtitle
```
POST /download
{
  "file_id": 123456
}
```
Returns download link (rate limited).

## Data Models

### SubtitleResult
```rust
struct SubtitleResult {
    id: String,
    file_id: u64,
    language: String,       // "en", "es", etc.
    language_name: String,  // "English", "Spanish"
    release: String,        // "The.Batman.2022.1080p.BluRay"
    fps: Option<f32>,       // Frame rate
    format: SubFormat,      // SRT, WebVTT
    downloads: u32,         // Popularity indicator
    from_trusted: bool,     // Verified uploader
    hearing_impaired: bool, // SDH subtitles
    ai_translated: bool,    // Machine translated
}
```

### SubFormat
```rust
enum SubFormat {
    Srt,
    WebVtt,
    Sub,
    Ass,
}
```

### SubtitleFile
```rust
struct SubtitleFile {
    id: String,
    language: String,
    path: PathBuf,          // Local cache path
    format: SubFormat,
}
```

## Client Interface

```rust
impl SubtitleClient {
    /// Search subtitles for movie
    async fn search_movie(
        &self, 
        imdb_id: &str, 
        languages: &[&str]
    ) -> Result<Vec<SubtitleResult>>;
    
    /// Search subtitles for TV episode
    async fn search_episode(
        &self,
        imdb_id: &str,
        season: u8,
        episode: u8,
        languages: &[&str]
    ) -> Result<Vec<SubtitleResult>>;
    
    /// Download subtitle to cache
    async fn download(&self, file_id: u64) -> Result<SubtitleFile>;
    
    /// Get cached subtitle if exists
    fn get_cached(&self, id: &str) -> Option<SubtitleFile>;
}
```

## Subtitle Cache

Location: `~/.cache/streamtui/subtitles/`

Structure:
```
subtitles/
├── tt1877830/
│   ├── en_12345.srt
│   └── es_12346.srt
└── tt0903747_1_1/
    └── en_78901.srt
```

## Format Conversion

Chromecast requires **WebVTT** format.

### SRT to WebVTT
```rust
fn srt_to_webvtt(srt: &str) -> String {
    // 1. Add "WEBVTT\n\n" header
    // 2. Replace ',' with '.' in timestamps
    // 3. Remove style tags if present
}
```

## Chromecast Subtitle Integration

When casting with subtitles:
1. Download subtitle file
2. Convert to WebVTT if needed
3. Serve via local HTTP (same as torrent stream)
4. Pass subtitle track URL to Chromecast

```bash
# catt supports subtitles
catt -d "Living Room TV" cast <video_url> -s <subtitle_url>
```

Subtitle URL format:
```
http://192.168.1.100:8889/subtitles/en.vtt
```

## CLI Commands

### streamtui subtitles <imdb_id> [options]
Search for subtitles.

```bash
$ streamtui subtitles tt1877830 --lang en,es
```

Output (JSON):
```json
{
  "subtitles": [
    {
      "id": "12345",
      "language": "en",
      "release": "The.Batman.2022.1080p.BluRay",
      "downloads": 50000,
      "trusted": true,
      "hearing_impaired": false
    }
  ]
}
```

### Cast with subtitles
```bash
$ streamtui cast tt1877830 \
    --device "Living Room TV" \
    --subtitle en \
    --subtitle-id 12345   # optional: specific subtitle
```

## TUI Integration

### Subtitle Selection Screen
```
╔═══════════════════════════════════════════════════════════════╗
║  📝 SUBTITLES - The Batman (2022)                             ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  🌐 English                                                   ║
║  ▸ [✓] The.Batman.2022.1080p.BluRay          50k ⬇  Trusted  ║
║    [ ] The.Batman.2022.2160p.WEB-DL          12k ⬇           ║
║    [ ] The.Batman.2022.HDCAM (AI)            2k ⬇   ⚠️       ║
║                                                               ║
║  🌐 Español                                                   ║
║    [ ] The.Batman.2022.1080p.Spanish         8k ⬇   Trusted  ║
║                                                               ║
║  [c] Cast with selected  [n] No subtitles  [Esc] Back        ║
╚═══════════════════════════════════════════════════════════════╝
```

### Keybindings
| Key | Action |
|-----|--------|
| `↑↓` | Navigate subtitles |
| `Enter` | Select subtitle |
| `c` | Cast with selected subtitle |
| `n` | Cast without subtitles |
| `Esc` | Back |

## Preferred Languages

Config option for default languages:
```toml
[subtitles]
languages = ["en", "es"]
auto_select = true        # Auto-select best match
prefer_trusted = true     # Prefer verified uploaders
prefer_hearing_impaired = false
```

## Tests (TDD)

### test_search_parses_results
- Input: Mock OpenSubtitles response
- Expect: Vec<SubtitleResult> with correct fields

### test_search_filters_language
- Request with languages = ["en"]
- Expect: Only English results

### test_download_caches_file
- Download subtitle
- Expect: File exists in cache directory
- Call again → returns cached, no network request

### test_srt_to_webvtt_conversion
- Input: SRT content with "00:01:23,456" timestamps
- Expect: "00:01:23.456" in output
- Expect: "WEBVTT" header

### test_srt_to_webvtt_preserves_content
- Input: SRT with dialogue
- Expect: Same dialogue in WebVTT

### test_subtitle_url_generation
- lan_ip = 192.168.1.100, port = 8889, lang = "en"
- Expect: "http://192.168.1.100:8889/subtitles/en.vtt"

### test_cast_command_with_subtitle
- Mock catt call
- Expect: `-s` flag with subtitle URL

### test_handles_no_subtitles
- Search returns empty
- Expect: Empty vec, no error
- UI shows "No subtitles found"

### test_rate_limit_handling
- OpenSubtitles returns 429
- Expect: Retry with backoff
- Expect: Clear error after max retries

### test_language_priority
- Results: [es_trusted, en_untrusted, en_trusted]
- Config: languages = ["en", "es"], prefer_trusted = true
- Auto-select → en_trusted
