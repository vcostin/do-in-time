use sqlx::sqlite::SqlitePool;
use crate::error::Result;

pub async fn initialize_schema(pool: &SqlitePool) -> Result<()> {
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
            execution_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL CHECK(status IN ('active', 'completed', 'failed')),
            next_open_execution TEXT,
            next_close_execution TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for scheduler efficiency
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

    Ok(())
}
