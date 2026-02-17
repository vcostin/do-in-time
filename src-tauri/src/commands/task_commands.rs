use std::sync::Arc;
use tauri::State;
use crate::db::{Database, Task};

#[tauri::command]
pub async fn get_all_tasks(db: State<'_, Arc<Database>>) -> Result<Vec<Task>, String> {
    db.get_all_tasks()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(id: i64, db: State<'_, Arc<Database>>) -> Result<Task, String> {
    db.get_task(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(
    task: Task,
    db: State<'_, Arc<Database>>,
) -> Result<Task, String> {
    db.create_task(task)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    id: i64,
    task: Task,
    db: State<'_, Arc<Database>>,
) -> Result<Task, String> {
    db.update_task(id, task)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(
    id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.delete_task(id)
        .await
        .map_err(|e| e.to_string())
}
