use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub name: String,
    pub browser: BrowserType,
    pub browser_profile: Option<String>,
    pub url: Option<String>,
    pub action: TaskAction,
    pub scheduled_time: DateTime<Utc>,
    pub timezone: String,
    pub repeat_config: Option<RepeatConfig>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
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
            _ => Err(format!("Unknown browser type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskAction {
    Open,
    Close,
}

impl std::fmt::Display for TaskAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskAction::Open => "open",
            TaskAction::Close => "close",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for TaskAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(TaskAction::Open),
            "close" => Ok(TaskAction::Close),
            _ => Err(format!("Unknown task action: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Disabled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Pending => "pending",
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
            "pending" => Ok(TaskStatus::Pending),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: i64,
    pub task_id: i64,
    pub executed_at: DateTime<Utc>,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Success,
    Failed,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ExecutionStatus::Success => "success",
            ExecutionStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ExecutionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "success" => Ok(ExecutionStatus::Success),
            "failed" => Ok(ExecutionStatus::Failed),
            _ => Err(format!("Unknown execution status: {}", s)),
        }
    }
}

impl Task {
    #[allow(dead_code)]
    pub fn new(
        name: String,
        browser: BrowserType,
        action: TaskAction,
        scheduled_time: DateTime<Utc>,
        timezone: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            browser,
            browser_profile: None,
            url: None,
            action,
            scheduled_time,
            timezone,
            repeat_config: None,
            status: TaskStatus::Active,
            created_at: now,
            updated_at: now,
            last_executed: None,
            next_execution: Some(scheduled_time),
        }
    }
}
