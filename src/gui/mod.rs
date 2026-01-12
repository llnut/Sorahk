//! GUI module for application interface components.
//!
//! This module provides the graphical user interface using the `egui` framework,
//! including the main window, dialogs, and utility functions.

mod about_dialog;
mod device_info;
pub mod device_manager_dialog;
mod error_dialog;
mod fonts;
mod hid_activation_dialog;
mod main_window;
mod mouse_direction_dialog;
mod mouse_scroll_dialog;
mod settings_dialog;
mod types;
mod utils;

use crate::config::AppConfig;
use crate::gui::types::KeyCaptureMode;
use crate::i18n::CachedTranslations;
use crate::state::AppState;
use eframe::egui;
use smallvec::SmallVec;
use std::sync::Arc;

pub use error_dialog::show_error;

/// Pre-parsed switch key configuration.
#[derive(Clone, Debug)]
enum ParsedSwitchKey {
    /// No switch key configured
    None,
    /// Single key
    Single(u32),
    /// Key combination with modifiers
    Combo {
        /// Modifier flags: bit0=ctrl, bit1=shift, bit2=alt
        modifiers: u8,
        /// Main keys (VK codes)
        keys: SmallVec<[u32; 4]>,
    },
}

/// Main GUI application structure.
///
/// Manages the application window state, dialogs, and user interactions.
pub struct SorahkGui {
    /// Shared application state
    app_state: Arc<AppState>,
    /// Application configuration
    config: AppConfig,
    /// Cached translations for rendering
    translations: CachedTranslations,
    /// Close confirmation dialog visibility
    show_close_dialog: bool,
    /// Settings dialog visibility
    show_settings_dialog: bool,
    /// About dialog visibility
    show_about_dialog: bool,
    /// Device manager dialog visibility
    show_device_manager: bool,
    /// Device manager dialog
    device_manager_dialog: Option<device_manager_dialog::DeviceManagerDialog>,
    /// HID device activation dialog
    hid_activation_dialog: Option<hid_activation_dialog::HidActivationDialog>,
    /// HID activation dialog creation time (for 10ms debounce)
    hid_activation_creation_time: Option<std::time::Instant>,
    /// Mouse direction selection dialog
    mouse_direction_dialog: Option<mouse_direction_dialog::MouseDirectionDialog>,
    /// Mouse scroll selection dialog
    mouse_scroll_dialog: Option<mouse_scroll_dialog::MouseScrollDialog>,
    /// Index of mapping being edited for mouse direction (None for new mapping)
    mouse_direction_mapping_idx: Option<usize>,
    /// Index of mapping being edited for mouse scroll (None for new mapping)
    mouse_scroll_mapping_idx: Option<usize>,
    /// Whether to minimize to tray on close
    minimize_on_close: bool,
    /// Current theme mode
    dark_mode: bool,
    /// Temporary config during settings edit
    temp_config: Option<AppConfig>,
    /// New mapping trigger key input
    new_mapping_trigger: String,
    /// New mapping target key input (single key for capture)
    new_mapping_target: String,
    /// New mapping target keys (multiple keys)
    new_mapping_target_keys: Vec<String>,
    /// New mapping interval input
    new_mapping_interval: String,
    /// New mapping duration input
    new_mapping_duration: String,
    /// New mapping turbo enabled state
    new_mapping_turbo: bool,
    /// New mapping move speed input
    new_mapping_move_speed: String,
    /// New process name input
    new_process_name: String,
    /// Current key capture state
    key_capture_mode: KeyCaptureMode,
    /// Captured key sequence for trigger
    sequence_capture_list: Vec<String>,
    /// Time window (ms) between inputs for sequence
    new_mapping_sequence_window: String,
    /// Whether new mapping uses sequence trigger mode
    new_mapping_is_sequence_mode: bool,
    /// Target mode: 0=Single, 1=Multi (simultaneous), 2=Sequence (sequential)
    new_mapping_target_mode: u8,
    /// Target sequence capture list
    target_sequence_capture_list: Vec<String>,
    /// Editing existing mapping target sequence capture list
    editing_target_seq_list: Vec<String>,
    /// Index of mapping being edited for target sequence
    editing_target_seq_idx: Option<usize>,
    /// Mouse position when sequence capture started or last direction changed
    sequence_last_mouse_pos: Option<egui::Pos2>,
    /// Last detected mouse direction in sequence (for deduplication)
    sequence_last_mouse_direction: Option<String>,
    /// Accumulated mouse movement delta for detecting direction change
    sequence_mouse_delta: egui::Vec2,
    /// Flag to prevent re-entering capture mode immediately after capturing
    just_captured_input: bool,
    /// Keys currently pressed during capture (VK codes)
    capture_pressed_keys: std::collections::HashSet<u32>,
    /// Keys that were pressed when capture mode started (noise baseline)
    capture_initial_pressed: std::collections::HashSet<u32>,
    /// Pre-parsed switch key configuration
    parsed_switch_key: ParsedSwitchKey,
    /// VK key states for switch key detection (16 bits using modulo mapping)
    last_vk_state: u16,
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
        let parsed_switch_key = Self::parse_switch_key(&config.switch_key);

