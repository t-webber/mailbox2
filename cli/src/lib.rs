//! Cli interface to the mailbox app.

#![expect(
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::infinite_loop,
    clippy::use_debug,
    reason = "cli"
)]

use std::io::{Write as _, stdin, stdout};

use mailbox_email::{EmailBody, EmailProvider};
use mailbox_shared::{Config, EmailConfig};

/// Prints without a newline and flushes to the console.
fn print_flush(msg: &str) {
    print!("{msg}");
    stdout().flush().unwrap();
}

/// Displays a prompt, inputs and returns the inputted text.
fn prompt(msg: &str) -> String {
    print_flush(msg);
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().to_owned()
}

/// Runs the CLI application.
///
/// # Panics
///
/// Returns an error if the app encounters an unrecoverable error.
pub async fn cli() {
    let mut config = Config::load().unwrap();
    let mut provider = if let Some(email) = config.as_first_email_config() {
        print_flush("Authenticating...\r");
        EmailProvider::auth(email).await.unwrap()
    } else {
        println!("No configuration found");
        let email = EmailConfig::new(
            prompt("user (e.g. bob@bob.com): ").into(),
            prompt("password: ").into(),
            prompt("domain (e.g. imap.google.com): ").into(),
            prompt("port (e.g. 993): ").parse().unwrap(),
        );
        let provider = EmailProvider::auth(&email).await.unwrap();
        config.add_email_config(email).unwrap();
        provider
    };
    print_flush("Fetching rooms...\r");
    let (rooms, errors) = provider.get_headers().await.unwrap();
    println!("{errors:?}");
    loop {
        let buf = prompt(
            "\x1b[33mEnter a positive integer to display content of email, \
             negative integer to display the first n emails: \x1b[0m",
        );
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
                println!("[{i}] {:30} {}", room.from(), room.subject());
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
                    .get_body(room.uid)
                    .await
                    .unwrap()
                    .first()
                    .map_or_else(|| "<no first>".to_owned(), EmailBody::debug)
            );
        } else {
            println!("<invalid id>");
        }
    }
}
