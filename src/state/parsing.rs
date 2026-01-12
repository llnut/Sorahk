//! Input/output name parsing utilities.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use smallvec::SmallVec;

use super::types::*;

/// Converts virtual key code to key name string.
#[inline]
pub fn vk_to_key_name(vk: u32) -> String {
    match vk {
        // A-Z
        0x41..=0x5A => char::from_u32(vk).unwrap().to_string(),
        // 0-9
        0x30..=0x39 => char::from_u32(vk).unwrap().to_string(),
        // Numpad 0-9
        0x60..=0x69 => format!("NUMPAD{}", vk - 0x60),
        // F1-F24
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        // Navigation keys
        0x20 => "SPACE".to_string(),
        0x0D => "RETURN".to_string(),
        0x09 => "TAB".to_string(),
        0x1B => "ESCAPE".to_string(),
        0x08 => "BACK".to_string(),
        0x2E => "DELETE".to_string(),
        0x2D => "INSERT".to_string(),
        0x24 => "HOME".to_string(),
        0x23 => "END".to_string(),
        0x21 => "PAGEUP".to_string(),
        0x22 => "PAGEDOWN".to_string(),
        0x26 => "UP".to_string(),
        0x28 => "DOWN".to_string(),
        0x25 => "LEFT".to_string(),
        0x27 => "RIGHT".to_string(),
        // Lock and special keys
        0x14 => "CAPITAL".to_string(),
        0x90 => "NUMLOCK".to_string(),
        0x91 => "SCROLL".to_string(),
        0x13 => "PAUSE".to_string(),
        0x2C => "SNAPSHOT".to_string(),
        // Numpad operators
        0x6A => "MULTIPLY".to_string(),
        0x6B => "ADD".to_string(),
        0x6C => "SEPARATOR".to_string(),
        0x6D => "SUBTRACT".to_string(),
        0x6E => "DECIMAL".to_string(),
        0x6F => "DIVIDE".to_string(),
        // OEM keys
        0xBA => "OEM_1".to_string(),
        0xBB => "OEM_PLUS".to_string(),
        0xBC => "OEM_COMMA".to_string(),
        0xBD => "OEM_MINUS".to_string(),
        0xBE => "OEM_PERIOD".to_string(),
        0xBF => "OEM_2".to_string(),
        0xC0 => "OEM_3".to_string(),
        0xDB => "OEM_4".to_string(),
        0xDC => "OEM_5".to_string(),
        0xDD => "OEM_6".to_string(),
        0xDE => "OEM_7".to_string(),
        0xDF => "OEM_8".to_string(),
        0xE2 => "OEM_102".to_string(),
        // Modifiers
        0xA2 => "LCTRL".to_string(),
        0xA3 => "RCTRL".to_string(),
        0xA4 => "LALT".to_string(),
        0xA5 => "RALT".to_string(),
        0xA0 => "LSHIFT".to_string(),
        0xA1 => "RSHIFT".to_string(),
        0x5B => "LWIN".to_string(),
        0x5C => "RWIN".to_string(),
        // Mouse buttons (for completeness)
        0x01 => "LBUTTON".to_string(),
        0x02 => "RBUTTON".to_string(),
        0x04 => "MBUTTON".to_string(),
        0x05 => "XBUTTON1".to_string(),
        0x06 => "XBUTTON2".to_string(),
        // Unknown key - format as hex
        _ => format!("VK_{:02X}", vk),
    }
}

