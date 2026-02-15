use std::sync::Arc;
use tauri::Manager;

mod commands;
mod core;
mod db;
mod error;
mod utils;

use commands::{browser_commands, scheduler_commands, task_commands};
use core::TaskScheduler;
use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                // Initialize database
                let db = Arc::new(
                    Database::new()
                        .await
                        .expect("Failed to initialize database"),
                );

                // Initialize scheduler with AppHandle
                let scheduler = Arc::new(TaskScheduler::new(Arc::clone(&db), app_handle));

                // Store in app state
                app.manage(db);
                app.manage(scheduler.clone());

                // Auto-start scheduler
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = scheduler.start().await {
                        eprintln!("Failed to start scheduler: {}", e);
                    }
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            task_commands::get_all_tasks,
            task_commands::get_task,
            task_commands::create_task,
            task_commands::update_task,
            task_commands::delete_task,
            task_commands::get_task_history,
            scheduler_commands::start_scheduler,
            scheduler_commands::stop_scheduler,
            scheduler_commands::get_scheduler_status,
            browser_commands::get_installed_browsers,
            browser_commands::get_default_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
