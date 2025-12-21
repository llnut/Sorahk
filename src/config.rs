//! Application configuration management.
//!
//! Handles loading, saving, and validation of application settings
//! including key mappings and runtime parameters.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::{collections::HashMap, fs, path::Path, str::FromStr};

use crate::i18n::Language;

/// Device API preference for input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum DeviceApiPreference {
    /// Auto-detect best API.
    #[default]
    Auto,
    /// Force XInput.
    XInput,
    /// Force Raw Input.
    RawInput,
}

/// XInput capture mode strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum XInputCaptureMode {
    /// Captures the most sustained input pattern.
    MostSustained,
    /// Captures the last stable input before release.
    LastStable,
    /// Prioritizes diagonal directions over straight directions with combo key support.
    #[default]
    DiagonalPriority,
}

impl FromStr for XInputCaptureMode {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LastStable" => Ok(Self::LastStable),
            "DiagonalPriority" => Ok(Self::DiagonalPriority),
            _ => Ok(Self::default()),
        }
    }
}

impl XInputCaptureMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MostSustained => "MostSustained",
            Self::LastStable => "LastStable",
            Self::DiagonalPriority => "DiagonalPriority",
        }
    }

    pub fn all_modes() -> &'static [XInputCaptureMode] {
        &[
            XInputCaptureMode::MostSustained,
            XInputCaptureMode::LastStable,
            XInputCaptureMode::DiagonalPriority,
        ]
    }
}

/// Main application configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Display tray icon
    pub show_tray_icon: bool,
    /// Show notification messages
    pub show_notifications: bool,
    /// Keep window always on top
    #[serde(default)]
    pub always_on_top: bool,
    /// Use dark theme mode
    #[serde(default)]
    pub dark_mode: bool,
    /// Application language
    #[serde(default)]
    pub language: Language,
    /// Toggle hotkey name
    pub switch_key: String,
    /// Key mapping configurations
    pub mappings: Vec<KeyMapping>,
    /// Input timeout in milliseconds
    #[serde(default = "default_input_timeout")]
    pub input_timeout: u64,
    /// Default key repeat interval in milliseconds
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// Default key press duration in milliseconds
    #[serde(default = "default_event_duration")]
    pub event_duration: u64,
    /// Worker thread count (0 for auto-detection)
    #[serde(default = "default_worker_count")]
    pub worker_count: usize,
    /// Process whitelist (empty means all processes)
    #[serde(default)]
    pub process_whitelist: Vec<String>,
    /// HID device baselines for button detection
    #[serde(default)]
    pub hid_baselines: Vec<HidDeviceBaseline>,
    /// Raw Input capture mode strategy
    #[serde(default = "default_capture_mode")]
    pub rawinput_capture_mode: String,
    /// XInput capture mode strategy
    #[serde(default = "default_xinput_capture_mode")]
    pub xinput_capture_mode: String,
    /// Device API preferences (VID:PID -> API preference)
    #[serde(default)]
    pub device_api_preferences: HashMap<String, DeviceApiPreference>,
}

/// HID device baseline configuration for button state detection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HidDeviceBaseline {
    /// Device identifier (vendor_id, product_id, serial or handle)
    pub device_id: String,
    /// Baseline HID data (idle state with no buttons pressed)
    pub baseline_data: Vec<u8>,
}

/// Key mapping configuration for trigger-target pairs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMapping {
    /// Trigger key name
    pub trigger_key: String,
    /// Target keys to send (supports multiple keys for simultaneous press)
    /// Uses SmallVec with inline capacity of 4 to reduce heap allocations for common cases
    #[serde(default = "default_target_keys")]
    pub target_keys: SmallVec<[String; 4]>,
    /// Optional override for repeat interval
    #[serde(default)]
    pub interval: Option<u64>,
    /// Optional override for press duration
    #[serde(default)]
    pub event_duration: Option<u64>,
    /// Enable turbo mode (auto-repeat)
    #[serde(default = "default_turbo_enabled")]
    pub turbo_enabled: bool,
    /// Mouse move speed in pixels per move (only for mouse movement)
    #[serde(default = "default_move_speed")]
    pub move_speed: i32,
}

fn default_move_speed() -> i32 {
    5
}

fn default_turbo_enabled() -> bool {
    true
}

fn default_target_keys() -> SmallVec<[String; 4]> {
    SmallVec::new()
}

impl KeyMapping {
    /// Gets the target keys slice
    pub fn get_target_keys(&self) -> &[String] {
        &self.target_keys
    }

    /// Adds a target key
    pub fn add_target_key(&mut self, key: String) {
        if !self.target_keys.contains(&key) {
            self.target_keys.push(key);
        }
    }

