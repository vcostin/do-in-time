use sqlx::{sqlite::SqlitePool, Row};
use crate::error::Result;

pub const SCHEMA_VERSION: i32 = 2;

pub async fn initialize_schema(pool: &SqlitePool) -> Result<()> {
    // Create schema_version table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Check current version
    let current_version: i32 = sqlx::query("SELECT COALESCE(MAX(version), 0) as version FROM schema_version")
        .fetch_one(pool)
        .await
        .map(|row| row.get("version"))
        .unwrap_or(0);

    if current_version == 0 {
        create_schema(pool).await?;
    }

    if current_version < 2 {
        migrate_to_v2(pool).await?;
    }

    Ok(())
}

async fn create_schema(pool: &SqlitePool) -> Result<()> {
    // Create tasks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            browser TEXT NOT NULL,
            browser_profile TEXT,
            url TEXT,
            allow_close_all INTEGER NOT NULL DEFAULT 0,
            start_time TEXT NOT NULL,
            close_time TEXT,
            timezone TEXT NOT NULL,
            repeat_interval TEXT,
            repeat_end_after INTEGER,
            repeat_end_date TEXT,
            status TEXT NOT NULL CHECK(status IN ('pending', 'active', 'completed', 'failed', 'disabled')),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_open_execution TEXT,
            last_close_execution TEXT,
            next_open_execution TEXT,
            next_close_execution TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for tasks table
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_tasks_next_open_execution
        ON tasks(next_open_execution)
        WHERE status = 'active' AND next_open_execution IS NOT NULL
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_tasks_next_close_execution
        ON tasks(next_close_execution)
        WHERE status = 'active' AND next_close_execution IS NOT NULL
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_tasks_status
        ON tasks(status)
        "#,
    )
    .execute(pool)
    .await?;

    // Create task_executions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS task_executions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            executed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            action TEXT NOT NULL CHECK(action IN ('open', 'close')),
            status TEXT NOT NULL CHECK(status IN ('success', 'failed')),
            error_message TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index for task_executions table
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_task_executions_task_id
        ON task_executions(task_id)
        "#,
    )
    .execute(pool)
    .await?;

    // Mark schema as initialized
    sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (?)")
        .bind(SCHEMA_VERSION)
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_to_v2(pool: &SqlitePool) -> Result<()> {
    // Add explicit permission flag for dangerous close-all behavior.
    let _ = sqlx::query(
        r#"
        ALTER TABLE tasks ADD COLUMN allow_close_all INTEGER NOT NULL DEFAULT 0
        "#,
    )
    .execute(pool)
    .await;

    sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (2)")
        .execute(pool)
        .await?;

    Ok(())
}
