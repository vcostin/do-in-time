use crate::core::browser_launcher::BrowserLauncher;
use crate::db::{Database, ExecutionAction, Task, TaskStatus};
use crate::error::{AppError, Result};
use crate::utils::schedule::{next_future_occurrence, should_schedule_occurrence};
use crate::utils::validation::{validate_browser_profile, validate_url};
use chrono::{DateTime, Utc};
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
                apply_success_schedule(&mut task, &action, Utc::now())?;

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
            Err(e) if action == ExecutionAction::Close && e.is_soft_close_miss() => {
                let message = e.to_string();
                apply_soft_close_miss(&mut task, &message);

                let _ = self
                    .db
                    .add_execution_log(task_id, &action, false, Some(&message))
                    .await;
                let _ = self.db.save_task_runtime(&task).await;
                let _ = self.app_handle.emit("task-updated", task_id);
                self.send_notification_if_enabled(&task, &action, Some(&message))
                    .await;

                Ok(())
            }
            Err(e) => {
                let message = e.to_string();
                apply_hard_failure(&mut task, &message);

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
                    Err(AppError::InvalidTask(
                        "Close without URL is blocked unless 'allow_close_all' is enabled for this task"
                            .to_string(),
                    ))
                }
            }
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
            if err.contains("Could not find") || err.starts_with("Close missed:") {
                (
                    format!("Close missed: {}", task.name),
                    format!("{} — {}", action, err),
                )
            } else {
                (
                    format!("Task failed: {}", task.name),
                    format!("{} — {}", action, err),
                )
            }
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

/// Soft close miss: consume the close slot; keep repeating (or still-open) tasks Active.
pub(crate) fn apply_soft_close_miss(task: &mut Task, message: &str) {
    task.next_close_execution = None;
    if task.repeat_config.is_none() && task.next_open_execution.is_none() {
        task.status = TaskStatus::Completed;
    } else if task.status != TaskStatus::Disabled {
        task.status = TaskStatus::Active;
    }
    task.set_last_error(&format!("Close missed: {message}"));
}

pub(crate) fn apply_hard_failure(task: &mut Task, message: &str) {
    task.status = TaskStatus::Failed;
    task.set_last_error(message);
}

