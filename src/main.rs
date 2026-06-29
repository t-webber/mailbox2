//! Mailbox crate.
//!
//! Crate to ease managing a mailbox, including fetching email bodies, checking
//! for new messages not yet pulled, and sending out new email.

/// Handles database connections.
mod db;
/// Handles interactions with the IMAP protocol.
mod imap;
/// Decodes the encoded subjects.
mod subject_decoder;
#[cfg(test)]
mod test_subject_decoder;

use std::env::var;
use std::path::Path;

use dotenv::dotenv;

use crate::db::setup_db;
use crate::imap::{connect_imap, fetch_headers};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenv()?;
    drop(setup_db(Path::new("db.sqlite")).await?);
    let mut session = connect_imap(
        &var("MBX_DOMAIN")?,
        var("MBX_PORT")?.parse()?,
        &var("MBX_USER")?,
        &var("MBX_PASSWORD")?,
    )
    .await?;
    fetch_headers(&mut session).await?;
    Ok(())
}
