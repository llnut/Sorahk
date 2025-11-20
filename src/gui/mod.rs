//! GUI module for application interface components.
//!
//! This module provides the graphical user interface using the `egui` framework,
//! including the main window, dialogs, and utility functions.

mod about_dialog;
mod error_dialog;
mod fonts;
mod main_window;
mod settings_dialog;
mod types;
mod utils;

use crate::config::AppConfig;
use crate::gui::types::KeyCaptureMode;
use crate::i18n::CachedTranslations;
use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub use error_dialog::show_error;

/// Main GUI application structure.
///
/// Manages the application window state, dialogs, and user interactions.
pub struct SorahkGui {
    /// Shared application state
    app_state: Arc<AppState>,
    /// Application configuration
    config: AppConfig,
    /// Cached translations for high-performance rendering
    translations: CachedTranslations,
    /// Close confirmation dialog visibility
    show_close_dialog: bool,
    /// Settings dialog visibility
    show_settings_dialog: bool,
    /// About dialog visibility
    show_about_dialog: bool,
    /// Whether to minimize to tray on close
    minimize_on_close: bool,
    /// Current theme mode
    dark_mode: bool,
    /// Temporary config during settings edit
    temp_config: Option<AppConfig>,
    /// New mapping trigger key input
    new_mapping_trigger: String,
    /// New mapping target key input
    new_mapping_target: String,
    /// New mapping interval input
    new_mapping_interval: String,
    /// New mapping duration input
    new_mapping_duration: String,
    /// New process name input
    new_process_name: String,
    /// Current key capture state
    key_capture_mode: KeyCaptureMode,
    /// Flag to prevent re-entering capture mode immediately after capturing
    just_captured_input: bool,
    /// Close dialog highlight expiration time
    dialog_highlight_until: Option<std::time::Instant>,
    /// Pause state before entering settings
    was_paused_before_settings: Option<bool>,
    /// Error message for duplicate mapping
    duplicate_mapping_error: Option<String>,
    /// Error message for duplicate process
    duplicate_process_error: Option<String>,
    /// Cached dark theme visuals
    cached_dark_visuals: egui::Visuals,
    /// Cached light theme visuals
    cached_light_visuals: egui::Visuals,
}

impl SorahkGui {
    /// Creates a new GUI instance with the given state and configuration.
    pub fn new(app_state: Arc<AppState>, config: AppConfig) -> Self {
        let dark_mode = config.dark_mode;
        let translations = CachedTranslations::new(config.language);
        let cached_dark_visuals = Self::create_dark_visuals();
        let cached_light_visuals = Self::create_light_visuals();

        Self {
            app_state,
            config,
            translations,
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
            just_captured_input: false,
            was_paused_before_settings: None,
            duplicate_mapping_error: None,
            duplicate_process_error: None,
            cached_dark_visuals,
            cached_light_visuals,
        }
    }

    /// Updates the cached translations for the given language.
    fn update_translations(&mut self, language: crate::i18n::Language) {
        self.translations = CachedTranslations::new(language);
    }

    /// Creates dark theme visuals configuration.
    fn create_dark_visuals() -> egui::Visuals {
        let mut visuals = egui::Visuals::dark();

        // Apply rounded corners for anime-style appearance
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

        // Remove all borders for clean flat design
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        visuals.selection.stroke.width = 0.0;

        // Dark mode: deep purple-blue gradient
        visuals.window_fill = egui::Color32::from_rgb(25, 27, 35);
        visuals.panel_fill = egui::Color32::from_rgb(30, 32, 40);
        visuals.faint_bg_color = egui::Color32::from_rgb(35, 37, 45);
        visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(38, 40, 50);
        visuals.extreme_bg_color = egui::Color32::from_rgb(42, 44, 55);

        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 18,
            spread: 0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 25),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 3],
            blur: 12,
            spread: 0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 20),
        };

        visuals
    }

    /// Creates light theme visuals configuration.
    fn create_light_visuals() -> egui::Visuals {
        let mut visuals = egui::Visuals::light();

        // Apply rounded corners for anime-style appearance
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

        // Remove all borders for clean flat design
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        visuals.selection.stroke.width = 0.0;

        // Light mode: soft lavender gradient with enhanced contrast
        visuals.window_fill = egui::Color32::from_rgb(240, 235, 245);
        visuals.panel_fill = egui::Color32::from_rgb(238, 233, 243);
        visuals.faint_bg_color = egui::Color32::from_rgb(245, 240, 250);
        visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(250, 245, 255);
        visuals.extreme_bg_color = egui::Color32::from_rgb(235, 230, 245);

        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 18,
            spread: 0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 25),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 3],
            blur: 12,
            spread: 0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 20),
        };

        visuals
    }

    /// Launches the GUI application.
    ///
    /// # Errors
    ///
    /// Returns an error if the GUI framework fails to initialize or run.
    pub fn run(app_state: Arc<AppState>, config: AppConfig) -> anyhow::Result<()> {
        let icon = crate::gui::utils::create_icon();

        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([600.0, 530.0])
            .with_min_inner_size([600.0, 530.0])
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

        let language = config.language;

        eframe::run_native(
            "Sorahk",
            options,
            Box::new(move |cc| {
                fonts::load_fonts(&cc.egui_ctx, language);
                Ok(Box::new(SorahkGui::new(app_state, config)))
            }),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
    }
}
