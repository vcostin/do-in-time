use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use crate::core::task_executor::TaskExecutor;
use crate::db::Database;
use crate::error::{AppError, Result};
use chrono::Utc;

pub struct TaskScheduler {
    db: Arc<Database>,
    executor: Arc<TaskExecutor>,
    running: Arc<RwLock<bool>>,
}

impl TaskScheduler {
    pub fn new(db: Arc<Database>) -> Self {
        let executor = Arc::new(TaskExecutor::new(Arc::clone(&db)));
        Self {
            db,
            executor,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(AppError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        // Main scheduler loop
        let running_clone = Arc::clone(&self.running);
        let db_clone = Arc::clone(&self.db);
        let executor_clone = Arc::clone(&self.executor);

        tokio::spawn(async move {
            while *running_clone.read().await {
                // Get next action to execute (either open or close)
                match db_clone.get_next_action().await {
                    Ok(Some((task, action))) => {
                        let now = Utc::now();

                        // Determine which execution time to check based on action
                        let action_time = match action {
                            crate::db::ExecutionAction::Open => task.next_open_execution,
                            crate::db::ExecutionAction::Close => task.next_close_execution,
                        };

                        if let Some(execution_time) = action_time {
                            if execution_time <= now {
                                // Execute task with the specific action
                                let task_name = task.name.clone();
                                let action_str = match action {
                                    crate::db::ExecutionAction::Open => "open",
                                    crate::db::ExecutionAction::Close => "close",
                                };

                                if let Err(e) = executor_clone.execute(task, action).await {
                                    eprintln!("Failed to {} task '{}': {}", action_str, task_name, e);
                                }
                            } else {
                                // Sleep until next action (with max 60 seconds interval)
                                let duration = (execution_time - now)
                                    .to_std()
                                    .unwrap_or(Duration::from_secs(60))
                                    .min(Duration::from_secs(60));

                                sleep(duration).await;
                            }
                        } else {
                            // No execution time set, sleep briefly
                            sleep(Duration::from_secs(10)).await;
                        }
                    }
                    Ok(None) => {
                        // No active tasks, sleep for 10 seconds
                        sleep(Duration::from_secs(10)).await;
                    }
                    Err(e) => {
                        eprintln!("Error fetching next action: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(AppError::NotRunning);
        }
        *running = false;
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}
