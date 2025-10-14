use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, LazyLock, OnceLock};

use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::AppConfig;

pub const SIMULATED_EVENT_MARKER: usize = 0x4659;

static GLOBAL_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum NotificationEvent {
    Info(String),
    Warning(String),
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Pressed(u32),
    Released(u32),
}

pub struct KeyMappingInfo {
    pub target_scancode: u16,
    pub interval: u64,
    pub event_duration: u64,
}

pub struct AppState {
    pub show_tray_icon: bool,
    pub show_notifications: bool,
    switch_key: u32,
    pub should_exit: Arc<AtomicBool>,
    is_paused: AtomicBool,
    input_timeout: u64,
    key_mappings: HashMap<u32, KeyMappingInfo>,
    event_sender: OnceLock<Sender<KeyEvent>>,
    notification_sender: OnceLock<Sender<NotificationEvent>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let switch_key = Self::key_name_to_vk(&config.switch_key)
            .ok_or_else(|| anyhow::anyhow!("Invalid switch key: {}", config.switch_key))?;

        let key_mappings = Self::create_key_mappings(&config)?;

        println!("Loaded {} key mappings", key_mappings.len());

        Ok(Self {
            show_tray_icon: config.show_tray_icon,
            show_notifications: config.show_notifications,
            switch_key,
            should_exit: Arc::new(AtomicBool::new(false)),
            is_paused: AtomicBool::new(false),
            input_timeout: config.input_timeout,
            key_mappings,
            event_sender: OnceLock::new(),
            notification_sender: OnceLock::new(),
        })
    }

    pub fn set_event_sender(&self, sender: Sender<KeyEvent>) {
        let _ = self.event_sender.set(sender);
    }

    pub fn set_notification_sender(&self, sender: Sender<NotificationEvent>) {
        let _ = self.notification_sender.set(sender);
    }

    pub fn get_notification_sender(&self) -> Option<&Sender<NotificationEvent>> {
        self.notification_sender.get()
    }

    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    /// toggle ahk status, return old statue
    pub fn toggle_paused(&self) -> bool {
        self.is_paused.fetch_xor(true, Ordering::Relaxed)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    pub fn input_timeout(&self) -> u64 {
        self.input_timeout
    }

    pub fn get_key_mapping(&self, trigger_key: &u32) -> Option<&KeyMappingInfo> {
        self.key_mappings.get(trigger_key)
    }

    pub fn simulate_key_press(&self, scancode: u16, duration: u64) {
        unsafe {
            // press the key
            let mut input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: scancode,
                        dwFlags: KEYEVENTF_SCANCODE,
                        time: 0,
                        dwExtraInfo: SIMULATED_EVENT_MARKER,
                    },
                },
            };

            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            std::thread::sleep(std::time::Duration::from_millis(duration));

            // release the key
            input.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    #[allow(non_snake_case)]
    pub fn handle_key_event(&self, message: u32, vk_code: u32) -> bool {
        let mut should_block = false;

        if !self.is_paused()
            && self.key_mappings.contains_key(&vk_code)
            && let Some(sender) = self.event_sender.get()
        {
            match message {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    let _ = sender.send(KeyEvent::Pressed(vk_code));
                    should_block = true;
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    let _ = sender.send(KeyEvent::Released(vk_code));
                    should_block = true;
                }
                _ => {}
            }
        }

        if vk_code == self.switch_key {
            match message {
                WM_KEYUP | WM_SYSKEYUP => {
                    let was_paused = self.toggle_paused();
                    if was_paused {
                        if let Some(sender) = self.notification_sender.get() {
                            let _ = sender
                                .send(NotificationEvent::Info("Sorahk activiting".to_string()));
                        }
                        println!("Sorahk activiting");
                    } else {
                        if let Some(sender) = self.notification_sender.get() {
                            let _ =
                                sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
                        }
                        println!("Sorahk paused");
                    }
                    should_block = true;
                }
                _ => {}
            }
        }
        should_block
    }

    fn create_key_mappings(config: &AppConfig) -> anyhow::Result<HashMap<u32, KeyMappingInfo>> {
        let mut mappings = HashMap::new();

        for mapping in &config.mappings {
            let trigger_vk = Self::key_name_to_vk(&mapping.trigger_key)
                .ok_or_else(|| anyhow::anyhow!("Invalid trigger key: {}", mapping.trigger_key))?;

            let target_vk = Self::key_name_to_vk(&mapping.target_key)
                .ok_or_else(|| anyhow::anyhow!("Invalid target key: {}", mapping.target_key))?;

            let target_scancode = Self::vk_to_scancode(target_vk);
            if target_scancode == 0 {
                anyhow::bail!("Failed to get key {}'s scancode", mapping.target_key);
            }

            let interval = mapping.interval.unwrap_or(config.interval).max(5);
            let event_duration = mapping
                .event_duration
                .unwrap_or(config.event_duration)
                .max(5);

            mappings.insert(
                trigger_vk,
                KeyMappingInfo {
                    target_scancode,
                    interval,
                    event_duration,
                },
            );

            println!(
                "Mapping: {} (VK:{:02X}) -> {} (SC:{:02X}), interval:{}ms, duration:{}ms",
                mapping.trigger_key,
                trigger_vk,
                mapping.target_key,
                target_scancode,
                interval,
                event_duration
            );
        }

        Ok(mappings)
    }

    fn key_name_to_vk(key_name: &str) -> Option<u32> {
        let key = key_name.to_uppercase();

        // letter keys
        if key.len() == 1
            && let Some(c) = key.chars().next()
            && (c.is_ascii_alphabetic() || c.is_ascii_digit())
        {
            return Some(c as u32);
        }

        // number keys
        if key.len() == 1
            && let Some(c) = key.chars().next()
            && c.is_ascii_digit()
        {
            return Some(c as u32);
        }

        // F1-F24
        if key.starts_with('F')
            && key.len() > 1
            && let Ok(num) = key[1..].parse::<u32>()
            && (1..=24).contains(&num)
        {
            return Some(0x70 + num - 1);
        }

        // special keys
        match key.as_str() {
            "ESC" => Some(0x1B),
            "ENTER" => Some(0x0D),
            "TAB" => Some(0x09),
            "CLEAR" => Some(0x0C),
            "SHIFT" => Some(0x10),
            "CTRL" => Some(0x11),
            "ALT" => Some(0x12),
            "PAUSE" => Some(0x13),
            "CAPSLOCK" => Some(0x14),
            "SPACE" => Some(0x20),
            "BACKSPACE" => Some(0x08),
            "DELETE" => Some(0x2E),
            "INSERT" => Some(0x2D),
            "HOME" => Some(0x24),
            "END" => Some(0x23),
            "PAGEUP" => Some(0x21),
            "PAGEDOWN" => Some(0x22),
            "UP" => Some(0x26),
            "DOWN" => Some(0x28),
            "LEFT" => Some(0x25),
            "RIGHT" => Some(0x27),
            "LSHIFT" => Some(0xA0),
            "RSHIFT" => Some(0xA1),
            "LCTRL" => Some(0xA2),
            "RCTRL" => Some(0xA3),
            "LALT" => Some(0xA4),
            "RALT" => Some(0xA5),
            "LWIN" => Some(0x5B),
            "RWIN" => Some(0x5C),
            _ => None,
        }
    }

    fn vk_to_scancode(vk_code: u32) -> u16 {
        SCANCODE_MAP.get(&vk_code).copied().unwrap_or(0)
    }
}

