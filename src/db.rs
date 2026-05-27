use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::models::Host;
use chrono::Utc;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            database_url.parse::<sqlx::sqlite::SqliteConnectOptions>()?
                .create_if_missing(true)
        )
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hosts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hostname TEXT NOT NULL UNIQUE,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            current_ip TEXT,
            last_update DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_hostname ON hosts(hostname)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_username ON hosts(username)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn create_host(
    pool: &SqlitePool,
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO hosts (hostname, username, password) VALUES (?, ?, ?)"
    )
    .bind(hostname)
    .bind(username)
    .bind(password)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_all_hosts(pool: &SqlitePool) -> Result<Vec<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_host_by_username(pool: &SqlitePool, username: &str) -> Result<Option<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_host_by_hostname(pool: &SqlitePool, hostname: &str) -> Result<Option<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts WHERE hostname = ?")
        .bind(hostname)
        .fetch_optional(pool)
        .await
}

pub async fn update_host_ip(
    pool: &SqlitePool,
    username: &str,
    ip: &str,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE hosts SET current_ip = ?, last_update = ? WHERE username = ?")
        .bind(ip)
        .bind(now)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_host(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hosts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