        Self {
            app_state,
            config,
            translations,
            show_close_dialog: false,
            show_settings_dialog: false,
            show_about_dialog: false,
            show_device_manager: false,
            device_manager_dialog: None,
            hid_activation_dialog: None,
            hid_activation_creation_time: None,
            mouse_direction_dialog: None,
            mouse_scroll_dialog: None,
            mouse_direction_mapping_idx: None,
            mouse_scroll_mapping_idx: None,
            minimize_on_close: true,
            dialog_highlight_until: None,
            dark_mode,
            temp_config: None,
            new_mapping_trigger: String::new(),
            new_mapping_target: String::new(),
            new_mapping_target_keys: Vec::new(),
            new_mapping_interval: String::new(),
            new_mapping_duration: String::new(),
            new_mapping_turbo: true,
            new_mapping_move_speed: "5".to_string(),
            new_process_name: String::new(),
            key_capture_mode: KeyCaptureMode::None,
            just_captured_input: false,
            sequence_capture_list: Vec::new(),
            new_mapping_sequence_window: "300".to_string(),
            new_mapping_is_sequence_mode: false,
            new_mapping_target_mode: 0,
            target_sequence_capture_list: Vec::new(),
            editing_target_seq_list: Vec::new(),
            editing_target_seq_idx: None,
            sequence_last_mouse_pos: None,
            sequence_last_mouse_direction: None,
            sequence_mouse_delta: egui::Vec2::ZERO,
            capture_pressed_keys: std::collections::HashSet::new(),
            capture_initial_pressed: std::collections::HashSet::new(),
            parsed_switch_key,
            last_vk_state: 0,
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

    /// Parse switch key configuration during initialization.
    fn parse_switch_key(switch_key: &str) -> ParsedSwitchKey {
        use crate::gui::utils::string_to_vk;

        if switch_key.is_empty() {
            return ParsedSwitchKey::None;
        }

        if !switch_key.contains('+') {
            if let Some(vk) = string_to_vk(switch_key) {
                return ParsedSwitchKey::Single(vk);
            }
            return ParsedSwitchKey::None;
        }

        // Combo key - parse modifiers and keys
        let parts: Vec<&str> = switch_key.split('+').collect();
        let mut modifiers = 0u8;
        let mut keys = SmallVec::new();

        for part in parts {
            let p = part.trim().to_uppercase();
            match p.as_str() {
                "LCTRL" | "RCTRL" | "CTRL" => modifiers |= 0b001,
                "LSHIFT" | "RSHIFT" | "SHIFT" => modifiers |= 0b010,
                "LALT" | "RALT" | "ALT" => modifiers |= 0b100,
                _ => {
                    if let Some(vk) = string_to_vk(part.trim()) {
                        keys.push(vk);
                    }
                }
            }
        }

        ParsedSwitchKey::Combo { modifiers, keys }
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
            .with_inner_size([820.0, 600.0])
            .with_min_inner_size([820.0, 600.0])
            .with_resizable(true)
            .with_title("Sorahk - Auto Key Press Tool")
            .with_icon(icon)
            .with_taskbar(true);

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
