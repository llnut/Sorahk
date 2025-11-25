<div align="center">

# 🌸 Sorahk 🌸

### ✨ *A Lightweight, Efficient Auto Key Press Tool* ✨

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows-blue?style=flat-square&logo=windows" alt="Platform"/>
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust" alt="Language"/>
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License"/>
  <img src="https://img.shields.io/badge/GUI-egui-purple?style=flat-square" alt="GUI"/>
</p>

---

</div>

## 📖 Overview

**Sorahk** is a Rust-based auto-key press tool designed for Windows, providing configurable key repeat automation with an anime-inspired graphical interface. Built using the Windows crate and egui framework, it offers low-latency input handling and runs efficiently in the system tray with minimal resource usage.

Suitable for gaming, productivity automation, and other scenarios requiring rapid key repetition. 🎮

> ⚠️ **Platform Requirement**: This application is designed exclusively for Windows and requires **Windows 10 or later**.

---

## 🎨 Screenshots

> 💡 *Click on any image to view full size!*

<div align="center">

### 🌸 Light Theme
*Pastel colors with anime-inspired design*

<table>
  <tr>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/07590bde-4169-40aa-8d8b-4c708f88f87f">
        <img src="https://github.com/user-attachments/assets/07590bde-4169-40aa-8d8b-4c708f88f87f" alt="Main Window - Light Theme" width="100%"/>
      </a>
      <br/>
      <sub>🖥️ <b>Main Window</b></sub>
    </td>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/894bd83c-954d-4863-8aad-8ae55e991e7f">
        <img src="https://github.com/user-attachments/assets/894bd83c-954d-4863-8aad-8ae55e991e7f" alt="Settings Dialog - Light Theme" width="100%"/>
      </a>
      <br/>
      <sub>⚙️ <b>Settings Dialog</b></sub>
    </td>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/4e94f0be-83db-496f-9786-d481422c78dc">
        <img src="https://github.com/user-attachments/assets/4e94f0be-83db-496f-9786-d481422c78dc" alt="About Dialog - Light Theme" width="100%"/>
      </a>
      <br/>
      <sub>💫 <b>About Dialog</b></sub>
    </td>
  </tr>
</table>

---

### 🌙 Dark Theme
*Dark interface optimized for extended use*

<table>
  <tr>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/8826544d-8dd7-41c3-8a80-9bfb420c25f7">
        <img src="https://github.com/user-attachments/assets/8826544d-8dd7-41c3-8a80-9bfb420c25f7" alt="Main Window - Dark Theme" width="100%"/>
      </a>
      <br/>
      <sub>🖥️ <b>Main Window</b></sub>
    </td>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/30c2094b-ec77-4f28-8b2c-2049707a62b0">
        <img src="https://github.com/user-attachments/assets/30c2094b-ec77-4f28-8b2c-2049707a62b0" alt="Settings Dialog - Dark Theme" width="100%"/>
      </a>
      <br/>
      <sub>⚙️ <b>Settings Dialog</b></sub>
    </td>
    <td align="center" width="33%">
      <a href="https://github.com/user-attachments/assets/2467be45-cc88-4cd6-8f33-ff3f2016c419">
        <img src="https://github.com/user-attachments/assets/2467be45-cc88-4cd6-8f33-ff3f2016c419" alt="About Dialog - Dark Theme" width="100%"/>
      </a>
      <br/>
      <sub>💫 <b>About Dialog</b></sub>
    </td>
  </tr>
</table>

---

### ✨ UI Highlights

