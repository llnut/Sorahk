//! Application configuration management.
//!
//! Handles loading, saving, and validation of application settings
//! including key mappings and runtime parameters.

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

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
}

/// Key mapping configuration for trigger-target pairs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMapping {
    /// Trigger key name
    pub trigger_key: String,
    /// Target key name to send
    pub target_key: String,
    /// Optional override for repeat interval
    #[serde(default)]
    pub interval: Option<u64>,
    /// Optional override for press duration
    #[serde(default)]
    pub event_duration: Option<u64>,
}

fn default_input_timeout() -> u64 {
    10
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

impl AppConfig {
    /// Creates a default configuration with sensible defaults.
    pub fn default() -> Self {
        Self {
            show_tray_icon: true,
            show_notifications: true,
            always_on_top: false, // Default: not always on top for backward compatibility
            dark_mode: false,     // Default: light theme for backward compatibility
            switch_key: "DELETE".to_string(),
            mappings: vec![KeyMapping {
                trigger_key: "Q".to_string(),
                target_key: "Q".to_string(),
                interval: None,
                event_duration: None,
            }],
            input_timeout: default_input_timeout(),
            interval: default_interval(),
            event_duration: default_event_duration(),
            worker_count: default_worker_count(),
            process_whitelist: vec![], // Empty means all processes enabled
        }
    }

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
        if config.event_duration < 5 {
            config.event_duration = 5;
        }

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
             dark_mode = {}           # Use dark theme (false = light theme, true = dark theme)\n\n\
             # ─── Performance Settings ───\n\
             input_timeout = {}          # Input timeout in ms\n\
             interval = {}                # Default repeat interval between keystrokes (ms)\n\
             event_duration = {}          # Duration of each simulated key press (ms)\n\
             worker_count = {}            # Number of turbo workers (0 = auto-detect based on CPU cores)\n\n\
             # ─── Control Settings ───   \n\
             switch_key = \"{}\"       # Reserved key to toggle SoraHK behavior\n\n\
             # ─── Process Whitelist ───\n\
             # Process whitelist (empty = all processes enabled)\n\
             # Only processes in this list will have turbo-fire enabled\n\
             process_whitelist = {:?}      # Example: [\"notepad.exe\", \"game.exe\"]\n\n\
             # ─── Key Mappings ───\n\
             # Key mapping definitions\n",
            self.show_tray_icon,
            self.show_notifications,
            self.always_on_top,
            self.dark_mode,
            self.input_timeout,
            self.interval,
            self.event_duration,
            self.worker_count,
            self.switch_key,
            self.process_whitelist
        );

        // Pre-allocate capacity for better performance
        let mut result = String::with_capacity(header.len() + self.mappings.len() * 200);
        result.push_str(&header);

        // Append mappings efficiently
        for mapping in &self.mappings {
            result.push_str("[[mappings]]\n");
            result.push_str(&format!(
                "trigger_key = \"{}\"           # Physical key you press\n\
                 target_key = \"{}\"            # Key that gets repeatedly sent\n",
                mapping.trigger_key, mapping.target_key
            ));
            if let Some(interval) = mapping.interval {
                result.push_str(&format!(
                    "interval = {}                  # Override global interval\n",
                    interval
                ));
            }
            if let Some(duration) = mapping.event_duration {
                result.push_str(&format!(
                    "event_duration = {}                  # Override global press duration\n",
                    duration
                ));
            }
            result.push('\n');
        }

        fs::write(path, result)?;
        Ok(())
    }
}
