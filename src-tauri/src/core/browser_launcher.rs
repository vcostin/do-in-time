use crate::db::models::BrowserType;
use crate::error::{AppError, Result};
#[cfg(target_os = "macos")]
use crate::utils::validation::escape_applescript_string;
use crate::utils::validation::validate_browser_profile;
use std::process::{Child, Command};

/// Extract a safe host substring used to match window titles for URL-based close.
#[cfg(any(target_os = "windows", target_os = "linux", test))]
fn url_close_needle(url: &str) -> Result<String> {
    let trimmed = url.trim();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("HTTPS://"))
        .or_else(|| trimmed.strip_prefix("HTTP://"))
        .ok_or_else(|| {
            AppError::InvalidTask("URL must start with http:// or https://".to_string())
        })?;

    let authority = without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");

    // Drop userinfo if present: user:pass@host
    let host_port = authority
        .rsplit('@')
        .next()
        .unwrap_or(authority);

    let host = host_port.split(':').next().unwrap_or(host_port).to_lowercase();

    if host.is_empty()
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(AppError::InvalidTask(format!(
            "Could not extract a safe host from URL: {}",
            url
        )));
    }

    Ok(host)
}

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

    fn spawn_browser(
        &self,
        command: &str,
        args: &[String],
        browser: &BrowserType,
    ) -> Result<Option<Child>> {
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
            let mut cmd = Command::new("/usr/bin/open");
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
            let has_display = std::env::var("DISPLAY").is_ok()
                || std::env::var("WAYLAND_DISPLAY").is_ok();

            if !has_display {
                let xvfb_available = Command::new("which")
                    .arg("xvfb-run")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if xvfb_available {
                    println!("No display detected, launching {} via xvfb-run", browser);
                    let mut cmd = Command::new("xvfb-run");
                    cmd.arg("-a");
                    cmd.arg(command);
                    for arg in args {
                        cmd.arg(arg);
                    }
                    cmd.env_remove("WEBKIT_EXEC_PATH")
                        .env_remove("WEBKIT_INJECTED_BUNDLE_PATH")
                        .env_remove("JSC_ARGS");
                    let child = cmd.spawn().map_err(|e| {
                        AppError::Scheduler(format!(
                            "Failed to launch {} with xvfb-run: {}",
                            browser, e
                        ))
                    })?;
                    return Ok(Some(child));
                } else {
                    println!(
                        "⚠ No display detected and xvfb-run not found. \
                        Install with: sudo apt install xvfb"
                    );
                }
            }

            let mut cmd = Command::new(command);
            for arg in args {
                cmd.arg(arg);
            }
            // Strip WebKitGTK env vars that leak into child processes and cause
            // unrecognized flag errors in Chromium
            cmd.env_remove("WEBKIT_EXEC_PATH")
                .env_remove("WEBKIT_INJECTED_BUNDLE_PATH")
                .env_remove("JSC_ARGS");

            let child = cmd
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to launch {}: {}", browser, e)))?;

            Ok(Some(child))
        }
    }

    /// Close browser tabs/windows that match the given URL.
    ///
    /// Platform behavior:
    /// - **macOS**: AppleScript closes tabs whose URL contains the target
    /// - **Windows / Linux**: close windows whose title contains the URL host;
    ///   if none match and `allow_close_all` is set, terminate all processes
    ///   for that browser. Never kills every browser instance for a URL close
    ///   unless that fallback is explicitly allowed.
    pub async fn close_browser_by_url(
        &self,
        browser: &BrowserType,
        url: &str,
        allow_close_all: bool,
    ) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let _ = allow_close_all;
            let app_name = match browser {
                BrowserType::Chrome => "Google Chrome",
                BrowserType::Edge => "Microsoft Edge",
                BrowserType::Firefox => "Firefox",
                BrowserType::Safari => "Safari",
                BrowserType::Brave => "Brave Browser",
                BrowserType::Opera => "Opera",
                BrowserType::Chromium => "Chromium",
                BrowserType::LibreWolf => "LibreWolf",
            };

            let escaped_url = escape_applescript_string(url);

            let script = format!(
                r#"tell application "{}"
                    close (every tab of every window whose URL contains "{}")
                end tell"#,
                app_name, escaped_url
            );

            let output = Command::new("/usr/bin/osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| {
                    AppError::Scheduler(format!("Failed to execute AppleScript: {}", e))
                })?;

            if output.status.success() {
                println!("Successfully closed {} tab(s) with URL: {}", browser, url);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(AppError::Scheduler(format!(
                    "AppleScript error: {}",
                    stderr
                )))
            }
        }

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            let needle = url_close_needle(url)?;
            let closed = self.close_windows_matching_title(browser, &needle)?;

            if closed {
                println!(
                    "Closed {} window(s) matching host '{}' from URL {}",
                    browser, needle, url
                );
                return Ok(());
            }

            if allow_close_all {
                println!(
                    "No {} window title matched '{}'; falling back to close-all (allowed)",
                    browser, needle
                );
                return self.close_browser(browser).await;
            }

            Err(AppError::Scheduler(format!(
                "Could not find a {} window whose title contains '{}'. \
                 Enable 'Allow close all browser instances' to terminate all {} processes as a fallback.",
                browser, needle, browser
            )))
        }
    }

    /// Best-effort: close top-level windows whose title contains `needle`.
    /// Returns true if at least one window was asked to close.
    #[cfg(target_os = "linux")]
    fn close_windows_matching_title(&self, browser: &BrowserType, needle: &str) -> Result<bool> {
        let _ = browser;
        let needle_lower = needle.to_lowercase();

        if Self::command_exists("wmctrl") {
            let output = Command::new("wmctrl")
                .arg("-l")
                .output()
                .map_err(|e| AppError::Scheduler(format!("wmctrl -l failed: {}", e)))?;

            if output.status.success() {
                let listing = String::from_utf8_lossy(&output.stdout);
                let mut closed_any = false;

                for line in listing.lines() {
                    let mut parts = line.split_whitespace();
                    let Some(window_id) = parts.next() else {
                        continue;
                    };
                    // desktop number
                    let _ = parts.next();
                    // client machine
                    let _ = parts.next();
                    let title = parts.collect::<Vec<_>>().join(" ");
                    if !title.to_lowercase().contains(&needle_lower) {
                        continue;
                    }

                    let status = Command::new("wmctrl")
                        .args(["-i", "-c", window_id])
                        .status()
                        .map_err(|e| {
                            AppError::Scheduler(format!("wmctrl close failed: {}", e))
                        })?;
                    if status.success() {
                        closed_any = true;
                    }
                }

                if closed_any {
                    return Ok(true);
                }
            }
        }

        if Self::command_exists("xdotool") {
            let output = Command::new("xdotool")
                .args(["search", "--name", needle])
                .output()
                .map_err(|e| AppError::Scheduler(format!("xdotool search failed: {}", e)))?;

            if output.status.success() {
                let ids = String::from_utf8_lossy(&output.stdout);
                let mut closed_any = false;
                for id in ids.split_whitespace() {
                    let status = Command::new("xdotool")
                        .args(["windowclose", id])
                        .status()
                        .map_err(|e| {
                            AppError::Scheduler(format!("xdotool windowclose failed: {}", e))
                        })?;
                    if status.success() {
                        closed_any = true;
                    }
                }
                if closed_any {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[cfg(target_os = "windows")]
    fn close_windows_matching_title(&self, browser: &BrowserType, needle: &str) -> Result<bool> {
        // Get-Process uses the name without .exe
        let process_name = self
            .get_process_name(browser)
            .trim_end_matches(".exe")
            .to_string();

        // needle/process_name are constrained; still quote safely for PowerShell.
        let needle_ps = needle.replace('\'', "''");
        let process_ps = process_name.replace('\'', "''");

        let script = format!(
            r#"
$Needle = '{needle}'
$ProcessName = '{process}'
$closed = 0
Get-Process -Name $ProcessName -ErrorAction SilentlyContinue |
  Where-Object {{
    $_.MainWindowHandle -ne 0 -and
    $_.MainWindowTitle -and
    ($_.MainWindowTitle.ToLower().Contains($Needle.ToLower()))
  }} |
  ForEach-Object {{
    if ($_.CloseMainWindow()) {{ $closed++ }}
  }}
Write-Output $closed
"#,
            needle = needle_ps,
            process = process_ps,
        );

        let output = Command::new(Self::windows_system32_exe(
            "WindowsPowerShell\\v1.0\\powershell.exe",
        ))
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|e| AppError::Scheduler(format!("PowerShell close failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Scheduler(format!(
                "PowerShell window close error: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let closed: u32 = stdout
            .lines()
            .rev()
            .find_map(|line| line.trim().parse().ok())
            .unwrap_or(0);

        Ok(closed > 0)
    }

    #[cfg(target_os = "linux")]
    fn command_exists(name: &str) -> bool {
        Command::new("which")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn close_browser(&self, browser: &BrowserType) -> Result<()> {
        let process_name = self.get_process_name(browser);

        #[cfg(target_os = "windows")]
        {
            Command::new(Self::windows_system32_exe("taskkill.exe"))
                .arg("/F")
                .arg("/IM")
                .arg(&process_name)
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to close {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("/usr/bin/pkill")
                .arg("-x")
                .arg(&process_name)
                .spawn()
                .map_err(|e| AppError::Scheduler(format!("Failed to close {}: {}", browser, e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("pkill")
                .arg("-f")
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
                    self.find_browser_path_windows(
                        "chrome.exe",
                        &[
                            "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                            "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
                        ],
                    )
                    .ok_or_else(|| {
                        AppError::BrowserNotFound("Google Chrome executable not found".to_string())
                    })?
                }

                #[cfg(target_os = "macos")]
                {
                    "Google Chrome".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &[
                            "/usr/bin/google-chrome",
                            "/usr/bin/google-chrome-stable",
                        ],
                        &["google-chrome", "google-chrome-stable", "chrome"],
                        Some("com.google.Chrome"),
                    )
                }
            }
            BrowserType::Firefox => {
                if let Some(prof) = profile {
                    args.push("-P".to_string());
                    args.push(prof.to_string());
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path_windows(
                        "firefox.exe",
                        &[
                            "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
                            "C:\\Program Files (x86)\\Mozilla Firefox\\firefox.exe",
                        ],
                    )
                    .ok_or_else(|| {
                        AppError::BrowserNotFound(
                            "Mozilla Firefox executable not found".to_string(),
                        )
                    })?
                }

                #[cfg(target_os = "macos")]
                {
                    "Firefox".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &["/usr/bin/firefox", "/snap/bin/firefox"],
                        &["firefox"],
                        Some("org.mozilla.firefox"),
                    )
                }
            }
            BrowserType::Edge => {
                if let Some(prof) = profile {
                    args.push(format!("--profile-directory={}", prof));
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path_windows(
                        "msedge.exe",
                        &[
                            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
                        ],
                    )
                    .ok_or_else(|| {
                        AppError::BrowserNotFound("Microsoft Edge executable not found".to_string())
                    })?
                }

                #[cfg(target_os = "macos")]
                {
                    "Microsoft Edge".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &[
                            "/usr/bin/microsoft-edge",
                            "/usr/bin/microsoft-edge-stable",
                        ],
                        &["microsoft-edge", "microsoft-edge-stable"],
                        Some("com.microsoft.Edge"),
                    )
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
                    self.find_browser_path_windows("brave.exe", &[
                        "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                        "C:\\Program Files (x86)\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                    ])
                    .ok_or_else(|| AppError::BrowserNotFound("Brave executable not found".to_string()))?
                }

                #[cfg(target_os = "macos")]
                {
                    "Brave Browser".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &["/usr/bin/brave-browser", "/snap/bin/brave"],
                        &["brave-browser", "brave"],
                        Some("com.brave.Browser"),
                    )
                }
            }
            BrowserType::Opera => {
                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path_windows(
                        "launcher.exe",
                        &[
                            "C:\\Program Files\\Opera\\launcher.exe",
                            "C:\\Program Files (x86)\\Opera\\launcher.exe",
                        ],
                    )
                    .ok_or_else(|| {
                        AppError::BrowserNotFound("Opera executable not found".to_string())
                    })?
                }

                #[cfg(target_os = "macos")]
                {
                    "Opera".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &["/usr/bin/opera", "/snap/bin/opera"],
                        &["opera"],
                        Some("com.opera.Opera"),
                    )
                }
            }
            BrowserType::Chromium => {
                #[cfg(target_os = "windows")]
                {
                    return Err(AppError::BrowserNotFound(
                        "Chromium is not supported on Windows. Use Chrome instead.".to_string(),
                    ));
                }

                #[cfg(target_os = "macos")]
                {
                    "Chromium".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &[
                            "/usr/bin/chromium-browser",
                            "/usr/bin/chromium",
                            "/snap/bin/chromium",
                        ],
                        &["chromium", "chromium-browser"],
                        Some("org.chromium.Chromium"),
                    )
                }
            }
            BrowserType::LibreWolf => {
                if let Some(prof) = profile {
                    args.push("-P".to_string());
                    args.push(prof.to_string());
                }

                #[cfg(target_os = "windows")]
                {
                    self.find_browser_path_windows(
                        "librewolf.exe",
                        &[
                            "C:\\Program Files\\LibreWolf\\librewolf.exe",
                            "C:\\Program Files (x86)\\LibreWolf\\librewolf.exe",
                        ],
                    )
                    .ok_or_else(|| {
                        AppError::BrowserNotFound("LibreWolf executable not found".to_string())
                    })?
                }

                #[cfg(target_os = "macos")]
                {
                    "LibreWolf".to_string()
                }

                #[cfg(target_os = "linux")]
                {
                    self.resolve_linux_command(
                        &mut args,
                        &["/usr/bin/librewolf", "/usr/local/bin/librewolf"],
                        &["librewolf"],
                        Some("io.gitlab.librewolf-community"),
                    )
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
            BrowserType::Safari => "Safari".to_string(),
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
            BrowserType::Chromium => {
                #[cfg(target_os = "windows")]
                {
                    "chromium.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "Chromium".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "chromium".to_string()
                }
            }
            BrowserType::LibreWolf => {
                #[cfg(target_os = "windows")]
                {
                    "librewolf.exe".to_string()
                }
                #[cfg(target_os = "macos")]
                {
                    "LibreWolf".to_string()
                }
                #[cfg(target_os = "linux")]
                {
                    "librewolf".to_string()
                }
            }
        }
    }

    /// Prefer a native binary; otherwise launch via Flatpak when available.
    #[cfg(target_os = "linux")]
    fn resolve_linux_command(
        &self,
        args: &mut Vec<String>,
        paths: &[&str],
        commands: &[&str],
        flatpak_id: Option<&str>,
    ) -> String {
        if let Some(path) = self.find_browser_path(paths) {
            return path;
        }

        for command in commands {
            let found = Command::new("which")
                .arg(command)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if found {
                return command.to_string();
            }
        }

        if let Some(app_id) = flatpak_id {
            let installed = Command::new("flatpak")
                .args(["info", app_id])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if installed {
                let mut flatpak_args = vec!["run".to_string(), app_id.to_string()];
                flatpak_args.append(args);
                *args = flatpak_args;
                return "flatpak".to_string();
            }
        }

        commands
            .first()
            .copied()
            .unwrap_or("browser")
            .to_string()
    }

    #[cfg(target_os = "linux")]
    fn find_browser_path(&self, paths: &[&str]) -> Option<String> {
        for path in paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn find_browser_path_windows(&self, exe_name: &str, paths: &[&str]) -> Option<String> {
        // Prefer absolute paths from the Windows registry App Paths key.
        // This avoids relying on the process search order (PATH / current directory).
        self.query_windows_app_path(exe_name).or_else(|| {
            for path in paths {
                if std::path::Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
            None
        })
    }

    #[cfg(target_os = "windows")]
    fn query_windows_app_path(&self, exe_name: &str) -> Option<String> {
        // Try HKLM first, then HKCU.
        for hive in ["HKLM", "HKCU"] {
            let key = format!(
                "{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{}",
                hive, exe_name
            );
            let output = Command::new(Self::windows_system32_exe("reg.exe"))
                .args(["query", &key, "/ve"])
                .output()
                .ok()?;

            if !output.status.success() {
                continue;
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Typical format:
                // (Default)    REG_SZ    C:\Program Files\...\chrome.exe
                if line.contains("REG_SZ") || line.contains("REG_EXPAND_SZ") {
                    let value = if let Some(idx) = line.find("REG_EXPAND_SZ") {
                        &line[idx + "REG_EXPAND_SZ".len()..]
                    } else if let Some(idx) = line.find("REG_SZ") {
                        &line[idx + "REG_SZ".len()..]
                    } else {
                        continue;
                    };
                    let path = value.trim().trim_matches('"');
                    if !path.is_empty() && std::path::Path::new(path).exists() {
                        return Some(path.to_string());
                    }
                }
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn windows_system32_exe(exe_name: &str) -> std::path::PathBuf {
        let windows_dir = std::env::var_os("SystemRoot")
            .or_else(|| std::env::var_os("WINDIR"))
            .unwrap_or_else(|| "C:\\Windows".into());

        std::path::PathBuf::from(windows_dir)
            .join("System32")
            .join(exe_name)
    }
}

impl Default for BrowserLauncher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::url_close_needle;

    #[test]
    fn extracts_host_from_https_url() {
        assert_eq!(
            url_close_needle("https://www.example.com/path?q=1").unwrap(),
            "www.example.com"
        );
    }

    #[test]
    fn strips_userinfo_and_port() {
        assert_eq!(
            url_close_needle("http://user:pass@example.com:8080/x").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn rejects_unsafe_host_characters() {
        assert!(url_close_needle("https://exam ple.com/").is_err());
    }
}

