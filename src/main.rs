use std::path::Path;

use crate::db::setup_db;

mod db;

#[tokio::main]
async fn main() {
    let _ = setup_db(Path::new("db.sqlite")).await.unwrap();
}
