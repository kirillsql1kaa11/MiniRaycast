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
    pub keywords: Option<Vec<String>>,
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
                title: "Блокировка экрана".to_string(),
                subtitle: "Заблокировать компьютер".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "lock".to_string(),
                badge: Some("Система".to_string()),
                keywords: Some(vec!["lock".to_string(), "экран".to_string(), "блок".to_string(), "компьютер".to_string()]),
            },
            LauncherItem {
                id: "sys_sleep".to_string(),
                title: "Спящий режим".to_string(),
                subtitle: "Перевести компьютер в спящий режим".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "sleep".to_string(),
                badge: Some("Питание".to_string()),
                keywords: Some(vec!["sleep".to_string(), "сон".to_string(), "спящий".to_string(), "ждущий".to_string()]),
            },
            LauncherItem {
                id: "sys_restart".to_string(),
                title: "Перезагрузка ПК".to_string(),
                subtitle: "Перезагрузить операционную систему Windows".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "restart".to_string(),
                badge: Some("Питание".to_string()),
                keywords: Some(vec!["restart".to_string(), "reboot".to_string(), "перезагрузка".to_string(), "рестарт".to_string()]),
            },
            LauncherItem {
                id: "sys_shutdown".to_string(),
                title: "Завершение работы".to_string(),
                subtitle: "Выключить компьютер".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "shutdown".to_string(),
                badge: Some("Питание".to_string()),
                keywords: Some(vec!["shutdown".to_string(), "power".to_string(), "выключить".to_string(), "выключение".to_string()]),
            },
            LauncherItem {
                id: "sys_empty_trash".to_string(),
                title: "Очистить корзину".to_string(),
                subtitle: "Удалить все файлы из корзины".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "empty_trash".to_string(),
                badge: Some("Очистка".to_string()),
                keywords: Some(vec!["trash".to_string(), "recycle".to_string(), "корзина".to_string(), "очистить".to_string(), "удалить".to_string()]),
            },
            LauncherItem {
                id: "sys_volume_up".to_string(),
                title: "Увеличить громкость".to_string(),
                subtitle: "Прибавить общую громкость звука".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "volume_up".to_string(),
                badge: Some("Звук".to_string()),
                keywords: Some(vec!["volume".to_string(), "sound".to_string(), "громкость".to_string(), "громче".to_string(), "звук".to_string(), "прибавить".to_string()]),
            },
            LauncherItem {
                id: "sys_volume_down".to_string(),
                title: "Уменьшить громкость".to_string(),
                subtitle: "Убавить общую громкость звука".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "volume_down".to_string(),
                badge: Some("Звук".to_string()),
                keywords: Some(vec!["volume".to_string(), "sound".to_string(), "громкость".to_string(), "тише".to_string(), "звук".to_string(), "убавить".to_string()]),
            },
            LauncherItem {
                id: "sys_mute".to_string(),
                title: "Без звука / Включить звук".to_string(),
                subtitle: "Переключить режим звука".to_string(),
                item_type: "system".to_string(),
                action: "system_control".to_string(),
                payload: "mute".to_string(),
                badge: Some("Звук".to_string()),
                keywords: Some(vec!["mute".to_string(), "sound".to_string(), "volume".to_string(), "без звука".to_string(), "выключить звук".to_string(), "заглушить".to_string()]),
            },
            LauncherItem {
                id: "app_taskmgr".to_string(),
                title: "Диспетчер задач".to_string(),
                subtitle: "Мониторинг процессов и ресурсов Windows".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "taskmgr.exe".to_string(),
                badge: Some("Утилита".to_string()),
                keywords: Some(vec!["task".to_string(), "taskmgr".to_string(), "диспетчер".to_string(), "процессы".to_string(), "память".to_string()]),
            },
            LauncherItem {
                id: "app_calc".to_string(),
                title: "Калькулятор Windows".to_string(),
                subtitle: "Встроенный калькулятор (calc.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "calc.exe".to_string(),
                badge: Some("Утилита".to_string()),
                keywords: Some(vec!["calc".to_string(), "calculator".to_string(), "калькулятор".to_string(), "счет".to_string()]),
            },
            LauncherItem {
                id: "app_notepad".to_string(),
                title: "Блокнот".to_string(),
                subtitle: "Текстовый редактор (notepad.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "notepad.exe".to_string(),
                badge: Some("Утилита".to_string()),
                keywords: Some(vec!["notepad".to_string(), "блокнот".to_string(), "текст".to_string(), "заметки".to_string()]),
            },
            LauncherItem {
                id: "app_cmd".to_string(),
                title: "Командная строка".to_string(),
                subtitle: "Классическая консоль Windows (cmd.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "cmd.exe".to_string(),
                badge: Some("Терминал".to_string()),
                keywords: Some(vec!["cmd".to_string(), "терминал".to_string(), "консоль".to_string(), "командная строка".to_string()]),
            },
            LauncherItem {
                id: "app_powershell".to_string(),
                title: "PowerShell".to_string(),
                subtitle: "Оболочка PowerShell (powershell.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "powershell.exe".to_string(),
                badge: Some("Терминал".to_string()),
                keywords: Some(vec!["powershell".to_string(), "ps".to_string(), "терминал".to_string(), "консоль".to_string()]),
            },
            LauncherItem {
                id: "app_regedit".to_string(),
                title: "Редактор реестра".to_string(),
                subtitle: "Системный реестр (regedit.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "regedit.exe".to_string(),
                badge: Some("Утилита".to_string()),
                keywords: Some(vec!["regedit".to_string(), "registry".to_string(), "реестр".to_string()]),
            },
            LauncherItem {
                id: "app_explorer".to_string(),
                title: "Проводник".to_string(),
                subtitle: "Файловый менеджер (explorer.exe)".to_string(),
                item_type: "app".to_string(),
                action: "launch".to_string(),
                payload: "explorer.exe".to_string(),
                badge: Some("Система".to_string()),
                keywords: Some(vec!["explorer".to_string(), "проводник".to_string(), "файлы".to_string(), "папки".to_string(), "диск".to_string()]),
            },
            LauncherItem {
                id: "app_settings".to_string(),
                title: "Параметры Windows".to_string(),
                subtitle: "Настройки операционной системы (ms-settings:)".to_string(),
                item_type: "app".to_string(),
                action: "open_uri".to_string(),
                payload: "ms-settings:".to_string(),
                badge: Some("Система".to_string()),
                keywords: Some(vec!["settings".to_string(), "параметры".to_string(), "настройки".to_string(), "опции".to_string()]),
            },
            LauncherItem {
                id: "hk_alt_space".to_string(),
                title: "Хоткей: Alt + Space".to_string(),
                subtitle: "Назначить вызов лаунчера на Alt + Space".to_string(),
                item_type: "system".to_string(),
                action: "set_hotkey".to_string(),
                payload: "Alt+Space".to_string(),
                badge: Some("Хоткей".to_string()),
                keywords: Some(vec!["hotkey".to_string(), "хоткей".to_string(), "клавиши".to_string(), "alt space".to_string()]),
            },
            LauncherItem {
                id: "hk_ctrl_space".to_string(),
                title: "Хоткей: Ctrl + Space".to_string(),
                subtitle: "Назначить вызов лаунчера на Ctrl + Space".to_string(),
                item_type: "system".to_string(),
                action: "set_hotkey".to_string(),
                payload: "Ctrl+Space".to_string(),
                badge: Some("Хоткей".to_string()),
                keywords: Some(vec!["hotkey".to_string(), "хоткей".to_string(), "клавиши".to_string(), "ctrl space".to_string()]),
            },
            LauncherItem {
                id: "hk_alt_k".to_string(),
                title: "Хоткей: Alt + K".to_string(),
                subtitle: "Назначить вызов лаунчера на Alt + K".to_string(),
                item_type: "system".to_string(),
                action: "set_hotkey".to_string(),
                payload: "Alt+K".to_string(),
                badge: Some("Хоткей".to_string()),
                keywords: Some(vec!["hotkey".to_string(), "хоткей".to_string(), "клавиши".to_string(), "alt k".to_string()]),
            },
            LauncherItem {
                id: "hk_ctrl_shift_space".to_string(),
                title: "Хоткей: Ctrl + Shift + Space".to_string(),
                subtitle: "Назначить вызов лаунчера на Ctrl + Shift + Space".to_string(),
                item_type: "system".to_string(),
                action: "set_hotkey".to_string(),
                payload: "Ctrl+Shift+Space".to_string(),
                badge: Some("Хоткей".to_string()),
                keywords: Some(vec!["hotkey".to_string(), "хоткей".to_string(), "клавиши".to_string(), "ctrl shift space".to_string()]),
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
        for entry in WalkDir::new(dir).follow_links(false).max_depth(4).into_iter().filter_map(|e| e.ok()) {
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
                        "Ярлык приложения".to_string()
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
                        badge: Some("Приложение".to_string()),
                        keywords: None,
                    });
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<LauncherItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            let mut top = Vec::new();
            top.extend(self.system_commands.iter().cloned());
            top.extend(self.cached_apps.iter().cloned());
            return top;
        }

        let mut results = Vec::new();

        for cmd in &self.system_commands {
            let mut matched = cmd.title.to_lowercase().contains(&q)
                || cmd.subtitle.to_lowercase().contains(&q)
                || cmd.payload.to_lowercase().contains(&q)
                || cmd.id.to_lowercase().contains(&q);

            if !matched {
                if let Some(kws) = &cmd.keywords {
                    matched = kws.iter().any(|k| k.to_lowercase().contains(&q));
                }
            }

            if matched {
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

        results
    }
}
