//! Application state management.
//!
//! Provides centralized state management for the key remapping application,
//! including configuration, keyboard event handling, and process filtering.

use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use scc::{AtomicShared, Guard, Shared, Tag};

use smallvec::SmallVec;

use crate::util::{likely, unlikely};

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PWSTR;

use crate::config::AppConfig;
use crate::i18n::Language;

/// HID device activation request information.
#[derive(Debug, Clone)]
pub struct HidActivationRequest {
    pub device_handle: isize,
    pub device_name: String,
    pub vid: u16,
    pub pid: u16,
    pub usage_page: u16,
    pub usage: u16,
}

/// Trait for dispatching input events to worker threads.
pub trait EventDispatcher: Send + Sync {
    fn dispatch(&self, event: InputEvent);
    /// Clear internal caches (called when configuration is reloaded)
    fn clear_cache(&self);
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
    /// XInput gamepad combo
    /// Format: (device_type, button_ids)
    XInputCombo {
        device_type: DeviceType,
        button_ids: Vec<u32>,
    },
    /// Generic device input for gamepads, joysticks, and other HID devices
    /// Format: (device_type, button_id)
    GenericDevice {
        device_type: DeviceType,
        button_id: u64,
    },
}

impl std::fmt::Display for InputDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputDevice::XInputCombo {
                device_type,
                button_ids,
            } => {
                let vid = match device_type {
                    DeviceType::Gamepad(vid) => *vid,
                    DeviceType::Joystick(vid) => *vid,
                    DeviceType::HidDevice { .. } => 0,
                };

                let prefix = match device_type {
                    DeviceType::Gamepad(_) => "GAMEPAD",
                    DeviceType::Joystick(_) => "JOYSTICK",
                    _ => "XINPUT",
                };

                write!(f, "{}_{:04X}_", prefix, vid)?;

                for (i, &input_id) in button_ids.iter().enumerate() {
                    if i > 0 {
                        write!(f, "+")?;
                    }
                    write!(
                        f,
                        "{}",
                        crate::xinput::XInputHandler::input_id_to_name(input_id)
                    )?;
                }

                Ok(())
            }
            InputDevice::GenericDevice {
                device_type,
                button_id,
            } => {
                // HID device format: [32-bit stable_device_id][32-bit position]
                let stable_device_id = (button_id >> 32) as u32;
                let position = (button_id & 0xFFFFFFFF) as u32;

                let display_info_opt =
                    crate::rawinput::get_device_display_info(stable_device_id as u64);

                let display_info = if let Some(info) = display_info_opt {
                    info
                } else {
                    let vid = match device_type {
                        DeviceType::Gamepad(vid) => *vid,
                        DeviceType::Joystick(vid) => *vid,
                        DeviceType::HidDevice { .. } => 0,
                    };
                    crate::rawinput::DeviceDisplayInfo {
                        vendor_id: vid,
                        product_id: 0,
                        serial_number: None,
                    }
                };

                let prefix = match device_type {
                    DeviceType::Gamepad(_) => "GAMEPAD",
                    DeviceType::Joystick(_) => "JOYSTICK",
                    DeviceType::HidDevice { usage_page, .. } => {
                        return if let Some(ref serial) = display_info.serial_number {
                            write!(
                                f,
                                "HID_{:04X}_{:04X}_{:04X}_{}",
                                usage_page, display_info.vendor_id, display_info.product_id, serial
                            )
                        } else {
                            write!(
                                f,
                                "HID_{:04X}_{:04X}_{:04X}_DEV{:08X}",
                                usage_page,
                                display_info.vendor_id,
                                display_info.product_id,
                                stable_device_id
                            )
                        };
                    }
                };

                // Format with VID/PID/Serial or VID/PID/DEV
                if let Some(ref serial) = display_info.serial_number {
                    // Has serial number: format with serial
                    if position & 0x80000000 != 0 {
                        // Byte-level
                        let byte_idx = position & 0x7FFFFFFF;
                        write!(
                            f,
                            "{}_{:04X}_{:04X}_{}_B{}",
                            prefix,
                            display_info.vendor_id,
                            display_info.product_id,
                            serial,
                            byte_idx
                        )
                    } else {
                        // Bit-level
                        let byte_idx = (position >> 16) as u16;
                        let bit_idx = (position & 0xFFFF) as u16;
                        write!(
                            f,
                            "{}_{:04X}_{:04X}_{}_B{}.{}",
                            prefix,
                            display_info.vendor_id,
                            display_info.product_id,
                            serial,
                            byte_idx,
                            bit_idx
                        )
                    }
                } else {
                    // No serial number: format with DEV prefix
                    if position & 0x80000000 != 0 {
                        // Byte-level
                        let byte_idx = position & 0x7FFFFFFF;
                        write!(
                            f,
                            "{}_{:04X}_{:04X}_DEV{:08X}_B{}",
                            prefix,
                            display_info.vendor_id,
                            display_info.product_id,
                            stable_device_id,
                            byte_idx
                        )
                    } else {
                        // Bit-level
                        let byte_idx = (position >> 16) as u16;
                        let bit_idx = (position & 0xFFFF) as u16;
                        write!(
                            f,
                            "{}_{:04X}_{:04X}_DEV{:08X}_B{}.{}",
                            prefix,
                            display_info.vendor_id,
                            display_info.product_id,
                            stable_device_id,
                            byte_idx,
                            bit_idx
                        )
                    }
                }
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Device type classification for generic input devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// HID gamepad (Xbox, PlayStation, etc.)
    Gamepad(u16),
    /// Joystick device
    Joystick(u16),
    /// Custom HID device with usage page and usage
    HidDevice { usage_page: u16, usage: u16 },
}

/// HID input capture mode strategy.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum CaptureMode {
    /// Captures the most sustained input pattern (default)
    #[default]
    MostSustained,
    /// Adaptive scoring based on encoding type detection
    AdaptiveIntelligent,
    /// Selects pattern with most changed bits
    MaxChangedBits,
    /// Selects pattern with most set bits
    MaxSetBits,
    /// Selects the last stable frame
    LastStable,
    /// For Hat Switch devices (prioritizes numeric value)
    HatSwitchOptimized,
    /// For analog devices (prioritizes deviation magnitude)
    AnalogOptimized,
}

impl FromStr for CaptureMode {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MostSustained" => Ok(Self::MostSustained),
            "AdaptiveIntelligent" => Ok(Self::AdaptiveIntelligent),
            "MaxChangedBits" => Ok(Self::MaxChangedBits),
            "MaxSetBits" => Ok(Self::MaxSetBits),
            "LastStable" => Ok(Self::LastStable),
            "HatSwitchOptimized" => Ok(Self::HatSwitchOptimized),
            "AnalogOptimized" => Ok(Self::AnalogOptimized),
            _ => Ok(Self::default()),
        }
    }
}

impl CaptureMode {
    pub fn all_modes() -> &'static [CaptureMode] {
        &[
            CaptureMode::MostSustained,
            CaptureMode::AdaptiveIntelligent,
            CaptureMode::MaxChangedBits,
            CaptureMode::MaxSetBits,
            CaptureMode::LastStable,
            CaptureMode::HatSwitchOptimized,
            CaptureMode::AnalogOptimized,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MostSustained => "MostSustained",
            Self::AdaptiveIntelligent => "AdaptiveIntelligent",
            Self::MaxChangedBits => "MaxChangedBits",
            Self::MaxSetBits => "MaxSetBits",
            Self::LastStable => "LastStable",
            Self::HatSwitchOptimized => "HatSwitchOptimized",
            Self::AnalogOptimized => "AnalogOptimized",
        }
    }
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

/// Mouse movement direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseMoveDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

/// Mouse scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseScrollDirection {
    Up,
    Down,
}

/// Output action type for input mapping.
#[derive(Debug, Clone)]
pub enum OutputAction {
    /// Keyboard key output with scancode
    KeyboardKey(u16),
    /// Mouse button output
    MouseButton(MouseButton),
    /// Mouse movement output (direction, speed in pixels per move)
    MouseMove(MouseMoveDirection, i32),
    /// Mouse scroll output (direction, wheel delta)
    MouseScroll(MouseScrollDirection, i32),
    /// Key combination output (modifier scancodes + main key scancode)
    /// Format: [modifier1_scancode, modifier2_scancode, ..., main_key_scancode]
    /// Using Arc to avoid cloning on every key repeat
    KeyCombo(Arc<[u16]>),
    /// Multiple simultaneous actions for handling combined inputs
    /// Uses SmallVec with inline capacity of 4 to reduce allocations
    MultipleActions(Arc<SmallVec<[OutputAction; 4]>>),
}

