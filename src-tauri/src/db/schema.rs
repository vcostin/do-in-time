use crate::error::Result;
use sqlx::sqlite::SqlitePool;

pub async fn initialize_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Fresh install: create current tasks table shape.
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
            status TEXT NOT NULL CHECK(status IN ('active', 'completed', 'failed', 'disabled')),
            next_open_execution TEXT,
            next_close_execution TEXT,
            last_error TEXT,
            last_execution_at TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS task_execution_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            success INTEGER NOT NULL,
            message TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_task_execution_log_task_id
        ON task_execution_log(task_id, created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    ensure_indexes(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT OR IGNORE INTO settings (key, value) VALUES
            ('minimize_to_tray', 'false'),
            ('start_minimized', 'false'),
            ('show_notifications', 'false'),
            ('auto_start', 'false'),
            ('use_24_hour_clock', 'true')
        "#,
    )
    .execute(pool)
    .await?;

    run_migrations(pool).await?;

    Ok(())
}

async fn ensure_indexes(pool: &SqlitePool) -> Result<()> {
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

async fn current_version(pool: &SqlitePool) -> Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

async fn mark_version(pool: &SqlitePool, version: i64) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO schema_migrations (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
}

async fn column_exists(pool: &SqlitePool, table: &str, column: &str) -> Result<bool> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(pool)
        .await?;

    for row in rows {
        let name: String = sqlx::Row::get(&row, "name");
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let mut version = current_version(pool).await?;

    // v1: last_error / last_execution_at + allow disabled status (rebuild CHECK)
    if version < 1 {
        if !column_exists(pool, "tasks", "last_error").await? {
            sqlx::query("ALTER TABLE tasks ADD COLUMN last_error TEXT")
                .execute(pool)
                .await?;
        }
        if !column_exists(pool, "tasks", "last_execution_at").await? {
            sqlx::query("ALTER TABLE tasks ADD COLUMN last_execution_at TEXT")
                .execute(pool)
                .await?;
        }

        // Rebuild so CHECK allows 'disabled' on DBs created before this version.
        rebuild_tasks_table(pool).await?;
        mark_version(pool, 1).await?;
        version = 1;
    }

    let _ = version;
    Ok(())
}

async fn table_exists(pool: &SqlitePool, table: &str) -> Result<bool> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_one(pool)
    .await?;
    Ok(row.0 > 0)
}

async fn rebuild_tasks_table(pool: &SqlitePool) -> Result<()> {
    // Recover / clean leftovers from older pool-based BEGIN/COMMIT (which could
    // run statements on different connections and leave `tasks_new` behind).
    let has_tasks = table_exists(pool, "tasks").await?;
    let has_tasks_new = table_exists(pool, "tasks_new").await?;
    if !has_tasks && has_tasks_new {
        sqlx::query("ALTER TABLE tasks_new RENAME TO tasks")
            .execute(pool)
            .await?;
        ensure_indexes(pool).await?;
        return Ok(());
    }
    if has_tasks_new {
        sqlx::query("DROP TABLE IF EXISTS tasks_new")
            .execute(pool)
            .await?;
    }

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        CREATE TABLE tasks_new (
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
            status TEXT NOT NULL CHECK(status IN ('active', 'completed', 'failed', 'disabled')),
            next_open_execution TEXT,
            next_close_execution TEXT,
            last_error TEXT,
            last_execution_at TEXT
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks_new (
            id, name, browser, browser_profile, url, allow_close_all,
            start_time, close_time, timezone,
            repeat_interval, repeat_end_after, repeat_end_date,
            execution_count, status,
            next_open_execution, next_close_execution,
            last_error, last_execution_at
        )
        SELECT
            id, name, browser, browser_profile, url, allow_close_all,
            start_time, close_time, timezone,
            repeat_interval, repeat_end_after, repeat_end_date,
            execution_count, status,
            next_open_execution, next_close_execution,
            last_error, last_execution_at
        FROM tasks
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query("DROP TABLE tasks").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE tasks_new RENAME TO tasks")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    ensure_indexes(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn recovers_from_leftover_tasks_new() {
        let pool = memory_pool().await;

        sqlx::query(
            r#"
            CREATE TABLE tasks (
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
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO tasks (name, browser, start_time, timezone, status)
             VALUES ('t', 'chrome', '2026-01-01T00:00:00Z', 'UTC', 'active')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Simulate interrupted migration leftover.
        sqlx::query("CREATE TABLE tasks_new (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();

        initialize_schema(&pool).await.expect("schema init should recover");

        assert!(table_exists(&pool, "tasks").await.unwrap());
        assert!(!table_exists(&pool, "tasks_new").await.unwrap());
        assert_eq!(current_version(&pool).await.unwrap(), 1);

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }
}
