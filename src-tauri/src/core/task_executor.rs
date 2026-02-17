use std::sync::Arc;
use chrono::{Datelike, Duration, Timelike, TimeZone, Utc};
use chrono_tz::Tz;
use crate::core::browser_launcher::BrowserLauncher;
use crate::db::{Database, ExecutionAction, RepeatInterval, Task, TaskStatus};
use crate::error::Result;
use crate::utils::validation::{validate_browser_profile, validate_url};
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

        // Defense-in-depth: validate inputs again right before any system interaction.
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        // Execute the browser action
        let result = match action {
            ExecutionAction::Open => {
                self.browser_launcher
                    .open_browser(
                        &task.browser,
                        task.url.as_deref(),
                        task.browser_profile.as_deref(),
                    )
                    .await
                    .map(|_| ())
            }
            ExecutionAction::Close => {
                if let Some(url) = &task.url {
                    self.browser_launcher
                        .close_browser_by_url(&task.browser, url)
                        .await
                } else {
                    if task.allow_close_all {
                        self.browser_launcher
                            .close_browser(&task.browser)
                            .await
                    } else {
                        Err(crate::error::AppError::InvalidTask(
                            "Close without URL is blocked unless 'allow_close_all' is enabled for this task"
                                .to_string(),
                        ))
                    }
                }
            }
        };

        // Update task record based on execution result
        match result {
            Ok(_) => {
                // Increment execution count for open actions
                if action == ExecutionAction::Open {
                    task.execution_count += 1;
                }

                // Handle repeat logic
                if let Some(repeat_config) = &task.repeat_config {
                    match action {
                        ExecutionAction::Open => {
                            let next = self.calculate_next_execution(&task, task.start_time)?;

                            let should_continue = self.should_continue_repeating(&task, next, repeat_config);

                            if should_continue {
                                task.next_open_execution = Some(next);
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
                            task.next_close_execution = None;
                        }
                    }
                } else {
                    // One-time task
                    match action {
                        ExecutionAction::Open => {
                            task.next_open_execution = None;
                            if task.close_time.is_none() {
                                task.status = TaskStatus::Completed;
                            }
                        }
                        ExecutionAction::Close => {
                            task.next_close_execution = None;
                            task.status = TaskStatus::Completed;
                        }
                    }
                }

                self.db.update_task(task_id, task).await?;
                let _ = self.app_handle.emit("task-updated", task_id);

                Ok(())
            }
            Err(e) => {
                task.status = TaskStatus::Failed;
                self.db.update_task(task_id, task).await?;
                let _ = self.app_handle.emit("task-updated", task_id);

                Err(e)
            }
        }
    }

    fn should_continue_repeating(
        &self,
        task: &Task,
        next: chrono::DateTime<Utc>,
        repeat_config: &crate::db::RepeatConfig,
    ) -> bool {
        match (&repeat_config.end_after, &repeat_config.end_date) {
            (Some(count), _) => task.execution_count < *count,
            (None, Some(end_date)) => next < *end_date,
            (None, None) => true,
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
                let month = local_time.month();
                let year = local_time.year();

                let (next_month, next_year) = if month == 12 {
                    (1, year + 1)
                } else {
                    (month + 1, year)
                };

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

        Ok(next_local.with_timezone(&Utc))
    }
}
