use crate::db::BrowserType;
use std::process::Command;

#[cfg(target_os = "windows")]
fn system32_exe(exe_name: &str) -> std::path::PathBuf {
    let windows_dir = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .unwrap_or_else(|| "C:\\Windows".into());

    std::path::PathBuf::from(windows_dir)
        .join("System32")
        .join(exe_name)
}

#[cfg(target_os = "windows")]
pub fn get_installed_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    // Method 1: Check registry for registered browsers
    // Windows browsers register in HKLM\SOFTWARE\Clients\StartMenuInternet
    let registry_browsers = check_registry_browsers();
    browsers.extend(registry_browsers);

    // Method 2: Fallback to common installation paths
    if !browsers.contains(&BrowserType::Chrome) {
        if check_chrome_installed() {
            browsers.push(BrowserType::Chrome);
        }
    }

    if !browsers.contains(&BrowserType::Edge) {
        if check_edge_installed() {
            browsers.push(BrowserType::Edge);
        }
    }

    if !browsers.contains(&BrowserType::Firefox) {
        if check_firefox_installed() {
            browsers.push(BrowserType::Firefox);
        }
    }

    if !browsers.contains(&BrowserType::Brave) {
        if check_brave_installed() {
            browsers.push(BrowserType::Brave);
        }
    }

    if !browsers.contains(&BrowserType::Opera) {
        if check_opera_installed() {
            browsers.push(BrowserType::Opera);
        }
    }

    browsers.dedup();
    browsers
}

#[cfg(target_os = "windows")]
fn check_registry_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    // Query registry for StartMenuInternet entries
    let output = Command::new(system32_exe("reg.exe"))
        .args(&[
            "query",
            "HKLM\\SOFTWARE\\Clients\\StartMenuInternet",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if stdout.contains("chrome") || stdout.contains("google chrome") {
            browsers.push(BrowserType::Chrome);
        }
        if stdout.contains("msedge") || stdout.contains("microsoft edge") {
            browsers.push(BrowserType::Edge);
        }
        if stdout.contains("firefox") {
            browsers.push(BrowserType::Firefox);
        }
        if stdout.contains("brave") {
            browsers.push(BrowserType::Brave);
        }
        if stdout.contains("opera") {
            browsers.push(BrowserType::Opera);
        }
    }

    browsers
}

#[cfg(target_os = "windows")]
fn check_chrome_installed() -> bool {
    // Check common paths and registry App Paths
    std::path::Path::new("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe").exists()
        || std::path::Path::new("C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe").exists()
        || check_app_path("chrome.exe")
}

#[cfg(target_os = "windows")]
fn check_edge_installed() -> bool {
    std::path::Path::new("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe").exists()
        || std::path::Path::new("C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe").exists()
        || check_app_path("msedge.exe")
}

#[cfg(target_os = "windows")]
fn check_firefox_installed() -> bool {
    std::path::Path::new("C:\\Program Files\\Mozilla Firefox\\firefox.exe").exists()
        || std::path::Path::new("C:\\Program Files (x86)\\Mozilla Firefox\\firefox.exe").exists()
        || check_app_path("firefox.exe")
}

#[cfg(target_os = "windows")]
fn check_brave_installed() -> bool {
    std::path::Path::new("C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe").exists()
        || std::path::Path::new("C:\\Program Files (x86)\\BraveSoftware\\Brave-Browser\\Application\\brave.exe").exists()
        || check_app_path("brave.exe")
}

#[cfg(target_os = "windows")]
fn check_opera_installed() -> bool {
    std::path::Path::new("C:\\Program Files\\Opera\\opera.exe").exists()
        || std::path::Path::new("C:\\Program Files (x86)\\Opera\\opera.exe").exists()
        || check_app_path("opera.exe")
}

