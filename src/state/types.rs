//! Type definitions for input/output handling.

use std::convert::Infallible;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use scc::{AtomicShared, Tag};
use smallvec::SmallVec;

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
    /// Mouse movement direction input (for sequence triggers)
    MouseMove(MouseMoveDirection),
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
            InputDevice::Keyboard(vk) => {
                write!(f, "{}", super::parsing::vk_to_key_name(*vk))
            }
            InputDevice::Mouse(button) => {
                let name = match button {
                    MouseButton::Left => "LBUTTON",
                    MouseButton::Right => "RBUTTON",
                    MouseButton::Middle => "MBUTTON",
                    MouseButton::X1 => "XBUTTON1",
                    MouseButton::X2 => "XBUTTON2",
                };
                write!(f, "{}", name)
            }
            InputDevice::MouseMove(direction) => {
                let name = match direction {
                    MouseMoveDirection::Up => "MOUSE_UP",
                    MouseMoveDirection::Down => "MOUSE_DOWN",
                    MouseMoveDirection::Left => "MOUSE_LEFT",
                    MouseMoveDirection::Right => "MOUSE_RIGHT",
                    MouseMoveDirection::UpLeft => "MOUSE_UP_LEFT",
                    MouseMoveDirection::UpRight => "MOUSE_UP_RIGHT",
                    MouseMoveDirection::DownLeft => "MOUSE_DOWN_LEFT",
                    MouseMoveDirection::DownRight => "MOUSE_DOWN_RIGHT",
                };
                write!(f, "{}", name)
            }
            InputDevice::KeyCombo(keys) => {
                for (i, &vk) in keys.iter().enumerate() {
                    if i > 0 {
                        write!(f, "+")?;
                    }
                    write!(f, "{}", super::parsing::vk_to_key_name(vk))?;
                }
                Ok(())
            }
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

impl MouseMoveDirection {
    /// Encodes direction to u8 for atomic storage (1-8)
    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Up => 1,
            Self::Down => 2,
            Self::Left => 3,
            Self::Right => 4,
            Self::UpLeft => 5,
            Self::UpRight => 6,
            Self::DownLeft => 7,
            Self::DownRight => 8,
        }
    }

    /// Decodes u8 to direction (1-8), returns None for 0
    #[inline(always)]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Up),
            2 => Some(Self::Down),
            3 => Some(Self::Left),
            4 => Some(Self::Right),
            5 => Some(Self::UpLeft),
            6 => Some(Self::UpRight),
            7 => Some(Self::DownLeft),
            8 => Some(Self::DownRight),
            _ => None,
        }
    }

    /// Checks if diagonal direction is a valid transition between two cardinal directions.
    /// Used for tolerating intermediate diagonals in mouse movement sequences.
    #[inline(always)]
    pub const fn is_transition_between(self, prev: Self, next: Self) -> bool {
        use MouseMoveDirection::*;
        matches!(
            (self, prev, next),
            (UpLeft, Up, Left)
                | (UpLeft, Left, Up)
                | (UpRight, Up, Right)
                | (UpRight, Right, Up)
                | (DownLeft, Down, Left)
                | (DownLeft, Left, Down)
                | (DownRight, Down, Right)
                | (DownRight, Right, Down)
        )
    }
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
    /// Sequential actions with interval between each action (for target sequence mode)
    /// (actions, interval_ms between each action)
    SequentialActions(Arc<SmallVec<[OutputAction; 4]>>, u64),
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
    /// Is this a sequence trigger (only triggered by sequence match)
    pub is_sequence: bool,
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
    pub fn new() -> Self {
        Self {
            keyboard_vk: AtomicU32::new(0),
            xinput_button_mask: AtomicU32::new(0),
            xinput_device_hash: AtomicU32::new(0),
            generic_button_id: AtomicU64::new(0),
            full_device: AtomicShared::null(),
        }
    }

    #[inline(always)]
    pub fn clear(&self) {
        self.keyboard_vk.store(0, Ordering::Relaxed);
        self.xinput_button_mask.store(0, Ordering::Relaxed);
        self.xinput_device_hash.store(0, Ordering::Relaxed);
        self.generic_button_id.store(0, Ordering::Relaxed);
        let _ = self.full_device.swap((None, Tag::None), Ordering::Relaxed);
    }
}

impl Default for SwitchKeyCache {
    fn default() -> Self {
        Self::new()
    }
}
