//! Internationalization support for multiple languages.
//!
//! Provides high-performance cached translation strings for UI elements.
//! All strings are pre-formatted to avoid repeated allocation in the render loop.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Supported languages in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Language {
    /// English
    #[default]
    English,
    /// Simplified Chinese
    SimplifiedChinese,
    /// Traditional Chinese
    TraditionalChinese,
    /// Japanese
    Japanese,
}

impl Language {
    /// Returns all available languages.
    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::Japanese,
        ]
    }

    /// Returns the display name of the language.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::TraditionalChinese => "繁體中文",
            Language::Japanese => "日本語",
        }
    }
}

/// Cached translations for high-performance rendering.
/// All strings are pre-formatted and stored in an Arc for efficient cloning.
#[derive(Clone)]
pub struct CachedTranslations {
    inner: Arc<TranslationCache>,
}

struct TranslationCache {
    app_title: String,
    settings_button: String,
    about_button: String,
    dark_theme: String,
    light_theme: String,
    status_title: String,
    paused_status: String,
    running_status: String,
    pause_button: String,
    start_button: String,
    exit_button: String,
    hotkey_settings_title: String,
    toggle_key_label: String,
    click_to_set: String,
    config_settings_title: String,
    input_timeout_display: String,
    default_interval_display: String,
    default_duration_display: String,
    show_tray_icon_display: String,
    show_notifications_display: String,
    always_on_top_display: String,
    yes: String,
    no: String,
    key_mappings_title: String,
    settings_dialog_title: String,
    language_label: String,
    dark_mode_label: String,
    always_on_top_label: String,
    show_tray_icon_label: String,
    show_notifications_label: String,
    toggle_key_section: String,
    key_label: String,
    press_any_key: String,
    global_config_title: String,
    input_timeout_label: String,
    default_interval_label: String,
    default_duration_label: String,
    worker_count_label: String,
    trigger_short: String,
    target_short: String,
    interval_short: String,
    duration_short: String,
    trigger_header: String,
    target_header: String,
    interval_header: String,
    duration_header: String,
    turbo_header: String,
    add_new_mapping_title: String,
    click_text: String,
    add_button_text: String,
    process_whitelist_hint: String,
    process_example: String,
    browse_button: String,
    save_changes_button: String,
    cancel_settings_button: String,
    changes_take_effect_hint: String,
    close_window_title: String,
    close_subtitle: String,
    minimize_to_tray_button: String,
    exit_program_button: String,
    cancel_close_button: String,
    error_title: String,
    error_close_button: String,
    duplicate_trigger_error: String,
    duplicate_process_error: String,
    about_version: String,
    about_description_line1: String,
    about_description_line2: String,
    about_author: String,
    about_github: String,
    about_license: String,
    about_built_with: String,
    about_mit_license: String,
    about_rust_egui: String,
    about_inspired: String,
    turbo_on_hover: String,
    turbo_off_hover: String,
    hid_activation_title: String,
    hid_activation_press_prompt: String,
    hid_activation_release_prompt: String,
    hid_activation_warning_title: String,
    hid_activation_warning_1: String,
    hid_activation_warning_2: String,
    hid_activation_warning_3: String,
    hid_activation_success_title: String,
    hid_activation_success_message: String,
    hid_activation_success_hint: String,
    hid_activation_auto_close: String,
    hid_activation_failed_title: String,
    hid_activation_error: String,
    hid_activation_retry: String,
    hid_activation_cancel: String,
    mouse_move_direction_label: String,
    mouse_move_up: String,
    mouse_move_down: String,
    mouse_move_left: String,
    mouse_move_right: String,
    mouse_move_up_left: String,
    mouse_move_up_right: String,
    mouse_move_down_left: String,
    mouse_move_down_right: String,
    set_mouse_direction_hover: String,
    mouse_scroll_direction_label: String,
    mouse_scroll_up: String,
    mouse_scroll_down: String,
    set_mouse_scroll_direction_hover: String,
    speed_label: String,
    capture_mode_label: String,
    capture_mode_most_sustained: String,
    capture_mode_adaptive_intelligent: String,
    capture_mode_max_changed_bits: String,
    capture_mode_max_set_bits: String,
    capture_mode_last_stable: String,
    capture_mode_hat_switch_optimized: String,
    capture_mode_analog_optimized: String,
    add_target_key_hover: String,
    clear_all_target_keys_hover: String,
    remove_target_key_prefix: String,
    diagonal_hint_prefix: String,
    diagonal_hint_suffix: String,
}

impl CachedTranslations {
    /// Creates a new cached translations instance for the specified language.
    /// All strings are pre-formatted to avoid allocation in the render loop.
    pub fn new(lang: Language) -> Self {
        let inner = Arc::new(TranslationCache::new(lang));
        Self { inner }
    }

    // Main Window - Title Bar
    pub fn app_title(&self) -> &str {
        &self.inner.app_title
    }
    pub fn settings_button(&self) -> &str {
        &self.inner.settings_button
    }
    pub fn about_button(&self) -> &str {
        &self.inner.about_button
    }
    pub fn dark_theme(&self) -> &str {
        &self.inner.dark_theme
    }
    pub fn light_theme(&self) -> &str {
        &self.inner.light_theme
    }
    pub fn language(&self) -> &str {
        &self.inner.language_label
    }
    pub fn dark_mode(&self) -> &str {
        &self.inner.dark_mode_label
    }
    pub fn always_on_top(&self) -> &str {
        &self.inner.always_on_top_label
    }
    pub fn show_tray_icon(&self) -> &str {
        &self.inner.show_tray_icon_label
    }
    pub fn show_notifications(&self) -> &str {
        &self.inner.show_notifications_label
    }
    pub fn toggle_key(&self) -> &str {
        &self.inner.toggle_key_section
    }
    pub fn key_label(&self) -> &str {
        &self.inner.key_label
    }
    pub fn save(&self) -> &str {
        &self.inner.save_changes_button
    }
    pub fn cancel(&self) -> &str {
        &self.inner.cancel_settings_button
    }

    // Main Window - Status Card
    pub fn status_title(&self) -> &str {
        &self.inner.status_title
    }
    pub fn paused_status(&self) -> &str {
        &self.inner.paused_status
    }

    // Main Window - Hotkey Settings Card
    pub fn hotkey_settings_title(&self) -> &str {
        &self.inner.hotkey_settings_title
    }
    pub fn toggle_key_label(&self) -> &str {
        &self.inner.toggle_key_label
    }
    pub fn click_to_set(&self) -> &str {
        &self.inner.click_to_set
    }

    // Main Window - Config Settings Card
    pub fn config_settings_title(&self) -> &str {
        &self.inner.config_settings_title
    }

    // Main Window - Key Mappings Card
    pub fn key_mappings_title(&self) -> &str {
        &self.inner.key_mappings_title
    }

    // Settings Dialog - Title
    pub fn settings_dialog_title(&self) -> &str {
        &self.inner.settings_dialog_title
    }

    // Settings Dialog - Toggle Key Section
    pub fn press_any_key(&self) -> &str {
        &self.inner.press_any_key
    }

    // Settings Dialog - Global Configuration Section
    pub fn global_config_title(&self) -> &str {
        &self.inner.global_config_title
    }
    pub fn input_timeout_label(&self) -> &str {
        &self.inner.input_timeout_label
    }
    pub fn default_interval_label(&self) -> &str {
        &self.inner.default_interval_label
    }
    pub fn default_duration_label(&self) -> &str {
        &self.inner.default_duration_label
    }

    // Close Dialog
    pub fn close_window_title(&self) -> &str {
        &self.inner.close_window_title
    }
    pub fn close_subtitle(&self) -> &str {
        &self.inner.close_subtitle
    }
    pub fn minimize_to_tray_button(&self) -> &str {
        &self.inner.minimize_to_tray_button
    }
    pub fn exit_program_button(&self) -> &str {
        &self.inner.exit_program_button
    }
    pub fn cancel_close_button(&self) -> &str {
        &self.inner.cancel_close_button
    }

    // Error Dialog
    pub fn error_title(&self) -> &str {
        &self.inner.error_title
    }
    pub fn error_close_button(&self) -> &str {
        &self.inner.error_close_button
    }
    pub fn duplicate_trigger_error(&self) -> &str {
        &self.inner.duplicate_trigger_error
    }

    pub fn duplicate_process_error(&self) -> &str {
        &self.inner.duplicate_process_error
    }

    // About Dialog
    pub fn about_version(&self) -> &str {
        &self.inner.about_version
    }
    pub fn about_description_line1(&self) -> &str {
        &self.inner.about_description_line1
    }
    pub fn about_description_line2(&self) -> &str {
        &self.inner.about_description_line2
    }
    pub fn about_author(&self) -> &str {
        &self.inner.about_author
    }
    pub fn about_github(&self) -> &str {
        &self.inner.about_github
    }
    pub fn about_license(&self) -> &str {
        &self.inner.about_license
    }
    pub fn about_built_with(&self) -> &str {
        &self.inner.about_built_with
    }
    pub fn about_mit_license(&self) -> &str {
        &self.inner.about_mit_license
    }
    pub fn about_rust_egui(&self) -> &str {
        &self.inner.about_rust_egui
    }
    pub fn about_inspired(&self) -> &str {
        &self.inner.about_inspired
    }

    // Turbo toggle tooltips
    pub fn turbo_on_hover(&self) -> &str {
        &self.inner.turbo_on_hover
    }
    pub fn turbo_off_hover(&self) -> &str {
        &self.inner.turbo_off_hover
    }

    // HID Activation Dialog
    pub fn hid_activation_title(&self) -> &str {
        &self.inner.hid_activation_title
    }
    pub fn hid_activation_press_prompt(&self) -> &str {
        &self.inner.hid_activation_press_prompt
    }
    pub fn hid_activation_release_prompt(&self) -> &str {
        &self.inner.hid_activation_release_prompt
    }
    pub fn hid_activation_warning_title(&self) -> &str {
        &self.inner.hid_activation_warning_title
    }
    pub fn hid_activation_warning_1(&self) -> &str {
        &self.inner.hid_activation_warning_1
    }
    pub fn hid_activation_warning_2(&self) -> &str {
        &self.inner.hid_activation_warning_2
    }
    pub fn hid_activation_warning_3(&self) -> &str {
        &self.inner.hid_activation_warning_3
    }
    pub fn hid_activation_success_title(&self) -> &str {
        &self.inner.hid_activation_success_title
    }
    pub fn hid_activation_success_message(&self) -> &str {
        &self.inner.hid_activation_success_message
    }
    pub fn hid_activation_success_hint(&self) -> &str {
        &self.inner.hid_activation_success_hint
    }
    pub fn hid_activation_auto_close(&self) -> &str {
        &self.inner.hid_activation_auto_close
    }
    pub fn hid_activation_failed_title(&self) -> &str {
        &self.inner.hid_activation_failed_title
    }
    pub fn hid_activation_error(&self) -> &str {
        &self.inner.hid_activation_error
    }
    pub fn hid_activation_retry(&self) -> &str {
        &self.inner.hid_activation_retry
    }
    pub fn hid_activation_cancel(&self) -> &str {
        &self.inner.hid_activation_cancel
    }