/// Configuration for a single input mapping.
#[derive(Debug, Clone)]
pub struct InputMappingInfo {
    /// Target output action
    pub target_action: OutputAction,
    /// Repeat interval in milliseconds
    pub interval: u64,
    /// Event duration in milliseconds (not used for MouseMove)
    pub event_duration: u64,
    /// Enable turbo mode (auto-repeat)
    pub turbo_enabled: bool,
}

/// Process information with timestamp for caching
#[derive(Debug, Clone)]
struct ProcessInfo {
    name: Option<String>,
    timestamp: Instant,
}

/// Cache for switch key detection with lock-free fast paths
pub struct SwitchKeyCache {
    pub keyboard_vk: AtomicU32,
    pub xinput_button_mask: AtomicU32,
    pub xinput_device_hash: AtomicU32,
    pub generic_button_id: AtomicU64,
    pub full_device: AtomicShared<InputDevice>,
}

impl SwitchKeyCache {
    #[inline(always)]
    fn new() -> Self {
        Self {
            keyboard_vk: AtomicU32::new(0),
            xinput_button_mask: AtomicU32::new(0),
            xinput_device_hash: AtomicU32::new(0),
            generic_button_id: AtomicU64::new(0),
            full_device: AtomicShared::null(),
        }
    }

    #[inline(always)]
    fn clear(&self) {
        self.keyboard_vk.store(0, Ordering::Relaxed);
        self.xinput_button_mask.store(0, Ordering::Relaxed);
        self.xinput_device_hash.store(0, Ordering::Relaxed);
        self.generic_button_id.store(0, Ordering::Relaxed);
        let _ = self.full_device.swap((None, Tag::None), Ordering::Relaxed);
    }
}

/// Central application state manager.
///
/// Manages all runtime state including configuration, key mappings,
/// worker threads, and process filtering.
pub struct AppState {
    /// Current UI language
    language: AtomicU8,
    /// Tray icon visibility flag
    show_tray_icon: AtomicBool,
    /// Notification display flag
    show_notifications: AtomicBool,
    /// Switch key cache for fast combo key detection
    pub switch_key_cache: SwitchKeyCache,
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
    input_mappings: scc::HashMap<InputDevice, InputMappingInfo>,
    /// Worker pool for event processing
    worker_pool: OnceLock<Arc<dyn EventDispatcher>>,
    /// Notification event sender
    notification_sender: OnceLock<Sender<NotificationEvent>>,
    /// Process whitelist (empty means all processes enabled)
    process_whitelist: AtomicShared<Vec<String>>,
    /// Cached foreground process name with timestamp
    cached_process_info: AtomicShared<ProcessInfo>,
    /// Currently pressed keys for combo detection
    pressed_keys: scc::HashSet<u32>,
    /// Active combo triggers (multiple combos can be active simultaneously)
    /// Maps combo device to the set of modifier keys that were suppressed
    active_combo_triggers: scc::HashMap<InputDevice, SmallVec<[u32; 8]>>,
    /// Cached turbo state for keyboard keys (VK 0-255) for fast access
    cached_turbo_keyboard: [AtomicBool; 256],
    /// Cached turbo state for mouse buttons and key combos
    cached_turbo_other: scc::HashMap<InputDevice, bool>,
    /// Cached combo key index for efficient combo matching
    /// Maps main key to all combos that end with that key
    cached_combo_index: scc::HashMap<u32, Vec<InputDevice>>,
    /// Cached XInput combo index for subset matching
    /// Maps device_type to all combo button_ids for that device
    cached_xinput_combos: scc::HashMap<DeviceType, Vec<Vec<u32>>>,
    /// Raw Input capture event sender for GUI
    raw_input_capture_sender: Sender<InputDevice>,
    /// Raw Input capture event receiver for GUI
    raw_input_capture_receiver: Receiver<InputDevice>,
    /// Flag indicating GUI is in capture mode for Raw Input
    is_capturing_raw_input: AtomicBool,
    /// Raw Input capture mode strategy
    rawinput_capture_mode: AtomicShared<CaptureMode>,
    /// XInput capture mode strategy
    xinput_capture_mode: AtomicShared<crate::config::XInputCaptureMode>,
    /// HID device activation request sender
    hid_activation_sender: Sender<HidActivationRequest>,
    /// HID device activation request receiver
    hid_activation_receiver: Receiver<HidActivationRequest>,
    /// HID activation data sender (device_handle, data)
    hid_activation_data_sender: Sender<(isize, Vec<u8>)>,
    /// HID activation data receiver
    hid_activation_data_receiver: Receiver<(isize, Vec<u8>)>,
    /// Currently activating device handle
    activating_device_handle: std::sync::atomic::AtomicIsize,
    /// XInput cache invalidation flag
    xinput_cache_invalid: AtomicBool,
}

impl AppState {
    /// Creates a new application state from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the toggle key name is invalid or key mappings cannot be created.
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let switch_key_cache = SwitchKeyCache::new();
        Self::update_switch_key_cache(&switch_key_cache, &config.switch_key)?;

        let input_mappings_map = Self::create_input_mappings(&config)?;

        // Create lock-free concurrent HashMap
        let input_mappings = scc::HashMap::new();

        // Populate input_mappings
        for (k, v) in input_mappings_map {
            let _ = input_mappings.insert_sync(k, v);
        }

        // Initialize cached data structures
        let cached_turbo_keyboard: [AtomicBool; 256] =
            std::array::from_fn(|_| AtomicBool::new(true));
        let cached_turbo_other = scc::HashMap::new();
        let cached_combo_index: scc::HashMap<u32, Vec<InputDevice>> = scc::HashMap::new();
        let cached_xinput_combos: scc::HashMap<DeviceType, Vec<Vec<u32>>> = scc::HashMap::new();

