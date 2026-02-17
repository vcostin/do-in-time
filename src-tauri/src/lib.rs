use std::sync::Arc;
use tauri::Manager;

mod commands;
mod core;
mod db;
mod error;
mod utils;
mod tray;

use commands::{browser_commands, scheduler_commands, task_commands, settings_commands, window_commands};
use core::TaskScheduler;
use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                // Initialize database
                let db = Arc::new(
                    Database::new()
                        .await
                        .expect("Failed to initialize database"),
                );

                // Load settings
                let settings = db
                    .get_settings()
                    .await
                    .expect("Failed to load settings");

                // Create system tray and store it to prevent destruction
                let tray = tray::create_tray(&app_handle).expect("Failed to create system tray");

                // Initialize scheduler with AppHandle
                let scheduler = Arc::new(TaskScheduler::new(Arc::clone(&db), app_handle.clone()));

                // Store in app state
                app.manage(db);
                app.manage(scheduler.clone());
                app.manage(tray);

                // Auto-start scheduler
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = scheduler.start().await {
                        eprintln!("Failed to start scheduler: {}", e);
                    }
                });

                // Handle start_minimized setting
                if settings.start_minimized {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app_handle = window.app_handle();
                let db = app_handle.state::<Arc<Database>>();

                // Check minimize_to_tray setting
                let should_minimize_to_tray = tauri::async_runtime::block_on(async {
                    db.get_settings()
                        .await
                        .map(|s| s.minimize_to_tray)
                        .unwrap_or(false)
                });

                if should_minimize_to_tray {
                    // Prevent close and hide window instead
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            task_commands::get_all_tasks,
            task_commands::get_task,
            task_commands::create_task,
            task_commands::update_task,
            task_commands::delete_task,
            scheduler_commands::start_scheduler,
            scheduler_commands::stop_scheduler,
            scheduler_commands::get_scheduler_status,
            browser_commands::get_installed_browsers,
            browser_commands::get_default_browser,
            settings_commands::get_settings,
            settings_commands::update_settings,
            window_commands::toggle_window_visibility,
            window_commands::apply_auto_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