    // Mouse Movement
    pub fn mouse_move_direction_label(&self) -> &str {
        &self.inner.mouse_move_direction_label
    }
    pub fn mouse_move_up(&self) -> &str {
        &self.inner.mouse_move_up
    }
    pub fn mouse_move_down(&self) -> &str {
        &self.inner.mouse_move_down
    }
    pub fn mouse_move_left(&self) -> &str {
        &self.inner.mouse_move_left
    }
    pub fn mouse_move_right(&self) -> &str {
        &self.inner.mouse_move_right
    }
    pub fn mouse_move_up_left(&self) -> &str {
        &self.inner.mouse_move_up_left
    }
    pub fn mouse_move_up_right(&self) -> &str {
        &self.inner.mouse_move_up_right
    }
    pub fn mouse_move_down_left(&self) -> &str {
        &self.inner.mouse_move_down_left
    }
    pub fn mouse_move_down_right(&self) -> &str {
        &self.inner.mouse_move_down_right
    }
    pub fn set_mouse_direction_hover(&self) -> &str {
        &self.inner.set_mouse_direction_hover
    }

    // Mouse Scroll
    pub fn mouse_scroll_direction_label(&self) -> &str {
        &self.inner.mouse_scroll_direction_label
    }
    pub fn mouse_scroll_up(&self) -> &str {
        &self.inner.mouse_scroll_up
    }
    pub fn mouse_scroll_down(&self) -> &str {
        &self.inner.mouse_scroll_down
    }
    pub fn set_mouse_scroll_direction_hover(&self) -> &str {
        &self.inner.set_mouse_scroll_direction_hover
    }
    pub fn speed_label(&self) -> &str {
        &self.inner.speed_label
    }

    // Capture Mode
    pub fn capture_mode_label(&self) -> &str {
        &self.inner.capture_mode_label
    }
    pub fn capture_mode_most_sustained(&self) -> &str {
        &self.inner.capture_mode_most_sustained
    }
    pub fn capture_mode_adaptive_intelligent(&self) -> &str {
        &self.inner.capture_mode_adaptive_intelligent
    }
    pub fn capture_mode_max_changed_bits(&self) -> &str {
        &self.inner.capture_mode_max_changed_bits
    }
    pub fn capture_mode_max_set_bits(&self) -> &str {
        &self.inner.capture_mode_max_set_bits
    }
    pub fn capture_mode_last_stable(&self) -> &str {
        &self.inner.capture_mode_last_stable
    }
    pub fn capture_mode_hat_switch_optimized(&self) -> &str {
        &self.inner.capture_mode_hat_switch_optimized
    }
    pub fn capture_mode_analog_optimized(&self) -> &str {
        &self.inner.capture_mode_analog_optimized
    }

    // Multi-target key support
    pub fn add_target_key_hover(&self) -> &str {
        &self.inner.add_target_key_hover
    }
    pub fn clear_all_target_keys_hover(&self) -> &str {
        &self.inner.clear_all_target_keys_hover
    }
    pub fn format_remove_target_key_hover(&self, key: &str) -> String {
        format!("{} {}", self.inner.remove_target_key_prefix, key)
    }
    pub fn format_diagonal_hint(&self, direction: &str) -> String {
        format!(
            "{}{}{}",
            self.inner.diagonal_hint_prefix, direction, self.inner.diagonal_hint_suffix
        )
    }

    // Additional main window status card
    pub fn running_status(&self) -> &str {
        &self.inner.running_status
    }
    pub fn pause_button(&self) -> &str {
        &self.inner.pause_button
    }
    pub fn start_button(&self) -> &str {
        &self.inner.start_button
    }
    pub fn exit_button(&self) -> &str {
        &self.inner.exit_button
    }

    // Main window config display
    pub fn input_timeout_display(&self) -> &str {
        &self.inner.input_timeout_display
    }
    pub fn default_interval_display(&self) -> &str {
        &self.inner.default_interval_display
    }
    pub fn default_duration_display(&self) -> &str {
        &self.inner.default_duration_display
    }
    pub fn show_tray_icon_display(&self) -> &str {
        &self.inner.show_tray_icon_display
    }
    pub fn show_notifications_display(&self) -> &str {
        &self.inner.show_notifications_display
    }
    pub fn always_on_top_display(&self) -> &str {
        &self.inner.always_on_top_display
    }
    pub fn yes(&self) -> &str {
        &self.inner.yes
    }
    pub fn no(&self) -> &str {
        &self.inner.no
    }

    // Additional settings dialog fields
    pub fn worker_count_label(&self) -> &str {
        &self.inner.worker_count_label
    }
    pub fn trigger_short(&self) -> &str {
        &self.inner.trigger_short
    }
    pub fn target_short(&self) -> &str {
        &self.inner.target_short
    }
    pub fn interval_short(&self) -> &str {
        &self.inner.interval_short
    }
    pub fn duration_short(&self) -> &str {
        &self.inner.duration_short
    }
    pub fn trigger_header(&self) -> &str {
        &self.inner.trigger_header
    }
    pub fn target_header(&self) -> &str {
        &self.inner.target_header
    }
    pub fn interval_header(&self) -> &str {
        &self.inner.interval_header
    }
    pub fn duration_header(&self) -> &str {
        &self.inner.duration_header
    }
    pub fn turbo_header(&self) -> &str {
        &self.inner.turbo_header
    }
    pub fn add_new_mapping_title(&self) -> &str {
        &self.inner.add_new_mapping_title
    }
    pub fn click_text(&self) -> &str {
        &self.inner.click_text
    }
    pub fn add_button_text(&self) -> &str {
        &self.inner.add_button_text
    }
    pub fn process_whitelist_hint(&self) -> &str {
        &self.inner.process_whitelist_hint
    }
    pub fn process_example(&self) -> &str {
        &self.inner.process_example
    }
    pub fn browse_button(&self) -> &str {
        &self.inner.browse_button
    }
    pub fn changes_take_effect_hint(&self) -> &str {
        &self.inner.changes_take_effect_hint
    }

    // Dynamic worker count formatting (for runtime values)
    pub fn format_worker_count(&self, count: usize) -> String {
        format!("{} {}", self.inner.worker_count_label, count)
    }
}

