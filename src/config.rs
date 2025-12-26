// Configuration module

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sha2::{Sha256, Digest};

const APP_NAME: &str = "UstajonAgent";
const SERVER_URL: &str = "http://31.220.75.75";
const RUSTDESK_SERVER: &str = "31.220.75.75";
const RUSTDESK_KEY: &str = "YHo+N4vp+ZWP7wedLh69zCGk3aFf4935hwDKX9OdFXE=";
const RUSTDESK_PASSWORD: &str = "ustajon2025";
const UPDATE_URL: &str = "http://31.220.75.75/api/update";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub server_url: String,
    pub rustdesk_server: String,
    pub rustdesk_key: String,
    pub rustdesk_password: String,
    pub update_url: String,
    pub client_name: Option<String>,
    pub client_phone: Option<String>,
    pub client_problem: Option<String>,
    pub registered: bool,
    pub registered_at: Option<String>,
    pub stealth_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: generate_client_id(),
            server_url: SERVER_URL.to_string(),
            rustdesk_server: RUSTDESK_SERVER.to_string(),
            rustdesk_key: RUSTDESK_KEY.to_string(),
            rustdesk_password: RUSTDESK_PASSWORD.to_string(),
            update_url: UPDATE_URL.to_string(),
            client_name: None,
            client_phone: None,
            client_problem: None,
            registered: false,
            registered_at: None,
            stealth_mode: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = get_config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = get_config_path();
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    pub fn get_client_id(&self) -> String {
        self.client_id.clone()
    }
    
    pub fn is_registered(&self) -> bool {
        self.registered
    }
    
    pub fn register(&mut self, name: &str, phone: &str, problem: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client_name = Some(name.to_string());
        self.client_phone = Some(phone.to_string());
        self.client_problem = Some(problem.to_string());
        self.registered = true;
        self.registered_at = Some(chrono::Utc::now().to_rfc3339());
        self.save()?;
        Ok(())
    }
}

fn get_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
        .join("config.json")
}

fn generate_client_id() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    let username = whoami::username();
    let machine_id = machine_uid::get().unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    
    let combined = format!("{}-{}-{}", hostname, username, machine_id);
    
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    
    hex::encode(&result[..8]).to_uppercase()
}

// Add machine_uid dependency simulation
mod machine_uid {
    pub fn get() -> Result<String, ()> {
        #[cfg(windows)]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("wmic")
                .args(["csproduct", "get", "UUID"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let uuid = line.trim();
                    if !uuid.is_empty() {
                        return Ok(uuid.to_string());
                    }
                }
            }
        }
        
        Ok(uuid::Uuid::new_v4().to_string())
    }
}
