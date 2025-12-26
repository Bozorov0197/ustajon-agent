// Process Manager module

use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, ProcessExt, PidExt, CpuExt};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub cmd: Vec<String>,
    pub cpu_usage: f32,
    pub memory: u64,
    pub memory_percent: f32,
    pub status: String,
    pub user: String,
    pub start_time: u64,
}

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }
    
    /// Refresh process list
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// List all processes
    pub fn list_all(&self) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
        let total_memory = self.system.total_memory() as f32;
        
        let processes: Vec<ProcessInfo> = self.system.processes()
            .iter()
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    path: process.exe().to_string_lossy().to_string(),
                    cmd: process.cmd().to_vec(),
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                    memory_percent: if total_memory > 0.0 {
                        (process.memory() as f32 / total_memory) * 100.0
                    } else {
                        0.0
                    },
                    status: format!("{:?}", process.status()),
                    user: process.user_id()
                        .map(|u| format!("{:?}", u))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    start_time: process.start_time(),
                }
            })
            .collect();
        
        Ok(processes)
    }
    
    /// Get process by PID
    pub fn get(&self, pid: u32) -> Option<ProcessInfo> {
        let total_memory = self.system.total_memory() as f32;
        
        self.system.process(sysinfo::Pid::from_u32(pid))
            .map(|process| {
                ProcessInfo {
                    pid,
                    name: process.name().to_string(),
                    path: process.exe().to_string_lossy().to_string(),
                    cmd: process.cmd().to_vec(),
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                    memory_percent: if total_memory > 0.0 {
                        (process.memory() as f32 / total_memory) * 100.0
                    } else {
                        0.0
                    },
                    status: format!("{:?}", process.status()),
                    user: process.user_id()
                        .map(|u| format!("{:?}", u))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    start_time: process.start_time(),
                }
            })
    }
    
    /// Kill process by PID
    pub fn kill(&self, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
            process.kill();
            Ok(())
        } else {
            // Try using taskkill on Windows
            #[cfg(windows)]
            {
                Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .output()?;
            }
            
            #[cfg(not(windows))]
            {
                Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()?;
            }
            
            Ok(())
        }
    }
    
    /// Kill process by name
    pub fn kill_by_name(&self, name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let mut killed = 0u32;
        
        for (_pid, process) in self.system.processes() {
            if process.name().to_lowercase() == name.to_lowercase() {
                process.kill();
                killed += 1;
            }
        }
        
        // Also try system command
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", name])
                .output();
        }
        
        Ok(killed)
    }
    
    /// Start a process
    pub fn start(&self, path: &str, args: &[&str]) -> Result<u32, Box<dyn std::error::Error>> {
        let child = Command::new(path)
            .args(args)
            .spawn()?;
        
        Ok(child.id())
    }
    
    /// Run shell command
    pub async fn run_command(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        #[cfg(not(windows))]
        let output = Command::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        Ok(format!("{}{}", stdout, stderr))
    }
    
    /// Run PowerShell command (Windows)
    pub async fn run_powershell(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("powershell")
                .args(["-NoProfile", "-Command", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            Ok(format!("{}{}", stdout, stderr))
        }
        
        #[cfg(not(windows))]
        {
            Err("PowerShell is only available on Windows".into())
        }
    }
    
    /// Get top processes by CPU usage
    pub fn top_by_cpu(&self, count: usize) -> Vec<ProcessInfo> {
        let mut processes = self.list_all().unwrap_or_default();
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        processes.into_iter().take(count).collect()
    }
    
    /// Get top processes by memory usage
    pub fn top_by_memory(&self, count: usize) -> Vec<ProcessInfo> {
        let mut processes = self.list_all().unwrap_or_default();
        processes.sort_by(|a, b| b.memory.cmp(&a.memory));
        processes.into_iter().take(count).collect()
    }
    
    /// Search processes by name
    pub fn search(&self, query: &str) -> Vec<ProcessInfo> {
        let query_lower = query.to_lowercase();
        self.list_all()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&query_lower))
            .collect()
    }
    
    /// Check if process is running
    pub fn is_running(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.system.processes()
            .values()
            .any(|p| p.name().to_lowercase() == name_lower)
    }
    
    /// Get CPU usage
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> (u64, u64, f32) {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        
        (used, total, percent)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