impl TranslationCache {
    fn new(lang: Language) -> Self {
        Self {
            // Main Window - Title Bar
            app_title: get_raw_translation(lang, RawKey::AppTitle).to_string(),
            settings_button: get_raw_translation(lang, RawKey::SettingsBtn).to_string(),
            about_button: get_raw_translation(lang, RawKey::AboutBtn).to_string(),
            dark_theme: get_raw_translation(lang, RawKey::Dark).to_string(),
            light_theme: get_raw_translation(lang, RawKey::Light).to_string(),

            // Main Window - Status Card
            status_title: get_raw_translation(lang, RawKey::StatusTitle).to_string(),
            paused_status: get_raw_translation(lang, RawKey::Paused).to_string(),
            running_status: get_raw_translation(lang, RawKey::Running).to_string(),
            pause_button: get_raw_translation(lang, RawKey::PauseBtn).to_string(),
            start_button: get_raw_translation(lang, RawKey::StartBtn).to_string(),
            exit_button: get_raw_translation(lang, RawKey::ExitBtn).to_string(),

            // Main Window - Hotkey Settings Card
            hotkey_settings_title: get_raw_translation(lang, RawKey::HotkeySettingsTitle)
                .to_string(),
            toggle_key_label: get_raw_translation(lang, RawKey::ToggleKeyLabel).to_string(),
            click_to_set: get_raw_translation(lang, RawKey::ClickToSet).to_string(),

            // Main Window - Config Settings Card
            config_settings_title: get_raw_translation(lang, RawKey::ConfigSettingsTitle)
                .to_string(),
            input_timeout_display: get_raw_translation(lang, RawKey::InputTimeoutDisplay)
                .to_string(),
            default_interval_display: get_raw_translation(lang, RawKey::DefaultIntervalDisplay)
                .to_string(),
            default_duration_display: get_raw_translation(lang, RawKey::DefaultDurationDisplay)
                .to_string(),
            show_tray_icon_display: get_raw_translation(lang, RawKey::ShowTrayIconDisplay)
                .to_string(),
            show_notifications_display: get_raw_translation(lang, RawKey::ShowNotificationsDisplay)
                .to_string(),
            always_on_top_display: get_raw_translation(lang, RawKey::AlwaysOnTopDisplay)
                .to_string(),
            yes: get_raw_translation(lang, RawKey::Yes).to_string(),
            no: get_raw_translation(lang, RawKey::No).to_string(),

            // Main Window - Key Mappings Card
            key_mappings_title: get_raw_translation(lang, RawKey::KeyMappingsTitle).to_string(),

            // Settings Dialog - Title
            settings_dialog_title: get_raw_translation(lang, RawKey::SettingsDialogTitle)
                .to_string(),

            // Settings Dialog - Language & Appearance Section
            language_label: get_raw_translation(lang, RawKey::Language).to_string(),
            dark_mode_label: get_raw_translation(lang, RawKey::DarkMode).to_string(),
            always_on_top_label: get_raw_translation(lang, RawKey::AlwaysOnTop).to_string(),
            show_tray_icon_label: get_raw_translation(lang, RawKey::ShowTrayIcon).to_string(),
            show_notifications_label: get_raw_translation(lang, RawKey::ShowNotifications)
                .to_string(),

            // Settings Dialog - Toggle Key Section
            toggle_key_section: get_raw_translation(lang, RawKey::ToggleKeySection).to_string(),
            key_label: get_raw_translation(lang, RawKey::KeyLabel).to_string(),
            press_any_key: get_raw_translation(lang, RawKey::PressAnyKey).to_string(),

            // Settings Dialog - Global Configuration Section
            global_config_title: get_raw_translation(lang, RawKey::GlobalConfigTitle).to_string(),
            input_timeout_label: get_raw_translation(lang, RawKey::InputTimeoutLabel).to_string(),
            default_interval_label: get_raw_translation(lang, RawKey::DefaultIntervalLabel)
                .to_string(),
            default_duration_label: get_raw_translation(lang, RawKey::DefaultDurationLabel)
                .to_string(),
            worker_count_label: get_raw_translation(lang, RawKey::WorkerCountLabel).to_string(),

            // Settings Dialog - Key Mappings Section
            trigger_short: get_raw_translation(lang, RawKey::TriggerShort).to_string(),
            target_short: get_raw_translation(lang, RawKey::TargetShort).to_string(),
            interval_short: get_raw_translation(lang, RawKey::IntShort).to_string(),
            duration_short: get_raw_translation(lang, RawKey::DurShort).to_string(),

            // Main Window - Key Mappings Table Headers
            trigger_header: get_raw_translation(lang, RawKey::Trigger).to_string(),
            target_header: get_raw_translation(lang, RawKey::Target).to_string(),
            interval_header: get_raw_translation(lang, RawKey::IntervalMs).to_string(),
            duration_header: get_raw_translation(lang, RawKey::DurationMs).to_string(),
            turbo_header: get_raw_translation(lang, RawKey::TurboHeader).to_string(),

            add_new_mapping_title: get_raw_translation(lang, RawKey::AddNewMappingTitle)
                .to_string(),
            click_text: get_raw_translation(lang, RawKey::Click).to_string(),
            add_button_text: get_raw_translation(lang, RawKey::AddBtn).to_string(),

            // Settings Dialog - Process Whitelist Section
            process_whitelist_hint: get_raw_translation(lang, RawKey::ProcessWhitelistHint)
                .to_string(),
            process_example: get_raw_translation(lang, RawKey::ProcessExample).to_string(),
            browse_button: get_raw_translation(lang, RawKey::BrowseBtn).to_string(),

            // Settings Dialog - Action Buttons
            save_changes_button: get_raw_translation(lang, RawKey::SaveChangesBtn).to_string(),
            cancel_settings_button: get_raw_translation(lang, RawKey::CancelSettingsBtn)
                .to_string(),
            changes_take_effect_hint: get_raw_translation(lang, RawKey::ChangesTakeEffect)
                .to_string(),

            // Close Dialog
            close_window_title: get_raw_translation(lang, RawKey::CloseWindowTitle).to_string(),
            close_subtitle: get_raw_translation(lang, RawKey::CloseSubtitle).to_string(),
            minimize_to_tray_button: get_raw_translation(lang, RawKey::MinimizeToTrayBtn)
                .to_string(),
            exit_program_button: get_raw_translation(lang, RawKey::ExitProgramBtn).to_string(),
            cancel_close_button: get_raw_translation(lang, RawKey::CancelCloseBtn).to_string(),

            // Error Dialog
            error_title: get_raw_translation(lang, RawKey::ErrorTitle).to_string(),
            error_close_button: get_raw_translation(lang, RawKey::Close).to_string(),
            duplicate_trigger_error: get_raw_translation(lang, RawKey::DuplicateTriggerError)
                .to_string(),
            duplicate_process_error: get_raw_translation(lang, RawKey::DuplicateProcessError)
                .to_string(),

            // About Dialog
            about_version: format!("✨ Version {}", env!("CARGO_PKG_VERSION")),
            about_description_line1: get_raw_translation(lang, RawKey::AboutDescriptionLine1)
                .to_string(),
            about_description_line2: get_raw_translation(lang, RawKey::AboutDescriptionLine2)
                .to_string(),
            about_author: get_raw_translation(lang, RawKey::Author).to_string(),
            about_github: get_raw_translation(lang, RawKey::GitHub).to_string(),
            about_license: get_raw_translation(lang, RawKey::License).to_string(),
            about_built_with: get_raw_translation(lang, RawKey::BuiltWith).to_string(),
            about_mit_license: "MIT License".to_string(),
            about_rust_egui: "Rust + egui".to_string(),
            about_inspired: get_raw_translation(lang, RawKey::AboutInspired).to_string(),

            // Turbo toggle tooltips
            turbo_on_hover: get_raw_translation(lang, RawKey::TurboOnHover).to_string(),
            turbo_off_hover: get_raw_translation(lang, RawKey::TurboOffHover).to_string(),

            // HID Activation Dialog
            hid_activation_title: get_raw_translation(lang, RawKey::HidActivationTitle).to_string(),
            hid_activation_press_prompt: get_raw_translation(
                lang,
                RawKey::HidActivationPressPrompt,
            )
            .to_string(),
            hid_activation_release_prompt: get_raw_translation(
                lang,
                RawKey::HidActivationReleasePrompt,
            )
            .to_string(),
            hid_activation_warning_title: get_raw_translation(
                lang,
                RawKey::HidActivationWarningTitle,
            )
            .to_string(),
            hid_activation_warning_1: get_raw_translation(lang, RawKey::HidActivationWarning1)
                .to_string(),
            hid_activation_warning_2: get_raw_translation(lang, RawKey::HidActivationWarning2)
                .to_string(),
            hid_activation_warning_3: get_raw_translation(lang, RawKey::HidActivationWarning3)
                .to_string(),
            hid_activation_success_title: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessTitle,
            )
            .to_string(),
            hid_activation_success_message: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessMessage,
            )
            .to_string(),
            hid_activation_success_hint: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessHint,
            )
            .to_string(),
            hid_activation_auto_close: get_raw_translation(lang, RawKey::HidActivationAutoClose)
                .to_string(),
            hid_activation_failed_title: get_raw_translation(
                lang,
                RawKey::HidActivationFailedTitle,
            )
            .to_string(),
            hid_activation_error: get_raw_translation(lang, RawKey::HidActivationError).to_string(),
            hid_activation_retry: get_raw_translation(lang, RawKey::HidActivationRetry).to_string(),
            hid_activation_cancel: get_raw_translation(lang, RawKey::HidActivationCancel)
                .to_string(),

            // Mouse Movement
            mouse_move_direction_label: get_raw_translation(lang, RawKey::MouseMoveDirectionLabel)
                .to_string(),
            mouse_move_up: get_raw_translation(lang, RawKey::MouseMoveUp).to_string(),
            mouse_move_down: get_raw_translation(lang, RawKey::MouseMoveDown).to_string(),
            mouse_move_left: get_raw_translation(lang, RawKey::MouseMoveLeft).to_string(),
            mouse_move_right: get_raw_translation(lang, RawKey::MouseMoveRight).to_string(),
            mouse_move_up_left: get_raw_translation(lang, RawKey::MouseMoveUpLeft).to_string(),
            mouse_move_up_right: get_raw_translation(lang, RawKey::MouseMoveUpRight).to_string(),
            mouse_move_down_left: get_raw_translation(lang, RawKey::MouseMoveDownLeft).to_string(),
            mouse_move_down_right: get_raw_translation(lang, RawKey::MouseMoveDownRight)
                .to_string(),
            set_mouse_direction_hover: get_raw_translation(lang, RawKey::SetMouseDirectionHover)
                .to_string(),

            // Mouse Scroll
            mouse_scroll_direction_label: get_raw_translation(
                lang,
                RawKey::MouseScrollDirectionLabel,
            )
            .to_string(),
            mouse_scroll_up: get_raw_translation(lang, RawKey::MouseScrollUp).to_string(),
            mouse_scroll_down: get_raw_translation(lang, RawKey::MouseScrollDown).to_string(),

            // Hover hints
            set_mouse_scroll_direction_hover: get_raw_translation(
                lang,
                RawKey::SetMouseScrollDirectionHover,
            )
            .to_string(),
            speed_label: get_raw_translation(lang, RawKey::SpeedLabel).to_string(),
            capture_mode_label: get_raw_translation(lang, RawKey::CaptureModeLabel).to_string(),
            capture_mode_most_sustained: get_raw_translation(
                lang,
                RawKey::CaptureModeMostSustained,
            )
            .to_string(),
            capture_mode_adaptive_intelligent: get_raw_translation(
                lang,
                RawKey::CaptureModeAdaptiveIntelligent,
            )
            .to_string(),
            capture_mode_max_changed_bits: get_raw_translation(
                lang,
                RawKey::CaptureModeMaxChangedBits,
            )
            .to_string(),
            capture_mode_max_set_bits: get_raw_translation(lang, RawKey::CaptureModeMaxSetBits)
                .to_string(),
            capture_mode_last_stable: get_raw_translation(lang, RawKey::CaptureModeLastStable)
                .to_string(),
            capture_mode_hat_switch_optimized: get_raw_translation(
                lang,
                RawKey::CaptureModeHatSwitchOptimized,
            )
            .to_string(),
            capture_mode_analog_optimized: get_raw_translation(
                lang,
                RawKey::CaptureModeAnalogOptimized,
            )
            .to_string(),
            add_target_key_hover: get_raw_translation(lang, RawKey::AddTargetKeyHover).to_string(),
            clear_all_target_keys_hover: get_raw_translation(lang, RawKey::ClearAllTargetKeysHover)
                .to_string(),
            remove_target_key_prefix: get_raw_translation(lang, RawKey::RemoveTargetKeyPrefix)
                .to_string(),
            diagonal_hint_prefix: get_raw_translation(lang, RawKey::DiagonalHintPrefix).to_string(),
            diagonal_hint_suffix: get_raw_translation(lang, RawKey::DiagonalHintSuffix).to_string(),
        }
    }
}

/// Raw translation keys for efficient lookup.
#[derive(Debug, Clone, Copy)]
enum RawKey {
    Dark,
    Light,
    Paused,
    Running,
    ClickToSet,
    AlwaysOnTop,
    ShowTrayIcon,
    ShowNotifications,
    SettingsDialogTitle,
    Language,
    DarkMode,
    ToggleKeySection,
    KeyLabel,
    PressAnyKey,
    Trigger,
    Target,
    IntervalMs,
    DurationMs,
    Click,
    ProcessWhitelistHint,
    ProcessExample,
    ChangesTakeEffect,
    CloseSubtitle,
    Close,
    AboutDescriptionLine1,
    AboutDescriptionLine2,
    Author,
    GitHub,
    License,
    BuiltWith,
    Yes,
    No,

    AppTitle,
    SettingsBtn,
    AboutBtn,
    StatusTitle,
    PauseBtn,
    StartBtn,
    ExitBtn,
    HotkeySettingsTitle,
    ToggleKeyLabel,
    ConfigSettingsTitle,
    InputTimeoutDisplay,
    DefaultIntervalDisplay,
    DefaultDurationDisplay,
    ShowTrayIconDisplay,
    ShowNotificationsDisplay,
    AlwaysOnTopDisplay,
    KeyMappingsTitle,
    GlobalConfigTitle,
    InputTimeoutLabel,
    DefaultIntervalLabel,
    DefaultDurationLabel,
    WorkerCountLabel,
    TriggerShort,
    TargetShort,
    IntShort,
    DurShort,
    AddNewMappingTitle,
    AddBtn,
    BrowseBtn,
    SaveChangesBtn,
    CancelSettingsBtn,
    CloseWindowTitle,
    MinimizeToTrayBtn,
    ExitProgramBtn,
    CancelCloseBtn,
    ErrorTitle,
    DuplicateTriggerError,
    DuplicateProcessError,
    AboutInspired,
    TurboOnHover,
    TurboOffHover,
    TurboHeader,
    HidActivationTitle,
    HidActivationPressPrompt,
    HidActivationReleasePrompt,
    HidActivationWarningTitle,
    HidActivationWarning1,
    HidActivationWarning2,
    HidActivationWarning3,
    HidActivationSuccessTitle,
    HidActivationSuccessMessage,
    HidActivationSuccessHint,
    HidActivationAutoClose,
    HidActivationFailedTitle,
    HidActivationError,
    HidActivationRetry,
    HidActivationCancel,
    MouseMoveDirectionLabel,
    MouseMoveUp,
    MouseMoveDown,
    MouseMoveLeft,
    MouseMoveRight,
    MouseMoveUpLeft,
    MouseMoveUpRight,
    MouseMoveDownLeft,
    MouseMoveDownRight,
    SetMouseDirectionHover,
    MouseScrollDirectionLabel,
    MouseScrollUp,
    MouseScrollDown,
    SetMouseScrollDirectionHover,
    SpeedLabel,
    CaptureModeLabel,
    CaptureModeMostSustained,
    CaptureModeAdaptiveIntelligent,
    CaptureModeMaxChangedBits,
    CaptureModeMaxSetBits,
    CaptureModeLastStable,
    CaptureModeHatSwitchOptimized,
    CaptureModeAnalogOptimized,
    AddTargetKeyHover,
    ClearAllTargetKeysHover,
    RemoveTargetKeyPrefix,
    DiagonalHintPrefix,
    DiagonalHintSuffix,
}