<table>
  <tr>
    <td align="center" width="33%">
      <h3>🎨</h3>
      <b>Anime-inspired Design</b>
      <br/>
      <sub>Pastel palettes and gradient backgrounds</sub>
    </td>
    <td align="center" width="33%">
      <h3>✨</h3>
      <b>Borderless Interface</b>
      <br/>
      <sub>Clean interface with subtle shadows</sub>
    </td>
    <td align="center" width="33%">
      <h3>🌈</h3>
      <b>Dual Theme Support</b>
      <br/>
      <sub>Light and dark modes available</sub>
    </td>
  </tr>
  <tr>
    <td align="center" width="33%">
      <h3>🔄</h3>
      <b>Responsive Layout</b>
      <br/>
      <sub>Adapts to different window sizes</sub>
    </td>
    <td align="center" width="33%">
      <h3>📜</h3>
      <b>Scrollable Content</b>
      <br/>
      <sub>Supports multiple key mappings</sub>
    </td>
    <td align="center" width="33%">
      <h3>🎯</h3>
      <b>Process Whitelist</b>
      <br/>
      <sub>Target specific applications</sub>
    </td>
  </tr>
</table>
</div>

---

## ✨ Features

### 🎨 **User Interface**
- **Modern GUI** – Anime-inspired interface with intuitive settings management
- **Multi-language Support** – 4 languages available: English, 简体中文, 繁體中文, 日本語
- **Dual Theme Support** – Switch between light and dark themes with persistent preferences
- **Real-time Status** – Monitor active mappings and system state
- **System Tray Integration** – Minimize to tray for background operation

### ⚙️ **Core Functionality**
- **Flexible Input Mapping** – Map any trigger input (keyboard or mouse) to auto-repeat any target action
- **Advanced Combo Key Support** – Full combo key triggers and outputs with:
  - Single or multiple modifier keys (e.g., `LALT+A`, `LCTRL+LSHIFT+F`)
  - Left/right modifier distinction (e.g., `LSHIFT+A` and `RSHIFT+A` are treated as different triggers)
  - Single modifier keys as triggers (e.g., `LSHIFT` alone)
  - Multiple simultaneous combos with shared modifiers (e.g., `LALT+1`, `LALT+2`)
- **Mouse Button Support** – Full support for left, right, middle, and side mouse buttons (X1/X2)
- **Adjustable Timing** – Configure repeat interval and press duration per mapping
- **Global Toggle** – Quick enable/disable with a single hotkey (default: DELETE)
- **Process Whitelist** – Optional filtering to restrict turbo-fire to specific applications
- **Multi-input Support** – Configure multiple independent input mappings simultaneously

### 🚀 **Performance & Reliability**
- **Multi-threaded Processing** – Worker pool with load balancing for efficient key handling
- **Native Input Injection** – Uses Windows keyboard event APIs for reliable operation
- **Low Resource Usage** – Minimal CPU and memory footprint

### 🔔 **Additional Features**
- **Toast Notifications** – Windows native notifications for status updates
- **TOML Configuration** – Simple, human-readable configuration file
- **Auto-configuration** – Automatic generation of default settings on first run
- **Always on Top** – Optional window pinning for easy access

---

## 🎯 Use Cases

- 🎮 **Gaming** – Rapid fire in action games, auto-farming in MMORPGs
- 📝 **Productivity** – Automate repetitive text input tasks
- 🎨 **Creative Work** – Streamline workflow in design applications
- 🧪 **Testing** – Generate repetitive input for software testing

---

## 🛠️ Configuration

Sorahk reads settings from `Config.toml` located in the executable directory. If the file doesn't exist, a default configuration is created automatically on first launch.

### 📝 Example Configuration

