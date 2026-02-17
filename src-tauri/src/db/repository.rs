use sqlx::Row;
use crate::db::models::*;
use crate::db::connection::Database;
use crate::error::{AppError, Result};
use crate::utils::validation::{validate_url, validate_browser_profile};
use std::str::FromStr;

impl Database {
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        // Validate inputs for security
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        if task.next_open_execution.is_none() {
            task.next_open_execution = Some(task.start_time);
        }

        if task.close_time.is_some() && task.next_close_execution.is_none() {
            task.next_close_execution = task.close_time;
        }

        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task.repeat_config.as_ref().and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        let result = sqlx::query(
            r#"
            INSERT INTO tasks (
                name, browser, browser_profile, url, allow_close_all,
                start_time, close_time, timezone,
                repeat_interval, repeat_end_after, repeat_end_date,
                execution_count, status,
                next_open_execution, next_close_execution
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task.name)
        .bind(task.browser.to_string())
        .bind(&task.browser_profile)
        .bind(&task.url)
        .bind(task.allow_close_all)
        .bind(task.start_time.to_rfc3339())
        .bind(task.close_time.map(|d| d.to_rfc3339()))
        .bind(&task.timezone)
        .bind(repeat_interval)
        .bind(repeat_end_after)
        .bind(repeat_end_date)
        .bind(task.execution_count)
        .bind(task.status.to_string())
        .bind(task.next_open_execution.map(|d| d.to_rfc3339()))
        .bind(task.next_close_execution.map(|d| d.to_rfc3339()))
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
        let rows = sqlx::query("SELECT * FROM tasks ORDER BY start_time ASC")
            .fetch_all(self.pool())
            .await?;

        rows.into_iter()
            .map(Self::row_to_task)
            .collect()
    }

    pub async fn get_next_action(&self) -> Result<Option<(Task, ExecutionAction)>> {
        // Find the earliest upcoming action (either open or close)
        let row = sqlx::query(
            r#"
            SELECT *,
                CASE
                    WHEN next_open_execution IS NOT NULL AND (next_close_execution IS NULL OR next_open_execution <= next_close_execution)
                        THEN next_open_execution
                    ELSE next_close_execution
                END as next_action_time,
                CASE
                    WHEN next_open_execution IS NOT NULL AND (next_close_execution IS NULL OR next_open_execution <= next_close_execution)
                        THEN 'open'
                    ELSE 'close'
                END as next_action
            FROM tasks
            WHERE status = 'active'
                AND (next_open_execution IS NOT NULL OR next_close_execution IS NOT NULL)
            ORDER BY next_action_time ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(self.pool())
        .await?;

        match row {
            Some(r) => {
                let action_str: String = r.try_get("next_action")?;
                let action = ExecutionAction::from_str(&action_str)
                    .map_err(|e| AppError::InvalidTask(e))?;
                let task = Self::row_to_task(r)?;
                Ok(Some((task, action)))
            }
            None => Ok(None),
        }
    }

    pub async fn update_task(&self, id: i64, mut task: Task) -> Result<Task> {
        // Validate inputs for security
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        // Get old task to check if times have changed
        let old_task = self.get_task(id).await?;

        // Check if times have changed
        let times_changed = old_task.start_time != task.start_time
            || old_task.close_time != task.close_time;

        if times_changed {
            let now = chrono::Utc::now();

            // If task was completed/failed, reactivate it
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                task.status = TaskStatus::Active;
            }

            // Recalculate next execution times based on current time and new scheduled times
            if task.start_time > now {
                task.next_open_execution = Some(task.start_time);
            } else {
                task.next_open_execution = None;
            }

            if let Some(close_time) = task.close_time {
                if close_time > now {
                    task.next_close_execution = Some(close_time);
                } else {
                    task.next_close_execution = None;
                }
            } else {
                task.next_close_execution = None;
            }
        }

        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task.repeat_config.as_ref().and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        sqlx::query(
            r#"
            UPDATE tasks SET
                name = ?, browser = ?, browser_profile = ?, url = ?, allow_close_all = ?,
                start_time = ?, close_time = ?, timezone = ?,
                repeat_interval = ?, repeat_end_after = ?, repeat_end_date = ?,
                execution_count = ?, status = ?,
                next_open_execution = ?, next_close_execution = ?
            WHERE id = ?
            "#,
        )
        .bind(&task.name)
        .bind(task.browser.to_string())
        .bind(&task.browser_profile)
        .bind(&task.url)
        .bind(task.allow_close_all)
        .bind(task.start_time.to_rfc3339())
        .bind(task.close_time.map(|d| d.to_rfc3339()))
        .bind(&task.timezone)
        .bind(repeat_interval)
        .bind(repeat_end_after)
        .bind(repeat_end_date)
        .bind(task.execution_count)
        .bind(task.status.to_string())
        .bind(task.next_open_execution.map(|d| d.to_rfc3339()))
        .bind(task.next_close_execution.map(|d| d.to_rfc3339()))
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
            allow_close_all: row.get("allow_close_all"),
            start_time: row.get::<String, _>("start_time").parse().map_err(|e| AppError::TimeParse(format!("{}", e)))?,
            close_time: row.get::<Option<String>, _>("close_time").and_then(|s| s.parse().ok()),
            timezone: row.get("timezone"),
            repeat_config,
            execution_count: row.get("execution_count"),
            status: TaskStatus::from_str(&row.get::<String, _>("status")).map_err(|e| AppError::InvalidTask(e))?,
            next_open_execution: row.get::<Option<String>, _>("next_open_execution").and_then(|s| s.parse().ok()),
            next_close_execution: row.get::<Option<String>, _>("next_close_execution").and_then(|s| s.parse().ok()),
        })
    }

    pub async fn get_settings(&self) -> Result<AppSettings> {
        let rows = sqlx::query("SELECT key, value FROM settings")
            .fetch_all(self.pool())
            .await?;

        let mut settings = AppSettings::default();

        for row in rows {
            let key: String = row.get("key");
            let value: String = row.get("value");
            let bool_value = value == "true";

            match key.as_str() {
                "minimize_to_tray" => settings.minimize_to_tray = bool_value,
                "start_minimized" => settings.start_minimized = bool_value,
                "show_notifications" => settings.show_notifications = bool_value,
                "auto_start" => settings.auto_start = bool_value,
                _ => {}
            }
        }

        Ok(settings)
    }

    pub async fn update_setting(&self, key: &str, value: bool) -> Result<()> {
        let value_str = if value { "true" } else { "false" };

        sqlx::query("UPDATE settings SET value = ? WHERE key = ?")
            .bind(value_str)
            .bind(key)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    pub async fn update_settings(&self, settings: AppSettings) -> Result<()> {
        self.update_setting("minimize_to_tray", settings.minimize_to_tray).await?;
        self.update_setting("start_minimized", settings.start_minimized).await?;
        self.update_setting("show_notifications", settings.show_notifications).await?;
        self.update_setting("auto_start", settings.auto_start).await?;
        Ok(())
    }
}
