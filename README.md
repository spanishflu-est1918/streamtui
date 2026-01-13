# StreamTUI

<div align="center">

```
╔═══════════════════════════════════════════════════════════════╗
║  ███████╗████████╗██████╗ ███████╗ █████╗ ███╗   ███╗████████╗║
║  ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██╔══██╗████╗ ████║╚══██╔══╝║
║  ███████╗   ██║   ██████╔╝█████╗  ███████║██╔████╔██║   ██║   ║
║  ╚════██║   ██║   ██╔══██╗██╔══╝  ██╔══██║██║╚██╔╝██║   ██║   ║
║  ███████║   ██║   ██║  ██║███████╗██║  ██║██║ ╚═╝ ██║   ██║   ║
║  ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝   ╚═╝   ║
║                        ████████╗██╗   ██╗██╗                   ║
║                        ╚══██╔══╝██║   ██║██║                   ║
║                           ██║   ██║   ██║██║                   ║
║                           ██║   ██║   ██║██║                   ║
║                           ██║   ╚██████╔╝██║                   ║
║                           ╚═╝    ╚═════╝ ╚═╝                   ║
╚═══════════════════════════════════════════════════════════════╝
```

**A neon-soaked terminal interface for streaming to Chromecast**

[![License: MIT](https://img.shields.io/badge/License-MIT-cyan.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

</div>

---

## Overview

StreamTUI is a cyberpunk-themed Terminal User Interface for searching movies and TV shows, selecting stream quality, and casting to your Chromecast — all from the comfort of your terminal.

Built with performance in mind: **zero cold start** (instant launch), **full keyboard navigation**, and **complete CLI automation** for scripting and AI agent integration.

## ✨ Features

### Interactive TUI
- 🎨 **Cyberpunk neon theme** with WCAG-compliant contrast ratios
- ⌨️ **Vim-style navigation** (j/k, /, Esc)
- 🔍 **Real-time search** with trending content
- 📺 **Multi-quality streams** (4K, 1080p, 720p, 480p)
- 🌐 **Subtitle support** with language selection and trust indicators
- 📊 **Live playback status** with progress bar

### CLI Automation
- 🤖 **JSON output** for scripting and automation
- 🔧 **Semantic exit codes** for error handling
- ⚡ **Single-command casting** for quick access
- 🎯 **Claude Code / AI agent friendly** — designed for LLM integration

### Playback Control
- ▶️ Play / ⏸️ Pause / ⏹️ Stop
- ⏩ Seek (absolute, relative, timestamp)
- 🔊 Volume control
- 📱 Device discovery and selection

## 📦 Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-username/streamtui.git
cd streamtui

# Build and install
cargo install --path .
```

### Dependencies

StreamTUI requires these external tools for full functionality:

```bash
# webtorrent-cli - Torrent streaming
npm install -g webtorrent-cli

# catt - Chromecast control
pip install catt
```

#### Verify Installation

```bash
# Check webtorrent
webtorrent --version

# Check catt  
catt --version

# Check streamtui
streamtui --help
```

## 🚀 Usage

### TUI Mode (Interactive)

Simply run without arguments to launch the interactive interface:

```bash
streamtui
```

#### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Start search |
| `↑/↓` or `j/k` | Navigate lists |
| `Page Up/Down` | Navigate by page |
| `Home/End` | Jump to first/last |
| `Enter` | Select item |
| `c` | View sources (from detail view) |
| `u` | Select subtitles |
| `Space` | Play/Pause |
| `←/→` | Seek ±10s |
| `Esc` | Go back |
| `q` | Quit |

### CLI Mode (Automation)

Every TUI action is available as a CLI command with JSON output. Commands support short aliases for faster typing.

#### Command Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `search` | `s` | Search for movies and TV shows |
| `trending` | `tr` | Get trending content |
| `info` | `i` | Get details for a movie or show |
| `streams` | `st` | Get available streams for content |
| `subtitles` | `sub` | Search for subtitles |
| `devices` | `dev` | List available Chromecast devices |
| `cast` | — | Start casting content to a device |
| `cast-magnet` | `cm` | Cast a raw magnet link directly |
| `play-local` | `pl` | Play locally in VLC or mpv |
| `status` | — | Get current playback status |
| `play` | — | Resume playback |
| `pause` | — | Pause playback |
| `stop` | — | Stop playback and disconnect |
| `seek` | — | Seek to a position |
| `volume` | `vol` | Set volume level |

---

#### Search Content

```bash
# Basic search
streamtui search "blade runner"
streamtui s "blade runner"  # alias

# Filter by type
streamtui search "breaking bad" --media-type tv

# Limit results and filter by year
streamtui search "batman" --limit 5 --year-from 2020

# JSON output (automatic when piped)
streamtui search "inception" --json
```

**Options:**
- `--limit, -l <N>` — Maximum results (default: 20)
- `--media-type, -t <movie|tv>` — Filter by type
- `--year-from <YYYY>` — Minimum year
- `--year-to <YYYY>` — Maximum year

---

#### Browse Trending

```bash
# Today's trending
streamtui trending
streamtui tr  # alias

# This week's trending movies
streamtui trending --window week --media-type movie
```

**Options:**
- `--window, -w <day|week>` — Time window (default: day)
- `--limit, -l <N>` — Maximum results (default: 20)
- `--media-type, -t <movie|tv>` — Filter by type

---

#### Get Content Info

```bash
# Get movie/show details
streamtui info tt1877830
streamtui i tt1877830  # alias
```

---

#### Get Streams

```bash
# Movie streams
streamtui streams tt1856101
streamtui st tt1856101  # alias

# TV episode streams  
streamtui streams tt0903747 --season 1 --episode 1

# Filter by quality and sort by seeds
streamtui streams tt1877830 --quality 1080p --sort seeds
```

**Options:**
- `--season, -s <N>` — Season number (TV only)
- `--episode, -e <N>` — Episode number (TV only)
- `--quality, -Q <4k|1080p|720p|480p>` — Filter by minimum quality
- `--limit, -l <N>` — Maximum results (default: 20)
- `--sort <seeds|quality|size>` — Sort criterion (default: seeds)

---

#### Find Subtitles

```bash
# English subtitles
streamtui subtitles tt1856101 --lang en
streamtui sub tt1856101 -l en  # alias

# Multiple languages
streamtui subtitles tt0903747 --lang "en,es,fr" -s 1 -e 1

# Only trusted/verified subtitles
streamtui subtitles tt1877830 --trusted
```

**Options:**
- `--lang, -l <codes>` — Comma-separated language codes (default: en)
- `--season, -s <N>` — Season number (TV only)
- `--episode, -e <N>` — Episode number (TV only)
- `--hearing-impaired` — Only show hearing-impaired subtitles
- `--trusted` — Only show trusted/verified subtitles
- `--limit <N>` — Maximum results (default: 20)

---

#### Discover Devices

```bash
# List available Chromecasts
streamtui devices
streamtui dev  # alias

# Extended scan with longer timeout
streamtui devices --timeout 10

# Force refresh device cache
streamtui devices --refresh
```

**Options:**
- `--timeout, -t <secs>` — Scan timeout (default: 5)
- `--refresh, -r` — Refresh device cache

---

#### Cast Content

```bash
# Cast movie to device
streamtui cast tt1856101 --device "Living Room TV"

# Cast with quality preference
streamtui cast tt1877830 -d "Bedroom TV" -Q 1080p

# Cast TV episode with subtitles
streamtui cast tt0903747 -s 1 -e 1 -d TV --subtitle en

# Cast specific stream index
streamtui cast tt1877830 -d TV --index 0

# Play locally in VLC instead of casting
streamtui cast tt1877830 --vlc
```

**Options:**
- `--device, -d <name>` — Target device (required unless default set)
- `--quality, -Q <4k|1080p|720p|480p>` — Preferred quality
- `--season, -s <N>` — Season number (TV only)
- `--episode, -e <N>` — Episode number (TV only)
- `--index, -i <N>` — Stream index from `streams` output
- `--subtitle <lang>` — Subtitle language code
- `--subtitle-id <id>` — Specific subtitle ID from `subtitles` output
- `--no-subtitle` — Explicitly disable subtitles
- `--start <secs>` — Start position in seconds
- `--vlc` — Play locally in VLC instead of casting

---

#### Cast Magnet Link Directly

```bash
# Cast a magnet link
streamtui cast-magnet "magnet:?xt=urn:btih:..." --device TV
streamtui cm "magnet:?xt=..." -d TV  # alias

# With subtitles
streamtui cast-magnet "magnet:?xt=..." -d TV --subtitle en

# With local subtitle file
streamtui cast-magnet "magnet:?xt=..." -d TV --subtitle-file /path/to/subs.srt
```

**Options:**
- `--device, -d <name>` — Target device
- `--subtitle <lang>` — Search OpenSubtitles for this language
- `--subtitle-file <path>` — Path to local subtitle file (.srt, .vtt)
- `--file-idx, -i <N>` — File index within torrent (default: largest video)
- `--start <secs>` — Start position
- `--vlc` — Play locally in VLC instead

---

#### Play Locally (No Chromecast)

```bash
# Play magnet in VLC
streamtui play-local "magnet:?xt=..."
streamtui pl "magnet:?xt=..."  # alias

# Play in mpv instead
streamtui play-local "magnet:?xt=..." --player mpv

# With subtitles
streamtui play-local "magnet:?xt=..." --subtitle-file /path/to/subs.srt
```

**Options:**
- `--player, -p <vlc|mpv>` — Player to use (default: vlc)
- `--subtitle-file <path>` — Path to local subtitle file
- `--file-idx, -i <N>` — File index within torrent

---

#### Playback Control

```bash
# Check status
streamtui status
streamtui status --json  # For scripting

# Watch mode - continuously update
streamtui status --watch --interval 2

# Play/Pause
streamtui play
streamtui pause

# Seek - multiple formats supported
streamtui seek 3600      # To 1 hour (absolute seconds)
streamtui seek +30       # Forward 30 seconds
streamtui seek -10       # Back 10 seconds
streamtui seek 1:30:00   # To timestamp (HH:MM:SS)
streamtui seek 5:30      # To timestamp (MM:SS)

# Volume
streamtui volume 50      # Set to 50%
streamtui vol +10        # Increase by 10%
streamtui vol -5         # Decrease by 5%

# Stop
streamtui stop
streamtui stop --kill-stream  # Also stop torrent
```

---

### Global Options

These flags work with any command:

```bash
--json, -j        # Force JSON output (default for non-TTY)
--device, -d      # Set default Chromecast device
--quiet, -q       # Suppress non-essential output
--config, -c      # Custom config file path
```

---

## 🤖 Claude Code / AI Agent Integration

StreamTUI is designed for seamless AI agent integration. Every action is scriptable with JSON output and semantic exit codes.

### Search and Cast Workflow

```bash
# Agent searches for content
result=$(streamtui search "the matrix" --json | jq -r '.data[0].imdb_id')

# Agent gets available streams
streamtui streams "$result" --json | jq '.data[] | {quality, seeds, title}'

# Agent casts best quality
streamtui cast "$result" --device "TV" --quality 1080p --json
```

### Status Monitoring

```bash
# Check if something is playing
state=$(streamtui status --json | jq -r '.data.state')
if [ "$state" = "playing" ]; then
    echo "Currently playing!"
fi

# Get playback progress
streamtui status --json | jq '{title: .data.title, progress: (.data.progress * 100 | floor | tostring + "%")}'
```

### Automated Evening Routine

```bash
#!/bin/bash
# Find a trending movie and cast it

movie=$(streamtui trending --media-type movie --json | jq -r '.data[0].imdb_id')
streamtui cast "$movie" --device "Living Room TV" --quality 1080p
```

### Direct Magnet Casting

```bash
# Agent has a magnet link from external source
streamtui cast-magnet "$MAGNET_LINK" -d "TV" --subtitle en
```

### Error Handling Pattern

```bash
#!/bin/bash
# Robust casting with error handling

imdb_id="tt1877830"
device="Living Room TV"

# Try to cast
if streamtui cast "$imdb_id" -d "$device" -Q 1080p --json 2>/dev/null; then
    echo "Casting started successfully"
else
    exit_code=$?
    case $exit_code in
        4) echo "Device '$device' not found" ;;
        5) echo "No streams available for $imdb_id" ;;
        6) echo "Cast failed" ;;
        *) echo "Unknown error: $exit_code" ;;
    esac
    exit $exit_code