```toml
# ═══════════════════════════════════════════════════════
#  🌸 Sorahk Configuration File 🌸
# ═══════════════════════════════════════════════════════

# ─── General Settings ───
show_tray_icon = true        # Show system tray icon on startup
show_notifications = false   # Enable/disable system notifications
always_on_top = false        # Keep window always on top of other windows
dark_mode = false            # Use dark theme (false = light theme, true = dark theme)
language = "English"         # UI language: "English", "SimplifiedChinese", "TraditionalChinese", "Japanese"

# ─── Performance Settings ───
input_timeout = 10           # Input timeout in ms
interval = 5                 # Default repeat interval between keystrokes (ms)
event_duration = 5           # Duration of each simulated key press (ms)
worker_count = 0             # Number of turbo workers (0 = auto-detect based on CPU cores)

# ─── Control Settings ───
switch_key = "DELETE"        # Reserved key to toggle SoraHK behavior

# ─── Process Whitelist ───
# Process whitelist (empty = all processes enabled)
# Only processes in this list will have turbo-fire enabled
process_whitelist = []       # Example: ["notepad.exe", "game.exe"]

# ─── Key Mappings ───
# Key mapping definitions
[[mappings]]
trigger_key = "A"            # Physical key you press
target_key = "A"             # Key that gets repeatedly sent
interval = 5                 # Override global interval
event_duration = 5           # Override global press duration
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "B"            # Physical key you press
target_key = "F"             # Key that gets repeatedly sent
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "F1"           # Physical key you press
target_key = "1"             # Key that gets repeatedly sent
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "F2"           # Physical key you press
target_key = "2"             # Key that gets repeatedly sent
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "LSHIFT"       # Physical key you press
target_key = "3"             # Key that gets repeatedly sent
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

# Mouse button examples
[[mappings]]
trigger_key = "LBUTTON"      # Left mouse button trigger
target_key = "LBUTTON"       # Auto-click left button
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "XBUTTON1"     # Side button 1 trigger
target_key = "SPACE"         # Press space when side button is held
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

# Key combination examples
# Use '+' to separate keys for combo triggers and outputs
[[mappings]]
trigger_key = "LALT+A"       # Press left ALT and A together
target_key = "B"             # Auto-press B key
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "LALT+1"       # Left ALT + 1
target_key = "F1"            # Auto-press F1
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press)

[[mappings]]
trigger_key = "LCTRL+LSHIFT+F" # Multiple modifiers
target_key = "LALT+F4"       # Output can also be combo
turbo_enabled = true         # Enable turbo mode (true = auto-repeat, false = single press) (close window)

[[mappings]]
trigger_key = "LSHIFT"       # Single modifier key as trigger
target_key = "SPACE"         # Auto-press space when holding left Shift

# Note: Left and right modifiers are distinguished (LSHIFT ≠ RSHIFT)
# Note: Multiple combos with shared modifiers work simultaneously
# Example: LALT+1 → auto-fire 1, LALT+2 → auto-fire 2 (both can work at once)
```

### 🔑 Supported Input Names

Input names support both keyboard keys and mouse buttons:

**Keyboard Keys:**
- **Letters**: `A`, `B`, `C`, ..., `Z`
- **Numbers**: `0`, `1`, `2`, ..., `9`
- **Numpad**: `NUMPAD0`, `NUMPAD1`, ..., `NUMPAD9`, `MULTIPLY`, `ADD`, `SUBTRACT`, `DECIMAL`, `DIVIDE`
- **Function Keys**: `F1`, `F2`, ..., `F24`
- **Navigation**: `UP`, `DOWN`, `LEFT`, `RIGHT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`
- **Editing**: `SPACE`, `RETURN`, `TAB`, `ESCAPE`, `BACKSPACE`, `DELETE`, `INSERT`
- **System**: `SNAPSHOT` (Print Screen), `PAUSE`, `SCROLL` (Scroll Lock), `CAPITAL` (Caps Lock), `NUMLOCK`
- **Modifiers**: `LSHIFT`, `RSHIFT`, `LCTRL`, `RCTRL`, `LALT`, `RALT`, `LWIN`, `RWIN`
  - Generic forms: `SHIFT`, `CTRL`, `ALT`, `WIN` (mapped to left variant: LSHIFT, LCTRL, LALT, LWIN)
  - Can be used alone as triggers (e.g., `LSHIFT` to auto-fire on left Shift press)
  - **Important**: `LSHIFT` and `RSHIFT` are distinct keys; `LSHIFT+A` will only trigger with left Shift
