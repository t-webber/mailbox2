//! Main application binary that runs everything that is needed.

use mailbox_gui::GuiApp;

/// Runs the CLI application.
#[expect(
    clippy::unwrap_used,
    reason = "if it reaches here, it is unrecoverable"
)]
fn main() {
    GuiApp::run().unwrap();
}
