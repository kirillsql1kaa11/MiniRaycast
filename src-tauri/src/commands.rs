use crate::db::{AppUsageRecord, HistoryRecord};
use crate::scanner::LauncherItem;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub opacity: String,
    pub blur: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ProcessJsonItem {
    pub id: u32,
    pub name: String,
    pub ram_mb: f64,
    pub cpu: f64,
    pub path: Option<String>,
}

#[tauri::command]
pub fn search(query: String, state: State<'_, AppState>) -> Vec<LauncherItem> {
    let mut items = Vec::new();
    let q = query.trim();
    let q_lower = q.to_lowercase();

    if q_lower == "kill"
        || q_lower == "закрыть"
        || q_lower == "процессы"
        || q_lower == "процесс"
        || q_lower.starts_with("kill ")
        || q_lower.starts_with("закрыть ")
    {
        let filter = if q_lower.starts_with("kill ") {
            q_lower["kill ".len()..].trim()
        } else if q_lower.starts_with("закрыть ") {
            q_lower["закрыть ".len()..].trim()
        } else {
            ""
        };

        if let Ok(json_str) = state.system.get_processes() {
            if let Ok(procs) = serde_json::from_str::<Vec<ProcessJsonItem>>(&json_str) {
                for p in procs {
                    let name_match = filter.is_empty()
                        || p.name.to_lowercase().contains(filter)
                        || p.id.to_string().contains(filter);
                    if name_match {
                        items.push(LauncherItem {
                            id: format!("proc_{}", p.id),
                            title: format!("{} (PID: {})", p.name, p.id),
                            subtitle: format!("RAM: {:.1} МБ | CPU: {:.1}%", p.ram_mb, p.cpu),
                            item_type: "process".to_string(),
                            action: "kill_process".to_string(),
                            payload: p.id.to_string(),
                            badge: Some("Процесс".to_string()),
                            keywords: Some(vec![
                                p.name.clone(),
                                format!("{:.1}", p.ram_mb),
                                format!("{:.1}", p.cpu),
                                p.path.unwrap_or_default(),
                            ]),
                        });
                    }
                }
            }
        }
        return items;
    }

    if q_lower.starts_with("ping ") || q_lower.starts_with("пинг ") {
        let target = if q_lower.starts_with("ping ") {
            q["ping ".len()..].trim()
        } else {
            q["пинг ".len()..].trim()
        };
        if !target.is_empty() {
            items.push(LauncherItem {
                id: format!("net_ping_{}", target),
                title: format!("Пинг {}", target),
                subtitle: format!("Проверить задержку сети до {}", target),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: format!("ping:{}", target),
                badge: Some("Пинг".to_string()),
                keywords: None,
            });
        }
    }

    if !q.is_empty() {
        let plugin_results = state.plugins.evaluate_query(q);
        items.extend(plugin_results);
    }

    if let Ok(scanner) = state.scanner.lock() {
        let scanned = scanner.search(q);
        items.extend(scanned);
    }

    if q.is_empty() {
        if let Ok(db) = state.db.lock() {
            if let Ok(top) = db.get_top_apps(5) {
                for app in top {
                    items.insert(
                        0,
                        LauncherItem {
                            id: format!("frec_{}", app.item_id),
                            title: app.title,
                            subtitle: format!("Запусков: {}", app.launch_count),
                            item_type: app.item_type,
                            action: "launch".to_string(),
                            payload: app.path,
                            badge: Some("Часто".to_string()),
                            keywords: None,
                        },
                    );
                }
            }
        }
    }

    items.truncate(60);
    items
}

