//! StreamTUI - Cyberpunk TUI for streaming to Chromecast
//!
//! A neon-soaked terminal interface for searching content, selecting quality,
//! and casting to your TV. Simple. Fast. Beautiful.
//!
//! # Usage
//!
//! ```bash
//! # Launch interactive TUI
//! streamtui
//!
//! # CLI mode (for automation)
//! streamtui search "blade runner"
//! streamtui cast tt1856101 --device "Living Room TV"
//! streamtui status --json
//! ```

mod app;
mod cli;
mod models;

mod api;
mod stream;
mod ui;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command, ExitCode, Output};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.is_cli_mode() {
        // CLI mode: execute command and exit
        let exit_code = run_cli(cli).await;
        std::process::exit(exit_code.into());
    } else {
        // TUI mode: launch interactive interface
        run_tui().await
    }
}

/// Run CLI command and return exit code
async fn run_cli(cli: Cli) -> ExitCode {
    let output = Output::new(&cli);

    match cli.command {
        Some(Command::Search(cmd)) => {
            output.info(format!("Searching for: {}", cmd.query));
            // TODO: Implement search
            output.error("Search not yet implemented", ExitCode::Error)
        }

        Some(Command::Trending(cmd)) => {
            output.info(format!("Fetching trending ({:?})...", cmd.window));
            // TODO: Implement trending
            output.error("Trending not yet implemented", ExitCode::Error)
        }

        Some(Command::Info(cmd)) => {
            output.info(format!("Getting info for: {}", cmd.id));
            // TODO: Implement info
            output.error("Info not yet implemented", ExitCode::Error)
        }

        Some(Command::Streams(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            output.info(format!("Finding streams for: {}", cmd.imdb_id));
            // TODO: Implement streams
            output.error("Streams not yet implemented", ExitCode::Error)
        }

        Some(Command::Subtitles(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            output.info(format!(
                "Searching subtitles for: {} ({})",
                cmd.imdb_id,
                cmd.lang
            ));
            // TODO: Implement subtitles
            output.error("Subtitles not yet implemented", ExitCode::Error)
        }

        Some(Command::Devices(_cmd)) => {
            output.info("Scanning for Chromecast devices...");
            // TODO: Implement device discovery
            output.error("Devices not yet implemented", ExitCode::Error)
        }

        Some(Command::Cast(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            let device = cmd.effective_device(&cli.device);
            match device {
                Some(d) => output.info(format!("Casting {} to {}", cmd.imdb_id, d)),
                None => {
                    return output.error(
                        "No device specified. Use --device or set default in config.",
                        ExitCode::DeviceNotFound,
                    )
                }
            }
            // TODO: Implement cast
            output.error("Cast not yet implemented", ExitCode::Error)
        }

        Some(Command::Status(_cmd)) => {
            // Return idle status for now
            let status = cli::PlaybackStatus::default();
            if output.print_json(&status).is_err() {
                return ExitCode::Error;
            }
            ExitCode::Success
        }

        Some(Command::Play(_)) => {
            output.info("Resuming playback...");
            // TODO: Implement play
            output.error("Play not yet implemented", ExitCode::Error)
        }

        Some(Command::Pause(_)) => {
            output.info("Pausing playback...");
            // TODO: Implement pause
            output.error("Pause not yet implemented", ExitCode::Error)
        }

        Some(Command::Stop(cmd)) => {
            output.info("Stopping playback...");
            if cmd.kill_stream {
                output.info("Also killing torrent stream...");
            }
            // TODO: Implement stop
            output.error("Stop not yet implemented", ExitCode::Error)
        }

        Some(Command::Seek(cmd)) => {
            match cmd.parse_position() {
                cli::SeekPosition::Absolute(secs) => {
                    output.info(format!("Seeking to {}s", secs));
                }
                cli::SeekPosition::Forward(secs) => {
                    output.info(format!("Seeking forward {}s", secs));
                }
                cli::SeekPosition::Backward(secs) => {
                    output.info(format!("Seeking backward {}s", secs));
                }
                cli::SeekPosition::Invalid(s) => {
                    return output.error(
                        format!("Invalid seek position: {}", s),
                        ExitCode::InvalidArgs,
                    );
                }
            }
            // TODO: Implement seek
            output.error("Seek not yet implemented", ExitCode::Error)
        }

        Some(Command::Volume(cmd)) => {
            match cmd.parse_level() {
                cli::VolumeLevel::Absolute(vol) => {
                    output.info(format!("Setting volume to {}%", vol));
                }
                cli::VolumeLevel::Relative(delta) => {
                    if delta >= 0 {
                        output.info(format!("Increasing volume by {}%", delta));
                    } else {
                        output.info(format!("Decreasing volume by {}%", -delta));
                    }
                }
                cli::VolumeLevel::Invalid(s) => {
                    return output.error(
                        format!("Invalid volume level: {}", s),
                        ExitCode::InvalidArgs,
                    );
                }
            }
            // TODO: Implement volume
            output.error("Volume not yet implemented", ExitCode::Error)
        }

        None => {
            // This shouldn't happen (handled by is_cli_mode check)
            ExitCode::Success
        }
    }
}

/// Run interactive TUI
async fn run_tui() -> Result<()> {
    // TODO: Launch TUI
    println!("StreamTUI - Interactive mode coming soon...");
    println!("Use 'streamtui --help' for CLI commands.");
    Ok(())
}
