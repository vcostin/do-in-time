use crate::db::BrowserType;
use crate::utils::browser_detector;

#[tauri::command]
pub fn get_installed_browsers() -> Vec<BrowserType> {
    browser_detector::get_installed_browsers()
}

#[tauri::command]
pub fn get_default_browser() -> Option<BrowserType> {
    browser_detector::get_default_browser()
}