    /// Removes a target key
    pub fn remove_target_key(&mut self, key: &str) {
        self.target_keys.retain(|k| k != key);
    }

    /// Clears all target keys
    pub fn clear_target_keys(&mut self) {
        self.target_keys.clear();
    }

    /// Gets target keys as display string (comma separated)
    pub fn target_keys_display(&self) -> String {
        if self.target_keys.is_empty() {
            String::new()
        } else {
            self.target_keys.join(", ")
        }
    }
}

fn default_input_timeout() -> u64 {
    5
}
fn default_interval() -> u64 {
    5
}
fn default_event_duration() -> u64 {
    5
}
fn default_worker_count() -> usize {
    0 // 0 means auto-detect based on CPU cores
}
fn default_capture_mode() -> String {
    "MostSustained".to_string()
}
fn default_xinput_capture_mode() -> String {
    "MostSustained".to_string()
}

impl Default for AppConfig {
    /// Creates a default configuration with sensible defaults.
    fn default() -> Self {
        Self {
            show_tray_icon: true,
            show_notifications: true,
            always_on_top: false, // Default: not always on top for backward compatibility
            dark_mode: false,     // Default: light theme for backward compatibility
            language: Language::default(),
            switch_key: "DELETE".to_string(),
            mappings: vec![KeyMapping {
                trigger_key: "Q".to_string(),
                target_keys: SmallVec::from_vec(vec!["Q".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            }],
            input_timeout: default_input_timeout(),
            interval: default_interval(),
            event_duration: default_event_duration(),
            worker_count: default_worker_count(),
            process_whitelist: vec![], // Empty means all processes enabled
            hid_baselines: Vec::new(),
            rawinput_capture_mode: default_capture_mode(),
            xinput_capture_mode: default_xinput_capture_mode(),
            device_api_preferences: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from file, creating default if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            let default_config = Self::default();
            default_config.save_to_file(&path)?;
            return Ok(default_config);
        }
        Self::load_from_file(path)
    }

    /// Loads configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        // Validate configuration
        if config.input_timeout < 2 {
            config.input_timeout = 2;
        }
        if config.interval < 5 {
            config.interval = 5;
        }
        if config.event_duration < 2 {
            config.event_duration = 2;
        }

        // Deduplicate process whitelist
        config.process_whitelist.sort();
        config.process_whitelist.dedup();

        Ok(config)
    }

    /// Saves configuration to a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or serialized.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        // Generate header and main config in one go
        let header = format!(
            "# ═══════════════════════════════════════════════════════\n\
             #  🌸 Sorahk Configuration File 🌸\n\
             # ═══════════════════════════════════════════════════════\n\n\
             # ─── General Settings ───\n\
             show_tray_icon = {}       # Show system tray icon on startup\n\
             show_notifications = {}   # Enable/disable system notifications\n\
             always_on_top = {}       # Keep window always on top of other windows\n\
             dark_mode = {}           # Use dark theme (false = light theme, true = dark theme)\n\
             language = \"{}\"        # UI language: \"English\", \"SimplifiedChinese\", \"TraditionalChinese\", \"Japanese\"\n\n\
             # ─── Performance Settings ───\n\
             input_timeout = {}          # Input timeout in ms\n\
             interval = {}                # Default repeat interval between keystrokes (ms)\n\
             event_duration = {}          # Duration of each simulated key press (ms)\n\
             worker_count = {}            # Number of turbo workers (0 = auto-detect based on CPU cores)\n\n\
             # ─── XInput Settings ───\n\
             xinput_capture_mode = \"{}\"  # XInput capture mode: \"DiagonalPriority\", \"MostSustained\", \"LastStable\"\n\
                                           # DiagonalPriority: Prioritizes diagonal stick directions over straight directions\n\
                                           # MostSustained: Captures the most sustained input pattern\n\
                                           # LastStable: Captures the last stable input before release\n\n\
             # ─── Control Settings ───   \n\
             switch_key = \"{}\"       # Reserved key to toggle SoraHK behavior\n\n\
             # ─── Process Whitelist ───\n\
             # Process whitelist (empty = all processes enabled)\n\
             # Only processes in this list will have turbo-fire enabled\n\
             process_whitelist = {:?}      # Example: [\"notepad.exe\", \"game.exe\"]\n\n\
             # ─── Input Mappings ───\n\
             # Input mapping definitions (supports keyboard, mouse, and HID devices)\n\
             # Supported mouse buttons: LBUTTON, RBUTTON, MBUTTON, XBUTTON1, XBUTTON2\n\
             # Key combinations: Use '+' to separate keys (e.g., \"LALT+A\", \"RCTRL+RSHIFT+S\")\n\n\
             # Turbo Mode Behavior:\n\
             # - turbo_enabled = true: Auto-repeat with configurable interval (for rapid fire/continuous action)\n\
             # - turbo_enabled = false: Press-to-press, release-to-release behavior\n\
             #   * Keyboard targets: Supports Windows key repeat (holding trigger sends repeated key presses)\n\
             #   * Mouse button targets: Pure follow mode (press follows trigger press, release follows trigger release)\n\
             #   * Note: event_duration is ignored in non-turbo mode\n\n\
             # ─── Key Combo Examples ───\n\
             # Combo key mappings: Use '+' to separate keys\n\
             # - Supports modifier keys: LSHIFT/RSHIFT, LCTRL/RCTRL, LALT/RALT, LWIN/RWIN\n\
             # - Supports single modifier keys as triggers (e.g., \"LSHIFT\")\n\
             # - Multiple combos with shared modifiers work simultaneously (e.g., LALT+1, RALT+2)\n\
             # - Distinguishes left/right modifiers (e.g., \"LSHIFT+1\" only triggers with left Shift)\n\
             # Uncomment to enable combo key mappings:\n\
             # [[mappings]]\n\
             # trigger_key = \"LALT+1\"      # Left ALT + 1 (won't trigger with right ALT)\n\
             # target_keys = [\"F1\"]        # Auto-press F1\n\n\
             # [[mappings]]\n\
             # trigger_key = \"CTRL+SHIFT+F\"  # Multiple modifiers\n\
             # target_keys = [\"LALT+F4\"]     # Output can also be combo\n\n\
             # ─── Mouse Button Examples ───\n\
             # Uncomment to enable mouse button mappings:\n\
             # [[mappings]]\n\
             # trigger_key = \"LBUTTON\"     # Left mouse button trigger\n\
             # target_keys = [\"LBUTTON\"]   # Auto-click left button\n\n\
             # [[mappings]]\n\
             # trigger_key = \"RBUTTON\"     # Right mouse button trigger\n\
             # target_keys = [\"SPACE\"]     # Press space when right-clicking\n\n\
             # [[mappings]]\n\
             # trigger_key = \"XBUTTON1\"    # Side button 1 trigger\n\
             # target_keys = [\"F\"]         # Auto-press F key\n\n\
             # ─── Mouse Movement Examples ───\n\
             # [[mappings]]\n\
             # trigger_key = \"W\"           # Trigger key\n\
             # target_keys = [\"MOUSE_UP\"]  # Move cursor upward\n\
             # move_speed = 5              # Speed in pixels (1-100)\n\
             # interval = 5                # Update interval in ms\n\
             # turbo_enabled = true        # Required for continuous movement\n\n\
             # [[mappings]]\n\
             # trigger_key = \"S\"\n\
             # target_keys = [\"MOUSE_DOWN\"]\n\
             # move_speed = 5\n\
             # interval = 5\n\
             # turbo_enabled = true\n\n\
             # Multiple target keys for diagonal movement (simultaneous press):\n\
             # [[mappings]]\n\
             # trigger_key = \"Q\"\n\
             # target_keys = [\"MOUSE_UP\", \"MOUSE_LEFT\"]  # Move diagonally up-left\n\
             # move_speed = 5\n\
             # interval = 5\n\
             # turbo_enabled = true\n\n\
             # Or use built-in diagonal directions:\n\
             # Diagonal: MOUSE_UP_LEFT, MOUSE_UP_RIGHT, MOUSE_DOWN_LEFT, MOUSE_DOWN_RIGHT\n\n\
             # ─── Mouse Scroll Examples ───\n\
             # [[mappings]]\n\
             # trigger_key = \"PAGEUP\"       # Trigger key\n\
             # target_keys = [\"SCROLL_UP\"]  # Scroll wheel upward\n\
             # move_speed = 120             # Wheel delta (120 = standard notch)\n\
             # interval = 5                 # Repeat interval in ms\n\
             # turbo_enabled = true         # true = continuous, false = Windows repeat\n\n\
             # [[mappings]]\n\
             # trigger_key = \"PAGEDOWN\"\n\
             # target_keys = [\"SCROLL_DOWN\"]\n\
             # move_speed = 240             # Wheel delta (240 = 2x)\n\
             # interval = 5\n\
             # turbo_enabled = true\n\n\
             # ─── HID Device Examples (Gamepads, Joysticks, Custom Controllers) ───\n\
             # Automatic support for any HID device via GUI capture!\n\
             # \n\
             # XInput Controllers (Xbox, compatible gamepads):\n\
             #   Format: GAMEPAD_VID_ButtonName[+ButtonName...] (readable format)\n\
             #   Buttons: A, B, X, Y, Start, Back, LB, RB, LS_Click, RS_Click, LT, RT\n\
             #   D-Pad: DPad_Up, DPad_Down, DPad_Left, DPad_Right, DPad_UpLeft, DPad_UpRight, DPad_DownLeft, DPad_DownRight\n\
             #   Left Stick: LS_Up, LS_Down, LS_Left, LS_Right, LS_LeftUp, LS_LeftDown, LS_RightUp, LS_RightDown\n\
             #   Right Stick: RS_Up, RS_Down, RS_Left, RS_Right, RS_LeftUp, RS_LeftDown, RS_RightUp, RS_RightDown\n\
             #   Combos: Use '+' to combine buttons (e.g., \"LS_RightUp+A+B\")\n\
             #\n\
             # Raw Input Devices (other gamepads, joysticks):\n\
             #   Format: DEVICE_VID_PID_SERIAL_Bx.x (with serial) or DEVICE_VID_PID_DEVxxxxxxxx_Bx.x (without serial)\n\
             #\n\
             # How to configure:\n\
             # 1. Connect your device (XInput controllers work automatically, others require activation)\n\
             # 2. First-time use for non-XInput devices: Activation dialog will appear when you press any button\n\
             #    - Follow instructions: press and release a single button to establish baseline\n\
             #    - Device activation data is saved automatically\n\
             # 3. Open settings dialog, click Capture button for trigger key\n\
             # 4. Press button(s) on your device (supports single button or combo keys)\n\
             # 5. Release all buttons to complete capture\n\
             #\n\
             # XInput Examples:\n\
             # [[mappings]]\n\
             # trigger_key = \"GAMEPAD_045E_A\"              # Xbox controller A button\n\
             # target_keys = [\"SPACE\"]                     # Press space\n\
             # turbo_enabled = true                        # Enable turbo mode\n\n\
             # [[mappings]]\n\
             # trigger_key = \"GAMEPAD_045E_LS_RightUp+A\"   # Left stick right-up + A button\n\
             # target_keys = [\"LCTRL+C\"]                   # Press Ctrl+C\n\
             # turbo_enabled = true                        # Enable turbo mode\n\n\
             # [[mappings]]\n\
             # trigger_key = \"GAMEPAD_045E_DPad_Up+B+X\"    # D-Pad up + B + X buttons\n\
             # target_keys = [\"F\"]                         # Auto-press F\n\
             # turbo_enabled = true                        # Enable turbo mode\n\n\
             # Raw Input Examples:\n\
             # [[mappings]]\n\
             # trigger_key = \"GAMEPAD_045E_0B05_ABC123_B2.0\"  # Raw Input format (with serial)\n\
             # target_keys = [\"SPACE\"]                        # Press space\n\
             # turbo_enabled = true                           # Enable turbo mode\n\n\
             # [[mappings]]\n\
             # trigger_key = \"JOYSTICK_046D_C21D_B1.0\"        # Logitech joystick button\n\
             # target_keys = [\"LBUTTON\"]                      # Left mouse click\n\
             # turbo_enabled = true                           # Enable turbo mode\n\n\
             # ─── HID Device Baselines (Auto-generated, Do Not Edit) ───\n\
             # This section is managed automatically by the application\n\
             # Device activation data for press/release detection\n\
             # Format: VID:PID:Serial or VID:PID (without serial)\n\
             # [[hid_baselines]]\n\
             # device_id = \"045E:028E:1234567\"\n\
             # baseline_data = [0, 255, 127, 255, 127, 0, 128, 0, 0, 0, 0]\n\n",
            self.show_tray_icon,
            self.show_notifications,
            self.always_on_top,
            self.dark_mode,
            match self.language {
                Language::English => "English",
                Language::SimplifiedChinese => "SimplifiedChinese",
                Language::TraditionalChinese => "TraditionalChinese",
                Language::Japanese => "Japanese",
            },
            self.input_timeout,
            self.interval,
            self.event_duration,
            self.worker_count,
            self.xinput_capture_mode,
            self.switch_key,
            self.process_whitelist
        );

        // Pre-allocate capacity for better performance
        let mut result = String::with_capacity(header.len() + self.mappings.len() * 200);
        result.push_str(&header);

        // Append mappings efficiently
        if self.mappings.is_empty() {
            // Write empty array to ensure field exists
            result.push_str("mappings = []\n");
        } else {
            for mapping in &self.mappings {
                result.push_str("[[mappings]]\n");
                result.push_str(&format!(
                    "trigger_key = \"{}\"           # Physical key you press\n",
                    mapping.trigger_key
                ));

                if mapping.target_keys.len() == 1 {
                    result.push_str("target_keys = [\"");
                    result.push_str(&mapping.target_keys[0]);
                    result.push_str("\"]         # Key(s) that get repeatedly sent\n");
                } else if mapping.target_keys.len() > 1 {
                    result.push_str("target_keys = [");
                    for (i, key) in mapping.target_keys.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push('"');
                        result.push_str(key);
                        result.push('"');
                    }
                    result.push_str("]  # Multiple keys for simultaneous press\n");
                } else {
                    result
                        .push_str("target_keys = []             # Keys that get repeatedly sent\n");
                }

                if let Some(interval) = mapping.interval {
                    result.push_str(&format!(
                        "interval = {}                # Override global interval\n",
                        interval
                    ));
                }
                if let Some(duration) = mapping.event_duration {
                    result.push_str(&format!(
                        "event_duration = {}          # Override global press duration\n",
                        duration
                    ));
                }
                result.push_str(&format!(
                    "move_speed = {}              # Speed (pixels for movement, wheel delta for scroll)\n",
                    mapping.move_speed
                ));
                result.push_str(&format!(
                    "turbo_enabled = {}        # Enable turbo mode (true = auto-repeat, false = follow trigger press/release)\n",
                    mapping.turbo_enabled
                ));
                result.push('\n');
            }
        }

