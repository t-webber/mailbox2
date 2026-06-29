use std::path::Path;

use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

/// Returns a connection pool to the database.
///
/// The sqlite file is created if it doesn't exist.
pub async fn setup_db(db_url: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options =
        SqliteConnectOptions::new().create_if_missing(true).filename(db_url);
    let pool = SqlitePool::connect_with(options).await?;

    sqlx::query(
        "create table if not exists message (
            uid integer primary key,
            subject text not null,
            from_email text not null,
            date text not null,
            body text
         );",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
