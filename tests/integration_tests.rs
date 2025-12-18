//! Integration tests for Sorahk application.
//!
//! Tests verify interactions between different modules
//! and check that application components work together as expected.

use smallvec::SmallVec;
use sorahk::config::{AppConfig, KeyMapping};
use sorahk::i18n::Language;
use std::fs;
use std::path::PathBuf;

/// Returns a unique temporary file path for test isolation.
fn get_test_file_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "sorahk_integration_test_{}_{}.toml",
        name,
        std::process::id()
    ));
    path
}

/// Removes a test file if it exists.
fn cleanup_test_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

/// Tests configuration save and load cycle preserves data.
#[test]
fn test_config_round_trip() {
    let path = get_test_file_path("round_trip");

    let config = AppConfig::default();

    config.save_to_file(&path).expect("Failed to save config");

    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(config.show_tray_icon, loaded_config.show_tray_icon);
    assert_eq!(config.show_notifications, loaded_config.show_notifications);
    assert_eq!(config.switch_key, loaded_config.switch_key);
    assert_eq!(config.interval, loaded_config.interval);
    assert_eq!(config.mappings.len(), loaded_config.mappings.len());

    cleanup_test_file(&path);
}

/// Tests configuration with multiple key mappings having mixed properties.
#[test]
fn test_config_with_complex_mappings() {
    let path = get_test_file_path("complex_mappings");

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
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        },
        KeyMapping {
            trigger_key: "F1".to_string(),
            target_keys: SmallVec::from_vec(vec!["SPACE".to_string()]),
            interval: Some(20),
            event_duration: Some(10),
            turbo_enabled: true,
            move_speed: 10,
        },
        KeyMapping {
            trigger_key: "LSHIFT".to_string(),
            target_keys: SmallVec::from_vec(vec!["ENTER".to_string()]),
            interval: Some(15),
            event_duration: Some(8),
            turbo_enabled: true,
            move_speed: 10,
        },
    ];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 4);

    assert_eq!(loaded_config.mappings[0].trigger_key, "A");
    assert_eq!(
        loaded_config.mappings[0].target_keys.as_slice(),
        &["1".to_string()]
    );
    assert_eq!(loaded_config.mappings[0].interval, Some(10));
    assert_eq!(loaded_config.mappings[0].event_duration, Some(5));

    assert_eq!(loaded_config.mappings[1].trigger_key, "B");
    assert_eq!(loaded_config.mappings[1].interval, None);

    cleanup_test_file(&path);
}

/// Tests process whitelist persistence across save/load operations.
#[test]
fn test_config_with_process_whitelist() {
    let path = get_test_file_path("process_whitelist");

    let mut config = AppConfig::default();
    config.process_whitelist = vec![
        "notepad.exe".to_string(),
        "chrome.exe".to_string(),
        "game.exe".to_string(),
    ];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.process_whitelist.len(), 3);
    assert!(
        loaded_config
            .process_whitelist
            .contains(&"notepad.exe".to_string())
    );
    assert!(
        loaded_config
            .process_whitelist
            .contains(&"chrome.exe".to_string())
    );
    assert!(
        loaded_config
            .process_whitelist
            .contains(&"game.exe".to_string())
    );

    cleanup_test_file(&path);
}

/// Tests language setting persistence for all supported languages.
#[test]
fn test_config_language_persistence() {
    let path = get_test_file_path("language_persistence");

    let languages = vec![
        Language::English,
        Language::SimplifiedChinese,
        Language::TraditionalChinese,
        Language::Japanese,
    ];

    for lang in languages {
        let mut config = AppConfig::default();
        config.language = lang;

        config.save_to_file(&path).expect("Failed to save config");
        let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

        assert_eq!(loaded_config.language, lang);
    }

    cleanup_test_file(&path);
}

/// Tests that invalid configuration values are corrected on load.
#[test]
fn test_config_validation_on_load() {
    let path = get_test_file_path("validation");

    let content = r#"
        show_tray_icon = true
        show_notifications = false
        switch_key = "F12"
        input_timeout = 1
        interval = 2
        event_duration = 3
        worker_count = 8
        process_whitelist = ["test.exe"]
        
        [[mappings]]
        trigger_key = "Q"
        target_keys = ["W"]
    "#;

    fs::write(&path, content).expect("Failed to write test config");

    let config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert!(
        config.input_timeout >= 2,
        "Input timeout below minimum is adjusted"
    );
    assert!(config.interval >= 2, "Interval below minimum is adjusted");
    assert!(
        config.event_duration >= 2,
        "Event duration below minimum is adjusted"
    );

    cleanup_test_file(&path);
}

