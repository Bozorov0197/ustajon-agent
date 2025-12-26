// Windows Services module

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
    pub description: String,
}

pub struct ServiceManager;

impl ServiceManager {
    pub fn new() -> Self {
        Self
    }
    
    /// List all services
    pub fn list_all(&self) -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
        let mut services = Vec::new();
        
        #[cfg(windows)]
        {
            // Use sc query
            let output = Command::new("sc")
                .args(["query", "state=", "all"])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut current_service = ServiceInfo {
                name: String::new(),
                display_name: String::new(),
                status: String::new(),
                start_type: String::new(),
                description: String::new(),
            };
            
            for line in stdout.lines() {
                let line = line.trim();
                
                if line.starts_with("SERVICE_NAME:") {
                    if !current_service.name.is_empty() {
                        services.push(current_service.clone());
                    }
                    current_service = ServiceInfo {
                        name: line.replace("SERVICE_NAME:", "").trim().to_string(),
                        display_name: String::new(),
                        status: String::new(),
                        start_type: String::new(),
                        description: String::new(),
                    };
                } else if line.starts_with("DISPLAY_NAME:") {
                    current_service.display_name = line.replace("DISPLAY_NAME:", "").trim().to_string();
                } else if line.contains("STATE") && line.contains(':') {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        current_service.status = parts[3].to_string();
                    }
                }
            }
            
            if !current_service.name.is_empty() {
                services.push(current_service);
            }
        }
        
        Ok(services)
    }
    
    /// Get service info
    pub fn get(&self, name: &str) -> Result<ServiceInfo, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("sc")
                .args(["qc", name])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            let mut service = ServiceInfo {
                name: name.to_string(),
                display_name: String::new(),
                status: String::new(),
                start_type: String::new(),
                description: String::new(),
            };
            
            for line in stdout.lines() {
                let line = line.trim();
                
                if line.starts_with("DISPLAY_NAME") {
                    service.display_name = line.split(':').nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                } else if line.starts_with("START_TYPE") {
                    service.start_type = line.split(':').nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                }
            }
            
            // Get status
            let status_output = Command::new("sc")
                .args(["query", name])
                .output()?;
            
            let status_stdout = String::from_utf8_lossy(&status_output.stdout);
            
            for line in status_stdout.lines() {
                if line.contains("STATE") && line.contains(':') {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        service.status = parts[3].to_string();
                    }
                }
            }
            
            return Ok(service);
        }
        
        #[cfg(not(windows))]
        {
            Err("Services are only available on Windows".into())
        }
    }
    
    /// Start service
    pub fn start(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("sc")
                .args(["start", name])
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to start service: {}", stderr).into());
            }
        }
        
        Ok(())
    }
    
    /// Stop service
    pub fn stop(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("sc")
                .args(["stop", name])
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to stop service: {}", stderr).into());
            }
        }
        
        Ok(())
    }
    
    /// Restart service
    pub fn restart(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stop(name)?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        self.start(name)?;
        Ok(())
    }
    
    /// Set service start type
    pub fn set_start_type(&self, name: &str, start_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let start_value = match start_type.to_lowercase().as_str() {
                "auto" | "automatic" => "auto",
                "manual" => "demand",
                "disabled" => "disabled",
                "delayed" | "delayed-auto" => "delayed-auto",
                _ => return Err("Invalid start type".into()),
            };
            
            let output = Command::new("sc")
                .args(["config", name, "start=", start_value])
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to set start type: {}", stderr).into());
            }
        }
        
        Ok(())
    }
    
    /// Create service
    pub fn create(&self, name: &str, display_name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("sc")
                .args(["create", name, &format!("binPath={}", path), &format!("DisplayName={}", display_name)])
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to create service: {}", stderr).into());
            }
        }
        
        Ok(())
    }
    
    /// Delete service
    pub fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            // Stop first
            let _ = self.stop(name);
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            let output = Command::new("sc")
                .args(["delete", name])
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to delete service: {}", stderr).into());
            }
        }
        
        Ok(())
    }
    
    /// Check if service exists
    pub fn exists(&self, name: &str) -> bool {
        #[cfg(windows)]
        {
            let output = Command::new("sc")
                .args(["query", name])
                .output();
            
            if let Ok(output) = output {
                return output.status.success();
            }
        }
        
        false
    }
    
    /// Get running services
    pub fn list_running(&self) -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
        let all = self.list_all()?;
        Ok(all.into_iter().filter(|s| s.status == "RUNNING").collect())
    }
    
    /// Get stopped services
    pub fn list_stopped(&self) -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
        let all = self.list_all()?;
        Ok(all.into_iter().filter(|s| s.status == "STOPPED").collect())
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
