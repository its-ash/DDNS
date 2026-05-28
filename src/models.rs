use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Config {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub current_ip: Option<String>,
    pub last_update: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Host {
    pub id: i64,
    pub hostname: String,
    pub config_id: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HostWithConfig {
    pub id: i64,
    pub hostname: String,
    pub config_id: i64,
    pub username: String,
    pub password: String,
    pub current_ip: Option<String>,
    pub last_update: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateHostRequest {
    pub subdomain: String,
    pub config_id: Option<i64>,  // If provided, adds hostname to existing config
}

#[derive(Debug, Deserialize)]
pub struct AddHostnameRequest {
    pub subdomain: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIpRequest {
    pub hostname: Option<String>,
    pub myip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdminPassword(pub String);

#[derive(Debug, Clone)]
pub struct BaseDomain(pub String);
