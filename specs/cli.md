# CLI Specification

## Overview
Command-line interface for automation and Claude Code integration.
StreamTUI runs in two modes: Interactive TUI (default) or CLI commands.

## Design Philosophy
**"Design for robots, not just humans"**
- Every action possible in TUI should be scriptable via CLI
- Output parseable JSON for automation
- Exit codes for success/failure
- No interactive prompts in CLI mode

## Commands

### streamtui (no args)
Launch interactive TUI.

### streamtui search <query>
Search for movies/TV shows.

```bash
$ streamtui search "the batman"
```

Output (JSON):
```json
{
  "results": [
    {
      "id": "tt1877830",
      "type": "movie",
      "title": "The Batman",
      "year": 2022,
      "rating": 7.8
    }
  ]
}
```

### streamtui trending
Get trending content.

```bash
$ streamtui trending --limit 10
```

Output: Same format as search.

### streamtui info <imdb_id>
Get details for a movie/show.

```bash
$ streamtui info tt1877830
```

Output (JSON):
```json
{
  "id": "tt1877830",
  "type": "movie",
  "title": "The Batman",
  "year": 2022,
  "runtime": 176,
  "genres": ["Action", "Crime", "Drama"],
  "rating": 7.8,
  "overview": "..."
}
```

### streamtui streams <imdb_id> [--season N --episode N]
Get available streams/sources.

```bash
$ streamtui streams tt1877830
$ streamtui streams tt0903747 --season 1 --episode 1
```

Output (JSON):
```json
{
  "streams": [
    {
      "quality": "1080p",
      "size": "4.2 GB",
      "seeds": 142,
      "title": "The.Batman.2022.1080p.BluRay.x264",
      "hash": "abc123..."
    }
  ]
}
```

### streamtui subtitles <imdb_id> [options]
Search for subtitles.

```bash
$ streamtui subtitles tt1877830 --lang en,es
$ streamtui subtitles tt0903747 --season 1 --episode 1 --lang en
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

Options:
- `--lang <codes>` — Comma-separated language codes (default: en)
- `--season <n>` — Season number (TV only)
- `--episode <n>` — Episode number (TV only)

### streamtui devices
List available Chromecast devices.

```bash
$ streamtui devices
```

Output (JSON):
```json
{
  "devices": [
    {"name": "Living Room TV", "address": "192.168.1.50"},
    {"name": "Bedroom TV", "address": "192.168.1.51"}
  ]
}
```

### streamtui cast <imdb_id> [options]
Start casting content.

```bash
$ streamtui cast tt1877830 \
    --device "Living Room TV" \
    --quality 1080p \
    --season 1 --episode 1  # for TV shows
```

Output (JSON):
```json
{
  "status": "casting",
  "device": "Living Room TV",
  "title": "The Batman",
  "stream_url": "http://192.168.1.100:8888/0"
}
```

Options:
- `--device <name>` — Target device (required, or uses default)
- `--quality <q>` — Preferred quality (4k, 1080p, 720p)
- `--season <n>` — Season number (TV only)
- `--episode <n>` — Episode number (TV only)
- `--index <n>` — Stream index from `streams` output
- `--subtitle <lang>` — Subtitle language code (e.g., "en")
- `--subtitle-id <id>` — Specific subtitle ID from `subtitles` output
- `--no-subtitle` — Explicitly disable subtitles

### streamtui status
Get current playback status.

```bash
$ streamtui status
```

Output (JSON):
```json
{
  "state": "playing",
  "title": "The Batman",
  "device": "Living Room TV",
  "position": 2345,
  "duration": 10560,
  "progress": 0.22
}
```

States: `idle`, `buffering`, `playing`, `paused`, `stopped`, `error`

### streamtui play / pause / stop
Playback controls.

```bash
$ streamtui pause
$ streamtui play
$ streamtui stop
```

Output: `{"status": "ok"}`

### streamtui seek <seconds>
Seek to position.

```bash
$ streamtui seek 3600  # 1 hour
```

### streamtui volume <0-100>
Set volume.

```bash
$ streamtui volume 50
```

## Global Options

| Flag | Description |
|------|-------------|
| `--json` | Force JSON output (default for non-TTY) |
| `--device <name>` | Default device for commands |
| `--quiet` | Suppress non-essential output |
| `--config <path>` | Config file path |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Network error |
| 4 | Device not found |
| 5 | No streams available |
| 6 | Cast failed |

## Config File

Location: `~/.config/streamtui/config.toml`

```toml
[cast]
default_device = "Living Room TV"
preferred_quality = "1080p"

[api]
tmdb_key = "your_api_key"
```

## Daemon Mode (Future)

For persistent streaming sessions:
```bash
streamtui daemon start   # Start background daemon
streamtui daemon stop    # Stop daemon
streamtui daemon status  # Check if running
```

Daemon keeps torrent alive while casting.

## Tests (TDD)

### test_search_json_output
- Run: `streamtui search "batman" --json`
- Expect: Valid JSON with results array
- Expect: Exit code 0

### test_search_no_results
- Run: `streamtui search "xyznonexistent123"`
- Expect: Empty results array
- Expect: Exit code 0 (not an error)

### test_devices_lists_chromecasts
- Run: `streamtui devices`
- Expect: JSON with devices array
- Expect: Each device has name and address

### test_cast_requires_device
- Run: `streamtui cast tt1877830` (no device, no default)
- Expect: Error message
- Expect: Exit code 4

### test_cast_validates_imdb_id
- Run: `streamtui cast "not_an_id"`
- Expect: Error "Invalid IMDB ID"
- Expect: Exit code 2

### test_status_when_idle
- Run: `streamtui status` (nothing playing)
- Expect: `{"state": "idle"}`
- Expect: Exit code 0

### test_streams_for_movie
- Run: `streamtui streams tt1877830`
- Expect: JSON with streams array
- Expect: Each stream has quality, size, seeds

### test_streams_for_tv_episode
- Run: `streamtui streams tt0903747 --season 1 --episode 1`
- Expect: Streams for Breaking Bad S01E01

### test_exit_codes
- Various failure scenarios
- Expect: Correct exit code for each
