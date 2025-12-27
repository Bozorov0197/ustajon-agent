#!/usr/bin/env python3
"""
Ustajon Support Client v2.0
Professional Remote Support Agent
"""

import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import threading
import urllib.request
import urllib.parse
import json
import os
import sys
import socket
import platform
import subprocess
import time
import uuid
import tempfile
import shutil
import ssl

# Configuration
SERVER_URL = "http://31.220.75.75"
RUSTDESK_SERVER = "31.220.75.75"
RUSTDESK_KEY = "YHo+N4vp+ZWP7wedLh69zCGk3aFf4935hwDKX9OdFXE="
RUSTDESK_PASSWORD = "ustajon2025"

class UstajonClient:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Ustajon Support - Masofaviy Yordam")
        self.root.geometry("500x600")
        self.root.resizable(False, False)
        
        # Client info
        self.client_id = str(uuid.uuid4())[:8]
        self.client_name = ""
        self.client_phone = ""
        self.rustdesk_id = ""
        self.registered = False
        
        # Setup UI
        self.setup_ui()
        
        # Center window
        self.center_window()
        
    def center_window(self):
        self.root.update_idletasks()
        w = self.root.winfo_width()
        h = self.root.winfo_height()
        x = (self.root.winfo_screenwidth() // 2) - (w // 2)
        y = (self.root.winfo_screenheight() // 2) - (h // 2)
        self.root.geometry(f"+{x}+{y}")
        
    def setup_ui(self):
        # Main frame
        self.main_frame = ttk.Frame(self.root, padding="20")
        self.main_frame.pack(fill=tk.BOTH, expand=True)
        
        # Title
        title = ttk.Label(
            self.main_frame,
            text="🛠️ Ustajon Support",
            font=("Segoe UI", 24, "bold")
        )
        title.pack(pady=(0, 5))
        
        subtitle = ttk.Label(
            self.main_frame,
            text="Professional Masofaviy Yordam Xizmati",
            font=("Segoe UI", 10)
        )
        subtitle.pack(pady=(0, 20))
        
        # Show registration form
        self.show_registration()
        
    def show_registration(self):
        # Clear main frame
        for widget in self.main_frame.winfo_children():
            widget.destroy()
            
        # Title
        title = ttk.Label(
            self.main_frame,
            text="🛠️ Ustajon Support",
            font=("Segoe UI", 24, "bold")
        )
        title.pack(pady=(0, 5))
        
        subtitle = ttk.Label(
            self.main_frame,
            text="Professional Masofaviy Yordam Xizmati",
            font=("Segoe UI", 10)
        )
        subtitle.pack(pady=(0, 20))
        
        # Registration form frame
        form_frame = ttk.LabelFrame(self.main_frame, text="Ro'yxatdan o'tish", padding="15")
        form_frame.pack(fill=tk.X, pady=10)
        
        # Name
        ttk.Label(form_frame, text="Ismingiz:", font=("Segoe UI", 10)).pack(anchor=tk.W)
        self.name_entry = ttk.Entry(form_frame, font=("Segoe UI", 12))
        self.name_entry.pack(fill=tk.X, pady=(5, 10))
        
        # Phone
        ttk.Label(form_frame, text="Telefon raqamingiz:", font=("Segoe UI", 10)).pack(anchor=tk.W)
        self.phone_entry = ttk.Entry(form_frame, font=("Segoe UI", 12))
        self.phone_entry.pack(fill=tk.X, pady=(5, 10))
        
        # Problem
        ttk.Label(form_frame, text="Muammo tavsifi:", font=("Segoe UI", 10)).pack(anchor=tk.W)
        self.problem_text = tk.Text(form_frame, height=4, font=("Segoe UI", 11))
        self.problem_text.pack(fill=tk.X, pady=(5, 10))
        
        # Status label
        self.status_label = ttk.Label(self.main_frame, text="", font=("Segoe UI", 9))
        self.status_label.pack(pady=5)
        
        # Submit button
        self.submit_btn = ttk.Button(
            self.main_frame,
            text="📋 Ro'yxatdan o'tish",
            command=self.register
        )
        self.submit_btn.pack(pady=10)
        
        # Progress
        self.progress = ttk.Progressbar(self.main_frame, mode='indeterminate')
        
    def show_chat(self):
        # Clear main frame
        for widget in self.main_frame.winfo_children():
            widget.destroy()
            
        # Header
        header_frame = ttk.Frame(self.main_frame)
        header_frame.pack(fill=tk.X, pady=(0, 10))
        
        ttk.Label(
            header_frame,
            text="💬 Mutaxassis bilan chat",
            font=("Segoe UI", 16, "bold")
        ).pack(side=tk.LEFT)
        
        ttk.Label(
            header_frame,
            text=f"ID: {self.client_id}",
            font=("Segoe UI", 9)
        ).pack(side=tk.RIGHT)
        
        # RustDesk ID display
        if self.rustdesk_id:
            rd_frame = ttk.Frame(self.main_frame)
            rd_frame.pack(fill=tk.X, pady=5)
            ttk.Label(
                rd_frame,
                text=f"🖥️ RustDesk ID: {self.rustdesk_id}",
                font=("Segoe UI", 10, "bold"),
                foreground="green"
            ).pack()
        
        # Chat display
        self.chat_display = scrolledtext.ScrolledText(
            self.main_frame,
            height=15,
            font=("Segoe UI", 10),
            wrap=tk.WORD,
            state=tk.DISABLED
        )
        self.chat_display.pack(fill=tk.BOTH, expand=True, pady=10)
        
        # Configure tags
        self.chat_display.tag_configure("system", foreground="#888888", font=("Segoe UI", 9, "italic"))
        self.chat_display.tag_configure("client", foreground="#0066cc")
        self.chat_display.tag_configure("specialist", foreground="#cc6600")
        
        # Message input frame
        input_frame = ttk.Frame(self.main_frame)
        input_frame.pack(fill=tk.X, pady=5)
        
        self.message_entry = ttk.Entry(input_frame, font=("Segoe UI", 11))
        self.message_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 10))
        self.message_entry.bind('<Return>', lambda e: self.send_message())
        
        ttk.Button(
            input_frame,
            text="📤 Yuborish",
            command=self.send_message
        ).pack(side=tk.RIGHT)
        
        # Add welcome message
        self.add_chat_message("Tizim", "Salom! Mutaxassis tez orada sizga ulanadi. Iltimos, kuting...", "system")
        
        # Start chat polling
        self.start_chat_polling()
        
    def add_chat_message(self, sender, message, tag="client"):
        self.chat_display.config(state=tk.NORMAL)
        timestamp = time.strftime("%H:%M")
        self.chat_display.insert(tk.END, f"[{timestamp}] {sender}: ", tag)
        self.chat_display.insert(tk.END, f"{message}\n")
        self.chat_display.see(tk.END)
        self.chat_display.config(state=tk.DISABLED)
        
    def register(self):
        name = self.name_entry.get().strip()
        phone = self.phone_entry.get().strip()
        problem = self.problem_text.get("1.0", tk.END).strip()
        
        if not name or not phone:
            messagebox.showerror("Xato", "Iltimos, ismingiz va telefon raqamingizni kiriting!")
            return
            
        self.client_name = name
        self.client_phone = phone
        
        self.submit_btn.config(state=tk.DISABLED)
        self.status_label.config(text="⏳ Ro'yxatdan o'tilmoqda...")
        self.progress.pack(pady=5)
        self.progress.start()
        
        # Run in thread
        threading.Thread(target=lambda: self.register_thread(name, phone, problem), daemon=True).start()
        
    def register_thread(self, name, phone, problem):
        try:
            # Step 1: Install RustDesk
            self.update_status("⏳ RustDesk o'rnatilmoqda...")
            self.install_rustdesk()
            
            # Step 2: Get system info
            self.update_status("⏳ Tizim ma'lumotlari olinmoqda...")
            system_info = self.get_system_info()
            
            # Step 3: Register with server
            self.update_status("⏳ Serverga ulanilmoqda...")
            
            data = {
                "client_id": self.client_id,
                "name": name,
                "phone": phone,
                "problem": problem,
                "system_info": system_info,
                "rustdesk_id": self.rustdesk_id
            }
            
            self.send_request("/api/register", data)
            
            # Success
            self.registered = True
            self.root.after(0, self.on_registration_complete)
            
        except Exception as e:
            self.root.after(0, lambda: self.on_registration_error(str(e)))
            
    def update_status(self, text):
        self.root.after(0, lambda: self.status_label.config(text=text))
        
    def on_registration_complete(self):
        self.progress.stop()
        self.progress.pack_forget()
        self.show_chat()
        
    def on_registration_error(self, error):
        self.progress.stop()
        self.progress.pack_forget()
        self.submit_btn.config(state=tk.NORMAL)
        self.status_label.config(text=f"❌ Xato: {error}")
        messagebox.showerror("Xato", f"Ro'yxatdan o'tishda xato:\n{error}")
        
    def install_rustdesk(self):
        """Download and install RustDesk silently"""
        try:
            if platform.system() != "Windows":
                self.rustdesk_id = "DEMO-MODE"
                return
                
            # Check if already installed
            rustdesk_path = os.path.join(os.environ.get("ProgramFiles", "C:\\Program Files"), "RustDesk", "rustdesk.exe")
            if os.path.exists(rustdesk_path):
                self.rustdesk_id = self.get_rustdesk_id()
                if self.rustdesk_id:
                    self.configure_rustdesk()
                    return
            
            # Download RustDesk
            self.update_status("⏳ RustDesk yuklanmoqda...")
            download_url = "https://github.com/rustdesk/rustdesk/releases/download/1.2.7/rustdesk-1.2.7-x86_64.exe"
            
            temp_dir = tempfile.gettempdir()
            installer_path = os.path.join(temp_dir, "rustdesk_setup.exe")
            
            # Download with progress
            ssl_context = ssl.create_default_context()
            ssl_context.check_hostname = False
            ssl_context.verify_mode = ssl.CERT_NONE
            
            req = urllib.request.Request(download_url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req, context=ssl_context, timeout=120) as response:
                with open(installer_path, 'wb') as f:
                    f.write(response.read())
            
            # Install silently
            self.update_status("⏳ RustDesk o'rnatilmoqda...")
            subprocess.run([installer_path, "--silent-install"], check=True, timeout=120)
            
            time.sleep(3)
            
            # Configure
            self.configure_rustdesk()
            
            # Get ID
            self.rustdesk_id = self.get_rustdesk_id()
            
        except Exception as e:
            print(f"RustDesk install error: {e}")
            self.rustdesk_id = f"ERROR-{self.client_id}"
            
    def configure_rustdesk(self):
        """Configure RustDesk with our server"""
        try:
            if platform.system() != "Windows":
                return
                
            config_dir = os.path.join(os.environ.get("APPDATA", ""), "RustDesk", "config")
            os.makedirs(config_dir, exist_ok=True)
            
            # Custom server config
            config = {
                "custom-rendezvous-server": RUSTDESK_SERVER,
                "relay-server": RUSTDESK_SERVER,
                "api-server": "",
                "key": RUSTDESK_KEY
            }
            
            config_path = os.path.join(config_dir, "RustDesk2.toml")
            with open(config_path, "w") as f:
                for key, value in config.items():
                    f.write(f'{key} = "{value}"\n')
                    
            # Set password
            password_config = {
                "password": RUSTDESK_PASSWORD
            }
            
            password_path = os.path.join(config_dir, "RustDesk.toml")
            with open(password_path, "w") as f:
                for key, value in password_config.items():
                    f.write(f'{key} = "{value}"\n')
                    
        except Exception as e:
            print(f"Configure error: {e}")
            
    def get_rustdesk_id(self):
        """Get RustDesk ID from config"""
        try:
            if platform.system() != "Windows":
                return "DEMO-ID"
                
            config_dir = os.path.join(os.environ.get("APPDATA", ""), "RustDesk", "config")
            id_path = os.path.join(config_dir, "RustDesk.id")
            
            if os.path.exists(id_path):
                with open(id_path, "r") as f:
                    return f.read().strip()
                    
            # Try to find from registry or other locations
            return None
            
        except Exception as e:
            print(f"Get ID error: {e}")
            return None
            
    def get_system_info(self):
        """Collect system information"""
        try:
            info = {
                "os": platform.system(),
                "os_version": platform.version(),
                "os_release": platform.release(),
                "machine": platform.machine(),
                "processor": platform.processor(),
                "hostname": socket.gethostname(),
                "username": os.getlogin() if hasattr(os, 'getlogin') else "unknown",
                "python_version": platform.python_version()
            }
            
            # Try to get more info on Windows
            if platform.system() == "Windows":
                try:
                    import ctypes
                    kernel32 = ctypes.windll.kernel32
                    c_ulong = ctypes.c_ulong
                    
                    class MEMORYSTATUS(ctypes.Structure):
                        _fields_ = [
                            ('dwLength', c_ulong),
                            ('dwMemoryLoad', c_ulong),
                            ('dwTotalPhys', c_ulong),
                            ('dwAvailPhys', c_ulong),
                            ('dwTotalPageFile', c_ulong),
                            ('dwAvailPageFile', c_ulong),
                            ('dwTotalVirtual', c_ulong),
                            ('dwAvailVirtual', c_ulong)
                        ]
                    
                    mem = MEMORYSTATUS()
                    mem.dwLength = ctypes.sizeof(MEMORYSTATUS)
                    kernel32.GlobalMemoryStatus(ctypes.byref(mem))
                    info["total_ram_mb"] = mem.dwTotalPhys // (1024 * 1024)
                except:
                    pass
                    
            return info
            
        except Exception as e:
            return {"error": str(e)}
            
    def send_request(self, endpoint, data):
        """Send HTTP request to server"""
        try:
            url = SERVER_URL + endpoint
            json_data = json.dumps(data).encode('utf-8')
            
            req = urllib.request.Request(
                url,
                data=json_data,
                headers={'Content-Type': 'application/json'}
            )
            
            with urllib.request.urlopen(req, timeout=30) as response:
                return json.loads(response.read().decode())
                
        except Exception as e:
            print(f"Request error: {e}")
            raise
            
    def send_message(self):
        """Send chat message"""
        message = self.message_entry.get().strip()
        if not message:
            return
            
        self.message_entry.delete(0, tk.END)
        self.add_chat_message(self.client_name, message, "client")
        
        # Send to server in background
        threading.Thread(
            target=lambda: self.send_message_thread(message),
            daemon=True
        ).start()
        
    def send_message_thread(self, message):
        try:
            data = {
                "client_id": self.client_id,
                "sender": "client",
                "message": message
            }
            self.send_request("/api/chat/send", data)
        except Exception as e:
            print(f"Send message error: {e}")
            
    def start_chat_polling(self):
        """Poll for new messages"""
        def poll():
            while self.registered:
                try:
                    data = {"client_id": self.client_id, "last_id": 0}
                    result = self.send_request("/api/chat/messages", data)
                    
                    if result.get("messages"):
                        for msg in result["messages"]:
                            if msg.get("sender") == "specialist":
                                self.root.after(0, lambda m=msg: self.add_chat_message(
                                    "Mutaxassis",
                                    m.get("message", ""),
                                    "specialist"
                                ))
                                
                except Exception as e:
                    print(f"Poll error: {e}")
                    
                time.sleep(3)
                
        threading.Thread(target=poll, daemon=True).start()
        
    def run(self):
        self.root.mainloop()


def main():
    try:
        app = UstajonClient()
        app.run()
    except Exception as e:
        print(f"Error: {e}")
        

if __name__ == "__main__":
    main()