        for mapping in config.mappings.iter() {
            if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                // Populate turbo cache and combo index
                match &device {
                    InputDevice::Keyboard(vk) if *vk < 256 => {
                        cached_turbo_keyboard[*vk as usize]
                            .store(mapping.turbo_enabled, Ordering::Relaxed);
                    }
                    InputDevice::KeyCombo(keys) => {
                        if let Some(&last_key) = keys.last() {
                            let mut combos = cached_combo_index
                                .get_sync(&last_key)
                                .map(|v| v.get().clone())
                                .unwrap_or_default();
                            combos.push(device.clone());
                            let _ = cached_combo_index.upsert_sync(last_key, combos);
                        }
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                    InputDevice::XInputCombo {
                        device_type,
                        button_ids,
                    } => {
                        let mut combos = cached_xinput_combos
                            .get_sync(device_type)
                            .map(|v| v.get().clone())
                            .unwrap_or_default();
                        combos.push(button_ids.clone());
                        let _ = cached_xinput_combos.upsert_sync(*device_type, combos);
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                    _ => {
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                }
            }
        }

        let (raw_input_capture_sender, raw_input_capture_receiver) = crossbeam_channel::unbounded();
        let (hid_activation_sender, hid_activation_receiver) = crossbeam_channel::unbounded();
        let (hid_activation_data_sender, hid_activation_data_receiver) =
            crossbeam_channel::unbounded();

        Ok(Self {
            language: AtomicU8::new(config.language.to_u8()),
            show_tray_icon: AtomicBool::new(config.show_tray_icon),
            show_notifications: AtomicBool::new(config.show_notifications),
            switch_key_cache,
            should_exit: Arc::new(AtomicBool::new(false)),
            is_paused: AtomicBool::new(false),
            show_window_requested: AtomicBool::new(false),
            show_about_requested: AtomicBool::new(false),
            input_timeout: AtomicU64::new(config.input_timeout),
            worker_count: AtomicU64::new(0),
            process_whitelist: AtomicShared::from(Shared::new(config.process_whitelist.clone())),
            configured_worker_count: config.worker_count,
            input_mappings,
            worker_pool: OnceLock::new(),
            notification_sender: OnceLock::new(),
            cached_process_info: AtomicShared::from(Shared::new(ProcessInfo {
                name: None,
                timestamp: Instant::now(),
            })),
            pressed_keys: scc::HashSet::new(),
            active_combo_triggers: scc::HashMap::new(),
            cached_turbo_keyboard,
            cached_turbo_other,
            cached_combo_index,
            cached_xinput_combos,
            raw_input_capture_sender,
            raw_input_capture_receiver,
            is_capturing_raw_input: AtomicBool::new(false),
            rawinput_capture_mode: AtomicShared::from(Shared::new(
                CaptureMode::from_str(&config.rawinput_capture_mode).unwrap(),
            )),
            xinput_capture_mode: AtomicShared::from(Shared::new(
                crate::config::XInputCaptureMode::from_str(&config.xinput_capture_mode)?,
            )),
            hid_activation_sender,
            hid_activation_receiver,
            hid_activation_data_sender,
            hid_activation_data_receiver,
            activating_device_handle: std::sync::atomic::AtomicIsize::new(-1),
            xinput_cache_invalid: AtomicBool::new(false),
        })
    }

    /// Reloads configuration at runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the new toggle key is invalid or mappings cannot be created.
    pub fn reload_config(&self, config: AppConfig) -> anyhow::Result<()> {
        Self::update_switch_key_cache(&self.switch_key_cache, &config.switch_key)?;

        // Update language
        self.language
            .store(config.language.to_u8(), Ordering::Relaxed);

        // Update show_tray_icon and show_notifications
        self.show_tray_icon
            .store(config.show_tray_icon, Ordering::Relaxed);
        self.show_notifications
            .store(config.show_notifications, Ordering::Relaxed);

        // Update input timeout
        self.input_timeout
            .store(config.input_timeout, Ordering::Relaxed);

        // Update capture modes
        let new_rawinput_mode =
            Shared::new(CaptureMode::from_str(&config.rawinput_capture_mode).unwrap());
        let _ = self
            .rawinput_capture_mode
            .swap((Some(new_rawinput_mode), Tag::None), Ordering::Release);

        let new_xinput_mode = Shared::new(crate::config::XInputCaptureMode::from_str(
            &config.xinput_capture_mode,
        )?);
        let _ = self
            .xinput_capture_mode
            .swap((Some(new_xinput_mode), Tag::None), Ordering::Release);

        // Update mappings (lock-free concurrent HashMap)
        let new_input_mappings = Self::create_input_mappings(&config)?;
        self.input_mappings.clear_sync();
        for (k, v) in new_input_mappings {
            let _ = self.input_mappings.insert_sync(k, v);
        }

        // Update cached data structures
        self.cached_turbo_other.clear_sync();
        self.cached_combo_index.clear_sync();
        self.cached_xinput_combos.clear_sync();

        // Reset keyboard turbo cache to default
        for i in 0..256 {
            self.cached_turbo_keyboard[i].store(true, Ordering::Relaxed);
        }

        for mapping in config.mappings.iter() {
            if let Some(device) = Self::input_name_to_device(&mapping.trigger_key) {
                // Update turbo cache and combo index
                match &device {
                    InputDevice::Keyboard(vk) if *vk < 256 => {
                        self.cached_turbo_keyboard[*vk as usize]
                            .store(mapping.turbo_enabled, Ordering::Relaxed);
                    }
                    InputDevice::KeyCombo(keys) => {
                        if let Some(&last_key) = keys.last() {
                            let mut combos = self
                                .cached_combo_index
                                .get_sync(&last_key)
                                .map(|v| v.clone())
                                .unwrap_or_default();
                            combos.push(device.clone());

                            let _ = self.cached_combo_index.upsert_sync(last_key, combos);
                        }
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                    InputDevice::XInputCombo {
                        device_type,
                        button_ids,
                    } => {
                        let mut combos = self
                            .cached_xinput_combos
                            .get_sync(device_type)
                            .map(|v| v.get().clone())
                            .unwrap_or_default();
                        combos.push(button_ids.clone());
                        let _ = self.cached_xinput_combos.upsert_sync(*device_type, combos);
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                    _ => {
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                }
            }
        }

        // Update process whitelist
        let new_whitelist = Shared::new(config.process_whitelist.clone());
        let _ = self
            .process_whitelist
            .swap((Some(new_whitelist), Tag::None), Ordering::Release);

        // Clear process name cache
        let new_cache = Shared::new(ProcessInfo {
            name: None,
            timestamp: Instant::now(),
        });
        let _ = self
            .cached_process_info
            .swap((Some(new_cache), Tag::None), Ordering::Release);

        // Clear pressed keys and active combos (lock-free concurrent structures)
        self.pressed_keys.clear_sync();
        self.active_combo_triggers.clear_sync();

        // Clear worker pool caches (mouse action cache, etc.)
        if let Some(pool) = self.worker_pool.get() {
            pool.clear_cache();
        }

        // Signal XInput to invalidate device-level caches
        self.xinput_cache_invalid.store(true, Ordering::Release);

        Ok(())
    }

    /// Sets the worker pool for event dispatching.
    pub fn set_worker_pool(&self, pool: Arc<dyn EventDispatcher>) {
        let _ = self.worker_pool.set(pool);
    }

    /// Gets the worker pool for event dispatching.
    pub fn get_worker_pool(&self) -> Option<&Arc<dyn EventDispatcher>> {
        self.worker_pool.get()
    }

    /// Gets the Raw Input capture sender for GUI key capture mode.
    pub fn get_raw_input_capture_sender(&self) -> &Sender<InputDevice> {
        &self.raw_input_capture_sender
    }

    /// Checks and resets XInput cache invalidation flag.
    #[inline(always)]
    pub fn check_and_reset_xinput_cache_invalid(&self) -> bool {
        self.xinput_cache_invalid.swap(false, Ordering::Acquire)
    }

    /// Gets the current Raw Input capture mode.
    pub fn get_rawinput_capture_mode(&self) -> CaptureMode {
        let guard = Guard::new();
        self.rawinput_capture_mode
            .load(Ordering::Acquire, &guard)
            .as_ref()
            .map(|mode| *mode)
            .unwrap_or_default()
    }

    /// Gets the current XInput capture mode.
    pub fn get_xinput_capture_mode(&self) -> crate::config::XInputCaptureMode {
        let guard = Guard::new();
        self.xinput_capture_mode
            .load(Ordering::Acquire, &guard)
            .as_ref()
            .map(|mode| *mode)
            .unwrap_or_default()
    }

    /// Tries to receive a captured Raw Input event (non-blocking).
    pub fn try_recv_raw_input_capture(&self) -> Option<InputDevice> {
        self.raw_input_capture_receiver.try_recv().ok()
    }

    /// Sets the Raw Input capture mode flag.
    pub fn set_raw_input_capture_mode(&self, enabled: bool) {
        self.is_capturing_raw_input
            .store(enabled, Ordering::Relaxed);

        // When entering capture mode, clear all caches to ensure fresh state
        if enabled {
            // Clear all old events in the channel to avoid showing stale captures
            while self.raw_input_capture_receiver.try_recv().is_ok() {}

            // Clear device display info cache to avoid showing stale device info
            crate::rawinput::clear_device_display_info_cache();

            // Reset HID device states to baseline for clean button detection
            crate::rawinput::reset_hid_device_states();
        }
    }

    /// Clears activation baseline for device.
    #[inline]
    pub fn clear_device_baseline(&self, vid: u16, pid: u16) {
        crate::rawinput::clear_device_baseline(vid, pid);
    }

    /// Checks if Raw Input capture mode is active.
    pub fn is_raw_input_capture_active(&self) -> bool {
        self.is_capturing_raw_input.load(Ordering::Relaxed)
    }

    /// Sends HID device activation request.
    #[inline]
    pub fn request_hid_activation(&self, request: HidActivationRequest) {
        self.activating_device_handle
            .store(request.device_handle, Ordering::Relaxed);
        let _ = self.hid_activation_sender.send(request);
    }

    /// Polls for HID device activation requests
    pub fn poll_hid_activation_requests(&self) -> SmallVec<[HidActivationRequest; 2]> {
        let mut requests = SmallVec::new();
        while let Ok(req) = self.hid_activation_receiver.try_recv() {
            requests.push(req);
        }
        requests
    }

    /// Sends HID activation data during activation process.
    #[inline]
    pub fn send_hid_activation_data(&self, device_handle: isize, data: Vec<u8>) {
        let _ = self.hid_activation_data_sender.send((device_handle, data));
    }

    /// Tries to receive HID activation data for a specific device.
    pub fn try_recv_hid_activation_data(&self, device_handle: isize) -> Option<Vec<u8>> {
        // Consume all messages until we find one for this device
        // or channel is empty
        while let Ok((handle, data)) = self.hid_activation_data_receiver.try_recv() {
            if handle == device_handle {
                return Some(data);
            }
            // Discard data for other devices (shouldn't happen in normal operation)
        }
        None
    }

    /// Checks if a device is currently being activated.
    #[inline(always)]
    pub fn is_device_activating(&self, device_handle: isize) -> bool {
        self.activating_device_handle.load(Ordering::Relaxed) == device_handle
    }

    /// Clears the activating device handle.
    #[inline]
    pub fn clear_activating_device(&self) {
        self.activating_device_handle.store(-1, Ordering::Relaxed);
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

    /// Checks if the application should exit (hot path - inlined)
    #[inline(always)]
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    /// Toggles pause state and returns the previous state.
    pub fn toggle_paused(&self) -> bool {
        self.is_paused.fetch_xor(true, Ordering::Relaxed)
    }

    /// Returns the current pause state (hot path - inlined)
    #[inline(always)]
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

    /// Returns the current UI language.
    #[inline(always)]
    pub fn language(&self) -> Language {
        Language::from_u8(self.language.load(Ordering::Relaxed))
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

    /// Fast mapping lookup using lock-free read
    #[inline(always)]
    pub fn get_input_mapping(&self, device: &InputDevice) -> Option<InputMappingInfo> {
        self.input_mappings.read_sync(device, |_, v| v.clone())
    }

    /// Gets all XInputCombo button_ids for a specific device type
    /// Used for subset matching in runtime (cached for performance)
    #[inline(always)]
    pub fn get_xinput_combos_for_device(&self, device_type: &DeviceType) -> Vec<Vec<u32>> {
        self.cached_xinput_combos
            .get_sync(device_type)
            .map(|v| v.get().clone())
            .unwrap_or_default()
    }

    /// Check turbo_enabled state from cache (hot path)
    #[inline(always)]
    fn is_turbo_enabled(&self, device: &InputDevice) -> bool {
        match device {
            InputDevice::Keyboard(vk) if *vk < 256 => {
                self.cached_turbo_keyboard[*vk as usize].load(Ordering::Relaxed)
            }
            _ => self
                .cached_turbo_other
                .read_sync(device, |_, v| *v)
                .unwrap_or(true),
        }
    }

    #[inline]
    pub fn handle_switch_key_toggle(&self) {
        let was_paused = self.toggle_paused();
        self.active_combo_triggers.clear_sync();

        if let Some(sender) = self.notification_sender.get() {
            let msg = if was_paused {
                "Sorahk activating".to_string()
            } else {
                "Sorahk paused".to_string()
            };
            let _ = sender.send(NotificationEvent::Info(msg));
        }
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

    /// Check if this key is the main key (last key) of an active combo with turbo disabled
    /// Used to allow Windows repeat behavior for turbo-disabled combos
    #[inline]
    fn is_main_key_in_active_combo_no_turbo(&self, vk_code: u32) -> bool {
        let mut result = false;
        self.active_combo_triggers.iter_sync(|combo_device, _| {
            if result {
                return false;
            }
            if let InputDevice::KeyCombo(keys) = combo_device {
                // Check if this is the main key (last key in combo)
                if let Some(&last_key) = keys.last()
                    && last_key == vk_code
                    && !self.is_turbo_enabled(combo_device)
                {
                    result = true;
                }
            }
            true
        });
        result
    }

    /// Add a combo to active triggers
    fn add_active_combo(&self, combo: InputDevice, modifiers: SmallVec<[u32; 8]>) {
        let _ = self.active_combo_triggers.insert_sync(combo, modifiers);
    }

    /// Check if a specific combo is active
    #[inline(always)]
    fn is_combo_active(&self, combo: &InputDevice) -> bool {
        self.active_combo_triggers.contains_sync(combo)
    }

    /// Release modifiers once (send KEYUP events using scancodes)
    /// Skips modifiers already suppressed by other active combos
    fn release_modifiers_once(&self, modifiers: &SmallVec<[u32; 8]>) {
        if modifiers.is_empty() {
            return;
        }

        // Check which modifiers are already suppressed by active combos
        let mut already_suppressed: SmallVec<[u32; 8]> = SmallVec::new();
        self.active_combo_triggers.iter_sync(|_, modifiers| {
            already_suppressed.extend_from_slice(modifiers);
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
    fn cleanup_released_combos(&self) -> SmallVec<[InputDevice; 4]> {
        let mut pressed: SmallVec<[u32; 16]> = SmallVec::new();
        self.pressed_keys.iter_sync(|&k| {
            pressed.push(k);
            true
        });

        let mut to_remove: SmallVec<[InputDevice; 4]> = SmallVec::new();

        self.active_combo_triggers.iter_sync(|combo_device, _| {
            if let InputDevice::KeyCombo(keys) = combo_device {
                // Fast linear scan — optimal for small N
                if !keys.iter().all(|&k| pressed.contains(&k)) {
                    to_remove.push(combo_device.clone());
                }
            }
            true
        });

        let mut removed = SmallVec::new();
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

    #[inline]
    pub fn simulate_action(&self, action: OutputAction, duration: u64) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut press_flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(scancode) {
                        press_flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    // Press the key
                    let mut input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: scancode,
                                dwFlags: press_flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Release the key
                    input.Anonymous.ki.dwFlags = press_flags | KEYEVENTF_KEYUP;
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
                OutputAction::MouseMove(direction, speed) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let (dx, dy) = match direction {
                        MouseMoveDirection::Up => (0, -speed),
                        MouseMoveDirection::Down => (0, speed),
                        MouseMoveDirection::Left => (-speed, 0),
                        MouseMoveDirection::Right => (speed, 0),
                        MouseMoveDirection::UpLeft => (-speed, -speed),
                        MouseMoveDirection::UpRight => (speed, -speed),
                        MouseMoveDirection::DownLeft => (-speed, speed),
                        MouseMoveDirection::DownRight => (speed, speed),
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx,
                                dy,
                                mouseData: 0,
                                dwFlags: MOUSEEVENTF_MOVE,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseScroll(direction, speed) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    // Direct speed control without multiplier
                    let wheel_delta = match direction {
                        MouseScrollDirection::Up => speed,
                        MouseScrollDirection::Down => -speed,
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: wheel_delta as u32,
                                dwFlags: MOUSEEVENTF_WHEEL,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::KeyCombo(scancodes) => {
                    // Press all keys in sequence (modifiers first, then main key)
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
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
                        // Short delay between keys for better compatibility
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }

                    // Hold duration
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Release all keys in reverse order (main key first, then modifiers)
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
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
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect press events for batch processing
                    let mut press_inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_press_inputs(&actions, &mut press_inputs);
                    if !press_inputs.is_empty() {
                        SendInput(&press_inputs, std::mem::size_of::<INPUT>() as i32);
                    }

                    // Hold duration
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Collect release events for batch processing
                    let mut release_inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_release_inputs(&actions, &mut release_inputs);
                    if !release_inputs.is_empty() {
                        SendInput(&release_inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
        }
    }

    /// Checks if a scancode requires KEYEVENTF_EXTENDEDKEY flag
    #[inline(always)]
    fn is_extended_scancode(scancode: u16) -> bool {
        const EXTENDED_KEYS_BITMAP: u128 = (1u128 << 0x1D)
            | (1u128 << 0x38)
            | (1u128 << 0x47)
            | (1u128 << 0x48)
            | (1u128 << 0x49)
            | (1u128 << 0x4B)
            | (1u128 << 0x4D)
            | (1u128 << 0x4F)
            | (1u128 << 0x50)
            | (1u128 << 0x51)
            | (1u128 << 0x52)
            | (1u128 << 0x53)
            | (1u128 << 0x5B)
            | (1u128 << 0x5C);

        scancode < 128 && (EXTENDED_KEYS_BITMAP & (1u128 << scancode)) != 0
    }

    /// Simulates only the press event for an action
    #[inline(always)]
    pub fn simulate_press(&self, action: &OutputAction) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let down_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
                        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XDOWN,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    let input = INPUT {
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
                }
                OutputAction::MouseMove(_, _) => {
                    // Mouse movement doesn't have press/release states
                }
                OutputAction::MouseScroll(_, _) => {
                    // Mouse scroll doesn't have press/release states
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
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
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect and send all press events in a single call
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_press_inputs(actions, &mut inputs);
                    if !inputs.is_empty() {
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
        }
    }

    /// Simulates only the release event for an action
    #[inline(always)]
    pub fn simulate_release(&self, action: &OutputAction) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let up_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTUP,
                        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XUP,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: up_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseMove(_, _) => {
                    // Mouse movement doesn't have press/release states
                }
                OutputAction::MouseScroll(_, _) => {
                    // Mouse scroll doesn't have press/release states
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
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
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect and send all release events in a single call
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_release_inputs(actions, &mut inputs);
                    if !inputs.is_empty() {
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
        }
    }

    /// Collects press INPUT events from actions into a buffer
    #[inline(always)]
    fn collect_press_inputs(
        &self,
        actions: &SmallVec<[OutputAction; 4]>,
        inputs: &mut SmallVec<[INPUT; 8]>,
    ) {
        for action in actions.iter() {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::MouseButton(button) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let down_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
                        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XDOWN,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    inputs.push(INPUT {
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
                    });
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }

                        inputs.push(INPUT {
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
                        });
                    }
                }
                OutputAction::MultipleActions(nested_actions) => {
                    // Recursively collect nested actions
                    self.collect_press_inputs(nested_actions, inputs);
                }
                OutputAction::MouseMove(_, _) | OutputAction::MouseScroll(_, _) => {
                    // Skip actions without press state
                }
            }
        }
    }

    /// Collects release INPUT events from actions into a buffer
    #[inline(always)]
    fn collect_release_inputs(
        &self,
        actions: &SmallVec<[OutputAction; 4]>,
        inputs: &mut SmallVec<[INPUT; 8]>,
    ) {
        for action in actions.iter() {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::MouseButton(button) => {
                    use windows::Win32::UI::Input::KeyboardAndMouse::*;

                    let up_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTUP,
                        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XUP,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    inputs.push(INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: up_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }

                        inputs.push(INPUT {
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
                        });
                    }
                }
                OutputAction::MultipleActions(nested_actions) => {
                    // Recursively collect nested actions
                    self.collect_release_inputs(nested_actions, inputs);
                }
                OutputAction::MouseMove(_, _) | OutputAction::MouseScroll(_, _) => {
                    // Skip actions without release state
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
    #[inline]
    fn is_process_whitelisted(&self) -> bool {
        let guard = Guard::new();
        let whitelist_ptr = self.process_whitelist.load(Ordering::Acquire, &guard);
        let empty_vec = vec![];
        let whitelist = whitelist_ptr.as_ref().unwrap_or(&empty_vec);

        if whitelist.is_empty() {
            return true;
        }

        const CACHE_DURATION_MS: u64 = 50;
        let now = Instant::now();

        let process_name = {
            let cache_ptr = self.cached_process_info.load(Ordering::Acquire, &guard);
            let cache = cache_ptr.as_ref();

            if let Some(info) = cache {
                if likely(
                    now.duration_since(info.timestamp) < Duration::from_millis(CACHE_DURATION_MS),
                ) {
                    // Cache hit: return cached name
                    info.name.clone()
                } else {
                    // Cache miss: need refresh
                    let new_name = Self::get_foreground_process_name();
                    let new_cache = Shared::new(ProcessInfo {
                        name: new_name.clone(),
                        timestamp: now,
                    });
                    let _ = self
                        .cached_process_info
                        .swap((Some(new_cache), Tag::None), Ordering::Release);
                    new_name
                }
            } else {
                // No cache exists
                let new_name = Self::get_foreground_process_name();
                let new_cache = Shared::new(ProcessInfo {
                    name: new_name.clone(),
                    timestamp: now,
                });
                let _ = self
                    .cached_process_info
                    .swap((Some(new_cache), Tag::None), Ordering::Release);
                new_name
            }
        };

        // Check if process is in whitelist
        if let Some(name) = process_name {
            whitelist.iter().any(|p| p.to_lowercase() == name)
        } else {
            // If we can't get process name, allow by default
            true
        }
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn handle_key_event(&self, message: u32, vk_code: u32) -> bool {
        let mut should_block = false;

        if matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN) {
            let _ = self.pressed_keys.insert_sync(vk_code);

            let kb_vk = self.switch_key_cache.keyboard_vk.load(Ordering::Relaxed);

            if kb_vk != 0 && vk_code == kb_vk {
                self.handle_switch_key_toggle();
                return true;
            }

            if kb_vk == 0 {
                let guard = Guard::new();
                let device_ptr = self
                    .switch_key_cache
                    .full_device
                    .load(Ordering::Acquire, &guard);
                if let Some(device) = device_ptr.as_ref()
                    && let InputDevice::KeyCombo(keys) = device
                    && keys.contains(&vk_code)
                {
                    let mut all_pressed = true;
                    for k in keys.iter() {
                        if !self.pressed_keys.contains_sync(k) {
                            all_pressed = false;
                            break;
                        }
                    }

                    if all_pressed {
                        self.handle_switch_key_toggle();
                        return true;
                    }
                }
            }
        }

        if matches!(message, WM_KEYUP | WM_SYSKEYUP) {
            let _ = self.pressed_keys.remove_sync(&vk_code);
        }

        if unlikely(self.is_paused() || !self.is_process_whitelisted()) {
            return should_block;
        }

        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // Check if this key is in an active combo
                if unlikely(self.is_in_active_combo(vk_code)) {
                    // Check if this is a main key of a combo with turbo disabled
                    let is_main_key_no_turbo = self.is_main_key_in_active_combo_no_turbo(vk_code);

                    if !is_main_key_no_turbo {
                        // Block repeat events for modifiers and turbo-enabled combo main keys
                        return true;
                    }
                }

                let mut pressed_snapshot: std::collections::HashSet<u32> =
                    std::collections::HashSet::with_capacity(16);
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
                    let already_active = self.is_combo_active(&device);

                    if already_active {
                        let allow_repeat = !self.is_turbo_enabled(&device);

                        if !allow_repeat {
                            return true;
                        }
                    }

                    if let InputDevice::KeyCombo(_) = &device {
                        let mut modifiers: SmallVec<[u32; 8]> = SmallVec::new();
                        self.pressed_keys.iter_sync(|key| {
                            if *key != vk_code && self.is_modifier_key(*key) {
                                modifiers.push(*key);
                            }
                            true
                        });

                        self.release_modifiers_once(&modifiers);
                        self.add_active_combo(device.clone(), modifiers.clone());
                    }

                    if let Some(pool) = self.worker_pool.get() {
                        pool.dispatch(InputEvent::Pressed(device));
                        should_block = true;
                    }
                }
            }

            WM_KEYUP | WM_SYSKEYUP => {
                let removed_combos = self.cleanup_released_combos();

                if !removed_combos.is_empty()
                    && let Some(pool) = self.worker_pool.get()
                {
                    for combo in removed_combos {
                        pool.dispatch(InputEvent::Released(combo));
                    }
                    should_block = true;
                }

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
    /// Supports multiple combos simultaneously (e.g., LALT+1, LALT+2, LALT+3 all active)
    #[inline]
    fn find_matching_combo(
        &self,
        pressed_keys: &std::collections::HashSet<u32>,
        main_key: u32,
    ) -> Option<InputDevice> {
        // Fast path: use lock-free read
        self.cached_combo_index
            .read_sync(&main_key, |_, combos| {
                // Iterate through potential combos
                for device in combos {
                    if let InputDevice::KeyCombo(combo_keys) = device {
                        // Check if all combo keys are pressed
                        let all_pressed = combo_keys.iter().all(|&k| pressed_keys.contains(&k));
                        if likely(all_pressed) {
                            return Some(device.clone());
                        }
                    }
                }
                None
            })
            .flatten()
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
    ) -> anyhow::Result<HashMap<InputDevice, InputMappingInfo>> {
        let mut input_mappings = HashMap::new();

        for mapping in &config.mappings {
            let trigger_device = Self::input_name_to_device(&mapping.trigger_key)
                .ok_or_else(|| anyhow::anyhow!("Invalid trigger input: {}", mapping.trigger_key))?;

            let target_keys = mapping.get_target_keys();
            if target_keys.is_empty() {
                continue; // Skip mappings without target keys
            }

            let interval = mapping.interval.unwrap_or(config.interval).max(5);
            let event_duration = mapping
                .event_duration
                .unwrap_or(config.event_duration)
                .max(2);
            let move_speed = mapping.move_speed.max(1);

            // Parse target keys into output actions
            let mut actions: SmallVec<[OutputAction; 4]> = SmallVec::new();
            for target_key in target_keys {
                if let Some(action) = Self::input_name_to_output(target_key) {
                    // Update MouseMove and MouseScroll actions with configured speed
                    let action = match action {
                        OutputAction::MouseMove(direction, _) => {
                            OutputAction::MouseMove(direction, move_speed)
                        }
                        OutputAction::MouseScroll(direction, _) => {
                            OutputAction::MouseScroll(direction, move_speed)
                        }
                        other => other,
                    };
                    actions.push(action);
                } else {
                    return Err(anyhow::anyhow!("Invalid target input: {}", target_key));
                }
            }

            if actions.is_empty() {
                continue; // Skip if no valid actions
            }

            // Create the final target action
            let target_action = if actions.len() == 1 {
                actions.into_iter().next().unwrap()
            } else {
                OutputAction::MultipleActions(Arc::new(actions))
            };

            // Create input mapping
            input_mappings.insert(
                trigger_device.clone(),
                InputMappingInfo {
                    target_action,
                    interval,
                    event_duration,
                    turbo_enabled: mapping.turbo_enabled,
                },
            );
        }

        Ok(input_mappings)
    }

    /// Parse and cache switch key configuration
    #[inline]
    fn update_switch_key_cache(cache: &SwitchKeyCache, key_name: &str) -> anyhow::Result<()> {
        cache.clear();

        let device = Self::input_name_to_device(key_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid switch key: {}", key_name))?;

        match &device {
            InputDevice::Keyboard(vk) => {
                cache.keyboard_vk.store(*vk, Ordering::Relaxed);
            }
            InputDevice::XInputCombo {
                device_type,
                button_ids,
            } => {
                let mask = Self::inputs_to_bitset(button_ids);
                cache.xinput_button_mask.store(mask, Ordering::Relaxed);

                let hash = Self::hash_device_type(device_type);
                cache.xinput_device_hash.store(hash, Ordering::Relaxed);
            }
            InputDevice::GenericDevice { button_id, .. } => {
                cache.generic_button_id.store(*button_id, Ordering::Relaxed);
            }
            InputDevice::KeyCombo(_) | InputDevice::Mouse(_) => {}
        }

        let shared_device = Shared::new(device);
        let _ = cache
            .full_device
            .swap((Some(shared_device), Tag::None), Ordering::Release);
        Ok(())
    }

    /// Convert button IDs to bitmask
    #[inline(always)]
    fn inputs_to_bitset(inputs: &[u32]) -> u32 {
        inputs
            .iter()
            .fold(0u32, |acc, &id| if id < 32 { acc | (1 << id) } else { acc })
    }

    /// Compute device type hash for comparison
    #[inline(always)]
    pub fn hash_device_type(device_type: &DeviceType) -> u32 {
        match device_type {
            DeviceType::Gamepad(vid) => (*vid as u32) ^ 0x01000000,
            DeviceType::Joystick(vid) => (*vid as u32) ^ 0x02000000,
            DeviceType::HidDevice { usage_page, usage } => {
                (*usage_page as u32) ^ ((*usage as u32) << 16)
            }
        }
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

    /// Parse input name to InputDevice (supports keyboard, mouse, gamepad, joystick, and custom devices)
    fn input_name_to_device(name: &str) -> Option<InputDevice> {
        let name_upper = name.to_uppercase();

        // Check for XInput combo format first (e.g., "GAMEPAD_045E_A" or "GAMEPAD_045E_LS_RightUp+A")
        if (name_upper.starts_with("GAMEPAD_") || name_upper.starts_with("JOYSTICK_"))
            && let Some(device) = Self::parse_xinput_combo(&name_upper)
        {
            return Some(device);
        }

        // Check for HID device formats (VID/PID with serial or device ID)
        // Format: "GAMEPAD_045E_0B05_ABC123_B2.0" (with serial)
        // Format: "GAMEPAD_045E_0B05_DEV12345678_B2.0" (without serial, uses device ID)
        if (name_upper.starts_with("GAMEPAD_")
            || name_upper.starts_with("JOYSTICK_")
            || name_upper.starts_with("HID_"))
            && let Some(device) = Self::parse_device_with_handle(&name_upper)
        {
            return Some(device);
        }

        // Try mouse button
        if let Some(button) = Self::mouse_button_name_to_type(&name_upper) {
            return Some(InputDevice::Mouse(button));
        }

        // Check if it's a key combination (contains '+')
        if name.contains('+') {
            let parts: Vec<&str> = name.split('+').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                return None;
            }

            // Try to parse each part as a device
            let mut devices: SmallVec<[InputDevice; 4]> = SmallVec::new();
            for part in &parts {
                if let Some(device) = Self::input_name_to_device(part) {
                    devices.push(device);
                } else {
                    // Fallback: try as VK code for keyboard
                    if let Some(vk) = Self::key_name_to_vk(part) {
                        devices.push(InputDevice::Keyboard(vk));
                    } else {
                        return None;
                    }
                }
            }

            // If all parts are keyboard keys, use KeyCombo for efficiency
            let all_keyboard = devices
                .iter()
                .all(|d| matches!(d, InputDevice::Keyboard(_)));
            if all_keyboard {
                let vk_codes: SmallVec<[u32; 4]> = devices
                    .into_iter()
                    .filter_map(|d| match d {
                        InputDevice::Keyboard(vk) => Some(vk),
                        _ => None,
                    })
                    .collect();
                if vk_codes.len() >= 2 {
                    return Some(InputDevice::KeyCombo(vk_codes.into_vec()));
                }
            }
        }

        // Try single keyboard key
        if let Some(vk) = Self::key_name_to_vk(name) {
            return Some(InputDevice::Keyboard(vk));
        }

        None
    }

    /// Supported formats:
    /// - "GAMEPAD_045E_0B05_ABC123_B2.0" (with serial number)
    /// - "GAMEPAD_045E_0B05_DEV12345678_B2.0" (without serial number, uses device handle)
    /// - "HID_0001_045E_0B05_ABC123" (HID with serial)
    /// - "HID_0001_045E_0B05_DEV12345678" (HID without serial)
    ///
    /// Parse XInput format (e.g., "GAMEPAD_045E_A" or "GAMEPAD_045E_DPad_Right+X+Y")
    fn parse_xinput_combo(input: &str) -> Option<InputDevice> {
        let mut parts = input.split('_');

        // XInput format: TYPE_VID_ButtonName[+ButtonName...]
        let device_type = match parts.next()? {
            "GAMEPAD" => DeviceType::Gamepad(0),
            "JOYSTICK" => DeviceType::Joystick(0),
            _ => return None,
        };

        // Parse VID (4-digit hex)
        let vid = u16::from_str_radix(parts.next()?, 16).ok()?;
        let device_type = match device_type {
            DeviceType::Gamepad(_) => DeviceType::Gamepad(vid),
            DeviceType::Joystick(_) => DeviceType::Joystick(vid),
            other => other,
        };

        // Collect remaining parts as button name (may contain underscores)
        let button_part: SmallVec<[&str; 4]> = parts.collect();
        if button_part.is_empty() {
            return None;
        }

        let button_str = button_part.join("_");
        let mut button_ids = SmallVec::<[u32; 8]>::new();

        // Split by '+' to support combinations like "DPad_Right+X+Y"
        for button_name in button_str.split('+') {
            let button_name = button_name.trim();
            if let Some(input_id) = crate::xinput::XInputHandler::name_to_input_id(button_name) {
                button_ids.push(input_id);
            } else {
                return None;
            }
        }

        if button_ids.is_empty() {
            return None;
        }

        Some(InputDevice::XInputCombo {
            device_type,
            button_ids: button_ids.into_vec(),
        })
    }

    fn parse_device_with_handle(input: &str) -> Option<InputDevice> {
        let parts: Vec<&str> = input.split('_').collect();

        // Minimum parts: TYPE_VID_PID_SERIAL_B (5 parts)
        // For HID: HID_USAGE_VID_PID_SERIAL (5 parts minimum)
        if parts.len() < 5 {
            return None;
        }

        let is_hid = parts[0] == "HID";

        // Determine device type
        let device_type = match parts[0] {
            "GAMEPAD" => DeviceType::Gamepad(0),
            "JOYSTICK" => DeviceType::Joystick(0),
            "HID" => {
                let usage_page = u16::from_str_radix(parts[1], 16).ok()?;
                DeviceType::HidDevice {
                    usage_page,
                    usage: 0,
                }
            }
            _ => return None,
        };

        // Parse VID/PID
        let (vid_idx, pid_idx, serial_idx) = if is_hid { (2, 3, 4) } else { (1, 2, 3) };

        let vendor_id = u16::from_str_radix(parts.get(vid_idx)?, 16).ok()?;
        let product_id = u16::from_str_radix(parts.get(pid_idx)?, 16).ok()?;

        // Check if serial part starts with "DEV" (no serial) or is a serial number
        let serial_part = parts.get(serial_idx)?;

        let stable_device_id = if serial_part.starts_with("DEV") {
            let handle_str = serial_part.strip_prefix("DEV")?;
            let parsed = u32::from_str_radix(handle_str, 16).ok()?;
            parsed as u64
        } else {
            // Use FNV-1a hash (same as rawinput) for consistency
            let mut hash = crate::util::fnv64::OFFSET_BASIS;
            hash = crate::util::fnv1a_hash_u64(hash, vendor_id as u64);
            hash = crate::util::fnv1a_hash_u64(hash, product_id as u64);
            hash = crate::util::fnv1a_hash_bytes(hash, serial_part.as_bytes());
            hash
        };

        // Update device_type with actual VID
        let device_type = match device_type {
            DeviceType::Gamepad(_) => DeviceType::Gamepad(vendor_id),
            DeviceType::Joystick(_) => DeviceType::Joystick(vendor_id),
            other => other,
        };

        // Parse position (button location)
        let pos_idx = serial_idx + 1;
        let button_id = if let Some(pos) = parts.get(pos_idx) {
            let pos_str = pos.strip_prefix('B')?;

            if pos_str.contains('.') {
                // Bit-level: "2.0" -> byte 2, bit 0
                let bit_parts: Vec<&str> = pos_str.split('.').collect();
                if bit_parts.len() != 2 {
                    return None;
                }
                let byte_idx = bit_parts[0].parse::<u64>().ok()?;
                let bit_idx = bit_parts[1].parse::<u64>().ok()?;
                (stable_device_id << 32) | (byte_idx << 16) | bit_idx
            } else {
                // Byte-level: "4" -> byte 4 (analog)
                let byte_idx = pos_str.parse::<u64>().ok()?;
                (stable_device_id << 32) | 0x80000000u64 | byte_idx
            }
        } else {
            // No position (HID device without button info)
            stable_device_id << 32
        };

        Some(InputDevice::GenericDevice {
            device_type,
            button_id,
        })
    }

    /// Parse input name to OutputAction
    /// Note: Output currently supports keyboard and mouse only.
    /// Generic device output would require virtual device drivers.
    fn input_name_to_output(name: &str) -> Option<OutputAction> {
        let name_upper = name.to_uppercase();

        // Try mouse scroll
        if let Some(direction) = Self::mouse_scroll_name_to_direction(&name_upper) {
            return Some(OutputAction::MouseScroll(direction, 1)); // Default speed, will be overridden by move_speed
        }

        // Try mouse movement
        if let Some(direction) = Self::mouse_move_name_to_direction(&name_upper) {
            return Some(OutputAction::MouseMove(direction, 5)); // Default speed, will be overridden by move_speed
        }

        // Try mouse button
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

    /// Parse mouse scroll name to MouseScrollDirection
    fn mouse_scroll_name_to_direction(name: &str) -> Option<MouseScrollDirection> {
        match name {
            "SCROLL_UP" | "SCROLLUP" | "WHEEL_UP" | "WHEELUP" => Some(MouseScrollDirection::Up),
            "SCROLL_DOWN" | "SCROLLDOWN" | "WHEEL_DOWN" | "WHEELDOWN" => {
                Some(MouseScrollDirection::Down)
            }
            _ => None,
        }
    }

    /// Parse mouse movement name to MouseMoveDirection
    fn mouse_move_name_to_direction(name: &str) -> Option<MouseMoveDirection> {
        match name {
            "MOUSE_UP" | "MOUSEUP" | "MOVE_UP" | "M_UP" => Some(MouseMoveDirection::Up),
            "MOUSE_DOWN" | "MOUSEDOWN" | "MOVE_DOWN" | "M_DOWN" => Some(MouseMoveDirection::Down),
            "MOUSE_LEFT" | "MOUSELEFT" | "MOVE_LEFT" | "M_LEFT" => Some(MouseMoveDirection::Left),
            "MOUSE_RIGHT" | "MOUSERIGHT" | "MOVE_RIGHT" | "M_RIGHT" => {
                Some(MouseMoveDirection::Right)
            }
            "MOUSE_UP_LEFT" | "MOUSEUPLEFT" | "M_UP_LEFT" => Some(MouseMoveDirection::UpLeft),
            "MOUSE_UP_RIGHT" | "MOUSEUPRIGHT" | "M_UP_RIGHT" => Some(MouseMoveDirection::UpRight),
            "MOUSE_DOWN_LEFT" | "MOUSEDOWNLEFT" | "M_DOWN_LEFT" => {
                Some(MouseMoveDirection::DownLeft)
            }
            "MOUSE_DOWN_RIGHT" | "MOUSEDOWNRIGHT" | "M_DOWN_RIGHT" => {
                Some(MouseMoveDirection::DownRight)
            }
            _ => None,
        }
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
                target_keys: SmallVec::from_vec(vec!["B".to_string()]),
                interval: Some(10),
                event_duration: Some(5),
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "F1".to_string(),
                target_keys: SmallVec::from_vec(vec!["SPACE".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            },
        ];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
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
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_input_mappings_invalid_target() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["INVALID_KEY".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
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
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(3), // Below minimum
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
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
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: None,
            event_duration: Some(3), // Below minimum
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        let device = InputDevice::Keyboard(0x41); // 'A' key
        let a_mapping = input_mappings.get(&device).unwrap();
        assert!(
            a_mapping.event_duration >= 2,
            "Duration should be clamped to minimum 2"
        );
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
                target_keys: SmallVec::from_vec(vec!["1".to_string()]),
                interval: Some(10),
                event_duration: Some(5),
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "B".to_string(),
                target_keys: SmallVec::from_vec(vec!["2".to_string()]),
                interval: Some(15),
                event_duration: Some(8),
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "C".to_string(),
                target_keys: SmallVec::from_vec(vec!["3".to_string()]),
                interval: Some(20),
                event_duration: Some(10),
                turbo_enabled: true,
                move_speed: 10,
            },
        ];

        let input_mappings = AppState::create_input_mappings(&config).unwrap();
        assert_eq!(input_mappings.len(), 3);

        let device_a = InputDevice::Keyboard(0x41);
        let device_b = InputDevice::Keyboard(0x42);
        let device_c = InputDevice::Keyboard(0x43);

        assert_eq!(input_mappings.get(&device_a).unwrap().interval, 10);
        assert_eq!(input_mappings.get(&device_b).unwrap().interval, 15);
        assert_eq!(input_mappings.get(&device_c).unwrap().interval, 20);
    }

    #[test]
    fn test_app_state_reload_config() {
        let config = AppConfig::default();
        let state = AppState::new(config).unwrap();

        // Initial state
        assert!(!state.is_paused());
        assert_eq!(
            state.switch_key_cache.keyboard_vk.load(Ordering::Relaxed),
            0x2E
        ); // DELETE

        // Create new config
        let mut new_config = AppConfig::default();
        new_config.switch_key = "F11".to_string();
        new_config.show_tray_icon = false;
        new_config.input_timeout = 50;

        // Reload config
        state.reload_config(new_config).unwrap();

        // Verify changes
        assert_eq!(
            state.switch_key_cache.keyboard_vk.load(Ordering::Relaxed),
            0x7A
        ); // F11
        assert!(!state.show_tray_icon());
        assert_eq!(state.input_timeout(), 50);
    }

    #[test]
    fn test_key_mapping_with_boundary_values() {
        let mut config = AppConfig::default();

        // Test with minimum interval
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(5), // Minimum valid value
            event_duration: Some(2),
            turbo_enabled: true,
            move_speed: 10,
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
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(0),
            event_duration: Some(0),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let state = AppState::new(config).unwrap();

        // Values should be adjusted to minimum of 2
        // This test verifies auto-adjustment behavior
        assert!(state.input_mappings.len() > 0);
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
    fn test_input_name_to_device_backward_compatibility() {
        // Ensure keyboard input still works
        assert!(matches!(
            AppState::input_name_to_device("A"),
            Some(InputDevice::Keyboard(0x41))
        ));

        // Ensure mouse input still works
        assert!(matches!(
            AppState::input_name_to_device("LBUTTON"),
            Some(InputDevice::Mouse(MouseButton::Left))
        ));

        // Ensure key combos still work
        let combo = AppState::input_name_to_device("LALT+A");
        assert!(matches!(combo, Some(InputDevice::KeyCombo(_))));
    }

    #[test]
    fn test_device_type_equality() {
        let gamepad1 = DeviceType::Gamepad(0x045e);
        let gamepad2 = DeviceType::Gamepad(0x045e);
        let gamepad3 = DeviceType::Gamepad(0x046d);

        assert_eq!(gamepad1, gamepad2);
        assert_ne!(gamepad1, gamepad3);

        let hid1 = DeviceType::HidDevice {
            usage_page: 0x01,
            usage: 0x05,
        };
        let hid2 = DeviceType::HidDevice {
            usage_page: 0x01,
            usage: 0x05,
        };
        assert_eq!(hid1, hid2);
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
    fn test_parse_device_with_vid_pid_serial() {
        // Test parsing new format with VID/PID/Serial
        let device = AppState::input_name_to_device("GAMEPAD_045E_0B05_ABC123_B2.0");
        assert!(device.is_some());
        match device.unwrap() {
            InputDevice::GenericDevice {
                device_type: DeviceType::Gamepad(_),
                button_id,
            } => {
                let stable_id = (button_id >> 32) as u32;
                let position = (button_id & 0xFFFFFFFF) as u32;
                let byte_idx = (position >> 16) as u16;
                let bit_idx = (position & 0xFFFF) as u16;

                // Stable ID should be a hash (non-zero)
                assert_ne!(stable_id, 0);
                assert_eq!(byte_idx, 2);
                assert_eq!(bit_idx, 0);
            }
            _ => panic!("Expected GenericDevice"),
        }
    }

    #[test]
    fn test_parse_device_with_vid_pid_no_serial() {
        // Test parsing new format with VID/PID but no serial (DEV fallback)
        let device = AppState::input_name_to_device("GAMEPAD_045E_0B05_DEV12345678_B2.0");
        assert!(device.is_some());
        match device.unwrap() {
            InputDevice::GenericDevice {
                device_type: DeviceType::Gamepad(_),
                button_id,
            } => {
                let stable_id = (button_id >> 32) as u32;
                let position = (button_id & 0xFFFFFFFF) as u32;
                let byte_idx = (position >> 16) as u16;
                let bit_idx = (position & 0xFFFF) as u16;

                assert_eq!(stable_id, 0x12345678); // Should match DEV value
                assert_eq!(byte_idx, 2);
                assert_eq!(bit_idx, 0);
            }
            _ => panic!("Expected GenericDevice"),
        }
    }

    #[test]
    fn test_multiple_device_handles() {
        // Test that different handles produce different button IDs
        let device1 = InputDevice::GenericDevice {
            device_type: DeviceType::Gamepad(0x045e),
            button_id: (0x11111111u64 << 32) | (2u64 << 16) | 0u64,
        };

        let device2 = InputDevice::GenericDevice {
            device_type: DeviceType::Gamepad(0x045e),
            button_id: (0x22222222u64 << 32) | (2u64 << 16) | 0u64,
        };

        assert_ne!(device1, device2);
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
                target_keys: SmallVec::from_vec(vec!["B".to_string()]),
                interval: Some(10),
                event_duration: Some(5),
                turbo_enabled: true,
                move_speed: 10,
            },
            KeyMapping {
                trigger_key: "CTRL+SHIFT+F".to_string(),
                target_keys: SmallVec::from_vec(vec!["ALT+F4".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
            },
        ];

        let input_mappings = AppState::create_input_mappings(&config).unwrap();
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
        let guard = Guard::new();
        let cache_ptr = state.cached_process_info.load(Ordering::Acquire, &guard);
        let initial_name = cache_ptr.as_ref().map(|c| c.name.clone());

        // Second call immediately - cache hit (should use cached value)
        let _ = state.is_process_whitelisted();

        // Verify cache still has same value
        let cache_ptr = state.cached_process_info.load(Ordering::Acquire, &guard);
        let cached_name = cache_ptr.as_ref().map(|c| c.name.clone());
        assert_eq!(cached_name, initial_name);

        // Wait for cache to expire (>50ms)
        thread::sleep(Duration::from_millis(60));

        // Third call after expiration - cache miss (will refresh)
        let _ = state.is_process_whitelisted();

        // Cache should be refreshed with new timestamp
        let guard = Guard::new();
        let cache_ptr = state.cached_process_info.load(Ordering::Acquire, &guard);
        let timestamp = cache_ptr.as_ref().map(|c| c.timestamp);
        if let Some(ts) = timestamp {
            assert!(ts.elapsed() < Duration::from_millis(10));
        }
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

    #[test]
    fn test_create_multiple_target_keys_mapping() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "Q".to_string(),
            target_keys: SmallVec::from_vec(vec!["MOUSE_UP".to_string(), "MOUSE_LEFT".to_string()]),
            interval: Some(5),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        assert_eq!(input_mappings.len(), 1);

        let device_q = InputDevice::Keyboard(0x51); // 'Q' key
        let q_mapping = input_mappings.get(&device_q).unwrap();

        // Should create MultipleActions
        assert!(matches!(
            &q_mapping.target_action,
            OutputAction::MultipleActions(_)
        ));
    }

    #[test]
    fn test_multiple_target_keys_creates_multiple_actions() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
            ]),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        let device_a = InputDevice::Keyboard(0x41);
        let a_mapping = input_mappings.get(&device_a).unwrap();

        if let OutputAction::MultipleActions(actions) = &a_mapping.target_action {
            assert_eq!(actions.len(), 3);
        } else {
            panic!("Expected MultipleActions variant");
        }
    }

    #[test]
    fn test_single_target_key_not_wrapped_in_multiple_actions() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        let device_a = InputDevice::Keyboard(0x41);
        let a_mapping = input_mappings.get(&device_a).unwrap();

        // Single target should NOT be wrapped in MultipleActions
        assert!(matches!(
            &a_mapping.target_action,
            OutputAction::KeyboardKey(_)
        ));
    }

    #[test]
    fn test_empty_target_keys_skipped() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::new(),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        // Empty target keys should be skipped
        assert_eq!(input_mappings.len(), 0);
    }

    #[test]
    fn test_multiple_target_keys_with_mixed_types() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "Q".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ]),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        let device_q = InputDevice::Keyboard(0x51);
        let q_mapping = input_mappings.get(&device_q).unwrap();

        if let OutputAction::MultipleActions(actions) = &q_mapping.target_action {
            assert_eq!(actions.len(), 3);
            // All should be KeyboardKey actions
            for action in actions.iter() {
                assert!(matches!(action, OutputAction::KeyboardKey(_)));
            }
        } else {
            panic!("Expected MultipleActions variant");
        }
    }

    #[test]
    fn test_multiple_target_keys_validation() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string(), "INVALID_KEY".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        // Should fail due to invalid target key
        assert!(result.is_err());
    }

    #[test]
    fn test_smallvec_optimization_in_multiple_actions() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["1".to_string(), "2".to_string()]),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let result = AppState::create_input_mappings(&config);
        assert!(result.is_ok());

        let input_mappings = result.unwrap();
        let device_a = InputDevice::Keyboard(0x41);
        let a_mapping = input_mappings.get(&device_a).unwrap();

        // Verify SmallVec is used (inline storage for small collections)
        if let OutputAction::MultipleActions(actions) = &a_mapping.target_action {
            assert_eq!(actions.len(), 2);
            assert!(!actions.spilled()); // Should use inline storage
        } else {
            panic!("Expected MultipleActions variant");
        }
    }
}
