//! Application state management.
//!
//! Provides centralized state management for the key remapping application,
//! including configuration, keyboard event handling, and process filtering.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PWSTR;

use crate::config::AppConfig;

/// Trait for dispatching keyboard events to worker threads.
pub trait EventDispatcher: Send + Sync {
    fn dispatch(&self, event: KeyEvent);
}

/// Marker value to identify simulated keyboard events.
pub const SIMULATED_EVENT_MARKER: usize = 0x4659;

static GLOBAL_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

/// Notification event types for user feedback.
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum NotificationEvent {
    Info(String),
    Warning(String),
    Error(String),
}

/// Keyboard event type with associated virtual key code.
#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Pressed(u32),
    Released(u32),
}

/// Configuration for a single key mapping.
#[derive(Debug, Clone, Copy)]
pub struct KeyMappingInfo {
    /// Target key scancode
    pub target_scancode: u16,
    /// Repeat interval in milliseconds
    pub interval: u64,
    /// Key press duration in milliseconds
    pub event_duration: u64,
}

/// Central application state manager.
///
/// Manages all runtime state including configuration, key mappings,
/// worker threads, and process filtering.
pub struct AppState {
    /// Tray icon visibility flag
    show_tray_icon: AtomicBool,
    /// Notification display flag
    show_notifications: AtomicBool,
    /// Toggle hotkey virtual key code
    switch_key: RwLock<u32>,
    /// Application exit flag
    pub should_exit: Arc<AtomicBool>,
    /// Key repeat pause state
    is_paused: AtomicBool,
    /// Window show request flag
    show_window_requested: AtomicBool,
    /// About dialog request flag
    show_about_requested: AtomicBool,
    /// Input timeout in milliseconds
    input_timeout: AtomicU64,
    /// Active worker thread count
    worker_count: AtomicU64,
    /// Configured worker count for display
    configured_worker_count: usize,
    /// Key mapping configuration
    key_mappings: RwLock<HashMap<u32, KeyMappingInfo>>,
    /// Pre-computed VK to worker index mapping for fast dispatch
    vk_to_worker: [u8; 256],
    /// Worker pool for event processing
    worker_pool: OnceLock<Arc<dyn EventDispatcher>>,
    /// Notification event sender
    notification_sender: OnceLock<Sender<NotificationEvent>>,
    /// Process whitelist (empty means all processes enabled)
    process_whitelist: RwLock<Vec<String>>,
}

impl AppState {
    /// Creates a new application state from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the toggle key name is invalid or key mappings cannot be created.
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let switch_key = Self::key_name_to_vk(&config.switch_key)
            .ok_or_else(|| anyhow::anyhow!("Invalid switch key: {}", config.switch_key))?;

        let key_mappings = Self::create_key_mappings(&config)?;

        // Pre-compute vk_to_worker mapping array for fast dispatch
        // Use mapping index for balanced distribution
        let mut vk_to_worker = [0u8; 256];
        for (idx, mapping) in config.mappings.iter().enumerate() {
            if let Some(trigger_vk) = Self::key_name_to_vk(&mapping.trigger_key)
                && trigger_vk < 256
            {
                // Store mapping index directly (will be modulo'd with worker_count in dispatch)
                // Using u8 to save memory and improve cache efficiency
                vk_to_worker[trigger_vk as usize] = idx as u8;
            }
        }

