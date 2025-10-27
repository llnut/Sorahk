## 🌟 Sorahk

  Sorahk is a pure Rust implementation of an AHK turbo function tool for Windows. it built solely on the windows crate and provides low-latency, low-overhead key repeat automation with a minimal binary footprint. Sorahk runs silently in the system tray and offers precise control over trigger-to-target key mapping and repeat intervals—ideal for users who need reliable, high-performance input macros without the bloat of a full scripting engine.

    ⚠️ Windows Only: Sorahk is designed specifically for Windows and will not work on macOS, Linux, or other operating systems.

## ✨ Features

- ⚡ **Extreme Performance** – Pure Rust implementation with zero-cost abstractions and consistent sub-millisecond response.
- 📦 **Minimal Footprint** – Highly optimized binary size; ideal for portable or low-resource use.
- 🪟 **Zero External Dependencies** – Built solely on the official `windows` crate; no .NET, C++ runtimes, or third-party DLLs.
- 🖥️ **Optional Tray Icon** – Runs silently in the background with a native Windows system tray interface.
- ⚙️ **Simple TOML Configuration** – Define trigger/target key pairs and repeat interval in `Config.toml`; no scripting required.
- 🔑 **Decoupled Trigger & Target Keys** – Bind any virtual key to auto-repeat a different target key.
- ⏱️ **Adjustable Repeat Interval** – Set inter-keystroke delay from 10 ms upward with millisecond precision.
- 🔒 **Low-Level Input Injection** – Uses Windows’ native keyboard event injection for reliable, high-priority delivery.

## 🛠️ Configuration

Sorahk reads its settings from a `Config.toml` file located in the same directory as the executable.
Example Config.toml:

```toml
show_tray_icon = true        # Show system tray icon on startup
show_notifications = false   # Enable/disable system notifications (may not work on stripped-down Windows)
input_timeout = 10           # Input timeout in ms (affects rapid-fire sequence termination)
interval = 5                 # Default repeat interval between keystrokes (ms)
event_duration = 5           # Duration of each simulated key press (ms)
switch_key = "DELETE"        # Reserved key to toggle Sorahk behavior (optional)

# Key mapping definitions
[[mappings]]
trigger_key = "A"            # Physical key you press
target_key = "A"             # Key that gets repeatedly sent
interval = 5                 # Optional: override global interval
event_duration = 5           # Optional: override global press duration

[[mappings]]
trigger_key = "B"
target_key = "F"             # Pressing 'B' will rapidly fire 'F'
```

    💡 Note: Key names must match Windows virtual key names (e.g., "A", "F1", "LWIN", "RETURN", "DELETE"). Full support for standard keys is included.


## 🧪 Building from Source

Sorahk requires Rust (stable channel) and is Windows-only.

Prerequisites:

    Install Rust via rustup.
    If you're using the GNU toolchain (e.g., x86_64-pc-windows-gnu), ensure MinGW-w64 is installed and available in your PATH. The MSVC toolchain (default on Windows) does not require MinGW.

Build Steps:

```bash
git clone https://github.com/llnut/Sorahk.git
cd Sorahk
cargo build --release
```

The optimized executable will be generated at:
`target\release\sorahk.exe`

    ✅ Tip: For maximum portability and smallest size, the release binary is statically linked and requires no external DLLs when built with the MSVC toolchain.

## 🤝 Contributing

Contributions are welcome! Whether it’s reporting bugs, suggesting new features, improving documentation, or submitting code—feel free to open an Issue or Pull Request on GitHub.

Please ensure your code follows Rust best practices and maintains the project’s focus on performance, simplicity.

## 📄 License

MIT License

## 🙌 Acknowledgements

- Built in Rust for memory safety and performance.
- Relies exclusively on the [`windows`](https://crates.io/crates/windows) crate for direct, safe access to Windows APIs.
- Designed for simplicity: drop `sorahk.exe` and `Config.toml` anywhere on Windows to run—no installation required.

