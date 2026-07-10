use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Time parsing error: {0}")]
    TimeParse(String),

    #[error("Browser not found: {0}")]
    #[allow(dead_code)]
    BrowserNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(i64),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    /// Best-effort close found no matching window/tab; not a hard task failure.
    #[error("{0}")]
    CloseTargetNotFound(String),

    #[error("Invalid task configuration: {0}")]
    InvalidTask(String),

    #[error("Already running")]
    AlreadyRunning,

    #[error("Not running")]
    NotRunning,
}

impl AppError {
    pub fn is_soft_close_miss(&self) -> bool {
        matches!(self, AppError::CloseTargetNotFound(_))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