/// Gets raw translation string without formatting.
fn get_raw_translation(lang: Language, key: RawKey) -> &'static str {
    match (lang, key) {
        // App Title
        (Language::English, RawKey::AppTitle) => "🌸 Sorahk ~ Auto Key Press Tool ~",
        (Language::SimplifiedChinese, RawKey::AppTitle) => "🌸 Sorahk ~ 自动连发工具 ~",
        (Language::TraditionalChinese, RawKey::AppTitle) => "🌸 Sorahk ~ 自動連發工具 ~",
        (Language::Japanese, RawKey::AppTitle) => "🌸 Sorahk ~ 自動連打ツール ~",

        // Dark
        (Language::English, RawKey::Dark) => "Dark",
        (Language::SimplifiedChinese, RawKey::Dark) => "深色",
        (Language::TraditionalChinese, RawKey::Dark) => "深色",
        (Language::Japanese, RawKey::Dark) => "ダーク",

        // Light
        (Language::English, RawKey::Light) => "Light",
        (Language::SimplifiedChinese, RawKey::Light) => "浅色",
        (Language::TraditionalChinese, RawKey::Light) => "淺色",
        (Language::Japanese, RawKey::Light) => "ライト",

        // Paused
        (Language::English, RawKey::Paused) => "Paused",
        (Language::SimplifiedChinese, RawKey::Paused) => "已暂停",
        (Language::TraditionalChinese, RawKey::Paused) => "已暫停",
        (Language::Japanese, RawKey::Paused) => "一時停止中",

        // Click to Set
        (Language::English, RawKey::ClickToSet) => "Click to set key",
        (Language::SimplifiedChinese, RawKey::ClickToSet) => "点击设置按键",
        (Language::TraditionalChinese, RawKey::ClickToSet) => "點擊設定按鍵",
        (Language::Japanese, RawKey::ClickToSet) => "クリックでキー設定",

        // Always on Top
        (Language::English, RawKey::AlwaysOnTop) => "Always on Top:",
        (Language::SimplifiedChinese, RawKey::AlwaysOnTop) => "置顶:",
        (Language::TraditionalChinese, RawKey::AlwaysOnTop) => "置頂:",
        (Language::Japanese, RawKey::AlwaysOnTop) => "常に手前に表示:",

        // Show Tray Icon
        (Language::English, RawKey::ShowTrayIcon) => "Show Tray Icon:",
        (Language::SimplifiedChinese, RawKey::ShowTrayIcon) => "显示托盘图标:",
        (Language::TraditionalChinese, RawKey::ShowTrayIcon) => "顯示托盤圖示:",
        (Language::Japanese, RawKey::ShowTrayIcon) => "トレイアイコンを表示:",

        // Show Notifications
        (Language::English, RawKey::ShowNotifications) => "Show Notifications:",
        (Language::SimplifiedChinese, RawKey::ShowNotifications) => "显示通知:",
        (Language::TraditionalChinese, RawKey::ShowNotifications) => "顯示通知:",
        (Language::Japanese, RawKey::ShowNotifications) => "通知を表示:",

        // Settings Dialog Title
        (Language::English, RawKey::SettingsDialogTitle) => "⚙ Settings ~ Configuration Panel ~",
        (Language::SimplifiedChinese, RawKey::SettingsDialogTitle) => "⚙ 设置 ~ 配置面板 ~",
        (Language::TraditionalChinese, RawKey::SettingsDialogTitle) => "⚙ 設定 ~ 配置面板 ~",
        (Language::Japanese, RawKey::SettingsDialogTitle) => "⚙ 設定 ~ 環境設定 ~",

        // Language & Dark Mode
        (Language::English, RawKey::Language) => "Language:",
        (Language::SimplifiedChinese, RawKey::Language) => "语言:",
        (Language::TraditionalChinese, RawKey::Language) => "語言:",
        (Language::Japanese, RawKey::Language) => "言語:",

        (Language::English, RawKey::DarkMode) => "Dark Mode:",
        (Language::SimplifiedChinese, RawKey::DarkMode) => "暗黑模式:",
        (Language::TraditionalChinese, RawKey::DarkMode) => "暗黑模式:",
        (Language::Japanese, RawKey::DarkMode) => "ダークモード:",

        // Toggle Key Section
        (Language::English, RawKey::KeyLabel) => "Key:",
        (Language::SimplifiedChinese, RawKey::KeyLabel) => "按键:",
        (Language::TraditionalChinese, RawKey::KeyLabel) => "按鍵:",
        (Language::Japanese, RawKey::KeyLabel) => "キー:",

        // Press Any Key
        (Language::English, RawKey::PressAnyKey) => "Press any key...",
        (Language::SimplifiedChinese, RawKey::PressAnyKey) => "请按任意键...",
        (Language::TraditionalChinese, RawKey::PressAnyKey) => "請按任意鍵...",
        (Language::Japanese, RawKey::PressAnyKey) => "任意のキーを押してください...",

        // Close Subtitle
        (Language::English, RawKey::CloseSubtitle) => "What would you like to do?",
        (Language::SimplifiedChinese, RawKey::CloseSubtitle) => "想做什么呢？",
        (Language::TraditionalChinese, RawKey::CloseSubtitle) => "想做什麼呢？",
        (Language::Japanese, RawKey::CloseSubtitle) => "いかがなさいますか？",

        // Close
        (Language::English, RawKey::Close) => "✨ Close",
        (Language::SimplifiedChinese, RawKey::Close) => "✨ 关闭",
        (Language::TraditionalChinese, RawKey::Close) => "✨ 關閉",
        (Language::Japanese, RawKey::Close) => "✨ 閉じる",

        // About Description Line 1
        (Language::English, RawKey::AboutDescriptionLine1) => {
            "A lightweight, efficient auto key press tool"
        }
        (Language::SimplifiedChinese, RawKey::AboutDescriptionLine1) => "轻量高效的自动连发工具",
        (Language::TraditionalChinese, RawKey::AboutDescriptionLine1) => "輕量高效的自動連發工具",
        (Language::Japanese, RawKey::AboutDescriptionLine1) => "軽量で高効率な自動連打ツール",

        // About Description Line 2
        (Language::English, RawKey::AboutDescriptionLine2) => "with a beautiful interface",
        (Language::SimplifiedChinese, RawKey::AboutDescriptionLine2) => "拥有漂亮的界面",
        (Language::TraditionalChinese, RawKey::AboutDescriptionLine2) => "擁有漂亮的介面",
        (Language::Japanese, RawKey::AboutDescriptionLine2) => "美しいインターフェース",

        // Running
        (Language::English, RawKey::Running) => "Running",
        (Language::SimplifiedChinese, RawKey::Running) => "连发中",
        (Language::TraditionalChinese, RawKey::Running) => "連發中",
        (Language::Japanese, RawKey::Running) => "連打中",

        // Worker Count
        // Trigger
        (Language::English, RawKey::Trigger) => "Trigger",
        (Language::SimplifiedChinese, RawKey::Trigger) => "触发键",
        (Language::TraditionalChinese, RawKey::Trigger) => "觸發鍵",
        (Language::Japanese, RawKey::Trigger) => "起動キー",

        // Target
        (Language::English, RawKey::Target) => "Target",
        (Language::SimplifiedChinese, RawKey::Target) => "连发键",
        (Language::TraditionalChinese, RawKey::Target) => "連發鍵",
        (Language::Japanese, RawKey::Target) => "連打キー",

        // Interval(ms) - Main window table header
        (Language::English, RawKey::IntervalMs) => "Interval(ms)",
        (Language::SimplifiedChinese, RawKey::IntervalMs) => "连发间隔(ms)",
        (Language::TraditionalChinese, RawKey::IntervalMs) => "連發間隔(ms)",
        (Language::Japanese, RawKey::IntervalMs) => "連打間隔(ms)",

        // Duration(ms) - Main window table header
        (Language::English, RawKey::DurationMs) => "Duration(ms)",
        (Language::SimplifiedChinese, RawKey::DurationMs) => "按键时长(ms)",
        (Language::TraditionalChinese, RawKey::DurationMs) => "按鍵時長(ms)",
        (Language::Japanese, RawKey::DurationMs) => "押下持続(ms)",

        // Add New Mapping
        // Click
        (Language::English, RawKey::Click) => "Click",
        (Language::SimplifiedChinese, RawKey::Click) => "点击",
        (Language::TraditionalChinese, RawKey::Click) => "點擊",
        (Language::Japanese, RawKey::Click) => "クリック",

        // Process Whitelist Hint
        (Language::English, RawKey::ProcessWhitelistHint) => {
            "Process Whitelist (Empty = All Enabled)"
        }
        (Language::SimplifiedChinese, RawKey::ProcessWhitelistHint) => "进程白名单（空=全部启用）",
        (Language::TraditionalChinese, RawKey::ProcessWhitelistHint) => "程序白名單（空=全部啟用）",
        (Language::Japanese, RawKey::ProcessWhitelistHint) => {
            "プロセスホワイトリスト（空=全て有効）"
        }

        // Process Example
        (Language::English, RawKey::ProcessExample) => "e.g., notepad.exe",
        (Language::SimplifiedChinese, RawKey::ProcessExample) => "如：notepad.exe",
        (Language::TraditionalChinese, RawKey::ProcessExample) => "如：notepad.exe",
        (Language::Japanese, RawKey::ProcessExample) => "例: notepad.exe",

        // Changes Take Effect
        (Language::English, RawKey::ChangesTakeEffect) => {
            "* Settings will take effect immediately after saving"
        }
        (Language::SimplifiedChinese, RawKey::ChangesTakeEffect) => "* 配置将在保存后立即生效",
        (Language::TraditionalChinese, RawKey::ChangesTakeEffect) => "* 配置將於儲存後立即生效",
        (Language::Japanese, RawKey::ChangesTakeEffect) => "* 設定は保存後すぐに反映されます",

        // Author
        (Language::English, RawKey::Author) => "👤 Author:",
        (Language::SimplifiedChinese, RawKey::Author) => "👤 作者:",
        (Language::TraditionalChinese, RawKey::Author) => "👤 作者:",
        (Language::Japanese, RawKey::Author) => "👤 作者:",

        // GitHub
        (Language::English, RawKey::GitHub) => "🔗 GitHub:",
        (Language::SimplifiedChinese, RawKey::GitHub) => "🔗 GitHub:",
        (Language::TraditionalChinese, RawKey::GitHub) => "🔗 GitHub:",
        (Language::Japanese, RawKey::GitHub) => "🔗 GitHub:",

        // License
        (Language::English, RawKey::License) => "📜 License:",
        (Language::SimplifiedChinese, RawKey::License) => "📜 许可证:",
        (Language::TraditionalChinese, RawKey::License) => "📜 許可證:",
        (Language::Japanese, RawKey::License) => "📜 ライセンス:",

        // Built With
        (Language::English, RawKey::BuiltWith) => "⚙ Built with:",
        (Language::SimplifiedChinese, RawKey::BuiltWith) => "⚙ 构建工具:",
        (Language::TraditionalChinese, RawKey::BuiltWith) => "⚙ 建置工具:",
        (Language::Japanese, RawKey::BuiltWith) => "⚙ 使用技術:",

        // Yes
        (Language::English, RawKey::Yes) => "Yes",
        (Language::SimplifiedChinese, RawKey::Yes) => "是",
        (Language::TraditionalChinese, RawKey::Yes) => "是",
        (Language::Japanese, RawKey::Yes) => "はい",

        // No
        (Language::English, RawKey::No) => "No",
        (Language::SimplifiedChinese, RawKey::No) => "否",
        (Language::TraditionalChinese, RawKey::No) => "否",
        (Language::Japanese, RawKey::No) => "いいえ",

        (Language::English, RawKey::SettingsBtn) => "⚙  Settings",
        (Language::SimplifiedChinese, RawKey::SettingsBtn) => "⚙  设置",
        (Language::TraditionalChinese, RawKey::SettingsBtn) => "⚙  設定",
        (Language::Japanese, RawKey::SettingsBtn) => "⚙  設定",

        (Language::English, RawKey::AboutBtn) => "❤  About",
        (Language::SimplifiedChinese, RawKey::AboutBtn) => "❤  关于",
        (Language::TraditionalChinese, RawKey::AboutBtn) => "❤  關於",
        (Language::Japanese, RawKey::AboutBtn) => "❤  概要",

        // Main Window - Status Card
        (Language::English, RawKey::StatusTitle) => "📊 Status",
        (Language::SimplifiedChinese, RawKey::StatusTitle) => "📊 状态",
        (Language::TraditionalChinese, RawKey::StatusTitle) => "📊 狀態",
        (Language::Japanese, RawKey::StatusTitle) => "📊 ステータス",

        (Language::English, RawKey::PauseBtn) => "⏸  Pause",
        (Language::SimplifiedChinese, RawKey::PauseBtn) => "⏸  暂停",
        (Language::TraditionalChinese, RawKey::PauseBtn) => "⏸  暫停",
        (Language::Japanese, RawKey::PauseBtn) => "⏸  一時停止",

        (Language::English, RawKey::StartBtn) => "▶  Start",
        (Language::SimplifiedChinese, RawKey::StartBtn) => "▶  启动",
        (Language::TraditionalChinese, RawKey::StartBtn) => "▶  啟動",
        (Language::Japanese, RawKey::StartBtn) => "▶  起動",

        (Language::English, RawKey::ExitBtn) => "✕  Exit",
        (Language::SimplifiedChinese, RawKey::ExitBtn) => "✕  退出",
        (Language::TraditionalChinese, RawKey::ExitBtn) => "✕  退出",
        (Language::Japanese, RawKey::ExitBtn) => "✕  終了",

        // Main Window - Config Settings Card
        (Language::English, RawKey::ShowTrayIconDisplay) => "Show Tray Icon:",
        (Language::SimplifiedChinese, RawKey::ShowTrayIconDisplay) => "显示托盘图标:",
        (Language::TraditionalChinese, RawKey::ShowTrayIconDisplay) => "顯示托盤圖示:",
        (Language::Japanese, RawKey::ShowTrayIconDisplay) => "トレイアイコンを表示:",

        (Language::English, RawKey::ShowNotificationsDisplay) => "Show Notifications:",
        (Language::SimplifiedChinese, RawKey::ShowNotificationsDisplay) => "显示通知:",
        (Language::TraditionalChinese, RawKey::ShowNotificationsDisplay) => "顯示通知:",
        (Language::Japanese, RawKey::ShowNotificationsDisplay) => "通知を表示:",

        (Language::English, RawKey::AlwaysOnTopDisplay) => "Always on Top:",
        (Language::SimplifiedChinese, RawKey::AlwaysOnTopDisplay) => "置顶:",
        (Language::TraditionalChinese, RawKey::AlwaysOnTopDisplay) => "置頂:",
        (Language::Japanese, RawKey::AlwaysOnTopDisplay) => "常に手前に表示:",

        // Settings Dialog - Key Mappings Section
        (Language::English, RawKey::TriggerShort) => "Trigger:",
        (Language::SimplifiedChinese, RawKey::TriggerShort) => "触发键:",
        (Language::TraditionalChinese, RawKey::TriggerShort) => "觸發鍵:",
        (Language::Japanese, RawKey::TriggerShort) => "起動キー:",

        (Language::English, RawKey::TargetShort) => "Target:",
        (Language::SimplifiedChinese, RawKey::TargetShort) => "连发键:",
        (Language::TraditionalChinese, RawKey::TargetShort) => "連發鍵:",
        (Language::Japanese, RawKey::TargetShort) => "連打キー:",

        (Language::English, RawKey::IntShort) => "Int:",
        (Language::SimplifiedChinese, RawKey::IntShort) => "间隔:",
        (Language::TraditionalChinese, RawKey::IntShort) => "間隔:",
        (Language::Japanese, RawKey::IntShort) => "間隔:",

        (Language::English, RawKey::DurShort) => "Dur:",
        (Language::SimplifiedChinese, RawKey::DurShort) => "时长:",
        (Language::TraditionalChinese, RawKey::DurShort) => "時長:",
        (Language::Japanese, RawKey::DurShort) => "持続:",

        (Language::English, RawKey::AddBtn) => "➕ Add",
        (Language::SimplifiedChinese, RawKey::AddBtn) => "➕ 添加",
        (Language::TraditionalChinese, RawKey::AddBtn) => "➕ 新增",
        (Language::Japanese, RawKey::AddBtn) => "➕ 追加",

        // Settings Dialog - Process Whitelist Section
        (Language::English, RawKey::BrowseBtn) => "🗂  Browse",
        (Language::SimplifiedChinese, RawKey::BrowseBtn) => "🗂  浏览",
        (Language::TraditionalChinese, RawKey::BrowseBtn) => "🗂  瀏覽",
        (Language::Japanese, RawKey::BrowseBtn) => "🗂  参照",

        // Settings Dialog - Action Buttons
        (Language::English, RawKey::CancelSettingsBtn) => "↩  Cancel",
        (Language::SimplifiedChinese, RawKey::CancelSettingsBtn) => "↩  取消",
        (Language::TraditionalChinese, RawKey::CancelSettingsBtn) => "↩  取消",
        (Language::Japanese, RawKey::CancelSettingsBtn) => "↩  キャンセル",

        // Close Dialog
        (Language::English, RawKey::CancelCloseBtn) => "↩  Cancel",
        (Language::SimplifiedChinese, RawKey::CancelCloseBtn) => "↩  取消",
        (Language::TraditionalChinese, RawKey::CancelCloseBtn) => "↩  取消",
        (Language::Japanese, RawKey::CancelCloseBtn) => "↩  キャンセル",

        // Error Dialog
        (Language::English, RawKey::ErrorTitle) => "❌ Configuration Error",
        (Language::SimplifiedChinese, RawKey::ErrorTitle) => "❌ 配置错误",
        (Language::TraditionalChinese, RawKey::ErrorTitle) => "❌ 配置錯誤",
        (Language::Japanese, RawKey::ErrorTitle) => "❌ 設定エラー",

        (Language::English, RawKey::DuplicateTriggerError) => "⚠ This trigger key already exists!",
        (Language::SimplifiedChinese, RawKey::DuplicateTriggerError) => "⚠ 该触发键已存在！",
        (Language::TraditionalChinese, RawKey::DuplicateTriggerError) => "⚠ 該觸發鍵已存在！",
        (Language::Japanese, RawKey::DuplicateTriggerError) => "⚠ この起動キーは既に存在します！",

        (Language::English, RawKey::DuplicateProcessError) => {
            "⚠ This process already exists in the whitelist!"
        }
        (Language::SimplifiedChinese, RawKey::DuplicateProcessError) => "⚠ 该进程已在白名单中！",
        (Language::TraditionalChinese, RawKey::DuplicateProcessError) => "⚠ 該進程已在白名單中！",
        (Language::Japanese, RawKey::DuplicateProcessError) => {
            "⚠ このプロセスは既にホワイトリストに存在します！"
        }

        // About Dialog
        (Language::English, RawKey::AboutInspired) => "🌸 Inspired by Kasugano Sora",
        (Language::SimplifiedChinese, RawKey::AboutInspired) => "🌸 灵感来源: 春日野穹",
        (Language::TraditionalChinese, RawKey::AboutInspired) => "🌸 靈感來源: 春日野穹",
        (Language::Japanese, RawKey::AboutInspired) => "🌸 インスパイア: かすがのそら",

        // Turbo toggle tooltips
        (Language::English, RawKey::TurboOnHover) => "Turbo ON - Auto-repeat enabled",
        (Language::SimplifiedChinese, RawKey::TurboOnHover) => "连发开启 - 自动重复输入",
        (Language::TraditionalChinese, RawKey::TurboOnHover) => "連發開啟 - 自動重複輸入",
        (Language::Japanese, RawKey::TurboOnHover) => "連打オン - 自動連打",

        (Language::English, RawKey::TurboOffHover) => "Turbo OFF - Single press only",
        (Language::SimplifiedChinese, RawKey::TurboOffHover) => "连发关闭 - 仅单次输入",
        (Language::TraditionalChinese, RawKey::TurboOffHover) => "連發關閉 - 僅單次輸入",
        (Language::Japanese, RawKey::TurboOffHover) => "連打オフ - 単発入力",

        (Language::English, RawKey::TurboHeader) => "Turbo",
        (Language::SimplifiedChinese, RawKey::TurboHeader) => "连发",
        (Language::TraditionalChinese, RawKey::TurboHeader) => "連發",
        (Language::Japanese, RawKey::TurboHeader) => "連打",

        (Language::English, RawKey::HotkeySettingsTitle) => "⌨ Hotkey Settings",
        (Language::SimplifiedChinese, RawKey::HotkeySettingsTitle) => "⌨ 快捷键设置",
        (Language::TraditionalChinese, RawKey::HotkeySettingsTitle) => "⌨ 快速鍵設定",
        (Language::Japanese, RawKey::HotkeySettingsTitle) => "⌨ ショートカット設定",

        (Language::English, RawKey::ToggleKeyLabel) => "Toggle Key:",
        (Language::SimplifiedChinese, RawKey::ToggleKeyLabel) => "开关键:",
        (Language::TraditionalChinese, RawKey::ToggleKeyLabel) => "開關鍵:",
        (Language::Japanese, RawKey::ToggleKeyLabel) => "切替キー:",

        (Language::English, RawKey::ConfigSettingsTitle) => "⚙ Config Settings",
        (Language::SimplifiedChinese, RawKey::ConfigSettingsTitle) => "⚙ 配置设置",
        (Language::TraditionalChinese, RawKey::ConfigSettingsTitle) => "⚙ 配置設定",
        (Language::Japanese, RawKey::ConfigSettingsTitle) => "⚙ 設定",

        (Language::English, RawKey::InputTimeoutDisplay) => "Input Timeout (ms):",
        (Language::SimplifiedChinese, RawKey::InputTimeoutDisplay) => "输入超时 (毫秒):",
        (Language::TraditionalChinese, RawKey::InputTimeoutDisplay) => "輸入超時 (毫秒):",
        (Language::Japanese, RawKey::InputTimeoutDisplay) => "入力タイムアウト (ms):",

        (Language::English, RawKey::DefaultIntervalDisplay) => "Default Interval (ms):",
        (Language::SimplifiedChinese, RawKey::DefaultIntervalDisplay) => "默认间隔 (毫秒):",
        (Language::TraditionalChinese, RawKey::DefaultIntervalDisplay) => "預設間隔 (毫秒):",
        (Language::Japanese, RawKey::DefaultIntervalDisplay) => "デフォルト間隔 (ms):",

        (Language::English, RawKey::DefaultDurationDisplay) => "Default Duration (ms):",
        (Language::SimplifiedChinese, RawKey::DefaultDurationDisplay) => "默认时长 (毫秒):",
        (Language::TraditionalChinese, RawKey::DefaultDurationDisplay) => "預設時長 (毫秒):",
        (Language::Japanese, RawKey::DefaultDurationDisplay) => "デフォルト持続時間 (ms):",

        (Language::English, RawKey::KeyMappingsTitle) => "🎯 Key Mappings",
        (Language::SimplifiedChinese, RawKey::KeyMappingsTitle) => "🎯 按键映射",
        (Language::TraditionalChinese, RawKey::KeyMappingsTitle) => "🎯 按鍵映射",
        (Language::Japanese, RawKey::KeyMappingsTitle) => "🎯 キーマッピング",

        (Language::English, RawKey::GlobalConfigTitle) => "⚙ Global Configuration",
        (Language::SimplifiedChinese, RawKey::GlobalConfigTitle) => "⚙ 全局配置",
        (Language::TraditionalChinese, RawKey::GlobalConfigTitle) => "⚙ 全局配置",
        (Language::Japanese, RawKey::GlobalConfigTitle) => "⚙ グローバル設定",

        (Language::English, RawKey::InputTimeoutLabel) => "Input Timeout (ms):",
        (Language::SimplifiedChinese, RawKey::InputTimeoutLabel) => "输入超时 (毫秒):",
        (Language::TraditionalChinese, RawKey::InputTimeoutLabel) => "輸入超時 (毫秒):",
        (Language::Japanese, RawKey::InputTimeoutLabel) => "入力タイムアウト (ms):",

        (Language::English, RawKey::DefaultIntervalLabel) => "Default Interval (ms):",
        (Language::SimplifiedChinese, RawKey::DefaultIntervalLabel) => "默认间隔 (毫秒):",
        (Language::TraditionalChinese, RawKey::DefaultIntervalLabel) => "預設間隔 (毫秒):",
        (Language::Japanese, RawKey::DefaultIntervalLabel) => "デフォルト間隔 (ms):",

        (Language::English, RawKey::DefaultDurationLabel) => "Default Duration (ms):",
        (Language::SimplifiedChinese, RawKey::DefaultDurationLabel) => "默认时长 (毫秒):",
        (Language::TraditionalChinese, RawKey::DefaultDurationLabel) => "預設時長 (毫秒):",
        (Language::Japanese, RawKey::DefaultDurationLabel) => "デフォルト持続時間 (ms):",

        (Language::English, RawKey::WorkerCountLabel) => "⚡ Worker Count:",
        (Language::SimplifiedChinese, RawKey::WorkerCountLabel) => "⚡ 连发线程数:",
        (Language::TraditionalChinese, RawKey::WorkerCountLabel) => "⚡ 連發執行緒數:",
        (Language::Japanese, RawKey::WorkerCountLabel) => "⚡ 連打スレッド数:",

        (Language::English, RawKey::AddNewMappingTitle) => "➕ Add New Mapping",
        (Language::SimplifiedChinese, RawKey::AddNewMappingTitle) => "➕ 添加连发映射",
        (Language::TraditionalChinese, RawKey::AddNewMappingTitle) => "➕ 新增連發映射",
        (Language::Japanese, RawKey::AddNewMappingTitle) => "➕ 新規マッピング追加",

        (Language::English, RawKey::SaveChangesBtn) => "💾  Save Settings",
        (Language::SimplifiedChinese, RawKey::SaveChangesBtn) => "💾  保存配置",
        (Language::TraditionalChinese, RawKey::SaveChangesBtn) => "💾  儲存配置",
        (Language::Japanese, RawKey::SaveChangesBtn) => "💾  設定を保存",

        (Language::English, RawKey::CloseWindowTitle) => "💫 Close Window",
        (Language::SimplifiedChinese, RawKey::CloseWindowTitle) => "💫 关闭窗口",
        (Language::TraditionalChinese, RawKey::CloseWindowTitle) => "💫 關閉視窗",
        (Language::Japanese, RawKey::CloseWindowTitle) => "💫 ウィンドウを閉じる",

        (Language::English, RawKey::MinimizeToTrayBtn) => "🗕  Minimize to Tray",
        (Language::SimplifiedChinese, RawKey::MinimizeToTrayBtn) => "🗕  最小化到托盘",
        (Language::TraditionalChinese, RawKey::MinimizeToTrayBtn) => "🗕  最小化至托盤",
        (Language::Japanese, RawKey::MinimizeToTrayBtn) => "🗕  トレイに最小化",

        (Language::English, RawKey::ExitProgramBtn) => "🚪  Exit Program",
        (Language::SimplifiedChinese, RawKey::ExitProgramBtn) => "🚪  退出程序",
        (Language::TraditionalChinese, RawKey::ExitProgramBtn) => "🚪  退出程式",
        (Language::Japanese, RawKey::ExitProgramBtn) => "🚪  プログラムを終了",

        (Language::English, RawKey::ToggleKeySection) => "⌨ Toggle Key",
        (Language::SimplifiedChinese, RawKey::ToggleKeySection) => "⌨ 开关键",
        (Language::TraditionalChinese, RawKey::ToggleKeySection) => "⌨ 開關鍵",
        (Language::Japanese, RawKey::ToggleKeySection) => "⌨ 切替キー",

        // HID Activation Dialog
        (Language::English, RawKey::HidActivationTitle) => "🎮 ✨ Device Activation ✨ 🎮",
        (Language::SimplifiedChinese, RawKey::HidActivationTitle) => "🎮 ✨ 设备激活 ✨ 🎮",
        (Language::TraditionalChinese, RawKey::HidActivationTitle) => "🎮 ✨ 裝置激活 ✨ 🎮",
        (Language::Japanese, RawKey::HidActivationTitle) => "🎮 ✨ デバイス初期化 ✨ 🎮",

        (Language::English, RawKey::HidActivationPressPrompt) => {
            "Press a button, nya~ (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧"
        }
        (Language::SimplifiedChinese, RawKey::HidActivationPressPrompt) => {
            "请按下一个按键喵~ (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧"
        }
        (Language::TraditionalChinese, RawKey::HidActivationPressPrompt) => {
            "請按下一個按鍵喵~ (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧"
        }
        (Language::Japanese, RawKey::HidActivationPressPrompt) => "ボタンを押してね〜 (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧",

        (Language::English, RawKey::HidActivationReleasePrompt) => "Good! Now release it~ ✧(｡•̀ᴗ-)✧",
        (Language::SimplifiedChinese, RawKey::HidActivationReleasePrompt) => {
            "很好！现在松开按键吧~ ✧(｡•̀ᴗ-)✧"
        }
        (Language::TraditionalChinese, RawKey::HidActivationReleasePrompt) => {
            "很好！現在鬆開按鍵吧~ ✧(｡•̀ᴗ-)✧"
        }
        (Language::Japanese, RawKey::HidActivationReleasePrompt) => {
            "いいね！今は離してね〜 ✧(｡•̀ᴗ-)✧"
        }

        (Language::English, RawKey::HidActivationWarningTitle) => "⚠️ Important ⚠️",
        (Language::SimplifiedChinese, RawKey::HidActivationWarningTitle) => "⚠️ 注意事项 ⚠️",
        (Language::TraditionalChinese, RawKey::HidActivationWarningTitle) => "⚠️ 注意事項 ⚠️",
        (Language::Japanese, RawKey::HidActivationWarningTitle) => "⚠️ 注意事項 ⚠️",

        (Language::English, RawKey::HidActivationWarning1) => "• Press only ONE button!",
        (Language::SimplifiedChinese, RawKey::HidActivationWarning1) => "• 只能按一个键哦！",
        (Language::TraditionalChinese, RawKey::HidActivationWarning1) => "• 只能按一個鍵哦！",
        (Language::Japanese, RawKey::HidActivationWarning1) => "• ボタン1個だけ押してね！",

        (Language::English, RawKey::HidActivationWarning2) => "• Don't press multiple buttons",
        (Language::SimplifiedChinese, RawKey::HidActivationWarning2) => "• 不要同时按多个键",
        (Language::TraditionalChinese, RawKey::HidActivationWarning2) => "• 不要同時按多個鍵",
        (Language::Japanese, RawKey::HidActivationWarning2) => "• 複数ボタン押さないでね",

        (Language::English, RawKey::HidActivationWarning3) => {
            "• Remember to release after pressing~"
        }
        (Language::SimplifiedChinese, RawKey::HidActivationWarning3) => "• 按下后记得松开~",
        (Language::TraditionalChinese, RawKey::HidActivationWarning3) => "• 按下後記得鬆開~",
        (Language::Japanese, RawKey::HidActivationWarning3) => "• 押したら必ず離してね〜",

        (Language::English, RawKey::HidActivationSuccessTitle) => "🎉 Success! 🎉",
        (Language::SimplifiedChinese, RawKey::HidActivationSuccessTitle) => "🎉 激活成功！ 🎉",
        (Language::TraditionalChinese, RawKey::HidActivationSuccessTitle) => "🎉 激活成功！ 🎉",
        (Language::Japanese, RawKey::HidActivationSuccessTitle) => "🎉 成功！ 🎉",

        (Language::English, RawKey::HidActivationSuccessMessage) => "Device activated!",
        (Language::SimplifiedChinese, RawKey::HidActivationSuccessMessage) => "设备激活完成！",
        (Language::TraditionalChinese, RawKey::HidActivationSuccessMessage) => "裝置激活完成！",
        (Language::Japanese, RawKey::HidActivationSuccessMessage) => "デバイス初期化完了！",

        (Language::English, RawKey::HidActivationSuccessHint) => {
            "You can now use turbo-fire~ (｡♥‿♥｡)"
        }
        (Language::SimplifiedChinese, RawKey::HidActivationSuccessHint) => {
            "现在可以使用连发功能啦~ (｡♥‿♥｡)"
        }
        (Language::TraditionalChinese, RawKey::HidActivationSuccessHint) => {
            "現在可以使用連發功能啦~ (｡♥‿♥｡)"
        }
        (Language::Japanese, RawKey::HidActivationSuccessHint) => {
            "連打機能が使えるようになったよ~ (｡♥‿♥｡)"
        }

        (Language::English, RawKey::HidActivationAutoClose) => "Closing automatically...",
        (Language::SimplifiedChinese, RawKey::HidActivationAutoClose) => "窗口即将自动关闭...",
        (Language::TraditionalChinese, RawKey::HidActivationAutoClose) => "視窗即將自動關閉...",
        (Language::Japanese, RawKey::HidActivationAutoClose) => "自動的に閉じます...",

        (Language::English, RawKey::HidActivationFailedTitle) => "❌ Activation Failed ❌",
        (Language::SimplifiedChinese, RawKey::HidActivationFailedTitle) => "❌ 激活失败 ❌",
        (Language::TraditionalChinese, RawKey::HidActivationFailedTitle) => "❌ 激活失敗 ❌",
        (Language::Japanese, RawKey::HidActivationFailedTitle) => "❌ 初期化失敗 ❌",

        (Language::English, RawKey::HidActivationError) => "Error",
        (Language::SimplifiedChinese, RawKey::HidActivationError) => "错误",
        (Language::TraditionalChinese, RawKey::HidActivationError) => "錯誤",
        (Language::Japanese, RawKey::HidActivationError) => "エラー",

        (Language::English, RawKey::HidActivationRetry) => "🔄 Retry",
        (Language::SimplifiedChinese, RawKey::HidActivationRetry) => "🔄 重试",
        (Language::TraditionalChinese, RawKey::HidActivationRetry) => "🔄 重試",
        (Language::Japanese, RawKey::HidActivationRetry) => "🔄 再試行",

        (Language::English, RawKey::HidActivationCancel) => "✖ Cancel",
        (Language::SimplifiedChinese, RawKey::HidActivationCancel) => "✖ 取消",
        (Language::TraditionalChinese, RawKey::HidActivationCancel) => "✖ 取消",
        (Language::Japanese, RawKey::HidActivationCancel) => "✖ キャンセル",

        // Mouse Movement
        (Language::English, RawKey::MouseMoveDirectionLabel) => "Direction:",
        (Language::SimplifiedChinese, RawKey::MouseMoveDirectionLabel) => "移动方向:",
        (Language::TraditionalChinese, RawKey::MouseMoveDirectionLabel) => "移動方向:",
        (Language::Japanese, RawKey::MouseMoveDirectionLabel) => "移動方向:",

        (Language::English, RawKey::MouseMoveUp) => "↑ Up",
        (Language::SimplifiedChinese, RawKey::MouseMoveUp) => "↑ 向上",
        (Language::TraditionalChinese, RawKey::MouseMoveUp) => "↑ 向上",
        (Language::Japanese, RawKey::MouseMoveUp) => "↑ 上",

        (Language::English, RawKey::MouseMoveDown) => "↓ Down",
        (Language::SimplifiedChinese, RawKey::MouseMoveDown) => "↓ 向下",
        (Language::TraditionalChinese, RawKey::MouseMoveDown) => "↓ 向下",
        (Language::Japanese, RawKey::MouseMoveDown) => "↓ 下",

        (Language::English, RawKey::MouseMoveLeft) => "← Left",
        (Language::SimplifiedChinese, RawKey::MouseMoveLeft) => "← 向左",
        (Language::TraditionalChinese, RawKey::MouseMoveLeft) => "← 向左",
        (Language::Japanese, RawKey::MouseMoveLeft) => "← 左",

        (Language::English, RawKey::MouseMoveRight) => "→ Right",
        (Language::SimplifiedChinese, RawKey::MouseMoveRight) => "→ 向右",
        (Language::TraditionalChinese, RawKey::MouseMoveRight) => "→ 向右",
        (Language::Japanese, RawKey::MouseMoveRight) => "→ 右",

        (Language::English, RawKey::MouseMoveUpLeft) => "↖ Up-Left",
        (Language::SimplifiedChinese, RawKey::MouseMoveUpLeft) => "↖ 左上",
        (Language::TraditionalChinese, RawKey::MouseMoveUpLeft) => "↖ 左上",
        (Language::Japanese, RawKey::MouseMoveUpLeft) => "↖ 左上",

        (Language::English, RawKey::MouseMoveUpRight) => "↗ Up-Right",
        (Language::SimplifiedChinese, RawKey::MouseMoveUpRight) => "↗ 右上",
        (Language::TraditionalChinese, RawKey::MouseMoveUpRight) => "↗ 右上",
        (Language::Japanese, RawKey::MouseMoveUpRight) => "↗ 右上",

        (Language::English, RawKey::MouseMoveDownLeft) => "↙ Down-Left",
        (Language::SimplifiedChinese, RawKey::MouseMoveDownLeft) => "↙ 左下",
        (Language::TraditionalChinese, RawKey::MouseMoveDownLeft) => "↙ 左下",
        (Language::Japanese, RawKey::MouseMoveDownLeft) => "↙ 左下",

        (Language::English, RawKey::MouseMoveDownRight) => "↘ Down-Right",
        (Language::SimplifiedChinese, RawKey::MouseMoveDownRight) => "↘ 右下",
        (Language::TraditionalChinese, RawKey::MouseMoveDownRight) => "↘ 右下",
        (Language::Japanese, RawKey::MouseMoveDownRight) => "↘ 右下",

        (Language::English, RawKey::SetMouseDirectionHover) => "Set mouse movement direction",
        (Language::SimplifiedChinese, RawKey::SetMouseDirectionHover) => "设置鼠标移动方向",
        (Language::TraditionalChinese, RawKey::SetMouseDirectionHover) => "設定滑鼠移動方向",
        (Language::Japanese, RawKey::SetMouseDirectionHover) => "マウス移動方向を設定",

        // Mouse Scroll
        (Language::English, RawKey::MouseScrollDirectionLabel) => "Scroll Direction",
        (Language::SimplifiedChinese, RawKey::MouseScrollDirectionLabel) => "滚动方向",
        (Language::TraditionalChinese, RawKey::MouseScrollDirectionLabel) => "滾動方向",
        (Language::Japanese, RawKey::MouseScrollDirectionLabel) => "スクロール方向",

        (Language::English, RawKey::MouseScrollUp) => "Scroll Up",
        (Language::SimplifiedChinese, RawKey::MouseScrollUp) => "向上滚动",
        (Language::TraditionalChinese, RawKey::MouseScrollUp) => "向上滾動",
        (Language::Japanese, RawKey::MouseScrollUp) => "上にスクロール",

        (Language::English, RawKey::MouseScrollDown) => "Scroll Down",
        (Language::SimplifiedChinese, RawKey::MouseScrollDown) => "向下滚动",
        (Language::TraditionalChinese, RawKey::MouseScrollDown) => "向下滾動",
        (Language::Japanese, RawKey::MouseScrollDown) => "下にスクロール",

        // Hover hints
        (Language::English, RawKey::SetMouseScrollDirectionHover) => "Set mouse scroll direction",
        (Language::SimplifiedChinese, RawKey::SetMouseScrollDirectionHover) => "设置鼠标滚动方向",
        (Language::TraditionalChinese, RawKey::SetMouseScrollDirectionHover) => "設定滑鼠滾動方向",
        (Language::Japanese, RawKey::SetMouseScrollDirectionHover) => "マウススクロール方向を設定",

        (Language::English, RawKey::SpeedLabel) => "Speed:",
        (Language::SimplifiedChinese, RawKey::SpeedLabel) => "速度:",
        (Language::TraditionalChinese, RawKey::SpeedLabel) => "速度:",
        (Language::Japanese, RawKey::SpeedLabel) => "速度:",

        // Capture Mode
        (Language::English, RawKey::CaptureModeLabel) => "HID Capture Mode:",
        (Language::SimplifiedChinese, RawKey::CaptureModeLabel) => "按键捕获模式:",
        (Language::TraditionalChinese, RawKey::CaptureModeLabel) => "按鍵捕獲模式:",
        (Language::Japanese, RawKey::CaptureModeLabel) => "入力検出モード:",

        (Language::English, RawKey::CaptureModeMostSustained) => "Most Sustained",
        (Language::SimplifiedChinese, RawKey::CaptureModeMostSustained) => "持续时间最长",
        (Language::TraditionalChinese, RawKey::CaptureModeMostSustained) => "持續時間最長",
        (Language::Japanese, RawKey::CaptureModeMostSustained) => "継続時間優先",

        (Language::English, RawKey::CaptureModeAdaptiveIntelligent) => "Adaptive Intelligent",
        (Language::SimplifiedChinese, RawKey::CaptureModeAdaptiveIntelligent) => "智能自适应",
        (Language::TraditionalChinese, RawKey::CaptureModeAdaptiveIntelligent) => "智能自適應",
        (Language::Japanese, RawKey::CaptureModeAdaptiveIntelligent) => "自動判別",

        (Language::English, RawKey::CaptureModeMaxChangedBits) => "Max Changed Bits",
        (Language::SimplifiedChinese, RawKey::CaptureModeMaxChangedBits) => "最大变化量",
        (Language::TraditionalChinese, RawKey::CaptureModeMaxChangedBits) => "最大變化量",
        (Language::Japanese, RawKey::CaptureModeMaxChangedBits) => "最大変化量",

        (Language::English, RawKey::CaptureModeMaxSetBits) => "Max Set Bits",
        (Language::SimplifiedChinese, RawKey::CaptureModeMaxSetBits) => "最大激活量",
        (Language::TraditionalChinese, RawKey::CaptureModeMaxSetBits) => "最大激活量",
        (Language::Japanese, RawKey::CaptureModeMaxSetBits) => "最大アクティブ量",

        (Language::English, RawKey::CaptureModeLastStable) => "Last Stable",
        (Language::SimplifiedChinese, RawKey::CaptureModeLastStable) => "最终稳定状态",
        (Language::TraditionalChinese, RawKey::CaptureModeLastStable) => "最終穩定狀態",
        (Language::Japanese, RawKey::CaptureModeLastStable) => "最終安定状態",

        (Language::English, RawKey::CaptureModeHatSwitchOptimized) => "Hat Switch Optimized",
        (Language::SimplifiedChinese, RawKey::CaptureModeHatSwitchOptimized) => "摇杆方向优化",
        (Language::TraditionalChinese, RawKey::CaptureModeHatSwitchOptimized) => "搖桿方向優化",
        (Language::Japanese, RawKey::CaptureModeHatSwitchOptimized) => "十字キー特化",

        (Language::English, RawKey::CaptureModeAnalogOptimized) => "Analog Optimized",
        (Language::SimplifiedChinese, RawKey::CaptureModeAnalogOptimized) => "模拟摇杆优化",
        (Language::TraditionalChinese, RawKey::CaptureModeAnalogOptimized) => "類比搖桿優化",
        (Language::Japanese, RawKey::CaptureModeAnalogOptimized) => "アナログスティック特化",

        (Language::English, RawKey::AddTargetKeyHover) => "➕ Add target key",
        (Language::SimplifiedChinese, RawKey::AddTargetKeyHover) => "➕ 添加目标键",
        (Language::TraditionalChinese, RawKey::AddTargetKeyHover) => "➕ 添加目標鍵",
        (Language::Japanese, RawKey::AddTargetKeyHover) => "➕ ターゲットキー追加",

        (Language::English, RawKey::ClearAllTargetKeysHover) => "🗑 Clear all target keys",
        (Language::SimplifiedChinese, RawKey::ClearAllTargetKeysHover) => "🗑 清除所有目标键",
        (Language::TraditionalChinese, RawKey::ClearAllTargetKeysHover) => "🗑 清除所有目標鍵",
        (Language::Japanese, RawKey::ClearAllTargetKeysHover) => "🗑 すべてのターゲットキーをクリア",

        (Language::English, RawKey::RemoveTargetKeyPrefix) => "🗑 Click to remove",
        (Language::SimplifiedChinese, RawKey::RemoveTargetKeyPrefix) => "🗑 点击移除",
        (Language::TraditionalChinese, RawKey::RemoveTargetKeyPrefix) => "🗑 點擊移除",
        (Language::Japanese, RawKey::RemoveTargetKeyPrefix) => "🗑 クリックで削除",

        (Language::English, RawKey::DiagonalHintPrefix) => {
            "💡 Hint: Adjacent directions detected, diagonal direction "
        }
        (Language::SimplifiedChinese, RawKey::DiagonalHintPrefix) => {
            "💡 提示：检测到相邻方向，斜方向 "
        }
        (Language::TraditionalChinese, RawKey::DiagonalHintPrefix) => {
            "💡 提示：檢測到相鄰方向，斜方向 "
        }
        (Language::Japanese, RawKey::DiagonalHintPrefix) => "💡 ヒント：隣接方向を検出、斜め方向 ",

        (Language::English, RawKey::DiagonalHintSuffix) => " has no effect by default",
        (Language::SimplifiedChinese, RawKey::DiagonalHintSuffix) => " 默认无效果",
        (Language::TraditionalChinese, RawKey::DiagonalHintSuffix) => " 預設無效果",
        (Language::Japanese, RawKey::DiagonalHintSuffix) => " はデフォルトで無効",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_default() {
        let lang = Language::default();
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_language_all() {
        let all_languages = Language::all();
        assert_eq!(all_languages.len(), 4);
        assert_eq!(all_languages[0], Language::English);
        assert_eq!(all_languages[1], Language::SimplifiedChinese);
        assert_eq!(all_languages[2], Language::TraditionalChinese);
        assert_eq!(all_languages[3], Language::Japanese);
    }

    #[test]
    fn test_language_display_names() {
        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::SimplifiedChinese.display_name(), "简体中文");
        assert_eq!(Language::TraditionalChinese.display_name(), "繁體中文");
        assert_eq!(Language::Japanese.display_name(), "日本語");
    }

    #[test]
    fn test_cached_translations_english() {
        let translations = CachedTranslations::new(Language::English);

        assert!(translations.app_title().contains("Sorahk"));
        assert_eq!(translations.settings_button(), "⚙  Settings");
        assert_eq!(translations.about_button(), "❤  About");
        assert_eq!(translations.dark_theme(), "Dark");
        assert_eq!(translations.light_theme(), "Light");
        assert_eq!(translations.paused_status(), "Paused");
        assert_eq!(translations.running_status(), "Running");
    }

    #[test]
    fn test_cached_translations_simplified_chinese() {
        let translations = CachedTranslations::new(Language::SimplifiedChinese);

        assert!(translations.app_title().contains("Sorahk"));
        assert_eq!(translations.settings_button(), "⚙  设置");
        assert_eq!(translations.about_button(), "❤  关于");
        assert_eq!(translations.dark_theme(), "深色");
        assert_eq!(translations.light_theme(), "浅色");
        assert_eq!(translations.paused_status(), "已暂停");
        assert_eq!(translations.running_status(), "连发中");
    }

    #[test]
    fn test_cached_translations_traditional_chinese() {
        let translations = CachedTranslations::new(Language::TraditionalChinese);

        assert!(translations.app_title().contains("Sorahk"));
        assert_eq!(translations.settings_button(), "⚙  設定");
        assert_eq!(translations.about_button(), "❤  關於");
        assert_eq!(translations.dark_theme(), "深色");
        assert_eq!(translations.light_theme(), "淺色");
        assert_eq!(translations.paused_status(), "已暫停");
        assert_eq!(translations.running_status(), "連發中");
    }

    #[test]
    fn test_cached_translations_japanese() {
        let translations = CachedTranslations::new(Language::Japanese);

        assert!(translations.app_title().contains("Sorahk"));
        assert_eq!(translations.settings_button(), "⚙  設定");
        assert_eq!(translations.about_button(), "❤  概要");
        assert_eq!(translations.dark_theme(), "ダーク");
        assert_eq!(translations.light_theme(), "ライト");
        assert_eq!(translations.paused_status(), "一時停止中");
        assert_eq!(translations.running_status(), "連打中");
    }

    #[test]
    fn test_key_mappings_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.trigger_header(), "Trigger");
        assert_eq!(translations.target_header(), "Target");
        assert_eq!(translations.interval_header(), "Interval(ms)");
        assert_eq!(translations.duration_header(), "Duration(ms)");
    }

    #[test]
    fn test_button_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.pause_button(), "⏸  Pause");
        assert_eq!(translations.start_button(), "▶  Start");
        assert_eq!(translations.exit_button(), "✕  Exit");
        assert_eq!(translations.save(), "💾  Save Settings");
        assert_eq!(translations.cancel(), "↩  Cancel");
    }

    #[test]
    fn test_dialog_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(
            translations.settings_dialog_title(),
            "⚙ Settings ~ Configuration Panel ~"
        );
        assert_eq!(translations.close_window_title(), "💫 Close Window");
        assert_eq!(translations.error_title(), "❌ Configuration Error");
        assert_eq!(translations.close_subtitle(), "What would you like to do?");
    }

    #[test]
    fn test_action_button_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(
            translations.minimize_to_tray_button(),
            "🗕  Minimize to Tray"
        );
        assert_eq!(translations.exit_program_button(), "🚪  Exit Program");
        assert_eq!(translations.cancel_close_button(), "↩  Cancel");
    }

    #[test]
    fn test_error_message_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(
            translations.duplicate_trigger_error(),
            "⚠ This trigger key already exists!"
        );
    }

    #[test]
    fn test_about_dialog_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert!(translations.about_version().contains("Version"));
        assert_eq!(
            translations.about_description_line1(),
            "A lightweight, efficient auto key press tool"
        );
        assert_eq!(translations.about_author(), "👤 Author:");
        assert_eq!(translations.about_github(), "🔗 GitHub:");
        assert_eq!(translations.about_license(), "📜 License:");
        assert_eq!(translations.about_mit_license(), "MIT License");
    }

    #[test]
    fn test_settings_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.language(), "Language:");
        assert_eq!(translations.dark_mode(), "Dark Mode:");
        assert_eq!(translations.always_on_top(), "Always on Top:");
        assert_eq!(translations.show_tray_icon(), "Show Tray Icon:");
        assert_eq!(translations.show_notifications(), "Show Notifications:");
    }

    #[test]
    fn test_configuration_labels() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.input_timeout_label(), "Input Timeout (ms):");
        assert_eq!(
            translations.default_interval_label(),
            "Default Interval (ms):"
        );
        assert_eq!(
            translations.default_duration_label(),
            "Default Duration (ms):"
        );
        assert_eq!(translations.worker_count_label(), "⚡ Worker Count:");
    }

    #[test]
    fn test_process_whitelist_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(
            translations.process_whitelist_hint(),
            "Process Whitelist (Empty = All Enabled)"
        );
        assert_eq!(translations.process_example(), "e.g., notepad.exe");
        assert_eq!(translations.browse_button(), "🗂  Browse");
    }

    #[test]
    fn test_yes_no_translations() {
        let en = CachedTranslations::new(Language::English);
        assert_eq!(en.yes(), "Yes");
        assert_eq!(en.no(), "No");

        let zh_cn = CachedTranslations::new(Language::SimplifiedChinese);
        assert_eq!(zh_cn.yes(), "是");
        assert_eq!(zh_cn.no(), "否");

        let zh_tw = CachedTranslations::new(Language::TraditionalChinese);
        assert_eq!(zh_tw.yes(), "是");
        assert_eq!(zh_tw.no(), "否");

        let ja = CachedTranslations::new(Language::Japanese);
        assert_eq!(ja.yes(), "はい");
        assert_eq!(ja.no(), "いいえ");
    }

    #[test]
    fn test_format_worker_count() {
        let translations = CachedTranslations::new(Language::English);
        let formatted = translations.format_worker_count(4);
        assert!(formatted.contains("4"));
        assert!(formatted.contains("Worker Count"));
    }

    #[test]
    fn test_translation_consistency_across_languages() {
        let languages = vec![
            Language::English,
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::Japanese,
        ];

        for lang in languages {
            let trans = CachedTranslations::new(lang);

            assert!(!trans.app_title().is_empty());
            assert!(!trans.settings_button().is_empty());
            assert!(!trans.about_button().is_empty());
            assert!(!trans.pause_button().is_empty());
            assert!(!trans.start_button().is_empty());
            assert!(!trans.exit_button().is_empty());
        }
    }

    #[test]
    fn test_cached_translations_cloning() {
        let original = CachedTranslations::new(Language::English);
        let cloned = original.clone();

        assert_eq!(original.app_title(), cloned.app_title());
        assert_eq!(original.settings_button(), cloned.settings_button());
    }

    #[test]
    fn test_hotkey_settings_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.hotkey_settings_title(), "⌨ Hotkey Settings");
        assert_eq!(translations.toggle_key_label(), "Toggle Key:");
        assert_eq!(translations.click_to_set(), "Click to set key");
        assert_eq!(translations.press_any_key(), "Press any key...");
    }

    #[test]
    fn test_config_settings_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.config_settings_title(), "⚙ Config Settings");
        assert_eq!(translations.global_config_title(), "⚙ Global Configuration");
        assert_eq!(translations.key_mappings_title(), "🎯 Key Mappings");
    }

    #[test]
    fn test_add_mapping_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.add_new_mapping_title(), "➕ Add New Mapping");
        assert_eq!(translations.add_button_text(), "➕ Add");
        assert_eq!(translations.click_text(), "Click");
    }

    #[test]
    fn test_short_form_translations() {
        let translations = CachedTranslations::new(Language::English);

        assert_eq!(translations.trigger_short(), "Trigger:");
        assert_eq!(translations.target_short(), "Target:");
        assert_eq!(translations.interval_short(), "Int:");
        assert_eq!(translations.duration_short(), "Dur:");
    }

    #[test]
    fn test_changes_hint_translation() {
        let translations = CachedTranslations::new(Language::English);
        assert_eq!(
            translations.changes_take_effect_hint(),
            "* Settings will take effect immediately after saving"
        );
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::English, Language::English);
        assert_ne!(Language::English, Language::SimplifiedChinese);
        assert_ne!(Language::SimplifiedChinese, Language::TraditionalChinese);
        assert_ne!(Language::TraditionalChinese, Language::Japanese);
    }

    #[test]
    fn test_all_translations_present() {
        let languages = Language::all();

        for lang in languages {
            let trans = CachedTranslations::new(*lang);

            assert!(
                !trans.app_title().is_empty(),
                "Missing app_title for {:?}",
                lang
            );
            assert!(
                !trans.status_title().is_empty(),
                "Missing status_title for {:?}",
                lang
            );
            assert!(
                !trans.key_mappings_title().is_empty(),
                "Missing key_mappings_title for {:?}",
                lang
            );
            assert!(
                !trans.settings_dialog_title().is_empty(),
                "Missing settings_dialog_title for {:?}",
                lang
            );
        }
    }
}
