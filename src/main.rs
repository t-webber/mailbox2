use std::env::var;
use std::path::Path;

use dotenv::dotenv;

use crate::db::setup_db;
use crate::imap::{connect_imap, fetch_headers};

mod db;
mod imap;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenv()?;
    let _ = setup_db(Path::new("db.sqlite")).await?;
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
