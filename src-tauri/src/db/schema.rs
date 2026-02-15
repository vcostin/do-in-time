use sqlx::{sqlite::SqlitePool, Row};
use crate::error::Result;

pub const SCHEMA_VERSION: i32 = 4;

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
    // Migration 2: Refactored schema with start_time/close_time
    if from_version < 2 {
        // Drop old tables if they exist (clean slate for refactor)
        sqlx::query("DROP TABLE IF EXISTS task_executions")
            .execute(pool)
            .await?;

        sqlx::query("DROP TABLE IF EXISTS tasks")
            .execute(pool)
            .await?;

        // Create new tasks table with start_time/close_time model
        // Note: window_pid was removed in v4 (not needed for URL-based closing)
        sqlx::query(
            r#"
            CREATE TABLE tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                browser TEXT NOT NULL,
                browser_profile TEXT,
                url TEXT,
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

        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_next_open_execution
            ON tasks(next_open_execution)
            WHERE status = 'active' AND next_open_execution IS NOT NULL
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_next_close_execution
            ON tasks(next_close_execution)
            WHERE status = 'active' AND next_close_execution IS NOT NULL
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_status
            ON tasks(status)
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE task_executions (
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

        sqlx::query(
            r#"
            CREATE INDEX idx_task_executions_task_id
            ON task_executions(task_id)
            "#,
        )
        .execute(pool)
        .await?;

        // Mark migration as applied - skip directly to version 4 for fresh installs
        sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (4)")
            .execute(pool)
            .await?;
    }

    // Migration 3: Add window_pid column (deprecated in v4, kept for upgrade path)
    if from_version >= 2 && from_version < 3 {
        sqlx::query("ALTER TABLE tasks ADD COLUMN window_pid INTEGER")
            .execute(pool)
            .await?;

        sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (3)")
            .execute(pool)
            .await?;
    }

    // Migration 4: Remove window_pid column (no longer needed for URL-based closing)
    if from_version >= 3 && from_version < 4 {
        // SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
        // Step 0: Drop tasks_new if it exists from a failed migration
        sqlx::query("DROP TABLE IF EXISTS tasks_new")
            .execute(pool)
            .await?;

        // Step 1: Create new table without window_pid
        sqlx::query(
            r#"
            CREATE TABLE tasks_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                browser TEXT NOT NULL,
                browser_profile TEXT,
                url TEXT,
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

        // Step 2: Copy data (excluding window_pid)
        sqlx::query(
            r#"
            INSERT INTO tasks_new
            SELECT id, name, browser, browser_profile, url, start_time, close_time,
                   timezone, repeat_interval, repeat_end_after, repeat_end_date,
                   status, created_at, updated_at, last_open_execution, last_close_execution,
                   next_open_execution, next_close_execution
            FROM tasks
            "#,
        )
        .execute(pool)
        .await?;

        // Step 3: Drop old table
        sqlx::query("DROP TABLE tasks")
            .execute(pool)
            .await?;

        // Step 4: Rename new table
        sqlx::query("ALTER TABLE tasks_new RENAME TO tasks")
            .execute(pool)
            .await?;

        // Step 5: Recreate indexes
        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_next_open_execution
            ON tasks(next_open_execution)
            WHERE status = 'active' AND next_open_execution IS NOT NULL
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_next_close_execution
            ON tasks(next_close_execution)
            WHERE status = 'active' AND next_close_execution IS NOT NULL
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX idx_tasks_status
            ON tasks(status)
            "#,
        )
        .execute(pool)
        .await?;

        // Mark migration as applied
        sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (4)")
            .execute(pool)
            .await?;
    }

    Ok(())
}
