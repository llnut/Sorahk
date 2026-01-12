//! Integration tests for Sorahk application.
//!
//! Tests verify interactions between different modules
//! and check that application components work together as expected.

use smallvec::SmallVec;
use sorahk::config::{AppConfig, KeyMapping};
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
        },
        KeyMapping {
            trigger_key: "B".to_string(),
            target_keys: SmallVec::from_vec(vec!["2".to_string()]),
            interval: None,
            event_duration: None,
            turbo_enabled: true,
            move_speed: 10,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
        },
        KeyMapping {
            trigger_key: "F1".to_string(),
            target_keys: SmallVec::from_vec(vec!["SPACE".to_string()]),
            interval: Some(20),
            event_duration: Some(10),
            turbo_enabled: true,
            move_speed: 10,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
        },
        KeyMapping {
            trigger_key: "LSHIFT".to_string(),
            target_keys: SmallVec::from_vec(vec!["ENTER".to_string()]),
            interval: Some(15),
            event_duration: Some(8),
            turbo_enabled: true,
            move_speed: 10,
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
        })
        .collect();

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 50);

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
            target_mode: 0,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
            target_mode: 0,
            move_speed: 10,
            trigger_sequence: None,
            sequence_window_ms: 500,
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
        target_mode: 0,
        interval: Some(10),
        event_duration: Some(5),
        turbo_enabled: true,
        move_speed: 10,
        trigger_sequence: None,
        sequence_window_ms: 500,
    }];

    config.save_to_file(&path).expect("Failed to save config");
    let loaded_config = AppConfig::load_from_file(&path).expect("Failed to load config");

    assert_eq!(loaded_config.mappings.len(), 1);
    assert_eq!(loaded_config.mappings[0].target_keys.len(), 5);
    assert_eq!(loaded_config.mappings[0].target_keys[0], "1");
    assert_eq!(loaded_config.mappings[0].target_keys[4], "5");

    cleanup_test_file(&path);
}
