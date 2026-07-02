//! Cli interface to the mailbox app.

use std::io::{Write as _, stdin, stdout};

use color_eyre::Result;
use mailbox_shared::{Provider, Room as _};

/// Prints without a newline and flushes to the console.
#[expect(clippy::print_stdout, reason = "cli")]
fn print_flush(msg: &str) -> Result<()> {
    print!("{msg}");
    Ok(stdout().flush()?)
}

/// Runs the CLI application.
///
/// # Errors
///
/// Returns an error if the app encounters an unrecoverable error.
#[expect(clippy::print_stdout, reason = "cli")]
pub async fn cli<P: Provider>() -> color_eyre::Result<()> {
    print_flush("Authenticating...\r")?;
    let mut provider = P::auth().await?;
    print_flush("Fetching rooms...\r")?;
    let rooms = provider.get_rooms().await?;
    loop {
        print_flush(
            "\x1b[33mEnter a positive integer to display content of email, \
             negative integer to display the first n emails: \x1b[0m",
        )?;
        stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        let Ok(nb) = buf.trim_end().parse::<i32>() else {
            println!("\x1b[31mInvalid number {buf}\x1b[0m");
            continue;
        };
        #[expect(
            clippy::as_conversions,
            clippy::cast_sign_loss,
            clippy::arithmetic_side_effects,
            reason = "checked"
        )]
        let (index, signed) = if nb >= 0i32 {
            (nb as usize, false)
        } else {
            ((-nb) as usize, true)
        };
        if signed {
            for (i, room) in rooms.iter().enumerate().take(index) {
                println!("[{i}] {:30} {}", room.name(), room.overview());
            }
        } else {
            println!(
                "{}",
                rooms.get(index).map_or_else(
                    || "<not found>".to_owned(),
                    |room| {
                        room.debug()
                            .split('\n')
                            .map(|line| {
                                let (name, value) =
                                    line.split_once(':').unwrap_or(("", line));
                                format!("\x1b[32m{name:10}:\x1b[0m{value}")
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                )
            );
        }
    }
}
