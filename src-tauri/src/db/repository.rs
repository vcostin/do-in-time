use sqlx::Row;
use chrono::Utc;
use crate::db::models::*;
use crate::db::connection::Database;
use crate::error::{AppError, Result};
use std::str::FromStr;

impl Database {
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        let now = Utc::now();
        task.created_at = now;
        task.updated_at = now;

        if task.next_execution.is_none() {
            task.next_execution = Some(task.scheduled_time);
        }

        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task.repeat_config.as_ref().and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        let result = sqlx::query(
            r#"
            INSERT INTO tasks (
                name, browser, browser_profile, url, action, scheduled_time, timezone,
                repeat_interval, repeat_end_after, repeat_end_date, status,
                created_at, updated_at, last_executed, next_execution
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task.name)
        .bind(task.browser.to_string())
        .bind(&task.browser_profile)
        .bind(&task.url)
        .bind(task.action.to_string())
        .bind(task.scheduled_time.to_rfc3339())
        .bind(&task.timezone)
        .bind(repeat_interval)
        .bind(repeat_end_after)
        .bind(repeat_end_date)
        .bind(task.status.to_string())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(task.last_executed.map(|d| d.to_rfc3339()))
        .bind(task.next_execution.map(|d| d.to_rfc3339()))
        .execute(self.pool())
        .await?;

        task.id = Some(result.last_insert_rowid());
        Ok(task)
    }

    pub async fn get_task(&self, id: i64) -> Result<Task> {
        let row = sqlx::query(
            r#"
            SELECT * FROM tasks WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| AppError::TaskNotFound(id))?;

        Self::row_to_task(row)
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query("SELECT * FROM tasks ORDER BY next_execution ASC")
            .fetch_all(self.pool())
            .await?;

        rows.into_iter()
            .map(Self::row_to_task)
            .collect()
    }

    pub async fn get_next_task(&self) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM tasks
            WHERE status = 'active' AND next_execution IS NOT NULL
            ORDER BY next_execution ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(self.pool())
        .await?;

        match row {
            Some(r) => Self::row_to_task(r).map(Some),
            None => Ok(None),
        }
    }

    pub async fn update_task(&self, id: i64, mut task: Task) -> Result<Task> {
        task.updated_at = Utc::now();

        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task.repeat_config.as_ref().and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        sqlx::query(
            r#"
            UPDATE tasks SET
                name = ?, browser = ?, browser_profile = ?, url = ?, action = ?,
                scheduled_time = ?, timezone = ?, repeat_interval = ?, repeat_end_after = ?,
                repeat_end_date = ?, status = ?, updated_at = ?, last_executed = ?,
                next_execution = ?
            WHERE id = ?
            "#,
        )
        .bind(&task.name)
        .bind(task.browser.to_string())
        .bind(&task.browser_profile)
        .bind(&task.url)
        .bind(task.action.to_string())
        .bind(task.scheduled_time.to_rfc3339())
        .bind(&task.timezone)
        .bind(repeat_interval)
        .bind(repeat_end_after)
        .bind(repeat_end_date)
        .bind(task.status.to_string())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.last_executed.map(|d| d.to_rfc3339()))
        .bind(task.next_execution.map(|d| d.to_rfc3339()))
        .bind(id)
        .execute(self.pool())
        .await?;

        task.id = Some(id);
        Ok(task)
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    pub async fn log_execution(&self, task_id: i64, status: ExecutionStatus, error_message: Option<String>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO task_executions (task_id, executed_at, status, error_message)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(task_id)
        .bind(Utc::now().to_rfc3339())
        .bind(status.to_string())
        .bind(error_message)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_task_executions(&self, task_id: i64) -> Result<Vec<TaskExecution>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM task_executions
            WHERE task_id = ?
            ORDER BY executed_at DESC
            LIMIT 50
            "#,
        )
        .bind(task_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskExecution {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    executed_at: row.get::<String, _>("executed_at").parse().map_err(|e| AppError::TimeParse(format!("{}", e)))?,
                    status: ExecutionStatus::from_str(&row.get::<String, _>("status")).map_err(|e| AppError::InvalidTask(e))?,
                    error_message: row.get("error_message"),
                })
            })
            .collect()
    }

    fn row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<Task> {
        let repeat_config = if let Some(interval_str) = row.get::<Option<String>, _>("repeat_interval") {
            Some(RepeatConfig {
                interval: RepeatInterval::from_str(&interval_str).map_err(|e| AppError::InvalidTask(e))?,
                end_after: row.get("repeat_end_after"),
                end_date: row.get::<Option<String>, _>("repeat_end_date")
                    .and_then(|s| s.parse().ok()),
            })
        } else {
            None
        };

        Ok(Task {
            id: Some(row.get("id")),
            name: row.get("name"),
            browser: BrowserType::from_str(&row.get::<String, _>("browser")).map_err(|e| AppError::InvalidTask(e))?,
            browser_profile: row.get("browser_profile"),
            url: row.get("url"),
            action: TaskAction::from_str(&row.get::<String, _>("action")).map_err(|e| AppError::InvalidTask(e))?,
            scheduled_time: row.get::<String, _>("scheduled_time").parse().map_err(|e| AppError::TimeParse(format!("{}", e)))?,
            timezone: row.get("timezone"),
            repeat_config,
            status: TaskStatus::from_str(&row.get::<String, _>("status")).map_err(|e| AppError::InvalidTask(e))?,
            created_at: row.get::<String, _>("created_at").parse().map_err(|e| AppError::TimeParse(format!("{}", e)))?,
            updated_at: row.get::<String, _>("updated_at").parse().map_err(|e| AppError::TimeParse(format!("{}", e)))?,
            last_executed: row.get::<Option<String>, _>("last_executed").and_then(|s| s.parse().ok()),
            next_execution: row.get::<Option<String>, _>("next_execution").and_then(|s| s.parse().ok()),
        })
    }
}
