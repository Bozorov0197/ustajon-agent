// API Client module

use reqwest;
use serde_json::Value;
use tracing::{info, error};

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    client_id: String,
}

impl ApiClient {
    pub fn new(base_url: &str, client_id: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            client,
            base_url: base_url.to_string(),
            client_id: client_id.to_string(),
        }
    }
    
    /// Send POST request
    pub async fn post(&self, endpoint: &str, data: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = self.client.post(&url)
            .header("Content-Type", "application/json")
            .header("X-Client-ID", &self.client_id)
            .header("User-Agent", format!("UstajonAgent/{}", env!("CARGO_PKG_VERSION")))
            .json(data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let body = response.json().await?;
            Ok(body)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("API error: {} - {}", status, body);
            Err(format!("API error: {} - {}", status, body).into())
        }
    }
    
    /// Send GET request
    pub async fn get(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = self.client.get(&url)
            .header("X-Client-ID", &self.client_id)
            .header("User-Agent", format!("UstajonAgent/{}", env!("CARGO_PKG_VERSION")))
            .send()
            .await?;
        
        if response.status().is_success() {
            let body = response.json().await?;
            Ok(body)
        } else {
            let status = response.status();
            Err(format!("API error: {}", status).into())
        }
    }
    
    /// Upload file
    pub async fn upload_file(&self, endpoint: &str, file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;
        
        let mut file = File::open(file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        let url = format!("{}{}", self.base_url, endpoint);
        
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        
        let part = reqwest::multipart::Part::bytes(buffer)
            .file_name(file_name);
        
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("client_id", self.client_id.clone());
        
        let response = self.client.post(&url)
            .multipart(form)
            .send()
            .await?;
        
        if response.status().is_success() {
            let body = response.json().await?;
            Ok(body)
        } else {
            let status = response.status();
            Err(format!("Upload error: {}", status).into())
        }
    }
    
    /// Download file
    pub async fn download_file(&self, endpoint: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = self.client.get(&url)
            .header("X-Client-ID", &self.client_id)
            .send()
            .await?;
        
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut file = File::create(save_path).await?;
            file.write_all(&bytes).await?;
            Ok(())
        } else {
            let status = response.status();
            Err(format!("Download error: {}", status).into())
        }
    }
    
    /// Check connection
    pub async fn ping(&self) -> bool {
        let url = format!("{}/api/ping", self.base_url);
        
        if let Ok(response) = self.client.get(&url).send().await {
            return response.status().is_success();
        }
        
        false
    }
    
    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Get client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}
