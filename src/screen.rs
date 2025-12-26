// Screen capture module

use std::io::Cursor;
use image::{DynamicImage, ImageOutputFormat};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct ScreenCapture;

impl ScreenCapture {
    pub fn new() -> Self {
        Self
    }
    
    /// Capture primary screen
    pub fn capture(&self) -> Result<String, Box<dyn std::error::Error>> {
        let screens = screenshots::Screen::all()?;
        
        if let Some(screen) = screens.first() {
            let image = screen.capture()?;
            let buffer = image.buffer();
            
            // Convert to JPEG for smaller size
            let img = DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(
                    image.width(),
                    image.height(),
                    buffer.to_vec(),
                ).ok_or("Failed to create image")?
            );
            
            let mut jpeg_bytes = Vec::new();
            img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageOutputFormat::Jpeg(75))?;
            
            Ok(BASE64.encode(&jpeg_bytes))
        } else {
            Err("No screens found".into())
        }
    }
    
    /// Capture specific screen
    pub fn capture_screen(&self, screen_index: usize) -> Result<String, Box<dyn std::error::Error>> {
        let screens = screenshots::Screen::all()?;
        
        if let Some(screen) = screens.get(screen_index) {
            let image = screen.capture()?;
            let buffer = image.buffer();
            
            let img = DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(
                    image.width(),
                    image.height(),
                    buffer.to_vec(),
                ).ok_or("Failed to create image")?
            );
            
            let mut jpeg_bytes = Vec::new();
            img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageOutputFormat::Jpeg(75))?;
            
            Ok(BASE64.encode(&jpeg_bytes))
        } else {
            Err("Screen not found".into())
        }
    }
    
    /// Capture region
    pub fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<String, Box<dyn std::error::Error>> {
        let screens = screenshots::Screen::all()?;
        
        if let Some(screen) = screens.first() {
            let image = screen.capture_area(x, y, width, height)?;
            let buffer = image.buffer();
            
            let img = DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(
                    image.width(),
                    image.height(),
                    buffer.to_vec(),
                ).ok_or("Failed to create image")?
            );
            
            let mut jpeg_bytes = Vec::new();
            img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageOutputFormat::Jpeg(75))?;
            
            Ok(BASE64.encode(&jpeg_bytes))
        } else {
            Err("No screens found".into())
        }
    }
    
    /// Get screen count
    pub fn screen_count(&self) -> usize {
        screenshots::Screen::all()
            .map(|s| s.len())
            .unwrap_or(0)
    }
    
    /// Get screen info
    pub fn get_screen_info(&self) -> Vec<ScreenInfo> {
        screenshots::Screen::all()
            .map(|screens| {
                screens.iter().enumerate().map(|(i, s)| {
                    ScreenInfo {
                        index: i,
                        name: format!("Screen {}", i + 1),
                        width: s.display_info.width,
                        height: s.display_info.height,
                        x: s.display_info.x,
                        y: s.display_info.y,
                        is_primary: s.display_info.is_primary,
                    }
                }).collect()
            })
            .unwrap_or_default()
    }
    
    /// Capture thumbnail (smaller for quick preview)
    pub fn capture_thumbnail(&self, max_width: u32) -> Result<String, Box<dyn std::error::Error>> {
        let screens = screenshots::Screen::all()?;
        
        if let Some(screen) = screens.first() {
            let image = screen.capture()?;
            let buffer = image.buffer();
            
            let img = DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(
                    image.width(),
                    image.height(),
                    buffer.to_vec(),
                ).ok_or("Failed to create image")?
            );
            
            // Resize if needed
            let thumbnail = if img.width() > max_width {
                let ratio = max_width as f32 / img.width() as f32;
                let new_height = (img.height() as f32 * ratio) as u32;
                img.resize(max_width, new_height, image::imageops::FilterType::Triangle)
            } else {
                img
            };
            
            let mut jpeg_bytes = Vec::new();
            thumbnail.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageOutputFormat::Jpeg(60))?;
            
            Ok(BASE64.encode(&jpeg_bytes))
        } else {
            Err("No screens found".into())
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}
