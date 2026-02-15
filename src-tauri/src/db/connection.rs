use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use crate::error::Result;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection_string = format!("sqlite://{}?mode=rwc", db_path.display());

        let options = SqliteConnectOptions::from_str(&connection_string)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Initialize schema
        crate::db::schema::initialize_schema(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn get_db_path() -> Result<std::path::PathBuf> {
        let data_dir = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|h| h.join("Library").join("Application Support"))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        } else {
            // Linux
            dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        };

        Ok(data_dir.join("do-in-time").join("data.db"))
    }
}