fi
```

### Exit Codes

For robust scripting, StreamTUI uses semantic exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Network error |
| 4 | Device not found |
| 5 | No streams available |
| 6 | Cast failed |

### JSON Output Format

All commands output consistent JSON when `--json` is used or when stdout is not a TTY:

```json
// Success
{
  "data": { ... },
  "exit_code": 0
}

// Error
{
  "error": "Error message",
  "exit_code": 4
}
```

---

## ⚙️ Configuration

StreamTUI looks for configuration at `~/.config/streamtui/config.toml`:

```toml
# Default Chromecast device
default_device = "Living Room TV"

# Preferred quality (4k, 1080p, 720p, 480p)
preferred_quality = "1080p"

# Preferred subtitle languages (first match wins)
subtitle_languages = ["en", "es"]

# API keys (optional - uses defaults)
# tmdb_api_key = "your-key"
```

---

## 📸 Screenshots

*Manual screenshots coming soon...*

### TUI Interface (ASCII Preview)

```
┌─────────────────────────────────────────────────────────────┐
│ STREAMTUI │ ⌕ blade runner                                  │
├─────────────────────────────────────────────────────────────┤
│ ⚡ RESULTS (5)                                              │
│                                                             │
│ ▸ Blade Runner 2049 (2017) [MOVIE] ★ 7.5                   │
│   Blade Runner (1982) [MOVIE] ★ 8.1                        │
│   Blade Runner: Black Lotus (2021) [TV] ★ 6.4              │
│   Blade Runner: Black Out 2022 (2017) [MOVIE] ★ 7.4        │
│   Making of Blade Runner (2007) [MOVIE] ★ 7.8              │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ NORMAL │ SEARCH │ 📺 Living Room TV │ q:quit /:search ESC:back │
└─────────────────────────────────────────────────────────────┘
```

### Now Playing Overlay

```
╔═══════════════════════════════════════════════════════════╗
║  ▶ NOW CASTING                                            ║
║                                                           ║
║  Blade Runner 2049 (2017)                                 ║
║  ████████████████░░░░░░░░░░░░░░  1:23:45 / 2:43:00       ║
║                                                           ║
║  📺 Living Room TV  │  🔊 75%  │  🎬 1080p               ║
║  [Space] Pause  [s] Stop  [←/→] Seek  [Esc] Hide         ║
╚═══════════════════════════════════════════════════════════╝
```

### Quality Selection

```
┌─ SELECT STREAM ─────────────────────────────────────────────┐
│                                                             │
│ ▸ [1] 2160p WEB-DL HDR       12.1 GB   Seeds: 89    ★★★★  │
│   [2] 1080p BluRay x264       4.2 GB   Seeds: 142   ★★★   │
│   [3] 1080p WEB-DL            2.8 GB   Seeds: 234   ★★★   │
│   [4] 720p WEB                1.8 GB   Seeds: 567   ★★    │
│   [5] 480p WEB                 890 MB  Seeds: 123   ★     │
│                                                             │
│ [Enter] Cast  [c] Cast to device  [Esc] Back               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Development

