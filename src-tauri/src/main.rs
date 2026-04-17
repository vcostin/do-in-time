// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Disable DMA-BUF renderer in WebKitGTK — causes rendering artifacts on ARM/Wayland
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    do_in_time_lib::run()
}
