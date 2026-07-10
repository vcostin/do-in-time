use crate::core::browser_launcher::BrowserLauncher;
use crate::db::{Database, ExecutionAction, Task, TaskStatus};
use crate::error::Result;
use crate::utils::schedule::next_future_occurrence;
use crate::utils::validation::{validate_browser_profile, validate_url};
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

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

        if task.status == TaskStatus::Disabled {
            return Ok(());
        }

        let result = self.run_action(&task, &action).await;

        match result {
            Ok(()) => {
                if action == ExecutionAction::Open {
                    task.execution_count += 1;
                }

                task.clear_last_error();

                if let Some(repeat_config) = &task.repeat_config {
                    match action {
                        ExecutionAction::Open => {
                            let base = task.next_open_execution.unwrap_or(task.start_time);
                            let next = next_future_occurrence(&task, base, Utc::now())?;

                            let should_continue =
                                self.should_continue_repeating(&task, next, repeat_config);

                            if should_continue {
                                task.next_open_execution = Some(next);
                                if let Some(close_time) = task.close_time {
                                    let time_diff =
                                        close_time.signed_duration_since(task.start_time);
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

                let _ = self
                    .db
                    .add_execution_log(task_id, &action, true, Some("ok"))
                    .await;
                self.db.save_task_runtime(&task).await?;
                let _ = self.app_handle.emit("task-updated", task_id);
                self.send_notification_if_enabled(&task, &action, None)
                    .await;

                Ok(())
            }
            Err(e) => {
                let message = e.to_string();
                task.status = TaskStatus::Failed;
                task.set_last_error(&message);

                let _ = self
                    .db
                    .add_execution_log(task_id, &action, false, Some(&message))
                    .await;
                let _ = self.db.save_task_runtime(&task).await;
                let _ = self.app_handle.emit("task-updated", task_id);
                self.send_notification_if_enabled(&task, &action, Some(&message))
                    .await;

                Err(e)
            }
        }
    }

    async fn run_action(&self, task: &Task, action: &ExecutionAction) -> Result<()> {
        if let Some(ref url) = task.url {
            validate_url(url)?;
        }
        if let Some(ref profile) = task.browser_profile {
            validate_browser_profile(profile)?;
        }

        match action {
            ExecutionAction::Open => self
                .browser_launcher
                .open_browser(
                    &task.browser,
                    task.url.as_deref(),
                    task.browser_profile.as_deref(),
                )
                .await
                .map(|_| ()),
            ExecutionAction::Close => {
                if let Some(url) = &task.url {
                    self.browser_launcher
                        .close_browser_by_url(&task.browser, url, task.allow_close_all)
                        .await
                } else if task.allow_close_all {
                    self.browser_launcher.close_browser(&task.browser).await
                } else {
                    Err(crate::error::AppError::InvalidTask(
                        "Close without URL is blocked unless 'allow_close_all' is enabled for this task"
                            .to_string(),
                    ))
                }
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

    async fn send_notification_if_enabled(
        &self,
        task: &Task,
        action: &ExecutionAction,
        error: Option<&str>,
    ) {
        let settings = match self.db.get_settings().await {
            Ok(s) => s,
            Err(_) => return,
        };

        if !settings.show_notifications {
            return;
        }

        let (title, body) = if let Some(err) = error {
            (
                format!("Task failed: {}", task.name),
                format!("{} — {}", action, err),
            )
        } else {
            let action_text = match action {
                ExecutionAction::Open => "opened",
                ExecutionAction::Close => "closed",
            };
            let message = if let Some(ref url) = task.url {
                format!("{} {} in {}", action_text, url, task.browser)
            } else {
                format!("{} {}", action_text, task.browser)
            };
            (format!("Task: {}", task.name), message)
        };

        let _ = self
            .app_handle
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show();
    }
}