#[cfg(target_os = "windows")]
fn check_app_path(exe_name: &str) -> bool {
    // Check registry App Paths for custom installations
    let output = Command::new(system32_exe("reg.exe"))
        .args(&[
            "query",
            &format!("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{}", exe_name),
            "/ve",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract path from REG_SZ output
            for line in stdout.lines() {
                if line.contains("REG_SZ") || line.contains("REG_EXPAND_SZ") {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
pub fn get_default_browser() -> Option<BrowserType> {
    // Try to read default browser from registry
    let output = Command::new(system32_exe("reg.exe"))
        .args(&[
            "query",
            "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\Shell\\Associations\\UrlAssociations\\http\\UserChoice",
            "/v",
            "ProgId",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("ChromeHTML") {
            return Some(BrowserType::Chrome);
        } else if stdout.contains("MSEdgeHTM") {
            return Some(BrowserType::Edge);
        } else if stdout.contains("FirefoxURL") {
            return Some(BrowserType::Firefox);
        } else if stdout.contains("BraveHTML") {
            return Some(BrowserType::Brave);
        } else if stdout.contains("OperaStable") {
            return Some(BrowserType::Opera);
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn get_installed_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    // Method 1: Use mdfind (Spotlight) to search for browser apps
    let output = Command::new("mdfind")
        .args(&["kMDItemKind == 'Application'"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if stdout.contains("chrome.app") || stdout.contains("google chrome") {
            browsers.push(BrowserType::Chrome);
        }
        if stdout.contains("firefox.app") {
            browsers.push(BrowserType::Firefox);
        }
        if stdout.contains("safari.app") {
            browsers.push(BrowserType::Safari);
        }
        if stdout.contains("brave") {
            browsers.push(BrowserType::Brave);
        }
        if stdout.contains("opera.app") {
            browsers.push(BrowserType::Opera);
        }
    }

    // Method 2: Fallback to standard paths
    if !browsers.contains(&BrowserType::Chrome) && std::path::Path::new("/Applications/Google Chrome.app").exists() {
        browsers.push(BrowserType::Chrome);
    }

    if !browsers.contains(&BrowserType::Firefox) && std::path::Path::new("/Applications/Firefox.app").exists() {
        browsers.push(BrowserType::Firefox);
    }

    if !browsers.contains(&BrowserType::Safari) && std::path::Path::new("/Applications/Safari.app").exists() {
        browsers.push(BrowserType::Safari);
    }

    if !browsers.contains(&BrowserType::Brave) && std::path::Path::new("/Applications/Brave Browser.app").exists() {
        browsers.push(BrowserType::Brave);
    }

    if !browsers.contains(&BrowserType::Opera) && std::path::Path::new("/Applications/Opera.app").exists() {
        browsers.push(BrowserType::Opera);
    }

    // Method 3: Check user Applications folder
    if let Ok(home) = std::env::var("HOME") {
        let user_apps = format!("{}/Applications", home);

        if !browsers.contains(&BrowserType::Chrome) {
            let chrome_path = format!("{}/Google Chrome.app", user_apps);
            if std::path::Path::new(&chrome_path).exists() {
                browsers.push(BrowserType::Chrome);
            }
        }

        if !browsers.contains(&BrowserType::Firefox) {
            let firefox_path = format!("{}/Firefox.app", user_apps);
            if std::path::Path::new(&firefox_path).exists() {
                browsers.push(BrowserType::Firefox);
            }
        }

        if !browsers.contains(&BrowserType::Brave) {
            let brave_path = format!("{}/Brave Browser.app", user_apps);
            if std::path::Path::new(&brave_path).exists() {
                browsers.push(BrowserType::Brave);
            }
        }

        if !browsers.contains(&BrowserType::Opera) {
            let opera_path = format!("{}/Opera.app", user_apps);
            if std::path::Path::new(&opera_path).exists() {
                browsers.push(BrowserType::Opera);
            }
        }
    }

    browsers.dedup();
    browsers
}

#[cfg(target_os = "macos")]
pub fn get_default_browser() -> Option<BrowserType> {
    let output = Command::new("defaults")
        .args(&["read", "com.apple.LaunchServices/com.apple.launchservices.secure", "LSHandlers"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if stdout.contains("chrome") {
            return Some(BrowserType::Chrome);
        } else if stdout.contains("firefox") {
            return Some(BrowserType::Firefox);
        } else if stdout.contains("safari") {
            return Some(BrowserType::Safari);
        } else if stdout.contains("brave") {
            return Some(BrowserType::Brave);
        } else if stdout.contains("opera") {
            return Some(BrowserType::Opera);
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub fn get_installed_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    // Method 1: Check for .desktop files in XDG standard locations
    let desktop_paths = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
        format!("{}/.local/share/applications", std::env::var("HOME").unwrap_or_default()),
    ];

    for desktop_dir in desktop_paths {
        if let Ok(entries) = std::fs::read_dir(&desktop_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    let filename_lower = filename.to_lowercase();

                    if !browsers.contains(&BrowserType::Chrome)
                        && (filename_lower.contains("google-chrome") || filename_lower.contains("chrome.desktop"))
                    {
                        browsers.push(BrowserType::Chrome);
                    }

                    if !browsers.contains(&BrowserType::Firefox) && filename_lower.contains("firefox") {
                        browsers.push(BrowserType::Firefox);
                    }

                    if !browsers.contains(&BrowserType::Brave)
                        && (filename_lower.contains("brave") || filename_lower.contains("brave-browser"))
                    {
                        browsers.push(BrowserType::Brave);
                    }

                    if !browsers.contains(&BrowserType::Opera) && filename_lower.contains("opera") {
                        browsers.push(BrowserType::Opera);
                    }
                }
            }
        }
    }

    // Method 2: Check if browser commands exist in PATH using 'which'
    if !browsers.contains(&BrowserType::Chrome) {
        if Command::new("which")
            .arg("google-chrome")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("which")
                .arg("google-chrome-stable")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            || Command::new("which")
                .arg("chrome")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        {
            browsers.push(BrowserType::Chrome);
        }
    }

    if !browsers.contains(&BrowserType::Firefox) {
        if Command::new("which")
            .arg("firefox")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            browsers.push(BrowserType::Firefox);
        }
    }

    if !browsers.contains(&BrowserType::Brave) {
        if Command::new("which")
            .arg("brave-browser")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("which")
                .arg("brave")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        {
            browsers.push(BrowserType::Brave);
        }
    }

    if !browsers.contains(&BrowserType::Opera) {
        if Command::new("which")
            .arg("opera")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            browsers.push(BrowserType::Opera);
        }
    }

    browsers.dedup();
    browsers
}

#[cfg(target_os = "linux")]
pub fn get_default_browser() -> Option<BrowserType> {
    let output = Command::new("xdg-settings")
        .args(&["get", "default-web-browser"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if stdout.contains("chrome") {
            return Some(BrowserType::Chrome);
        } else if stdout.contains("firefox") {
            return Some(BrowserType::Firefox);
        } else if stdout.contains("brave") {
            return Some(BrowserType::Brave);
        } else if stdout.contains("opera") {
            return Some(BrowserType::Opera);
        }
    }

    None
}
