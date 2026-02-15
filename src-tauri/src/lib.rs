use std::sync::Arc;

mod commands;
mod core;
mod db;
mod error;

use commands::{scheduler_commands, task_commands};
use core::TaskScheduler;
use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::block_on(async {
        // Initialize database
        let db = Arc::new(
            Database::new()
                .await
                .expect("Failed to initialize database"),
        );

        // Initialize scheduler
        let scheduler = Arc::new(TaskScheduler::new(Arc::clone(&db)));

        // Auto-start scheduler
        let scheduler_clone = Arc::clone(&scheduler);
        tauri::async_runtime::spawn(async move {
            if let Err(e) = scheduler_clone.start().await {
                eprintln!("Failed to start scheduler: {}", e);
            }
        });

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .manage(db)
            .manage(scheduler)
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
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
