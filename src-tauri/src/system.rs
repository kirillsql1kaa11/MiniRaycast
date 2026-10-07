use std::path::PathBuf;
use std::process::Command;

pub struct SystemManager {
    scripts_dir: PathBuf,
}

impl SystemManager {
    pub fn new() -> Self {
        let mut candidates = Vec::new();

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("scripts"));
                candidates.push(parent.join("../scripts"));
            }
        }

        candidates.push(PathBuf::from("scripts"));
        candidates.push(PathBuf::from("../scripts"));
        candidates.push(PathBuf::from("../../scripts"));

        let mut found = PathBuf::from("scripts");
        for c in candidates {
            if c.exists() {
                found = c;
                break;
            }
        }

        Self { scripts_dir: found }
    }

    pub fn execute_action(&self, action: &str, payload: &str) -> Result<(), String> {
        match action {
            "launch" => self.launch_app(payload),
            "system_control" => self.run_system_control(payload),
            "open_uri" => self.open_uri(payload),
            "copy" => Ok(()),
            "timer" => Ok(()),
            _ => Err(format!("Unsupported action: {}", action)),
        }
    }

    fn launch_app(&self, path: &str) -> Result<(), String> {
        let res = Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to launch {}: {}", path, e)),
        }
    }

    fn run_system_control(&self, control_action: &str) -> Result<(), String> {
        let script_path = self.scripts_dir.join("system_controls.ps1");
        let res = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_path.to_str().unwrap_or("scripts/system_controls.ps1"),
                "-Action",
                control_action,
            ])
            .spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to run control {}: {}", control_action, e)),
        }
    }

    fn open_uri(&self, uri: &str) -> Result<(), String> {
        let res = Command::new("cmd")
            .args(["/C", "start", uri])
            .spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open uri {}: {}", uri, e)),
        }
    }
}
