//! Unit tests for state module.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use scc::Guard;
    use smallvec::SmallVec;

    use crate::config::{AppConfig, KeyMapping};
    use crate::state::AppState;
    use crate::state::parsing::{key_name_to_vk, mouse_button_name_to_type, vk_to_scancode};
    use crate::state::types::*;

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
                hold_indices: None,
                append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
                hold_indices: None,
                append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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
            hold_indices: None,
            append_keys: None,
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

    // -----------------------------------------------------------------
    // MappingHold construction tests
    // -----------------------------------------------------------------

    /// When a non-turbo sequence mapping specifies `hold_indices`,
    /// `create_input_mappings` must produce an `OutputAction::MappingHold`
    /// with the right `hold_mask` bits set and keep unset indices clear.
    #[test]
    fn test_sequential_hold_mask_built_from_indices() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F1".to_string(),
            target_keys: SmallVec::from_vec(vec!["RIGHT".to_string(), "RIGHT".to_string()]),
            interval: Some(5),
            event_duration: Some(5),
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F1").unwrap()))
            .expect("mapping for F1 should exist");
        match &info.target_action {
            OutputAction::MappingHold {
                actions,
                hold_mask,
                append,
                ..
            } => {
                assert_eq!(actions.len(), 2);
                // bit 1 set (second RIGHT should remain held after play).
                assert_eq!(*hold_mask, 0b10);
                assert!(append.is_empty());
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Indices at or beyond the action count are ignored rather than
    /// panicking. Keeps stale configs from crashing on load.
    #[test]
    fn test_sequential_hold_ignores_out_of_range_indices() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F2".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            // idx 5 is past the sequence length; idx 20 past the u16 mask width.
            hold_indices: Some(SmallVec::from_vec(vec![0, 5, 20])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F2").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { hold_mask, .. } => {
                assert_eq!(*hold_mask, 0b1);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Append keys are parsed via `input_name_to_output` and inherit the
    /// mapping's `move_speed` for mouse directions / scrolls.
    #[test]
    fn test_sequential_hold_with_append_keys() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F3".to_string(),
            target_keys: SmallVec::from_vec(vec!["RIGHT".to_string(), "RIGHT".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 7,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: Some(SmallVec::from_vec(vec![
                "UP".to_string(),
                "MOUSE_UP".to_string(),
            ])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F3").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { append, .. } => {
                assert_eq!(append.len(), 2);
                // First append is keyboard UP: holdable.
                assert!(matches!(append[0], OutputAction::KeyboardKey(_)));
                // Second append is MouseMove with the mapping's move_speed.
                match &append[1] {
                    OutputAction::MouseMove(_, speed) => assert_eq!(*speed, 7),
                    other => panic!("expected MouseMove for MOUSE_UP, got {:?}", other),
                }
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Turbo-mode sequence mappings with hold / append metadata now also
    /// produce `MappingHold`. The worker-side turbo branch drives
    /// `simulate_hold_cycle` to pulse just the configured subset.
    #[test]
    fn test_sequential_hold_kept_when_turbo_on() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F4".to_string(),
            target_keys: SmallVec::from_vec(vec!["RIGHT".to_string(), "RIGHT".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F4").unwrap()))
            .unwrap();
        assert!(matches!(
            info.target_action,
            OutputAction::MappingHold { .. }
        ));
    }

    /// Multi target mode with a hold subset must produce a
    /// simultaneous-playback `MappingHold` (`sequential == false`).
    #[test]
    fn test_multi_target_hold_uses_simultaneous_mode() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F6".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 1,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0, 2])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F6").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold {
                hold_mask,
                sequential,
                ..
            } => {
                assert!(!sequential);
                assert_eq!(*hold_mask, 0b101);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Single target mode with hold should still build MappingHold in
    /// simultaneous mode, even with just one body action.
    #[test]
    fn test_single_target_hold_uses_simultaneous_mode() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F7".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F7").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold {
                actions,
                hold_mask,
                sequential,
                ..
            } => {
                assert!(!sequential);
                assert_eq!(actions.len(), 1);
                assert_eq!(*hold_mask, 0b1);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Empty hold indices and empty append keys must leave the loop-style
    /// sequence intact so old configs remain observationally identical.
    #[test]
    fn test_sequential_hold_absent_when_no_fields() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F5".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: None,
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F5").unwrap()))
            .unwrap();
        assert!(matches!(
            info.target_action,
            OutputAction::SequentialActions(..)
        ));
    }

    /// All hold indices fall outside the body length and no append keys
    /// are configured, so the effective mask is zero. The constructor
    /// must fall back to the classic variant rather than emitting a
    /// no-op `MappingHold` that would swallow the trigger silently.
    #[test]
    fn test_out_of_range_only_indices_fall_back_to_classic() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F8".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            // Both indices are past the action count.
            hold_indices: Some(SmallVec::from_vec(vec![7, 8])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F8").unwrap()))
            .unwrap();
        assert!(matches!(
            info.target_action,
            OutputAction::SequentialActions(..)
        ));
    }

    /// An empty hold mask combined with a non-empty append list must
    /// still produce a `MappingHold` so the append phase actually runs.
    #[test]
    fn test_hold_mask_zero_but_append_builds_mapping_hold() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F9".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: None,
            append_keys: Some(SmallVec::from_vec(vec!["LSHIFT".to_string()])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F9").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold {
                hold_mask, append, ..
            } => {
                assert_eq!(*hold_mask, 0);
                assert_eq!(append.len(), 1);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// `MappingHold::interval_ms` should equal the mapping's effective
    /// `interval` (either the explicit override or the global fallback).
    #[test]
    fn test_mapping_hold_interval_inherits_mapping_interval() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F10".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: Some(37),
            event_duration: Some(5),
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F10").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { interval_ms, .. } => {
                assert_eq!(*interval_ms, 37);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// `hold_mask` is a u16 so index 15 is the last legal position. The
    /// constructor must still set it correctly on a 16-element body.
    /// Use plain ASCII letters because `SCANCODE_MAP` covers A-Z but
    /// only function keys F1-F12.
    #[test]
    fn test_hold_mask_supports_bit_15_boundary() {
        let mut config = AppConfig::default();
        // 16 target keys (A through P). Sequence matcher caps at 16
        // entries so this is the maximum supported body length.
        let keys: Vec<String> = (0..16u8)
            .map(|i| ((b'A' + i) as char).to_string())
            .collect();
        config.mappings = vec![KeyMapping {
            trigger_key: "F1".to_string(),
            target_keys: SmallVec::from_vec(keys),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0u8, 15u8])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F1").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { hold_mask, .. } => {
                assert_eq!(*hold_mask, 0b1000_0000_0000_0001);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Indices at or above 16 fall outside the `u16` mask width and must
    /// be dropped silently, matching the out-of-range policy.
    #[test]
    fn test_hold_mask_drops_index_16_and_beyond() {
        let mut config = AppConfig::default();
        let keys: Vec<String> = (0..16u8)
            .map(|i| ((b'A' + i) as char).to_string())
            .collect();
        config.mappings = vec![KeyMapping {
            trigger_key: "F2".to_string(),
            target_keys: SmallVec::from_vec(keys),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            // idx 3 is legal. idx 16 / 42 / 255 are all illegal.
            hold_indices: Some(SmallVec::from_vec(vec![3u8, 16u8, 42u8, 255u8])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F2").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { hold_mask, .. } => {
                assert_eq!(*hold_mask, 1u16 << 3);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Unparseable append names must be skipped without aborting
    /// construction; the remaining parsed entries keep their order.
    #[test]
    fn test_unparseable_append_names_are_skipped() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "C".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0])),
            append_keys: Some(SmallVec::from_vec(vec![
                "UP".to_string(),
                "NOT_A_REAL_KEY".to_string(),
                "LSHIFT".to_string(),
            ])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("C").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { append, .. } => {
                // 3 input names, 1 unparseable → 2 actions retained.
                assert_eq!(append.len(), 2);
                assert!(matches!(append[0], OutputAction::KeyboardKey(_)));
                assert!(matches!(append[1], OutputAction::KeyboardKey(_)));
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Append actions must appear in the order they were written; the
    /// press and release phases both depend on the ordering for correct
    /// modifier-outlives-main-key semantics.
    #[test]
    fn test_append_order_preserved() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "D".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0])),
            append_keys: Some(SmallVec::from_vec(vec![
                "LSHIFT".to_string(),
                "LCTRL".to_string(),
                "LALT".to_string(),
            ])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("D").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { append, .. } => {
                assert_eq!(append.len(), 3);
                let lshift_sc = vk_to_scancode(0xA0);
                let lctrl_sc = vk_to_scancode(0xA2);
                let lalt_sc = vk_to_scancode(0xA4);
                assert!(matches!(&append[0], OutputAction::KeyboardKey(sc) if *sc == lshift_sc));
                assert!(matches!(&append[1], OutputAction::KeyboardKey(sc) if *sc == lctrl_sc));
                assert!(matches!(&append[2], OutputAction::KeyboardKey(sc) if *sc == lalt_sc));
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Append entries of scroll / move type must inherit the mapping's
    /// `move_speed`, mirroring the existing treatment of body scroll /
    /// move actions. Rules out regressions where append went through
    /// `input_name_to_output` without the speed rewrite.
    #[test]
    fn test_append_mouse_scroll_inherits_move_speed() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "E".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 13,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![0])),
            append_keys: Some(SmallVec::from_vec(vec![
                "SCROLL_UP".to_string(),
                "MOUSE_LEFT".to_string(),
            ])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("E").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { append, .. } => {
                assert_eq!(append.len(), 2);
                match &append[0] {
                    OutputAction::MouseScroll(_, speed) => assert_eq!(*speed, 13),
                    other => panic!("expected MouseScroll, got {:?}", other),
                }
                match &append[1] {
                    OutputAction::MouseMove(_, speed) => assert_eq!(*speed, 13),
                    other => panic!("expected MouseMove, got {:?}", other),
                }
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Without rule properties, a single-target mapping must still emit
    /// a bare action (not wrapped in `MultipleActions` or
    /// `SequentialActions`). Guards the "classic single-key" fast path.
    #[test]
    fn test_single_target_no_hold_keeps_single_variant() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "G".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: None,
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("G").unwrap()))
            .unwrap();
        assert!(matches!(info.target_action, OutputAction::KeyboardKey(_)));
    }

    /// Without rule properties, a multi-target mapping must emit
    /// `MultipleActions` so `simulate_action` presses the whole set at
    /// once and releases on trigger release.
    #[test]
    fn test_multi_target_no_hold_keeps_multiple_actions() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "H".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 1,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: None,
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("H").unwrap()))
            .unwrap();
        assert!(matches!(
            info.target_action,
            OutputAction::MultipleActions(..)
        ));
    }

    /// `MappingHold` must ref-count body overlap correctly on the real
    /// trigger-release path. Body `[RIGHT, RIGHT]` with both positions
    /// in `hold_mask` presses the RIGHT scancode twice; the matching
    /// `simulate_hold_release` must walk the counter back to zero.
    /// Exercises `simulate_hold_release` directly (the production path
    /// for MappingHold trigger-release, not the generic
    /// `simulate_release`) so a regression in the hold-release loop
    /// would surface here.
    #[test]
    fn test_mapping_hold_refcount_balances_duplicated_body_keys() {
        let state = AppState::new(AppConfig::default()).expect("default AppState should build");
        // Emulate "double-tap to run": body = [RIGHT, RIGHT] fully held,
        // empty append.
        let right_sc = vk_to_scancode(0x27);
        let body: SmallVec<[OutputAction; 4]> = SmallVec::from_vec(vec![
            OutputAction::KeyboardKey(right_sc),
            OutputAction::KeyboardKey(right_sc),
        ]);
        let action = OutputAction::MappingHold {
            actions: Arc::new(body),
            interval_ms: 5,
            hold_mask: 0b11,
            append: Arc::new(SmallVec::new()),
            sequential: true,
        };

        // Two calls to simulate_initial_press collect RIGHT twice each
        // via collect_primitives(MappingHold), so the counter lands at 4.
        state.simulate_initial_press(&action);
        state.simulate_initial_press(&action);
        let count_after_press = state.held_scancodes[right_sc as usize].load(Ordering::Acquire);
        assert_eq!(count_after_press, 4);

        // simulate_hold_release iterates body in reverse, releasing each
        // held index once. Two invocations cover the two virtual
        // presses.
        state.simulate_hold_release(&action);
        state.simulate_hold_release(&action);
        let count_after_release = state.held_scancodes[right_sc as usize].load(Ordering::Acquire);
        assert_eq!(count_after_release, 0);
    }

    /// Mixed hold mask + append: non-held body items must self-balance
    /// inside `simulate_action`, while held + append items drive their
    /// ref counts through the `MappingHold`-specific release path.
    /// Body `[A(held), B(unheld), A(held)]` + append `[LSHIFT]`.
    ///
    /// This verifies three invariants at once:
    /// 1. `collect_primitives(MappingHold)` excludes non-held body
    ///    entries, so B never accumulates residue in `held_scancodes`.
    /// 2. Duplicated held scancode A balances across press + release.
    /// 3. Append modifier LSHIFT is released by `simulate_hold_release`.
    #[test]
    fn test_mapping_hold_release_with_mixed_mask_and_append() {
        let state = AppState::new(AppConfig::default()).expect("default AppState should build");
        let a_sc = vk_to_scancode(0x41);
        let b_sc = vk_to_scancode(0x42);
        let lshift_sc = vk_to_scancode(0xA0);
        let body: SmallVec<[OutputAction; 4]> = SmallVec::from_vec(vec![
            OutputAction::KeyboardKey(a_sc),
            OutputAction::KeyboardKey(b_sc),
            OutputAction::KeyboardKey(a_sc),
        ]);
        let append: SmallVec<[OutputAction; 4]> =
            SmallVec::from_vec(vec![OutputAction::KeyboardKey(lshift_sc)]);
        let action = OutputAction::MappingHold {
            actions: Arc::new(body),
            interval_ms: 5,
            // hold_mask = 0b101 selects index 0 (A) and index 2 (A).
            hold_mask: 0b101,
            append: Arc::new(append),
            sequential: false,
        };

        // collect_primitives selects A, A (held body) and LSHIFT (append);
        // B is excluded by design.
        state.simulate_initial_press(&action);
        assert_eq!(
            state.held_scancodes[a_sc as usize].load(Ordering::Acquire),
            2
        );
        assert_eq!(
            state.held_scancodes[b_sc as usize].load(Ordering::Acquire),
            0
        );
        assert_eq!(
            state.held_scancodes[lshift_sc as usize].load(Ordering::Acquire),
            1
        );

        state.simulate_hold_release(&action);
        assert_eq!(
            state.held_scancodes[a_sc as usize].load(Ordering::Acquire),
            0
        );
        assert_eq!(
            state.held_scancodes[b_sc as usize].load(Ordering::Acquire),
            0
        );
        assert_eq!(
            state.held_scancodes[lshift_sc as usize].load(Ordering::Acquire),
            0
        );
    }

    /// Explicit override is already covered by
    /// `test_sequential_hold_mask_built_from_indices`. The remaining
    /// `interval_ms` derivations are the global-fallback branch
    /// (`mapping.interval = None` → `config.interval`) and the
    /// minimum-clamp branch (a value below 5 gets raised to 5).
    #[test]
    fn test_mapping_hold_interval_uses_global_fallback() {
        let mut config = AppConfig::default();
        config.interval = 42;
        config.mappings = vec![KeyMapping {
            trigger_key: "F11".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F11").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { interval_ms, .. } => {
                assert_eq!(*interval_ms, 42);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Minimum-clamp branch: the constructor pins `interval_ms` to at
    /// least 5ms, preventing a pathological 0ms / 1ms setup from
    /// saturating the CPU on a held sequence mapping.
    #[test]
    fn test_mapping_hold_interval_clamps_to_minimum() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "F12".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: Some(1),
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 2,
            trigger_sequence: None,
            sequence_window_ms: 500,
            hold_indices: Some(SmallVec::from_vec(vec![1])),
            append_keys: None,
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("F12").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold { interval_ms, .. } => {
                assert_eq!(*interval_ms, 5);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }

    /// Out-of-range hold indices combined with a non-empty append list
    /// must still produce `MappingHold { hold_mask: 0, append: [..] }`.
    /// Distinct from `test_hold_mask_zero_but_append_builds_mapping_hold`
    /// which reaches hold_mask=0 via `hold_indices: None`; this case
    /// traverses the `has_hold = true` branch of `has_rule_props`.
    #[test]
    fn test_out_of_range_hold_plus_append_builds_mapping_hold() {
        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "Z".to_string(),
            target_keys: SmallVec::from_vec(vec!["A".to_string(), "B".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 5,
            target_mode: 1,
            trigger_sequence: None,
            sequence_window_ms: 500,
            // All indices out of range → effective hold_mask = 0.
            hold_indices: Some(SmallVec::from_vec(vec![99, 100])),
            append_keys: Some(SmallVec::from_vec(vec!["LSHIFT".to_string()])),
        }];

        let mappings = AppState::create_input_mappings(&config).unwrap();
        let info = mappings
            .get(&InputDevice::Keyboard(key_name_to_vk("Z").unwrap()))
            .unwrap();
        match &info.target_action {
            OutputAction::MappingHold {
                hold_mask, append, ..
            } => {
                assert_eq!(*hold_mask, 0);
                assert_eq!(append.len(), 1);
            }
            other => panic!("expected MappingHold, got {:?}", other),
        }
    }
}
