use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::models::{Host, Config, HostWithConfig};
use chrono::Utc;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            database_url.parse::<sqlx::sqlite::SqliteConnectOptions>()?
                .create_if_missing(true)
        )
        .await?;

    // Create configs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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

    // Create hosts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hosts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hostname TEXT NOT NULL UNIQUE,
            config_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (config_id) REFERENCES configs(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_hostname ON hosts(hostname)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_config_id ON hosts(config_id)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_username ON configs(username)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

// Config operations
pub async fn create_config(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO configs (username, password) VALUES (?, ?)"
    )
    .bind(username)
    .bind(password)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_config_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Config>, sqlx::Error> {
    sqlx::query_as::<_, Config>("SELECT * FROM configs WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_config_by_username(pool: &SqlitePool, username: &str) -> Result<Option<Config>, sqlx::Error> {
    sqlx::query_as::<_, Config>("SELECT * FROM configs WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_configs(pool: &SqlitePool) -> Result<Vec<Config>, sqlx::Error> {
    sqlx::query_as::<_, Config>("SELECT * FROM configs ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn update_config_ip(
    pool: &SqlitePool,
    username: &str,
    ip: &str,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE configs SET current_ip = ?, last_update = ? WHERE username = ?")
        .bind(ip)
        .bind(now)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_config(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM configs WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// Host operations
pub async fn create_host(
    pool: &SqlitePool,
    hostname: &str,
    config_id: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO hosts (hostname, config_id) VALUES (?, ?)"
    )
    .bind(hostname)
    .bind(config_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_all_hosts(pool: &SqlitePool) -> Result<Vec<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_all_hosts_with_config(pool: &SqlitePool) -> Result<Vec<HostWithConfig>, sqlx::Error> {
    sqlx::query_as::<_, HostWithConfig>(
        r#"
        SELECT 
            h.id,
            h.hostname,
            h.config_id,
            c.username,
            c.password,
            c.current_ip,
            c.last_update,
            h.created_at
        FROM hosts h
        JOIN configs c ON h.config_id = c.id
        ORDER BY c.created_at DESC, h.created_at ASC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_hosts_by_config_id(pool: &SqlitePool, config_id: i64) -> Result<Vec<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts WHERE config_id = ?")
        .bind(config_id)
        .fetch_all(pool)
        .await
}

pub async fn get_host_by_hostname(pool: &SqlitePool, hostname: &str) -> Result<Option<Host>, sqlx::Error> {
    sqlx::query_as::<_, Host>("SELECT * FROM hosts WHERE hostname = ?")
        .bind(hostname)
        .fetch_optional(pool)
        .await
}

pub async fn get_host_with_config_by_hostname(pool: &SqlitePool, hostname: &str) -> Result<Option<HostWithConfig>, sqlx::Error> {
    sqlx::query_as::<_, HostWithConfig>(
        r#"
        SELECT 
            h.id,
            h.hostname,
            h.config_id,
            c.username,
            c.password,
            c.current_ip,
            c.last_update,
            h.created_at
        FROM hosts h
        JOIN configs c ON h.config_id = c.id
        WHERE h.hostname = ?
        "#
    )
    .bind(hostname)
    .fetch_optional(pool)
    .await
}

pub async fn delete_host(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hosts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
