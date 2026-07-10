use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub name: String,
    pub browser: BrowserType,
    pub browser_profile: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub allow_close_all: bool,
    pub start_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub timezone: String,
    pub repeat_config: Option<RepeatConfig>,
    #[serde(default)]
    pub execution_count: i32,
    pub status: TaskStatus,
    pub next_open_execution: Option<DateTime<Utc>>,
    pub next_close_execution: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_execution_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionLogEntry {
    pub id: i64,
    pub task_id: i64,
    pub action: String,
    pub success: bool,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentExecutionLogEntry {
    pub id: i64,
    pub task_id: i64,
    pub task_name: String,
    pub action: String,
    pub success: bool,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BrowserType {
    Chrome,
    Firefox,
    Edge,
    Safari,
    Brave,
    Opera,
    Chromium,
    LibreWolf,
}

impl std::fmt::Display for BrowserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BrowserType::Chrome => "chrome",
            BrowserType::Firefox => "firefox",
            BrowserType::Edge => "edge",
            BrowserType::Safari => "safari",
            BrowserType::Brave => "brave",
            BrowserType::Opera => "opera",
            BrowserType::Chromium => "chromium",
            BrowserType::LibreWolf => "librewolf",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for BrowserType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chrome" => Ok(BrowserType::Chrome),
            "firefox" => Ok(BrowserType::Firefox),
            "edge" => Ok(BrowserType::Edge),
            "safari" => Ok(BrowserType::Safari),
            "brave" => Ok(BrowserType::Brave),
            "opera" => Ok(BrowserType::Opera),
            "chromium" => Ok(BrowserType::Chromium),
            "librewolf" => Ok(BrowserType::LibreWolf),
            _ => Err(format!("Unknown browser type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Active,
    Completed,
    Failed,
    Disabled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Active => "active",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Disabled => "disabled",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(TaskStatus::Active),
            "completed" => Ok(TaskStatus::Completed),
            "failed" => Ok(TaskStatus::Failed),
            "disabled" => Ok(TaskStatus::Disabled),
            _ => Err(format!("Unknown task status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatConfig {
    pub interval: RepeatInterval,
    pub end_after: Option<i32>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepeatInterval {
    Daily,
    Weekly,
    Monthly,
}

impl std::fmt::Display for RepeatInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RepeatInterval::Daily => "daily",
            RepeatInterval::Weekly => "weekly",
            RepeatInterval::Monthly => "monthly",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for RepeatInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(RepeatInterval::Daily),
            "weekly" => Ok(RepeatInterval::Weekly),
            "monthly" => Ok(RepeatInterval::Monthly),
            _ => Err(format!("Unknown repeat interval: {}", s)),
        }
    }
}

/// Action type used internally by the scheduler to determine what to execute.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionAction {
    Open,
    Close,
}

impl std::fmt::Display for ExecutionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ExecutionAction::Open => "open",
            ExecutionAction::Close => "close",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ExecutionAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ExecutionAction::Open),
            "close" => Ok(ExecutionAction::Close),
            _ => Err(format!("Unknown execution action: {}", s)),
        }
    }
}

const MAX_ERROR_LEN: usize = 500;

impl Task {
    pub fn set_last_error(&mut self, message: impl AsRef<str>) {
        let mut msg = message.as_ref().trim().to_string();
        if msg.len() > MAX_ERROR_LEN {
            msg.truncate(MAX_ERROR_LEN);
            msg.push('…');
        }
        self.last_error = Some(msg);
        self.last_execution_at = Some(Utc::now());
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
        self.last_execution_at = Some(Utc::now());
    }

    #[allow(dead_code)]
    pub fn new(
        name: String,
        browser: BrowserType,
        start_time: DateTime<Utc>,
        timezone: String,
    ) -> Self {
        Self {
            id: None,
            name,
            browser,
            browser_profile: None,
            url: None,
            allow_close_all: false,
            start_time,
            close_time: None,
            timezone,
            repeat_config: None,
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: Some(start_time),
            next_close_execution: None,
            last_error: None,
            last_execution_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub show_notifications: bool,
    pub auto_start: bool,
}
