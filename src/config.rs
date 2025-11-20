//! Application configuration management.
//!
//! Handles loading, saving, and validation of application settings
//! including key mappings and runtime parameters.

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::i18n::Language;

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
        if config.event_duration < 5 {
            config.event_duration = 5;
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
             # ─── Control Settings ───   \n\
             switch_key = \"{}\"       # Reserved key to toggle SoraHK behavior\n\n\
             # ─── Process Whitelist ───\n\
             # Process whitelist (empty = all processes enabled)\n\
             # Only processes in this list will have turbo-fire enabled\n\
             process_whitelist = {:?}      # Example: [\"notepad.exe\", \"game.exe\"]\n\n\
             # ─── Key Mappings ───\n\
             # Input mapping definitions (supports both keyboard and mouse)\n\
             # Supported mouse buttons: LBUTTON, RBUTTON, MBUTTON, XBUTTON1, XBUTTON2\n\n\
             # ─── Mouse Button Examples ───\n\
             # Uncomment to enable mouse button mappings:\n\
             # [[mappings]]\n\
             # trigger_key = \"LBUTTON\"     # Left mouse button trigger\n\
             # target_key = \"LBUTTON\"      # Auto-click left button\n\n\
             # [[mappings]]\n\
             # trigger_key = \"RBUTTON\"     # Right mouse button trigger\n\
             # target_key = \"SPACE\"        # Press space when right-clicking\n\n\
             # [[mappings]]\n\
             # trigger_key = \"XBUTTON1\"    # Side button 1 trigger\n\
             # target_key = \"F\"            # Auto-press F key\n\n",
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
                    "trigger_key = \"{}\"           # Physical key you press\n\
                     target_key = \"{}\"            # Key that gets repeatedly sent\n",
                    mapping.trigger_key, mapping.target_key
                ));
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
                result.push('\n');
            }
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
        assert_eq!(config.input_timeout, 10);
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
            config.event_duration >= 5,
            "Event duration should be clamped to minimum 5"
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
            target_key: "B".to_string(),
            interval: Some(10),
            event_duration: Some(8),
        };

        assert_eq!(mapping.trigger_key, "A");
        assert_eq!(mapping.target_key, "B");
        assert_eq!(mapping.interval, Some(10));
        assert_eq!(mapping.event_duration, Some(8));
    }

    #[test]
    fn test_key_mapping_without_overrides() {
        let mapping = KeyMapping {
            trigger_key: "C".to_string(),
            target_key: "D".to_string(),
            interval: None,
            event_duration: None,
        };

        assert_eq!(mapping.trigger_key, "C");
        assert_eq!(mapping.target_key, "D");
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
                target_key: "1".to_string(),
                interval: Some(10),
                event_duration: Some(5),
            },
            KeyMapping {
                trigger_key: "B".to_string(),
                target_key: "2".to_string(),
                interval: None,
                event_duration: None,
            },
            KeyMapping {
                trigger_key: "F1".to_string(),
                target_key: "SPACE".to_string(),
                interval: Some(20),
                event_duration: Some(10),
            },
        ];

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.mappings.len(), 3);
        assert_eq!(loaded_config.mappings[0].trigger_key, "A");
        assert_eq!(loaded_config.mappings[1].target_key, "2");
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
        assert!(loaded.event_duration >= 5);

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
}
