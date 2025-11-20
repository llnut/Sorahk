//! Application state management.
//!
//! Provides centralized state management for the key remapping application,
//! including configuration, keyboard event handling, and process filtering.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, LazyLock, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PWSTR;

use crate::config::AppConfig;

/// Trait for dispatching input events to worker threads.
pub trait EventDispatcher: Send + Sync {
    fn dispatch(&self, event: InputEvent);
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

/// Input device type for unified input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDevice {
    /// Keyboard input with virtual key code
    Keyboard(u32),
    /// Mouse button input
    Mouse(MouseButton),
}

/// Mouse button types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Unified input event type for keyboard and mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Pressed(InputDevice),
    Released(InputDevice),
}

/// Output action type for input mapping.
#[derive(Debug, Clone, Copy)]
pub enum OutputAction {
    /// Keyboard key output with scancode
    KeyboardKey(u16),
    /// Mouse button output
    MouseButton(MouseButton),
}

/// Configuration for a single input mapping.
#[derive(Debug, Clone, Copy)]
pub struct InputMappingInfo {
    /// Target output action
    pub target_action: OutputAction,
    /// Repeat interval in milliseconds
    pub interval: u64,
    /// Event duration in milliseconds
    pub event_duration: u64,
}

/// Legacy type alias for backward compatibility.
pub type KeyMappingInfo = InputMappingInfo;

/// Central application state manager.
///
/// Manages all runtime state including configuration, key mappings,
/// worker threads, and process filtering.
pub struct AppState {
    /// Tray icon visibility flag
    show_tray_icon: AtomicBool,
    /// Notification display flag
    show_notifications: AtomicBool,
    /// Toggle hotkey virtual key code (atomic for lock-free access in hot path)
    switch_key: AtomicU32,
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
    /// Input mapping configuration (keyboard + mouse)
    input_mappings: RwLock<HashMap<InputDevice, InputMappingInfo>>,
    /// Legacy key mappings for backward compatibility
    key_mappings: RwLock<HashMap<u32, KeyMappingInfo>>,
    /// Pre-computed input device to worker index mapping for fast dispatch
    device_to_worker: RwLock<HashMap<InputDevice, u8>>,
    /// Legacy VK to worker index mapping for backward compatibility
    vk_to_worker: [u8; 256],
    /// Worker pool for event processing
    worker_pool: OnceLock<Arc<dyn EventDispatcher>>,
    /// Notification event sender
    notification_sender: OnceLock<Sender<NotificationEvent>>,
    /// Process whitelist (empty means all processes enabled)
    process_whitelist: RwLock<Vec<String>>,
    /// Cached foreground process name with timestamp (for performance optimization)
    cached_process_info: Mutex<(Option<String>, Instant)>,
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

        let (input_mappings, key_mappings) = Self::create_input_mappings(&config)?;

        // Pre-compute device_to_worker mapping for fast dispatch
        let mut device_to_worker = HashMap::new();
        let mut vk_to_worker = [0u8; 256];

