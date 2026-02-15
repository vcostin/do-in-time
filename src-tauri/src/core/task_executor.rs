use std::sync::Arc;
use chrono::{Datelike, Duration, Timelike, TimeZone, Utc};
use chrono_tz::Tz;
use crate::core::browser_launcher::BrowserLauncher;
use crate::db::{Database, ExecutionAction, ExecutionStatus, RepeatInterval, Task, TaskStatus};
use crate::error::Result;
use tauri::{AppHandle, Emitter};

pub struct TaskExecutor {
    browser_launcher: BrowserLauncher,
    db: Arc<Database>,
    app_handle: AppHandle,
}

impl TaskExecutor {
    pub fn new(db: Arc<Database>, app_handle: AppHandle) -> Self {
        Self {
            browser_launcher: BrowserLauncher::new(),
            db,
            app_handle,
        }
    }

    pub async fn execute(&self, mut task: Task, action: ExecutionAction) -> Result<()> {
        let task_id = task.id.expect("Task must have an ID");

        // Execute the browser action
        let result = match action {
            ExecutionAction::Open => {
                // Open browser (will use existing browser session, preserving login state)
                self.browser_launcher
                    .open_browser(
                        &task.browser,
                        task.url.as_deref(),
                        task.browser_profile.as_deref(),
                    )
                    .await
                    .map(|_| ()) // Ignore PID since we're using URL-based closing
            }
            ExecutionAction::Close => {
                // Close tabs matching the task's URL
                if let Some(url) = &task.url {
                    // Use URL-based closing (matches macOS AppleScript approach)
                    self.browser_launcher
                        .close_browser_by_url(&task.browser, url)
                        .await
                } else {
                    // No URL specified, close all browser instances
                    self.browser_launcher
                        .close_browser(&task.browser)
                        .await
                }
            }
        };

        // Update task record based on execution result
        match result {
            Ok(_) => {
                // Log successful execution
                self.db
                    .log_execution(task_id, action.clone(), ExecutionStatus::Success, None)
                    .await?;

                let now = Utc::now();

                // Update last execution time for this action
                match action {
                    ExecutionAction::Open => task.last_open_execution = Some(now),
                    ExecutionAction::Close => task.last_close_execution = Some(now),
                }

                // Handle repeat logic
                if let Some(repeat_config) = &task.repeat_config {
                    // Calculate next execution times based on action
                    match action {
                        ExecutionAction::Open => {
                            let next = self.calculate_next_execution(&task, task.start_time)?;

                            // Check if we should continue repeating
                            let should_continue = self.should_continue_repeating(task_id, &task, next, repeat_config).await?;

                            if should_continue {
                                task.next_open_execution = Some(next);
                                // Also update next_close_execution if close_time is set
                                if let Some(close_time) = task.close_time {
                                    let time_diff = close_time.signed_duration_since(task.start_time);
                                    task.next_close_execution = Some(next + time_diff);
                                }
                                task.status = TaskStatus::Active;
                            } else {
                                task.next_open_execution = None;
                                task.next_close_execution = None;
                                task.status = TaskStatus::Completed;
                            }
                        }
                        ExecutionAction::Close => {
                            // After close, just clear the close execution
                            // The next_open_execution should already be set from the open action
                            task.next_close_execution = None;
                        }
                    }
                } else {
                    // One-time task - clear the execution that just happened
                    match action {
                        ExecutionAction::Open => {
                            task.next_open_execution = None;
                            // If there's no close time, mark as completed
                            if task.close_time.is_none() {
                                task.status = TaskStatus::Completed;
                            }
                        }
                        ExecutionAction::Close => {
                            task.next_close_execution = None;
                            // One-time task with close - now completed
                            task.status = TaskStatus::Completed;
                        }
                    }
                }

                // Update task in database
                self.db.update_task(task_id, task).await?;

                // Emit event to notify frontend of task status change
                let _ = self.app_handle.emit("task-updated", task_id);

                Ok(())
            }
            Err(e) => {
                // Log failed execution
                let error_msg = e.to_string();
                self.db
                    .log_execution(task_id, action, ExecutionStatus::Failed, Some(error_msg.clone()))
                    .await?;

                // Update task status to failed
                task.status = TaskStatus::Failed;
                self.db.update_task(task_id, task).await?;

                // Emit event to notify frontend of task status change
                let _ = self.app_handle.emit("task-updated", task_id);

                Err(e)
            }
        }
    }

    async fn should_continue_repeating(
        &self,
        task_id: i64,
        _task: &Task,
        next: chrono::DateTime<Utc>,
        repeat_config: &crate::db::RepeatConfig,
    ) -> Result<bool> {
        match (&repeat_config.end_after, &repeat_config.end_date) {
            (Some(count), _) => {
                // Count open executions for this task
                let executions = self.db.get_task_executions(task_id).await?;
                let open_count = executions.iter().filter(|e| e.action == ExecutionAction::Open).count();
                Ok((open_count as i32) < *count)
            }
            (None, Some(end_date)) => Ok(next < *end_date),
            (None, None) => Ok(true),
        }
    }

    fn calculate_next_execution(&self, task: &Task, base_time: chrono::DateTime<Utc>) -> Result<chrono::DateTime<Utc>> {
        let repeat_config = task
            .repeat_config
            .as_ref()
            .expect("Task must have repeat config");

        // Parse timezone
        let tz: Tz = task
            .timezone
            .parse()
            .map_err(|_| crate::error::AppError::TimeParse(format!("Invalid timezone: {}", task.timezone)))?;

        // Convert base time to task's timezone
        let local_time = base_time.with_timezone(&tz);

        // Calculate next occurrence based on interval
        let next_local = match repeat_config.interval {
            RepeatInterval::Daily => local_time + Duration::days(1),
            RepeatInterval::Weekly => local_time + Duration::weeks(1),
            RepeatInterval::Monthly => {
                // Add one month, handling month overflow
                let month = local_time.month();
                let year = local_time.year();

                let (next_month, next_year) = if month == 12 {
                    (1, year + 1)
                } else {
                    (month + 1, year)
                };

                // Find the last day of next month
                let last_day_of_month = chrono::NaiveDate::from_ymd_opt(
                    next_year,
                    next_month + 1,
                    1,
                )
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(next_year + 1, 1, 1).unwrap())
                .pred_opt()
                .unwrap()
                .day();

                let day = local_time.day().min(last_day_of_month);

                let next_date = chrono::NaiveDate::from_ymd_opt(next_year, next_month, day)
                    .ok_or_else(|| crate::error::AppError::TimeParse("Failed to calculate next month".to_string()))?;

                let next_datetime = next_date
                    .and_hms_opt(local_time.hour(), local_time.minute(), local_time.second())
                    .ok_or_else(|| crate::error::AppError::TimeParse("Failed to create next datetime".to_string()))?;

                tz.from_local_datetime(&next_datetime)
                    .single()
                    .ok_or_else(|| crate::error::AppError::TimeParse("Ambiguous local time".to_string()))?
            }
        };

        // Convert back to UTC
        Ok(next_local.with_timezone(&Utc))
    }
}
