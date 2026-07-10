use crate::db::{Database, RecentExecutionLogEntry, Task, TaskExecutionLogEntry};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_all_tasks(db: State<'_, Arc<Database>>) -> Result<Vec<Task>, String> {
    db.get_all_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(id: i64, db: State<'_, Arc<Database>>) -> Result<Task, String> {
    db.get_task(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(task: Task, db: State<'_, Arc<Database>>) -> Result<Task, String> {
    db.create_task(task).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    id: i64,
    task: Task,
    db: State<'_, Arc<Database>>,
) -> Result<Task, String> {
    db.update_task(id, task).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(id: i64, db: State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_task(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_task_paused(
    id: i64,
    paused: bool,
    db: State<'_, Arc<Database>>,
) -> Result<Task, String> {
    db.set_task_paused(id, paused)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task_execution_log(
    id: i64,
    limit: Option<i64>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<TaskExecutionLogEntry>, String> {
    db.get_task_execution_log(id, limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_execution_log(
    limit: Option<i64>,
    failures_only: Option<bool>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<RecentExecutionLogEntry>, String> {
    db.get_recent_execution_log(limit.unwrap_or(50), failures_only.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}
