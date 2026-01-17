//! CLI Command Handlers
//!
//! Implements all CLI commands by calling the appropriate backend services.
//! Each handler takes CLI args and Output, returns ExitCode.

use serde::Serialize;

use crate::api::{TmdbClient, TorrentioClient};
use crate::cli::{
    CastCmd, CastMagnetCmd, DevicesCmd, ExitCode, InfoCmd, MediaTypeFilter, Output, PauseCmd,
    PlayCmd, PlayLocalCmd, PlaybackState, PlaybackStatus, PlayerChoice, SearchCmd, SeekCmd,
    SeekPosition, StatusCmd, StopCmd, StreamsCmd, SubtitlesCmd, TrendingCmd, TrendingWindow,
    VolumeCmd, VolumeLevel,
};
use crate::config::Config;
use crate::models::{CastDevice, MediaType, Quality, StreamSource};
use crate::stream::{LocalPlayer, PlayerType, SubtitleClient};

// =============================================================================
// Search Command
// =============================================================================

pub async fn search_cmd(cmd: SearchCmd, output: &Output) -> ExitCode {
    let mut config = Config::load();
    let api_key = config.get_tmdb_api_key();
    let client = TmdbClient::new(api_key);

    output.info(format!("Searching for: {}", cmd.query));

    match client.search(&cmd.query).await {
        Ok(mut results) => {
            // Filter by media type if specified
            if let Some(filter) = cmd.media_type {
                results.retain(|r| match filter {
                    MediaTypeFilter::Movie => r.media_type == MediaType::Movie,
                    MediaTypeFilter::Tv => r.media_type == MediaType::Tv,
                });
            }

            // Filter by year range
            if let Some(year_from) = cmd.year_from {
                results.retain(|r| r.year.map(|y| y >= year_from).unwrap_or(false));
            }
            if let Some(year_to) = cmd.year_to {
                results.retain(|r| r.year.map(|y| y <= year_to).unwrap_or(false));
            }

            // Limit results
            results.truncate(cmd.limit);

            if let Err(e) = output.print(&results) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => output.error(format!("Search failed: {}", e), ExitCode::NetworkError),
    }
}

// =============================================================================
// Trending Command
// =============================================================================

pub async fn trending_cmd(cmd: TrendingCmd, output: &Output) -> ExitCode {
    let mut config = Config::load();
    let api_key = config.get_tmdb_api_key();
    let client = TmdbClient::new(api_key);

    let window_str = match cmd.window {
        TrendingWindow::Day => "day",
        TrendingWindow::Week => "week",
    };
    output.info(format!("Fetching trending ({})...", window_str));

    match client.trending().await {
        Ok(mut results) => {
            // Filter by media type if specified
            if let Some(filter) = cmd.media_type {
                results.retain(|r| match filter {
                    MediaTypeFilter::Movie => r.media_type == MediaType::Movie,
                    MediaTypeFilter::Tv => r.media_type == MediaType::Tv,
                });
            }

            // Limit results
            results.truncate(cmd.limit);

            if let Err(e) = output.print(&results) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => output.error(
            format!("Trending fetch failed: {}", e),
            ExitCode::NetworkError,
        ),
    }
}

// =============================================================================
// Info Command
// =============================================================================

pub async fn info_cmd(cmd: InfoCmd, output: &Output) -> ExitCode {
    let mut config = Config::load();
    let api_key = config.get_tmdb_api_key();
    let client = TmdbClient::new(api_key);

    output.info(format!("Getting info for: {}", cmd.id));

    // Try to parse as TMDB ID (number)
    if let Ok(tmdb_id) = cmd.id.parse::<u64>() {
        // Need media type for TMDB ID lookup
        match cmd.media_type {
            Some(MediaTypeFilter::Movie) => match client.movie_detail(tmdb_id).await {
                Ok(detail) => {
                    if let Err(e) = output.print(&detail) {
                        return output
                            .error(format!("Failed to serialize: {}", e), ExitCode::Error);
                    }
                    ExitCode::Success
                }
                Err(e) => output.error(format!("Movie info failed: {}", e), ExitCode::NetworkError),
            },
            Some(MediaTypeFilter::Tv) => match client.tv_detail(tmdb_id).await {
                Ok(detail) => {
                    if let Err(e) = output.print(&detail) {
                        return output
                            .error(format!("Failed to serialize: {}", e), ExitCode::Error);
                    }
                    ExitCode::Success
                }
                Err(e) => output.error(format!("TV info failed: {}", e), ExitCode::NetworkError),
            },
            None => output.error(
                "Media type required for TMDB ID lookup. Use -t movie or -t tv.",
                ExitCode::InvalidArgs,
            ),
        }
    } else {
        // Assume it's an IMDB ID - search for it
        output.error(
            "IMDB ID lookup not yet supported. Use TMDB ID with -t flag.",
            ExitCode::InvalidArgs,
        )
    }
}

// =============================================================================
// Streams Command
// =============================================================================

pub async fn streams_cmd(cmd: StreamsCmd, output: &Output) -> ExitCode {
    let client = TorrentioClient::new();

    output.info(format!("Finding streams for: {}", cmd.imdb_id));

    let result = if let (Some(season), Some(episode)) = (cmd.season, cmd.episode) {
        client
            .episode_streams(&cmd.imdb_id, season as u16, episode)
            .await
    } else {
        client.movie_streams(&cmd.imdb_id).await
    };

    match result {
        Ok(mut streams) => {
            if streams.is_empty() {
                return output.error("No streams found", ExitCode::NoStreams);
            }

            // Filter by quality if specified
            if let Some(quality_filter) = cmd.quality {
                let min_quality = match quality_filter {
                    crate::cli::QualityFilter::Q4k => Quality::UHD4K,
                    crate::cli::QualityFilter::Q1080p => Quality::FHD1080p,
                    crate::cli::QualityFilter::Q720p => Quality::HD720p,
                    crate::cli::QualityFilter::Q480p => Quality::SD480p,
                };
                streams.retain(|s| s.quality.rank() >= min_quality.rank());
            }

            // Sort streams
            match cmd.sort {
                crate::cli::StreamSort::Seeds => {
                    streams.sort_by(|a, b| b.seeds.cmp(&a.seeds));
                }
                crate::cli::StreamSort::Quality => {
                    streams.sort_by(|a, b| b.quality.rank().cmp(&a.quality.rank()));
                }
                crate::cli::StreamSort::Size => {
                    streams
                        .sort_by(|a, b| b.size_bytes.unwrap_or(0).cmp(&a.size_bytes.unwrap_or(0)));
                }
            }

            // Limit results
            streams.truncate(cmd.limit);

            // Create output with index for easy reference
            let indexed: Vec<IndexedStream> = streams
                .into_iter()
                .enumerate()
                .map(|(i, s)| IndexedStream {
                    index: i,
                    stream: s,
                })
                .collect();

            if let Err(e) = output.print(&indexed) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => output.error(
            format!("Stream fetch failed: {}", e),
            ExitCode::NetworkError,
        ),
    }
}

#[derive(Serialize)]
struct IndexedStream {
    index: usize,
    #[serde(flatten)]
    stream: StreamSource,
}

// =============================================================================
// Subtitles Command
// =============================================================================

pub async fn subtitles_cmd(cmd: SubtitlesCmd, output: &Output) -> ExitCode {
    let client = SubtitleClient::new();
    let languages = cmd.languages();
    let lang = languages.first().copied();

    output.info(format!(
        "Searching subtitles for: {} ({})",
        cmd.imdb_id, cmd.lang
    ));

    let result = if let (Some(season), Some(episode)) = (cmd.season, cmd.episode) {
        client
            .search_episode(&cmd.imdb_id, season as u16, episode, lang)
            .await
    } else {
        client.search(&cmd.imdb_id, lang).await
    };

    match result {
        Ok(mut subs) => {
            if subs.is_empty() {
                return output.error("No subtitles found", ExitCode::NoStreams);
            }

            // Filter by preferences
            if cmd.hearing_impaired {
                subs.retain(|s| s.hearing_impaired);
            }
            if cmd.trusted {
                subs.retain(|s| s.from_trusted);
            }

            // Sort by trust score
            subs.sort_by_key(|s| std::cmp::Reverse(s.trust_score()));

            // Limit results
            subs.truncate(cmd.limit);

            if let Err(e) = output.print(&subs) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => output.error(
            format!("Subtitle search failed: {}", e),
            ExitCode::NetworkError,
        ),
    }
}

// =============================================================================
// Devices Command
// =============================================================================

pub async fn devices_cmd(_cmd: DevicesCmd, output: &Output) -> ExitCode {
    output.info("Scanning for Chromecast devices...");

    // Use catt scan to discover devices (no timeout flag in catt 0.13+)
    match tokio::process::Command::new("catt")
        .arg("scan")
        .output()
        .await
    {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            // Parse catt output
            let devices = CastDevice::parse_catt_scan(&stdout);

            if devices.is_empty() {
                // Check if catt reported anything in stderr
                if !stderr.is_empty() && stderr.contains("No devices") {
                    return output.error("No Chromecast devices found", ExitCode::DeviceNotFound);
                }
                // Try parsing stderr too (catt sometimes outputs there)
                let devices_stderr = CastDevice::parse_catt_scan(&stderr);
                if devices_stderr.is_empty() {
                    return output.error("No Chromecast devices found", ExitCode::DeviceNotFound);
                }
                if let Err(e) = output.print(&devices_stderr) {
                    return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
                }
            } else if let Err(e) = output.print(&devices) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output.error(
                    "catt not found. Install with: pip install catt",
                    ExitCode::Error,
                )
            } else {
                output.error(format!("Device scan failed: {}", e), ExitCode::NetworkError)
            }
        }
    }
}

// =============================================================================
// Local Player Helpers
// =============================================================================

/// Convert CLI PlayerChoice to stream PlayerType
fn to_player_type(choice: PlayerChoice) -> PlayerType {
    match choice {
        PlayerChoice::Vlc => PlayerType::Vlc,
        PlayerChoice::Mpv => PlayerType::Mpv,
    }
}

/// Play a stream URL locally in VLC/mpv
/// Wait for a stream URL to become available (webtorrent needs time to connect)
async fn wait_for_stream(url: &str, timeout_secs: u64, output: &Output) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    output.info("Waiting for stream to be ready...");

    while start.elapsed() < timeout {
        match client.head(url).send().await {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 206 => {
                output.info("Stream ready!");
                return true;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    false
}

async fn play_locally(
    stream_url: &str,
    subtitle_path: Option<&std::path::Path>,
    player_type: PlayerType,
    output: &Output,
) -> ExitCode {
    let player = LocalPlayer::new(player_type);

    // Check if player is available
    if !player.is_available().await {
        return output.error(
            format!(
                "{} not found. Install it first.",
                player_type.display_name()
            ),
            ExitCode::Error,
        );
    }

    output.info(format!("Opening in {}...", player_type.display_name()));

    match player.play(stream_url, subtitle_path).await {
        Ok(_child) => {
            #[derive(Serialize)]
            struct PlayLocalSuccess {
                status: &'static str,
                player: String,
                stream_url: String,
            }

            let response = PlayLocalSuccess {
                status: "playing",
                player: player_type.display_name().to_string(),
                stream_url: stream_url.to_string(),
            };

            if let Err(e) = output.print(&response) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }
            ExitCode::Success
        }
        Err(e) => output.error(format!("Failed to start player: {}", e), ExitCode::Error),
    }
}

// =============================================================================
// Cast Command
// =============================================================================

pub async fn cast_cmd(cmd: CastCmd, device: Option<&str>, output: &Output) -> ExitCode {
    // If --vlc flag is set, we don't need a device
    let device_name = if cmd.vlc {
        None
    } else {
        match cmd.device.as_deref().or(device) {
            Some(d) => Some(d),
            None => return output.error(
                "No device specified. Use --device or -d flag, or use --vlc for local playback.",
                ExitCode::DeviceNotFound,
            ),
        }
    };

    if cmd.vlc {
        output.info(format!("Playing {} in VLC...", cmd.imdb_id));
    } else {
        output.info(format!(
            "Casting {} to {}...",
            cmd.imdb_id,
            device_name.unwrap()
        ));
    }

    // Step 1: Get streams
    let torrentio = TorrentioClient::new();
    let streams_result = if let (Some(season), Some(episode)) = (cmd.season, cmd.episode) {
        torrentio
            .episode_streams(&cmd.imdb_id, season as u16, episode)
            .await
    } else {
        torrentio.movie_streams(&cmd.imdb_id).await
    };

    let mut streams = match streams_result {
        Ok(s) if s.is_empty() => {
            return output.error("No streams found for this content", ExitCode::NoStreams);
        }
        Ok(s) => s,
        Err(e) => {
            return output.error(
                format!("Failed to get streams: {}", e),
                ExitCode::NetworkError,
            )
        }
    };

    // Step 2: Sort streams (same as `streams` command to ensure index consistency)
    // Default sort by seeds to match what user sees in `streams` output
    streams.sort_by(|a, b| b.seeds.cmp(&a.seeds));

    // Step 3: Select stream (by index or quality preference)
    let stream = if let Some(idx) = cmd.index {
        if idx >= streams.len() {
            return output.error(
                format!(
                    "Stream index {} out of range (0-{})",
                    idx,
                    streams.len() - 1
                ),
                ExitCode::InvalidArgs,
            );
        }
        streams.remove(idx)
    } else {
        // Re-sort by quality preference if specified
        if let Some(quality_filter) = cmd.quality {
            let target_quality = match quality_filter {
                crate::cli::QualityFilter::Q4k => Quality::UHD4K,
                crate::cli::QualityFilter::Q1080p => Quality::FHD1080p,
                crate::cli::QualityFilter::Q720p => Quality::HD720p,
                crate::cli::QualityFilter::Q480p => Quality::SD480p,
            };
            // Prefer exact match, then closest higher quality
            streams.sort_by(|a, b| {
                let a_diff = (a.quality.rank() as i8 - target_quality.rank() as i8).abs();
                let b_diff = (b.quality.rank() as i8 - target_quality.rank() as i8).abs();
                a_diff.cmp(&b_diff).then_with(|| b.seeds.cmp(&a.seeds))
            });
        }
        streams.remove(0)
    };

    // Step 4: Generate magnet link
    let magnet = stream.to_magnet(&cmd.imdb_id);
    output.info(format!(
        "Selected: {} ({}) - {} seeds",
        stream.name, stream.quality, stream.seeds
    ));

    // Step 5: Handle subtitles if requested
    let subtitle_path: Option<std::path::PathBuf> = if cmd.no_subtitle {
        None
    } else if let Some(ref sub_file) = cmd.subtitle_file {
        // Use local subtitle file directly
        if !sub_file.exists() {
            output.info(format!(
                "Warning: Subtitle file not found: {}",
                sub_file.display()
            ));
            None
        } else {
            output.info(format!("Using local subtitle: {}", sub_file.display()));
            Some(sub_file.clone())
        }
    } else if let Some(ref sub_id) = cmd.subtitle_id {
        // Download specific subtitle by ID
        output.info(format!("Downloading subtitle {}...", sub_id));
        let sub_client = SubtitleClient::new();
        match sub_client
            .download_by_id(
                &cmd.imdb_id,
                sub_id,
                cmd.season.map(|s| s as u16),
                cmd.episode,
            )
            .await
        {
            Ok(path) => {
                output.info(format!("Subtitle downloaded: {}", path.display()));
                Some(path)
            }
            Err(e) => {
                output.info(format!("Warning: Failed to download subtitle: {}", e));
                None
            }
        }
    } else if let Some(ref lang) = cmd.subtitle {
        // Search for subtitle by language and download the best one
        output.info(format!("Searching for {} subtitles...", lang));
        let sub_client = SubtitleClient::new();
        let search_result = if let (Some(season), Some(episode)) = (cmd.season, cmd.episode) {
            sub_client
                .search_episode(&cmd.imdb_id, season as u16, episode, Some(lang))
                .await
        } else {
            sub_client.search(&cmd.imdb_id, Some(lang)).await
        };

        match search_result {
            Ok(mut subs) if !subs.is_empty() => {
                // Sort by trust score and take the best one
                subs.sort_by_key(|s| std::cmp::Reverse(s.trust_score()));
                let best_sub = &subs[0];
                output.info(format!(
                    "Found subtitle: {} ({})",
                    best_sub.release, best_sub.language_name
                ));

                match sub_client.download(best_sub).await {
                    Ok(_) => {
                        let cache_dir = dirs::cache_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                            .join("streamtui")
                            .join("subtitles");
                        let path =
                            cache_dir.join(format!("{}_{}.vtt", best_sub.language, best_sub.id));
                        Some(path)
                    }
                    Err(e) => {
                        output.info(format!("Warning: Failed to download subtitle: {}", e));
                        None
                    }
                }
            }
            Ok(_) => {
                output.info(format!("No {} subtitles found", lang));
                None
            }
            Err(e) => {
                output.info(format!("Warning: Subtitle search failed: {}", e));
                None
            }
        }
    } else {
        None
    };

    // Step 6: Start webtorrent with built-in Chromecast/VLC support
    output.info("Starting torrent stream...");

    let file_idx = stream.file_idx.unwrap_or(0);

    // Build webtorrent command with player flag
    let mut wt_cmd = tokio::process::Command::new("webtorrent");
    wt_cmd
        .arg(&magnet)
        .arg("-s")
        .arg(file_idx.to_string());

    // Add subtitle file if we have one
    if let Some(ref sub_path) = subtitle_path {
        wt_cmd.arg("-t").arg(sub_path);
        output.info(format!("Using subtitles: {}", sub_path.display()));
    }

    if cmd.vlc {
        wt_cmd.arg("--vlc");
    } else {
        // Use webtorrent's built-in Chromecast support
        wt_cmd.arg("--chromecast").arg(device_name.unwrap());
    }

    wt_cmd.arg("--not-on-top");

    // Start webtorrent (blocks until playback ends or user quits)
    output.info("Connecting to peers and starting playback...");

    let result = wt_cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .await;

    match result {
        Ok(status) if status.success() => {
            output.info("Playback completed");
            ExitCode::Success
        }
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            if code == 130 {
                // User interrupted with Ctrl+C
                output.info("Playback stopped by user");
                ExitCode::Success
            } else {
                output.error(
                    format!("webtorrent exited with code {}", code),
                    ExitCode::CastFailed,
                )
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output.error(
                    "webtorrent not found. Install with: npm install -g webtorrent-cli",
                    ExitCode::Error,
                )
            } else {
                output.error(
                    format!("Failed to start webtorrent: {}", e),
                    ExitCode::Error,
                )
            }
        }
    }
}

// =============================================================================
// Cast Magnet Command
// =============================================================================

pub async fn cast_magnet_cmd(
    cmd: CastMagnetCmd,
    device: Option<&str>,
    output: &Output,
) -> ExitCode {
    // If --vlc flag is set, we don't need a device
    let device_name = if cmd.vlc {
        None
    } else {
        match cmd.device.as_deref().or(device) {
            Some(d) => Some(d),
            None => return output.error(
                "No device specified. Use --device or -d flag, or use --vlc for local playback.",
                ExitCode::DeviceNotFound,
            ),
        }
    };

    // Validate magnet link
    if !cmd.magnet.starts_with("magnet:?") {
        return output.error(
            "Invalid magnet link. Must start with 'magnet:?'",
            ExitCode::InvalidArgs,
        );
    }

    // If --vlc flag, use webtorrent's built-in VLC support
    if cmd.vlc {
        output.info("Playing magnet in VLC...");

        let file_idx = cmd.file_idx.unwrap_or(0);

        // Build webtorrent command with --vlc flag
        let mut wt_args = vec![
            cmd.magnet.clone(),
            "--vlc".to_string(),
            "--not-on-top".to_string(),
            "-s".to_string(), file_idx.to_string(), // Select file index
        ];

        // Add subtitle file if provided
        if let Some(sub_file) = &cmd.subtitle_file {
            if !sub_file.exists() {
                return output.error(
                    format!("Subtitle file not found: {}", sub_file.display()),
                    ExitCode::InvalidArgs,
                );
            }
            // VLC subtitle args passed through webtorrent (must be single arg with =)
            wt_args.push(format!("--player-args=--sub-file={}", sub_file.display()));
        }

        // Start webtorrent with --vlc (it handles opening VLC when ready)
        match tokio::process::Command::new("webtorrent")
            .args(&wt_args)
            .spawn()
        {
            Ok(_child) => {
                #[derive(Serialize)]
                struct VlcSuccess {
                    status: &'static str,
                    player: &'static str,
                    magnet: String,
                }
                let response = VlcSuccess {
                    status: "playing",
                    player: "VLC",
                    magnet: cmd.magnet,
                };
                if let Err(e) = output.print(&response) {
                    return output.error(format!("Failed to print: {}", e), ExitCode::Error);
                }
                return ExitCode::Success;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return output.error(
                        "webtorrent not found. Install with: npm install -g webtorrent-cli",
                        ExitCode::Error,
                    );
                }
                return output.error(
                    format!("Failed to start webtorrent: {}", e),
                    ExitCode::Error,
                );
            }
        }
    }

    let device_name = device_name.unwrap();
    let file_idx = cmd.file_idx.unwrap_or(0);

    output.info(format!("Casting magnet to {}...", device_name));

    // Build webtorrent command with --chromecast flag
    // webtorrent handles HTTP server and casting internally
    let mut wt_args = vec![
        cmd.magnet.clone(),
        "--chromecast".to_string(),
        device_name.to_string(),
        "--not-on-top".to_string(),
        "-s".to_string(),
        file_idx.to_string(),
    ];

    // Add subtitle file if provided
    if let Some(sub_file) = &cmd.subtitle_file {
        if !sub_file.exists() {
            return output.error(
                format!("Subtitle file not found: {}", sub_file.display()),
                ExitCode::InvalidArgs,
            );
        }
        wt_args.push("-t".to_string());
        wt_args.push(sub_file.to_string_lossy().to_string());
        output.info(format!("Using subtitle file: {}", sub_file.display()));
    }

    output.info("Starting torrent stream and casting to Chromecast...");
    output.info("Note: If no audio, the source may use DTS/AC3 codec (not supported by Chromecast). Try --vlc for local playback.");

    // Start webtorrent with --chromecast (it handles everything internally)
    match tokio::process::Command::new("webtorrent")
        .args(&wt_args)
        .spawn()
    {
        Ok(_child) => {
            #[derive(Serialize)]
            struct CastMagnetSuccess {
                status: &'static str,
                device: String,
            }

            let response = CastMagnetSuccess {
                status: "casting",
                device: device_name.to_string(),
            };

            if let Err(e) = output.print(&response) {
                return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
            }

            // webtorrent continues in background
            ExitCode::Success
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output.error(
                    "webtorrent not found. Install with: npm install -g webtorrent-cli",
                    ExitCode::Error,
                )
            } else {
                output.error(format!("Cast failed: {}", e), ExitCode::CastFailed)
            }
        }
    }
}

