//! Main application binary that runs everything that is needed.

use std::env::args;

use color_eyre::Result;
use mailbox_cli::cli;
use mailbox_gui::GuiApp;

/// Runs the CLI application.
fn main() -> Result<()> {
    color_eyre::install()?;
    match args().nth(1) {
        Some(arg) if arg == "cli" => tokio_cli(),
        _ => GuiApp::run(),
    }
}

#[tokio::main]
async fn tokio_cli() -> Result<()> {
    cli().await
}
