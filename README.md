<div align="center">

# 🌸 Sorahk 🌸

### *A lightweight, efficient auto key press tool.*

[![Platform](https://img.shields.io/badge/Platform-Windows-blue?style=flat-square&logo=windows)](https://www.microsoft.com/windows)
[![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![GUI](https://img.shields.io/badge/GUI-egui-purple?style=flat-square)](https://github.com/emilk/egui)

---

</div>

## Overview

Sorahk is a Windows application for automating repetitive key press operations. It provides configurable input-to-output mappings with adjustable timing controls through a graphical interface. The application runs in the system tray and uses native Windows APIs for input detection and event simulation.

**Platform Requirement**: Windows 10 or later.

---

## Screenshots

### Light Theme

**Main Interface**

<table>
  <tr>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/47a815d3-6e2f-4a8f-a154-f065b1688a41">
        <img src="https://github.com/user-attachments/assets/47a815d3-6e2f-4a8f-a154-f065b1688a41" width="100%"/>
      </a>
      <br/>
      <sub><b>Main Window</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/241278f9-7a83-41c6-85f1-93f83a22703a">
        <img src="https://github.com/user-attachments/assets/241278f9-7a83-41c6-85f1-93f83a22703a" width="100%"/>
      </a>
      <br/>
      <sub><b>Settings Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/e659ac9e-f062-4fb5-aedd-5d65c8338875">
        <img src="https://github.com/user-attachments/assets/e659ac9e-f062-4fb5-aedd-5d65c8338875" width="100%"/>
      </a>
      <br/>
      <sub><b>Device Manager Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/08231eb6-e6ab-4340-8e53-057d3d18c033">
        <img src="https://github.com/user-attachments/assets/08231eb6-e6ab-4340-8e53-057d3d18c033" width="100%"/>
      </a>
      <br/>
      <sub><b>Hid Activation Dialog</b></sub>
    </td>
  </tr>
</table>

<details>
<summary><b>Show More Screenshots</b> (4 additional)</summary>
<br/>

<table>
  <tr>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/eaf573f2-55c7-48d9-b4c2-41a7fa86ae2d">
        <img src="https://github.com/user-attachments/assets/eaf573f2-55c7-48d9-b4c2-41a7fa86ae2d" width="100%"/>
      </a>
      <br/>
      <sub><b>Mouse Direction Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/e18ace51-fbdd-401a-8f69-f072b2cd91b6">
        <img src="https://github.com/user-attachments/assets/e18ace51-fbdd-401a-8f69-f072b2cd91b6" width="100%"/>
      </a>
      <br/>
      <sub><b>Mouse Scroll Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/5bc200fa-6ca2-4e33-afce-ac0ef9204cbc">
        <img src="https://github.com/user-attachments/assets/5bc200fa-6ca2-4e33-afce-ac0ef9204cbc" width="100%"/>
      </a>
      <br/>
      <sub><b>About Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/e879e20d-56f2-4653-b144-98cef8f958e2">
        <img src="https://github.com/user-attachments/assets/e879e20d-56f2-4653-b144-98cef8f958e2" width="100%"/>
      </a>
      <br/>
      <sub><b>Close Dialog</b></sub>
    </td>
  </tr>
</table>

</details>

---

### Dark Theme

**Main Interface**

<table>
  <tr>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/fd75d412-84e6-4aae-81fe-5d8ba93638dc">
        <img src="https://github.com/user-attachments/assets/fd75d412-84e6-4aae-81fe-5d8ba93638dc" width="100%"/>
      </a>
      <br/>
      <sub><b>Main Window</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/d98bf3de-8e5e-4c3b-a605-b32a14f74c37">
        <img src="https://github.com/user-attachments/assets/d98bf3de-8e5e-4c3b-a605-b32a14f74c37" width="100%"/>
      </a>
      <br/>
      <sub><b>Settings Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/4bd4de07-7ae4-443b-9ff8-88279d86c021">
        <img src="https://github.com/user-attachments/assets/4bd4de07-7ae4-443b-9ff8-88279d86c021" width="100%"/>
      </a>
      <br/>
      <sub><b>Device Manager Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/28a348f5-bcb3-4b63-b66a-db497481aeaf">
        <img src="https://github.com/user-attachments/assets/28a348f5-bcb3-4b63-b66a-db497481aeaf" width="100%"/>
      </a>
      <br/>
      <sub><b>Hid Activation Dialog</b></sub>
    </td>
  </tr>
</table>

<details>
<summary><b>Show More Screenshots</b> (4 additional)</summary>
<br/>

<table>
  <tr>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/28a348f5-bcb3-4b63-b66a-db497481aeaf">
        <img src="https://github.com/user-attachments/assets/28a348f5-bcb3-4b63-b66a-db497481aeaf" width="100%"/>
      </a>
      <br/>
      <sub><b>Device Activation Dialog 2</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/7c203746-173c-4da0-83b1-5cf92420ccc0">
        <img src="https://github.com/user-attachments/assets/7c203746-173c-4da0-83b1-5cf92420ccc0" width="100%"/>
      </a>
      <br/>
      <sub><b>Settings Dialog 2</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/f1acbec4-a367-4ec0-a577-cb07b340cf5e">
        <img src="https://github.com/user-attachments/assets/f1acbec4-a367-4ec0-a577-cb07b340cf5e" width="100%"/>
      </a>
      <br/>
      <sub><b>About Dialog</b></sub>
    </td>
    <td align="center" width="25%">
      <a href="https://github.com/user-attachments/assets/67cd68c5-352c-4f2c-a776-38800da217e6">
        <img src="https://github.com/user-attachments/assets/67cd68c5-352c-4f2c-a776-38800da217e6" width="100%"/>
      </a>
      <br/>
      <sub><b>Close Dialog</b></sub>
    </td>
  </tr>
</table>

</details>

---

## Features

### User Interface

- Graphical configuration management through egui framework
- Multi-language support: English, Simplified Chinese, Traditional Chinese, Japanese
- Light and dark theme options
- System tray integration
- Real-time status monitoring

### Input/Output Support

- Single keys and combinations with modifier support
- Five standard mouse buttons (left, right, middle, X1, X2)
- Eight-directional cursor control with configurable speed
- Vertical scrolling with configurable wheel delta
- Xbox-compatible gamepad support via polling
- HID device integration for other controllers
- Input sequence detection for combo triggers
- Sequential output for macro execution

### Key Mapping

- Flexible input-to-output mapping configuration
- Support for key combinations as both triggers and targets
- Sequence input triggers with configurable time windows
- Sequential output targets with customizable intervals
- Three target modes: Single, Multi, Sequence
- Individual turbo mode control per mapping
- Adjustable repeat interval and press duration
- Multiple simultaneous input mappings
- Process whitelist for application-specific operation

### Performance

- Multi-threaded event processing with worker pool
- Lock-free concurrent data structures for sequence matching
- Atomic operations for minimal synchronization overhead
- Cache-aligned data structures for better CPU performance
- Ring buffer for efficient input history tracking
- Optional AVX2 SIMD acceleration (compile-time)
- SmallVec optimization to reduce heap allocations
- Optimized pattern matching for sequence detection
- Native Windows API integration for input handling

---

## Installation

### Binary Release

Download pre-built executables from the [releases page](https://github.com/llnut/Sorahk/releases):

- `sorahk-x.y.z-$target.zip` - Standard build (compatible with all x86_64 CPUs)
- `sorahk-avx2-x.y.z-$target.zip` - AVX2-optimized build (Intel 2013+ / AMD 2015+ CPUs)

Extract the archive and run `sorahk.exe`. The application will generate a default `Config.toml` on first launch.

### Building from Source

**Prerequisites:**

- [Rust](https://rustup.rs/) toolchain (stable channel)
- Windows 10 or later

**Build commands:**

```bash
# Clone repository
git clone https://github.com/llnut/Sorahk.git
cd Sorahk

# Standard build
cargo build --release

# AVX2-optimized build (requires AVX2 support)
# PowerShell:
$env:RUSTFLAGS="-C target-feature=+avx2"; cargo build --release

# CMD:
set RUSTFLAGS=-C target-feature=+avx2 && cargo build --release

# Auto-detect CPU features:
$env:RUSTFLAGS="-C target-cpu=native"; cargo build --release

# Output: target\release\sorahk.exe
```

---

## Configuration

Sorahk reads settings from `Config.toml` in the executable directory. The file is created automatically with default values on first run.

### Basic Example

```toml
# General settings
show_tray_icon = true
show_notifications = false
always_on_top = false
dark_mode = false
language = "English"

# Performance settings
input_timeout = 10
interval = 5
event_duration = 5
worker_count = 0

# Control settings
switch_key = "DELETE"

# Process whitelist (empty = all processes)
process_whitelist = []

# Key mappings
[[mappings]]
trigger_key = "A"
target_key = "A"
interval = 5
event_duration = 5
turbo_enabled = true

[[mappings]]
trigger_key = "LCTRL+C"
target_keys = ["LCTRL+V"]
turbo_enabled = true

# Sequence trigger example (fighting game combo)
[[mappings]]
trigger_sequence = "LS_Down,LS_DownRight,LS_Right,A"
target_keys = ["J"]
sequence_window_ms = 500
turbo_enabled = true

# Sequence output example (macro)
[[mappings]]
trigger_key = "F5"
target_keys = ["H", "E", "L", "L", "O"]
target_mode = 2
interval = 50
turbo_enabled = false
```

### Supported Input Types

Keyboard Keys:

- Letters: `A` to `Z`
- Numbers: `0` to `9`
- Function keys: `F1` to `F24`
- Navigation: `UP`, `DOWN`, `LEFT`, `RIGHT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`
- Editing: `SPACE`, `RETURN`, `TAB`, `ESCAPE`, `BACKSPACE`, `DELETE`, `INSERT`
- Modifiers: `LSHIFT`, `RSHIFT`, `LCTRL`, `RCTRL`, `LALT`, `RALT`, `LWIN`, `RWIN`
- Numpad: `NUMPAD0` to `NUMPAD9`, `MULTIPLY`, `ADD`, `SUBTRACT`, `DECIMAL`, `DIVIDE`
- System: `SNAPSHOT`, `PAUSE`, `SCROLL`, `CAPITAL`, `NUMLOCK`

Key Combinations:

- Format: `MODIFIER+KEY` (e.g., `LCTRL+C`, `LALT+RSHIFT+F1`)
- Left and right modifiers are distinguished

Mouse:

- Buttons: `LBUTTON`, `RBUTTON`, `MBUTTON`, `XBUTTON1`, `XBUTTON2`
- Movement: `MOUSE_UP`, `MOUSE_DOWN`, `MOUSE_LEFT`, `MOUSE_RIGHT`, `MOUSE_UP_LEFT`, etc.
- Scroll: `SCROLL_UP`, `SCROLL_DOWN`

XInput Controllers:

- Format: `GAMEPAD_VID_ButtonName` (e.g., `GAMEPAD_045E_A`)
- Buttons: `A`, `B`, `X`, `Y`, `Start`, `Back`, `LB`, `RB`, `LT`, `RT`, `LS_Click`, `RS_Click`
- D-Pad: `DPad_Up`, `DPad_Down`, `DPad_Left`, `DPad_Right`
- Analog sticks: `LS_Up`, `LS_Down`, `RS_Left`, `RS_Right`, etc.
- Combinations: `GAMEPAD_045E_LS_RightUp+A`

Raw Input Devices:

- Format: `DEVICE_VID_PID_SERIAL_Bx.x`
- Requires initial device activation to establish baseline data

Sequence Triggers:

- Format: `trigger_sequence = "Key1,Key2,Key3"`
- Comma-separated input sequence (e.g., `"DOWN,RIGHT,A"`)
- Configurable time window for completion (default: 500ms)
- Smart transition tolerance for intermediate inputs
- Bidirectional diagonal matching for XInput sticks

Sequence Targets:

- Format: `target_keys = ["Key1", "Key2", "Key3"]` with `target_mode = 2`
- Execute keys in sequential order
- Configurable interval between keys
- Turbo mode for repeating sequences

For complete configuration documentation, see the example `Config.toml` generated on first run.

---

## Usage

1. Run `sorahk.exe` to start the application
2. Configure mappings through the GUI or by editing `Config.toml`
3. Press the switch key (default: `DELETE`) to toggle operation
4. Application runs in system tray when minimized

### GUI Key Capture

When capturing input through the settings dialog:

1. Click the capture button for the desired field
2. Press the key or button to capture
3. For combinations, hold all keys/buttons, then release
4. Captured input appears in the field automatically

---

## Testing

Sorahk includes a test suite covering core functionality. Run tests with:

```bash
cargo test
```

For detailed testing information, see [TESTING.md](TESTING.md).

---

## Performance

Sorahk has been benchmarked against AutoHotkey v2.0.19 using microsecond-precision measurements with a 5ms target interval (200 Hz). Tests were conducted using [sorahk-perf-monitor](https://github.com/llnut/sorahk-perf-monitor), recording 1000 events per scenario.

### Results

<table>
<tr>
<th>Scenario</th>
<th>Tool</th>
<th>Avg Interval</th>
<th>Rate</th>
<th>Std Dev</th>
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

**Test Environment:**
- Processor: Intel Core i9-13900H @ 2.60 GHz
- Memory: 32 GB RAM
- OS: Windows 11 23H2
- Measurement: RDTSC-based timing (calibrated at 2995.19 MHz)

---

## Contributing

Bug reports and feature requests can be submitted through the issue tracker. Code contributions should follow Rust coding conventions and maintain compatibility with existing functionality.

---

## License

MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgements

Built with:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [windows-rs](https://crates.io/crates/windows) - Windows API bindings
- [egui](https://crates.io/crates/egui) - Immediate mode GUI framework
- [eframe](https://crates.io/crates/eframe) - Application framework for egui
- [toml](https://crates.io/crates/toml) - Configuration file parser
