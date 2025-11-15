//! GUI module for application interface components.
//!
//! This module provides the graphical user interface using the `egui` framework,
//! including the main window, dialogs, and utility functions.

mod about_dialog;
mod error_dialog;
mod main_window;
mod settings_dialog;
mod types;
mod utils;

use crate::config::AppConfig;
use crate::gui::types::KeyCaptureMode;
use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub use error_dialog::show_error;

/// Main GUI application structure.
///
/// Manages the application window state, dialogs, and user interactions.
pub struct SorahkGui {
    /// Shared application state
    pub(super) app_state: Arc<AppState>,
    /// Application configuration
    pub(super) config: AppConfig,
    /// Close confirmation dialog visibility
    pub(super) show_close_dialog: bool,
    /// Settings dialog visibility
    pub(super) show_settings_dialog: bool,
    /// About dialog visibility
    pub(super) show_about_dialog: bool,
    /// Whether to minimize to tray on close
    pub(super) minimize_on_close: bool,
    /// Current theme mode
    pub(super) dark_mode: bool,
    /// Temporary config during settings edit
    pub(super) temp_config: Option<AppConfig>,
    /// New mapping trigger key input
    pub(super) new_mapping_trigger: String,
    /// New mapping target key input
    pub(super) new_mapping_target: String,
    /// New mapping interval input
    pub(super) new_mapping_interval: String,
    /// New mapping duration input
    pub(super) new_mapping_duration: String,
    /// New process name input
    pub(super) new_process_name: String,
    /// Current key capture state
    pub(super) key_capture_mode: KeyCaptureMode,
    /// Close dialog highlight expiration time
    pub(super) dialog_highlight_until: Option<std::time::Instant>,
    /// Pause state before entering settings
    pub(super) was_paused_before_settings: Option<bool>,
}

impl SorahkGui {
    /// Creates a new GUI instance with the given state and configuration.
    pub fn new(app_state: Arc<AppState>, config: AppConfig) -> Self {
        let dark_mode = config.dark_mode;

        Self {
            app_state,
            config,
            show_close_dialog: false,
            show_settings_dialog: false,
            show_about_dialog: false,
            minimize_on_close: true,
            dialog_highlight_until: None,
            dark_mode,
            temp_config: None,
            new_mapping_trigger: String::new(),
            new_mapping_target: String::new(),
            new_mapping_interval: String::new(),
            new_mapping_duration: String::new(),
            new_process_name: String::new(),
            key_capture_mode: KeyCaptureMode::None,
            was_paused_before_settings: None,
        }
    }

    /// Launches the GUI application.
    ///
    /// # Errors
    ///
    /// Returns an error if the GUI framework fails to initialize or run.
    pub fn run(app_state: Arc<AppState>, config: AppConfig) -> anyhow::Result<()> {
        let icon = crate::gui::utils::create_icon();

        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([580.0, 550.0])
            .with_min_inner_size([580.0, 550.0])
            .with_resizable(true)
            .with_title("Sorahk - Auto Key Press Tool")
            .with_icon(icon)
            .with_taskbar(false);

        if config.always_on_top {
            viewport = viewport.with_always_on_top();
        }

        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        eframe::run_native(
            "Sorahk",
            options,
            Box::new(|_cc| Ok(Box::new(SorahkGui::new(app_state, config)))),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
    }
}
