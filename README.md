## 🌟 Sorahk

Sorahk is a Rust-based auto-key press tool for Windows, providing configurable key repeat automation with a graphical interface. Built using the Windows crate, it offers low-latency input handling and runs efficiently in the system tray with minimal resource usage.

⚠️ **Windows Only**: This application is designed specifically for Windows and requires Windows 10 or later.

## ✨ Features

- 🖥️ **Graphical Interface** – Modern GUI with settings management and real-time status monitoring
- 🎨 **Theme Support** – Configurable light/dark themes with persistent preferences
- 🎯 **Process Whitelist** – Optional filtering to restrict turbo-fire to specific applications
- 🔔 **System Notifications** – Windows Toast notifications for status updates
- ⚙️ **TOML Configuration** – Simple configuration file with automatic generation of defaults
- 🔑 **Flexible Key Mapping** – Map any trigger key to auto-repeat any target key
- ⏱️ **Adjustable Intervals** – Configure repeat interval and press duration per mapping
- 🪟 **System Tray Integration** – Optional tray icon for background operation
- ⚡ **Multi-threaded Processing** – Worker pool with load balancing for efficient key handling
- 🔒 **Native Input Injection** – Uses Windows keyboard event APIs for reliable operation

## 🛠️ Configuration

Sorahk reads settings from `Config.toml` in the executable directory. If the file doesn't exist, a default configuration is created automatically.

Example `Config.toml`:

```toml
show_tray_icon = true        # Show system tray icon on startup
show_notifications = true    # Enable/disable system notifications
always_on_top = false        # Keep window always on top of other windows
dark_mode = false            # Use dark theme (false = light theme, true = dark theme)
input_timeout = 10           # Input timeout in ms
interval = 5                 # Default repeat interval between keystrokes (ms)
event_duration = 5           # Duration of each simulated key press (ms)
worker_count = 0             # Number of turbo workers (0 = auto-detect based on CPU cores)
switch_key = "DELETE"        # Reserved key to toggle Sorahk behavior

# Process whitelist (empty = all processes enabled)
# Only processes in this list will have turbo-fire enabled
process_whitelist = []       # Example: ["notepad.exe", "game.exe"]

# Key mapping definitions
[[mappings]]
trigger_key = "Q"            # Physical key you press
target_key = "Q"             # Key that gets repeatedly sent
interval = 5                 # Optional: override global interval
event_duration = 5           # Optional: override global press duration

[[mappings]]
trigger_key = "W"
target_key = "F"             # Pressing 'W' will rapidly fire 'F'
```

💡 **Note**: Key names must match Windows virtual key codes (e.g., "A", "F1", "LWIN", "RETURN", "DELETE"). Full support for standard keys is included.

## ▶️ Usage

1. Download or build `sorahk.exe`
2. Place it in any directory
3. Run the executable - it will create a default `Config.toml` if none exists
4. Use the GUI to modify settings or edit `Config.toml` directly
5. Press the configured switch key (default: DELETE) to enable/disable turbo-fire

## 🧪 Building from Source

**Prerequisites:**
- Rust (stable channel) via [rustup](https://rustup.rs/)
- Windows 10 or later

**Build Steps:**

```bash
git clone https://github.com/llnut/Sorahk.git
cd Sorahk
cargo build --release
```

The executable will be at: `target\release\sorahk.exe`

## 🤝 Contributing

Contributions are welcome! Please open issues for bugs or feature requests, and submit pull requests for improvements. Ensure code follows Rust conventions and maintains the project's focus on reliability and efficiency.

## 📄 License

MIT License

## 🙌 Acknowledgements

- Built with Rust for memory safety and concurrency
- Uses the [`windows`](https://crates.io/crates/windows) crate for Windows API access
- UI powered by [`egui`](https://crates.io/crates/egui) and [`eframe`](https://crates.io/crates/eframe)
- Toast notifications via Windows Runtime APIs
