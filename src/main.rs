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
mod commands;
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
    let device = cli.device.as_deref();

    match cli.command {
        Some(Command::Search(cmd)) => {
            commands::search_cmd(cmd, &output).await
        }

        Some(Command::Trending(cmd)) => {
            commands::trending_cmd(cmd, &output).await
        }

        Some(Command::Info(cmd)) => {
            commands::info_cmd(cmd, &output).await
        }

        Some(Command::Streams(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::streams_cmd(cmd, &output).await
        }

        Some(Command::Subtitles(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::subtitles_cmd(cmd, &output).await
        }

        Some(Command::Devices(cmd)) => {
            commands::devices_cmd(cmd, &output).await
        }

        Some(Command::Cast(cmd)) => {
            if let Err(e) = cli::validate_imdb_id(&cmd.imdb_id) {
                return output.error(e, ExitCode::InvalidArgs);
            }
            commands::cast_cmd(cmd, device, &output).await
        }

        Some(Command::Status(cmd)) => {
            commands::status_cmd(cmd, device, &output).await
        }

        Some(Command::Play(cmd)) => {
            commands::play_cmd(cmd, device, &output).await
        }

        Some(Command::Pause(cmd)) => {
            commands::pause_cmd(cmd, device, &output).await
        }

        Some(Command::Stop(cmd)) => {
            commands::stop_cmd(cmd, device, &output).await
        }

        Some(Command::Seek(cmd)) => {
            commands::seek_cmd(cmd, device, &output).await
        }

        Some(Command::Volume(cmd)) => {
            commands::volume_cmd(cmd, device, &output).await
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
