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
- 🎨 **Cyberpunk neon theme** with smooth animations
- ⌨️ **Vim-style navigation** (j/k, /, Esc)
- 🔍 **Real-time search** with fuzzy matching
- 📺 **Multi-quality streams** (4K, 1080p, 720p, 480p)
- 🌐 **Subtitle support** with language selection
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
| `Enter` | Select item |
| `c` | View sources (from detail view) |
| `u` | Select subtitles |
| `Space` | Play/Pause |
| `←/→` | Seek ±10s |
| `Esc` | Go back |
| `q` | Quit |

### CLI Mode (Automation)

Every TUI action is available as a CLI command with JSON output:

#### Search Content

```bash
# Basic search
streamtui search "blade runner"

# Filter by type, limit results
streamtui search "breaking bad" --media-type tv --limit 10

# JSON output (automatic when piped)
streamtui search "inception" --json
```

#### Browse Trending

```bash
# Today's trending
streamtui trending

# This week's trending movies
streamtui trending --window week --media-type movie
```

#### Get Streams

```bash
# Movie streams
streamtui streams tt1856101

# TV episode streams  
streamtui streams tt0903747 --season 1 --episode 1

# Filter by quality
streamtui streams tt1877830 --quality 1080p --sort seeds
```

#### Find Subtitles

```bash
# English subtitles
streamtui subtitles tt1856101 --lang en

# Multiple languages
streamtui subtitles tt0903747 --lang "en,es,fr" -s 1 -e 1
```

#### Discover Devices

```bash
# List available Chromecasts
streamtui devices

# Extended scan
streamtui devices --timeout 10
```

#### Cast Content

```bash
# Cast movie to device
streamtui cast tt1856101 --device "Living Room TV"

# Cast with quality preference
streamtui cast tt1877830 -d "Bedroom TV" -Q 1080p

# Cast TV episode with subtitles
streamtui cast tt0903747 -s 1 -e 1 -d TV --subtitle en
```

#### Playback Control

```bash
# Check status
streamtui status
streamtui status --json  # For scripting

# Play/Pause
streamtui play
streamtui pause

# Seek
streamtui seek 3600      # To 1 hour
streamtui seek +30       # Forward 30s
streamtui seek -10       # Back 10s
streamtui seek 1:30:00   # To timestamp

# Volume
streamtui volume 50      # Set to 50%
streamtui volume +10     # Increase by 10%

# Stop
streamtui stop
```

### Global Options

These flags work with any command:

```bash
--json, -j        # Force JSON output
--device, -d      # Set default Chromecast device
--quiet, -q       # Suppress non-essential output
--config, -c      # Custom config file path
```

## 🤖 Claude Code Integration

StreamTUI is designed for seamless AI agent integration. Here are example workflows:

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
```

### Automated Evening Routine

```bash
#!/bin/bash
# Find a trending movie and cast it

movie=$(streamtui trending --media-type movie --json | jq -r '.data[0].imdb_id')
streamtui cast "$movie" --device "Living Room TV" --quality 1080p
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

## ⚙️ Configuration

StreamTUI looks for configuration at `~/.config/streamtui/config.toml`:

```toml
# Default Chromecast device
default_device = "Living Room TV"

# Preferred quality (4k, 1080p, 720p, 480p)
preferred_quality = "1080p"

# Preferred subtitle languages
subtitle_languages = ["en", "es"]

# API keys (optional - uses defaults)
# tmdb_api_key = "your-key"
```

## 📸 Screenshots

<!-- TODO: Add screenshots -->

*Screenshots coming soon...*

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

## 🛠️ Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with 💜 by Gorka & Hermes**

*"The future is now, old man."*

</div>
