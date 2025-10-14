//mod about;
mod config;
mod keyboard;
mod signal;
mod state;
mod tray;

use std::sync::Arc;
use std::thread;

use anyhow::Result;
use config::AppConfig;
use keyboard::KeyboardHook;
use state::AppState;
use tray::TrayIcon;

fn main() -> Result<()> {
    signal::set_control_ctrl_handler()?;

    let config = AppConfig::load_from_file("Config.toml")?;
    let app_state = Arc::new(AppState::new(config)?);
    let keyboard_hook = KeyboardHook::new(app_state.clone())?;

    let handler = if app_state.show_tray_icon {
        Some(thread::spawn(move || {
            let mut tray =
                TrayIcon::new(app_state.should_exit.clone()).expect("Failed to create tray icon");
            if let Err(e) = tray.show_info("Sorahk launched", "Sorahk is running in the background")
            {
                eprintln!("Failed to show notification: {}", e);
            }
            let _ = tray.run_message_loop();
        }))
    } else {
        None
    };

    keyboard_hook.run_message_loop()?;
    if let Some(handler) = handler {
        let _ = handler.join();
    }
    Ok(())
}
