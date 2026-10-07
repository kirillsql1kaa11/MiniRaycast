pub mod commands;
pub mod db;
pub mod plugins;
pub mod scanner;
pub mod system;

use db::Database;
use plugins::PluginRunner;
use scanner::Scanner;
use system::SystemManager;
use tauri::{Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::str::FromStr;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
    pub scanner: Mutex<Scanner>,
    pub plugins: PluginRunner,
    pub system: SystemManager,
}

#[cfg(target_os = "windows")]
pub fn force_focus_window(window: &WebviewWindow) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
    };

    if let Ok(hwnd) = window.hwnd() {
        let hwnd_raw = hwnd.0 as HWND;
        unsafe {
            ShowWindow(hwnd_raw, SW_RESTORE);
            ShowWindow(hwnd_raw, SW_SHOW);
            BringWindowToTop(hwnd_raw);
            SetForegroundWindow(hwnd_raw);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn force_focus_window(window: &WebviewWindow) {
    let _ = window.set_focus();
}

pub fn run() {
    let db_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("MiniRaycast");

    let db = Database::new(db_dir).expect("Failed to initialize database");
    let scanner = Scanner::new();
    let plugins = PluginRunner::new();
    let system = SystemManager::new();

    let state = AppState {
        db: Mutex::new(db),
        scanner: Mutex::new(scanner),
        plugins,
        system,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                window.hide().ok();
                            } else {
                                window.show().ok();
                                window.set_focus().ok();
                                force_focus_window(&window);
                            }
                        }
                    }
                })
                .build(),
        )
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::execute_item,
            commands::get_history,
            commands::get_top_apps,
            commands::clear_history,
            commands::hide_window,
            commands::toggle_window
        ])
        .setup(|app| {
            if let Ok(shortcut) = Shortcut::from_str("Alt+Space") {
                app.global_shortcut().register(shortcut).ok();
            }

            if let Some(window) = app.get_webview_window("main") {
                window.show().ok();
                window.set_focus().ok();
                force_focus_window(&window);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
