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

### 4. [x] TMDB client tests (tests/tmdb_test.rs)
- Write all tests from specs/search.md
- Use mockito for HTTP mocking
- 13 tests: all pass
- Tests: search parsing, year extraction, person filtering, trending, movie_detail, tv_detail with seasons, tv_season episodes, rate limit retry, 404 handling, server errors, invalid JSON, bearer token auth

### 5. [x] TMDB client implementation (src/api/tmdb.rs)
- Implement TmdbClient with reqwest
- search(), trending(), movie_detail(), tv_detail(), tv_season()
- Rate limit handling with automatic 429 retry and backoff
- Error types: NotFound, RateLimited, ServerError, InvalidResponse
- All 13 tests pass

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

### 15. [x] Theme and styles (src/ui/theme.rs)
- Cyberpunk color palette (all 10 core colors + 4 derived colors)
- Component styles (30+ style helpers: list items, inputs, quality indicators, seeds, status bar, etc.)
- WCAG contrast ratio validation (12 unit tests ensure accessibility)
- Visual test (optional) - contrast ratio tests cover accessibility requirements

### 16. [x] App state (src/app.rs)
- Define AppState enum (Home, Search, Detail, Sources, Subtitles, Playing)
- Navigation stack with back behavior
- Selection state per view (ListState with scroll support)
- Loading states (Idle, Loading, Error)
- Input mode (Normal, Editing)
- Full keyboard event handling (up/down/enter/escape/q, vim keys j/k)
- View-specific states (HomeState, SearchState, DetailState, SourcesState, SubtitlesState, PlayingState)
- 18 unit tests: all pass

### 17. [x] Search view (src/ui/search.rs)
- Search input widget with cursor, backspace, delete, home/end navigation
- Results list with cyberpunk neon styling (rating colors, type badges)
- Trending section (shown when no active search)
- Loading and error states
- Popup mode for overlay rendering
- 31 unit tests: all pass

### 18. [x] Browser view (src/ui/browser.rs)
- Content list with selection
- Quality/source selection popup
- Keyboard handling
- BrowserView and SourceBrowserView with navigation, paging, styling
- 26 unit tests: all pass

### 19. [x] Detail view (src/ui/detail.rs)
- Movie/TV show info display
- Season/episode picker for TV
- Source list
- DetailView with seasons, episodes, sources, overview scroll
- DetailFocus cycling for TV vs Movie
- 32 unit tests: all pass

### 20. [x] Subtitle selection view (src/ui/subtitles.rs)
- Group subtitles by language (BTreeMap for consistent ordering)
- Trust indicators (✓ for trusted, ⚠️AI for machine translated, 👂SDH for hearing impaired)
- Download count display (formatted as k/M for large numbers)
- Selection with Enter, navigation with ↑↓, Page Up/Down, Home/End
- Cyberpunk styling (neon borders, accent colors, language headers)

### 21. [x] Now Playing overlay (src/ui/player.rs)
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

### 23. [x] CLI commands implementation (src/commands.rs)
- search, trending, info, subtitles commands with TMDB API
- streams command with Torrentio integration
- devices command with catt scan
- cast command with webtorrent streaming, quality selection, subtitle support
- cast_magnet, play_local commands for direct magnet casting
- status, play, pause, stop, seek, volume via catt playback control
- Proper error handling with semantic exit codes

### 24. [x] CLI tests (tests/cli_test.rs)
- Test all CLI argument parsing (search, trending, info, streams, subtitles, devices, cast, status, playback)
- Test global flags (--json, --device, --quiet, --config)
- Test command aliases (s, tr, i, st, sub, dev, vol)
- Test seek position parsing (absolute, relative +/-, timestamp HH:MM:SS)
- Test volume parsing (absolute 0-100, relative +/-, capped at 100)
- Test IMDB ID validation
- Test JSON output format (success/error responses, playback states)
- Test cast device parsing from catt scan output
- Test playback status parsing from catt status output
- Test exit codes (0-6 for success, error, invalid args, network, device not found, no streams, cast failed)
- Test stream source quality ranking, size parsing, seed parsing, magnet generation
- Test subtitle trust scoring and display formatting
- Test torrent session URL generation, speed/progress parsing
- 75 tests: all pass

## Phase 6: Integration

### 25. [x] Main entry point (src/main.rs)
- Route to TUI or CLI based on args (no subcommand = TUI)
- Initialize terminal with crossterm (raw mode, alternate screen)
- Create App state and run main event loop
- Handle key events, update state, render UI
- Restore terminal on exit (always, even on error)
- CLI mode dispatches to all command handlers
- Full TUI rendering for all AppStates (Home, Search, Detail, Sources, Subtitles, Playing)
- Error popup overlay
- Status bar with mode, state, device, keybinds
- Cyberpunk neon theme integration

### 26. [x] End-to-end flow
- Search → Select → Subtitles → Cast (TUI)
- CLI: streamtui search → streams → subtitles → cast
- Integration test with mocks (17 tests)
- All E2E tests passing
- Tests: TUI state machine flows, API integration, CLI command flows, full mocked E2E, error handling, edge cases

### 27. [x] README and docs
- Installation instructions (cargo install, npm/pip deps)
- Usage guide (TUI keyboard shortcuts + full CLI reference)
- Claude Code examples (search/cast workflow, status monitoring, automation)
- ASCII art mockup (screenshots TODO - require manual capture)
- Dependencies documented (webtorrent-cli, catt)

### 28. [x] Polish
- Error messages: CLI has descriptive errors with exit codes, TUI has error popup overlay
- Loading indicators: TUI shows ⟳ spinner for search, sources, subtitles, connecting states
- Edge case handling: 654 tests covering all edge cases, empty states, bounds checking
- Performance: Zero warnings in build, clean clippy, all tests pass in ~3 seconds
- Code quality: Fixed all test warnings, removed unused imports/variables

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
