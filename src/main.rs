use std::env::var;
use std::path::Path;

use dotenv::dotenv;

use crate::db::setup_db;
use crate::imap::connect_imap;

mod db;
mod imap;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenv()?;
    let _ = setup_db(Path::new("db.sqlite")).await?;
    let _ = connect_imap(
        &var("MBX_DOMAIN")?,
        var("MBX_PORT")?.parse()?,
        &var("MBX_USER")?,
        &var("MBX_PASSWORD")?,
    )
    .await?;
    Ok(())
}
