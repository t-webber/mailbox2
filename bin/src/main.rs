//! Main application binary that runs everything that is needed.

use mailbox_cli::cli;
use mailbox_email::EmailProvider;

/// Runs the CLI application.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cli::<EmailProvider>().await
}
