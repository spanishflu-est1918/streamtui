# Implementation Plan

## Approach: TDD (Test-Driven Development)
For each component: **Write tests → Make them pass → Refactor**

## Phase 1: Foundation

### 1. [x] Project setup
- Initialize Cargo.toml with dependencies
- Set up module structure
- Configure clippy and rustfmt
- Create tests/ directory

### 2. [x] Models and types (src/models.rs)
- Define all data structures from specs
- Implement Display traits
- Write unit tests for type conversions

### 3. [x] Theme (src/ui/theme.rs)
- Define color constants
- Create style helpers
- Test: Color contrast ratios

## Phase 2: API Clients (TDD)

### 4. [ ] TMDB client tests (tests/tmdb_test.rs)
- Write all tests from specs/search.md
- Use mockito for HTTP mocking
- Tests should fail initially

### 5. [ ] TMDB client implementation (src/api/tmdb.rs)
- Implement TmdbClient
- Make tests pass
- Handle rate limits and errors

### 6. [x] Torrentio client tests (tests/torrentio_test.rs)
- Write all tests from specs/addons.md
- Mock HTTP responses
- Tests compile but fail (TDD - no implementation yet)
- 15 tests: 9 pass (parsing), 6 fail (HTTP client needs implementation)

### 7. [x] Torrentio client implementation (src/api/torrentio.rs)
- Implement TorrentioClient
- Parse quality, seeds, size
- Make tests pass

## Phase 3: Streaming (TDD)

### 8. [x] Torrent manager tests (tests/torrent_test.rs)
- Write tests from specs/torrent.md
- Mock webtorrent subprocess
- 41 tests: all pass (TDD validation functions, state machine, parsing)
- Tests: magnet validation, session states, progress/speed parsing, URL generation, process management, error handling

### 9. [x] Torrent manager implementation (src/stream/torrent.rs)
- Implement TorrentManager
- webtorrent subprocess handling
- Progress parsing
- Make tests pass
- All 41 tests GREEN: validation, state machine, parsing, URL generation, process management

### 10. [x] Cast manager tests (tests/cast_test.rs)
- Write tests from specs/cast.md
- Mock catt subprocess
- 27 tests: 26 pass, 1 ignored (integration test requiring real catt)
- Tests: discovery parsing, status parsing, volume clamping, seek validation, error handling, concurrency, state display

### 11. [x] Cast manager implementation (src/stream/cast.rs)
- Implement CastManager
- catt integration
- Device discovery
- Make tests pass
- All 26 tests GREEN (1 ignored integration test)

### 12. [x] Subtitle client tests (tests/subtitles_test.rs)
- Write tests from specs/subtitles.md
- Mock OpenSubtitles API
- 25 tests: all pass
- Tests: search parsing, language filtering, caching, SRT→WebVTT conversion, URL generation, cast command with subtitles, empty results, rate limiting, language priority/auto-select

### 13. [x] Subtitle client implementation (src/stream/subtitles.rs)
- Implement SubtitleClient
- OpenSubtitles API integration
- SRT to WebVTT conversion
- Caching
- Make tests pass
- All 25 tests GREEN

## Phase 4: TUI (TDD)

### 14. [x] UI components tests (tests/ui_test.rs)
- Test layout at various terminal sizes
- Test navigation state machine
- Test search input handling
- 30 tests: all pass
- Tests: theme colors (valid RGB, WCAG contrast ratios), responsive layout (80x24, 200x50), navigation (up/down, enter, escape), search focus (/, typing, enter, escape), content card render, now playing overlay

### 15. [ ] Theme and styles (src/ui/theme.rs)
- Cyberpunk color palette
- Component styles
- Visual test (screenshot comparison optional)

### 16. [x] App state (src/app.rs)
- Define AppState enum (Home, Search, Detail, Sources, Subtitles, Playing)
- Navigation stack with back behavior
- Selection state per view (ListState with scroll support)
- Loading states (Idle, Loading, Error)
- Input mode (Normal, Editing)
- Full keyboard event handling (up/down/enter/escape/q, vim keys j/k)
- View-specific states (HomeState, SearchState, DetailState, SourcesState, SubtitlesState, PlayingState)
- 18 unit tests: all pass

### 17. [ ] Search view (src/ui/search.rs)
- Search input widget
- Results list
- Trending section

### 18. [ ] Browser view (src/ui/browser.rs)
- Content list with selection
- Quality/source selection popup
- Keyboard handling

### 19. [ ] Detail view (src/ui/detail.rs)
- Movie/TV show info display
- Season/episode picker for TV
- Source list

### 20. [ ] Subtitle selection view (src/ui/subtitles.rs)
- Language grouped list
- Trust/quality indicators
- Auto-select option

### 21. [ ] Now Playing overlay (src/ui/player.rs)
- Casting status
- Progress bar
- Playback controls
- Subtitle indicator

## Phase 5: CLI (for Claude Code)

### 22. [x] CLI argument parsing (src/cli.rs)
- Use clap for argument parsing
- Define all subcommands from specs/cli.md
- JSON output formatting
- Exit codes
- Complete Cli struct with global flags (--json, --device, --quiet, --config)
- All subcommands: Search, Trending, Info, Streams, Subtitles, Devices, Cast, Status, Play, Pause, Stop, Seek, Volume
- JSON output helpers (JsonOutput<T>, Output struct)
- IMDB ID validation
- Seek/Volume parsing (absolute, relative, timestamp)
- 10 unit tests: all pass

### 23. [ ] CLI commands implementation
- search, trending, info, subtitles commands
- streams command
- devices command
- cast command with subtitle support
- status, play, pause, stop, seek, volume

### 24. [ ] CLI tests (tests/cli_test.rs)
- Test all commands with mocked backends
- Test JSON output format
- Test exit codes

## Phase 6: Integration

### 25. [ ] Main entry point (src/main.rs)
- Route to TUI or CLI based on args
- Event handling for TUI
- State management
- Error boundaries

### 26. [ ] End-to-end flow
- Search → Select → Subtitles → Cast (TUI)
- CLI: streamtui search → streams → subtitles → cast
- Integration test with mocks
- Manual testing

### 27. [ ] README and docs
- Installation instructions
- Usage guide (TUI + CLI)
- Claude Code examples
- Screenshots
- Dependencies (webtorrent, catt)

### 28. [ ] Polish
- Error messages
- Loading indicators
- Edge case handling
- Performance optimization

## Done Criteria
- All tests pass
- Can search for content
- Can browse trending
- Can select quality/source
- Can cast to Chromecast
- UI is responsive and beautiful
- Error handling is graceful

## Dependencies (Cargo.toml)
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
ratatui = "0.29"
crossterm = "0.28"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
regex = "1"
urlencoding = "2"
local-ip-address = "0.6"
dirs = "5"
toml = "0.8"

[dev-dependencies]
mockito = "1"
tokio-test = "0.4"
```