/// Tests that missing configuration fields use default values.
#[test]
fn test_config_default_values() {
    let path = get_test_file_path("default_values");

    let content = r#"
        show_tray_icon = false
        show_notifications = false
        switch_key = "DELETE"
        process_whitelist = []
        
        [[mappings]]
        trigger_key = "A"
        target_keys = ["B"]
    "#;

    fs::write(&path, content).expect("Failed to write test config");

    let config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(config.input_timeout, 5);
    assert_eq!(config.interval, 5);
    assert_eq!(config.event_duration, 5);
    assert_eq!(config.worker_count, 0);

    cleanup_test_file(&path);
}

/// Tests configuration with no key mappings defined.
#[test]
fn test_config_empty_mappings() {
    let path = get_test_file_path("empty_mappings");

    let mut config = AppConfig::default();
    config.mappings = vec![];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert!(loaded_config.mappings.is_empty());

    cleanup_test_file(&path);
}

/// Tests configuration with empty process whitelist.
#[test]
fn test_config_empty_process_whitelist() {
    let path = get_test_file_path("empty_whitelist");

    let mut config = AppConfig::default();
    config.process_whitelist = vec![];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert!(loaded_config.process_whitelist.is_empty());

    cleanup_test_file(&path);
}

/// Tests configuration with a large number of key mappings.
#[test]
fn test_config_maximum_mappings() {
    let path = get_test_file_path("max_mappings");

    let mut config = AppConfig::default();
    config.mappings = (0..50)
        .map(|i| KeyMapping {
            trigger_key: format!("F{}", (i % 12) + 1),
            target_keys: SmallVec::from_vec(vec![format!("{}", i % 10)]),
            interval: Some((i as u64 + 1) * 5),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        })
        .collect();

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 50);

    cleanup_test_file(&path);
}

/// Tests configuration with all available settings modified.
#[test]
fn test_config_all_settings_combined() {
    let path = get_test_file_path("all_settings");

    let mut config = AppConfig::default();
    config.show_tray_icon = false;
    config.show_notifications = true;
    config.always_on_top = true;
    config.dark_mode = true;
    config.language = Language::Japanese;
    config.switch_key = "F11".to_string();
    config.input_timeout = 15;
    config.interval = 8;
    config.event_duration = 6;
    config.worker_count = 6;
    config.process_whitelist = vec!["app1.exe".to_string(), "app2.exe".to_string()];
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
            trigger_key: "C".to_string(),
            target_keys: SmallVec::from_vec(vec!["D".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
        },
    ];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.show_tray_icon, false);
    assert_eq!(loaded_config.show_notifications, true);
    assert_eq!(loaded_config.always_on_top, true);
    assert_eq!(loaded_config.dark_mode, true);
    assert_eq!(loaded_config.language, Language::Japanese);
    assert_eq!(loaded_config.switch_key, "F11");
    assert_eq!(loaded_config.input_timeout, 15);
    assert_eq!(loaded_config.interval, 8);
    assert_eq!(loaded_config.event_duration, 6);
    assert_eq!(loaded_config.worker_count, 6);
    assert_eq!(loaded_config.process_whitelist.len(), 2);
    assert_eq!(loaded_config.mappings.len(), 2);

    cleanup_test_file(&path);
}

/// Tests that saved configuration files contain expected section headers.
#[test]
fn test_config_file_format_preservation() {
    let path = get_test_file_path("format_preservation");

    let config = AppConfig::default();
    config.save_to_file(&path).expect("Failed to save config");

    let content = fs::read_to_string(&path).expect("Failed to read config file");

    assert!(content.contains("Sorahk Configuration File"));
    assert!(content.contains("General Settings"));
    assert!(content.contains("Performance Settings"));
    assert!(content.contains("Control Settings"));
    assert!(content.contains("Process Whitelist"));
    assert!(content.contains("Input Mappings"));

    cleanup_test_file(&path);
}

