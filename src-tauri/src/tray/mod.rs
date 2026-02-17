use tauri::{
    AppHandle, Manager, Emitter,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState, TrayIcon},
    image::Image,
};

// Helper function to create menu
fn create_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let toggle_window = MenuItem::with_id(app, "toggle_window", "Toggle Window", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    Menu::with_items(app, &[&toggle_window, &settings, &quit])
        .map_err(|e| e.to_string())
}

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon<tauri::Wry>, String> {
    // Use platform-specific icon formats for best compatibility
    let icon_filename = if cfg!(target_os = "windows") {
        "icons/icon.ico"
    } else if cfg!(target_os = "macos") {
        "icons/icon.icns"
    } else {
        "icons/32x32.png"
    };

    // Try to load icon using Tauri's resource resolver (production)
    // Fall back to relative path (development)
    let icon_bytes = app.path()
        .resolve(icon_filename, tauri::path::BaseDirectory::Resource)
        .ok()
        .and_then(|path| std::fs::read(&path).ok())
        .or_else(|| {
            // Fallback for dev mode: load from source directory
            let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(icon_filename);
            std::fs::read(&dev_path).ok()
        })
        .ok_or_else(|| format!("Failed to load icon from {}", icon_filename))?;

    // Decode image to RGBA
    let img = image::load_from_memory(&icon_bytes)
        .map_err(|e| format!("Failed to decode icon: {}", e))?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    let icon = Image::new_owned(rgba, width, height);

    // Create menu
    let menu = create_menu(app)?;

    // Create the tray icon with tooltip
    let tray = TrayIconBuilder::with_id("main_tray")
        .icon(icon)
        .tooltip("Browser Scheduler")
        .menu(&menu)
        .show_menu_on_left_click(false)  // Only show menu on right-click
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
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
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
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(tray)
}
