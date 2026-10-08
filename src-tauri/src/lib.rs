pub mod commands;
pub mod db;
pub mod plugins;
pub mod scanner;
pub mod system;

use db::Database;
use plugins::PluginRunner;
use scanner::Scanner;
use system::SystemManager;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

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

pub fn position_bottom_window(window: &WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale_factor = monitor.scale_factor();
        let screen_size = monitor.size().to_logical::<f64>(scale_factor);
        let screen_pos = monitor.position().to_logical::<f64>(scale_factor);

        let width = 740.0;
        let height = 540.0;

        let x = screen_pos.x + (screen_size.width - width) / 2.0;
        let y = screen_pos.y + screen_size.height - height - 60.0;

        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)));
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    }
}

fn update_hotkey_from_tray(app: &AppHandle, new_hotkey: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        let old_hotkey = if let Ok(db) = state.db.lock() {
            db.get_setting("hotkey")
                .ok()
                .flatten()
                .unwrap_or_else(|| "Alt+Space".to_string())
        } else {
            "Alt+Space".to_string()
        };

        if let Ok(old_sc) = Shortcut::from_str(&old_hotkey) {
            app.global_shortcut().unregister(old_sc).ok();
        }

        if let Ok(new_sc) = Shortcut::from_str(new_hotkey) {
            app.global_shortcut().register(new_sc).ok();
        }

        if let Ok(db) = state.db.lock() {
            db.set_setting("hotkey", new_hotkey).ok();
        }
    }
}

pub fn run() {
    let db_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("MiniRaycast");

    let db = Database::new(db_dir).expect("Failed to initialize database");
    let current_hotkey = db
        .get_setting("hotkey")
        .ok()
        .flatten()
        .unwrap_or_else(|| "Alt+Space".to_string());

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
                                position_bottom_window(&window);
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
            commands::toggle_window,
            commands::resize_and_position,
            commands::get_current_hotkey,
            commands::change_hotkey
        ])
        .setup(move |app| {
            if let Ok(shortcut) = Shortcut::from_str(&current_hotkey) {
                app.global_shortcut().register(shortcut).ok();
            }

            let toggle_item = MenuItem::with_id(app, "toggle", "Show MiniRaycast", true, None::<&str>)?;
            let hk_alt_space = CheckMenuItem::with_id(
                app,
                "hk_alt_space",
                "Alt + Space",
                true,
                current_hotkey == "Alt+Space",
                None::<&str>,
            )?;
            let hk_ctrl_space = CheckMenuItem::with_id(
                app,
                "hk_ctrl_space",
                "Ctrl + Space",
                true,
                current_hotkey == "Ctrl+Space",
                None::<&str>,
            )?;
            let hk_alt_k = CheckMenuItem::with_id(
                app,
                "hk_alt_k",
                "Alt + K",
                true,
                current_hotkey == "Alt+K",
                None::<&str>,
            )?;
            let hk_ctrl_shift = CheckMenuItem::with_id(
                app,
                "hk_ctrl_shift_space",
                "Ctrl + Shift + Space",
                true,
                current_hotkey == "Ctrl+Shift+Space",
                None::<&str>,
            )?;

            let hotkey_submenu = Submenu::with_items(
                app,
                "Change Hotkey",
                true,
                &[&hk_alt_space, &hk_ctrl_space, &hk_alt_k, &hk_ctrl_shift],
            )?;

            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit MiniRaycast", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&toggle_item, &hotkey_submenu, &sep, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let id_str = event.id().as_ref();
                    match id_str {
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let is_visible = window.is_visible().unwrap_or(false);
                                if is_visible {
                                    window.hide().ok();
                                } else {
                                    position_bottom_window(&window);
                                    window.show().ok();
                                    window.set_focus().ok();
                                    force_focus_window(&window);
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        "hk_alt_space" => {
                            update_hotkey_from_tray(app, "Alt+Space");
                        }
                        "hk_ctrl_space" => {
                            update_hotkey_from_tray(app, "Ctrl+Space");
                        }
                        "hk_alt_k" => {
                            update_hotkey_from_tray(app, "Alt+K");
                        }
                        "hk_ctrl_shift_space" => {
                            update_hotkey_from_tray(app, "Ctrl+Shift+Space");
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                window.hide().ok();
                            } else {
                                position_bottom_window(&window);
                                window.show().ok();
                                window.set_focus().ok();
                                force_focus_window(&window);
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            tray_builder.build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(false) => {
                            win_clone.hide().ok();
                        }
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            win_clone.hide().ok();
                        }
                        _ => {}
                    }
                });

                position_bottom_window(&window);
                window.show().ok();
                window.set_focus().ok();
                force_focus_window(&window);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