pub fn key_name_to_vk(key_name: &str) -> Option<u32> {
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

pub fn vk_to_scancode(vk_code: u32) -> u16 {
    SCANCODE_MAP.get(&vk_code).copied().unwrap_or(0)
}

pub fn input_name_to_device(name: &str) -> Option<InputDevice> {
    let name_upper = name.to_uppercase();

    // Check for XInput combo format first (e.g., "GAMEPAD_045E_A" or "GAMEPAD_045E_LS_RightUp+A")
    if (name_upper.starts_with("GAMEPAD_") || name_upper.starts_with("JOYSTICK_"))
        && let Some(device) = parse_xinput_combo(&name_upper)
    {
        return Some(device);
    }

    // Check for HID device formats (VID/PID with serial or device ID)
    // Format: "GAMEPAD_045E_0B05_ABC123_B2.0" (with serial)
    // Format: "GAMEPAD_045E_0B05_DEV12345678_B2.0" (without serial, uses device ID)
    if (name_upper.starts_with("GAMEPAD_")
        || name_upper.starts_with("JOYSTICK_")
        || name_upper.starts_with("HID_"))
        && let Some(device) = parse_device_with_handle(&name_upper)
    {
        return Some(device);
    }

    // Try mouse button
    if let Some(button) = mouse_button_name_to_type(&name_upper) {
        return Some(InputDevice::Mouse(button));
    }

    // Try mouse movement direction
    if let Some(direction) = mouse_move_name_to_direction(&name_upper) {
        return Some(InputDevice::MouseMove(direction));
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
            if let Some(device) = input_name_to_device(part) {
                devices.push(device);
            } else {
                // Try parsing as VK code
                if let Some(vk) = key_name_to_vk(part) {
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
    if let Some(vk) = key_name_to_vk(name) {
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
            // Check for diagonal stick directions (these need to be expanded to two IDs)
            let diagonal_ids = expand_diagonal_direction(button_name)?;
            button_ids.extend_from_slice(&diagonal_ids);
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

/// Expands diagonal direction names to their component button IDs
/// For example: "LS_LeftDown" → [0x11, 0x13] (left + down)
fn expand_diagonal_direction(name: &str) -> Option<SmallVec<[u32; 2]>> {
    let name_upper = name.to_uppercase();
    match name_upper.as_str() {
        // Left Stick diagonals
        "LS_RIGHTUP" | "LS_RIGHT_UP" => Some(SmallVec::from_buf([0x10, 0x12])), // Right + Up
        "LS_RIGHTDOWN" | "LS_RIGHT_DOWN" => Some(SmallVec::from_buf([0x10, 0x13])), // Right + Down
        "LS_LEFTUP" | "LS_LEFT_UP" => Some(SmallVec::from_buf([0x11, 0x12])),       // Left + Up
        "LS_LEFTDOWN" | "LS_LEFT_DOWN" => Some(SmallVec::from_buf([0x11, 0x13])), // Left + Down

        // Right Stick diagonals
        "RS_RIGHTUP" | "RS_RIGHT_UP" => Some(SmallVec::from_buf([0x14, 0x16])), // Right + Up
        "RS_RIGHTDOWN" | "RS_RIGHT_DOWN" => Some(SmallVec::from_buf([0x14, 0x17])), // Right + Down
        "RS_LEFTUP" | "RS_LEFT_UP" => Some(SmallVec::from_buf([0x15, 0x16])),       // Left + Up
        "RS_LEFTDOWN" | "RS_LEFT_DOWN" => Some(SmallVec::from_buf([0x15, 0x17])), // Left + Down

        // D-Pad diagonals
        "DPAD_UPRIGHT" | "DPAD_UP_RIGHT" => Some(SmallVec::from_buf([0x01, 0x04])), // Up + Right
        "DPAD_UPLEFT" | "DPAD_UP_LEFT" => Some(SmallVec::from_buf([0x01, 0x03])),   // Up + Left
        "DPAD_DOWNRIGHT" | "DPAD_DOWN_RIGHT" => Some(SmallVec::from_buf([0x02, 0x04])), // Down + Right
        "DPAD_DOWNLEFT" | "DPAD_DOWN_LEFT" => Some(SmallVec::from_buf([0x02, 0x03])), // Down + Left

        _ => None,
    }
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
pub fn input_name_to_output(name: &str) -> Option<OutputAction> {
    let name_upper = name.to_uppercase();

    // Try mouse scroll
    if let Some(direction) = mouse_scroll_name_to_direction(&name_upper) {
        return Some(OutputAction::MouseScroll(direction, 1)); // Default speed, will be overridden by move_speed
    }

    // Try mouse movement
    if let Some(direction) = mouse_move_name_to_direction(&name_upper) {
        return Some(OutputAction::MouseMove(direction, 5)); // Default speed, will be overridden by move_speed
    }

    // Try mouse button
    if let Some(button) = mouse_button_name_to_type(&name_upper) {
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
            if let Some(vk) = key_name_to_vk(part) {
                let scancode = vk_to_scancode(vk);
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
    if let Some(vk) = key_name_to_vk(name) {
        let scancode = vk_to_scancode(vk);
        if scancode != 0 {
            return Some(OutputAction::KeyboardKey(scancode));
        }
    }

    None
}

/// Parse mouse scroll name to MouseScrollDirection
pub fn mouse_scroll_name_to_direction(name: &str) -> Option<MouseScrollDirection> {
    match name {
        "SCROLL_UP" | "SCROLLUP" | "WHEEL_UP" | "WHEELUP" => Some(MouseScrollDirection::Up),
        "SCROLL_DOWN" | "SCROLLDOWN" | "WHEEL_DOWN" | "WHEELDOWN" => {
            Some(MouseScrollDirection::Down)
        }
        _ => None,
    }
}

/// Parse mouse movement name to MouseMoveDirection
pub fn mouse_move_name_to_direction(name: &str) -> Option<MouseMoveDirection> {
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
pub fn mouse_button_name_to_type(name: &str) -> Option<MouseButton> {
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

pub static SCANCODE_MAP: LazyLock<HashMap<u32, u16>> = LazyLock::new(|| {
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