// =============================================================================
// Play Local Command
// =============================================================================

pub async fn play_local_cmd(cmd: PlayLocalCmd, output: &Output) -> ExitCode {
    // Validate magnet link
    if !cmd.magnet.starts_with("magnet:?") {
        return output.error(
            "Invalid magnet link. Must start with 'magnet:?'",
            ExitCode::InvalidArgs,
        );
    }

    let player_type = to_player_type(cmd.player);
    output.info(format!(
        "Playing magnet in {}...",
        player_type.display_name()
    ));

    // Check if player is available
    let player = LocalPlayer::new(player_type);
    if !player.is_available().await {
        return output.error(
            format!(
                "{} not found. Install it first.",
                player_type.display_name()
            ),
            ExitCode::Error,
        );
    }

    // Get local IP for streaming
    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => return output.error(format!("Failed to get local IP: {}", e), ExitCode::Error),
    };

    let port = 8888u16;
    let file_idx = cmd.file_idx.unwrap_or(0);

    // Start webtorrent in background
    let webtorrent = match tokio::process::Command::new("webtorrent")
        .arg(&cmd.magnet)
        .arg("--port")
        .arg(port.to_string())
        .arg("-s")
        .arg(file_idx.to_string())
        .arg("--not-on-top")
        .arg("--keep-seeding")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return output.error(
                    "webtorrent not found. Install with: npm install -g webtorrent-cli",
                    ExitCode::Error,
                );
            }
            return output.error(
                format!("Failed to start webtorrent: {}", e),
                ExitCode::Error,
            );
        }
    };

    output.info("Starting torrent stream...");

    // Build stream URL
    let stream_url = format!("http://{}:{}/{}", local_ip, port, file_idx);
    output.info(format!("Stream URL: {}", stream_url));

    // Wait for stream to be ready (webtorrent needs to connect to peers)
    if !wait_for_stream(&stream_url, 60, output).await {
        output.info("Stream not ready yet, but opening player anyway...");
    }

    // Validate subtitle file if provided
    let subtitle_path = cmd.subtitle_file.as_deref();
    if let Some(sub_file) = subtitle_path {
        if !sub_file.exists() {
            return output.error(
                format!("Subtitle file not found: {}", sub_file.display()),
                ExitCode::InvalidArgs,
            );
        }
    }

    // webtorrent keeps running in background
    let _ = webtorrent;

    // Play locally
    play_locally(&stream_url, subtitle_path, player_type, output).await
}

