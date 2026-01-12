# Cast Specification

## Overview
Chromecast discovery and casting control. This is the **paramount** feature.

## Discovery

### mDNS/Zeroconf
Chromecasts advertise via mDNS:
- Service type: `_googlecast._tcp.local`
- TXT record contains device name, ID, capabilities

### Alternative: catt
Use `catt` CLI as subprocess for simpler integration:
```bash
catt -d "Living Room TV" cast <url>
catt -d "Living Room TV" status
catt -d "Living Room TV" stop
```

## Data Models

### CastDevice
```rust
struct CastDevice {
    id: String,
    name: String,           // "Living Room TV"
    address: IpAddr,
    port: u16,
    model: Option<String>,  // "Chromecast Ultra"
}
```

### CastState
```rust
enum CastState {
    Idle,
    Connecting,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Error(String),
}
```

### PlaybackStatus
```rust
struct PlaybackStatus {
    state: CastState,
    position: Duration,     // Current position
    duration: Duration,     // Total duration
    volume: f32,            // 0.0 - 1.0
    title: Option<String>,  // Media title
}
```

## Manager Interface

```rust
impl CastManager {
    /// Discover devices on local network
    async fn discover(&self, timeout: Duration) -> Vec<CastDevice>;
    
    /// Get cached devices (last discovery)
    fn devices(&self) -> &[CastDevice];
    
    /// Set active cast target
    fn set_target(&mut self, device: &CastDevice);
    
    /// Get current target
    fn target(&self) -> Option<&CastDevice>;
    
    /// Cast URL to target device
    async fn cast(&self, url: &str, title: Option<&str>) -> Result<()>;
    
    /// Get playback status
    async fn status(&self) -> Result<PlaybackStatus>;
    
    /// Playback controls
    async fn play(&self) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn seek(&self, position: Duration) -> Result<()>;
    async fn set_volume(&self, volume: f32) -> Result<()>;
}
```

## catt Integration (Primary)

### Discovery
```bash
catt scan
```
Output:
```
Scanning for Chromecast devices...
Living Room TV - 192.168.1.50
Bedroom TV - 192.168.1.51
```

### Cast
```bash
catt -d "Living Room TV" cast "http://192.168.1.100:8888/0"
```

### Status
```bash
catt -d "Living Room TV" status
```
Output:
```
State: PLAYING
Duration: 10234.5
Current time: 1234.5
Volume: 80
```

### Controls
```bash
catt -d "Living Room TV" pause
catt -d "Living Room TV" play
catt -d "Living Room TV" stop
catt -d "Living Room TV" seek 3600  # seconds
catt -d "Living Room TV" volume 50  # percent
```

## Native Cast Protocol (Future)

For pure Rust implementation:
1. mDNS discovery with `mdns` crate
2. CASTV2 protocol over TLS
3. Media namespace for playback control

## User Flow

1. On startup, discover devices (async, non-blocking)
2. Show device selector if multiple found
3. Remember last used device
4. On cast: start torrent stream → get URL → cast to device
5. Show now playing overlay with controls

## Configuration

Store last used device in config:
```toml
[cast]
last_device = "Living Room TV"
```

## Tests (TDD)

### test_discover_parses_catt_output
- Input: "Living Room TV - 192.168.1.50\nBedroom - 192.168.1.51"
- Expect: 2 CastDevice items with correct names and IPs

### test_discover_handles_no_devices
- Input: "No devices found"
- Expect: Empty Vec, no error

### test_cast_forms_correct_command
- device = "Living Room TV", url = "http://192.168.1.100:8888/0"
- Expect: `catt -d "Living Room TV" cast "http://192.168.1.100:8888/0"`

### test_status_parsing
- Input: "State: PLAYING\nDuration: 10234.5\nCurrent time: 1234.5"
- Expect: state = Playing, duration = 10234s, position = 1234s

### test_status_buffering
- Input: "State: BUFFERING"
- Expect: state = Buffering

### test_volume_clamps
- Input: volume = 1.5
- Expect: clamped to 1.0
- Input: volume = -0.5
- Expect: clamped to 0.0

### test_seek_validation
- Seek beyond duration → Error or clamp
- Seek negative → Error or clamp to 0

### test_handles_catt_not_installed
- When catt not in PATH
- Expect: Clear error "catt not found. Install: pip install catt"

### test_handles_device_offline
- Target device goes offline
- Expect: Error with device name
- Expect: Graceful recovery option

### test_cast_failure_recovery
- Cast fails mid-stream
- Expect: State = Error with message
- Expect: Can retry cast

### test_concurrent_discovery
- Discovery running
- User requests cast
- Expect: Queue or wait, no crash
