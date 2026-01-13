# StreamTUI

A cyberpunk terminal interface for streaming movies and TV shows to Chromecast.

## Vision
Neon-soaked terminal UI. Search content, pick quality, cast to TV. Simple. Fast. Beautiful.

## Core Specs

| Spec | Description | File |
|------|-------------|------|
| cli | Command-line interface for automation | specs/cli.md |
| tui | Terminal UI layout, themes, interactions | specs/tui.md |
| search | TMDB integration for search and metadata | specs/search.md |
| addons | Stremio addon client (Torrentio, Cinemeta) | specs/addons.md |
| torrent | Magnet parsing, webtorrent streaming | specs/torrent.md |
| cast | Chromecast discovery and casting | specs/cast.md |
| subtitles | Stremio subtitle integration | specs/subtitles.md |

## Search Keywords
- tui, terminal, ratatui, crossterm, ui, interface
- streaming, stream, cast, chromecast, dlna
- torrent, magnet, webtorrent, peerflix
- search, tmdb, imdb, movies, tv, shows, series
- stremio, torrentio, cinemeta, addon
- cyberpunk, neon, theme, colors

## Tech Stack
- **Language**: Rust 2021 edition
- **TUI**: ratatui + crossterm
- **HTTP**: reqwest (async)
- **Async**: tokio
- **JSON**: serde + serde_json
- **Cast**: rust-cast or mdns + custom
- **Torrent**: webtorrent-cli (shelled) or libtorrent bindings

## Directory Structure
```
streamtui/
├── specs/              # Specifications
├── src/
│   ├── main.rs         # Entry point
│   ├── app.rs          # App state and logic
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── theme.rs    # Cyberpunk colors
│   │   ├── search.rs   # Search view
│   │   ├── browser.rs  # Content browser
│   │   ├── detail.rs   # Movie/show detail
│   │   └── player.rs   # Now playing overlay
│   ├── api/
│   │   ├── mod.rs
│   │   ├── tmdb.rs     # TMDB client
│   │   └── torrentio.rs # Torrentio addon
│   ├── stream/
│   │   ├── mod.rs
│   │   ├── torrent.rs  # Torrent management
│   │   └── cast.rs     # Chromecast control
│   └── models.rs       # Data structures
├── tests/              # Integration tests
├── Cargo.toml
└── README.md
```

## Design Principles
1. **TDD** — Tests first, implementation follows
2. **Fast startup** — Under 100ms to interactive
3. **Responsive** — Async everything, never block UI
4. **Beautiful** — Cyberpunk neon aesthetic throughout
5. **Keyboard-first** — Arrow keys, Enter, Escape, single letters