// =============================================================================
// Status Command
// =============================================================================

pub async fn status_cmd(_cmd: StatusCmd, device: Option<&str>, output: &Output) -> ExitCode {
    let mut catt_args = vec!["status".to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);

            if let Some(status) = crate::models::PlaybackStatus::parse_catt_status(&stdout) {
                // Compute values before moving
                let progress = status.progress() as f64;
                let position = status.position.as_secs();
                let duration = status.duration.as_secs();
                let volume = (status.volume * 100.0) as u8;

                // Convert to CLI PlaybackStatus
                let cli_status = PlaybackStatus {
                    state: match status.state {
                        crate::models::CastState::Idle => PlaybackState::Idle,
                        crate::models::CastState::Buffering => PlaybackState::Buffering,
                        crate::models::CastState::Playing => PlaybackState::Playing,
                        crate::models::CastState::Paused => PlaybackState::Paused,
                        crate::models::CastState::Stopped => PlaybackState::Stopped,
                        crate::models::CastState::Connecting => PlaybackState::Buffering,
                        crate::models::CastState::Error(_) => PlaybackState::Error,
                    },
                    title: status.title,
                    device: device.map(String::from),
                    position: Some(position),
                    duration: Some(duration),
                    progress: Some(progress),
                    volume: Some(volume),
                };

                if let Err(e) = output.print_json(&cli_status) {
                    return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
                }
                ExitCode::Success
            } else {
                // Return idle status if can't parse
                let status = PlaybackStatus::default();
                if let Err(e) = output.print_json(&status) {
                    return output.error(format!("Failed to serialize: {}", e), ExitCode::Error);
                }
                ExitCode::Success
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output.error(
                    "catt not found. Install with: pip install catt",
                    ExitCode::Error,
                )
            } else {
                // Return idle on error
                let status = PlaybackStatus::default();
                if output.print_json(&status).is_err() {
                    return ExitCode::Error;
                }
                ExitCode::Success
            }
        }
    }
}