/// Tests multiple threads performing configuration operations simultaneously.
#[test]
fn test_concurrent_config_operations() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let barrier = Arc::new(Barrier::new(5));
    let mut handles = vec![];

    for i in 0..5 {
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            let path = get_test_file_path(&format!("concurrent_{}", i));

            let mut config = AppConfig::default();
            config.worker_count = i;

            barrier_clone.wait();

            config.save_to_file(&path).expect("Failed to save config");
            let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

            assert_eq!(loaded_config.worker_count, i);

            cleanup_test_file(&path);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

/// Tests multiple target keys in configuration.
#[test]
fn test_config_multiple_target_keys() {
    let path = get_test_file_path("multiple_target_keys");

    let mut config = AppConfig::default();
    config.mappings = vec![
        KeyMapping {
            trigger_key: "Q".to_string(),
            target_keys: SmallVec::from_vec(vec!["MOUSE_UP".to_string(), "MOUSE_LEFT".to_string()]),
            interval: Some(5),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        },
        KeyMapping {
            trigger_key: "E".to_string(),
            target_keys: SmallVec::from_vec(vec![
                "MOUSE_UP".to_string(),
                "MOUSE_RIGHT".to_string(),
            ]),
            interval: Some(5),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        },
    ];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 2);
    assert_eq!(loaded_config.mappings[0].target_keys.len(), 2);
    assert_eq!(loaded_config.mappings[0].target_keys[0], "MOUSE_UP");
    assert_eq!(loaded_config.mappings[0].target_keys[1], "MOUSE_LEFT");
    assert_eq!(loaded_config.mappings[1].target_keys.len(), 2);
    assert_eq!(loaded_config.mappings[1].target_keys[0], "MOUSE_UP");
    assert_eq!(loaded_config.mappings[1].target_keys[1], "MOUSE_RIGHT");

    cleanup_test_file(&path);
}

/// Tests empty target keys list in configuration.
#[test]
fn test_config_empty_target_keys() {
    let path = get_test_file_path("empty_target_keys");

    let mut config = AppConfig::default();
    config.mappings = vec![KeyMapping {
        trigger_key: "A".to_string(),
        target_keys: SmallVec::new(),
        interval: Some(10),
        event_duration: Some(5),
        turbo_enabled: true,
        move_speed: 10,
    }];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 1);
    assert_eq!(loaded_config.mappings[0].target_keys.len(), 0);

    cleanup_test_file(&path);
}

/// Tests many target keys in a single mapping.
#[test]
fn test_config_many_target_keys() {
    let path = get_test_file_path("many_target_keys");

    let mut config = AppConfig::default();
    config.mappings = vec![KeyMapping {
        trigger_key: "A".to_string(),
        target_keys: SmallVec::from_vec(vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ]),
        interval: Some(10),
        event_duration: Some(5),
        turbo_enabled: true,
        move_speed: 10,
    }];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 1);
    assert_eq!(loaded_config.mappings[0].target_keys.len(), 5);
    assert_eq!(loaded_config.mappings[0].target_keys[0], "1");
    assert_eq!(loaded_config.mappings[0].target_keys[4], "5");

    cleanup_test_file(&path);
}

/// Tests target_keys field backward compatibility with TOML.
#[test]
fn test_config_target_keys_toml_format() {
    let path = get_test_file_path("target_keys_toml");

    let content = r#"
        show_tray_icon = true
        show_notifications = true
        switch_key = "DELETE"
        input_timeout = 5
        interval = 5
        event_duration = 5
        worker_count = 0
        process_whitelist = []
        
        [[mappings]]
        trigger_key = "Q"
        target_keys = ["MOUSE_UP", "MOUSE_LEFT"]
        turbo_enabled = true
        
        [[mappings]]
        trigger_key = "E"
        target_keys = ["A"]
        turbo_enabled = true
    "#;

    fs::write(&path, content).expect("Failed to write test config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 2);
    assert_eq!(loaded_config.mappings[0].target_keys.len(), 2);
    assert_eq!(loaded_config.mappings[0].target_keys[0], "MOUSE_UP");
    assert_eq!(loaded_config.mappings[0].target_keys[1], "MOUSE_LEFT");
    assert_eq!(loaded_config.mappings[1].target_keys.len(), 1);
    assert_eq!(loaded_config.mappings[1].target_keys[0], "A");

    cleanup_test_file(&path);
}
