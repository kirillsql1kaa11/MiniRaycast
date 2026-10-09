use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
            "open_folder" => self.open_folder(payload),
            "system_control" => self.run_system_control(payload),
            "kill_process" => self.kill_process(payload),
            "network_control" => {
                let parts: Vec<&str> = payload.splitn(2, ':').collect();
                let act = parts[0];
                let target = if parts.len() > 1 { parts[1] } else { "" };
                self.run_network_control(act, target).map(|_| ())
            }
            "open_uri" => self.open_uri(payload),
            "copy" => Ok(()),
            "timer" => Ok(()),
            "set_theme" => Ok(()),
            "set_opacity" => Ok(()),
            "set_blur" => Ok(()),
            _ => Err(format!("Unsupported action: {}", action)),
        }
    }

    pub fn kill_process(&self, pid: &str) -> Result<(), String> {
        let mut cmd = Command::new("taskkill");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let res = cmd.args(["/F", "/PID", pid]).output();

        match res {
            Ok(out) => {
                if out.status.success() {
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to kill process {}: {}", pid, e)),
        }
    }

    pub fn get_processes(&self) -> Result<String, String> {
        let script_path = self.scripts_dir.join("process_manager.ps1");
        let mut cmd = Command::new("powershell");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let res = cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_str().unwrap_or("scripts/process_manager.ps1"),
            "-Action",
            "list",
        ]).output();

        match res {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                Ok(text)
            }
            Err(e) => Err(format!("Failed to get processes: {}", e)),
        }
    }

    pub fn run_network_control(&self, control_action: &str, target: &str) -> Result<String, String> {
        let script_path = self.scripts_dir.join("network_controls.ps1");
        let mut cmd = Command::new("powershell");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let mut args = vec![
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_str().unwrap_or("scripts/network_controls.ps1"),
            "-Action",
            control_action,
        ];

        if !target.is_empty() {
            args.push("-Target");
            args.push(target);
        }

        let res = cmd.args(args).output();

        match res {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                Ok(text)
            }
            Err(e) => Err(format!("Failed to run network control {}: {}", control_action, e)),
        }
    }

    fn open_folder(&self, path: &str) -> Result<(), String> {
        let mut cmd = Command::new("explorer");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let arg = format!("/select,{}", path);
        let res = cmd.arg(arg).spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open folder for {}: {}", path, e)),
        }
    }

    fn launch_app(&self, path: &str) -> Result<(), String> {
        let mut cmd = Command::new("cmd");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let res = cmd.args(["/C", "start", "", path]).spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to launch {}: {}", path, e)),
        }
    }

    fn run_system_control(&self, control_action: &str) -> Result<(), String> {
        let script_path = self.scripts_dir.join("system_controls.ps1");
        let mut cmd = Command::new("powershell");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let res = cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_str().unwrap_or("scripts/system_controls.ps1"),
            "-Action",
            control_action,
        ]).spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to run control {}: {}", control_action, e)),
        }
    }

    fn open_uri(&self, uri: &str) -> Result<(), String> {
        let mut cmd = Command::new("cmd");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let res = cmd.args(["/C", "start", uri]).spawn();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open uri {}: {}", uri, e)),
        }
    }
}
