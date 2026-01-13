# Subtitles Specification

## Overview
Subtitle search, download, and casting integration using Stremio's free public endpoint.

## Source

### Stremio OpenSubtitles Addon
Free public endpoint - no API key required.

Base URL: `https://opensubtitles-v3.strem.io`

## API Endpoints

### Search Movie Subtitles
```
GET /subtitles/movie/{imdb_id}.json
```

Example: `/subtitles/movie/tt1877830.json`

### Search Episode Subtitles
```
GET /subtitles/series/{imdb_id}:{season}:{episode}.json
```

Example: `/subtitles/series/tt0903747:1:1.json`

### Response Format
```json
{
  "subtitles": [
    {
      "id": "55419",
      "url": "https://subs5.strem.io/en/download/file/70235",
      "lang": "eng",
      "SubEncoding": "CP1252"
    }
  ],
  "cacheMaxAge": 14400
}
```

Note: Language codes are 3-letter ISO codes (eng, spa, fre, etc.)

## Data Models

### SubtitleResult
```rust
struct SubtitleResult {
    id: String,
    url: String,             // Direct download URL
    language: String,        // "eng", "spa", etc.
    language_name: String,   // "English", "Spanish"
    release: String,         // Release name if available
    fps: Option<f32>,        // Frame rate
    format: SubFormat,       // SRT, WebVTT
    downloads: u32,          // Popularity indicator
    from_trusted: bool,      // Stremio subs default to trusted
    hearing_impaired: bool,  // SDH subtitles
    ai_translated: bool,     // Machine translated
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
    /// Create client (no API key needed)
    fn new() -> Self;

    /// Search subtitles for movie
    async fn search(
        &self,
        imdb_id: &str,
        language: Option<&str>
    ) -> Result<Vec<SubtitleResult>>;

    /// Search subtitles for TV episode
    async fn search_episode(
        &self,
        imdb_id: &str,
        season: u16,
        episode: u16,
        language: Option<&str>
    ) -> Result<Vec<SubtitleResult>>;

    /// Download subtitle to cache (direct URL)
    async fn download(&self, url: &str) -> Result<SubtitleFile>;

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
1. Download subtitle file from Stremio URL
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
$ streamtui subtitles tt1877830 --lang eng
```

Output (JSON):
```json
{
  "subtitles": [
    {
      "id": "12345",
      "language": "eng",
      "url": "https://subs5.strem.io/en/download/file/12345",
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
    --subtitle eng
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
languages = ["eng", "spa"]  # 3-letter codes
auto_select = true          # Auto-select best match
prefer_trusted = true       # Prefer verified uploaders
prefer_hearing_impaired = false
```

## Tests (TDD)

### test_search_parses_results
- Input: Mock Stremio response
- Expect: Vec<SubtitleResult> with correct fields

### test_search_filters_language
- Request with language = "eng"
- Expect: Only English results

### test_download_caches_file
- Download subtitle from URL
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
- Search returns empty `{"subtitles": []}`
- Expect: Empty vec, no error
- UI shows "No subtitles found"

### test_language_priority
- Results: [spa_trusted, eng_untrusted, eng_trusted]
- Config: languages = ["eng", "spa"], prefer_trusted = true
- Auto-select → eng_trusted

### test_imdb_id_normalization
- Input: "1877830" (without tt prefix)
- Expect: Request uses "tt1877830"
