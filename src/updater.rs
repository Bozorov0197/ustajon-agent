// Auto-updater module

use std::path::PathBuf;
use std::process::Command;
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub changelog: String,
    pub mandatory: bool,
    pub checksum: String,
}

pub struct AutoUpdater {
    update_url: String,
    current_version: String,
}

impl AutoUpdater {
    pub fn new(update_url: &str) -> Self {
        Self {
            update_url: update_url.to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Check for updates
    pub async fn check(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        
        let response = client.get(&self.update_url)
            .query(&[("current_version", &self.current_version)])
            .send()
            .await?;
        
        if response.status().is_success() {
            let update_info: UpdateInfo = response.json().await?;
            
            if self.is_newer(&update_info.version) {
                info!("New version available: {}", update_info.version);
                return Ok(Some(update_info));
            }
        }
        
        Ok(None)
    }
    
    /// Check if version is newer
    fn is_newer(&self, new_version: &str) -> bool {
        let current_parts: Vec<u32> = self.current_version
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        
        let new_parts: Vec<u32> = new_version
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        
        for (c, n) in current_parts.iter().zip(new_parts.iter()) {
            if n > c {
                return true;
            } else if c > n {
                return false;
            }
        }
        
        new_parts.len() > current_parts.len()
    }
    
    /// Check and update if available
    pub async fn check_and_update(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(update) = self.check().await? {
            self.download_and_install(&update).await?;
            return Ok(true);
        }
        Ok(false)
    }
    
    /// Force update to latest version
    pub async fn force_update(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(update) = self.check().await? {
            self.download_and_install(&update).await?;
        }
        Ok(())
    }
    
    /// Download and install update
    async fn download_and_install(&self, update: &UpdateInfo) -> Result<(), Box<dyn std::error::Error>> {
        info!("Downloading update v{}...", update.version);
        
        let client = reqwest::Client::new();
        let response = client.get(&update.download_url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err("Failed to download update".into());
        }
        
        let bytes = response.bytes().await?;
        
        // Verify checksum
        if !update.checksum.is_empty() {
            let calculated = format!("{:x}", sha2::Sha256::digest(&bytes));
            if calculated != update.checksum {
                return Err("Checksum mismatch".into());
            }
        }
        
        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let update_path = temp_dir.join("ustajon_update.exe");
        
        std::fs::write(&update_path, &bytes)?;
        
        info!("Installing update...");
        
        // Create update script
        let current_exe = std::env::current_exe()?;
        let script_path = temp_dir.join("update.bat");
        
        let script = format!(
            r#"@echo off
timeout /t 2 /nobreak > nul
del "{current_exe}"
move "{update_path}" "{current_exe}"
start "" "{current_exe}"
del "%~f0"
"#,
            current_exe = current_exe.display(),
            update_path = update_path.display()
        );
        
        std::fs::write(&script_path, script)?;
        
        // Run update script
        Command::new("cmd")
            .args(["/C", &script_path.to_string_lossy()])
            .spawn()?;
        
        // Exit current process
        std::process::exit(0);
    }
    
    /// Get current version
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
}

use sha2::Digest;
