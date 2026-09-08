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

use mailbox_whatsapp::ffi::{
    initialise_client, is_synced, needs_pairing, pair_phone
};

macro_rules! print {
    ($x:expr) => {
        eprintln!("\x1b[35m>>>>> {}\x1b[0m", $x)
    };
}

fn main() {
    print!("Connecting client...");
    initialise_client().unwrap();
    print!("Checking for existing device...");
    if needs_pairing() {
        print!("Enter phone number (w/ contry code, no +, no 0): ");
        io::stderr().flush().unwrap();
        let mut phone_buf = String::new();
        io::stdin().read_line(&mut phone_buf).expect("failed to read line");

        print!("Pairing code");
        print!(pair_phone(&phone_buf).unwrap());
    } else {
        print!("Already paired!");
    }

    print!("Waiting for sync...");
    while !is_synced() {
        sleep(Duration::from_secs(1));
    }
    print!("Sync complete!");

    sleep(Duration::from_secs(10000));
}
