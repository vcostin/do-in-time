use crate::db::{RepeatInterval, Task};
use crate::error::{AppError, Result};
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

/// Advance `base_time` by one repeat interval in the task's timezone.
pub fn add_one_interval(task: &Task, base_time: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let repeat_config = task
        .repeat_config
        .as_ref()
        .ok_or_else(|| AppError::InvalidTask("Task has no repeat config".to_string()))?;

    let tz: Tz = task.timezone.parse().map_err(|_| {
        AppError::TimeParse(format!("Invalid timezone: {}", task.timezone))
    })?;

    let local_time = base_time.with_timezone(&tz);

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

            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(next_year, next_month + 1, 1)
                .unwrap_or_else(|| {
                    chrono::NaiveDate::from_ymd_opt(next_year + 1, 1, 1).unwrap()
                })
                .pred_opt()
                .unwrap()
                .day();

            let day = local_time.day().min(last_day_of_month);

            let next_date = chrono::NaiveDate::from_ymd_opt(next_year, next_month, day)
                .ok_or_else(|| {
                    AppError::TimeParse("Failed to calculate next month".to_string())
                })?;

            let next_datetime = next_date
                .and_hms_opt(local_time.hour(), local_time.minute(), local_time.second())
                .ok_or_else(|| {
                    AppError::TimeParse("Failed to create next datetime".to_string())
                })?;

            tz.from_local_datetime(&next_datetime)
                .single()
                .ok_or_else(|| AppError::TimeParse("Ambiguous local time".to_string()))?
        }
    };

    Ok(next_local.with_timezone(&Utc))
}

