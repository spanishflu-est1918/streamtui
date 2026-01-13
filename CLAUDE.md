# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build (optimized)

# Run
cargo run                      # Launch TUI mode
cargo run -- search "query"    # CLI mode

# Test
cargo test                     # Run all tests (654 tests)
cargo test <name>              # Run tests matching name
cargo test --test tmdb_test    # Run specific test file
cargo test -- --nocapture      # Show test output

# Lint & Format
cargo clippy                   # Lint (see .clippy.toml for thresholds)
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting
```

## Architecture Overview

StreamTUI is a Rust TUI/CLI application for searching movies/TV shows and casting to Chromecast. It operates in two modes:
- **TUI mode**: Interactive ratatui-based interface (default, no args)
- **CLI mode**: Scriptable JSON-output commands (when subcommand provided)

### Core Flow

```
main.rs → cli.rs (parse) → Either:
  ├── TUI: app.rs (state machine) → ui/*.rs (render)
  └── CLI: commands.rs → api/*.rs + stream/*.rs
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `app.rs` | State machine (AppState enum), navigation stack, keyboard handling |
| `cli.rs` | Clap command definitions, exit codes, Output formatter |
| `commands.rs` | CLI command implementations (search, cast, status, etc.) |
| `models.rs` | Shared data types: SearchResult, StreamSource, CastDevice, etc. |
| `api/tmdb.rs` | TMDB API client for search and content details |
| `api/torrentio.rs` | Torrentio addon client for stream sources |
| `stream/cast.rs` | `catt` CLI wrapper for Chromecast control |
| `stream/torrent.rs` | `webtorrent-cli` wrapper for torrent streaming |
| `stream/subtitles.rs` | OpenSubtitles API client |
| `ui/theme.rs` | Cyberpunk color palette (WCAG-compliant) |

### State Machine (app.rs)

The TUI navigates through states: `Home → Search → Detail → Sources → Subtitles → Playing`

Each state has its own UI component in `ui/`:
- `search.rs` - Search results list
- `browser.rs` - Content browser
- `detail.rs` - Movie/TV detail view
- `subtitles.rs` - Subtitle selection
- `player.rs` - Now playing overlay

### External Dependencies

Casting requires these CLI tools (not Rust crates):
- `webtorrent-cli` (npm) - Torrent streaming with HTTP server
- `catt` (pip) - Chromecast control

## Test Structure

Tests live in `tests/` directory:
- `tmdb_test.rs` - TMDB API tests
- `torrentio_test.rs` - Torrentio API tests
- `subtitles_test.rs` - OpenSubtitles tests
- `cast_test.rs` - Chromecast/catt tests
- `torrent_test.rs` - Webtorrent tests
- `cli_test.rs` - CLI parsing and command tests
- `ui_test.rs` - UI component tests
- `e2e_test.rs` - End-to-end flow tests

Use `mockito` for HTTP mocking in tests.

## CLI Exit Codes

Semantic exit codes for scripting (defined in `cli.rs`):
- 0: Success
- 1: General error
- 2: Invalid arguments
- 3: Network error
- 4: Device not found
- 5: No streams available
- 6: Cast failed

## Design Specs

Detailed specifications in `specs/`:
- `tui.md` - UI layout, cyberpunk theme colors
- `cli.md` - Command structure and JSON output format
- `search.md` - TMDB search integration
- `torrent.md` - Webtorrent streaming
- `cast.md` - Chromecast/catt integration
- `subtitles.md` - OpenSubtitles integration
