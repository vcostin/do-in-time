use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

// Helper function to create menu
fn create_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let toggle_window =
        MenuItem::with_id(app, "toggle_window", "Toggle Window", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit =
        MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?;

    Menu::with_items(app, &[&toggle_window, &settings, &quit]).map_err(|e| e.to_string())
}

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon<tauri::Wry>, String> {
    let icon = tauri::include_image!("icons/32x32.png");

    // Create menu
    let menu = create_menu(app)?;

    // Create the tray icon with tooltip
    let tray = TrayIconBuilder::with_id("main_tray")
        .icon(icon)
        .tooltip("Browser Scheduler")
        .menu(&menu)
        .show_menu_on_left_click(false) // Only show menu on right-click
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "toggle_window" => {
                    // Toggle window visibility
                    if let Some(window) = app.get_webview_window("main") {
                        match window.is_visible() {
                            Ok(true) => {
                                let _ = window.hide();
                            }
                            Ok(false) | Err(_) => {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.unminimize();
                            }
                        }
                    }
                }
                "settings" => {
                    // Emit event to frontend to open settings modal
                    let _ = app.emit("open-settings", ());

                    // Also show the window to display the settings
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Left-click: toggle window visibility
                if let Some(window) = app.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(true) => {
                            let _ = window.hide();
                        }
                        Ok(false) | Err(_) => {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.unminimize();
                        }
                    }
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(tray)
}
