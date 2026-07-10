use crate::db::Task;
use crate::error::{AppError, Result};
use chrono_tz::Tz;
#[cfg(target_os = "macos")]
use std::borrow::Cow;

/// Normalize and validate an IANA timezone. Returns the trimmed zone name.
pub fn normalize_timezone(timezone: &str) -> Result<String> {
    let trimmed = timezone.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::TimeParse("Timezone cannot be empty".to_string()));
    }
    trimmed.parse::<Tz>().map(|_| trimmed.clone()).map_err(|_| {
        AppError::TimeParse(format!("Invalid timezone: {}", trimmed))
    })
}

/// Validates that `timezone` is a known IANA zone name.
pub fn validate_timezone(timezone: &str) -> Result<()> {
    normalize_timezone(timezone).map(|_| ())
}

/// Require close after start, and repeat end_date after start when set.
pub fn validate_task_times(task: &Task) -> Result<()> {
    if let Some(close) = task.close_time {
        if close <= task.start_time {
            return Err(AppError::InvalidTask(
                "Close time must be after start time".to_string(),
            ));
        }
    }
    if let Some(ref repeat) = task.repeat_config {
        if let Some(end_date) = repeat.end_date {
            if end_date <= task.start_time {
                return Err(AppError::InvalidTask(
                    "Repeat end date must be after start time".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Validates and sanitizes a URL string
///
/// # Security
/// - Ensures the URL starts with http:// or https://
/// - Prevents javascript:, data:, and other potentially dangerous schemes
/// - Validates basic URL structure
pub fn validate_url(url: &str) -> Result<()> {
    let url_trimmed = url.trim();

    if url_trimmed.is_empty() {
        return Err(AppError::InvalidTask("URL cannot be empty".to_string()));
    }

    // Check for dangerous URL schemes
    let dangerous_schemes = ["javascript:", "data:", "vbscript:", "file:", "about:"];

    let url_lower = url_trimmed.to_lowercase();
    for scheme in &dangerous_schemes {
        if url_lower.starts_with(scheme) {
            return Err(AppError::InvalidTask(format!(
                "Dangerous URL scheme not allowed: {}",
                scheme
            )));
        }
    }

    // Ensure URL starts with http:// or https://
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return Err(AppError::InvalidTask(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    // Basic URL validation - check for domain
    if url_trimmed.len() < 10 || !url_trimmed.contains('.') {
        return Err(AppError::InvalidTask("Invalid URL format".to_string()));
    }

    Ok(())
}

/// Validates a browser profile name
///
/// # Security
/// - Prevents path traversal attacks (../)
/// - Allows only alphanumeric characters, hyphens, underscores, and spaces
/// - Enforces reasonable length limits
pub fn validate_browser_profile(profile: &str) -> Result<()> {
    let profile_trimmed = profile.trim();

    if profile_trimmed.is_empty() {
        return Ok(()); // Empty profile is allowed (uses default)
    }

    // Check length
    if profile_trimmed.len() > 100 {
        return Err(AppError::InvalidTask(
            "Browser profile name too long (max 100 characters)".to_string(),
        ));
    }

    // Check for path traversal attempts
    if profile_trimmed.contains("..")
        || profile_trimmed.contains('/')
        || profile_trimmed.contains('\\')
    {
        return Err(AppError::InvalidTask(
            "Browser profile name cannot contain path separators or '..'".to_string(),
        ));
    }

    // Check for dangerous characters
    for c in profile_trimmed.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ' {
            return Err(AppError::InvalidTask(format!(
                "Browser profile name contains invalid character: '{}'",
                c
            )));
        }
    }

    Ok(())
}

/// Escapes a string for safe use in AppleScript
///
/// # Security
/// - Escapes backslashes and double quotes to prevent injection
/// - Prevents breaking out of AppleScript string contexts
#[cfg(target_os = "macos")]
pub fn escape_applescript_string(input: &str) -> Cow<'_, str> {
    if input.contains('\\') || input.contains('"') {
        let mut escaped = String::with_capacity(input.len() + 10);
        for c in input.chars() {
            match c {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                _ => escaped.push(c),
            }
        }
        Cow::Owned(escaped)
    } else {
        Cow::Borrowed(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://google.com").is_ok());
        assert!(validate_url("http://example.org/path").is_ok());
        assert!(validate_url("https://sub.domain.com:8080/path?query=1").is_ok());
    }

    #[test]
    fn test_validate_timezone_accepts_iana() {
        assert!(validate_timezone("UTC").is_ok());
        assert!(validate_timezone("America/New_York").is_ok());
        assert!(validate_timezone("Asia/Tokyo").is_ok());
        assert_eq!(
            normalize_timezone("  America/New_York  ").unwrap(),
            "America/New_York"
        );
    }

    #[test]
    fn test_validate_timezone_rejects_invalid() {
        assert!(validate_timezone("").is_err());
        assert!(validate_timezone("Not/A_Zone").is_err());
        assert!(validate_timezone("InvalidZone").is_err());
    }

    #[test]
    fn test_validate_task_times_close_and_end_date() {
        use crate::db::{BrowserType, RepeatConfig, RepeatInterval, Task, TaskStatus};
        use chrono::{Duration, TimeZone, Utc};

        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let mut task = Task {
            id: None,
            name: "t".into(),
            browser: BrowserType::Firefox,
            browser_profile: None,
            url: None,
            allow_close_all: false,
            start_time: start,
            close_time: Some(start + Duration::hours(2)),
            timezone: "UTC".into(),
            repeat_config: None,
            execution_count: 0,
            status: TaskStatus::Active,
            next_open_execution: None,
            next_close_execution: None,
            last_error: None,
            last_execution_at: None,
        };
        assert!(validate_task_times(&task).is_ok());

        task.close_time = Some(start);
        assert!(validate_task_times(&task).is_err());

        task.close_time = Some(start + Duration::hours(1));
        task.repeat_config = Some(RepeatConfig {
            interval: RepeatInterval::Daily,
            end_after: None,
            end_date: Some(start),
        });
        assert!(validate_task_times(&task).is_err());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        assert!(validate_url("javascript:alert('xss')").is_err());
        assert!(validate_url("data:text/html,<script>alert('xss')</script>").is_err());
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("vbscript:msgbox").is_err());
    }

    #[test]
    fn test_validate_url_missing_scheme() {
        assert!(validate_url("google.com").is_err());
        assert!(validate_url("www.example.com").is_err());
    }

    #[test]
    fn test_validate_browser_profile_valid() {
        assert!(validate_browser_profile("Default").is_ok());
        assert!(validate_browser_profile("Profile_1").is_ok());
        assert!(validate_browser_profile("My-Profile").is_ok());
        assert!(validate_browser_profile("").is_ok()); // Empty is ok
        assert!(validate_browser_profile("Work Profile").is_ok());
    }

    #[test]
    fn test_validate_browser_profile_path_traversal() {
        assert!(validate_browser_profile("../../../etc/passwd").is_err());
        assert!(validate_browser_profile("..\\windows\\system32").is_err());
        assert!(validate_browser_profile("profile/../other").is_err());
    }

    #[test]
    fn test_validate_browser_profile_invalid_chars() {
        assert!(validate_browser_profile("profile$name").is_err());
        assert!(validate_browser_profile("profile;rm -rf /").is_err());
        assert!(validate_browser_profile("profile`cmd`").is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_escape_applescript() {
        assert_eq!(escape_applescript_string("normal text"), "normal text");
        assert_eq!(
            escape_applescript_string("text with \"quotes\""),
            "text with \\\"quotes\\\""
        );
        assert_eq!(
            escape_applescript_string("path\\with\\backslashes"),
            "path\\\\with\\\\backslashes"
        );
        assert_eq!(
            escape_applescript_string("malicious\" end tell tell application \"Terminal"),
            "malicious\\\" end tell tell application \\\"Terminal"
        );
    }
}
