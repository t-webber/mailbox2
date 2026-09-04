//! Interactive example: pairs via phone code.
//!
//! Run with: `cargo run -p mailbox-whatsapp --example wa_pair`.
//!
//! Enter the code on your phone: `WhatsApp` > Linked
//! Devices > Link a Device > Link with Phone Number.

#![allow(clippy::restriction)]

use core::time::Duration;
use std::io::{self, Write};
use std::thread::sleep;

use mailbox_whatsapp::ffi::{initialise_client, needs_pairing, pair_phone};

fn main() {
    eprintln!("Connecting client...");
    initialise_client().unwrap();
    eprintln!("Checking for existing device...");
    if needs_pairing() {
        eprint!("Enter phone number (w/ contry code, no +, no 0): ");
        io::stderr().flush().unwrap();
        let mut phone_buf = String::new();
        io::stdin().read_line(&mut phone_buf).expect("failed to read line");

        pair_phone(&phone_buf).unwrap();
    } else {
        eprintln!("Already paired!");
    }

    sleep(Duration::from_secs(10000));
}