/// Next occurrence strictly after `now`, stepping from `from` by one interval at a time.
///
/// Use the just-fired (or original) open time as `from`. If the app was offline,
/// this skips missed slots so the scheduler does not tight-loop on past times.
pub fn next_future_occurrence(
    task: &Task,
    from: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<DateTime<Utc>> {
    let mut next = add_one_interval(task, from)?;
    // Bound iterations: daily for ~30 years is enough for any realistic catch-up.
    for _ in 0..12_000 {
        if next > now {
            return Ok(next);
        }
        next = add_one_interval(task, next)?;
    }
    Err(AppError::TimeParse(
        "Failed to find a future occurrence within iteration limit".to_string(),
    ))
}

/// Form fields that drive `next_open_execution` / `next_close_execution`.
///
/// Used by `update_task` so editing only the repeat settings still re-derives
/// the schedule (not only start/close time edits).
pub fn schedule_inputs_changed(old: &Task, new: &Task) -> bool {
    old.start_time != new.start_time
        || old.close_time != new.close_time
        || old.repeat_config != new.repeat_config
}

/// Schedule the next open (and matching close) for a repeating task whose
/// `start_time` may already be in the past.
///
/// For one-shot tasks with no remaining future open/close, sets status to
/// `Completed` when the task was `Active` (avoids zombie Active cards).
pub fn schedule_next_open_close(task: &mut Task, now: DateTime<Utc>) -> Result<()> {
    use crate::db::TaskStatus;

    if task.repeat_config.is_none() {
        if task.start_time > now {
            task.next_open_execution = Some(task.start_time);
        } else {
            task.next_open_execution = None;
        }

        if let Some(close_time) = task.close_time {
            task.next_close_execution = if close_time > now {
                Some(close_time)
            } else {
                None
            };
        } else {
            task.next_close_execution = None;
        }

        if task.next_open_execution.is_none()
            && task.next_close_execution.is_none()
            && task.status == TaskStatus::Active
        {
            task.status = TaskStatus::Completed;
        }

        return Ok(());
    }

    let next_open = if task.start_time > now {
        task.start_time
    } else {
        next_future_occurrence(task, task.start_time, now)?
    };

    task.next_open_execution = Some(next_open);

    if let Some(close_time) = task.close_time {
        let time_diff = close_time.signed_duration_since(task.start_time);
        task.next_close_execution = Some(next_open + time_diff);
    } else {
        task.next_close_execution = None;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{BrowserType, RepeatConfig, RepeatInterval, Task, TaskStatus};
    use chrono::TimeZone;

    fn daily_task(start: DateTime<Utc>) -> Task {
        Task {
            id: Some(1),
            name: "test".into(),
            browser: BrowserType::Firefox,
            browser_profile: None,
            url: None,
            allow_close_all: false,
            start_time: start,
            close_time: Some(start + Duration::hours(2)),
            timezone: "UTC".into(),
            repeat_config: Some(RepeatConfig {
                interval: RepeatInterval::Daily,
                end_after: None,
                end_date: None,
            }),
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: Some(start),
            next_close_execution: Some(start + Duration::hours(2)),
            last_error: None,
            last_execution_at: None,
        }
    }

    #[test]
    fn advances_from_last_run_not_original_start() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let task = daily_task(start);
        // Second open fires on Jan 2 — next must be Jan 3, not Jan 2 again.
        let fired = start + Duration::days(1);
        let next = next_future_occurrence(&task, fired, fired).unwrap();
        assert_eq!(next, start + Duration::days(2));
    }

    #[test]
    fn skips_missed_slots_after_downtime() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let task = daily_task(start);
        let now = Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap();
        let next = next_future_occurrence(&task, start, now).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 1, 6, 9, 0, 0).unwrap());
    }

    #[test]
    fn schedule_next_preserves_close_offset() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = daily_task(start);
        let now = Utc.with_ymd_and_hms(2026, 1, 3, 12, 0, 0).unwrap();
        schedule_next_open_close(&mut task, now).unwrap();
        assert_eq!(
            task.next_open_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 4, 9, 0, 0).unwrap())
        );
        assert_eq!(
            task.next_close_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 4, 11, 0, 0).unwrap())
        );
    }

    #[test]
    fn past_one_shot_becomes_completed() {
        let start = Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap();
        let mut task = Task {
            id: Some(1),
            name: "past".into(),
            browser: BrowserType::Firefox,
            browser_profile: None,
            url: None,
            allow_close_all: false,
            start_time: start,
            close_time: Some(start + Duration::hours(1)),
            timezone: "UTC".into(),
            repeat_config: None,
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: Some(start),
            next_close_execution: Some(start + Duration::hours(1)),
            last_error: None,
            last_execution_at: None,
        };
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        schedule_next_open_close(&mut task, now).unwrap();
        assert!(task.next_open_execution.is_none());
        assert!(task.next_close_execution.is_none());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn future_one_shot_stays_active() {
        let start = Utc.with_ymd_and_hms(2030, 1, 1, 9, 0, 0).unwrap();
        let mut task = Task {
            id: Some(1),
            name: "future".into(),
            browser: BrowserType::Firefox,
            browser_profile: None,
            url: None,
            allow_close_all: false,
            start_time: start,
            close_time: None,
            timezone: "UTC".into(),
            repeat_config: None,
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: Some(start),
            next_close_execution: None,
            last_error: None,
            last_execution_at: None,
        };
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        schedule_next_open_close(&mut task, now).unwrap();
        assert_eq!(task.next_open_execution, Some(start));
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn schedule_inputs_changed_detects_repeat_only() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let old = daily_task(start);
        let mut new = daily_task(start);
        assert!(!schedule_inputs_changed(&old, &new));

        new.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Weekly,
            end_after: None,
            end_date: None,
        });
        assert!(schedule_inputs_changed(&old, &new));
    }

    #[test]
    fn schedule_inputs_changed_detects_repeat_added_or_removed() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let with_repeat = daily_task(start);
        let mut one_shot = daily_task(start);
        one_shot.repeat_config = None;

        assert!(schedule_inputs_changed(&with_repeat, &one_shot));
        assert!(schedule_inputs_changed(&one_shot, &with_repeat));
    }

    #[test]
    fn daily_to_weekly_recomputes_next_open() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap(); // Thursday
        let mut task = daily_task(start);
        let now = Utc.with_ymd_and_hms(2026, 1, 3, 12, 0, 0).unwrap(); // Saturday
        schedule_next_open_close(&mut task, now).unwrap();
        assert_eq!(
            task.next_open_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 4, 9, 0, 0).unwrap()) // next daily
        );

        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Weekly,
            end_after: None,
            end_date: None,
        });
        schedule_next_open_close(&mut task, now).unwrap();
        assert_eq!(
            task.next_open_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 8, 9, 0, 0).unwrap()) // next Thursday
        );
        assert_eq!(
            task.next_close_execution,
            Some(Utc.with_ymd_and_hms(2026, 1, 8, 11, 0, 0).unwrap())
        );
    }
}