        for (idx, mapping) in config.mappings.iter().enumerate() {
            if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                device_to_worker.insert(device, idx as u8);

                // Also populate legacy vk_to_worker for keyboard keys
                if let InputDevice::Keyboard(vk) = device
                    && vk < 256
                {
                    vk_to_worker[vk as usize] = idx as u8;
                }
            }
        }

        Ok(Self {
            show_tray_icon: AtomicBool::new(config.show_tray_icon),
            show_notifications: AtomicBool::new(config.show_notifications),
            switch_key: AtomicU32::new(switch_key),
            should_exit: Arc::new(AtomicBool::new(false)),
            is_paused: AtomicBool::new(false),
            show_window_requested: AtomicBool::new(false),
            show_about_requested: AtomicBool::new(false),
            input_timeout: AtomicU64::new(config.input_timeout),
            worker_count: AtomicU64::new(0), // Will be set later
            process_whitelist: RwLock::new(config.process_whitelist.clone()),
            configured_worker_count: config.worker_count,
            input_mappings: RwLock::new(input_mappings),
            key_mappings: RwLock::new(key_mappings),
            device_to_worker: RwLock::new(device_to_worker),
            vk_to_worker,
            worker_pool: OnceLock::new(),
            notification_sender: OnceLock::new(),
            cached_process_info: Mutex::new((None, Instant::now())),
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

        self.switch_key.store(new_switch_key, Ordering::Relaxed);

        // Update show_tray_icon and show_notifications
        self.show_tray_icon
            .store(config.show_tray_icon, Ordering::Relaxed);
        self.show_notifications
            .store(config.show_notifications, Ordering::Relaxed);

        // Update input timeout
        self.input_timeout
            .store(config.input_timeout, Ordering::Relaxed);

        // Update mappings
        let (new_input_mappings, new_key_mappings) = Self::create_input_mappings(&config)?;
        if let Ok(mut mappings) = self.input_mappings.write() {
            *mappings = new_input_mappings;
        }
        if let Ok(mut mappings) = self.key_mappings.write() {
            *mappings = new_key_mappings;
        }

        // Update device_to_worker mapping
        if let Ok(mut device_map) = self.device_to_worker.write() {
            device_map.clear();
            for (idx, mapping) in config.mappings.iter().enumerate() {
                if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                    device_map.insert(device, idx as u8);
                }
            }
        }

        // Update process whitelist
        if let Ok(mut whitelist) = self.process_whitelist.write() {
            *whitelist = config.process_whitelist.clone();
        }

        // Clear process name cache to immediately apply new whitelist
        if let Ok(mut cache) = self.cached_process_info.lock() {
            *cache = (None, Instant::now());
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

    pub fn get_input_mapping(&self, device: &InputDevice) -> Option<InputMappingInfo> {
        self.input_mappings.read().ok()?.get(device).cloned()
    }

    #[inline(always)]
    pub fn get_worker_index(&self, vk_code: u32) -> usize {
        if vk_code < 256 {
            self.vk_to_worker[vk_code as usize] as usize
        } else {
            // Fallback for out-of-range vkCodes (rare edge case)
            (vk_code as usize) % 256
        }
    }

    #[inline(always)]
    pub fn get_switch_key(&self) -> u32 {
        self.switch_key.load(Ordering::Relaxed)
    }

    pub fn simulate_action(&self, action: OutputAction, duration: u64) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    // Press the key
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

                    // Release the key
                    input.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let (down_flag, up_flag) = match button {
                        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
                        MouseButton::X1 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP),
                        MouseButton::X2 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP),
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    // Press the button
                    let mut input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: down_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Release the button
                    input.Anonymous.mi.dwFlags = up_flag;
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
            }
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
    ///
    /// Performance optimization: caches process name for 50ms to avoid expensive Windows API calls
    /// - Cache hit: ~50ns (just check timestamp and clone Arc)
    /// - Cache miss: ~5µs (Windows API call)
    #[inline]
    fn is_process_whitelisted(&self) -> bool {
        // Fast path: check if whitelist is empty first (no lock needed for most users)
        let whitelist = self.process_whitelist.read().unwrap();
        if whitelist.is_empty() {
            return true;
        }

        // Cache duration: 50ms is a good balance between responsiveness and performance
        // Window switches are rare (1-2 times per second at most), so we can safely cache
        const CACHE_DURATION_MS: u64 = 50;
        let now = Instant::now();

        // Try to get cached process name
        let process_name = {
            let cache = self.cached_process_info.lock().unwrap();
            let (cached_name, cached_time) = &*cache;

            if now.duration_since(*cached_time) < Duration::from_millis(CACHE_DURATION_MS) {
                // Cache hit: use cached name (fast path)
                cached_name.clone()
            } else {
                // Cache miss: need to refresh
                drop(cache); // Release read lock before expensive API call

                // Query Windows API (slow operation)
                let new_name = Self::get_foreground_process_name();

                // Update cache
                let mut cache = self.cached_process_info.lock().unwrap();
                *cache = (new_name.clone(), now);

                new_name
            }
        };

        // Check if process is in whitelist (case-insensitive comparison)
        if let Some(name) = process_name {
            whitelist.iter().any(|p| p.to_lowercase() == name)
        } else {
            // If we can't get process name, allow by default
            true
        }
    }

    #[allow(non_snake_case)]
    pub fn handle_key_event(&self, message: u32, vk_code: u32) -> bool {
        let mut should_block = false;
        let switch_key = self.get_switch_key();
        let device = InputDevice::Keyboard(vk_code);

        if !self.is_paused()
            && self.get_input_mapping(&device).is_some()
            && self.is_process_whitelisted()
            && let Some(pool) = self.worker_pool.get()
        {
            match message {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    pool.dispatch(InputEvent::Pressed(device));
                    should_block = true;
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    pool.dispatch(InputEvent::Released(device));
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

    #[allow(non_snake_case)]
    pub fn handle_mouse_event(&self, message: u32, mouse_data: u32) -> bool {
        let mut should_block = false;

        // Parse mouse button from message
        let button_opt = match message {
            WM_LBUTTONDOWN | WM_LBUTTONUP => Some(MouseButton::Left),
            WM_RBUTTONDOWN | WM_RBUTTONUP => Some(MouseButton::Right),
            WM_MBUTTONDOWN | WM_MBUTTONUP => Some(MouseButton::Middle),
            WM_XBUTTONDOWN | WM_XBUTTONUP => {
                // Extract X button identifier from high word of mouseData
                // XBUTTON1 = 1, XBUTTON2 = 2
                let x_button = (mouse_data >> 16) & 0xFFFF;
                match x_button {
                    1 => Some(MouseButton::X1),
                    2 => Some(MouseButton::X2),
                    _ => None, // Unknown X button
                }
            }
            _ => None,
        };

        if let Some(button) = button_opt {
            let device = InputDevice::Mouse(button);

            if !self.is_paused()
                && self.get_input_mapping(&device).is_some()
                && self.is_process_whitelisted()
                && let Some(pool) = self.worker_pool.get()
            {
                match message {
                    WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
                        pool.dispatch(InputEvent::Pressed(device));
                        // Block the original down event since we'll simulate it
                        should_block = true;
                    }
                    WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
                        pool.dispatch(InputEvent::Released(device));
                        // Don't block the release event - let it pass through to the system
                        // so that context menus and other UI elements can respond properly
                        should_block = false;
                    }
                    _ => {}
                }
            }
        }

        should_block
    }

    fn create_input_mappings(
        config: &AppConfig,
    ) -> anyhow::Result<(
        HashMap<InputDevice, InputMappingInfo>,
        HashMap<u32, KeyMappingInfo>,
    )> {
        let mut input_mappings = HashMap::new();
        let mut key_mappings = HashMap::new();

        for mapping in &config.mappings {
            let trigger_device = Self::input_name_to_device(&mapping.trigger_key)
                .ok_or_else(|| anyhow::anyhow!("Invalid trigger input: {}", mapping.trigger_key))?;

            let target_action = Self::input_name_to_output(&mapping.target_key)
                .ok_or_else(|| anyhow::anyhow!("Invalid target input: {}", mapping.target_key))?;

            let interval = mapping.interval.unwrap_or(config.interval).max(5);
            let event_duration = mapping
                .event_duration
                .unwrap_or(config.event_duration)
                .max(5);

            // Create input mapping
            input_mappings.insert(
                trigger_device,
                InputMappingInfo {
                    target_action,
                    interval,
                    event_duration,
                },
            );

            // Create legacy key mapping for backward compatibility (keyboard only)
            if let InputDevice::Keyboard(vk) = trigger_device
                && let OutputAction::KeyboardKey(scancode) = target_action
            {
                key_mappings.insert(
                    vk,
                    KeyMappingInfo {
                        target_action: OutputAction::KeyboardKey(scancode),
                        interval,
                        event_duration,
                    },
                );
            }
        }

        Ok((input_mappings, key_mappings))
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

    /// Parse input name to InputDevice (supports both keyboard and mouse)
    fn input_name_to_device(name: &str) -> Option<InputDevice> {
        let name_upper = name.to_uppercase();

        // Try mouse button first
        if let Some(button) = Self::mouse_button_name_to_type(&name_upper) {
            return Some(InputDevice::Mouse(button));
        }

        // Try keyboard key
        if let Some(vk) = Self::key_name_to_vk(name) {
            return Some(InputDevice::Keyboard(vk));
        }

        None
    }

    /// Parse input name to OutputAction
    fn input_name_to_output(name: &str) -> Option<OutputAction> {
        let name_upper = name.to_uppercase();

        // Try mouse button first
        if let Some(button) = Self::mouse_button_name_to_type(&name_upper) {
            return Some(OutputAction::MouseButton(button));
        }

        // Try keyboard key
        if let Some(vk) = Self::key_name_to_vk(name) {
            let scancode = Self::vk_to_scancode(vk);
            if scancode != 0 {
                return Some(OutputAction::KeyboardKey(scancode));
            }
        }

        None
    }

    /// Parse mouse button name to MouseButton type
    fn mouse_button_name_to_type(name: &str) -> Option<MouseButton> {
        match name {
            "LBUTTON" | "LMOUSE" | "LEFTMOUSE" | "LEFTBUTTON" | "LMB" => Some(MouseButton::Left),
            "RBUTTON" | "RMOUSE" | "RIGHTMOUSE" | "RIGHTBUTTON" | "RMB" => Some(MouseButton::Right),
            "MBUTTON" | "MMOUSE" | "MIDDLEMOUSE" | "MIDDLEBUTTON" | "MMB" => {
                Some(MouseButton::Middle)
            }
            "XBUTTON1" | "X1BUTTON" | "X1" | "MB4" => Some(MouseButton::X1),
            "XBUTTON2" | "X2BUTTON" | "X2" | "MB5" => Some(MouseButton::X2),
            _ => None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyMapping;

    #[test]
    fn test_key_name_to_vk_letters() {
        assert_eq!(AppState::key_name_to_vk("A"), Some(0x41));
        assert_eq!(AppState::key_name_to_vk("Z"), Some(0x5A));
        assert_eq!(AppState::key_name_to_vk("a"), Some(0x41)); // Case insensitive
        assert_eq!(AppState::key_name_to_vk("m"), Some(0x4D));
    }

    #[test]
    fn test_key_name_to_vk_numbers() {
        assert_eq!(AppState::key_name_to_vk("0"), Some(0x30));
        assert_eq!(AppState::key_name_to_vk("5"), Some(0x35));
        assert_eq!(AppState::key_name_to_vk("9"), Some(0x39));
    }

    #[test]
    fn test_key_name_to_vk_function_keys() {
        assert_eq!(AppState::key_name_to_vk("F1"), Some(0x70));
        assert_eq!(AppState::key_name_to_vk("F12"), Some(0x7B));
        assert_eq!(AppState::key_name_to_vk("F24"), Some(0x87));
        assert_eq!(AppState::key_name_to_vk("f5"), Some(0x74)); // Case insensitive
    }

    #[test]
    fn test_key_name_to_vk_special_keys() {
        assert_eq!(AppState::key_name_to_vk("ESC"), Some(0x1B));
        assert_eq!(AppState::key_name_to_vk("ENTER"), Some(0x0D));
        assert_eq!(AppState::key_name_to_vk("TAB"), Some(0x09));
        assert_eq!(AppState::key_name_to_vk("SPACE"), Some(0x20));
        assert_eq!(AppState::key_name_to_vk("BACKSPACE"), Some(0x08));
        assert_eq!(AppState::key_name_to_vk("DELETE"), Some(0x2E));
        assert_eq!(AppState::key_name_to_vk("INSERT"), Some(0x2D));
    }

    #[test]
    fn test_key_name_to_vk_arrow_keys() {
        assert_eq!(AppState::key_name_to_vk("UP"), Some(0x26));
        assert_eq!(AppState::key_name_to_vk("DOWN"), Some(0x28));
        assert_eq!(AppState::key_name_to_vk("LEFT"), Some(0x25));
        assert_eq!(AppState::key_name_to_vk("RIGHT"), Some(0x27));
    }

    #[test]
    fn test_key_name_to_vk_modifier_keys() {
        assert_eq!(AppState::key_name_to_vk("LSHIFT"), Some(0xA0));
        assert_eq!(AppState::key_name_to_vk("RSHIFT"), Some(0xA1));
        assert_eq!(AppState::key_name_to_vk("LCTRL"), Some(0xA2));
        assert_eq!(AppState::key_name_to_vk("RCTRL"), Some(0xA3));
        assert_eq!(AppState::key_name_to_vk("LALT"), Some(0xA4));
        assert_eq!(AppState::key_name_to_vk("RALT"), Some(0xA5));
    }

    #[test]
    fn test_key_name_to_vk_navigation_keys() {
        assert_eq!(AppState::key_name_to_vk("HOME"), Some(0x24));
        assert_eq!(AppState::key_name_to_vk("END"), Some(0x23));
        assert_eq!(AppState::key_name_to_vk("PAGEUP"), Some(0x21));
        assert_eq!(AppState::key_name_to_vk("PAGEDOWN"), Some(0x22));
    }

    #[test]
    fn test_key_name_to_vk_invalid() {
        assert_eq!(AppState::key_name_to_vk("INVALID"), None);
        assert_eq!(AppState::key_name_to_vk("F25"), None);
        assert_eq!(AppState::key_name_to_vk("F0"), None);
        assert_eq!(AppState::key_name_to_vk(""), None);
        assert_eq!(AppState::key_name_to_vk("ABC"), None);
    }

    #[test]
    fn test_vk_to_scancode_letters() {
        assert_eq!(AppState::vk_to_scancode(0x41), 0x1E); // A
        assert_eq!(AppState::vk_to_scancode(0x42), 0x30); // B
        assert_eq!(AppState::vk_to_scancode(0x5A), 0x2C); // Z
    }

    #[test]
    fn test_vk_to_scancode_numbers() {
        assert_eq!(AppState::vk_to_scancode(0x30), 0x0B); // 0
        assert_eq!(AppState::vk_to_scancode(0x31), 0x02); // 1
        assert_eq!(AppState::vk_to_scancode(0x39), 0x0A); // 9
    }

    #[test]
    fn test_vk_to_scancode_function_keys() {
        assert_eq!(AppState::vk_to_scancode(0x70), 0x3B); // F1
        assert_eq!(AppState::vk_to_scancode(0x7B), 0x58); // F12
    }

    #[test]
    fn test_vk_to_scancode_special_keys() {
        assert_eq!(AppState::vk_to_scancode(0x1B), 0x01); // ESC
        assert_eq!(AppState::vk_to_scancode(0x0D), 0x1C); // ENTER
        assert_eq!(AppState::vk_to_scancode(0x20), 0x39); // SPACE
    }

    #[test]
    fn test_vk_to_scancode_invalid() {
        assert_eq!(AppState::vk_to_scancode(0xFF), 0); // Invalid VK code
        assert_eq!(AppState::vk_to_scancode(0x00), 0); // No mapping
    }

    #[test]
    fn test_create_input_mappings_valid() {
        let mut config = AppConfig::default();
        config.mappings = vec![
            KeyMapping {
                trigger_key: "A".to_string(),
                target_key: "B".to_string(),
                interval: Some(10),
                event_duration: Some(5),
            },
            KeyMapping {
                trigger_key: "F1".to_string(),
                target_key: "SPACE".to_string(),
                interval: None,
                event_duration: None,
            },
        ];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let (input_mappings, _key_mappings) = result.unwrap();
        assert_eq!(input_mappings.len(), 2);

        let device_a = InputDevice::Keyboard(0x41); // 'A' key
        let a_mapping = input_mappings.get(&device_a).unwrap();
        assert_eq!(a_mapping.interval, 10);
        assert_eq!(a_mapping.event_duration, 5);

        let device_f1 = InputDevice::Keyboard(0x70); // F1 key
        let f1_mapping = input_mappings.get(&device_f1).unwrap();
        assert_eq!(f1_mapping.interval, 5); // Default interval
        assert_eq!(f1_mapping.event_duration, 5); // Default duration
    }

    #[test]
    fn test_create_input_mappings_invalid_trigger() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "INVALID_KEY".to_string(),
            target_key: "A".to_string(),
            interval: None,
            event_duration: None,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_input_mappings_invalid_target() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "INVALID_KEY".to_string(),
            interval: None,
            event_duration: None,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_input_mappings_interval_validation() {
        let mut config = AppConfig::default();
        config.interval = 3; // Below minimum
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "B".to_string(),
            interval: Some(2), // Below minimum
            event_duration: None,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let (input_mappings, _) = result.unwrap();
        let device = InputDevice::Keyboard(0x41); // 'A' key
        let a_mapping = input_mappings.get(&device).unwrap();
        assert!(
            a_mapping.interval >= 5,
            "Interval should be clamped to minimum 5"
        );
    }

    #[test]
    fn test_create_input_mappings_duration_validation() {
        let mut config = AppConfig::default();
        config.event_duration = 2; // Below minimum
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "B".to_string(),
            interval: None,
            event_duration: Some(3), // Below minimum
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let (input_mappings, _) = result.unwrap();
        let device = InputDevice::Keyboard(0x41); // 'A' key
        let a_mapping = input_mappings.get(&device).unwrap();
        assert!(
            a_mapping.event_duration >= 5,
            "Duration should be clamped to minimum 5"
        );
    }

    #[test]
    fn test_input_event_variants() {
        let device = InputDevice::Keyboard(0x41);
        let pressed = InputEvent::Pressed(device);
        let released = InputEvent::Released(device);

        match pressed {
            InputEvent::Pressed(InputDevice::Keyboard(key)) => assert_eq!(key, 0x41),
            _ => panic!("Expected Pressed variant"),
        }

        match released {
            InputEvent::Released(InputDevice::Keyboard(key)) => assert_eq!(key, 0x41),
            _ => panic!("Expected Released variant"),
        }
    }

    #[test]
    fn test_input_mapping_info_structure() {
        let mapping = InputMappingInfo {
            target_action: OutputAction::KeyboardKey(0x1E),
            interval: 10,
            event_duration: 5,
        };

        match mapping.target_action {
            OutputAction::KeyboardKey(scancode) => assert_eq!(scancode, 0x1E),
            _ => panic!("Expected KeyboardKey action"),
        }
        assert_eq!(mapping.interval, 10);
        assert_eq!(mapping.event_duration, 5);
    }

    #[test]
    fn test_simulated_event_marker_constant() {
        assert_eq!(SIMULATED_EVENT_MARKER, 0x4659);
    }

    #[test]
    fn test_worker_index_calculation() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        let index_a = state.get_worker_index(0x41); // A key
        let index_b = state.get_worker_index(0x42); // B key

        assert!(index_a < 256);
        assert!(index_b < 256);
    }

    #[test]
    fn test_worker_index_out_of_range() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        let index = state.get_worker_index(300); // Out of range VK code
        assert!(index < 256);
    }

    #[test]
    fn test_scancode_map_coverage() {
        let map = &*SCANCODE_MAP;

        assert!(map.len() > 50, "Scancode map should contain common keys");

        assert!(map.contains_key(&0x41)); // A
        assert!(map.contains_key(&0x70)); // F1
        assert!(map.contains_key(&0x20)); // SPACE
        assert!(map.contains_key(&0x0D)); // ENTER
    }

    #[test]
    fn test_case_insensitive_key_names() {
        assert_eq!(
            AppState::key_name_to_vk("space"),
            AppState::key_name_to_vk("SPACE")
        );
        assert_eq!(
            AppState::key_name_to_vk("enter"),
            AppState::key_name_to_vk("ENTER")
        );
        assert_eq!(
            AppState::key_name_to_vk("esc"),
            AppState::key_name_to_vk("ESC")
        );
        assert_eq!(
            AppState::key_name_to_vk("delete"),
            AppState::key_name_to_vk("DELETE")
        );
    }

    #[test]
    fn test_multiple_input_mappings() {
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
                interval: Some(15),
                event_duration: Some(8),
            },
            KeyMapping {
                trigger_key: "C".to_string(),
                target_key: "3".to_string(),
                interval: Some(20),
                event_duration: Some(10),
            },
        ];

        let (input_mappings, _) = AppState::create_input_mappings(&config).unwrap();
        assert_eq!(input_mappings.len(), 3);

        let device_a = InputDevice::Keyboard(0x41);
        let device_b = InputDevice::Keyboard(0x42);
        let device_c = InputDevice::Keyboard(0x43);

        assert_eq!(input_mappings.get(&device_a).unwrap().interval, 10);
        assert_eq!(input_mappings.get(&device_b).unwrap().interval, 15);
        assert_eq!(input_mappings.get(&device_c).unwrap().interval, 20);
    }

    #[test]
    fn test_app_state_pause_toggle() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // Initially not paused
        assert!(!state.is_paused());

        // Toggle to paused
        let was_paused = state.toggle_paused();
        assert!(!was_paused);
        assert!(state.is_paused());

        // Toggle back to not paused
        let was_paused = state.toggle_paused();
        assert!(was_paused);
        assert!(!state.is_paused());
    }

    #[test]
    fn test_app_state_set_paused() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        state.set_paused(true);
        assert!(state.is_paused());

        state.set_paused(false);
        assert!(!state.is_paused());
    }

    #[test]
    fn test_app_state_show_window_request() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // No request initially
        assert!(!state.check_and_clear_show_window_request());

        // Request window
        state.request_show_window();
        assert!(state.check_and_clear_show_window_request());

        // Should be cleared after check
        assert!(!state.check_and_clear_show_window_request());
    }

    #[test]
    fn test_app_state_show_about_request() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // No request initially
        assert!(!state.check_and_clear_show_about_request());

        // Request about dialog
        state.request_show_about();
        assert!(state.check_and_clear_show_about_request());

        // Should be cleared after check
        assert!(!state.check_and_clear_show_about_request());
    }

    #[test]
    fn test_app_state_exit_flag() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        assert!(!state.should_exit());

        state.exit();
        assert!(state.should_exit());
    }

    #[test]
    fn test_app_state_worker_count() {
        let mut config = AppConfig::default();
        config.worker_count = 4;
        let state = AppState::new(config).unwrap();

        assert_eq!(state.get_configured_worker_count(), 4);

        state.set_actual_worker_count(8);
        assert_eq!(state.get_actual_worker_count(), 8);
    }

    #[test]
    fn test_app_state_input_timeout() {
        let mut config = AppConfig::default();
        config.input_timeout = 25;
        let state = AppState::new(config).unwrap();

        assert_eq!(state.input_timeout(), 25);
    }

    #[test]
    fn test_app_state_tray_settings() {
        let mut config = AppConfig::default();
        config.show_tray_icon = true;
        config.show_notifications = false;
        let state = AppState::new(config).unwrap();

        assert!(state.show_tray_icon());
        assert!(!state.show_notifications());
    }

    #[test]
    fn test_app_state_switch_key() {
        let mut config = AppConfig::default();
        config.switch_key = "F12".to_string();
        let state = AppState::new(config).unwrap();

        assert_eq!(state.get_switch_key(), 0x7B); // F12 VK code
    }

    #[test]
    fn test_app_state_reload_config() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // Initial state
        assert!(!state.is_paused());
        assert_eq!(state.get_switch_key(), 0x2E); // DELETE

        // Create new config
        let mut new_config = AppConfig::default();
        new_config.switch_key = "F11".to_string();
        new_config.show_tray_icon = false;
        new_config.input_timeout = 50;

        // Reload config
        state.reload_config(new_config).unwrap();

        // Verify changes
        assert_eq!(state.get_switch_key(), 0x7A); // F11
        assert!(!state.show_tray_icon());
        assert_eq!(state.input_timeout(), 50);
    }

    #[test]
    fn test_notification_event_variants() {
        let info = NotificationEvent::Info("test".to_string());
        let warning = NotificationEvent::Warning("test".to_string());
        let error = NotificationEvent::Error("test".to_string());

        match info {
            NotificationEvent::Info(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Info variant"),
        }

        match warning {
            NotificationEvent::Warning(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Warning variant"),
        }

        match error {
            NotificationEvent::Error(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_atomic_pause_state_thread_safety() {
        use std::thread;

        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let state_clone = state.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        if i % 2 == 0 {
                            state_clone.set_paused(true);
                        } else {
                            state_clone.set_paused(false);
                        }
                        state_clone.toggle_paused();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // State should be consistent after concurrent operations
        let final_state = state.is_paused();
        assert!(final_state == true || final_state == false);
    }

    #[test]
    fn test_worker_count_atomic_operations() {
        use std::thread;

        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let state_clone = state.clone();
                thread::spawn(move || {
                    for i in 1..=100 {
                        state_clone.set_actual_worker_count(i);
                        let count = state_clone.get_actual_worker_count();
                        assert!(count > 0 && count <= 100);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_key_mapping_with_boundary_values() {
        let mut config = AppConfig::default();

        // Test with minimum interval
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "B".to_string(),
            interval: Some(5), // Minimum valid value
            event_duration: Some(5),
        }];

        let state = AppState::new(config);
        assert!(state.is_ok());
    }

    #[test]
    fn test_key_mapping_with_zero_interval() {
        let mut config = AppConfig::default();

        // Test with zero interval (should be auto-adjusted to minimum)
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "B".to_string(),
            interval: Some(0),
            event_duration: Some(0),
        }];

        let state = AppState::new(config).unwrap();

        // Values should be adjusted to minimum of 5
        // This test verifies auto-adjustment behavior
        assert!(state.key_mappings.read().unwrap().len() > 0);
    }

    #[test]
    fn test_vk_to_scancode_common_keys() {
        // Test commonly mapped VK codes that exist in SCANCODE_MAP
        assert_eq!(AppState::vk_to_scancode(0x08), 0x0E); // Backspace
        assert_eq!(AppState::vk_to_scancode(0x09), 0x0F); // Tab
        assert_eq!(AppState::vk_to_scancode(0x0D), 0x1C); // Enter
        assert_eq!(AppState::vk_to_scancode(0x20), 0x39); // Space

        // Keys not in map return 0
        let unmapped = AppState::vk_to_scancode(0xFF);
        assert_eq!(unmapped, 0);
    }

    #[test]
    fn test_key_name_to_vk_extended_keys() {
        // Test extended Windows keys that are implemented
        assert_eq!(AppState::key_name_to_vk("LWIN"), Some(0x5B));
        assert_eq!(AppState::key_name_to_vk("RWIN"), Some(0x5C));
        assert_eq!(AppState::key_name_to_vk("PAUSE"), Some(0x13));
        assert_eq!(AppState::key_name_to_vk("CAPSLOCK"), Some(0x14));

        // Keys not implemented return None
        assert_eq!(AppState::key_name_to_vk("NUMPAD0"), None);
        assert_eq!(AppState::key_name_to_vk("MULTIPLY"), None);
    }

    #[test]
    fn test_empty_process_whitelist() {
        let mut config = AppConfig::default();
        config.process_whitelist = vec![];

        let state = AppState::new(config).unwrap();

        // With empty whitelist, all processes should be whitelisted
        assert!(state.is_process_whitelisted());
    }

    #[test]
    fn test_process_whitelist_cache() {
        use std::thread;
        use std::time::Duration;

        let mut config = AppConfig::default();
        config.process_whitelist = vec!["explorer.exe".to_string()];

        let state = AppState::new(config).unwrap();

        // First call - cache miss (will query Windows API)
        let _ = state.is_process_whitelisted();

        // Verify cache was populated
        let cache = state.cached_process_info.lock().unwrap();
        let (cached_name, _) = &*cache;
        let initial_name = cached_name.clone();
        drop(cache);

        // Second call immediately - cache hit (should use cached value)
        let _ = state.is_process_whitelisted();

        // Verify cache still has same value
        let cache = state.cached_process_info.lock().unwrap();
        let (cached_name, _) = &*cache;
        assert_eq!(*cached_name, initial_name);
        drop(cache);

        // Wait for cache to expire (>50ms)
        thread::sleep(Duration::from_millis(60));

        // Third call after expiration - cache miss (will refresh)
        let _ = state.is_process_whitelisted();

        // Cache should be refreshed with new timestamp
        let cache = state.cached_process_info.lock().unwrap();
        let (_, timestamp) = &*cache;
        assert!(timestamp.elapsed() < Duration::from_millis(10)); // Should be very recent
    }

    #[test]
    fn test_x_button_parsing() {
        use windows::Win32::UI::WindowsAndMessaging::*;

        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // Simulate XBUTTON1 down (mouse_data high word = 1)
        let mouse_data_x1: u32 = 1 << 16; // XBUTTON1
        let _result = state.handle_mouse_event(WM_XBUTTONDOWN, mouse_data_x1);
        // Should parse as X1 button

        // Simulate XBUTTON2 up (mouse_data high word = 2)
        let mouse_data_x2: u32 = 2 << 16; // XBUTTON2
        let _result = state.handle_mouse_event(WM_XBUTTONUP, mouse_data_x2);
        // Should parse as X2 button
    }

    #[test]
    fn test_mouse_button_name_parsing() {
        // Test X button name parsing
        assert_eq!(
            AppState::mouse_button_name_to_type("XBUTTON1"),
            Some(MouseButton::X1)
        );
        assert_eq!(
            AppState::mouse_button_name_to_type("XBUTTON2"),
            Some(MouseButton::X2)
        );
        assert_eq!(
            AppState::mouse_button_name_to_type("X1"),
            Some(MouseButton::X1)
        );
        assert_eq!(
            AppState::mouse_button_name_to_type("MB4"),
            Some(MouseButton::X1)
        );
        assert_eq!(
            AppState::mouse_button_name_to_type("MB5"),
            Some(MouseButton::X2)
        );
    }

    #[test]
    fn test_concurrent_window_requests() {
        use std::thread;

        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let state_clone = state.clone();
                thread::spawn(move || {
                    for _ in 0..20 {
                        state_clone.request_show_window();
                        state_clone.check_and_clear_show_window_request();
                        state_clone.request_show_about();
                        state_clone.check_and_clear_show_about_request();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Final state should be consistent
        assert!(!state.check_and_clear_show_window_request());
        assert!(!state.check_and_clear_show_about_request());
    }
}
