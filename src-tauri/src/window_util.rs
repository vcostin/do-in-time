use tauri::WebviewWindow;

/// Show the main window after tray hide / start-minimized.
///
/// On Linux (especially KDE Wayland), `hide()` then `show()` can leave client-side
/// decoration buttons (minimize / maximize / close) unclickable until the title
/// bar is double-clicked. Toggling `resizable` forces a decoration refresh.
/// See https://github.com/tauri-apps/tauri/issues/11856
pub fn show_main_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    refresh_window_decorations(window);
}

pub fn refresh_window_decorations(window: &WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        let _ = window.set_resizable(false);
        let _ = window.set_resizable(true);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
    }
}
