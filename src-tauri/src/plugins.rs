use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use crate::scanner::LauncherItem;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct PluginRunner {
    plugins_dir: PathBuf,
}

impl PluginRunner {
    pub fn new() -> Self {
        let mut candidates = Vec::new();

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("plugins"));
                candidates.push(parent.join("../plugins"));
            }
        }

        candidates.push(PathBuf::from("plugins"));
        candidates.push(PathBuf::from("../plugins"));
        candidates.push(PathBuf::from("../../plugins"));

        let mut found = PathBuf::from("plugins");
        for c in candidates {
            if c.exists() {
                found = c;
                break;
            }
        }

        Self { plugins_dir: found }
    }

    fn run_python_script(&self, script_name: &str, arg: &str) -> Option<Value> {
        let script_path = self.plugins_dir.join(script_name);
        if !script_path.exists() {
            return None;
        }

        let mut cmd = Command::new("python");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        cmd.arg(&script_path).arg(arg);

        let output = cmd.output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(stdout.trim()).ok()
    }

    pub fn evaluate_query(&self, query: &str) -> Vec<LauncherItem> {
        let q = query.trim();
        if q.is_empty() {
            return Vec::new();
        }

        let q_lower = q.to_lowercase();
        let mut items = Vec::new();

        let has_digits = q.chars().any(|c| c.is_ascii_digit());
        let currency_keywords = ["usd", "eur", "rub", "gbp", "cny", "jpy", "kzt", "try", "cad", "chf", "aud", "btc", "eth", "$", "€", "£", "₽", "¥", "₸"];
        let is_currency_candidate = has_digits && currency_keywords.iter().any(|k| q_lower.contains(k));

        if is_currency_candidate {
            if let Some(json) = self.run_python_script("currency.py", q) {
                if json["success"].as_bool() == Some(true) {
                    if let Some(formatted) = json["formatted"].as_str() {
                        items.push(LauncherItem {
                            id: "currency_result".to_string(),
                            title: formatted.to_string(),
                            subtitle: "Press Enter to copy result".to_string(),
                            item_type: "currency".to_string(),
                            action: "copy".to_string(),
                            payload: formatted.to_string(),
                            badge: Some("Currency".to_string()),
                        });
                    }
                }
            }
        }

        let unit_keywords = ["km", "mi", "mile", "ft", "feet", "inch", "kg", "lb", "pound", "oz", "gb", "mb", "kb", "celsius", "fahrenheit"];
        let is_unit_candidate = has_digits && unit_keywords.iter().any(|k| q_lower.contains(k));

        if is_unit_candidate {
            if let Some(json) = self.run_python_script("units.py", q) {
                if json["success"].as_bool() == Some(true) {
                    if let Some(formatted) = json["formatted"].as_str() {
                        items.push(LauncherItem {
                            id: "units_result".to_string(),
                            title: formatted.to_string(),
                            subtitle: "Press Enter to copy result".to_string(),
                            item_type: "unit".to_string(),
                            action: "copy".to_string(),
                            payload: formatted.to_string(),
                            badge: Some("Unit".to_string()),
                        });
                    }
                }
            }
        }

        if q_lower.starts_with("timer") || q_lower.starts_with("таймер") {
            if let Some(json) = self.run_python_script("timer.py", q) {
                if json["success"].as_bool() == Some(true) {
                    if let (Some(formatted), Some(secs)) = (json["formatted"].as_str(), json["seconds"].as_i64()) {
                        items.push(LauncherItem {
                            id: "timer_result".to_string(),
                            title: format!("Set {}", formatted),
                            subtitle: "Press Enter to start countdown".to_string(),
                            item_type: "timer".to_string(),
                            action: "timer".to_string(),
                            payload: secs.to_string(),
                            badge: Some("Timer".to_string()),
                        });
                    }
                }
            }
        }

        let text_prefixes = ["base64 ", "b64 ", "unbase64 ", "upper ", "lower ", "reverse ", "rev ", "length ", "len "];
        if text_prefixes.iter().any(|p| q_lower.starts_with(p)) {
            if let Some(json) = self.run_python_script("text_tools.py", q) {
                if json["success"].as_bool() == Some(true) {
                    if let (Some(result), Some(label)) = (json["result"].as_str(), json["label"].as_str()) {
                        items.push(LauncherItem {
                            id: "text_tool_result".to_string(),
                            title: result.to_string(),
                            subtitle: format!("{}: {}", label, q),
                            item_type: "calc".to_string(),
                            action: "copy".to_string(),
                            payload: result.to_string(),
                            badge: Some("Text".to_string()),
                        });
                    }
                }
            }
        }

        if items.is_empty() {
            let has_math_operators = q.chars().any(|c| "+-*/^%".contains(c));
            if has_digits && has_math_operators {
                if let Some(json) = self.run_python_script("calculator.py", q) {
                    if json["success"].as_bool() == Some(true) {
                        if let Some(result) = json["result"].as_str() {
                            items.push(LauncherItem {
                                id: "calc_result".to_string(),
                                title: format!("= {}", result),
                                subtitle: format!("Expression: {}", q),
                                item_type: "calc".to_string(),
                                action: "copy".to_string(),
                                payload: result.to_string(),
                                badge: Some("Calculator".to_string()),
                            });
                        }
                    }
                }
            }
        }

        items
    }
}
