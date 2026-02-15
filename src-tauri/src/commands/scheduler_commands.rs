use std::sync::Arc;
use tauri::State;
use crate::core::TaskScheduler;

#[derive(serde::Serialize)]
pub struct SchedulerStatus {
    pub running: bool,
}

#[tauri::command]
pub async fn start_scheduler(scheduler: State<'_, Arc<TaskScheduler>>) -> Result<(), String> {
    scheduler
        .start()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_scheduler(scheduler: State<'_, Arc<TaskScheduler>>) -> Result<(), String> {
    scheduler
        .stop()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_scheduler_status(scheduler: State<'_, Arc<TaskScheduler>>) -> Result<SchedulerStatus, String> {
    Ok(SchedulerStatus {
        running: scheduler.is_running().await,
    })
}
