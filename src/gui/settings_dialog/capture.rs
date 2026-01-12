//! Key capture functionality for settings dialog.

use crate::gui::SorahkGui;
use smallvec::SmallVec;
use std::collections::HashSet;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

impl SorahkGui {
    #[inline]
    pub(super) fn poll_all_pressed_keys() -> HashSet<u32> {
        const ALL_VK_CODES: &[u32] = &[
            0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C,
            0x20, 0x0D, 0x09, 0x1B, 0x08, 0x2E, 0x2D, 0x24, 0x23, 0x21, 0x22, 0x26, 0x28, 0x25, 0x27,
            0x14, 0x90, 0x91, 0x13, 0x2C,
            0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
            0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF, 0xE2,
            0x01, 0x02, 0x04, 0x05, 0x06,
        ];

        let mut pressed_keys = HashSet::with_capacity(16);

        unsafe {
            for vk in 0x30u32..=0x5A {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            for vk in 0x60u32..=0x87 {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            for &vk in ALL_VK_CODES {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }
        }

        pressed_keys
    }

    #[inline]
    pub(super) fn format_captured_keys(vk_codes: &HashSet<u32>) -> Option<String> {
        if vk_codes.is_empty() {
            return None;
        }

        let mut modifiers: SmallVec<[u32; 8]> = SmallVec::new();
        let mut main_key: Option<u32> = None;

        for &vk in vk_codes {
            if matches!(vk, 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0x5B | 0x5C) {
                modifiers.push(vk);
            } else if main_key.is_none() {
                main_key = Some(vk);
            }
        }

        let mut result = String::with_capacity(64);
        let mut first = true;

        for &vk in &modifiers {
            if !first {
                result.push('+');
            }
            first = false;
            result.push_str(match vk {
                0xA2 => "LCTRL",
                0xA3 => "RCTRL",
                0xA4 => "LALT",
                0xA5 => "RALT",
                0xA0 => "LSHIFT",
                0xA1 => "RSHIFT",
                0x5B => "LWIN",
                0x5C => "RWIN",
                _ => continue,
            });
        }

        if let Some(vk) = main_key
            && let Some(name) = Self::vk_to_string(vk) {
                if !first {
                    result.push('+');
                }
                result.push_str(&name);
                first = false;
            }

        if !first {
            Some(result)
        } else {
            None
        }
    }

    /// Converts VK code to key name string
    #[inline]
    pub(super) fn vk_to_string(vk: u32) -> Option<String> {
        match vk {
            // A-Z
            0x41..=0x5A => Some(char::from_u32(vk).unwrap().to_string()),
            // 0-9
            0x30..=0x39 => Some(char::from_u32(vk).unwrap().to_string()),
            // Numpad 0-9
            0x60..=0x69 => Some(format!("NUMPAD{}", vk - 0x60)),
            // F1-F24
            0x70..=0x87 => Some(format!("F{}", vk - 0x70 + 1)),
            // Navigation keys
            0x20 => Some("SPACE".to_string()),
            0x0D => Some("RETURN".to_string()),
            0x09 => Some("TAB".to_string()),
            0x1B => Some("ESCAPE".to_string()),
            0x08 => Some("BACK".to_string()),
            0x2E => Some("DELETE".to_string()),
            0x2D => Some("INSERT".to_string()),
            0x24 => Some("HOME".to_string()),
            0x23 => Some("END".to_string()),
            0x21 => Some("PAGEUP".to_string()),
            0x22 => Some("PAGEDOWN".to_string()),
            0x26 => Some("UP".to_string()),
            0x28 => Some("DOWN".to_string()),
            0x25 => Some("LEFT".to_string()),
            0x27 => Some("RIGHT".to_string()),
            // Lock and special keys
            0x14 => Some("CAPITAL".to_string()),
            0x90 => Some("NUMLOCK".to_string()),
            0x91 => Some("SCROLL".to_string()),
            0x13 => Some("PAUSE".to_string()),
            0x2C => Some("SNAPSHOT".to_string()),
            // Numpad operators
            0x6A => Some("MULTIPLY".to_string()),
            0x6B => Some("ADD".to_string()),
            0x6C => Some("SEPARATOR".to_string()),
            0x6D => Some("SUBTRACT".to_string()),
            0x6E => Some("DECIMAL".to_string()),
            0x6F => Some("DIVIDE".to_string()),
            // OEM keys
            0xBA => Some("OEM_1".to_string()),
            0xBB => Some("OEM_PLUS".to_string()),
            0xBC => Some("OEM_COMMA".to_string()),
            0xBD => Some("OEM_MINUS".to_string()),
            0xBE => Some("OEM_PERIOD".to_string()),
            0xBF => Some("OEM_2".to_string()),
            0xC0 => Some("OEM_3".to_string()),
            0xDB => Some("OEM_4".to_string()),
            0xDC => Some("OEM_5".to_string()),
            0xDD => Some("OEM_6".to_string()),
            0xDE => Some("OEM_7".to_string()),
            0xDF => Some("OEM_8".to_string()),
            0xE2 => Some("OEM_102".to_string()),
            // Modifiers
            0xA2 => Some("LCTRL".to_string()),
            0xA3 => Some("RCTRL".to_string()),
            0xA4 => Some("LALT".to_string()),
            0xA5 => Some("RALT".to_string()),
            0xA0 => Some("LSHIFT".to_string()),
            0xA1 => Some("RSHIFT".to_string()),
            0x5B => Some("LWIN".to_string()),
            0x5C => Some("RWIN".to_string()),
            // Mouse buttons
            0x01 => Some("LBUTTON".to_string()),
            0x02 => Some("RBUTTON".to_string()),
            0x04 => Some("MBUTTON".to_string()),
            0x05 => Some("XBUTTON1".to_string()),
            0x06 => Some("XBUTTON2".to_string()),
            _ => None,
        }
    }
}
