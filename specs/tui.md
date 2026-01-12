# TUI Specification

## Overview
Ratatui-based terminal interface with cyberpunk neon aesthetic.

## Theme: Cyberpunk Neon

```
Background:     #0a0a0f (deep black-blue)
Primary:        #00fff2 (cyan neon)
Secondary:      #ff00ff (magenta)
Accent:         #ffff00 (yellow)
Highlight:      #ff0080 (hot pink)
Text:           #e0e0e0 (soft white)
Dim:            #404050 (muted)
Success:        #00ff00 (green)
Warning:        #ffaa00 (orange)
Error:          #ff0040 (red)
```

## Layout

### Main Screen
```
┌─ Header ─────────────────────────────────────────────────────┐
│  Logo/Title                              Search: [_______]   │
├─ Content ────────────────────────────────────────────────────┤
│                                                              │
│  Content list (scrollable)                                   │
│  - Selected item highlighted with neon glow effect          │
│  - Shows: Title, Year, Quality, Size                        │
│                                                              │
├─ Status Bar ─────────────────────────────────────────────────┤
│  Keybindings        |  Cast target: Device Name  |  Status  │
└──────────────────────────────────────────────────────────────┘
```

### Views
1. **Home** — Search box + trending content
2. **Search Results** — Grid/list of matches
3. **Detail** — Movie/show info + stream sources
4. **Seasons** — For TV: season/episode picker
5. **Sources** — Quality/source selection
6. **Casting** — Now playing overlay

## Interactions

| Key | Action |
|-----|--------|
| `/` or `s` | Focus search |
| `↑↓` | Navigate list |
| `Enter` | Select / Confirm |
| `Escape` | Back / Cancel |
| `c` | Cast selected |
| `i` | Show info/detail |
| `q` | Quit |
| `Tab` | Switch panels |
| `1-9` | Quick select source |

## Components

### ContentCard
```
┌────────────────────────────────────────┐
│ ▸ The Batman (2022)         1080p 4.2GB│
│   Action, Crime • 2h 56m    Seeds: 142 │
└────────────────────────────────────────┘
```

### SourceItem
```
│ [1] 1080p BluRay x264      4.2 GB  S:142 │
│ [2] 2160p WEB-DL HDR      12.1 GB  S:89  │
│ [3] 720p WEB              1.8 GB  S:203  │
```

### NowPlaying (overlay)
```
╔═══════════════════════════════════════════╗
║  ▶ NOW CASTING                            ║
║                                           ║
║  The Batman (2022)                        ║
║  ████████████░░░░░░░░  45:23 / 2:56:00   ║
║                                           ║
║  📺 Living Room TV                        ║
║  [Space] Pause  [s] Stop  [Esc] Close    ║
╚═══════════════════════════════════════════╝
```

## Tests (TDD)

### test_theme_colors
- All theme colors are valid RGB hex values
- Contrast ratio between text and background >= 4.5:1

### test_layout_responsive
- Layout renders correctly at 80x24 (minimum)
- Layout renders correctly at 200x50 (large)
- Content area scrolls when items exceed height

### test_navigation
- Up/Down moves selection in list
- Enter on content opens detail view
- Escape from detail returns to list
- Search input captures keystrokes

### test_search_focus
- `/` focuses search input
- Typing updates search query
- Enter submits search
- Escape clears and unfocuses

### test_content_card_render
- Displays title, year, quality, size
- Truncates long titles with ellipsis
- Highlights selected item with accent color

### test_now_playing_overlay
- Renders centered on screen
- Shows progress bar
- Updates playback time
- Responds to pause/stop keys