#[tauri::command]
pub fn execute_item(
    item: LauncherItem,
    query: Option<String>,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(q) = query {
        if !q.trim().is_empty() {
            if let Ok(db) = state.db.lock() {
                db.record_search(q.trim()).ok();
            }
        }
    }

    if item.action == "set_hotkey" {
        let app = window.app_handle().clone();
        change_hotkey(item.payload.clone(), app, state)?;
        window.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }

    if item.action == "kill_process" {
        state.system.kill_process(&item.payload)?;
        return Ok(());
    }

    if item.action == "set_theme" {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.set_setting("theme", &item.payload).map_err(|e| e.to_string())?;
        return Ok(());
    }

    if item.action == "fill_search" || item.action == "copy" || item.action == "timer" || item.action == "speedtest" {
        return Ok(());
    }

    if item.action == "toggle_autostart" {
        let cur = crate::system::is_autostart_enabled();
        crate::system::set_autostart_enabled(!cur)?;
        return Ok(());
    }

    if item.item_type == "app" {
        if let Ok(db) = state.db.lock() {
            db.record_launch(&item.id, &item.title, &item.payload, &item.item_type).ok();
        }
    }

    state.system.execute_action(&item.action, &item.payload)?;

    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_appearance_settings(state: State<'_, AppState>) -> Result<AppearanceSettings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let theme = db.get_setting("theme").ok().flatten().unwrap_or_else(|| "oled".to_string());
    let opacity = db.get_setting("opacity").ok().flatten().unwrap_or_else(|| "94".to_string());
    let blur = db.get_setting("blur").ok().flatten().unwrap_or_else(|| "32".to_string());
    Ok(AppearanceSettings { theme, opacity, blur })
}

#[tauri::command]
pub fn set_appearance_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn kill_process_by_pid(pid: String, state: State<'_, AppState>) -> Result<(), String> {
    state.system.kill_process(&pid)
}

#[tauri::command]
pub fn run_network_command(
    action: String,
    target: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let t = target.unwrap_or_default();
    state.system.run_network_control(&action, &t)
}

#[tauri::command]
pub fn open_folder_path(path: String, state: State<'_, AppState>) -> Result<(), String> {
    state.system.execute_action("open_folder", &path)
}

#[tauri::command]
pub fn run_speedtest_command(state: State<'_, AppState>) -> Result<String, String> {
    state.system.run_speedtest()
}

#[tauri::command]
pub fn get_autostart_status() -> bool {
    crate::system::is_autostart_enabled()
}

#[tauri::command]
pub fn toggle_autostart() -> Result<bool, String> {
    let current = crate::system::is_autostart_enabled();
    let new_state = !current;
    crate::system::set_autostart_enabled(new_state)?;
    Ok(new_state)
}

#[tauri::command]
pub fn get_history(state: State<'_, AppState>) -> Result<Vec<HistoryRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_history(20).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_top_apps(state: State<'_, AppState>) -> Result<Vec<AppUsageRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_top_apps(10).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_all_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_window(window: WebviewWindow) -> Result<(), String> {
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        window.hide().map_err(|e| e.to_string())?;
    } else {
        crate::position_bottom_window(&window);
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        crate::force_focus_window(&window);
    }
    Ok(())
}

#[tauri::command]
pub fn resize_and_position(window: WebviewWindow) -> Result<(), String> {
    crate::position_bottom_window(&window);
    Ok(())
}

#[tauri::command]
pub fn get_current_hotkey(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let hk = db.get_setting("hotkey").map_err(|e| e.to_string())?;
    Ok(hk.unwrap_or_else(|| "Alt+Space".to_string()))
}

#[tauri::command]
pub fn change_hotkey(
    new_hotkey: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let new_shortcut = Shortcut::from_str(&new_hotkey).map_err(|e| format!("Invalid shortcut: {}", e))?;
    
    let old_hotkey = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("hotkey")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "Alt+Space".to_string())
    };

    if let Ok(old_shortcut) = Shortcut::from_str(&old_hotkey) {
        app.global_shortcut().unregister(old_shortcut).ok();
    }

    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|e| format!("Failed to register {}: {}", new_hotkey, e))?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting("hotkey", &new_hotkey).map_err(|e| e.to_string())?;

    Ok(())
}
