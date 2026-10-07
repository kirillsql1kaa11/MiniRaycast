use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub item_type: String,
    pub action: String,
    pub payload: String,
    pub badge: Option<String>,
}

pub struct Scanner {
    cached_apps: Vec<LauncherItem>,
    system_commands: Vec<LauncherItem>,
}

impl Scanner {
    pub fn new() -> Self {
        let system_commands = vec![
            LauncherItem {
                id: "sys_lock".to_string(),
                title: "Lock Screen".to_string(),
                subtitle: "Lock this computer".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "lock".to_string(),
                badge: Some("System".to_string()),
            },
            LauncherItem {
                id: "sys_sleep".to_string(),
                title: "Sleep".to_string(),
                subtitle: "Put computer into sleep mode".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "sleep".to_string(),
                badge: Some("Power".to_string()),
            },
            LauncherItem {
                id: "sys_empty_trash".to_string(),
                title: "Empty Recycle Bin".to_string(),
                subtitle: "Permanently delete files in Recycle Bin".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "empty_trash".to_string(),
                badge: Some("Cleanup".to_string()),
            },
            LauncherItem {
                id: "sys_volume_up".to_string(),
                title: "Volume Up".to_string(),
                subtitle: "Increase system master volume".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "volume_up".to_string(),
                badge: Some("Audio".to_string()),
            },
            LauncherItem {
                id: "sys_volume_down".to_string(),
                title: "Volume Down".to_string(),
                subtitle: "Decrease system master volume".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "volume_down".to_string(),
                badge: Some("Audio".to_string()),
            },
            LauncherItem {
                id: "sys_mute".to_string(),
                title: "Mute Audio".to_string(),
                subtitle: "Toggle mute on/off".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "mute".to_string(),
                badge: Some("Audio".to_string()),
            },
            LauncherItem {
                id: "sys_restart".to_string(),
                title: "Restart Computer".to_string(),
                subtitle: "Reboot Windows".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "restart".to_string(),
                badge: Some("Power".to_string()),
            },
            LauncherItem {
                id: "sys_shutdown".to_string(),
                title: "Shutdown Computer".to_string(),
                subtitle: "Turn off PC".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "shutdown".to_string(),
                badge: Some("Power".to_string()),
            },
            LauncherItem {
                id: "app_taskmgr".to_string(),
                title: "Task Manager".to_string(),
                subtitle: "View running processes and performance".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "taskmgr.exe".to_string(),
                badge: Some("App".to_string()),
            },
            LauncherItem {
                id: "app_calc".to_string(),
                title: "Windows Calculator".to_string(),
                subtitle: "calc.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "calc.exe".to_string(),
                badge: Some("App".to_string()),
            },
            LauncherItem {
                id: "app_notepad".to_string(),
                title: "Notepad".to_string(),
                subtitle: "notepad.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "notepad.exe".to_string(),
                badge: Some("App".to_string()),
            },
            LauncherItem {
                id: "app_cmd".to_string(),
                title: "Command Prompt".to_string(),
                subtitle: "cmd.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "cmd.exe".to_string(),
                badge: Some("Terminal".to_string()),
            },
            LauncherItem {
                id: "app_powershell".to_string(),
                title: "PowerShell".to_string(),
                subtitle: "powershell.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "powershell.exe".to_string(),
                badge: Some("Terminal".to_string()),
            },
            LauncherItem {
                id: "app_regedit".to_string(),
                title: "Registry Editor".to_string(),
                subtitle: "regedit.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "regedit.exe".to_string(),
                badge: Some("Tool".to_string()),
            },
            LauncherItem {
                id: "app_explorer".to_string(),
                title: "File Explorer".to_string(),
                subtitle: "explorer.exe".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "explorer.exe".to_string(),
                badge: Some("System".to_string()),
            },
            LauncherItem {
                id: "app_settings".to_string(),
                title: "Windows Settings".to_string(),
                subtitle: "ms-settings:".to_string(),
                item_type: "app".to_string(),
                action: "open_uri".to_string(),
                payload: "ms-settings:".to_string(),
                badge: Some("System".to_string()),
            },
        ];

        let mut scanner = Self {
            cached_apps: Vec::new(),
            system_commands,
        };
        scanner.refresh_cache();
        scanner
    }

    pub fn refresh_cache(&mut self) {
        let mut apps = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let mut scan_dirs = Vec::new();
        if let Ok(app_data) = env::var("APPDATA") {
            scan_dirs.push(PathBuf::from(app_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
        }
        if let Ok(prog_data) = env::var("ProgramData") {
            scan_dirs.push(PathBuf::from(prog_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
        }
        if let Ok(user_profile) = env::var("USERPROFILE") {
            scan_dirs.push(PathBuf::from(user_profile).join("Desktop"));
        }
        if let Ok(public) = env::var("PUBLIC") {
            scan_dirs.push(PathBuf::from(public).join("Desktop"));
        }

        for dir in scan_dirs {
            if dir.exists() {
                self.scan_directory(&dir, &mut apps, &mut seen);
            }
        }

        self.cached_apps = apps;
    }

    fn scan_directory(
        &self,
        dir: &Path,
        apps: &mut Vec<LauncherItem>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                if ext == "lnk" || ext == "exe" || ext == "url" {
                    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if file_stem.is_empty() || file_stem.to_lowercase().contains("uninstall") {
                        continue;
                    }

                    let clean_name = file_stem.to_string();
                    let key = clean_name.to_lowercase();
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.insert(key);

                    let path_str = path.to_string_lossy().to_string();
                    let subtitle = if ext == "lnk" {
                        "Shortcut".to_string()
                    } else {
                        path_str.clone()
                    };

                    apps.push(LauncherItem {
                        id: format!("app_{}", clean_name),
                        title: clean_name,
                        subtitle,
                        item_type: "app".to_string(),
                        action: "launch".to_string(),
                        payload: path_str,
                        badge: Some("App".to_string()),
                    });
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<LauncherItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            let mut top = Vec::new();
            top.extend(self.system_commands.iter().take(6).cloned());
            top.extend(self.cached_apps.iter().take(10).cloned());
            return top;
        }

        let mut results = Vec::new();

        for cmd in &self.system_commands {
            if cmd.title.to_lowercase().contains(&q) || cmd.subtitle.to_lowercase().contains(&q) {
                results.push(cmd.clone());
            }
        }

        for app in &self.cached_apps {
            let title_lower = app.title.to_lowercase();
            if title_lower.starts_with(&q) {
                results.push(app.clone());
            } else if title_lower.contains(&q) {
                results.push(app.clone());
            }
        }

        results.truncate(20);
        results
    }
}