/// Advance or clear next open/close after a successful action.
pub(crate) fn apply_success_schedule(
    task: &mut Task,
    action: &ExecutionAction,
    now: DateTime<Utc>,
) -> Result<()> {
    if task.repeat_config.is_some() {
        match action {
            ExecutionAction::Open => {
                // The open that just fired — its close must still run this session.
                let fired = task.next_open_execution.unwrap_or(task.start_time);
                let next = next_future_occurrence(task, fired, now)?;
                let continue_opens = should_schedule_occurrence(task, next);

                task.next_open_execution = if continue_opens { Some(next) } else { None };

                if let Some(close_time) = task.close_time {
                    let time_diff = close_time.signed_duration_since(task.start_time);
                    let close_for_fired = fired + time_diff;
                    task.next_close_execution = if close_for_fired > now {
                        Some(close_for_fired)
                    } else {
                        None
                    };
                } else {
                    task.next_close_execution = None;
                }

                if task.next_open_execution.is_none() && task.next_close_execution.is_none() {
                    task.status = TaskStatus::Completed;
                } else {
                    task.status = TaskStatus::Active;
                }
            }
            ExecutionAction::Close => {
                task.next_close_execution = None;
                if task.next_open_execution.is_none() {
                    task.status = TaskStatus::Completed;
                }
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{BrowserType, RepeatConfig, RepeatInterval};
    use chrono::{Duration, TimeZone};

    fn base_task(start: DateTime<Utc>) -> Task {
        Task {
            id: Some(1),
            name: "t".into(),
            browser: BrowserType::Firefox,
            browser_profile: None,
            url: Some("https://example.com".into()),
            allow_close_all: false,
            start_time: start,
            close_time: Some(start + Duration::hours(2)),
            timezone: "UTC".into(),
            repeat_config: None,
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: Some(start),
            next_close_execution: Some(start + Duration::hours(2)),
            last_error: None,
            last_execution_at: None,
        }
    }

    #[test]
    fn soft_close_miss_keeps_repeating_task_active() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: None,
            end_date: None,
        });
        task.next_open_execution = Some(start + Duration::days(1));
        task.next_close_execution = Some(start + Duration::hours(2));

        apply_soft_close_miss(&mut task, "Could not find a firefox window");

        assert_eq!(task.status, TaskStatus::Active);
        assert!(task.next_close_execution.is_none());
        assert_eq!(
            task.next_open_execution,
            Some(start + Duration::days(1))
        );
        assert!(task
            .last_error
            .as_deref()
            .unwrap()
            .starts_with("Close missed:"));
    }

    #[test]
    fn soft_close_miss_completes_one_shot_with_nothing_left() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.next_open_execution = None;
        task.next_close_execution = Some(start + Duration::hours(2));

        apply_soft_close_miss(&mut task, "no matching window");

        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.next_close_execution.is_none());
    }

    #[test]
    fn soft_close_miss_keeps_one_shot_active_if_open_remains() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        // Unusual but possible: close fired while a future open is still queued.
        task.next_open_execution = Some(start + Duration::days(1));

        apply_soft_close_miss(&mut task, "miss");

        assert_eq!(task.status, TaskStatus::Active);
        assert!(task.next_close_execution.is_none());
        assert_eq!(task.next_open_execution, Some(start + Duration::days(1)));
    }

    #[test]
    fn soft_close_miss_does_not_unpause_disabled() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: None,
            end_date: None,
        });
        task.status = TaskStatus::Disabled;

        apply_soft_close_miss(&mut task, "miss");

        assert_eq!(task.status, TaskStatus::Disabled);
        assert!(task.next_close_execution.is_none());
    }

    #[test]
    fn hard_failure_marks_failed() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        apply_hard_failure(&mut task, "launch failed");
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.last_error.as_deref(), Some("launch failed"));
    }

    #[test]
    fn close_target_not_found_is_soft_miss() {
        let soft = AppError::CloseTargetNotFound("gone".into());
        let hard = AppError::Scheduler("boom".into());
        assert!(soft.is_soft_close_miss());
        assert!(!hard.is_soft_close_miss());
    }

    #[test]
    fn success_open_one_shot_with_close_clears_open_keeps_active() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        apply_success_schedule(&mut task, &ExecutionAction::Open, start).unwrap();
        assert!(task.next_open_execution.is_none());
        assert_eq!(task.status, TaskStatus::Active);
        assert!(task.next_close_execution.is_some());
    }

    #[test]
    fn success_open_one_shot_without_close_completes() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.close_time = None;
        task.next_close_execution = None;
        apply_success_schedule(&mut task, &ExecutionAction::Open, start).unwrap();
        assert!(task.next_open_execution.is_none());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn success_close_one_shot_completes() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.next_open_execution = None;
        apply_success_schedule(&mut task, &ExecutionAction::Close, start).unwrap();
        assert!(task.next_close_execution.is_none());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn success_open_repeating_keeps_close_for_fired_session() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: None,
            end_date: None,
        });
        task.execution_count = 1;
        let now = start;
        apply_success_schedule(&mut task, &ExecutionAction::Open, now).unwrap();
        assert_eq!(
            task.next_open_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 2, 9, 0, 0).unwrap())
        );
        // Close belongs to the session that just opened (same day 11:00), not tomorrow.
        assert_eq!(
            task.next_close_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 11, 0, 0).unwrap())
        );
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn success_open_repeating_completes_when_end_after_reached() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: Some(1),
            end_date: None,
        });
        // After this open, execution_count is already incremented by execute().
        task.execution_count = 1;
        apply_success_schedule(&mut task, &ExecutionAction::Open, start).unwrap();
        assert!(task.next_open_execution.is_none());
        // Last open still needs its close.
        assert_eq!(
            task.next_close_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 11, 0, 0).unwrap())
        );
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn success_open_last_repeat_completes_after_close() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: Some(1),
            end_date: None,
        });
        task.execution_count = 1;
        task.next_open_execution = None;
        task.next_close_execution = Some(start + Duration::hours(2));
        apply_success_schedule(&mut task, &ExecutionAction::Close, start + Duration::hours(2))
            .unwrap();
        assert!(task.next_close_execution.is_none());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn success_open_repeating_completes_when_end_date_reached_even_with_end_after() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: Some(10),
            end_date: Some(Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap()),
        });
        task.execution_count = 1;
        apply_success_schedule(&mut task, &ExecutionAction::Open, start).unwrap();
        // Next open would be Jan 2 09:00, past end_date; keep today's close.
        assert!(task.next_open_execution.is_none());
        assert_eq!(
            task.next_close_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 11, 0, 0).unwrap())
        );
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn success_close_repeating_clears_close_keeps_open() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = base_task(start);
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: None,
            end_date: None,
        });
        let next_open = start + Duration::days(1);
        task.next_open_execution = Some(next_open);
        apply_success_schedule(&mut task, &ExecutionAction::Close, start).unwrap();
        assert!(task.next_close_execution.is_none());
        assert_eq!(task.next_open_execution, Some(next_open));
        assert_eq!(task.status, TaskStatus::Active);
    }
}