```bash
# Run in development
cargo run

# Run tests (654 tests)
cargo test

# Run specific test module
cargo test --test tmdb_test

# Lint
cargo clippy

# Format
cargo fmt

# Build release binary
cargo build --release
```

### Project Structure

```
streamtui/
├── src/
│   ├── main.rs          # Entry point, TUI loop
│   ├── app.rs           # Application state machine
│   ├── cli.rs           # CLI argument parsing
│   ├── commands.rs      # CLI command handlers
│   ├── models.rs        # Data structures
│   ├── api/             # API clients
│   │   ├── tmdb.rs      # TMDB for search/info
│   │   └── torrentio.rs # Torrentio for streams
│   ├── stream/          # Streaming components
│   │   ├── torrent.rs   # webtorrent-cli wrapper
│   │   ├── cast.rs      # catt wrapper
│   │   └── subtitles.rs # OpenSubtitles client
│   └── ui/              # TUI components
│       ├── theme.rs     # Cyberpunk color palette
│       ├── search.rs    # Search view
│       ├── browser.rs   # Content browser
│       ├── detail.rs    # Detail view
│       ├── subtitles.rs # Subtitle selection
│       └── player.rs    # Now playing overlay
├── tests/               # Integration tests
└── specs/               # Design specifications
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with 💜 by Gorka & Hermes**

*"The future is now, old man."*

</div>
