use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn toggle_window_visibility(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main")
        .ok_or("Main window not found")?;

    if window.is_visible().map_err(|e| e.to_string())? {
        window.hide().map_err(|e| e.to_string())?;
    } else {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn apply_auto_start(enabled: bool, app: AppHandle) -> Result<(), String> {
    let autostart_manager = app.autolaunch();

    if enabled {
        autostart_manager
            .enable()
            .map_err(|e| format!("Failed to enable auto-start: {}", e))?;
    } else {
        autostart_manager
            .disable()
            .map_err(|e| format!("Failed to disable auto-start: {}", e))?;
    }

    Ok(())
}