// =============================================================================
// Playback Control Commands
// =============================================================================

pub async fn play_cmd(_cmd: PlayCmd, device: Option<&str>, output: &Output) -> ExitCode {
    playback_control("play", device, output).await
}

pub async fn pause_cmd(_cmd: PauseCmd, device: Option<&str>, output: &Output) -> ExitCode {
    playback_control("pause", device, output).await
}

pub async fn stop_cmd(cmd: StopCmd, device: Option<&str>, output: &Output) -> ExitCode {
    let result = playback_control("stop", device, output).await;

    if cmd.kill_stream && result == ExitCode::Success {
        // Try to kill any running webtorrent processes
        let _ = tokio::process::Command::new("pkill")
            .arg("-f")
            .arg("webtorrent")
            .output()
            .await;
        output.info("Stopped torrent stream");
    }

    result
}

async fn playback_control(action: &str, device: Option<&str>, output: &Output) -> ExitCode {
    let mut catt_args = vec![action.to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            if result.status.success() {
                #[derive(Serialize)]
                struct ActionOk {
                    status: &'static str,
                }
                if output.print(&ActionOk { status: "ok" }).is_err() {
                    return ExitCode::Error;
                }
                ExitCode::Success
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                output.error(format!("{} failed: {}", action, stderr), ExitCode::Error)
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output.error(
                    "catt not found. Install with: pip install catt",
                    ExitCode::Error,
                )
            } else {
                output.error(format!("{} failed: {}", action, e), ExitCode::Error)
            }
        }
    }
}

