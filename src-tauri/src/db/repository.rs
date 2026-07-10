use crate::db::connection::Database;
use crate::db::models::*;
use crate::error::{AppError, Result};
use crate::utils::schedule::schedule_next_open_close;
use crate::utils::validation::{validate_browser_profile, validate_url};
use chrono::Utc;
use sqlx::Row;
use std::str::FromStr;

impl Database {
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        schedule_next_open_close(&mut task, Utc::now())?;
        // Do not force Active over Completed (past one-shots) or Disabled.
        if task.status != TaskStatus::Disabled && task.status != TaskStatus::Completed {
            task.status = TaskStatus::Active;
        }

        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task
            .repeat_config
            .as_ref()
            .and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        let result = sqlx::query(
            r#"
            INSERT INTO tasks (
                name, browser, browser_profile, url, allow_close_all,
                start_time, close_time, timezone,
                repeat_interval, repeat_end_after, repeat_end_date,
                execution_count, status,
                next_open_execution, next_close_execution,
                last_error, last_execution_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(&task.last_error)
        .bind(task.last_execution_at.map(|d| d.to_rfc3339()))
        .execute(self.pool())
        .await?;

        task.id = Some(result.last_insert_rowid());
        Ok(task)
    }

    pub async fn get_task(&self, id: i64) -> Result<Task> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ?")
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

        rows.into_iter().map(Self::row_to_task).collect()
    }

