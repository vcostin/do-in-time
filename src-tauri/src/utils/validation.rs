use crate::error::{AppError, Result};
#[cfg(target_os = "macos")]
use std::borrow::Cow;

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
    let dangerous_schemes = [
        "javascript:",
        "data:",
        "vbscript:",
        "file:",
        "about:",
    ];

    let url_lower = url_trimmed.to_lowercase();
    for scheme in &dangerous_schemes {
        if url_lower.starts_with(scheme) {
            return Err(AppError::InvalidTask(
                format!("Dangerous URL scheme not allowed: {}", scheme)
            ));
        }
    }

    // Ensure URL starts with http:// or https://
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return Err(AppError::InvalidTask(
            "URL must start with http:// or https://".to_string()
        ));
    }

    // Basic URL validation - check for domain
    if url_trimmed.len() < 10 || !url_trimmed.contains('.') {
        return Err(AppError::InvalidTask(
            "Invalid URL format".to_string()
        ));
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
            "Browser profile name too long (max 100 characters)".to_string()
        ));
    }

    // Check for path traversal attempts
    if profile_trimmed.contains("..") || profile_trimmed.contains('/') || profile_trimmed.contains('\\') {
        return Err(AppError::InvalidTask(
            "Browser profile name cannot contain path separators or '..'".to_string()
        ));
    }

    // Check for dangerous characters
    for c in profile_trimmed.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ' {
            return Err(AppError::InvalidTask(
                format!("Browser profile name contains invalid character: '{}'", c)
            ));
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
        assert_eq!(escape_applescript_string("text with \"quotes\""), "text with \\\"quotes\\\"");
        assert_eq!(escape_applescript_string("path\\with\\backslashes"), "path\\\\with\\\\backslashes");
        assert_eq!(
            escape_applescript_string("malicious\" end tell tell application \"Terminal"),
            "malicious\\\" end tell tell application \\\"Terminal"
        );
    }
}
