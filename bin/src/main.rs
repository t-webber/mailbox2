//! Main application binary that runs everything that is needed.

use mailbox_cli::cli;

/// Runs the CLI application.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cli().await
}