        // Append HID device baselines
        if !self.hid_baselines.is_empty() {
            result.push_str("# ─── HID Device Baselines (Auto-managed) ───\n");
            result.push_str("# Device activation data for press/release detection\n");
            result.push_str("# Format: VID:PID:Serial or VID:PID (without serial)\n");
            for baseline in &self.hid_baselines {
                result.push_str("[[hid_baselines]]\n");
                result.push_str(&format!("device_id = \"{}\"\n", baseline.device_id));
                result.push_str(&format!("baseline_data = {:?}\n", baseline.baseline_data));
                result.push('\n');
            }
        }

        // Append device API preferences
        if !self.device_api_preferences.is_empty() {
            result.push_str("# ─── Device API Preferences (Auto-managed) ───\n");
            result.push_str("# Preferred input API for each device (VID:PID format)\n");
            result.push_str("# Options: \"Auto\" (default), \"XInput\", \"RawInput\"\n");
            result.push_str("# Configure via Device Manager GUI window\n");
            result.push_str("[device_api_preferences]\n");
            for (device_key, preference) in &self.device_api_preferences {
                let pref_str = match preference {
                    DeviceApiPreference::Auto => "\"Auto\"",
                    DeviceApiPreference::XInput => "\"XInput\"",
                    DeviceApiPreference::RawInput => "\"RawInput\"",
                };
                result.push_str(&format!("\"{}\" = {}\n", device_key, pref_str));
            }
            result.push('\n');
        }

