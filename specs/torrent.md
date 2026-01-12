# Torrent Specification

## Overview
Torrent-to-HTTP streaming via webtorrent-cli for casting.

## Approach
Use `webtorrent-cli` as subprocess — reliable, maintained, handles DHT/trackers.

Alternative: Native Rust with `libtorrent` bindings (future optimization).

## webtorrent-cli Interface

### Start Streaming
```bash
webtorrent "<magnet_url>" \
  --select <file_idx> \
  --stdout \
  --no-quit \
  2>/dev/null
```

Or with HTTP server:
```bash
webtorrent "<magnet_url>" \
  --select <file_idx> \
  --port 8888 \
  --hostname 0.0.0.0 \
  --no-quit
```

### Output Parsing
webtorrent outputs progress to stderr:
```
Downloading: The.Batman.2022.mkv
Speed: 5.2 MB/s  Downloaded: 1.2 GB  Time: 3:45
Server running at http://localhost:8888
```

## Data Models

### TorrentSession
```rust
struct TorrentSession {
    id: Uuid,
    magnet: String,
    file_idx: Option<u32>,
    state: TorrentState,
    stream_url: Option<String>,
    progress: f32,           // 0.0 - 1.0
    download_speed: u64,     // bytes/sec
    downloaded: u64,         // bytes
}
```

### TorrentState
```rust
enum TorrentState {
    Starting,
    Connecting,     // Finding peers
    Downloading,    // Active download
    Streaming,      // HTTP server ready
    Paused,
    Stopped,
    Error(String),
}
```

## Manager Interface

```rust
impl TorrentManager {
    fn new() -> Self;
    
    /// Start streaming a magnet, returns stream URL when ready
    async fn start(&mut self, magnet: &str, file_idx: Option<u32>) -> Result<TorrentSession>;
    
    /// Get current session status
    fn status(&self, id: Uuid) -> Option<&TorrentSession>;
    
    /// Stop streaming
    async fn stop(&mut self, id: Uuid) -> Result<()>;
    
    /// Stop all active sessions
    async fn stop_all(&mut self) -> Result<()>;
}
```

## Stream URL
Once webtorrent HTTP server is running:
```
http://localhost:8888/0  # file index 0
```

For casting, need to use machine's LAN IP:
```
http://192.168.1.100:8888/0
```

## LAN IP Detection
```rust
fn get_lan_ip() -> Result<IpAddr> {
    // Use local_ip_address crate
    // Or parse from `ip route get 1.1.1.1`
}
```

## Process Management
- Spawn webtorrent as child process
- Parse stderr for progress updates
- Kill on stop/quit
- Cleanup on app exit (SIGTERM handler)

## Tests (TDD)

### test_magnet_validation
- Valid magnet with infohash → Ok
- Invalid magnet → Error
- Empty string → Error

### test_session_state_transitions
- New session starts in Starting
- After peers found → Connecting
- After download begins → Downloading  
- After HTTP ready → Streaming
- After stop() → Stopped

### test_progress_parsing
- Input: "Downloaded: 1.2 GB"
- With total 4.0 GB → progress = 0.3

### test_speed_parsing
- Input: "Speed: 5.2 MB/s"
- Expect: download_speed = 5452595 bytes/sec

### test_stream_url_generation
- port = 8888, file_idx = 0, lan_ip = 192.168.1.100
- Expect: "http://192.168.1.100:8888/0"

### test_stop_kills_process
- Start a session
- Call stop()
- Expect: Child process terminated
- Expect: Port freed

### test_stop_all_cleanup
- Start 3 sessions
- Call stop_all()
- Expect: All processes terminated
- Expect: All sessions in Stopped state

### test_handles_webtorrent_not_installed
- When webtorrent not in PATH
- Expect: Clear error message "webtorrent-cli not found"

### test_handles_connection_failure
- Magnet with no peers
- After timeout → Error state
- Expect: "No peers found" error