        Ok(Self {
            show_tray_icon: AtomicBool::new(config.show_tray_icon),
            show_notifications: AtomicBool::new(config.show_notifications),
            switch_key: RwLock::new(switch_key),
            should_exit: Arc::new(AtomicBool::new(false)),
            is_paused: AtomicBool::new(false),
            show_window_requested: AtomicBool::new(false),
            show_about_requested: AtomicBool::new(false),
            input_timeout: AtomicU64::new(config.input_timeout),
            worker_count: AtomicU64::new(0), // Will be set later
            process_whitelist: RwLock::new(config.process_whitelist.clone()),
            configured_worker_count: config.worker_count,
            key_mappings: RwLock::new(key_mappings),
            vk_to_worker,
            worker_pool: OnceLock::new(),
            notification_sender: OnceLock::new(),
        })
    }

    /// Reloads configuration at runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the new toggle key is invalid or mappings cannot be created.
    pub fn reload_config(&self, config: AppConfig) -> anyhow::Result<()> {
        // Update switch key
        let new_switch_key = Self::key_name_to_vk(&config.switch_key)
            .ok_or_else(|| anyhow::anyhow!("Invalid switch key: {}", config.switch_key))?;

        if let Ok(mut switch_key) = self.switch_key.write() {
            *switch_key = new_switch_key;
        }

        // Update show_tray_icon and show_notifications
        self.show_tray_icon
            .store(config.show_tray_icon, Ordering::Relaxed);
        self.show_notifications
            .store(config.show_notifications, Ordering::Relaxed);

        // Update input timeout
        self.input_timeout
            .store(config.input_timeout, Ordering::Relaxed);

        // Update key mappings
        let new_mappings = Self::create_key_mappings(&config)?;
        if let Ok(mut mappings) = self.key_mappings.write() {
            *mappings = new_mappings;
        }

        // Update process whitelist
        if let Ok(mut whitelist) = self.process_whitelist.write() {
            *whitelist = config.process_whitelist.clone();
        }

        Ok(())
    }

    /// Sets the worker pool for event dispatching.
    pub fn set_worker_pool(&self, pool: Arc<dyn EventDispatcher>) {
        let _ = self.worker_pool.set(pool);
    }

    /// Sets the notification event sender.
    pub fn set_notification_sender(&self, sender: Sender<NotificationEvent>) {
        let _ = self.notification_sender.set(sender);
    }

    /// Returns the notification sender if available.
    pub fn get_notification_sender(&self) -> Option<&Sender<NotificationEvent>> {
        self.notification_sender.get()
    }

    /// Signals the application to exit.
    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
    }

    /// Checks if the application should exit.
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    /// Toggles pause state and returns the previous state.
    pub fn toggle_paused(&self) -> bool {
        self.is_paused.fetch_xor(true, Ordering::Relaxed)
    }

    /// Returns the current pause state.
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    /// Sets the pause state.
    pub fn set_paused(&self, paused: bool) {
        self.is_paused.store(paused, Ordering::Relaxed);
    }

    /// Returns whether the tray icon should be shown.
    pub fn show_tray_icon(&self) -> bool {
        self.show_tray_icon.load(Ordering::Relaxed)
    }

    /// Returns whether notifications should be displayed.
    pub fn show_notifications(&self) -> bool {
        self.show_notifications.load(Ordering::Relaxed)
    }

    /// Requests the main window to be shown.
    pub fn request_show_window(&self) {
        self.show_window_requested.store(true, Ordering::Relaxed);
    }

    /// Checks and clears the show window request flag.
    pub fn check_and_clear_show_window_request(&self) -> bool {
        self.show_window_requested.swap(false, Ordering::Relaxed)
    }

    /// Requests the about dialog to be shown.
    pub fn request_show_about(&self) {
        self.show_about_requested.store(true, Ordering::Relaxed);
    }

    /// Checks and clears the show about request flag.
    pub fn check_and_clear_show_about_request(&self) -> bool {
        self.show_about_requested.swap(false, Ordering::Relaxed)
    }

    /// Returns the input timeout in milliseconds.
    pub fn input_timeout(&self) -> u64 {
        self.input_timeout.load(Ordering::Relaxed)
    }

    /// Returns the configured worker thread count.
    pub fn get_configured_worker_count(&self) -> usize {
        self.configured_worker_count
    }

    /// Returns the actual number of active worker threads.
    pub fn get_actual_worker_count(&self) -> u64 {
        self.worker_count.load(Ordering::Relaxed)
    }

    pub fn set_actual_worker_count(&self, count: usize) {
        self.worker_count.store(count as u64, Ordering::Relaxed);
    }

    pub fn get_key_mapping(&self, trigger_key: &u32) -> Option<KeyMappingInfo> {
        self.key_mappings.read().ok()?.get(trigger_key).cloned()
    }

    #[inline(always)] // Force inline for performance
    pub fn get_worker_index(&self, vk_code: u32) -> usize {
        // Bounds check is optimized away by compiler due to u8 cast
        if vk_code < 256 {
            self.vk_to_worker[vk_code as usize] as usize
        } else {
            // Fallback for out-of-range vkCodes (rare edge case)
            (vk_code as usize) % 256
        }
    }

    pub fn get_switch_key(&self) -> u32 {
        *self.switch_key.read().unwrap_or_else(|e| e.into_inner())
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

    /// Get the process name of the foreground window
    fn get_foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
            if process_id == 0 {
                return None;
            }

            let process_handle =
                match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                    Ok(handle) => handle,
                    Err(_) => return None,
                };

            let mut buffer = [0u16; MAX_PATH as usize];
            let mut size = buffer.len() as u32;

            match QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            ) {
                Ok(_) => {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    // Extract filename from full path
                    path.split('\\').next_back().map(|s| s.to_lowercase())
                }
                Err(_) => None,
            }
        }
    }

    /// Check if current foreground process is in whitelist (empty whitelist = all allowed)
    fn is_process_whitelisted(&self) -> bool {
        let whitelist = self.process_whitelist.read().unwrap();

        // Empty whitelist means all processes are enabled
        if whitelist.is_empty() {
            return true;
        }

        // Get current foreground process name
        if let Some(process_name) = Self::get_foreground_process_name() {
            // Check if process is in whitelist (case-insensitive)
            whitelist.iter().any(|p| p.to_lowercase() == process_name)
        } else {
            // If we can't get process name, allow by default
            true
        }
    }

    #[allow(non_snake_case)]
    pub fn handle_key_event(&self, message: u32, vk_code: u32) -> bool {
        let mut should_block = false;
        let switch_key = self.get_switch_key();

        if !self.is_paused()
            && self.get_key_mapping(&vk_code).is_some()
            && self.is_process_whitelisted()
            && let Some(pool) = self.worker_pool.get()
        {
            match message {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    pool.dispatch(KeyEvent::Pressed(vk_code));
                    should_block = true;
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    pool.dispatch(KeyEvent::Released(vk_code));
                    should_block = true;
                }
                _ => {}
            }
        }

        if vk_code == switch_key {
            match message {
                WM_KEYUP | WM_SYSKEYUP => {
                    let was_paused = self.toggle_paused();
                    if was_paused {
                        if let Some(sender) = self.notification_sender.get() {
                            let _ = sender
                                .send(NotificationEvent::Info("Sorahk activiting".to_string()));
                        }
                    } else if let Some(sender) = self.notification_sender.get() {
                        let _ = sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
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