pub fn set_global_state(state: Arc<AppState>) -> Result<(), Arc<AppState>> {
    GLOBAL_STATE.set(state)
}

pub fn get_global_state() -> Option<&'static Arc<AppState>> {
    GLOBAL_STATE.get()
}

static SCANCODE_MAP: LazyLock<HashMap<u32, u16>> = LazyLock::new(|| {
    [
        // letter keys (A-Z)
        (0x41, 0x1E),
        (0x42, 0x30),
        (0x43, 0x2E),
        (0x44, 0x20),
        (0x45, 0x12),
        (0x46, 0x21),
        (0x47, 0x22),
        (0x48, 0x23),
        (0x49, 0x17),
        (0x4A, 0x24),
        (0x4B, 0x25),
        (0x4C, 0x26),
        (0x4D, 0x32),
        (0x4E, 0x31),
        (0x4F, 0x18),
        (0x50, 0x19),
        (0x51, 0x10),
        (0x52, 0x13),
        (0x53, 0x1F),
        (0x54, 0x14),
        (0x55, 0x16),
        (0x56, 0x2F),
        (0x57, 0x11),
        (0x58, 0x2D),
        (0x59, 0x15),
        (0x5A, 0x2C),
        // number keys (0-9)
        (0x30, 0x0B),
        (0x31, 0x02),
        (0x32, 0x03),
        (0x33, 0x04),
        (0x34, 0x05),
        (0x35, 0x06),
        (0x36, 0x07),
        (0x37, 0x08),
        (0x38, 0x09),
        (0x39, 0x0A),
        // function keys (F1-F12)
        (0x70, 0x3B),
        (0x71, 0x3C),
        (0x72, 0x3D),
        (0x73, 0x3E),
        (0x74, 0x3F),
        (0x75, 0x40),
        (0x76, 0x41),
        (0x77, 0x42),
        (0x78, 0x43),
        (0x79, 0x44),
        (0x7A, 0x57),
        (0x7B, 0x58),
        // special keys
        (0x1B, 0x01), // ESC
        (0x0D, 0x1C), // ENTER
        (0x09, 0x0F), // TAB
        (0x20, 0x39), // SPACE
        (0x08, 0x0E), // BACKSPACE
        (0x2E, 0x53), // DELETE
        (0x2D, 0x52), // INSERT
        (0x24, 0x47), // HOME
        (0x23, 0x4F), // END
        (0x21, 0x49), // PAGEUP
        (0x22, 0x51), // PAGEDOWN
        (0x26, 0x48), // UP
        (0x28, 0x50), // DOWN
        (0x25, 0x4B), // LEFT
        (0x27, 0x4D), // RIGHT
        // keypad
        (0x60, 0x52),
        (0x61, 0x4F),
        (0x62, 0x50),
        (0x63, 0x51),
        (0x64, 0x4B),
        (0x65, 0x4C),
        (0x66, 0x4D),
        (0x67, 0x47),
        (0x68, 0x48),
        (0x69, 0x49),
        (0x6A, 0x37),
        (0x6B, 0x4E),
        (0x6C, 0x53),
        (0x6D, 0x4A),
        (0x6E, 0x52),
        (0x6F, 0x53),
    ]
    .iter()
    .cloned()
    .collect()
});
