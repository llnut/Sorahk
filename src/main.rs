// Hide console window in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gui;
mod i18n;
mod keyboard;
mod mouse;
mod rawinput;
mod signal;
mod state;
mod tray;

use std::sync::Arc;
use std::thread;

use anyhow::Result;
use config::AppConfig;
use gui::{SorahkGui, show_error};
use keyboard::KeyboardHook;
use mouse::MouseHook;
use rawinput::RawInputHandler;
use state::AppState;
use tray::TrayIcon;

fn main() -> Result<()> {
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

    // Start Raw Input handler for HID devices (gamepads, joysticks, etc.)
    let rawinput_state = app_state.clone();
    let _rawinput_thread = RawInputHandler::start_thread(rawinput_state);

    // Give hooks time to initialize
    thread::sleep(std::time::Duration::from_millis(200));

    // Start tray icon if enabled
    if app_state.show_tray_icon() {
        let tray_state = app_state.clone();
        thread::spawn(move || {
            let mut tray =
                TrayIcon::new(tray_state.should_exit.clone()).expect("Failed to create tray icon");
            let _ = tray.show_info("Sorahk launched", "Sorahk is running in the background");
            let _ = tray.run_message_loop();
        });
    }

    SorahkGui::run(app_state.clone(), config)
}
