// Hide console window in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//mod about;
mod config;
mod gui;
mod keyboard;
mod signal;
mod state;
mod tray;

use std::sync::Arc;
use std::thread;

use anyhow::Result;
use config::AppConfig;
use gui::SorahkGui;
use keyboard::KeyboardHook;
use state::AppState;
use tray::TrayIcon;

fn main() -> Result<()> {
    signal::set_control_ctrl_handler()?;

    // Load config or create default if not exists
    let config = match AppConfig::load_or_create("Config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            // If config loading fails, show error via GUI
            let error_msg = format!("Failed to load configuration: {}", e);
            eprintln!("{}", error_msg);
            // Start GUI with error message
            return SorahkGui::show_error(&error_msg);
        }
    };

    let app_state = Arc::new(match AppState::new(config.clone()) {
        Ok(state) => state,
        Err(e) => {
            let error_msg = format!("Failed to initialize application state: {}", e);
            eprintln!("{}", error_msg);
            return SorahkGui::show_error(&error_msg);
        }
    });

    // Start keyboard hook in a separate thread BEFORE GUI
    // Create hook INSIDE the thread to ensure proper message loop
    let keyboard_state = app_state.clone();
    thread::spawn(move || match KeyboardHook::new(keyboard_state) {
        Ok(hook) => hook.run_message_loop(),
        Err(e) => {
            eprintln!("Failed to create keyboard hook: {}", e);
            Err(e)
        }
    });

    // Give keyboard hook time to initialize
    thread::sleep(std::time::Duration::from_millis(200));

    // Start tray icon if enabled
    if app_state.show_tray_icon() {
        let tray_state = app_state.clone();
        thread::spawn(move || {
            let mut tray =
                TrayIcon::new(tray_state.should_exit.clone()).expect("Failed to create tray icon");
            if let Err(e) = tray.show_info("Sorahk launched", "Sorahk is running in the background")
            {
                eprintln!("Failed to show notification: {}", e);
            }
            let _ = tray.run_message_loop();
        });
    }

    // Run GUI in main thread (GUI must run on main thread for proper window handling)
    

    // Exit immediately - the OS will clean up all resources (threads, windows, etc.)
    // This provides the smoothest user experience with no black window
    SorahkGui::run(app_state.clone(), config)
}