// =============================================================================
// Seek Command
// =============================================================================

pub async fn seek_cmd(cmd: SeekCmd, device: Option<&str>, output: &Output) -> ExitCode {
    let position = match cmd.parse_position() {
        SeekPosition::Absolute(secs) => secs as f64,
        SeekPosition::Forward(secs) => {
            // Need to get current position first for relative seek
            // For now, catt supports relative seek with +/- syntax
            return relative_seek(secs, device, output).await;
        }
        SeekPosition::Backward(secs) => {
            return relative_seek(-secs, device, output).await;
        }
        SeekPosition::Invalid(s) => {
            return output.error(
                format!("Invalid seek position: {}", s),
                ExitCode::InvalidArgs,
            );
        }
    };

    let mut catt_args = vec!["seek".to_string(), position.to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            if result.status.success() {
                #[derive(Serialize)]
                struct SeekOk {
                    status: &'static str,
                    position: f64,
                }
                if output
                    .print(&SeekOk {
                        status: "ok",
                        position,
                    })
                    .is_err()
                {
                    return ExitCode::Error;
                }
                ExitCode::Success
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                output.error(format!("Seek failed: {}", stderr), ExitCode::Error)
            }
        }
        Err(e) => output.error(format!("Seek failed: {}", e), ExitCode::Error),
    }
}