    pub async fn get_next_action(&self) -> Result<Option<(Task, ExecutionAction)>> {
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
                let action =
                    ExecutionAction::from_str(&action_str).map_err(AppError::InvalidTask)?;
                let task = Self::row_to_task(r)?;
                Ok(Some((task, action)))
            }
            None => Ok(None),
        }
    }

    pub async fn update_task(&self, id: i64, mut task: Task) -> Result<Task> {
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        let old_task = self.get_task(id).await?;

        let times_changed =
            old_task.start_time != task.start_time || old_task.close_time != task.close_time;

        if times_changed {
            let now = Utc::now();

            if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                task.status = TaskStatus::Active;
                task.last_error = None;
            }

            if task.status == TaskStatus::Active {
                schedule_next_open_close(&mut task, now)?;
            }
        }

        self.write_task_row(id, &task).await?;
        task.id = Some(id);
        Ok(task)
    }

    /// Persist scheduler/runtime fields without re-deriving schedule from form edits.
    pub async fn save_task_runtime(&self, task: &Task) -> Result<()> {
        let id = task.id.ok_or_else(|| {
            AppError::InvalidTask("Task must have an ID to save runtime state".to_string())
        })?;
        self.write_task_row(id, task).await
    }

    async fn write_task_row(&self, id: i64, task: &Task) -> Result<()> {
        let repeat_interval = task.repeat_config.as_ref().map(|r| r.interval.to_string());
        let repeat_end_after = task.repeat_config.as_ref().and_then(|r| r.end_after);
        let repeat_end_date = task
            .repeat_config
            .as_ref()
            .and_then(|r| r.end_date.map(|d| d.to_rfc3339()));

        sqlx::query(
            r#"
            UPDATE tasks SET
                name = ?, browser = ?, browser_profile = ?, url = ?, allow_close_all = ?,
                start_time = ?, close_time = ?, timezone = ?,
                repeat_interval = ?, repeat_end_after = ?, repeat_end_date = ?,
                execution_count = ?, status = ?,
                next_open_execution = ?, next_close_execution = ?,
                last_error = ?, last_execution_at = ?
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
        .bind(&task.last_error)
        .bind(task.last_execution_at.map(|d| d.to_rfc3339()))
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn set_task_paused(&self, id: i64, paused: bool) -> Result<Task> {
        let mut task = self.get_task(id).await?;

        if paused {
            if task.status != TaskStatus::Active && task.status != TaskStatus::Failed {
                return Err(AppError::InvalidTask(
                    "Only active or failed tasks can be paused".to_string(),
                ));
            }
            task.status = TaskStatus::Disabled;
        } else {
            if task.status != TaskStatus::Disabled && task.status != TaskStatus::Failed {
                return Err(AppError::InvalidTask(
                    "Only disabled or failed tasks can be resumed".to_string(),
                ));
            }
            task.status = TaskStatus::Active;
            task.last_error = None;
            schedule_next_open_close(&mut task, Utc::now())?;
        }

        self.write_task_row(id, &task).await?;
        task.id = Some(id);
        Ok(task)
    }

    pub async fn add_execution_log(
        &self,
        task_id: i64,
        action: &ExecutionAction,
        success: bool,
        message: Option<&str>,
    ) -> Result<()> {
        let msg = message.map(|m| {
            let mut s = m.trim().to_string();
            if s.len() > 500 {
                s.truncate(500);
                s.push('…');
            }
            s
        });

        sqlx::query(
            r#"
            INSERT INTO task_execution_log (task_id, action, success, message, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(task_id)
        .bind(action.to_string())
        .bind(success)
        .bind(msg)
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_task_execution_log(
        &self,
        task_id: i64,
        limit: i64,
    ) -> Result<Vec<TaskExecutionLogEntry>> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, action, success, message, created_at
            FROM task_execution_log
            WHERE task_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskExecutionLogEntry {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    action: row.get("action"),
                    success: row.get("success"),
                    message: row.get("message"),
                    created_at: row
                        .get::<String, _>("created_at")
                        .parse()
                        .map_err(|e| AppError::TimeParse(format!("{}", e)))?,
                })
            })
            .collect()
    }

    pub async fn get_recent_execution_log(
        &self,
        limit: i64,
        failures_only: bool,
    ) -> Result<Vec<RecentExecutionLogEntry>> {
        let limit = limit.clamp(1, 200);
        let rows = if failures_only {
            sqlx::query(
                r#"
                SELECT l.id, l.task_id, t.name AS task_name, l.action, l.success, l.message, l.created_at
                FROM task_execution_log l
                INNER JOIN tasks t ON t.id = l.task_id
                WHERE l.success = 0
                ORDER BY l.created_at DESC, l.id DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT l.id, l.task_id, t.name AS task_name, l.action, l.success, l.message, l.created_at
                FROM task_execution_log l
                INNER JOIN tasks t ON t.id = l.task_id
                ORDER BY l.created_at DESC, l.id DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        };

        rows.into_iter()
            .map(|row| {
                Ok(RecentExecutionLogEntry {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    task_name: row.get("task_name"),
                    action: row.get("action"),
                    success: row.get("success"),
                    message: row.get("message"),
                    created_at: row
                        .get::<String, _>("created_at")
                        .parse()
                        .map_err(|e| AppError::TimeParse(format!("{}", e)))?,
                })
            })
            .collect()
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    fn row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<Task> {
        let repeat_config = if let Some(interval_str) =
            row.get::<Option<String>, _>("repeat_interval")
        {
            Some(RepeatConfig {
                interval: RepeatInterval::from_str(&interval_str).map_err(AppError::InvalidTask)?,
                end_after: row.get("repeat_end_after"),
                end_date: row
                    .get::<Option<String>, _>("repeat_end_date")
                    .and_then(|s| s.parse().ok()),
            })
        } else {
            None
        };

        Ok(Task {
            id: Some(row.get("id")),
            name: row.get("name"),
            browser: BrowserType::from_str(&row.get::<String, _>("browser"))
                .map_err(AppError::InvalidTask)?,
            browser_profile: row.get("browser_profile"),
            url: row.get("url"),
            allow_close_all: row.get("allow_close_all"),
            start_time: row
                .get::<String, _>("start_time")
                .parse()
                .map_err(|e| AppError::TimeParse(format!("{}", e)))?,
            close_time: row
                .get::<Option<String>, _>("close_time")
                .and_then(|s| s.parse().ok()),
            timezone: row.get("timezone"),
            repeat_config,
            execution_count: row.get("execution_count"),
            status: TaskStatus::from_str(&row.get::<String, _>("status"))
                .map_err(AppError::InvalidTask)?,
            next_open_execution: row
                .get::<Option<String>, _>("next_open_execution")
                .and_then(|s| s.parse().ok()),
            next_close_execution: row
                .get::<Option<String>, _>("next_close_execution")
                .and_then(|s| s.parse().ok()),
            last_error: row.get("last_error"),
            last_execution_at: row
                .get::<Option<String>, _>("last_execution_at")
                .and_then(|s| s.parse().ok()),
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
        self.update_setting("minimize_to_tray", settings.minimize_to_tray)
            .await?;
        self.update_setting("start_minimized", settings.start_minimized)
            .await?;
        self.update_setting("show_notifications", settings.show_notifications)
            .await?;
        self.update_setting("auto_start", settings.auto_start)
            .await?;
        Ok(())
    }
}
