use crate::db::models::BrowserType;
use crate::error::{AppError, Result};
use crate::utils::validation::validate_browser_profile;
#[cfg(target_os = "macos")]
use crate::utils::validation::escape_applescript_string;
use std::process::{Child, Command};

pub struct BrowserLauncher;

impl BrowserLauncher {
    pub fn new() -> Self {
        Self
    }

    /// Open browser in existing session (preserves logged-in state)
    pub async fn open_browser(
        &self,
        browser: &BrowserType,
        url: Option<&str>,
        profile: Option<&str>,
    ) -> Result<Option<u32>> {
        let (command, mut args) = self.get_browser_command(browser, profile)?;

        // Add URL if provided
        if let Some(u) = url {
            args.push(u.to_string());
        }

        let child = self.spawn_browser(&command, &args, browser)?;
        let pid = child.map(|c| c.id());

        if let Some(u) = url {
            println!("Opening {} with URL: {}", browser, u);
        } else {
            println!("Opening {}", browser);
        }

        Ok(pid)
    }

    fn spawn_browser(&self, command: &str, args: &[String], browser: &BrowserType) -> Result<Option<Child>> {
        #[cfg(target_os = "windows")]
        {
            // On Windows, launch directly to get PID
            let mut cmd = Command::new(command);
            for arg in args {
                cmd.arg(arg);
            }

            let child = cmd
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to launch {}: {}", browser, e)))?;

            Ok(Some(child))
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, use open command but can't easily track PID
            let mut cmd = Command::new("open");
            cmd.arg("-a").arg(command);

            if !args.is_empty() {
                cmd.arg("--args");
                for arg in args {
                    cmd.arg(arg);
                }
            }

            cmd.spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to launch {}: {}", browser, e)))?;

            // Can't reliably get PID on macOS with open command
            Ok(None)
        }

        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new(command);
            for arg in args {
                cmd.arg(arg);
            }

            let child = cmd
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to launch {}: {}", browser, e)))?;

            Ok(Some(child))
        }
    }

    /// Close browser tabs/windows that match the given URL
    ///
    /// Platform-specific implementations:
    /// - macOS: Uses AppleScript to close tabs matching URL (automatic)
    /// - Windows: Manual close required (no native tab-level control available)
    /// - Linux: Closes all browser instances (fallback)
    ///
    /// ## Windows Limitation
    ///
    /// Unlike macOS which has AppleScript for scriptable browser control, Windows does not
    /// provide a native, straightforward mechanism to close specific browser tabs programmatically.
    ///
    /// Why native solutions don't work on Windows:
    /// - **PowerShell cmdlet approach**: Process command lines don't contain tab URLs in Chromium browsers
    /// - **UI Automation API**: Complex to implement, unreliable for dynamic web content
    /// - **Window title matching**: Fragile, titles change frequently and aren't unique
    ///
    /// Alternative solutions (not implemented due to complexity):
    /// - **Chrome DevTools Protocol (CDP)**: Requires browser to run with `--remote-debugging-port` flag
    /// - **Browser extensions**: Requires pre-installation and browser-specific implementations
    /// - **Native messaging**: Requires separate browser extension for each browser
    ///
    /// For now, Windows users must manually close tabs after they're opened by the scheduler.
    pub async fn close_browser_by_url(&self, browser: &BrowserType, url: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Windows: Manual close required
            // See function documentation above for detailed explanation of Windows limitations
            println!(
                "⚠ Windows: Please manually close the {} tab with URL: {}",
                browser, url
            );
            println!("Automatic tab closing is not available on Windows without additional setup.");
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            // Use AppleScript like the original Deno implementation
            let app_name = match browser {
                BrowserType::Chrome => "Google Chrome",
                BrowserType::Edge => "Microsoft Edge",
                BrowserType::Firefox => "Firefox",
                BrowserType::Safari => "Safari",
                BrowserType::Brave => "Brave Browser",
                BrowserType::Opera => "Opera",
            };

            // Sanitize URL to prevent AppleScript injection
            let escaped_url = escape_applescript_string(url);

            let script = format!(
                r#"tell application "{}"
                    close (every tab of every window whose URL contains "{}")
                end tell"#,
                app_name, escaped_url
            );

            let output = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| AppError::Scheduler(format!("Failed to execute AppleScript: {}", e)))?;

            if output.status.success() {
                println!("Successfully closed {} tab(s) with URL: {}", browser, url);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(AppError::Scheduler(format!("AppleScript error: {}", stderr)))
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: fallback to closing all instances since we don't have easy tab control
            println!("Linux: URL-based closing not supported, closing all {} instances", browser);
            self.close_browser(browser).await
        }
    }

    pub async fn close_browser(&self, browser: &BrowserType) -> Result<()> {
        let process_name = self.get_process_name(browser);

        #[cfg(target_os = "windows")]
        {
            Command::new("taskkill")
                .arg("/F")
                .arg("/IM")
                .arg(&process_name)
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to close {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("pkill")
                .arg("-x")
                .arg(&process_name)
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to close {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("pkill")
                .arg(&process_name)
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to close {}: {}", browser, e)))?;
        }

        Ok(())
    }

    fn get_browser_command(
        &self,
        browser: &BrowserType,
        profile: Option<&str>,
    ) -> Result<(String, Vec<String>)> {
        // Validate browser profile for security
        if let Some(prof) = profile {
            validate_browser_profile(prof)?;
        }

        let mut args = Vec::new();

        let command = match browser {
            BrowserType::Chrome => {
                if let Some(prof) = profile {
                    args.push(format!("--profile-directory={}", prof));
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path(&[
                        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
                    ])
                    .unwrap_or_else(|| "chrome.exe".to_string())
                }

                #[cfg(target_os = "macos")]
                {
                    "Google Chrome".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.find_browser_path(&[
                        "/usr/bin/google-chrome",
                        "/usr/bin/google-chrome-stable",
                        "/snap/bin/chromium",
                        "/usr/bin/chromium-browser",
                    ])
                    .unwrap_or_else(|| "google-chrome".to_string())
                }
            }
            BrowserType::Firefox => {
                if let Some(prof) = profile {
                    args.push("-P".to_string());
                    args.push(prof.to_string());
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path(&[
                        "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
                        "C:\\Program Files (x86)\\Mozilla Firefox\\firefox.exe",
                    ])
                    .unwrap_or_else(|| "firefox.exe".to_string())
                }

                #[cfg(target_os = "macos")]
                {
                    "Firefox".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.find_browser_path(&[
                        "/usr/bin/firefox",
                        "/snap/bin/firefox",
                    ])
                    .unwrap_or_else(|| "firefox".to_string())
                }
            }
            BrowserType::Edge => {
                if let Some(prof) = profile {
                    args.push(format!("--profile-directory={}", prof));
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path(&[
                        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
                    ])
                    .unwrap_or_else(|| "msedge.exe".to_string())
                }

                #[cfg(target_os = "macos")]
                {
                    "Microsoft Edge".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.find_browser_path(&[
                        "/usr/bin/microsoft-edge",
                        "/usr/bin/microsoft-edge-stable",
                    ])
                    .unwrap_or_else(|| "microsoft-edge".to_string())
                }
            }
            BrowserType::Safari => {
                #[cfg(target_os = "macos")]
                {
                    "Safari".to_string()
                }

                #[cfg(not(target_os = "macos"))]
                {
                    return Err(AppError::BrowserNotFound(
                        "Safari is only available on macOS".to_string(),
                    ));
                }
            }
            BrowserType::Brave => {
                if let Some(prof) = profile {
                    args.push(format!("--profile-directory={}", prof));
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path(&[
                        "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                        "C:\\Program Files (x86)\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                    ])
                    .unwrap_or_else(|| "brave.exe".to_string())
                }

                #[cfg(target_os = "macos")]
                {
                    "Brave Browser".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.find_browser_path(&[
                        "/usr/bin/brave-browser",
                        "/snap/bin/brave",
                    ])
                    .unwrap_or_else(|| "brave-browser".to_string())
                }
            }
            BrowserType::Opera => {
                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path(&[
                        "C:\\Program Files\\Opera\\launcher.exe",
                        "C:\\Program Files (x86)\\Opera\\launcher.exe",
                    ])
                    .unwrap_or_else(|| "opera.exe".to_string())
                }

                #[cfg(target_os = "macos")]
                {
                    "Opera".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.find_browser_path(&[
                        "/usr/bin/opera",
                        "/snap/bin/opera",
                    ])
                    .unwrap_or_else(|| "opera".to_string())
                }
            }
        };

        Ok((command, args))
    }

    fn get_process_name(&self, browser: &BrowserType) -> String {
        match browser {
            BrowserType::Chrome => {
                #[cfg(target_os = "windows")]
                {
                    "chrome.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Google Chrome".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "chrome".to_string()
                }
            }
            BrowserType::Firefox => {
                #[cfg(target_os = "windows")]
                {
                    "firefox.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Firefox".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "firefox".to_string()
                }
            }
            BrowserType::Edge => {
                #[cfg(target_os = "windows")]
                {
                    "msedge.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Microsoft Edge".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "microsoft-edge".to_string()
                }
            }
            BrowserType::Safari => {
                "Safari".to_string()
            }
            BrowserType::Brave => {
                #[cfg(target_os = "windows")]
                {
                    "brave.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Brave Browser".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "brave-browser".to_string()
                }
            }
            BrowserType::Opera => {
                #[cfg(target_os = "windows")]
                {
                    "opera.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Opera".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "opera".to_string()
                }
            }
        }
    }

    fn find_browser_path(&self, paths: &[&str]) -> Option<String> {
        for path in paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    }
}

impl Default for BrowserLauncher {
    fn default() -> Self {
        Self::new()
    }
}
