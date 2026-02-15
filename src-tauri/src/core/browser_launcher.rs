use crate::db::models::BrowserType;
use crate::error::{AppError, Result};
use std::process::Command;

pub struct BrowserLauncher;

impl BrowserLauncher {
    pub fn new() -> Self {
        Self
    }

    pub async fn open_browser(
        &self,
        browser: &BrowserType,
        url: Option<&str>,
        profile: Option<&str>,
    ) -> Result<()> {
        let (command, mut args) = self.get_browser_command(browser, profile)?;

        // Add URL if provided
        if let Some(u) = url {
            args.push(u.to_string());
        }

        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C")
                .arg("start")
                .arg("")
                .arg(&command);

            for arg in &args {
                cmd.arg(arg);
            }

            cmd.spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to open {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            let mut cmd = Command::new("open");
            cmd.arg("-a").arg(&command);

            if !args.is_empty() {
                cmd.arg("--args");
                for arg in &args {
                    cmd.arg(arg);
                }
            }

            cmd.spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to open {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new(&command);
            for arg in &args {
                cmd.arg(arg);
            }

            cmd.spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to open {}: {}", browser, e)))?;
        }

        Ok(())
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