async fn relative_seek(delta: i64, device: Option<&str>, output: &Output) -> ExitCode {
    // catt uses ffwd/rewind for relative seeking
    let (action, amount) = if delta >= 0 {
        ("ffwd", delta as u64)
    } else {
        ("rewind", (-delta) as u64)
    };

    let mut catt_args = vec![action.to_string(), amount.to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            if result.status.success() {
                #[derive(Serialize)]
                struct SeekOk {
                    status: &'static str,
                    delta: i64,
                }
                if output
                    .print(&SeekOk {
                        status: "ok",
                        delta,
                    })
                    .is_err()
                {
                    return ExitCode::Error;
                }
                ExitCode::Success
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                output.error(format!("Seek failed: {}", stderr), ExitCode::Error)
            }
        }
        Err(e) => output.error(format!("Seek failed: {}", e), ExitCode::Error),
    }
}

// =============================================================================
// Volume Command
// =============================================================================

pub async fn volume_cmd(cmd: VolumeCmd, device: Option<&str>, output: &Output) -> ExitCode {
    let level = match cmd.parse_level() {
        VolumeLevel::Absolute(vol) => vol,
        VolumeLevel::Relative(delta) => {
            // For relative volume, we need current volume first
            // catt volume accepts 0-100, and "volume up/down" for relative
            if delta > 0 {
                return volume_relative("volumeup", delta as u8, device, output).await;
            } else {
                return volume_relative("volumedown", (-delta) as u8, device, output).await;
            }
        }
        VolumeLevel::Invalid(s) => {
            return output.error(
                format!("Invalid volume level: {}", s),
                ExitCode::InvalidArgs,
            );
        }
    };

    let mut catt_args = vec!["volume".to_string(), level.to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            if result.status.success() {
                #[derive(Serialize)]
                struct VolumeOk {
                    status: &'static str,
                    volume: u8,
                }
                if output
                    .print(&VolumeOk {
                        status: "ok",
                        volume: level,
                    })
                    .is_err()
                {
                    return ExitCode::Error;
                }
                ExitCode::Success
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                output.error(format!("Volume failed: {}", stderr), ExitCode::Error)
            }
        }
        Err(e) => output.error(format!("Volume failed: {}", e), ExitCode::Error),
    }
}

async fn volume_relative(
    action: &str,
    _steps: u8,
    device: Option<&str>,
    output: &Output,
) -> ExitCode {
    // catt doesn't have stepped volume, just volumeup/volumedown
    let mut catt_args = vec![action.to_string()];

    if let Some(d) = device {
        catt_args.insert(0, "-d".to_string());
        catt_args.insert(1, d.to_string());
    }

    match tokio::process::Command::new("catt")
        .args(&catt_args)
        .output()
        .await
    {
        Ok(result) => {
            if result.status.success() {
                #[derive(Serialize)]
                struct VolumeOk {
                    status: &'static str,
                }
                if output.print(&VolumeOk { status: "ok" }).is_err() {
                    return ExitCode::Error;
                }
                ExitCode::Success
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                output.error(format!("Volume failed: {}", stderr), ExitCode::Error)
            }
        }
        Err(e) => output.error(format!("Volume failed: {}", e), ExitCode::Error),
    }
}