- **OEM Keys**: `OEM_1` (;:), `OEM_2` (/?), `OEM_3` (\`~), `OEM_4` ([{), `OEM_5` (\\|), `OEM_6` (]}), `OEM_7` ('"), `OEM_PLUS` (=+), `OEM_COMMA` (,<), `OEM_MINUS` (-\_), `OEM_PERIOD` (.>), `OEM_102` (<>)

**Key Combinations:**
- Combine keys with `+`: `LALT+A`, `LCTRL+LSHIFT+F`, `LALT+RSHIFT+1`
- Both trigger and target can be combos: `LCTRL+C` → `LCTRL+V`
- Multiple combos with shared modifiers work simultaneously
- **Important**: Modifiers are matched exactly; `LSHIFT+1` requires left Shift, `RSHIFT+1` requires right Shift

**Mouse Buttons:**
- **Left Button**: `LBUTTON`, `LMOUSE`, `LMB`
- **Right Button**: `RBUTTON`, `RMOUSE`, `RMB`
- **Middle Button**: `MBUTTON`, `MMOUSE`, `MMB`
- **Side Button 1**: `XBUTTON1`, `X1`, `MB4`
- **Side Button 2**: `XBUTTON2`, `X2`, `MB5`

**GUI Capture Notes:**
- **Print Screen (SNAPSHOT)**: Requires two key presses to capture reliably due to Windows API timing constraints
- **Windows Keys (LWIN/RWIN)**: Can be captured, but will trigger the Start menu; return to the application to verify capture success
- **Other Keys**: All other keys (including Pause, Scroll Lock) can be captured with a single press

Full support for standard Windows virtual key codes and mouse buttons is included.

---

## 🚀 Getting Started

### ▶️ Quick Start

1. **Download** or build `sorahk.exe`
2. **Place** it in any directory of your choice
3. **Run** the executable – it will auto-generate `Config.toml` on first launch
4. **Configure** settings using the GUI or by editing `Config.toml` directly
5. **Press** the switch key (default: `DELETE`) to toggle turbo-fire on/off

### 🎮 GUI Key Capture

When capturing keys through the settings dialog:

1. **Click the Capture button** for the trigger or target key field
2. **Press the desired key(s)** – the capture completes when the first key is released
3. For **combo keys**, hold all keys down, then release to capture the full combination
4. For **single modifier keys** (e.g., `LSHIFT` alone), press and release the modifier

**Special Key Behavior:**
- **Print Screen (SNAPSHOT)**: Press the key twice to capture reliably. This is due to Windows API timing constraints and does not affect functionality once configured.
- **Windows Keys (LWIN/RWIN)**: Capturing these keys will trigger the Start menu. Return to the application window to verify the capture was successful. Once configured, these keys will function normally for auto-fire without triggering the Start menu.

### 🔨 Building from Source

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable channel via rustup)
- Windows 10 or later

**Build Steps:**

```bash
# Clone the repository
git clone https://github.com/llnut/Sorahk.git
cd Sorahk

# Build release version
cargo build --release

# The executable will be at: target\release\sorahk.exe
```

---

## 🧪 Testing

Sorahk includes a test suite covering core functionality. For detailed information, see [TESTING.md](TESTING.md).

### Quick Start

Run all tests on Windows:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --lib config::tests

# Run with verbose output
cargo test -- --nocapture
```

Or use the provided test script:

```batch
run_tests.bat
```

### Test Coverage

- **Configuration Management**: Loading, saving, and validation
- **Key Mapping**: Virtual key code conversion and scancode mapping
- **Mouse Support**: Button name parsing and event handling
- **Internationalization**: Multi-language support and translations
- **Worker Pool**: Event distribution and multi-threading
- **Integration**: Cross-module interactions and data persistence

For additional testing documentation, see [TESTING.md](TESTING.md).

---

## 📊 Performance Benchmarks

**Sorahk v0.3.0** has been benchmarked against **AutoHotkey v2.0.19** using microsecond-precision measurements. The tests were conducted with a 5ms target interval (200 Hz) across three scenarios.

### 🔬 Test Methodology

Performance data was collected using [sorahk-perf-monitor](https://github.com/llnut/sorahk-perf-monitor), a specialized keyboard event recorder with RDTSC-based timing. Each test recorded 1000 events to ensure statistical reliability.

### 📈 Results Summary

<table>
<tr>
<th>Test Scenario</th>
<th>Tool</th>
<th>Avg Interval</th>
<th>Actual Rate</th>
<th>Std Deviation</th>
</tr>
<tr>
<td rowspan="2"><b>Single Key (Same)</b><br/><sub>A → A</sub></td>
<td>Sorahk v0.3.0</td>
<td>9.47 ms</td>
<td>105.55 Hz</td>
<td>6.69 ms</td>
</tr>
<tr>
<td>AutoHotkey v2.0.19</td>
<td>32.67 ms</td>
<td>30.61 Hz</td>
<td>8.21 ms</td>
</tr>
<tr>
<td rowspan="2"><b>Single Key (Different)</b><br/><sub>A → B</sub></td>
<td>Sorahk v0.3.0</td>
<td>12.90 ms</td>
<td>77.53 Hz</td>
<td>4.23 ms</td>
</tr>
<tr>
<td>AutoHotkey v2.0.19</td>
<td>32.23 ms</td>
<td>31.03 Hz</td>
<td>8.03 ms</td>
</tr>
<tr>
<td rowspan="2"><b>Concurrent (3 Keys)</b><br/><sub>A→1, B→2, C→3</sub></td>
<td>Sorahk v0.3.0</td>
<td>15.63 ms</td>
<td>64.00 Hz</td>
<td>0.96 ms</td>
</tr>
<tr>
<td>AutoHotkey v2.0.19</td>
<td>49.13 ms</td>
<td>20.43 Hz</td>
<td>32.35 ms</td>
</tr>
</table>

*Concurrent test values represent averages across all three keys.*

### 🎯 Key Observations

**Single Key Performance:**
- Sorahk v0.3.0 shows **3.4x higher** turbo rate compared to AutoHotkey v2.0.19 in same-key scenarios
- Sorahk v0.3.0 shows **2.5x higher** turbo rate compared to AutoHotkey v2.0.19 in cross-key scenarios
- Both tools maintain consistent performance characteristics in single-key workloads

**Concurrent Performance:**
- Sorahk v0.3.0 maintains a **64 Hz** average rate across three simultaneous turbo keys
- AutoHotkey v2.0.19 averages **20 Hz** under the same conditions, resulting in **3.1x difference**
- Sorahk's multi-threaded architecture scales well with multiple active mappings

**Timing Consistency:**
- Sorahk v0.3.0 shows **0.96ms standard deviation** in concurrent scenarios
- AutoHotkey v2.0.19 shows **32.35ms standard deviation** under the same conditions
- Sorahk's lock-free worker pool architecture contributes to stable interval timing

### 📝 Test Environment

**Hardware Configuration:**
- **Processor**: Intel Core i9-13900H (13th Gen) @ 2.60 GHz base clock
- **Memory**: 32 GB RAM
- **Architecture**: x86_64

**Software Configuration:**
- **Operating System**: Windows 11 23H2
- **Sorahk Version**: 0.3.0
- **AutoHotkey Version**: 2.0.19
- **Measurement Tool**: sorahk-perf-monitor with RDTSC timing (calibrated at 2995.19 MHz)

**Test Parameters:**
- Target interval: 5ms (200 Hz theoretical maximum)
- Sample size: 1000 events per test
- Both tools configured with equivalent settings where applicable

---


## 🤝 Contributing

Contributions are accepted through issues and pull requests. Please report bugs or suggest features via issues. Code contributions should follow Rust conventions and maintain compatibility with existing functionality.

---

## 📄 License

**MIT License** – see the [LICENSE](LICENSE) file for details.

---

## 🙌 Acknowledgements

Sorahk is built using modern Rust technologies:

- 🦀 **[Rust](https://www.rust-lang.org/)** – Memory safety and zero-cost abstractions
- 🪟 **[windows-rs](https://crates.io/crates/windows)** – Native Windows API access
- 🎨 **[egui](https://crates.io/crates/egui)** – Immediate mode GUI framework
- 🖼️ **[eframe](https://crates.io/crates/eframe)** – egui application framework
- 📝 **[toml](https://crates.io/crates/toml)** – Configuration file parsing
