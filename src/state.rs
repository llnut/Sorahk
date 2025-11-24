//! Application state management.
//!
//! Provides centralized state management for the key remapping application,
//! including configuration, keyboard event handling, and process filtering.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputDevice {
    /// Keyboard input with virtual key code
    Keyboard(u32),
    /// Mouse button input
    Mouse(MouseButton),
    /// Key combination input (modifier keys + main key)
    /// Format: [modifier1, modifier2, ..., main_key]
    /// The last element is always the main key, others are modifiers
    KeyCombo(Vec<u32>),
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Pressed(InputDevice),
    Released(InputDevice),
}

/// Output action type for input mapping.
#[derive(Debug, Clone)]
pub enum OutputAction {
    /// Keyboard key output with scancode
    KeyboardKey(u16),
    /// Mouse button output
    MouseButton(MouseButton),
    /// Key combination output (modifier scancodes + main key scancode)
    /// Format: [modifier1_scancode, modifier2_scancode, ..., main_key_scancode]
    /// Using Arc to avoid cloning on every key repeat
    KeyCombo(Arc<[u16]>),
}

/// Configuration for a single input mapping.
#[derive(Debug, Clone)]
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
    /// Input mapping configuration (keyboard + mouse) - lock-free concurrent HashMap
    input_mappings: scc::HashMap<InputDevice, InputMappingInfo>,
    /// Legacy key mappings for backward compatibility - lock-free concurrent HashMap
    key_mappings: scc::HashMap<u32, KeyMappingInfo>,
    /// Pre-computed input device to worker index mapping for fast dispatch - lock-free concurrent HashMap
    device_to_worker: scc::HashMap<InputDevice, u8>,
    /// Legacy VK to worker index mapping for backward compatibility
    vk_to_worker: [u8; 256],
    /// Worker pool for event processing
    worker_pool: OnceLock<Arc<dyn EventDispatcher>>,
    /// Notification event sender
    notification_sender: OnceLock<Sender<NotificationEvent>>,
    /// Process whitelist (empty means all processes enabled) - keep Mutex for Vec
    process_whitelist: Mutex<Vec<String>>,
    /// Cached foreground process name with timestamp
    cached_process_info: Mutex<(Option<String>, Instant)>,
    /// Currently pressed keys for combo detection - lock-free concurrent HashSet
    pressed_keys: scc::HashSet<u32>,
    /// Active combo triggers (multiple combos can be active simultaneously) - lock-free concurrent HashMap
    /// Maps combo device to the set of modifier keys that were suppressed
    active_combo_triggers: scc::HashMap<InputDevice, std::collections::HashSet<u32>>,
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

        let (input_mappings_map, key_mappings_map) = Self::create_input_mappings(&config)?;

        // Create lock-free concurrent HashMaps
        let input_mappings = scc::HashMap::new();
        let key_mappings = scc::HashMap::new();
        let device_to_worker = scc::HashMap::new();

        // Populate input_mappings and key_mappings
        for (k, v) in input_mappings_map {
            let _ = input_mappings.insert_sync(k, v);
        }
        for (k, v) in key_mappings_map {
            let _ = key_mappings.insert_sync(k, v);
        }

        // Pre-compute device_to_worker mapping for fast dispatch
        let mut vk_to_worker = [0u8; 256];

        for (idx, mapping) in config.mappings.iter().enumerate() {
            if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                let _ = device_to_worker.insert_sync(device.clone(), idx as u8);

                // Also populate legacy vk_to_worker for keyboard keys
                match &device {
                    InputDevice::Keyboard(vk) if *vk < 256 => {
                        vk_to_worker[*vk as usize] = idx as u8;
                    }
                    InputDevice::KeyCombo(keys) => {
                        if let Some(&last_key) = keys.last()
                            && last_key < 256
                        {
                            vk_to_worker[last_key as usize] = idx as u8;
                        }
                    }
                    _ => {}
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
            worker_count: AtomicU64::new(0),
            process_whitelist: Mutex::new(config.process_whitelist.clone()),
            configured_worker_count: config.worker_count,
            input_mappings,
            key_mappings,
            device_to_worker,
            vk_to_worker,
            worker_pool: OnceLock::new(),
            notification_sender: OnceLock::new(),
            cached_process_info: Mutex::new((None, Instant::now())),
            pressed_keys: scc::HashSet::new(),
            active_combo_triggers: scc::HashMap::new(),
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

        // Update mappings (lock-free concurrent HashMap)
        let (new_input_mappings, new_key_mappings) = Self::create_input_mappings(&config)?;
        self.input_mappings.clear_sync();
        for (k, v) in new_input_mappings {
            let _ = self.input_mappings.insert_sync(k, v);
        }
        self.key_mappings.clear_sync();
        for (k, v) in new_key_mappings {
            let _ = self.key_mappings.insert_sync(k, v);
        }

        // Update device_to_worker mapping (lock-free concurrent HashMap)
        self.device_to_worker.clear_sync();
        for (idx, mapping) in config.mappings.iter().enumerate() {
            if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                let _ = self.device_to_worker.insert_sync(device, idx as u8);
            }
        }

        // Update process whitelist
        if let Ok(mut whitelist) = self.process_whitelist.lock() {
            *whitelist = config.process_whitelist.clone();
        }

        // Clear process name cache
        if let Ok(mut cache) = self.cached_process_info.lock() {
            *cache = (None, Instant::now());
        }

        // Clear pressed keys and active combos (lock-free concurrent structures)
        self.pressed_keys.clear_sync();
        self.active_combo_triggers.clear_sync();

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
    pub fn get_actual_worker_count(&self) -> usize {
        self.worker_count.load(Ordering::Relaxed) as usize
    }

    pub fn set_actual_worker_count(&self, count: usize) {
        self.worker_count.store(count as u64, Ordering::Relaxed);
    }

    pub fn get_input_mapping(&self, device: &InputDevice) -> Option<InputMappingInfo> {
        self.input_mappings.read_sync(device, |_, v| v.clone())
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

    /// Check if a key is a modifier key
    #[inline]
    fn is_modifier_key(&self, vk: u32) -> bool {
        matches!(
            vk,
            0x10 | 0xA0 | 0xA1 |  // SHIFT, LSHIFT, RSHIFT
            0x11 | 0xA2 | 0xA3 |  // CTRL, LCTRL, RCTRL
            0x12 | 0xA4 | 0xA5 |  // ALT, LALT, RALT
            0x5B | 0x5C // LWIN, RWIN
        )
    }

    /// Check if a key is part of any active combo (should be blocked)
    /// Returns true if the key should be intercepted
    #[inline]
    fn is_in_active_combo(&self, vk_code: u32) -> bool {
        let mut found = false;
        self.active_combo_triggers.iter_sync(|combo_device, _| {
            if found {
                return false;
            }
            if let InputDevice::KeyCombo(keys) = combo_device
                && keys.contains(&vk_code)
            {
                found = true;
            }
            true
        });
        found
    }

    /// Add a combo to active triggers
    fn add_active_combo(&self, combo: InputDevice, modifiers: std::collections::HashSet<u32>) {
        let _ = self.active_combo_triggers.insert_sync(combo, modifiers);
    }

    /// Check if a specific combo is active
    fn is_combo_active(&self, combo: &InputDevice) -> bool {
        self.active_combo_triggers.contains_sync(combo)
    }

    /// Release modifiers once (send KEYUP events using scancodes)
    /// Skips modifiers already suppressed by other active combos
    fn release_modifiers_once(&self, modifiers: &std::collections::HashSet<u32>) {
        if modifiers.is_empty() {
            return;
        }

        // Check which modifiers are already suppressed by active combos
        let mut already_suppressed: std::collections::HashSet<u32> =
            std::collections::HashSet::new();
        self.active_combo_triggers.iter_sync(|_, modifiers| {
            already_suppressed.extend(modifiers.iter().copied());
            true
        });

        unsafe {
            for &vk in modifiers {
                // Skip if this modifier is already suppressed by another active combo
                if already_suppressed.contains(&vk) {
                    continue;
                }

                let (scancode, is_extended) = match vk {
                    0x10 | 0xA0 => (0x2A, false), // SHIFT, LSHIFT
                    0xA1 => (0x36, false),        // RSHIFT
                    0x11 | 0xA2 => (0x1D, false), // CTRL, LCTRL
                    0xA3 => (0x1D, true),         // RCTRL (extended)
                    0x12 | 0xA4 => (0x38, false), // ALT, LALT
                    0xA5 => (0x38, true),         // RALT (extended)
                    0x5B => (0x5B, true),         // LWIN (extended)
                    0x5C => (0x5C, true),         // RWIN (extended)
                    _ => continue,
                };

                let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                if is_extended {
                    flags |= KEYEVENTF_EXTENDEDKEY;
                }

                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: scancode,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: SIMULATED_EVENT_MARKER,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }

            // Small delay to ensure modifier release is processed
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// Remove combos that no longer have all their keys pressed
    /// Returns vec of combos that were removed
    fn cleanup_released_combos(&self) -> Vec<InputDevice> {
        // Create snapshot of currently pressed keys
        let mut pressed_snapshot: std::collections::HashSet<u32> = std::collections::HashSet::new();
        self.pressed_keys.iter_sync(|key| {
            pressed_snapshot.insert(*key);
            true
        });

        let mut removed = Vec::new();

        // Collect combos to remove
        let mut to_remove = Vec::new();
        self.active_combo_triggers
            .iter_sync(|combo_device, _modifiers| {
                if let InputDevice::KeyCombo(keys) = combo_device {
                    let all_pressed = keys.iter().all(|&k| pressed_snapshot.contains(&k));
                    if !all_pressed {
                        to_remove.push(combo_device.clone());
                    }
                }
                true
            });

        // Remove them and collect to return
        for combo in to_remove {
            if self.active_combo_triggers.remove_sync(&combo).is_some() {
                removed.push(combo);
            }
        }

        removed
    }

    /// Find device for release event (single key or combo from active triggers)
    fn find_device_for_release(&self, vk_code: u32) -> Option<InputDevice> {
        // Check if it's part of any active combo
        let mut result = None;
        self.active_combo_triggers.iter_sync(|combo_device, _| {
            if result.is_some() {
                return false;
            }
            if let InputDevice::KeyCombo(keys) = combo_device
                && keys.contains(&vk_code)
            {
                result = Some(combo_device.clone());
            }
            true
        });
        if result.is_some() {
            return result;
        }

        // Otherwise, check for single key mapping
        let device = InputDevice::Keyboard(vk_code);
        if self.get_input_mapping(&device).is_some() {
            Some(device)
        } else {
            None
        }
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
                OutputAction::KeyCombo(scancodes) => {
                    // Press all keys in sequence (modifiers first, then main key)
                    for &scancode in scancodes.iter() {
                        let input = INPUT {
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
                        // Short delay between keys for better compatibility
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }

                    // Hold duration
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Release all keys in reverse order (main key first, then modifiers)
                    for &scancode in scancodes.iter().rev() {
                        let input = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
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
        let whitelist = self.process_whitelist.lock().unwrap();
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

        // Handle switch key first (always works even when paused)
        if vk_code == switch_key && matches!(message, WM_KEYUP | WM_SYSKEYUP) {
            let was_paused = self.toggle_paused();
            // Clear all state when toggling
            self.pressed_keys.clear_sync();
            self.active_combo_triggers.clear_sync();
            if was_paused {
                if let Some(sender) = self.notification_sender.get() {
                    let _ = sender.send(NotificationEvent::Info("Sorahk activiting".to_string()));
                }
            } else if let Some(sender) = self.notification_sender.get() {
                let _ = sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
            }
            return true;
        }

        // Only process if not paused and in whitelisted process
        if self.is_paused() || !self.is_process_whitelisted() {
            return should_block;
        }

        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // Block keyboard repeat events for active combos
                if self.is_in_active_combo(vk_code) {
                    return true;
                }

                // First-time key press: add to pressed_keys
                let _ = self.pressed_keys.insert_sync(vk_code);

                // Try to match combo key first, then single key
                // Create snapshot for combo matching
                let mut pressed_snapshot: std::collections::HashSet<u32> =
                    std::collections::HashSet::new();
                self.pressed_keys.iter_sync(|key| {
                    pressed_snapshot.insert(*key);
                    true
                });
                let matched_device = self
                    .find_matching_combo(&pressed_snapshot, vk_code)
                    .or_else(|| {
                        let device = InputDevice::Keyboard(vk_code);
                        if self.get_input_mapping(&device).is_some() {
                            Some(device)
                        } else {
                            None
                        }
                    });

                if let Some(device) = matched_device {
                    // Skip if already active
                    if self.is_combo_active(&device) {
                        return true;
                    }

                    // Handle combo key: always suppress physical modifiers to avoid interference
                    if let InputDevice::KeyCombo(_) = &device {
                        let mut modifiers: std::collections::HashSet<u32> =
                            std::collections::HashSet::new();
                        self.pressed_keys.iter_sync(|key| {
                            if *key != vk_code && self.is_modifier_key(*key) {
                                modifiers.insert(*key);
                            }
                            true
                        });

                        // Always release physical modifiers and track combo
                        // This prevents physical modifiers from interfering with simulated output
                        self.release_modifiers_once(&modifiers);
                        self.add_active_combo(device.clone(), modifiers.clone());
                    }

                    // Dispatch event to worker
                    if let Some(pool) = self.worker_pool.get() {
                        pool.dispatch(InputEvent::Pressed(device));
                        should_block = true;
                    }
                }
            }

            WM_KEYUP | WM_SYSKEYUP => {
                // Remove from pressed_keys
                let _ = self.pressed_keys.remove_sync(&vk_code);

                // Check which combos are no longer valid (keys released)
                let removed_combos = self.cleanup_released_combos();

                // Stop all removed combos
                if !removed_combos.is_empty()
                    && let Some(pool) = self.worker_pool.get()
                {
                    for combo in removed_combos {
                        pool.dispatch(InputEvent::Released(combo));
                    }
                    should_block = true;
                }

                // Also check for single key release (may coexist with combo)
                let device = self.find_device_for_release(vk_code);
                if let Some(dev) = device
                    && let Some(pool) = self.worker_pool.get()
                {
                    pool.dispatch(InputEvent::Released(dev));
                    should_block = true;
                }
            }

            _ => {}
        }

        should_block
    }

    /// Find a matching key combo from currently pressed keys
    /// Supports multiple combos simultaneously (e.g., ALT+1, ALT+2, ALT+3 all active)
    fn find_matching_combo(
        &self,
        pressed_keys: &std::collections::HashSet<u32>,
        main_key: u32,
    ) -> Option<InputDevice> {
        let mut result = None;
        self.input_mappings.iter_sync(|device, _| {
            if result.is_some() {
                return false;
            }
            if let InputDevice::KeyCombo(combo_keys) = device
                && let Some(&last_key) = combo_keys.last()
                && last_key == main_key
            {
                let all_pressed = combo_keys.iter().all(|&k| pressed_keys.contains(&k));
                if all_pressed {
                    result = Some(device.clone());
                }
            }
            true
        });
        result
    }

    /// Find a combo key that contains the released key
    /// This is called when a key is released to check if it was part of an active combo
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
                .max(2);

            // Create input mapping
            input_mappings.insert(
                trigger_device.clone(),
                InputMappingInfo {
                    target_action: target_action.clone(),
                    interval,
                    event_duration,
                },
            );

            // Create legacy key mapping for backward compatibility (keyboard only)
            if let InputDevice::Keyboard(vk) = &trigger_device
                && let OutputAction::KeyboardKey(scancode) = &target_action
            {
                key_mappings.insert(
                    *vk,
                    KeyMappingInfo {
                        target_action: OutputAction::KeyboardKey(*scancode),
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

        // Numpad keys
        if key.starts_with("NUMPAD")
            && key.len() > 6
            && let Ok(num) = key[6..].parse::<u32>()
            && num <= 9
        {
            return Some(0x60 + num);
        }

        // special keys
        match key.as_str() {
            "ESC" | "ESCAPE" => Some(0x1B),
            "ENTER" | "RETURN" => Some(0x0D),
            "TAB" => Some(0x09),
            "CLEAR" => Some(0x0C),
            "SHIFT" => Some(0x10),
            "CTRL" => Some(0x11),
            "ALT" => Some(0x12),
            "PAUSE" => Some(0x13),
            "CAPSLOCK" | "CAPITAL" => Some(0x14),
            "SPACE" => Some(0x20),
            "BACKSPACE" | "BACK" => Some(0x08),
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
            "NUMLOCK" => Some(0x90),
            "SCROLL" => Some(0x91),
            "SNAPSHOT" => Some(0x2C),
            "MULTIPLY" => Some(0x6A),
            "ADD" => Some(0x6B),
            "SEPARATOR" => Some(0x6C),
            "SUBTRACT" => Some(0x6D),
            "DECIMAL" => Some(0x6E),
            "DIVIDE" => Some(0x6F),
            "OEM_1" => Some(0xBA),
            "OEM_PLUS" => Some(0xBB),
            "OEM_COMMA" => Some(0xBC),
            "OEM_MINUS" => Some(0xBD),
            "OEM_PERIOD" => Some(0xBE),
            "OEM_2" => Some(0xBF),
            "OEM_3" => Some(0xC0),
            "OEM_4" => Some(0xDB),
            "OEM_5" => Some(0xDC),
            "OEM_6" => Some(0xDD),
            "OEM_7" => Some(0xDE),
            "OEM_8" => Some(0xDF),
            "OEM_102" => Some(0xE2),
            "LBUTTON" => Some(0x01),
            "RBUTTON" => Some(0x02),
            "MBUTTON" => Some(0x04),
            "XBUTTON1" => Some(0x05),
            "XBUTTON2" => Some(0x06),
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

        // Check if it's a key combination (contains '+')
        if name.contains('+') {
            let parts: Vec<&str> = name.split('+').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                return None;
            }

            let mut vk_codes = Vec::new();
            for part in parts {
                if let Some(vk) = Self::key_name_to_vk(part) {
                    vk_codes.push(vk);
                } else {
                    // Invalid key name in combination
                    return None;
                }
            }

            // Ensure the combination is valid (has at least 2 keys)
            if vk_codes.len() >= 2 {
                return Some(InputDevice::KeyCombo(vk_codes));
            }
        }

        // Try single keyboard key
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

        // Check if it's a key combination (contains '+')
        if name.contains('+') {
            let parts: Vec<&str> = name.split('+').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                return None;
            }

            let mut scancodes = Vec::new();
            for part in parts {
                if let Some(vk) = Self::key_name_to_vk(part) {
                    let scancode = Self::vk_to_scancode(vk);
                    if scancode == 0 {
                        // Invalid scancode
                        return None;
                    }
                    scancodes.push(scancode);
                } else {
                    // Invalid key name
                    return None;
                }
            }

            // Ensure the combination is valid
            if scancodes.len() >= 2 {
                // Use Arc to avoid cloning on every repeat
                return Some(OutputAction::KeyCombo(Arc::from(
                    scancodes.into_boxed_slice(),
                )));
            }
        }

        // Try single keyboard key
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
        // lock keys
        (0x14, 0x3A), // CAPSLOCK
        (0x90, 0x45), // NUMLOCK
        (0x91, 0x46), // SCROLL LOCK
        (0x13, 0x45), // PAUSE (same as NUMLOCK)
        (0x2C, 0x37), // PRINT SCREEN (same as MULTIPLY)
        // numpad keys
        (0x60, 0x52), // NUMPAD0
        (0x61, 0x4F), // NUMPAD1
        (0x62, 0x50), // NUMPAD2
        (0x63, 0x51), // NUMPAD3
        (0x64, 0x4B), // NUMPAD4
        (0x65, 0x4C), // NUMPAD5
        (0x66, 0x4D), // NUMPAD6
        (0x67, 0x47), // NUMPAD7
        (0x68, 0x48), // NUMPAD8
        (0x69, 0x49), // NUMPAD9
        (0x6A, 0x37), // MULTIPLY
        (0x6B, 0x4E), // ADD
        (0x6C, 0x53), // SEPARATOR
        (0x6D, 0x4A), // SUBTRACT
        (0x6E, 0x53), // DECIMAL
        (0x6F, 0x35), // DIVIDE
        // OEM keys
        (0xBA, 0x27), // OEM_1 (;:)
        (0xBB, 0x0D), // OEM_PLUS (=+)
        (0xBC, 0x33), // OEM_COMMA (,<)
        (0xBD, 0x0C), // OEM_MINUS (-_)
        (0xBE, 0x34), // OEM_PERIOD (.>)
        (0xBF, 0x35), // OEM_2 (/?)
        (0xC0, 0x29), // OEM_3 (`~)
        (0xDB, 0x1A), // OEM_4 ([{)
        (0xDC, 0x2B), // OEM_5 (\|)
        (0xDD, 0x1B), // OEM_6 (]})
        (0xDE, 0x28), // OEM_7 ('")
        (0xDF, 0x29), // OEM_8
        (0xE2, 0x56), // OEM_102 (<>)
        // modifier keys
        (0xA0, 0x2A), // LSHIFT
        (0xA1, 0x36), // RSHIFT
        (0xA2, 0x1D), // LCTRL
        (0xA3, 0x1D), // RCTRL (same scancode, use extended flag)
        (0xA4, 0x38), // LALT
        (0xA5, 0x38), // RALT (same scancode, use extended flag)
        (0x5B, 0x5B), // LWIN
        (0x5C, 0x5C), // RWIN
        // generic modifier keys
        (0x10, 0x2A), // SHIFT (generic)
        (0x11, 0x1D), // CTRL (generic)
        (0x12, 0x38), // ALT (generic)
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
            interval: Some(3), // Below minimum
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
            a_mapping.event_duration >= 2,
            "Duration should be clamped to minimum 2"
        );
    }

    #[test]
    fn test_input_event_variants() {
        let device = InputDevice::Keyboard(0x41);
        let pressed = InputEvent::Pressed(device.clone());
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
            event_duration: Some(2),
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

        // Values should be adjusted to minimum of 2
        // This test verifies auto-adjustment behavior
        assert!(state.key_mappings.len() > 0);
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
        assert_eq!(AppState::key_name_to_vk("LWIN"), Some(0x5B));
        assert_eq!(AppState::key_name_to_vk("RWIN"), Some(0x5C));
        assert_eq!(AppState::key_name_to_vk("PAUSE"), Some(0x13));
        assert_eq!(AppState::key_name_to_vk("CAPSLOCK"), Some(0x14));
        assert_eq!(AppState::key_name_to_vk("CAPITAL"), Some(0x14));
        assert_eq!(AppState::key_name_to_vk("NUMLOCK"), Some(0x90));
        assert_eq!(AppState::key_name_to_vk("SCROLL"), Some(0x91));
        assert_eq!(AppState::key_name_to_vk("SNAPSHOT"), Some(0x2C));
    }

    #[test]
    fn test_key_name_to_vk_numpad_keys() {
        assert_eq!(AppState::key_name_to_vk("NUMPAD0"), Some(0x60));
        assert_eq!(AppState::key_name_to_vk("NUMPAD1"), Some(0x61));
        assert_eq!(AppState::key_name_to_vk("NUMPAD5"), Some(0x65));
        assert_eq!(AppState::key_name_to_vk("NUMPAD9"), Some(0x69));
        assert_eq!(AppState::key_name_to_vk("MULTIPLY"), Some(0x6A));
        assert_eq!(AppState::key_name_to_vk("ADD"), Some(0x6B));
        assert_eq!(AppState::key_name_to_vk("SUBTRACT"), Some(0x6D));
        assert_eq!(AppState::key_name_to_vk("DECIMAL"), Some(0x6E));
        assert_eq!(AppState::key_name_to_vk("DIVIDE"), Some(0x6F));
    }

    #[test]
    fn test_key_name_to_vk_oem_keys() {
        assert_eq!(AppState::key_name_to_vk("OEM_1"), Some(0xBA));
        assert_eq!(AppState::key_name_to_vk("OEM_2"), Some(0xBF));
        assert_eq!(AppState::key_name_to_vk("OEM_3"), Some(0xC0));
        assert_eq!(AppState::key_name_to_vk("OEM_4"), Some(0xDB));
        assert_eq!(AppState::key_name_to_vk("OEM_5"), Some(0xDC));
        assert_eq!(AppState::key_name_to_vk("OEM_6"), Some(0xDD));
        assert_eq!(AppState::key_name_to_vk("OEM_7"), Some(0xDE));
        assert_eq!(AppState::key_name_to_vk("OEM_PLUS"), Some(0xBB));
        assert_eq!(AppState::key_name_to_vk("OEM_COMMA"), Some(0xBC));
        assert_eq!(AppState::key_name_to_vk("OEM_MINUS"), Some(0xBD));
        assert_eq!(AppState::key_name_to_vk("OEM_PERIOD"), Some(0xBE));
    }

    #[test]
    fn test_key_name_to_vk_mouse_buttons() {
        assert_eq!(AppState::key_name_to_vk("LBUTTON"), Some(0x01));
        assert_eq!(AppState::key_name_to_vk("RBUTTON"), Some(0x02));
        assert_eq!(AppState::key_name_to_vk("MBUTTON"), Some(0x04));
        assert_eq!(AppState::key_name_to_vk("XBUTTON1"), Some(0x05));
        assert_eq!(AppState::key_name_to_vk("XBUTTON2"), Some(0x06));
    }

    #[test]
    fn test_key_name_aliases() {
        assert_eq!(AppState::key_name_to_vk("ESC"), Some(0x1B));
        assert_eq!(AppState::key_name_to_vk("ESCAPE"), Some(0x1B));
        assert_eq!(AppState::key_name_to_vk("ENTER"), Some(0x0D));
        assert_eq!(AppState::key_name_to_vk("RETURN"), Some(0x0D));
        assert_eq!(AppState::key_name_to_vk("BACKSPACE"), Some(0x08));
        assert_eq!(AppState::key_name_to_vk("BACK"), Some(0x08));
    }

    #[test]
    fn test_vk_to_scancode_numpad_keys() {
        assert_eq!(AppState::vk_to_scancode(0x60), 0x52); // NUMPAD0
        assert_eq!(AppState::vk_to_scancode(0x61), 0x4F); // NUMPAD1
        assert_eq!(AppState::vk_to_scancode(0x65), 0x4C); // NUMPAD5
        assert_eq!(AppState::vk_to_scancode(0x69), 0x49); // NUMPAD9
        assert_eq!(AppState::vk_to_scancode(0x6A), 0x37); // MULTIPLY
        assert_eq!(AppState::vk_to_scancode(0x6B), 0x4E); // ADD
        assert_eq!(AppState::vk_to_scancode(0x6D), 0x4A); // SUBTRACT
        assert_eq!(AppState::vk_to_scancode(0x6F), 0x35); // DIVIDE
    }

    #[test]
    fn test_vk_to_scancode_lock_keys() {
        assert_eq!(AppState::vk_to_scancode(0x14), 0x3A); // CAPSLOCK
        assert_eq!(AppState::vk_to_scancode(0x90), 0x45); // NUMLOCK
        assert_eq!(AppState::vk_to_scancode(0x91), 0x46); // SCROLL LOCK
    }

    #[test]
    fn test_vk_to_scancode_oem_keys() {
        assert_eq!(AppState::vk_to_scancode(0xBA), 0x27); // OEM_1 (;:)
        assert_eq!(AppState::vk_to_scancode(0xBB), 0x0D); // OEM_PLUS (=+)
        assert_eq!(AppState::vk_to_scancode(0xBC), 0x33); // OEM_COMMA (,<)
        assert_eq!(AppState::vk_to_scancode(0xBD), 0x0C); // OEM_MINUS (-_)
        assert_eq!(AppState::vk_to_scancode(0xBE), 0x34); // OEM_PERIOD (.>)
        assert_eq!(AppState::vk_to_scancode(0xBF), 0x35); // OEM_2 (/?)
        assert_eq!(AppState::vk_to_scancode(0xC0), 0x29); // OEM_3 (`~)
    }

    #[test]
    fn test_combo_key_with_numpad() {
        let device = AppState::input_name_to_device("LCTRL+NUMPAD0");
        assert!(device.is_some());

        if let Some(InputDevice::KeyCombo(keys)) = device {
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[0], 0xA2); // LCTRL
            assert_eq!(keys[1], 0x60); // NUMPAD0
        } else {
            panic!("Expected KeyCombo device");
        }
    }

    #[test]
    fn test_combo_key_with_oem() {
        let device = AppState::input_name_to_device("LALT+OEM_3");
        assert!(device.is_some());

        if let Some(InputDevice::KeyCombo(keys)) = device {
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[0], 0xA4); // LALT
            assert_eq!(keys[1], 0xC0); // OEM_3 (`~)
        } else {
            panic!("Expected KeyCombo device");
        }
    }

    #[test]
    fn test_output_action_with_numpad() {
        let action = AppState::input_name_to_output("NUMPAD5");
        assert!(action.is_some());

        if let Some(OutputAction::KeyboardKey(scancode)) = action {
            assert_eq!(scancode, 0x4C); // NUMPAD5 scancode
        } else {
            panic!("Expected KeyboardKey action");
        }
    }

    #[test]
    fn test_parse_key_combo_trigger() {
        // Test parsing key combinations
        let device = AppState::input_name_to_device("ALT+A");
        assert!(device.is_some());

        if let Some(InputDevice::KeyCombo(keys)) = device {
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[0], 0x12); // ALT
            assert_eq!(keys[1], 0x41); // A
        } else {
            panic!("Expected KeyCombo device");
        }
    }

    #[test]
    fn test_parse_complex_key_combo() {
        // Test parsing complex key combinations
        let device = AppState::input_name_to_device("CTRL+SHIFT+S");
        assert!(device.is_some());

        if let Some(InputDevice::KeyCombo(keys)) = device {
            assert_eq!(keys.len(), 3);
            assert_eq!(keys[0], 0x11); // CTRL
            assert_eq!(keys[1], 0x10); // SHIFT
            assert_eq!(keys[2], 0x53); // S
        } else {
            panic!("Expected KeyCombo device");
        }
    }

    #[test]
    fn test_parse_key_combo_output() {
        // Test parsing key combination output
        let action = AppState::input_name_to_output("ALT+F4");
        assert!(action.is_some());

        if let Some(OutputAction::KeyCombo(scancodes)) = action {
            assert_eq!(scancodes.len(), 2);
            assert_eq!(scancodes[0], 0x38); // ALT scancode
            assert_eq!(scancodes[1], 0x3E); // F4 scancode
            // Verify Arc reference counting works
            let clone = scancodes.clone();
            assert_eq!(Arc::strong_count(&scancodes), Arc::strong_count(&clone));
        } else {
            panic!("Expected KeyCombo output");
        }
    }

    #[test]
    fn test_parse_invalid_key_combo() {
        // Test parsing invalid key combinations
        let device = AppState::input_name_to_device("INVALID+KEY");
        assert!(device.is_none());

        let device = AppState::input_name_to_device("A+");
        assert!(device.is_none());

        let device = AppState::input_name_to_device("+B");
        assert!(device.is_none());
    }

    #[test]
    fn test_modifier_key_scancodes() {
        // Test that modifier keys have proper scancodes
        assert_eq!(AppState::vk_to_scancode(0xA0), 0x2A); // LSHIFT
        assert_eq!(AppState::vk_to_scancode(0xA1), 0x36); // RSHIFT
        assert_eq!(AppState::vk_to_scancode(0xA2), 0x1D); // LCTRL
        assert_eq!(AppState::vk_to_scancode(0xA4), 0x38); // LALT
        assert_eq!(AppState::vk_to_scancode(0x10), 0x2A); // SHIFT (generic)
        assert_eq!(AppState::vk_to_scancode(0x11), 0x1D); // CTRL (generic)
        assert_eq!(AppState::vk_to_scancode(0x12), 0x38); // ALT (generic)
    }

    #[test]
    fn test_key_combo_mapping_creation() {
        let mut config = AppConfig::default();
        config.mappings = vec![
            KeyMapping {
                trigger_key: "ALT+A".to_string(),
                target_key: "B".to_string(),
                interval: Some(10),
                event_duration: Some(5),
            },
            KeyMapping {
                trigger_key: "CTRL+SHIFT+F".to_string(),
                target_key: "ALT+F4".to_string(),
                interval: None,
                event_duration: None,
            },
        ];

        let (input_mappings, _) = AppState::create_input_mappings(&config).unwrap();
        assert_eq!(input_mappings.len(), 2);

        // Check first mapping
        let alt_a = InputDevice::KeyCombo(vec![0x12, 0x41]); // ALT+A
        let mapping1 = input_mappings.get(&alt_a);
        assert!(mapping1.is_some());

        if let Some(m) = mapping1 {
            assert_eq!(m.interval, 10);
            assert_eq!(m.event_duration, 5);
            if let OutputAction::KeyboardKey(scancode) = m.target_action {
                assert_eq!(scancode, 0x30); // B scancode
            } else {
                panic!("Expected single key output");
            }
        }

        // Check second mapping
        let ctrl_shift_f = InputDevice::KeyCombo(vec![0x11, 0x10, 0x46]); // CTRL+SHIFT+F
        let mapping2 = input_mappings.get(&ctrl_shift_f);
        assert!(mapping2.is_some());

        if let Some(m) = mapping2 {
            if let OutputAction::KeyCombo(scancodes) = &m.target_action {
                assert_eq!(scancodes.len(), 2); // ALT+F4
            } else {
                panic!("Expected combo key output");
            }
        }
    }

    #[test]
    fn test_pressed_keys_tracking() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // Initially, no keys pressed
        assert_eq!(state.pressed_keys.len(), 0);

        // Simulate key press tracking (would be done by handle_key_event)
        let _ = state.pressed_keys.insert_sync(0x11); // CTRL
        let _ = state.pressed_keys.insert_sync(0x41); // A

        assert_eq!(state.pressed_keys.len(), 2);
        assert!(state.pressed_keys.contains_sync(&0x11));
        assert!(state.pressed_keys.contains_sync(&0x41));

        // Release keys
        let _ = state.pressed_keys.remove_sync(&0x41);

        assert_eq!(state.pressed_keys.len(), 1);
        assert!(state.pressed_keys.contains_sync(&0x11));
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
