//! StreamTUI - Cyberpunk TUI for streaming to Chromecast
//!
//! A neon-soaked terminal interface for searching content, selecting quality,
//! and casting to your TV. Simple. Fast. Beautiful.

mod app;
mod models;

mod api;
mod stream;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Route to TUI or CLI based on args
    println!("StreamTUI - Coming soon...");
    Ok(())
}
