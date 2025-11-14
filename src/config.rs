use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub show_tray_icon: bool,
    pub show_notifications: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)] // Default to false (light theme) for backward compatibility
    pub dark_mode: bool,
    pub switch_key: String,
    pub mappings: Vec<KeyMapping>,
    #[serde(default = "default_input_timeout")]
    pub input_timeout: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_event_duration")]
    pub event_duration: u64,
    #[serde(default = "default_worker_count")]
    pub worker_count: usize,
    #[serde(default)]
    pub process_whitelist: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMapping {
    pub trigger_key: String,
    pub target_key: String,
    #[serde(default)]
    pub interval: Option<u64>,
    #[serde(default)]
    pub event_duration: Option<u64>,
}

fn default_input_timeout() -> u64 {
    100
}
fn default_interval() -> u64 {
    50
}
fn default_event_duration() -> u64 {
    10
}
fn default_worker_count() -> usize {
    0 // 0 means auto-detect based on CPU cores
}

impl AppConfig {
    /// Create a default configuration
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

    /// Load config from file, or create default if not exists
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            let default_config = Self::default();
            default_config.save_to_file(&path)?;
            return Ok(default_config);
        }
        Self::load_from_file(path)
    }

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

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        // Add comments to make the config file more readable
        let commented = format!(
            "show_tray_icon = {}        # Show system tray icon on startup\n\
             show_notifications = {}   # Enable/disable system notifications\n\
             always_on_top = {}          # Keep window always on top of other windows\n\
             dark_mode = {}               # Use dark theme (false = light theme, true = dark theme)\n\
             input_timeout = {}           # Input timeout in ms\n\
             interval = {}                 # Default repeat interval between keystrokes (ms)\n\
             event_duration = {}           # Duration of each simulated key press (ms)\n\
             worker_count = {}             # Number of turbo workers (0 = auto-detect based on CPU cores)\n\
             switch_key = \"{}\"        # Reserved key to toggle SoraHK behavior\n\n\
             # Process whitelist (empty = all processes enabled)\n\
             # Only processes in this list will have turbo-fire enabled\n\
             process_whitelist = {:?}     # Example: [\"notepad.exe\", \"game.exe\"]\n\n\
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

        let mut result = commented;
        for mapping in &self.mappings {
            result.push_str("[[mappings]]\n");
            result.push_str(&format!(
                "trigger_key = \"{}\"            # Physical key you press\n",
                mapping.trigger_key
            ));
            result.push_str(&format!(
                "target_key = \"{}\"             # Key that gets repeatedly sent\n",
                mapping.target_key
            ));
            if let Some(interval) = mapping.interval {
                result.push_str(&format!(
                    "interval = {}                 # Override global interval\n",
                    interval
                ));
            }
            if let Some(duration) = mapping.event_duration {
                result.push_str(&format!(
                    "event_duration = {}           # Override global press duration\n",
                    duration
                ));
            }
            result.push('\n');
        }

        fs::write(path, result)?;
        Ok(())
    }
}
