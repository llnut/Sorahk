// Hide console window in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gui;
mod i18n;
mod input_manager;
mod input_ownership;
mod keyboard;
mod mouse;
mod rawinput;
mod sequence_matcher;
mod signal;
mod state;
mod tray;
mod util;
mod xinput;

use std::sync::Arc;
use std::thread;

use anyhow::Result;
use config::AppConfig;
use gui::{SorahkGui, show_error};
use input_manager::InputManager;
use keyboard::KeyboardHook;
use mouse::MouseHook;
use state::AppState;
use tray::TrayIcon;
use windows::Win32::Media::timeBeginPeriod;

fn main() -> Result<()> {
    // Request 1ms timer resolution for precise timing in input processing
    unsafe { timeBeginPeriod(1) };

    signal::set_control_ctrl_handler()?;

    // Load config or create default if not exists
    let config = match AppConfig::load_or_create("Config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            let error_msg = format!("Failed to load configuration: {}", e);
            return show_error(&error_msg);
        }
    };

    let app_state = Arc::new(match AppState::new(config.clone()) {
        Ok(state) => state,
        Err(e) => {
            let error_msg = format!("Failed to initialize application state: {}", e);
            return show_error(&error_msg);
        }
    });

    // Start keyboard hook in a separate thread BEFORE GUI
    // Create hook INSIDE the thread to ensure proper message loop
    let keyboard_state = app_state.clone();
    thread::spawn(move || match KeyboardHook::new(keyboard_state) {
        Ok(hook) => hook.run_message_loop(),
        Err(e) => Err(e),
    });

    // Start mouse hook in a separate thread
    let mouse_state = app_state.clone();
    thread::spawn(move || match MouseHook::new(mouse_state) {
        Ok(hook) => hook.run_message_loop(),
        Err(e) => Err(e),
    });

    // Start input manager
    let _input_manager = match InputManager::new(
        app_state.clone(),
        config.hid_baselines.clone(),
        config.device_api_preferences.clone(),
    ) {
        Ok(manager) => manager,
        Err(e) => {
            let error_msg = format!("Failed to initialize input manager: {}", e);
            return show_error(&error_msg);
        }
    };

    // Give hooks and input managers time to initialize
    thread::sleep(std::time::Duration::from_millis(200));

    // Start tray icon if enabled
    if app_state.show_tray_icon() {
        let tray_state = app_state.clone();
        thread::spawn(move || {
            let mut tray =
                TrayIcon::new(tray_state.should_exit.clone()).expect("Failed to create tray icon");
            let language = tray_state.language();
            let translations = crate::i18n::CachedTranslations::new(language);
            let msg = translations.tray_notification_launched().to_string();
            let _ = tray.show_info(&msg);
            let _ = tray.run_message_loop();
        });
    }

    SorahkGui::run(app_state.clone(), config)
}
