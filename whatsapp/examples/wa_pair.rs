//! Interactive example: pairs via phone code.
//!
//! Run with: `cargo run -p mailbox-whatsapp --example wa_pair`.
//!
//! Enter the code on your phone: `WhatsApp` > Linked
//! Devices > Link a Device > Link with Phone Number.

#![allow(clippy::restriction)]

use std::io::{self, Write};

use mailbox_whatsapp as wa;

fn main() {
    eprint!("Enter your phone number (with country code, no +): ");
    io::stderr().flush().unwrap();
    let mut phone_buf = String::new();
    io::stdin().read_line(&mut phone_buf).expect("failed to read line");
    let phone = phone_buf.trim().to_owned();

    match wa::pair_phone(&phone) {
        Ok(code) => {
            eprintln!();
            eprintln!("Enter this code on your phone:");
            eprintln!();
            eprintln!("  {code}");
            eprintln!();
            eprintln!(
                "(WhatsApp > Linked Devices > Link a Device > Link with Phone \
                 Number)"
            );
        }
        Err(err) => {
            eprintln!("pair_phone error: {err}");
        }
    }
}
