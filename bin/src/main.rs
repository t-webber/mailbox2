//! Main application binary that runs everything that is needed.

use std::env::args;

use mailbox_cli::cli;
use mailbox_gui::GuiApp;
use tokio::runtime::Builder;

/// Runs the CLI application.
#[expect(
    clippy::unwrap_used,
    reason = "if it reaches here, it is unrecoverable"
)]
fn main() {
    match args().nth(1) {
        Some(arg) if arg == "cli" => Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { cli().await }),
        _ => GuiApp::run().unwrap(),
    }
}
