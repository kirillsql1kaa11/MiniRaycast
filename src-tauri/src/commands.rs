use crate::db::{AppUsageRecord, HistoryRecord};
use crate::scanner::LauncherItem;
use crate::AppState;
use std::str::FromStr;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[tauri::command]
pub fn search(query: String, state: State<'_, AppState>) -> Vec<LauncherItem> {
    let mut items = Vec::new();
    let q = query.trim();

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