        fs::write(path, result)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_config_path(name: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("sorahk_test_{}_{}.toml", name, timestamp));
        path
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_default_config_creation() {
        let config = AppConfig::default();

        assert!(config.show_tray_icon);
        assert!(config.show_notifications);
        assert!(!config.always_on_top);
        assert!(!config.dark_mode);
        assert_eq!(config.switch_key, "DELETE");
        assert_eq!(config.input_timeout, 5);
        assert_eq!(config.interval, 5);
        assert_eq!(config.event_duration, 5);
        assert_eq!(config.worker_count, 0);
        assert!(config.process_whitelist.is_empty());
        assert_eq!(config.mappings.len(), 1);
    }

    #[test]
    fn test_config_save_and_load() {
        let path = get_test_config_path("save_and_load");
        cleanup_test_file(&path); // Clean up before test

        let mut config = AppConfig::default();
        config.show_tray_icon = false;
        config.show_notifications = false;
        config.always_on_top = true;
        config.dark_mode = true;
        config.switch_key = "F12".to_string();
        config.input_timeout = 20;
        config.interval = 10;
        config.event_duration = 15;
        config.worker_count = 4;

        config.save_to_file(&path).expect("Failed to save config");

        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.show_tray_icon, config.show_tray_icon);
        assert_eq!(loaded_config.show_notifications, config.show_notifications);
        assert_eq!(loaded_config.always_on_top, config.always_on_top);
        assert_eq!(loaded_config.dark_mode, config.dark_mode);
        assert_eq!(loaded_config.switch_key, config.switch_key);
        assert_eq!(loaded_config.input_timeout, config.input_timeout);
        assert_eq!(loaded_config.interval, config.interval);
        assert_eq!(loaded_config.event_duration, config.event_duration);
        assert_eq!(loaded_config.worker_count, config.worker_count);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_validation_input_timeout() {
        let path = get_test_config_path("validation_timeout");
        cleanup_test_file(&path);

        let content = r#"
            show_tray_icon = true
            show_notifications = true
            switch_key = "DELETE"
            input_timeout = 1
            interval = 5
            event_duration = 5
            worker_count = 0
            process_whitelist = []
            mappings = []
        "#;

        fs::write(&path, content).expect("Failed to write test config");

        let config = AppConfig::load_from_file(&path).expect("Failed to load config");
        assert!(
            config.input_timeout >= 2,
            "Input timeout should be clamped to minimum 2"
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_validation_interval() {
        let path = get_test_config_path("validation_interval");
        cleanup_test_file(&path);

        let content = r#"
            show_tray_icon = true
            show_notifications = true
            switch_key = "DELETE"
            input_timeout = 10
            interval = 2
            event_duration = 5
            worker_count = 0
            process_whitelist = []
            mappings = []
        "#;

        fs::write(&path, content).expect("Failed to write test config");

        let config = AppConfig::load_from_file(&path).expect("Failed to load config");
        assert!(
            config.interval >= 5,
            "Interval should be clamped to minimum 5"
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_validation_event_duration() {
        let path = get_test_config_path("validation_duration");
        cleanup_test_file(&path);

        let content = r#"
            show_tray_icon = true
            show_notifications = true
            switch_key = "DELETE"
            input_timeout = 10
            interval = 5
            event_duration = 2
            worker_count = 0
            process_whitelist = []
            mappings = []
        "#;

        fs::write(&path, content).expect("Failed to write test config");

        let config = AppConfig::load_from_file(&path).expect("Failed to load config");
        assert!(
            config.event_duration >= 2,
            "Event duration should be clamped to minimum 2"
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_load_or_create_missing_file() {
        let path = get_test_config_path("missing_file");
        cleanup_test_file(&path);

        let config = AppConfig::load_or_create(&path).expect("Failed to load or create config");

        assert!(path.exists(), "Config file should be created");
        assert_eq!(config.switch_key, "DELETE");
        assert_eq!(config.interval, 5);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_load_or_create_existing_file() {
        let path = get_test_config_path("existing_file");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.switch_key = "F11".to_string();
        config.save_to_file(&path).expect("Failed to save config");

        let loaded_config = AppConfig::load_or_create(&path).expect("Failed to load config");

        assert_eq!(loaded_config.switch_key, "F11");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_key_mapping_with_overrides() {
        let mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(10),
            event_duration: Some(8),
            turbo_enabled: true,
            move_speed: 10,
        };

        assert_eq!(mapping.trigger_key, "A");
        assert_eq!(mapping.target_keys.as_slice(), &["B".to_string()]);
        assert_eq!(mapping.interval, Some(10));
        assert_eq!(mapping.event_duration, Some(8));
    }

    #[test]
    fn test_key_mapping_without_overrides() {
        let mapping = KeyMapping {
            trigger_key: "C".to_string(),
            target_keys: SmallVec::from_vec(vec!["D".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        assert_eq!(mapping.trigger_key, "C");
        assert_eq!(mapping.target_keys.as_slice(), &["D".to_string()]);
        assert_eq!(mapping.interval, None);
        assert_eq!(mapping.event_duration, None);
    }

    #[test]
    fn test_process_whitelist_serialization() {
        let path = get_test_config_path("whitelist");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.process_whitelist = vec!["notepad.exe".to_string(), "chrome.exe".to_string()];

        config.save_to_file(&path).expect("Failed to save config");

        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.process_whitelist.len(), 2);
        assert!(
            loaded_config
                .process_whitelist
                .contains(&"notepad.exe".to_string())
        );
        assert!(
            loaded_config
                .process_whitelist
                .contains(&"chrome.exe".to_string())
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_language_serialization() {
        let languages = vec![
            Language::English,
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::Japanese,
        ];

        for (idx, lang) in languages.iter().enumerate() {
            let path = get_test_config_path(&format!("language_{}", idx));
            cleanup_test_file(&path);

            let mut config = AppConfig::default();
            config.language = *lang;

            config.save_to_file(&path).expect("Failed to save config");
            let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

            assert_eq!(loaded_config.language, *lang);

            cleanup_test_file(&path);
        }
    }

    #[test]
    fn test_multiple_mappings_serialization() {
        let path = get_test_config_path("multiple_mappings");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.mappings = vec![
            KeyMapping {
                trigger_key: "A".to_string(),
                target_keys: SmallVec::from_vec(vec!["1".to_string()]),
                interval: Some(10),
                event_duration: Some(5),
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "B".to_string(),
                target_keys: SmallVec::from_vec(vec!["2".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "F1".to_string(),
                target_keys: SmallVec::from_vec(vec!["SPACE".to_string()]),
                interval: Some(20),
                event_duration: Some(10),
                turbo_enabled: true,
                move_speed: 10,
            },
        ];

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.mappings.len(), 3);
        assert_eq!(loaded_config.mappings[0].trigger_key, "A");
        assert_eq!(
            loaded_config.mappings[1].target_keys.as_slice(),
            &["2".to_string()]
        );
        assert_eq!(loaded_config.mappings[2].interval, Some(20));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let path = get_test_config_path("invalid_toml");
        cleanup_test_file(&path);

        // Write invalid TOML
        std::fs::write(&path, "invalid [[ toml \n syntax").expect("Failed to write");

        let result = AppConfig::load_from_file(&path);
        assert!(result.is_err());

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let path = get_test_config_path("nonexistent");

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&path);

        let result = AppConfig::load_from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_extreme_values() {
        let path = get_test_config_path("extreme_values");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.interval = 1000; // Very large interval
        config.event_duration = 500;
        config.input_timeout = 10000;
        config.worker_count = 64; // Large worker count

        config.save_to_file(&path).expect("Failed to save");
        let loaded = AppConfig::load_from_file(&path).expect("Failed to load");

        assert_eq!(loaded.interval, 1000);
        assert_eq!(loaded.event_duration, 500);
        assert_eq!(loaded.input_timeout, 10000);
        assert_eq!(loaded.worker_count, 64);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_with_special_characters_in_process_name() {
        let path = get_test_config_path("special_chars");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.process_whitelist = vec![
            "app-name.exe".to_string(),
            "app_name.exe".to_string(),
            "app123.exe".to_string(),
        ];

        config.save_to_file(&path).expect("Failed to save");
        let loaded = AppConfig::load_from_file(&path).expect("Failed to load");

        assert_eq!(loaded.process_whitelist.len(), 3);
        assert!(
            loaded
                .process_whitelist
                .contains(&"app-name.exe".to_string())
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_save_to_readonly_path() {
        // This test verifies error handling for readonly paths
        // On Windows, we can't easily create readonly directories in tests
        // so we test with an invalid path
        let path = PathBuf::from("/nonexistent/invalid/path/config.toml");

        let config = AppConfig::default();
        let result = config.save_to_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn test_config_language_default() {
        let config = AppConfig::default();
        assert_eq!(config.language, Language::default());
    }

    #[test]
    fn test_config_with_duplicate_process_names() {
        let path = get_test_config_path("duplicate_processes");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.process_whitelist = vec![
            "app.exe".to_string(),
            "app.exe".to_string(), // Duplicate
            "other.exe".to_string(),
        ];

        config.save_to_file(&path).expect("Failed to save");
        let loaded = AppConfig::load_from_file(&path).expect("Failed to load");

        assert_eq!(loaded.process_whitelist.len(), 2);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_validation_clamps_negative_values() {
        let path = get_test_config_path("negative_values");
        cleanup_test_file(&path);

        // Manually write config with "negative" (actually minimum) values
        let content = r#"
            show_tray_icon = true
            show_notifications = true
            switch_key = "DELETE"
            input_timeout = 1
            interval = 1
            event_duration = 1
            worker_count = 0
            process_whitelist = []
            mappings = []
        "#;

        std::fs::write(&path, content).expect("Failed to write");
        let loaded = AppConfig::load_from_file(&path).expect("Failed to load");

        // Values should be clamped to minimums
        assert!(loaded.input_timeout >= 2);
        assert!(loaded.interval >= 5);
        assert!(loaded.event_duration >= 2);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_config_deduplicates_process_whitelist() {
        // Test that duplicate processes are automatically removed when loading config
        let path = get_test_config_path("process_dedup");
        cleanup_test_file(&path);

        let content = r#"
            show_tray_icon = true
            show_notifications = true
            switch_key = "DELETE"
            input_timeout = 10
            interval = 5
            event_duration = 5
            worker_count = 0
            process_whitelist = ["chrome.exe", "notepad.exe", "chrome.exe", "firefox.exe", "notepad.exe"]
            mappings = []
        "#;

        std::fs::write(&path, content).expect("Failed to write test config");
        let loaded = AppConfig::load_from_file(&path).expect("Failed to load config");

        // Should have exactly 3 unique processes after deduplication
        assert_eq!(loaded.process_whitelist.len(), 3);
        assert!(loaded.process_whitelist.contains(&"chrome.exe".to_string()));
        assert!(
            loaded
                .process_whitelist
                .contains(&"notepad.exe".to_string())
        );
        assert!(
            loaded
                .process_whitelist
                .contains(&"firefox.exe".to_string())
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_multiple_target_keys_single() {
        let mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        assert_eq!(mapping.target_keys.len(), 1);
        assert_eq!(mapping.target_keys_display(), "B");
    }

    #[test]
    fn test_multiple_target_keys_multiple() {
        let mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["MOUSE_UP".to_string(), "MOUSE_LEFT".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        assert_eq!(mapping.target_keys.len(), 2);
        assert_eq!(mapping.target_keys_display(), "MOUSE_UP, MOUSE_LEFT");
    }

    #[test]
    fn test_multiple_target_keys_empty() {
        let mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::new(),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        assert_eq!(mapping.target_keys.len(), 0);
        assert_eq!(mapping.target_keys_display(), "");
    }

    #[test]
    fn test_add_target_key() {
        let mut mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        mapping.add_target_key("C".to_string());
        assert_eq!(mapping.target_keys.len(), 2);
        assert_eq!(mapping.target_keys[1], "C");

        // Adding duplicate should not increase count
        mapping.add_target_key("B".to_string());
        assert_eq!(mapping.target_keys.len(), 2);
    }

    #[test]
    fn test_remove_target_key() {
        let mut mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        mapping.remove_target_key("C");
        assert_eq!(mapping.target_keys.len(), 2);
        assert_eq!(mapping.target_keys[0], "B");
        assert_eq!(mapping.target_keys[1], "D");
    }

    #[test]
    fn test_clear_target_keys() {
        let mut mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string(), "C".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        mapping.clear_target_keys();
        assert_eq!(mapping.target_keys.len(), 0);
    }

    #[test]
    fn test_multiple_target_keys_serialization() {
        let path = get_test_config_path("multi_target_keys");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.mappings = vec![
            KeyMapping {
                trigger_key: "Q".to_string(),
                target_keys: SmallVec::from_vec(vec![
                    "MOUSE_UP".to_string(),
                    "MOUSE_LEFT".to_string(),
                ]),
                interval: Some(5),
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "E".to_string(),
                target_keys: SmallVec::from_vec(vec![
                    "MOUSE_UP".to_string(),
                    "MOUSE_RIGHT".to_string(),
                ]),
                interval: Some(5),
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            },
        ];

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.mappings.len(), 2);
        assert_eq!(loaded_config.mappings[0].target_keys.len(), 2);
        assert_eq!(loaded_config.mappings[0].target_keys[0], "MOUSE_UP");
        assert_eq!(loaded_config.mappings[0].target_keys[1], "MOUSE_LEFT");
        assert_eq!(loaded_config.mappings[1].target_keys.len(), 2);
        assert_eq!(loaded_config.mappings[1].target_keys[0], "MOUSE_UP");
        assert_eq!(loaded_config.mappings[1].target_keys[1], "MOUSE_RIGHT");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_smallvec_inline_capacity() {
        // Test that SmallVec uses inline storage for small collections
        let small_vec: SmallVec<[String; 4]> =
            SmallVec::from_vec(vec!["A".to_string(), "B".to_string(), "C".to_string()]);

        // Should be stored inline (capacity <= 4)
        assert_eq!(small_vec.len(), 3);
        assert!(small_vec.spilled() == false); // Not heap allocated

        let mut large_vec: SmallVec<[String; 4]> = SmallVec::new();
        for i in 0..6 {
            large_vec.push(format!("KEY_{}", i));
        }

        // Should spill to heap (capacity > 4)
        assert_eq!(large_vec.len(), 6);
        assert!(large_vec.spilled()); // Heap allocated
    }

    #[test]
    fn test_get_target_keys() {
        let mapping = KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string(), "C".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        };

        let keys = mapping.get_target_keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "B");
        assert_eq!(keys[1], "C");
    }

    #[test]
    fn test_empty_target_keys_serialization() {
        let path = get_test_config_path("empty_target_keys");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::new(),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        }];

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.mappings.len(), 1);
        assert_eq!(loaded_config.mappings[0].target_keys.len(), 0);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_many_target_keys_serialization() {
        let path = get_test_config_path("many_target_keys");
        cleanup_test_file(&path);

        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
            ]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        }];

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.mappings.len(), 1);
        assert_eq!(loaded_config.mappings[0].target_keys.len(), 6);
        assert_eq!(loaded_config.mappings[0].target_keys[5], "6");

        cleanup_test_file(&path);
    }
}
