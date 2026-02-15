use sqlx::{sqlite::SqlitePool, Row};
use crate::error::Result;

pub const SCHEMA_VERSION: i32 = 1;

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

    if current_version < SCHEMA_VERSION {
        apply_migrations(pool, current_version).await?;
    }

    Ok(())
}

async fn apply_migrations(pool: &SqlitePool, from_version: i32) -> Result<()> {
    // Migration 1: Initial schema
    if from_version < 1 {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                browser TEXT NOT NULL,
                browser_profile TEXT,
                url TEXT,
                action TEXT NOT NULL CHECK(action IN ('open', 'close')),
                scheduled_time TEXT NOT NULL,
                timezone TEXT NOT NULL,
                repeat_interval TEXT,
                repeat_end_after INTEGER,
                repeat_end_date TEXT,
                status TEXT NOT NULL CHECK(status IN ('pending', 'active', 'completed', 'failed', 'disabled')),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_executed TEXT,
                next_execution TEXT
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_tasks_next_execution
            ON tasks(next_execution)
            WHERE status = 'active'
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS task_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                executed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL CHECK(status IN ('success', 'failed')),
                error_message TEXT,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_task_executions_task_id
            ON task_executions(task_id)
            "#,
        )
        .execute(pool)
        .await?;

        // Mark migration as applied
        sqlx::query("INSERT INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
    }

    Ok(())
}
