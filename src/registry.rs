// Windows Registry module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryValue {
    pub name: String,
    pub value_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKey {
    pub path: String,
    pub subkeys: Vec<String>,
    pub values: Vec<RegistryValue>,
}

pub struct RegistryManager;

impl RegistryManager {
    pub fn new() -> Self {
        Self
    }
    
    /// Read registry value
    #[cfg(windows)]
    pub fn read(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let (hive, subpath) = parse_registry_path(path)?;
        let (key_path, value_name) = split_key_value(subpath);
        
        let hkey = get_hkey(hive)?;
        let key = hkey.open_subkey(key_path)?;
        
        let value: String = key.get_value(value_name)?;
        Ok(value)
    }
    
    #[cfg(not(windows))]
    pub fn read(&self, _path: &str) -> Result<String, Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Write registry value
    #[cfg(windows)]
    pub fn write(&self, path: &str, value_name: &str, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let (hive, subpath) = parse_registry_path(path)?;
        
        let hkey = get_hkey(hive)?;
        let (key, _) = hkey.create_subkey(subpath)?;
        
        key.set_value(value_name, &data)?;
        Ok(())
    }
    
    #[cfg(not(windows))]
    pub fn write(&self, _path: &str, _value_name: &str, _data: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Delete registry value
    #[cfg(windows)]
    pub fn delete_value(&self, path: &str, value_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let (hive, subpath) = parse_registry_path(path)?;
        
        let hkey = get_hkey(hive)?;
        let key = hkey.open_subkey_with_flags(subpath, KEY_WRITE)?;
        
        key.delete_value(value_name)?;
        Ok(())
    }
    
    #[cfg(not(windows))]
    pub fn delete_value(&self, _path: &str, _value_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Delete registry key
    #[cfg(windows)]
    pub fn delete_key(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let (hive, subpath) = parse_registry_path(path)?;
        
        let hkey = get_hkey(hive)?;
        hkey.delete_subkey_all(subpath)?;
        Ok(())
    }
    
    #[cfg(not(windows))]
    pub fn delete_key(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// List registry key contents
    #[cfg(windows)]
    pub fn list_key(&self, path: &str) -> Result<RegistryKey, Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let (hive, subpath) = parse_registry_path(path)?;
        
        let hkey = get_hkey(hive)?;
        let key = hkey.open_subkey(subpath)?;
        
        // Get subkeys
        let subkeys: Vec<String> = key.enum_keys()
            .filter_map(|k| k.ok())
            .collect();
        
        // Get values
        let values: Vec<RegistryValue> = key.enum_values()
            .filter_map(|v| v.ok())
            .map(|(name, value)| {
                RegistryValue {
                    name,
                    value_type: format!("{:?}", value.vtype),
                    data: format!("{:?}", value),
                }
            })
            .collect();
        
        Ok(RegistryKey {
            path: path.to_string(),
            subkeys,
            values,
        })
    }
    
    #[cfg(not(windows))]
    pub fn list_key(&self, _path: &str) -> Result<RegistryKey, Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Get startup programs
    #[cfg(windows)]
    pub fn get_startup_programs(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let mut programs = Vec::new();
        
        // Current user startup
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
            for value in key.enum_values().filter_map(|v| v.ok()) {
                let data: String = key.get_value(&value.0).unwrap_or_default();
                programs.push((value.0, data));
            }
        }
        
        // Local machine startup
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
            for value in key.enum_values().filter_map(|v| v.ok()) {
                let data: String = key.get_value(&value.0).unwrap_or_default();
                programs.push((value.0, data));
            }
        }
        
        Ok(programs)
    }
    
    #[cfg(not(windows))]
    pub fn get_startup_programs(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Add to startup
    #[cfg(windows)]
    pub fn add_to_startup(&self, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")?;
        key.set_value(name, &path)?;
        
        Ok(())
    }
    
    #[cfg(not(windows))]
    pub fn add_to_startup(&self, _name: &str, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
    
    /// Remove from startup
    #[cfg(windows)]
    pub fn remove_from_startup(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_WRITE
        )?;
        key.delete_value(name)?;
        
        Ok(())
    }
    
    #[cfg(not(windows))]
    pub fn remove_from_startup(&self, _name: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Registry is only available on Windows".into())
    }
}

#[cfg(windows)]
fn parse_registry_path(path: &str) -> Result<(&str, &str), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = path.splitn(2, '\\').collect();
    if parts.len() < 2 {
        return Err("Invalid registry path".into());
    }
    Ok((parts[0], parts[1]))
}

#[cfg(windows)]
fn split_key_value(path: &str) -> (&str, &str) {
    if let Some(idx) = path.rfind('\\') {
        (&path[..idx], &path[idx+1..])
    } else {
        (path, "")
    }
}

#[cfg(windows)]
fn get_hkey(hive: &str) -> Result<winreg::RegKey, Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    match hive.to_uppercase().as_str() {
        "HKEY_CURRENT_USER" | "HKCU" => Ok(RegKey::predef(HKEY_CURRENT_USER)),
        "HKEY_LOCAL_MACHINE" | "HKLM" => Ok(RegKey::predef(HKEY_LOCAL_MACHINE)),
        "HKEY_CLASSES_ROOT" | "HKCR" => Ok(RegKey::predef(HKEY_CLASSES_ROOT)),
        "HKEY_USERS" | "HKU" => Ok(RegKey::predef(HKEY_USERS)),
        "HKEY_CURRENT_CONFIG" | "HKCC" => Ok(RegKey::predef(HKEY_CURRENT_CONFIG)),
        _ => Err(format!("Unknown registry hive: {}", hive).into()),
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
}
