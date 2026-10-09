use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use sublime_fuzzy::best_match;
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
    cached_recent_docs: Vec<LauncherItem>,
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
            LauncherItem {
                id: "cmd_kill_processes".to_string(),
                title: "Завершение зависших процессов".to_string(),
                subtitle: "Диспетчер процессов с RAM и CPU (команда kill)".to_string(),
                item_type: "process".to_string(),
                action: "fill_search".to_string(),
                payload: "kill ".to_string(),
                badge: Some("Процессы".to_string()),
                keywords: Some(vec!["kill".to_string(), "закрыть".to_string(), "процессы".to_string(), "память".to_string(), "cpu".to_string(), "ram".to_string(), "завершить".to_string()]),
            },
            LauncherItem {
                id: "net_speedtest".to_string(),
                title: "Скорость интернета (Speedtest)".to_string(),
                subtitle: "Замер пинга, входящей и исходящей скорости без рекламы".to_string(),
                item_type: "network".to_string(),
                action: "speedtest".to_string(),
                payload: "speedtest".to_string(),
                badge: Some("Тест".to_string()),
                keywords: Some(vec![
                    "speedtest".to_string(),
                    "скорость".to_string(),
                    "скорость интернета".to_string(),
                    "интернет".to_string(),
                    "speed".to_string(),
                    "тест".to_string(),
                    "замер".to_string(),
                ]),
            },
            LauncherItem {
                id: "cmd_autostart_toggle".to_string(),
                title: "Автозагрузка вместе с системой".to_string(),
                subtitle: "Включить или отключить автозапуск приложения через реестр Windows".to_string(),
                item_type: "system".to_string(),
                action: "toggle_autostart".to_string(),
                payload: "autostart".to_string(),
                badge: Some("Система".to_string()),
                keywords: Some(vec![
                    "автозагрузка".to_string(),
                    "автозапуск".to_string(),
                    "autostart".to_string(),
                    "реестр".to_string(),
                    "старт".to_string(),
                ]),
            },
            LauncherItem {
                id: "cmd_recent_docs".to_string(),
                title: "Недавние документы и файлы".to_string(),
                subtitle: "Открыть список недавних файлов Word, Excel, PDF и проектов".to_string(),
                item_type: "document".to_string(),
                action: "fill_search".to_string(),
                payload: "recent".to_string(),
                badge: Some("Файлы".to_string()),
                keywords: Some(vec![
                    "recent".to_string(),
                    "недавние".to_string(),
                    "документы".to_string(),
                    "файлы".to_string(),
                    "открыть".to_string(),
                ]),
            },
            LauncherItem {
                id: "net_flushdns".to_string(),
                title: "Сброс кэша DNS (flushdns)".to_string(),
                subtitle: "Очистить кэш DNS-клиента (ipconfig /flushdns)".to_string(),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: "flushdns".to_string(),
                badge: Some("Сеть".to_string()),
                keywords: Some(vec!["flushdns".to_string(), "dns".to_string(), "сброс dns".to_string(), "кэш".to_string(), "сеть".to_string(), "очистить dns".to_string()]),
            },
            LauncherItem {
                id: "net_local_ip".to_string(),
                title: "Мой локальный IP".to_string(),
                subtitle: "Определить IPv4 адрес локальной сети".to_string(),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: "get_ip:local".to_string(),
                badge: Some("Сеть".to_string()),
                keywords: Some(vec!["ip".to_string(), "мой ip".to_string(), "my ip".to_string(), "локальный ip".to_string(), "сеть".to_string(), "адрес".to_string()]),
            },
            LauncherItem {
                id: "net_external_ip".to_string(),
                title: "Мой внешний IP (Интернет)".to_string(),
                subtitle: "Определить публичный интернет-адрес".to_string(),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: "get_ip:external".to_string(),
                badge: Some("Интернет".to_string()),
                keywords: Some(vec!["ip".to_string(), "мой ip".to_string(), "my ip".to_string(), "внешний ip".to_string(), "интернет".to_string(), "публичный".to_string()]),
            },
            LauncherItem {
                id: "net_ping".to_string(),
                title: "Проверка пинга (ya.ru)".to_string(),
                subtitle: "Проверить сетевой отклик до ya.ru".to_string(),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: "ping:ya.ru".to_string(),
                badge: Some("Пинг".to_string()),
                keywords: Some(vec!["ping".to_string(), "пинг".to_string(), "связь".to_string(), "сеть".to_string(), "задержка".to_string(), "ya.ru".to_string()]),
            },
            LauncherItem {
                id: "net_bt_toggle".to_string(),
                title: "Переключить Bluetooth".to_string(),
                subtitle: "Включить или выключить службу Bluetooth".to_string(),
                item_type: "network".to_string(),
                action: "network_control".to_string(),
                payload: "bluetooth_toggle".to_string(),
                badge: Some("Устройство".to_string()),
                keywords: Some(vec!["bluetooth".to_string(), "блютуз".to_string(), "bt".to_string(), "устройства".to_string()]),
            },
            LauncherItem {
                id: "theme_oled".to_string(),
                title: "Тема: OLED Black".to_string(),
                subtitle: "Глубокая черная тема с максимальным контрастом".to_string(),
                item_type: "theme".to_string(),
                action: "set_theme".to_string(),
                payload: "oled".to_string(),
                badge: Some("Тема".to_string()),
                keywords: Some(vec!["theme".to_string(), "тема".to_string(), "oled".to_string(), "black".to_string(), "черный".to_string()]),
            },
            LauncherItem {
                id: "theme_midnight".to_string(),
                title: "Тема: Midnight Blue".to_string(),
                subtitle: "Темно-синяя ночная тема".to_string(),
                item_type: "theme".to_string(),
                action: "set_theme".to_string(),
                payload: "midnight".to_string(),
                badge: Some("Тема".to_string()),
                keywords: Some(vec!["theme".to_string(), "тема".to_string(), "midnight".to_string(), "blue".to_string(), "синий".to_string()]),
            },
            LauncherItem {
                id: "theme_slate".to_string(),
                title: "Тема: Slate Gray".to_string(),
                subtitle: "Сдержанная графитово-серая тема".to_string(),
                item_type: "theme".to_string(),
                action: "set_theme".to_string(),
                payload: "slate".to_string(),
                badge: Some("Тема".to_string()),
                keywords: Some(vec!["theme".to_string(), "тема".to_string(), "slate".to_string(), "gray".to_string(), "серый".to_string()]),
            },
            LauncherItem {
                id: "theme_monokai".to_string(),
                title: "Тема: Monokai".to_string(),
                subtitle: "Культовая темная палитра для разработчиков".to_string(),
                item_type: "theme".to_string(),
                action: "set_theme".to_string(),
                payload: "monokai".to_string(),
                badge: Some("Тема".to_string()),
                keywords: Some(vec!["theme".to_string(), "тема".to_string(), "monokai".to_string(), "монокай".to_string()]),
            },
        ];

        let mut scanner = Self {
            cached_apps: Vec::new(),
            system_commands,
            cached_recent_docs: Vec::new(),
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
        self.scan_recent_documents();
    }

    fn scan_recent_documents(&mut self) {
        let mut docs = Vec::new();
        let mut recent_dir = None;

        if let Ok(app_data) = env::var("APPDATA") {
            let p = PathBuf::from(app_data).join("Microsoft\\Windows\\Recent");
            if p.exists() {
                recent_dir = Some(p);
            }
        }

        if let Some(dir) = recent_dir {
            let mut entries: Vec<_> = WalkDir::new(dir)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .collect();

            entries.sort_by(|a, b| {
                let time_a = a.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let time_b = b.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                time_b.cmp(&time_a)
            });

            for entry in entries.into_iter().take(50) {
                let path = entry.path();
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if file_name.is_empty() || !file_name.ends_with(".lnk") {
                    continue;
                }

                let clean_name = file_name.trim_end_matches(".lnk");
                let path_str = path.to_string_lossy().to_string();

                let ext = Path::new(clean_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let (badge, doc_type) = match ext.as_str() {
                    "docx" | "doc" => ("Word", "Документ Word"),
                    "xlsx" | "xls" | "csv" => ("Excel", "Таблица Excel"),
                    "pptx" | "ppt" => ("PowerPoint", "Презентация PowerPoint"),
                    "pdf" => ("PDF", "Документ PDF"),
                    "txt" | "md" | "json" | "rs" | "ts" | "py" => ("Текст", "Текстовый файл"),
                    "jpg" | "png" | "jpeg" | "webp" | "gif" => ("Фото", "Изображение"),
                    "zip" | "rar" | "7z" => ("Архив", "Архив"),
                    "mp4" | "mkv" | "avi" => ("Видео", "Видеофайл"),
                    "mp3" | "wav" | "flac" => ("Аудио", "Аудиозапись"),
                    _ => ("Файл", "Недавний документ"),
                };

                docs.push(LauncherItem {
                    id: format!("recent_{}", clean_name),
                    title: clean_name.to_string(),
                    subtitle: format!("{} (Недавний файл)", doc_type),
                    item_type: "document".to_string(),
                    action: "launch".to_string(),
                    payload: path_str,
                    badge: Some(badge.to_string()),
                    keywords: Some(vec![
                        "recent".to_string(),
                        "недавние".to_string(),
                        "документы".to_string(),
                        "файлы".to_string(),
                        ext,
                        clean_name.to_string(),
                    ]),
                });
            }
        }

        self.cached_recent_docs = docs;
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

    fn calculate_score(query: &str, item: &LauncherItem) -> Option<isize> {
        let title_lower = item.title.to_lowercase();
        let mut best: Option<isize> = None;

        if title_lower == query {
            return Some(6000);
        }

        if title_lower.starts_with(query) {
            let s = 3500 - (title_lower.len() as isize * 5);
            best = Some(best.map_or(s, |p| p.max(s)));
        } else if title_lower.contains(query) {
            let s = 2500 - (title_lower.len() as isize * 5);
            best = Some(best.map_or(s, |p| p.max(s)));
        }

        if let Some(m) = best_match(query, &title_lower) {
            let s = m.score() * 4;
            best = Some(best.map_or(s, |p| p.max(s)));
        }

        let sub_lower = item.subtitle.to_lowercase();
        if sub_lower.contains(query) {
            let s = 1200;
            best = Some(best.map_or(s, |p| p.max(s)));
        } else if let Some(m) = best_match(query, &sub_lower) {
            let s = m.score() * 2;
            best = Some(best.map_or(s, |p| p.max(s)));
        }

        if let Some(kws) = &item.keywords {
            for kw in kws {
                let kw_lower = kw.to_lowercase();
                if kw_lower == query || kw_lower.starts_with(query) {
                    let s = 1800;
                    best = Some(best.map_or(s, |p| p.max(s)));
                } else if kw_lower.contains(query) {
                    let s = 1400;
                    best = Some(best.map_or(s, |p| p.max(s)));
                } else if let Some(m) = best_match(query, &kw_lower) {
                    let s = m.score() * 2;
                    best = Some(best.map_or(s, |p| p.max(s)));
                }
            }
        }

        best
    }

    pub fn search(&self, query: &str) -> Vec<LauncherItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            let mut top = Vec::new();
            top.extend(self.system_commands.iter().cloned());
            top.extend(self.cached_recent_docs.iter().take(12).cloned());
            top.extend(self.cached_apps.iter().cloned());
            return top;
        }

        if q == "recent" || q == "недавние" || q == "документы" || q == "файлы" {
            return self.cached_recent_docs.clone();
        }

        let mut scored: Vec<(isize, LauncherItem)> = Vec::new();

        let all_items = self.system_commands.iter()
            .chain(self.cached_recent_docs.iter())
            .chain(self.cached_apps.iter());

        for item in all_items {
            if let Some(score) = Self::calculate_score(&q, item) {
                scored.push((score, item.clone()));
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, item)| item).collect()
    }
}
