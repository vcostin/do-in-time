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
                // Get next task to execute
                match db_clone.get_next_task().await {
                    Ok(Some(task)) => {
                        let now = Utc::now();
                        let task_time = task.next_execution.unwrap_or(task.scheduled_time);

                        if task_time <= now {
                            // Execute task
                            let task_name = task.name.clone();
                            if let Err(e) = executor_clone.execute(task).await {
                                eprintln!("Failed to execute task '{}': {}", task_name, e);
                            }
                        } else {
                            // Sleep until next task (with max 60 seconds interval)
                            let duration = (task_time - now)
                                .to_std()
                                .unwrap_or(Duration::from_secs(60))
                                .min(Duration::from_secs(60));

                            sleep(duration).await;
                        }
                    }
                    Ok(None) => {
                        // No active tasks, sleep for 10 seconds
                        sleep(Duration::from_secs(10)).await;
                    }
                    Err(e) => {
                        eprintln!("Error fetching next task: {}", e);
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
