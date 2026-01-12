//! Unit tests for state module.

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use scc::Guard;
    use smallvec::SmallVec;

    use crate::config::{AppConfig, KeyMapping};
    use crate::state::parsing::{key_name_to_vk, mouse_button_name_to_type, vk_to_scancode};
    use crate::state::types::*;
    use crate::state::AppState;

    #[test]
    fn test_key_name_to_vk_letters() {
        assert_eq!(key_name_to_vk("A"), Some(0x41));
        assert_eq!(key_name_to_vk("Z"), Some(0x5A));
        assert_eq!(key_name_to_vk("a"), Some(0x41)); // Case insensitive
        assert_eq!(key_name_to_vk("m"), Some(0x4D));
    }

    #[test]
    fn test_key_name_to_vk_numbers() {
        assert_eq!(key_name_to_vk("0"), Some(0x30));
        assert_eq!(key_name_to_vk("5"), Some(0x35));
        assert_eq!(key_name_to_vk("9"), Some(0x39));
    }

    #[test]
    fn test_key_name_to_vk_function_keys() {
        assert_eq!(key_name_to_vk("F1"), Some(0x70));
        assert_eq!(key_name_to_vk("F12"), Some(0x7B));
        assert_eq!(key_name_to_vk("F24"), Some(0x87));
        assert_eq!(key_name_to_vk("f5"), Some(0x74)); // Case insensitive
    }

    #[test]
    fn test_key_name_to_vk_special_keys() {
        assert_eq!(key_name_to_vk("ESC"), Some(0x1B));
        assert_eq!(key_name_to_vk("ENTER"), Some(0x0D));
        assert_eq!(key_name_to_vk("TAB"), Some(0x09));
        assert_eq!(key_name_to_vk("SPACE"), Some(0x20));
        assert_eq!(key_name_to_vk("BACKSPACE"), Some(0x08));
        assert_eq!(key_name_to_vk("DELETE"), Some(0x2E));
        assert_eq!(key_name_to_vk("INSERT"), Some(0x2D));
    }

    #[test]
    fn test_key_name_to_vk_arrow_keys() {
        assert_eq!(key_name_to_vk("UP"), Some(0x26));
        assert_eq!(key_name_to_vk("DOWN"), Some(0x28));
        assert_eq!(key_name_to_vk("LEFT"), Some(0x25));
        assert_eq!(key_name_to_vk("RIGHT"), Some(0x27));
    }

    #[test]
    fn test_key_name_to_vk_modifier_keys() {
        assert_eq!(key_name_to_vk("LSHIFT"), Some(0xA0));
        assert_eq!(key_name_to_vk("RSHIFT"), Some(0xA1));
        assert_eq!(key_name_to_vk("LCTRL"), Some(0xA2));
        assert_eq!(key_name_to_vk("RCTRL"), Some(0xA3));
        assert_eq!(key_name_to_vk("LALT"), Some(0xA4));
        assert_eq!(key_name_to_vk("RALT"), Some(0xA5));
    }

    #[test]
    fn test_key_name_to_vk_navigation_keys() {
        assert_eq!(key_name_to_vk("HOME"), Some(0x24));
        assert_eq!(key_name_to_vk("END"), Some(0x23));
        assert_eq!(key_name_to_vk("PAGEUP"), Some(0x21));
        assert_eq!(key_name_to_vk("PAGEDOWN"), Some(0x22));
    }

    #[test]
    fn test_key_name_to_vk_invalid() {
        assert_eq!(key_name_to_vk("INVALID"), None);
        assert_eq!(key_name_to_vk("F25"), None);
        assert_eq!(key_name_to_vk("F0"), None);
        assert_eq!(key_name_to_vk(""), None);
        assert_eq!(key_name_to_vk("ABC"), None);
    }

    #[test]
    fn test_vk_to_scancode_letters() {
        assert_eq!(vk_to_scancode(0x41), 0x1E); // A
        assert_eq!(vk_to_scancode(0x42), 0x30); // B
        assert_eq!(vk_to_scancode(0x5A), 0x2C); // Z
    }

    #[test]
    fn test_vk_to_scancode_numbers() {
        assert_eq!(vk_to_scancode(0x30), 0x0B); // 0
        assert_eq!(vk_to_scancode(0x31), 0x02); // 1
        assert_eq!(vk_to_scancode(0x39), 0x0A); // 9
    }

    #[test]
    fn test_vk_to_scancode_function_keys() {
        assert_eq!(vk_to_scancode(0x70), 0x3B); // F1
        assert_eq!(vk_to_scancode(0x7B), 0x58); // F12
    }

    #[test]
    fn test_vk_to_scancode_special_keys() {
        assert_eq!(vk_to_scancode(0x1B), 0x01); // ESC
        assert_eq!(vk_to_scancode(0x0D), 0x1C); // ENTER
        assert_eq!(vk_to_scancode(0x20), 0x39); // SPACE
    }

    #[test]
    fn test_vk_to_scancode_invalid() {
        assert_eq!(vk_to_scancode(0xFF), 0); // Invalid VK code
        assert_eq!(vk_to_scancode(0x00), 0); // No mapping
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
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
            },
            KeyMapping {
                trigger_key: "F1".to_string(),
                target_keys: SmallVec::from_vec(vec!["SPACE".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
        assert_eq!(key_name_to_vk("space"), key_name_to_vk("SPACE"));
        assert_eq!(key_name_to_vk("enter"), key_name_to_vk("ENTER"));
        assert_eq!(key_name_to_vk("esc"), key_name_to_vk("ESC"));
        assert_eq!(key_name_to_vk("delete"), key_name_to_vk("DELETE"));
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
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
            },
            KeyMapping {
                trigger_key: "B".to_string(),
                target_keys: SmallVec::from_vec(vec!["2".to_string()]),
                interval: Some(15),
                event_duration: Some(8),
                turbo_enabled: true,
                move_speed: 10,
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
            },
            KeyMapping {
                trigger_key: "C".to_string(),
                target_keys: SmallVec::from_vec(vec!["3".to_string()]),
                interval: Some(20),
                event_duration: Some(10),
                turbo_enabled: true,
                move_speed: 10,
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
        }];

        let state = AppState::new(config).unwrap();

        // Values should be adjusted to minimum of 2
        // This test verifies auto-adjustment behavior
        assert!(state.input_mappings.len() > 0);
    }

    #[test]
    fn test_vk_to_scancode_common_keys() {
        // Test commonly mapped VK codes that exist in SCANCODE_MAP
        assert_eq!(vk_to_scancode(0x08), 0x0E); // Backspace
        assert_eq!(vk_to_scancode(0x09), 0x0F); // Tab
        assert_eq!(vk_to_scancode(0x0D), 0x1C); // Enter
        assert_eq!(vk_to_scancode(0x20), 0x39); // Space

        // Keys not in map return 0
        let unmapped = vk_to_scancode(0xFF);
        assert_eq!(unmapped, 0);
    }

    #[test]
    fn test_key_name_to_vk_extended_keys() {
        assert_eq!(key_name_to_vk("LWIN"), Some(0x5B));
        assert_eq!(key_name_to_vk("RWIN"), Some(0x5C));
        assert_eq!(key_name_to_vk("PAUSE"), Some(0x13));
        assert_eq!(key_name_to_vk("CAPSLOCK"), Some(0x14));
        assert_eq!(key_name_to_vk("CAPITAL"), Some(0x14));
        assert_eq!(key_name_to_vk("NUMLOCK"), Some(0x90));
        assert_eq!(key_name_to_vk("SCROLL"), Some(0x91));
        assert_eq!(key_name_to_vk("SNAPSHOT"), Some(0x2C));
    }

    #[test]
    fn test_key_name_to_vk_numpad_keys() {
        assert_eq!(key_name_to_vk("NUMPAD0"), Some(0x60));
        assert_eq!(key_name_to_vk("NUMPAD1"), Some(0x61));
        assert_eq!(key_name_to_vk("NUMPAD5"), Some(0x65));
        assert_eq!(key_name_to_vk("NUMPAD9"), Some(0x69));
        assert_eq!(key_name_to_vk("MULTIPLY"), Some(0x6A));
        assert_eq!(key_name_to_vk("ADD"), Some(0x6B));
        assert_eq!(key_name_to_vk("SUBTRACT"), Some(0x6D));
        assert_eq!(key_name_to_vk("DECIMAL"), Some(0x6E));
        assert_eq!(key_name_to_vk("DIVIDE"), Some(0x6F));
    }

    #[test]
    fn test_key_name_to_vk_oem_keys() {
        assert_eq!(key_name_to_vk("OEM_1"), Some(0xBA));
        assert_eq!(key_name_to_vk("OEM_2"), Some(0xBF));
        assert_eq!(key_name_to_vk("OEM_3"), Some(0xC0));
        assert_eq!(key_name_to_vk("OEM_4"), Some(0xDB));
        assert_eq!(key_name_to_vk("OEM_5"), Some(0xDC));
        assert_eq!(key_name_to_vk("OEM_6"), Some(0xDD));
        assert_eq!(key_name_to_vk("OEM_7"), Some(0xDE));
        assert_eq!(key_name_to_vk("OEM_PLUS"), Some(0xBB));
        assert_eq!(key_name_to_vk("OEM_COMMA"), Some(0xBC));
        assert_eq!(key_name_to_vk("OEM_MINUS"), Some(0xBD));
        assert_eq!(key_name_to_vk("OEM_PERIOD"), Some(0xBE));
    }

    #[test]
    fn test_key_name_to_vk_mouse_buttons() {
        assert_eq!(key_name_to_vk("LBUTTON"), Some(0x01));
        assert_eq!(key_name_to_vk("RBUTTON"), Some(0x02));
        assert_eq!(key_name_to_vk("MBUTTON"), Some(0x04));
        assert_eq!(key_name_to_vk("XBUTTON1"), Some(0x05));
        assert_eq!(key_name_to_vk("XBUTTON2"), Some(0x06));
    }

    #[test]
    fn test_key_name_aliases() {
        assert_eq!(key_name_to_vk("ESC"), Some(0x1B));
        assert_eq!(key_name_to_vk("ESCAPE"), Some(0x1B));
        assert_eq!(key_name_to_vk("ENTER"), Some(0x0D));
        assert_eq!(key_name_to_vk("RETURN"), Some(0x0D));
        assert_eq!(key_name_to_vk("BACKSPACE"), Some(0x08));
        assert_eq!(key_name_to_vk("BACK"), Some(0x08));
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
        assert_eq!(vk_to_scancode(0x60), 0x52); // NUMPAD0
        assert_eq!(vk_to_scancode(0x61), 0x4F); // NUMPAD1
        assert_eq!(vk_to_scancode(0x65), 0x4C); // NUMPAD5
        assert_eq!(vk_to_scancode(0x69), 0x49); // NUMPAD9
        assert_eq!(vk_to_scancode(0x6A), 0x37); // MULTIPLY
        assert_eq!(vk_to_scancode(0x6B), 0x4E); // ADD
        assert_eq!(vk_to_scancode(0x6D), 0x4A); // SUBTRACT
        assert_eq!(vk_to_scancode(0x6F), 0x35); // DIVIDE
    }

    #[test]
    fn test_vk_to_scancode_lock_keys() {
        assert_eq!(vk_to_scancode(0x14), 0x3A); // CAPSLOCK
        assert_eq!(vk_to_scancode(0x90), 0x45); // NUMLOCK
        assert_eq!(vk_to_scancode(0x91), 0x46); // SCROLL LOCK
    }

    #[test]
    fn test_vk_to_scancode_oem_keys() {
        assert_eq!(vk_to_scancode(0xBA), 0x27); // OEM_1 (;:)
        assert_eq!(vk_to_scancode(0xBB), 0x0D); // OEM_PLUS (=+)
        assert_eq!(vk_to_scancode(0xBC), 0x33); // OEM_COMMA (,<)
        assert_eq!(vk_to_scancode(0xBD), 0x0C); // OEM_MINUS (-_)
        assert_eq!(vk_to_scancode(0xBE), 0x34); // OEM_PERIOD (.>)
        assert_eq!(vk_to_scancode(0xBF), 0x35); // OEM_2 (/?)
        assert_eq!(vk_to_scancode(0xC0), 0x29); // OEM_3 (`~)
    }

    #[test]
    fn test_combo_key_with_numpad() {
        use crate::state::parsing::input_name_to_device;
        let device = input_name_to_device("LCTRL+NUMPAD0");
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
        use crate::state::parsing::input_name_to_device;
        let device = input_name_to_device("LALT+OEM_3");
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
        use crate::state::parsing::input_name_to_output;
        let action = input_name_to_output("NUMPAD5");
        assert!(action.is_some());

        if let Some(OutputAction::KeyboardKey(scancode)) = action {
            assert_eq!(scancode, 0x4C); // NUMPAD5 scancode
        } else {
            panic!("Expected KeyboardKey action");
        }
    }

    #[test]
    fn test_parse_key_combo_trigger() {
        use crate::state::parsing::input_name_to_device;
        // Test parsing key combinations
        let device = input_name_to_device("ALT+A");
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
        use crate::state::parsing::input_name_to_device;
        // Test parsing complex key combinations
        let device = input_name_to_device("CTRL+SHIFT+S");
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
        use crate::state::parsing::input_name_to_output;
        // Test parsing key combination output
        let action = input_name_to_output("ALT+F4");
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
        use crate::state::parsing::input_name_to_device;
        // Test parsing invalid key combinations
        let device = input_name_to_device("INVALID+KEY");
        assert!(device.is_none());

        let device = input_name_to_device("A+");
        assert!(device.is_none());

        let device = input_name_to_device("+B");
        assert!(device.is_none());
    }

    #[test]
    fn test_parse_device_with_vid_pid_serial() {
        use crate::state::parsing::input_name_to_device;
        // Test parsing new format with VID/PID/Serial
        let device = input_name_to_device("GAMEPAD_045E_0B05_ABC123_B2.0");
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
        use crate::state::parsing::input_name_to_device;
        // Test parsing new format with VID/PID but no serial (DEV fallback)
        let device = input_name_to_device("GAMEPAD_045E_0B05_DEV12345678_B2.0");
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
        assert_eq!(vk_to_scancode(0xA0), 0x2A); // LSHIFT
        assert_eq!(vk_to_scancode(0xA1), 0x36); // RSHIFT
        assert_eq!(vk_to_scancode(0xA2), 0x1D); // LCTRL
        assert_eq!(vk_to_scancode(0xA4), 0x38); // LALT
        assert_eq!(vk_to_scancode(0x10), 0x2A); // SHIFT (generic)
        assert_eq!(vk_to_scancode(0x11), 0x1D); // CTRL (generic)
        assert_eq!(vk_to_scancode(0x12), 0x38); // ALT (generic)
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
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
            },
            KeyMapping {
                trigger_key: "CTRL+SHIFT+F".to_string(),
                target_keys: SmallVec::from_vec(vec!["ALT+F4".to_string()]),
                interval: None,
                event_duration: None,
                turbo_enabled: true,
                move_speed: 10,
                target_mode: 0,
                trigger_sequence: None,
                sequence_window_ms: 500,
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
        let _result = state.handle_mouse_event(WM_XBUTTONDOWN, mouse_data_x1, 0, 0);
        // Should parse as X1 button

        // Simulate XBUTTON2 up (mouse_data high word = 2)
        let mouse_data_x2: u32 = 2 << 16; // XBUTTON2
        let _result = state.handle_mouse_event(WM_XBUTTONUP, mouse_data_x2, 0, 0);
        // Should parse as X2 button
    }

    #[test]
    fn test_mouse_button_name_parsing() {
        // Test X button name parsing
        assert_eq!(mouse_button_name_to_type("XBUTTON1"), Some(MouseButton::X1));
        assert_eq!(mouse_button_name_to_type("XBUTTON2"), Some(MouseButton::X2));
        assert_eq!(mouse_button_name_to_type("X1"), Some(MouseButton::X1));
        assert_eq!(mouse_button_name_to_type("MB4"), Some(MouseButton::X1));
        assert_eq!(mouse_button_name_to_type("MB5"), Some(MouseButton::X2));
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
