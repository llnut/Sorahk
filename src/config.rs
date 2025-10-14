use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub show_tray_icon: bool,
    pub show_notifications: bool,
    pub switch_key: String,
    pub mappings: Vec<KeyMapping>,
    #[serde(default = "default_input_timeout")]
    pub input_timeout: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_event_duration")]
    pub event_duration: u64,
}

#[derive(Debug, Clone, Deserialize)]
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

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        // 验证配置
        if config.input_timeout < 2 {
            println!("Warning: the input timeout must be at least 2ms");
            config.input_timeout = 2;
        }
        if config.interval < 5 {
            println!("Warning: the default interval must be at least 5ms");
            config.interval = 5;
        }
        if config.event_duration < 5 {
            println!("Warning: the duration of the key press event must be at least 5ms");
            config.event_duration = 5;
        }

        Ok(config)
    }
}
