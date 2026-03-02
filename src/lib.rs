pub mod db;
pub mod detector;
pub mod screenshot;
pub mod cli;
pub mod api;

pub use db::{Database, Monitor, Change};
pub use detector::{ChangeType, detect_changes};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub screenshot_dir: String,
    pub port: u16,
    pub log_level: String,
    pub webhook_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "./data/driftguard.db".to_string()),
            screenshot_dir: std::env::var("SCREENSHOT_DIR").unwrap_or_else(|_| "./data/screenshots".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            webhook_url: std::env::var("WEBHOOK_URL").ok(),
        }
    }
}
