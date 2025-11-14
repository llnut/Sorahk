// GUI module

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

// Public exports
pub use error_dialog::show_error;

/// Main GUI application structure
pub struct SorahkGui {
    pub(super) app_state: Arc<AppState>,
    pub(super) config: AppConfig,
    pub(super) show_close_dialog: bool,
    pub(super) show_settings_dialog: bool,
    pub(super) show_about_dialog: bool,
    pub(super) minimize_on_close: bool,
    pub(super) dark_mode: bool,
    // Temporary settings for editing
    pub(super) temp_config: Option<AppConfig>,
    // UI state for editing
    pub(super) new_mapping_trigger: String,
    pub(super) new_mapping_target: String,
    pub(super) new_mapping_interval: String,
    pub(super) new_mapping_duration: String,
    pub(super) new_process_name: String,
    // Key capture state
    pub(super) key_capture_mode: KeyCaptureMode,
    // Close dialog highlight effect
    pub(super) dialog_highlight_until: Option<std::time::Instant>,
}

impl SorahkGui {
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
        }
    }

    pub fn run(app_state: Arc<AppState>, config: AppConfig) -> anyhow::Result<()> {
        let icon = crate::gui::utils::create_icon();

        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([580.0, 530.0])
            .with_min_inner_size([500.0, 480.0])
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
