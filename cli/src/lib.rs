//! Cli interface to the mailbox app.

use std::io::{Write as _, stdin, stdout};

use color_eyre::Result;
use mailbox_email::EmailProvider;
use mailbox_shared::{Config, EmailConfig, Message, Provider as _, Room as _};

/// Prints without a newline and flushes to the console.
#[expect(clippy::print_stdout, reason = "cli")]
fn print_flush(msg: &str) -> Result<()> {
    print!("{msg}");
    Ok(stdout().flush()?)
}

/// Displays a prompt, inputs and returns the inputted text.
fn prompt(msg: &str) -> Result<String> {
    print_flush(msg)?;
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_owned())
}

/// Runs the CLI application.
///
/// # Errors
///
/// Returns an error if the app encounters an unrecoverable error.
#[expect(clippy::print_stdout, reason = "cli")]
pub async fn cli() -> color_eyre::Result<()> {
    let mut config = Config::load()?;
    let mut provider = if let Some(email) = config.as_first_email_config() {
        print_flush("Authenticating...\r")?;
        EmailProvider::auth(email).await?
    } else {
        println!("No configuration found");
        let email = EmailConfig::new(
            prompt("user (e.g. bob@bob.com): ")?,
            prompt("password: ")?,
            prompt("domain (e.g. imap.google.com): ")?,
            prompt("port (e.g. 993): ")?.parse()?,
        );
        let provider = EmailProvider::auth(&email).await?;
        config.add_email_config(email)?;
        provider
    };
    print_flush("Fetching rooms...\r")?;
    let rooms = provider.get_rooms().await?;
    loop {
        let buf = prompt(
            "\x1b[33mEnter a positive integer to display content of email, \
             negative integer to display the first n emails: \x1b[0m",
        )?;
        let Ok(nb) = buf.parse::<i32>() else {
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
        } else if let Some(room) = rooms.get(index) {
            let headers = room
                .debug()
                .split('\n')
                .map(|line| {
                    let (name, value) =
                        line.split_once(':').unwrap_or(("", line));
                    format!("\x1b[32m{name:10}:\x1b[0m{value}")
                })
                .collect::<Vec<_>>()
                .join("\n");

            println!(
                "\x1b[35m### Headers:\x1b[0m\n{headers}\n\x1b[35m### \
                 Body:\x1b[0m\n{}",
                provider
                    .get_messages(room)
                    .await?
                    .first()
                    .map_or_else(|| "<no first>".to_owned(), Message::debug)
            );
        } else {
            println!("<invalid id>");
        }
    }
}
