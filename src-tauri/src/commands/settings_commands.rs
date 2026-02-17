use std::sync::Arc;
use tauri::State;
use crate::db::{Database, AppSettings};

#[tauri::command]
pub async fn get_settings(db: State<'_, Arc<Database>>) -> Result<AppSettings, String> {
    db.get_settings()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    db: State<'_, Arc<Database>>,
) -> Result<AppSettings, String> {
    db.update_settings(settings.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(settings)
}
