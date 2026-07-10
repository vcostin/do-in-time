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
        .args(&["query", "HKLM\\SOFTWARE\\Clients\\StartMenuInternet"])
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
        || std::path::Path::new("C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe")
            .exists()
        || check_app_path("chrome.exe")
}

#[cfg(target_os = "windows")]
fn check_edge_installed() -> bool {
    std::path::Path::new("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe")
        .exists()
        || std::path::Path::new("C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe")
            .exists()
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
    std::path::Path::new("C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe")
        .exists()
        || std::path::Path::new(
            "C:\\Program Files (x86)\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
        )
        .exists()
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
            &format!(
                "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{}",
                exe_name
            ),
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
        .args(["kMDItemKind == 'Application'"])
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
    if !browsers.contains(&BrowserType::Chrome)
        && std::path::Path::new("/Applications/Google Chrome.app").exists()
    {
        browsers.push(BrowserType::Chrome);
    }

    if !browsers.contains(&BrowserType::Firefox)
        && std::path::Path::new("/Applications/Firefox.app").exists()
    {
        browsers.push(BrowserType::Firefox);
    }

    if !browsers.contains(&BrowserType::Safari)
        && std::path::Path::new("/Applications/Safari.app").exists()
    {
        browsers.push(BrowserType::Safari);
    }

    if !browsers.contains(&BrowserType::Brave)
        && std::path::Path::new("/Applications/Brave Browser.app").exists()
    {
        browsers.push(BrowserType::Brave);
    }

    if !browsers.contains(&BrowserType::Opera)
        && std::path::Path::new("/Applications/Opera.app").exists()
    {
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
        .args([
            "read",
            "com.apple.LaunchServices/com.apple.launchservices.secure",
            "LSHandlers",
        ])
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

    // Method 1: Check .desktop files in XDG + Flatpak export locations
    for desktop_dir in linux_desktop_dirs() {
        if let Ok(entries) = std::fs::read_dir(&desktop_dir) {
            for entry in entries.flatten() {
                if let Some(browser) = browser_from_desktop_filename(&entry.file_name()) {
                    if !browsers.contains(&browser) {
                        browsers.push(browser);
                    }
                }
            }
        }
    }

    // Method 2: PATH / common binary locations
    add_if_missing(&mut browsers, BrowserType::Chrome, || {
        command_exists("google-chrome")
            || command_exists("google-chrome-stable")
            || command_exists("chrome")
    });
    add_if_missing(&mut browsers, BrowserType::Chromium, || {
        command_exists("chromium")
            || command_exists("chromium-browser")
            || std::path::Path::new("/usr/bin/chromium").exists()
            || std::path::Path::new("/usr/bin/chromium-browser").exists()
            || std::path::Path::new("/snap/bin/chromium").exists()
    });
    add_if_missing(&mut browsers, BrowserType::Firefox, || {
        command_exists("firefox") || std::path::Path::new("/snap/bin/firefox").exists()
    });
    add_if_missing(&mut browsers, BrowserType::LibreWolf, || {
        command_exists("librewolf")
            || std::path::Path::new("/usr/bin/librewolf").exists()
            || std::path::Path::new("/usr/local/bin/librewolf").exists()
    });
    add_if_missing(&mut browsers, BrowserType::Brave, || {
        command_exists("brave-browser") || command_exists("brave")
    });
    add_if_missing(&mut browsers, BrowserType::Opera, || command_exists("opera"));
    add_if_missing(&mut browsers, BrowserType::Edge, || {
        command_exists("microsoft-edge") || command_exists("microsoft-edge-stable")
    });

    // Method 3: Flatpak app IDs (covers browsers not exported to PATH)
    for (browser, app_id) in linux_flatpak_browsers() {
        add_if_missing(&mut browsers, browser, || flatpak_app_installed(app_id));
    }

    browsers
}

#[cfg(target_os = "linux")]
fn linux_desktop_dirs() -> Vec<String> {
    let mut dirs = vec![
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        "/var/lib/flatpak/exports/share/applications".to_string(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        dirs.push(format!("{}/.local/share/applications", home));
        dirs.push(format!(
            "{}/.local/share/flatpak/exports/share/applications",
            home
        ));
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_data_dirs.split(':').filter(|d| !d.is_empty()) {
            dirs.push(format!("{}/applications", dir));
        }
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

#[cfg(target_os = "linux")]
fn browser_from_desktop_filename(name: &std::ffi::OsStr) -> Option<BrowserType> {
    let filename = name.to_str()?.to_lowercase();
    if !filename.ends_with(".desktop") {
        return None;
    }

    // Order matters: more specific names before generic ones (librewolf before firefox).
    if filename.contains("librewolf") {
        Some(BrowserType::LibreWolf)
    } else if filename.contains("google-chrome") || filename == "chrome.desktop" {
        Some(BrowserType::Chrome)
    } else if filename == "chromium.desktop"
        || filename == "chromium-browser.desktop"
        || filename.contains("org.chromium.chromium")
    {
        Some(BrowserType::Chromium)
    } else if filename.contains("firefox") {
        Some(BrowserType::Firefox)
    } else if filename.contains("brave") {
        Some(BrowserType::Brave)
    } else if filename.contains("opera") {
        Some(BrowserType::Opera)
    } else if filename.contains("microsoft-edge") || filename.contains("microsoft_edge") {
        Some(BrowserType::Edge)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn linux_flatpak_browsers() -> [(BrowserType, &'static str); 7] {
    [
        (BrowserType::LibreWolf, "io.gitlab.librewolf-community"),
        (BrowserType::Firefox, "org.mozilla.firefox"),
        (BrowserType::Chrome, "com.google.Chrome"),
        (BrowserType::Chromium, "org.chromium.Chromium"),
        (BrowserType::Brave, "com.brave.Browser"),
        (BrowserType::Opera, "com.opera.Opera"),
        (BrowserType::Edge, "com.microsoft.Edge"),
    ]
}

#[cfg(target_os = "linux")]
fn add_if_missing(browsers: &mut Vec<BrowserType>, browser: BrowserType, detect: impl FnOnce() -> bool) {
    if !browsers.contains(&browser) && detect() {
        browsers.push(browser);
    }
}

#[cfg(target_os = "linux")]
fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn flatpak_app_installed(app_id: &str) -> bool {
    Command::new("flatpak")
        .args(["info", app_id])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub fn get_default_browser() -> Option<BrowserType> {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return browser_from_default_id(&stdout);
    }

    None
}

#[cfg(target_os = "linux")]
fn browser_from_default_id(value: &str) -> Option<BrowserType> {
    // Order matters: librewolf before firefox, chromium before chrome.
    if value.contains("librewolf") {
        Some(BrowserType::LibreWolf)
    } else if value.contains("chromium") {
        Some(BrowserType::Chromium)
    } else if value.contains("chrome") {
        Some(BrowserType::Chrome)
    } else if value.contains("firefox") {
        Some(BrowserType::Firefox)
    } else if value.contains("brave") {
        Some(BrowserType::Brave)
    } else if value.contains("opera") {
        Some(BrowserType::Opera)
    } else if value.contains("edge") {
        Some(BrowserType::Edge)
    } else {
        None
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn parses_librewolf_default_desktop() {
        assert_eq!(
            browser_from_default_id("io.gitlab.librewolf-community.desktop"),
            Some(BrowserType::LibreWolf)
        );
    }

    #[test]
    fn parses_librewolf_desktop_filename() {
        assert_eq!(
            browser_from_desktop_filename(std::ffi::OsStr::new(
                "io.gitlab.librewolf-community.desktop"
            )),
            Some(BrowserType::LibreWolf)
        );
    }

    #[test]
    fn detects_installed_and_default_on_this_host() {
        let installed = get_installed_browsers();
        let default = get_default_browser();

        // This machine's default is Flatpak LibreWolf; skip if the environment differs.
        if flatpak_app_installed("io.gitlab.librewolf-community") {
            assert!(
                installed.contains(&BrowserType::LibreWolf),
                "expected LibreWolf in {:?}",
                installed
            );
        }

        if let Some(default_browser) = default {
            assert!(
                installed.contains(&default_browser),
                "default {:?} missing from installed {:?}",
                default_browser,
                installed
            );
        }
    }
}
