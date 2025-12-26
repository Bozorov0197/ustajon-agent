// System information module

use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfo {
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub local_ip: String,
    pub public_ip: String,
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub disk_usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedInfo {
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_usage: f32,
    pub total_ram: u64,
    pub used_ram: u64,
    pub ram_usage: f32,
    pub disks: Vec<DiskInfo>,
    pub local_ip: String,
    pub public_ip: String,
    pub mac_address: String,
    pub uptime: u64,
    pub boot_time: u64,
    pub antivirus: String,
    pub installed_software: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f32,
}

pub struct SystemInfo {
    system: System,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }
    
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    pub fn get_info(&self) -> BasicInfo {
        BasicInfo {
            hostname: self.get_hostname(),
            username: self.get_username(),
            os: self.get_os(),
            local_ip: self.get_local_ip(),
            public_ip: self.get_public_ip(),
            cpu_usage: self.get_cpu_usage(),
            ram_usage: self.get_ram_usage(),
            disk_usage: self.get_disk_usage(),
        }
    }
    
    pub fn get_detailed_info(&self) -> DetailedInfo {
        let disks = self.get_disks();
        
        DetailedInfo {
            hostname: self.get_hostname(),
            username: self.get_username(),
            os: self.get_os(),
            os_version: self.get_os_version(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_name: self.get_cpu_name(),
            cpu_cores: self.system.cpus().len(),
            cpu_usage: self.get_cpu_usage(),
            total_ram: self.system.total_memory(),
            used_ram: self.system.used_memory(),
            ram_usage: self.get_ram_usage(),
            disks,
            local_ip: self.get_local_ip(),
            public_ip: self.get_public_ip(),
            mac_address: self.get_mac_address(),
            uptime: System::uptime(),
            boot_time: System::boot_time(),
            antivirus: self.get_antivirus(),
            installed_software: self.get_installed_software(),
        }
    }
    
    fn get_hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "Unknown".to_string())
    }
    
    fn get_username(&self) -> String {
        whoami::username()
    }
    
    fn get_os(&self) -> String {
        format!("{} {}", 
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_default()
        )
    }
    
    fn get_os_version(&self) -> String {
        System::os_version().unwrap_or_else(|| "Unknown".to_string())
    }
    
    fn get_cpu_name(&self) -> String {
        self.system.cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
    
    fn get_cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }
    
    fn get_ram_usage(&self) -> f32 {
        let total = self.system.total_memory() as f32;
        let used = self.system.used_memory() as f32;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
    
    fn get_disk_usage(&self) -> f32 {
        let mut total: u64 = 0;
        let mut used: u64 = 0;
        
        for disk in self.system.disks() {
            total += disk.total_space();
            used += disk.total_space() - disk.available_space();
        }
        
        if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }
    
    fn get_disks(&self) -> Vec<DiskInfo> {
        self.system.disks().iter().map(|disk| {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total - free;
            
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total,
                used,
                free,
                usage_percent: if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                },
            }
        }).collect()
    }
    
    fn get_local_ip(&self) -> String {
        local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string())
    }
    
    fn get_public_ip(&self) -> String {
        // Try to get public IP
        if let Ok(response) = reqwest::blocking::get("https://api.ipify.org") {
            if let Ok(ip) = response.text() {
                return ip;
            }
        }
        String::new()
    }
    
    fn get_mac_address(&self) -> String {
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("getmac")
                .args(["/FO", "CSV", "/NH"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    if let Some(mac) = line.split(',').next() {
                        return mac.trim_matches('"').to_string();
                    }
                }
            }
        }
        "Unknown".to_string()
    }
    
    fn get_antivirus(&self) -> String {
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("wmic")
                .args(["/namespace:\\\\root\\SecurityCenter2", "path", "AntiVirusProduct", "get", "displayName"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let antiviruses: Vec<&str> = stdout.lines()
                    .skip(1)
                    .filter(|line| !line.trim().is_empty())
                    .collect();
                
                if !antiviruses.is_empty() {
                    return antiviruses.join(", ");
                }
            }
        }
        "Not detected".to_string()
    }
    
    fn get_installed_software(&self) -> Vec<String> {
        let mut software = Vec::new();
        
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("wmic")
                .args(["product", "get", "Name"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let name = line.trim();
                    if !name.is_empty() {
                        software.push(name.to_string());
                    }
                }
            }
        }
        
        software
    }
    
    // System control functions
    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            Command::new("shutdown")
                .args(["/s", "/t", "5"])
                .spawn()?;
        }
        Ok(())
    }
    
    pub fn restart(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            Command::new("shutdown")
                .args(["/r", "/t", "5"])
                .spawn()?;
        }
        Ok(())
    }
    
    pub fn logoff(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            Command::new("shutdown")
                .args(["/l"])
                .spawn()?;
        }
        Ok(())
    }
    
    pub fn sleep(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0", "1", "0"])
                .spawn()?;
        }
        Ok(())
    }
    
    pub fn lock(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn()?;
        }
        Ok(())
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}
