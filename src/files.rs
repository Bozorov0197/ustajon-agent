// File Manager module

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub created: Option<String>,
    pub permissions: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub total: u64,
    pub free: u64,
    pub drive_type: String,
}

pub struct FileManager;

impl FileManager {
    pub fn new() -> Self {
        Self
    }
    
    /// List directory contents
    pub fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let path = if path.is_empty() || path == "/" {
            // Return drives on Windows
            #[cfg(windows)]
            {
                return self.list_drives();
            }
            #[cfg(not(windows))]
            {
                PathBuf::from("/")
            }
        } else {
            PathBuf::from(path)
        };
        
        let mut files = Vec::new();
        
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_path = entry.path();
            
            let modified = metadata.modified().ok()
                .map(|t| DateTime::<Utc>::from(t).to_rfc3339());
            
            let created = metadata.created().ok()
                .map(|t| DateTime::<Utc>::from(t).to_rfc3339());
            
            let hidden = is_hidden(&file_path);
            
            files.push(FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: file_path.to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified,
                created,
                permissions: format_permissions(&metadata),
                hidden,
            });
        }
        
        // Sort: directories first, then by name
        files.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        Ok(files)
    }
    
    /// List Windows drives
    #[cfg(windows)]
    fn list_drives(&self) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let mut drives = Vec::new();
        
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = Path::new(&drive);
            
            if path.exists() {
                drives.push(FileInfo {
                    name: drive.clone(),
                    path: drive,
                    is_dir: true,
                    size: 0,
                    modified: None,
                    created: None,
                    permissions: "drwxr-xr-x".to_string(),
                    hidden: false,
                });
            }
        }
        
        Ok(drives)
    }
    
    /// Read file contents
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(fs::read(path)?)
    }
    
    /// Read file as text
    pub fn read_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(fs::read_to_string(path)?)
    }
    
    /// Write file
    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)?;
        Ok(())
    }
    
    /// Delete file or directory
    pub fn delete(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path);
        
        if path.is_dir() {
            // Try to move to trash first
            if trash::delete(path).is_err() {
                fs::remove_dir_all(path)?;
            }
        } else {
            if trash::delete(path).is_err() {
                fs::remove_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Create directory
    pub fn create_dir(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(path)?;
        Ok(())
    }
    
    /// Rename/move file
    pub fn rename(&self, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::rename(from, to)?;
        Ok(())
    }
    
    /// Copy file
    pub fn copy(&self, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
        let from_path = Path::new(from);
        let to_path = Path::new(to);
        
        if from_path.is_dir() {
            self.copy_dir(from_path, to_path)?;
        } else {
            if let Some(parent) = to_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(from, to)?;
        }
        
        Ok(())
    }
    
    fn copy_dir(&self, from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(to)?;
        
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let dest = to.join(entry.file_name());
            
            if file_type.is_dir() {
                self.copy_dir(&entry.path(), &dest)?;
            } else {
                fs::copy(entry.path(), dest)?;
            }
        }
        
        Ok(())
    }
    
    /// Search files
    pub fn search(&self, base_path: &str, pattern: &str, max_results: usize) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let pattern_lower = pattern.to_lowercase();
        
        for entry in WalkDir::new(base_path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if results.len() >= max_results {
                break;
            }
            
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            
            if file_name.contains(&pattern_lower) {
                if let Ok(metadata) = entry.metadata() {
                    let file_path = entry.path();
                    
                    results.push(FileInfo {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: file_path.to_string_lossy().to_string(),
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                        modified: metadata.modified().ok()
                            .map(|t| DateTime::<Utc>::from(t).to_rfc3339()),
                        created: metadata.created().ok()
                            .map(|t| DateTime::<Utc>::from(t).to_rfc3339()),
                        permissions: format_permissions(&metadata),
                        hidden: is_hidden(file_path),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get directory size
    pub fn get_dir_size(&self, path: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let mut total = 0u64;
        
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        
        Ok(total)
    }
    
    /// Zip directory
    pub fn zip_directory(&self, dir_path: &str, zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        use zip::write::FileOptions;
        
        let file = fs::File::create(zip_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        
        let base = Path::new(dir_path);
        
        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let name = path.strip_prefix(base)
                .unwrap_or(path)
                .to_string_lossy();
            
            if path.is_file() {
                zip.start_file(name.to_string(), options)?;
                let data = fs::read(path)?;
                zip.write_all(&data)?;
            } else if !name.is_empty() {
                zip.add_directory(name.to_string(), options)?;
            }
        }
        
        zip.finish()?;
        Ok(())
    }
    
    /// Extract zip
    pub fn unzip(&self, zip_path: &str, dest_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = Path::new(dest_path).join(file.name());
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        
        Ok(())
    }
}

fn is_hidden(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = path.metadata() {
            return metadata.file_attributes() & 2 != 0; // FILE_ATTRIBUTE_HIDDEN
        }
    }
    
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

fn format_permissions(metadata: &fs::Metadata) -> String {
    let is_dir = metadata.is_dir();
    
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let attrs = metadata.file_attributes();
        let readonly = attrs & 1 != 0;
        let hidden = attrs & 2 != 0;
        let system = attrs & 4 != 0;
        
        format!("{}{}{}{}",
            if is_dir { "d" } else { "-" },
            if readonly { "r" } else { "w" },
            if hidden { "h" } else { "-" },
            if system { "s" } else { "-" }
        )
    }
    
    #[cfg(not(windows))]
    {
        if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" }.to_string()
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}
