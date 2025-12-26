//! Tests for application state management.

use smallvec::SmallVec;
use sorahk::config::{AppConfig, KeyMapping};
use sorahk::state::{AppState, CaptureMode, InputDevice, MouseButton};

#[test]
fn test_state_creation_from_config() {
    let config = AppConfig::default();
    let state = AppState::new(config).expect("Failed to create state");

    assert!(!state.is_paused());
    assert!(!state.should_exit());
}

#[test]
fn test_state_pause_toggle() {
    let config = AppConfig::default();
    let state = AppState::new(config).expect("Failed to create state");

    assert!(!state.is_paused());

    state.toggle_paused();
    assert!(state.is_paused());

    state.toggle_paused();
    assert!(!state.is_paused());
}

#[test]
fn test_state_input_mapping_lookup() {
    let mut config = AppConfig::default();
    config.mappings = vec![KeyMapping {
        trigger_key: "A".to_string(),
        target_keys: SmallVec::from_vec(vec!["B".to_string()]),
        interval: Some(10),
        event_duration: Some(5),
        turbo_enabled: true,
        move_speed: 10,
    }];

    let state = AppState::new(config).expect("Failed to create state");

    let device = InputDevice::Keyboard(0x41);
    let mapping = state.get_input_mapping(&device);

    assert!(mapping.is_some());
    let mapping = mapping.unwrap();
    assert_eq!(mapping.interval, 10);
    assert_eq!(mapping.event_duration, 5);
    assert!(mapping.turbo_enabled);
}

#[test]
fn test_state_reload_config() {
    let mut config = AppConfig::default();
    config.interval = 10;
    config.event_duration = 8;

    let state = AppState::new(config.clone()).expect("Failed to create state");

    config.interval = 20;
    config.event_duration = 15;

    state
        .reload_config(config)
        .expect("Failed to reload config");

    assert_eq!(state.input_timeout(), 5);
}

#[test]
fn test_state_capture_mode_toggle() {
    let config = AppConfig::default();
    let state = AppState::new(config).expect("Failed to create state");

    assert!(!state.is_raw_input_capture_active());

    state.set_raw_input_capture_mode(true);
    assert!(state.is_raw_input_capture_active());

    state.set_raw_input_capture_mode(false);
    assert!(!state.is_raw_input_capture_active());
}

#[test]
fn test_state_capture_mode_values() {
    let modes = CaptureMode::all_modes();
    assert!(modes.len() >= 5);

    assert_eq!(CaptureMode::MostSustained.as_str(), "MostSustained");
    assert_eq!(
        CaptureMode::AdaptiveIntelligent.as_str(),
        "AdaptiveIntelligent"
    );
    assert_eq!(CaptureMode::MaxChangedBits.as_str(), "MaxChangedBits");
    assert_eq!(CaptureMode::MaxSetBits.as_str(), "MaxSetBits");
    assert_eq!(CaptureMode::LastStable.as_str(), "LastStable");
}

#[test]
fn test_state_input_device_keyboard() {
    let device = InputDevice::Keyboard(0x41);
    match device {
        InputDevice::Keyboard(vk) => assert_eq!(vk, 0x41),
        _ => panic!("Expected keyboard device"),
    }
}

#[test]
fn test_state_input_device_mouse() {
    let device = InputDevice::Mouse(MouseButton::Left);
    match device {
        InputDevice::Mouse(button) => assert_eq!(button, MouseButton::Left),
        _ => panic!("Expected mouse device"),
    }
}

#[test]
fn test_state_input_device_key_combo() {
    let device = InputDevice::KeyCombo(vec![0x10, 0x11, 0x41]);
    match device {
        InputDevice::KeyCombo(keys) => {
            assert_eq!(keys.len(), 3);
            assert_eq!(keys[0], 0x10);
            assert_eq!(keys[1], 0x11);
            assert_eq!(keys[2], 0x41);
        }
        _ => panic!("Expected key combo device"),
    }
}

#[test]
fn test_state_multiple_mappings() {
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
            interval: None,
            event_duration: None,
            turbo_enabled: false,
            move_speed: 10,
        },
    ];

    let state = AppState::new(config).expect("Failed to create state");

    let device_a = InputDevice::Keyboard(0x41);
    let mapping_a = state.get_input_mapping(&device_a);
    assert!(mapping_a.is_some());
    assert_eq!(mapping_a.unwrap().interval, 10);

    let device_b = InputDevice::Keyboard(0x42);
    let mapping_b = state.get_input_mapping(&device_b);
    assert!(mapping_b.is_some());
    assert_eq!(mapping_b.unwrap().interval, 15);

    let device_c = InputDevice::Keyboard(0x43);
    let mapping_c = state.get_input_mapping(&device_c);
    assert!(mapping_c.is_some());
    assert!(!mapping_c.unwrap().turbo_enabled);
}

#[test]
fn test_state_nonexistent_mapping() {
    let config = AppConfig::default();
    let state = AppState::new(config).expect("Failed to create state");

    let device = InputDevice::Keyboard(0xFF);
    let mapping = state.get_input_mapping(&device);

    assert!(mapping.is_none());
}

#[test]
fn test_state_config_reload_clears_mappings() {
    let mut config = AppConfig::default();
    config.mappings = vec![KeyMapping {
        trigger_key: "A".to_string(),
        target_keys: SmallVec::from_vec(vec!["B".to_string()]),
        interval: Some(10),
        event_duration: Some(5),
        turbo_enabled: true,
        move_speed: 10,
    }];

    let state = AppState::new(config.clone()).expect("Failed to create state");

    let device = InputDevice::Keyboard(0x41);
    assert!(state.get_input_mapping(&device).is_some());

    config.mappings.clear();
    state
        .reload_config(config)
        .expect("Failed to reload config");

    assert!(state.get_input_mapping(&device).is_none());
}

#[test]
fn test_state_turbo_enabled_in_mapping() {
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
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: false,
            move_speed: 10,
        },
    ];

    let state = AppState::new(config).expect("Failed to create state");

    let device_a = InputDevice::Keyboard(0x41);
    let mapping_a = state.get_input_mapping(&device_a);
    assert!(mapping_a.is_some());
    assert!(mapping_a.unwrap().turbo_enabled);

    let device_b = InputDevice::Keyboard(0x42);
    let mapping_b = state.get_input_mapping(&device_b);
    assert!(mapping_b.is_some());
    assert!(!mapping_b.unwrap().turbo_enabled);
}
