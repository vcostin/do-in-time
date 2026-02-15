use std::sync::Arc;
use chrono::{Datelike, Duration, Timelike, TimeZone, Utc};
use chrono_tz::Tz;
use crate::core::browser_launcher::BrowserLauncher;
use crate::db::{Database, ExecutionStatus, RepeatInterval, Task, TaskAction, TaskStatus};
use crate::error::Result;

pub struct TaskExecutor {
    browser_launcher: BrowserLauncher,
    db: Arc<Database>,
}

impl TaskExecutor {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            browser_launcher: BrowserLauncher::new(),
            db,
        }
    }

    pub async fn execute(&self, mut task: Task) -> Result<()> {
        let task_id = task.id.expect("Task must have an ID");

        // Execute the task action
        let result = match task.action {
            TaskAction::Open => {
                self.browser_launcher
                    .open_browser(
                        &task.browser,
                        task.url.as_deref(),
                        task.browser_profile.as_deref(),
                    )
                    .await
            }
            TaskAction::Close => {
                self.browser_launcher.close_browser(&task.browser).await
            }
        };

        // Update task record based on execution result
        match result {
            Ok(_) => {
                // Log successful execution
                self.db
                    .log_execution(task_id, ExecutionStatus::Success, None)
                    .await?;

                // Update last executed time
                task.last_executed = Some(Utc::now());

                // Handle repeat logic
                if let Some(repeat_config) = &task.repeat_config {
                    let next = self.calculate_next_execution(&task)?;

                    // Check if we should continue repeating
                    let should_continue = match (&repeat_config.end_after, &repeat_config.end_date) {
                        (Some(count), _) => {
                            // Count executions for this task
                            let executions = self.db.get_task_executions(task_id).await?;
                            (executions.len() as i32) < *count
                        }
                        (None, Some(end_date)) => next < *end_date,
                        (None, None) => true,
                    };

                    if should_continue {
                        task.next_execution = Some(next);
                        task.status = TaskStatus::Active;
                    } else {
                        task.next_execution = None;
                        task.status = TaskStatus::Completed;
                    }
                } else {
                    // One-time task completed
                    task.next_execution = None;
                    task.status = TaskStatus::Completed;
                }

                // Update task in database
                self.db.update_task(task_id, task).await?;

                Ok(())
            }
            Err(e) => {
                // Log failed execution
                let error_msg = e.to_string();
                self.db
                    .log_execution(task_id, ExecutionStatus::Failed, Some(error_msg.clone()))
                    .await?;

                // Update task status to failed
                task.status = TaskStatus::Failed;
                self.db.update_task(task_id, task).await?;

                Err(e)
            }
        }
    }

    fn calculate_next_execution(&self, task: &Task) -> Result<chrono::DateTime<Utc>> {
        let repeat_config = task
            .repeat_config
            .as_ref()
            .expect("Task must have repeat config");

        // Parse timezone
        let tz: Tz = task
            .timezone
            .parse()
            .map_err(|_| crate::error::AppError::TimeParse(format!("Invalid timezone: {}", task.timezone)))?;

        // Convert scheduled time to task's timezone
        let local_time = task.scheduled_time.with_timezone(&tz);

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
