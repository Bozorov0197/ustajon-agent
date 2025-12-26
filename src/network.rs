// Network module

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
    pub public_ip: String,
    pub local_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub mac: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
    pub process_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: i32,
    pub security: String,
    pub connected: bool,
}

pub struct NetworkManager;

impl NetworkManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_info(&self) -> Result<NetworkInfo, Box<dyn std::error::Error>> {
        let interfaces = self.get_interfaces()?;
        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_default();
        
        let public_ip = self.get_public_ip();
        
        Ok(NetworkInfo {
            interfaces,
            public_ip,
            local_ip,
        })
    }
    
    fn get_interfaces(&self) -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        let mut interfaces = Vec::new();
        
        #[cfg(windows)]
        {
            // Use ipconfig
            if let Ok(output) = Command::new("ipconfig")
                .args(["/all"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut current_name = String::new();
                let mut current_ip = String::new();
                let mut current_mac = String::new();
                
                for line in stdout.lines() {
                    let line = line.trim();
                    
                    if line.ends_with(':') && !line.contains("  ") {
                        if !current_name.is_empty() {
                            interfaces.push(NetworkInterface {
                                name: current_name.clone(),
                                ip: current_ip.clone(),
                                mac: current_mac.clone(),
                                status: if current_ip.is_empty() { "Disconnected" } else { "Connected" }.to_string(),
                            });
                        }
                        current_name = line.trim_end_matches(':').to_string();
                        current_ip.clear();
                        current_mac.clear();
                    } else if line.contains("IPv4") && line.contains(':') {
                        current_ip = line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
                    } else if line.contains("Physical Address") && line.contains(':') {
                        current_mac = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                    }
                }
                
                if !current_name.is_empty() {
                    interfaces.push(NetworkInterface {
                        name: current_name,
                        ip: current_ip,
                        mac: current_mac,
                        status: "Unknown".to_string(),
                    });
                }
            }
        }
        
        Ok(interfaces)
    }
    
    fn get_public_ip(&self) -> String {
        if let Ok(response) = reqwest::blocking::get("https://api.ipify.org") {
            if let Ok(ip) = response.text() {
                return ip;
            }
        }
        String::new()
    }
    
    pub fn get_connections(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();
        
        #[cfg(windows)]
        {
            // Use netstat
            if let Ok(output) = Command::new("netstat")
                .args(["-ano"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for line in stdout.lines().skip(4) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    
                    if parts.len() >= 5 {
                        let protocol = parts[0].to_string();
                        let local = parts[1];
                        let remote = parts[2];
                        let state = if parts.len() > 3 { parts[3] } else { "" };
                        let pid: u32 = parts.last()
                            .and_then(|p| p.parse().ok())
                            .unwrap_or(0);
                        
                        let (local_addr, local_port) = parse_address(local);
                        let (remote_addr, remote_port) = parse_address(remote);
                        
                        connections.push(NetworkConnection {
                            protocol,
                            local_address: local_addr,
                            local_port,
                            remote_address: remote_addr,
                            remote_port,
                            state: state.to_string(),
                            pid,
                            process_name: get_process_name(pid),
                        });
                    }
                }
            }
        }
        
        Ok(connections)
    }
    
    pub fn get_wifi_networks(&self) -> Result<Vec<WifiNetwork>, Box<dyn std::error::Error>> {
        let mut networks = Vec::new();
        
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("netsh")
                .args(["wlan", "show", "networks", "mode=bssid"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut current_ssid = String::new();
                let mut current_signal = 0i32;
                let mut current_security = String::new();
                
                for line in stdout.lines() {
                    let line = line.trim();
                    
                    if line.starts_with("SSID") && line.contains(':') {
                        if !current_ssid.is_empty() {
                            networks.push(WifiNetwork {
                                ssid: current_ssid.clone(),
                                signal: current_signal,
                                security: current_security.clone(),
                                connected: false,
                            });
                        }
                        current_ssid = line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
                    } else if line.contains("Signal") && line.contains(':') {
                        let signal_str = line.split(':').nth(1)
                            .map(|s| s.trim().trim_end_matches('%'))
                            .unwrap_or("0");
                        current_signal = signal_str.parse().unwrap_or(0);
                    } else if line.contains("Authentication") && line.contains(':') {
                        current_security = line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
                    }
                }
                
                if !current_ssid.is_empty() {
                    networks.push(WifiNetwork {
                        ssid: current_ssid,
                        signal: current_signal,
                        security: current_security,
                        connected: false,
                    });
                }
            }
        }
        
        Ok(networks)
    }
    
    pub fn get_current_wifi(&self) -> Option<String> {
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("netsh")
                .args(["wlan", "show", "interfaces"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for line in stdout.lines() {
                    if line.contains("SSID") && !line.contains("BSSID") {
                        if let Some(ssid) = line.split(':').nth(1) {
                            return Some(ssid.trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }
    
    pub fn check_port(&self, host: &str, port: u16) -> bool {
        TcpStream::connect((host, port)).is_ok()
    }
    
    pub fn ping(&self, host: &str) -> Result<bool, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            let output = Command::new("ping")
                .args(["-n", "1", "-w", "3000", host])
                .output()?;
            
            return Ok(output.status.success());
        }
        
        #[cfg(not(windows))]
        {
            let output = Command::new("ping")
                .args(["-c", "1", "-W", "3", host])
                .output()?;
            
            return Ok(output.status.success());
        }
    }
}

fn parse_address(addr: &str) -> (String, u16) {
    if let Some(idx) = addr.rfind(':') {
        let ip = addr[..idx].to_string();
        let port: u16 = addr[idx+1..].parse().unwrap_or(0);
        (ip, port)
    } else {
        (addr.to_string(), 0)
    }
}

fn get_process_name(pid: u32) -> String {
    use sysinfo::{System, Pid};
    
    let sys = System::new_all();
    
    sys.process(Pid::from_u32(pid))
        .map(|p| p.name().to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}
