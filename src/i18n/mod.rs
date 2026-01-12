//! Internationalization support for multiple languages.
//!
//! Cached translation strings for UI elements. All strings resolve to
//! `&'static str` values from compile-time constant tables. Lookup is a
//! pointer dereference with no runtime allocation.
//!
//! The per-language string data lives in the `en`, `zh_cn`, `zh_tw`, and
//! `ja` submodules. This module exposes the cached API surface.

mod en;
mod ja;
mod zh_cn;
mod zh_tw;

use serde::{Deserialize, Serialize};

// Split into sibling files to stay under the file-size guideline.
mod accessors;
mod raw_key;
#[cfg(test)]
mod tests;

use raw_key::{RawKey, get_raw_translation};

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

    /// Convert Language to u8 for atomic storage
    pub fn to_u8(self) -> u8 {
        match self {
            Language::English => 0,
            Language::SimplifiedChinese => 1,
            Language::TraditionalChinese => 2,
            Language::Japanese => 3,
        }
    }

    /// Convert u8 to Language
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Language::SimplifiedChinese,
            2 => Language::TraditionalChinese,
            3 => Language::Japanese,
            _ => Language::English,
        }
    }
}

/// Cached translations for the render loop.
///
/// Holds a reference to one of four compile-time `TranslationCache`
/// tables. The struct is a single pointer and is `Copy`, so passing it
/// around is cheap.
#[derive(Clone, Copy)]
pub struct CachedTranslations {
    inner: &'static TranslationCache,
}

#[derive(Clone, Copy)]
struct TranslationCache {
    app_title: &'static str,
    settings_button: &'static str,
    about_button: &'static str,
    dark_theme: &'static str,
    light_theme: &'static str,
    status_title: &'static str,
    paused_status: &'static str,
    running_status: &'static str,
    pause_button: &'static str,
    start_button: &'static str,
    exit_button: &'static str,
    hotkey_settings_title: &'static str,
    toggle_key_label: &'static str,
    click_to_set: &'static str,
    config_settings_title: &'static str,
    input_timeout_display: &'static str,
    default_interval_display: &'static str,
    default_duration_display: &'static str,
    show_tray_icon_display: &'static str,
    show_notifications_display: &'static str,
    always_on_top_display: &'static str,
    yes: &'static str,
    no: &'static str,
    key_mappings_title: &'static str,
    settings_dialog_title: &'static str,
    language_label: &'static str,
    dark_mode_label: &'static str,
    always_on_top_label: &'static str,
    show_tray_icon_label: &'static str,
    show_notifications_label: &'static str,
    toggle_key_section: &'static str,
    sequence_finalize_row_label: &'static str,
    sequence_finalize_hint: &'static str,
    press_any_key: &'static str,
    click_to_set_trigger: &'static str,
    click_to_set_target: &'static str,
    global_config_title: &'static str,
    input_timeout_label: &'static str,
    default_interval_label: &'static str,
    default_duration_label: &'static str,
    worker_count_label: &'static str,
    trigger_short: &'static str,
    target_short: &'static str,
    interval_short: &'static str,
    duration_short: &'static str,
    add_new_mapping_title: &'static str,
    add_button_text: &'static str,
    process_whitelist_hint: &'static str,
    process_example: &'static str,
    browse_button: &'static str,
    save_changes_button: &'static str,
    cancel_settings_button: &'static str,
    changes_take_effect_hint: &'static str,
    close_window_title: &'static str,
    close_subtitle: &'static str,
    minimize_to_tray_button: &'static str,
    exit_program_button: &'static str,
    cancel_close_button: &'static str,
    error_title: &'static str,
    error_close_button: &'static str,
    duplicate_trigger_error: &'static str,
    duplicate_process_error: &'static str,
    about_version: &'static str,
    about_description_line1: &'static str,
    about_description_line2: &'static str,
    about_author: &'static str,
    about_github: &'static str,
    about_license: &'static str,
    about_built_with: &'static str,
    about_mit_license: &'static str,
    about_rust_egui: &'static str,
    about_inspired: &'static str,
    turbo_on_hover: &'static str,
    turbo_off_hover: &'static str,
    hid_activation_title: &'static str,
    hid_activation_press_prompt: &'static str,
    hid_activation_release_prompt: &'static str,
    hid_activation_warning_title: &'static str,
    hid_activation_warning_1: &'static str,
    hid_activation_warning_2: &'static str,
    hid_activation_warning_3: &'static str,
    hid_activation_success_title: &'static str,
    hid_activation_success_message: &'static str,
    hid_activation_success_hint: &'static str,
    hid_activation_auto_close: &'static str,
    hid_activation_failed_title: &'static str,
    hid_activation_error: &'static str,
    hid_activation_retry: &'static str,
    hid_activation_cancel: &'static str,
    mouse_move_direction_label: &'static str,
    mouse_move_up: &'static str,
    mouse_move_down: &'static str,
    mouse_move_left: &'static str,
    mouse_move_right: &'static str,
    mouse_move_up_left: &'static str,
    mouse_move_up_right: &'static str,
    mouse_move_down_left: &'static str,
    mouse_move_down_right: &'static str,
    set_mouse_direction_hover: &'static str,
    mouse_scroll_direction_label: &'static str,
    mouse_scroll_up: &'static str,
    mouse_scroll_down: &'static str,
    mouse_middle_button: &'static str,
    set_mouse_scroll_direction_hover: &'static str,
    speed_label: &'static str,
    rawinput_capture_mode_label: &'static str,
    xinput_capture_mode_label: &'static str,
    capture_mode_most_sustained: &'static str,
    capture_mode_adaptive_intelligent: &'static str,
    capture_mode_max_changed_bits: &'static str,
    capture_mode_max_set_bits: &'static str,
    capture_mode_last_stable: &'static str,
    capture_mode_hat_switch_optimized: &'static str,
    capture_mode_analog_optimized: &'static str,
    capture_mode_diagonal_priority: &'static str,
    add_target_key_hover: &'static str,
    add_sequence_key_hover: &'static str,
    clear_all_trigger_keys_hover: &'static str,
    clear_all_target_keys_hover: &'static str,
    diagonal_hint_title: &'static str,
    diagonal_hint: &'static str,

    // Sequence Trigger
    trigger_mode_label: &'static str,
    trigger_mode_single: &'static str,
    trigger_mode_sequence: &'static str,
    trigger_mode_single_badge: &'static str,
    trigger_mode_sequence_badge: &'static str,
    sequence_trigger_explanation: &'static str,
    sequence_window_label: &'static str,
    sequence_window_hint: &'static str,
    sequence_capturing: &'static str,
    sequence_capture_hint: &'static str,
    sequence_complete: &'static str,
    sequence_clear_btn: &'static str,
    sequence_example_label: &'static str,

    // Target Mode
    target_mode_label: &'static str,
    target_mode_single: &'static str,
    target_mode_multi: &'static str,
    target_mode_sequence: &'static str,
    target_mode_single_badge: &'static str,
    target_mode_multi_badge: &'static str,
    target_mode_sequence_badge: &'static str,
    target_mode_multi_explanation: &'static str,
    target_mode_sequence_explanation: &'static str,
    target_sequence_output_hint: &'static str,

    // Device Manager Dialog
    devices_button: &'static str,
    device_manager_title: &'static str,
    connected_devices_title: &'static str,
    refresh_button: &'static str,
    xinput_controllers_title: &'static str,
    no_controllers_connected: &'static str,
    hid_devices_title: &'static str,
    no_hid_devices_detected: &'static str,
    slot_label: &'static str,
    hide_button: &'static str,
    device_settings_button: &'static str,
    vibration_control_title: &'static str,
    left_motor_label: &'static str,
    right_motor_label: &'static str,
    power_label: &'static str,
    test_vibration_button: &'static str,
    stop_vibration_button: &'static str,
    deadzone_settings_title: &'static str,
    stick_label: &'static str,
    trigger_label_short: &'static str,
    threshold_label: &'static str,
    device_manager_close_button: &'static str,
    preferred_api_label: &'static str,
    api_auto: &'static str,
    api_xinput: &'static str,
    api_rawinput: &'static str,
    reactivate_button: &'static str,
    all_devices_filter: &'static str,
    game_devices_only_filter: &'static str,
    no_game_devices_detected: &'static str,

    // Tray Icon
    tray_activate: &'static str,
    tray_pause: &'static str,
    tray_show_window: &'static str,
    tray_about: &'static str,
    tray_exit: &'static str,
    tray_notification_launched: &'static str,
    tray_notification_activated: &'static str,
    tray_notification_paused: &'static str,

    // UI Icons
    sequence_icon: &'static str,
    target_icon: &'static str,
    arrow_icon: &'static str,
    delete_icon: &'static str,
    keys_text: &'static str,
    targets_text: &'static str,
}

/// Pre-built translation tables, one per supported language. Built at
/// compile time from `TranslationCache::new(lang)`.
static EN_CACHE: TranslationCache = TranslationCache::new(Language::English);
static ZH_CN_CACHE: TranslationCache = TranslationCache::new(Language::SimplifiedChinese);
static ZH_TW_CACHE: TranslationCache = TranslationCache::new(Language::TraditionalChinese);
static JA_CACHE: TranslationCache = TranslationCache::new(Language::Japanese);

impl TranslationCache {
    /// Builds a translation cache at compile time for the given language.
    /// Only callable in const contexts via the static caches above.
    const fn new(lang: Language) -> Self {
        Self {
            // Main Window - Title Bar
            app_title: get_raw_translation(lang, RawKey::AppTitle),
            settings_button: get_raw_translation(lang, RawKey::SettingsBtn),
            about_button: get_raw_translation(lang, RawKey::AboutBtn),
            dark_theme: get_raw_translation(lang, RawKey::Dark),
            light_theme: get_raw_translation(lang, RawKey::Light),

            // Main Window - Status Card
            status_title: get_raw_translation(lang, RawKey::StatusTitle),
            paused_status: get_raw_translation(lang, RawKey::Paused),
            running_status: get_raw_translation(lang, RawKey::Running),
            pause_button: get_raw_translation(lang, RawKey::PauseBtn),
            start_button: get_raw_translation(lang, RawKey::StartBtn),
            exit_button: get_raw_translation(lang, RawKey::ExitBtn),

            // Main Window - Hotkey Settings Card
            hotkey_settings_title: get_raw_translation(lang, RawKey::HotkeySettingsTitle),
            toggle_key_label: get_raw_translation(lang, RawKey::ToggleKeyLabel),
            click_to_set: get_raw_translation(lang, RawKey::ClickToSet),

            // Main Window - Config Settings Card
            config_settings_title: get_raw_translation(lang, RawKey::ConfigSettingsTitle),
            input_timeout_display: get_raw_translation(lang, RawKey::InputTimeoutDisplay),
            default_interval_display: get_raw_translation(lang, RawKey::DefaultIntervalDisplay),
            default_duration_display: get_raw_translation(lang, RawKey::DefaultDurationDisplay),
            show_tray_icon_display: get_raw_translation(lang, RawKey::ShowTrayIconDisplay),
            show_notifications_display: get_raw_translation(lang, RawKey::ShowNotificationsDisplay),
            always_on_top_display: get_raw_translation(lang, RawKey::AlwaysOnTopDisplay),
            yes: get_raw_translation(lang, RawKey::Yes),
            no: get_raw_translation(lang, RawKey::No),

            // Main Window - Key Mappings Card
            key_mappings_title: get_raw_translation(lang, RawKey::KeyMappingsTitle),

            // Settings Dialog - Title
            settings_dialog_title: get_raw_translation(lang, RawKey::SettingsDialogTitle),

            // Settings Dialog - Language & Appearance Section
            language_label: get_raw_translation(lang, RawKey::Language),
            dark_mode_label: get_raw_translation(lang, RawKey::DarkMode),
            always_on_top_label: get_raw_translation(lang, RawKey::AlwaysOnTop),
            show_tray_icon_label: get_raw_translation(lang, RawKey::ShowTrayIcon),
            show_notifications_label: get_raw_translation(lang, RawKey::ShowNotifications),

            // Settings Dialog - Toggle Key Section
            toggle_key_section: get_raw_translation(lang, RawKey::ToggleKeySection),
            sequence_finalize_row_label: get_raw_translation(
                lang,
                RawKey::SequenceFinalizeRowLabel,
            ),
            sequence_finalize_hint: get_raw_translation(lang, RawKey::SequenceFinalizeHint),
            press_any_key: get_raw_translation(lang, RawKey::PressAnyKey),
            click_to_set_trigger: get_raw_translation(lang, RawKey::ClickToSetTrigger),
            click_to_set_target: get_raw_translation(lang, RawKey::ClickToSetTarget),

            // Settings Dialog - Global Configuration Section
            global_config_title: get_raw_translation(lang, RawKey::GlobalConfigTitle),
            input_timeout_label: get_raw_translation(lang, RawKey::InputTimeoutLabel),
            default_interval_label: get_raw_translation(lang, RawKey::DefaultIntervalLabel),
            default_duration_label: get_raw_translation(lang, RawKey::DefaultDurationLabel),
            worker_count_label: get_raw_translation(lang, RawKey::WorkerCountLabel),

            // Settings Dialog - Key Mappings Section
            trigger_short: get_raw_translation(lang, RawKey::TriggerShort),
            target_short: get_raw_translation(lang, RawKey::TargetShort),
            interval_short: get_raw_translation(lang, RawKey::IntShort),
            duration_short: get_raw_translation(lang, RawKey::DurShort),

            add_new_mapping_title: get_raw_translation(lang, RawKey::AddNewMappingTitle),
            add_button_text: get_raw_translation(lang, RawKey::AddBtn),

            // Settings Dialog - Process Whitelist Section
            process_whitelist_hint: get_raw_translation(lang, RawKey::ProcessWhitelistHint),
            process_example: get_raw_translation(lang, RawKey::ProcessExample),
            browse_button: get_raw_translation(lang, RawKey::BrowseBtn),

            // Settings Dialog - Action Buttons
            save_changes_button: get_raw_translation(lang, RawKey::SaveChangesBtn),
            cancel_settings_button: get_raw_translation(lang, RawKey::CancelSettingsBtn),
            changes_take_effect_hint: get_raw_translation(lang, RawKey::ChangesTakeEffect),

            // Close Dialog
            close_window_title: get_raw_translation(lang, RawKey::CloseWindowTitle),
            close_subtitle: get_raw_translation(lang, RawKey::CloseSubtitle),
            minimize_to_tray_button: get_raw_translation(lang, RawKey::MinimizeToTrayBtn),
            exit_program_button: get_raw_translation(lang, RawKey::ExitProgramBtn),
            cancel_close_button: get_raw_translation(lang, RawKey::CancelCloseBtn),

            // Error Dialog
            error_title: get_raw_translation(lang, RawKey::ErrorTitle),
            error_close_button: get_raw_translation(lang, RawKey::Close),
            duplicate_trigger_error: get_raw_translation(lang, RawKey::DuplicateTriggerError),
            duplicate_process_error: get_raw_translation(lang, RawKey::DuplicateProcessError),

            // About Dialog
            about_version: concat!("✨ Version ", env!("CARGO_PKG_VERSION")),
            about_description_line1: get_raw_translation(lang, RawKey::AboutDescriptionLine1),
            about_description_line2: get_raw_translation(lang, RawKey::AboutDescriptionLine2),
            about_author: get_raw_translation(lang, RawKey::Author),
            about_github: get_raw_translation(lang, RawKey::GitHub),
            about_license: get_raw_translation(lang, RawKey::License),
            about_built_with: get_raw_translation(lang, RawKey::BuiltWith),
            about_mit_license: "MIT License",
            about_rust_egui: "Rust + egui",
            about_inspired: get_raw_translation(lang, RawKey::AboutInspired),

            // Turbo toggle tooltips
            turbo_on_hover: get_raw_translation(lang, RawKey::TurboOnHover),
            turbo_off_hover: get_raw_translation(lang, RawKey::TurboOffHover),

            // HID Activation Dialog
            hid_activation_title: get_raw_translation(lang, RawKey::HidActivationTitle),
            hid_activation_press_prompt: get_raw_translation(
                lang,
                RawKey::HidActivationPressPrompt,
            ),
            hid_activation_release_prompt: get_raw_translation(
                lang,
                RawKey::HidActivationReleasePrompt,
            ),
            hid_activation_warning_title: get_raw_translation(
                lang,
                RawKey::HidActivationWarningTitle,
            ),
            hid_activation_warning_1: get_raw_translation(lang, RawKey::HidActivationWarning1),
            hid_activation_warning_2: get_raw_translation(lang, RawKey::HidActivationWarning2),
            hid_activation_warning_3: get_raw_translation(lang, RawKey::HidActivationWarning3),
            hid_activation_success_title: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessTitle,
            ),
            hid_activation_success_message: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessMessage,
            ),
            hid_activation_success_hint: get_raw_translation(
                lang,
                RawKey::HidActivationSuccessHint,
            ),
            hid_activation_auto_close: get_raw_translation(lang, RawKey::HidActivationAutoClose),
            hid_activation_failed_title: get_raw_translation(
                lang,
                RawKey::HidActivationFailedTitle,
            ),
            hid_activation_error: get_raw_translation(lang, RawKey::HidActivationError),
            hid_activation_retry: get_raw_translation(lang, RawKey::HidActivationRetry),
            hid_activation_cancel: get_raw_translation(lang, RawKey::HidActivationCancel),

            // Mouse Movement
            mouse_move_direction_label: get_raw_translation(lang, RawKey::MouseMoveDirectionLabel),
            mouse_move_up: get_raw_translation(lang, RawKey::MouseMoveUp),
            mouse_move_down: get_raw_translation(lang, RawKey::MouseMoveDown),
            mouse_move_left: get_raw_translation(lang, RawKey::MouseMoveLeft),
            mouse_move_right: get_raw_translation(lang, RawKey::MouseMoveRight),
            mouse_move_up_left: get_raw_translation(lang, RawKey::MouseMoveUpLeft),
            mouse_move_up_right: get_raw_translation(lang, RawKey::MouseMoveUpRight),
            mouse_move_down_left: get_raw_translation(lang, RawKey::MouseMoveDownLeft),
            mouse_move_down_right: get_raw_translation(lang, RawKey::MouseMoveDownRight),
            set_mouse_direction_hover: get_raw_translation(lang, RawKey::SetMouseDirectionHover),

            // Mouse Scroll
            mouse_scroll_direction_label: get_raw_translation(
                lang,
                RawKey::MouseScrollDirectionLabel,
            ),
            mouse_scroll_up: get_raw_translation(lang, RawKey::MouseScrollUp),
            mouse_scroll_down: get_raw_translation(lang, RawKey::MouseScrollDown),
            mouse_middle_button: get_raw_translation(lang, RawKey::MouseMiddleButton),

            // Hover hints
            set_mouse_scroll_direction_hover: get_raw_translation(
                lang,
                RawKey::SetMouseScrollDirectionHover,
            ),
            speed_label: get_raw_translation(lang, RawKey::SpeedLabel),
            rawinput_capture_mode_label: get_raw_translation(
                lang,
                RawKey::RawInputCaptureModeLabel,
            ),
            xinput_capture_mode_label: get_raw_translation(lang, RawKey::XInputCaptureModeLabel),
            capture_mode_most_sustained: get_raw_translation(
                lang,
                RawKey::CaptureModeMostSustained,
            ),
            capture_mode_adaptive_intelligent: get_raw_translation(
                lang,
                RawKey::CaptureModeAdaptiveIntelligent,
            ),
            capture_mode_max_changed_bits: get_raw_translation(
                lang,
                RawKey::CaptureModeMaxChangedBits,
            ),
            capture_mode_max_set_bits: get_raw_translation(lang, RawKey::CaptureModeMaxSetBits),
            capture_mode_last_stable: get_raw_translation(lang, RawKey::CaptureModeLastStable),
            capture_mode_hat_switch_optimized: get_raw_translation(
                lang,
                RawKey::CaptureModeHatSwitchOptimized,
            ),
            capture_mode_analog_optimized: get_raw_translation(
                lang,
                RawKey::CaptureModeAnalogOptimized,
            ),
            capture_mode_diagonal_priority: get_raw_translation(
                lang,
                RawKey::CaptureModeDiagonalPriority,
            ),
            add_target_key_hover: get_raw_translation(lang, RawKey::AddTargetKeyHover),
            add_sequence_key_hover: get_raw_translation(lang, RawKey::AddSequenceKeyHover),
            clear_all_trigger_keys_hover: get_raw_translation(
                lang,
                RawKey::ClearAllTriggerKeysHover,
            ),
            clear_all_target_keys_hover: get_raw_translation(lang, RawKey::ClearAllTargetKeysHover),
            diagonal_hint_title: get_raw_translation(lang, RawKey::DiagonalHintTitle),
            diagonal_hint: get_raw_translation(lang, RawKey::DiagonalHint),

            // Sequence Trigger
            trigger_mode_label: get_raw_translation(lang, RawKey::TriggerModeLabel),
            trigger_mode_single: get_raw_translation(lang, RawKey::TriggerModeSingle),
            trigger_mode_sequence: get_raw_translation(lang, RawKey::TriggerModeSequence),
            trigger_mode_single_badge: get_raw_translation(lang, RawKey::TriggerModeSingleBadge),
            trigger_mode_sequence_badge: get_raw_translation(
                lang,
                RawKey::TriggerModeSequenceBadge,
            ),
            sequence_trigger_explanation: get_raw_translation(
                lang,
                RawKey::SequenceTriggerExplanation,
            ),
            sequence_window_label: get_raw_translation(lang, RawKey::SequenceWindowLabel),
            sequence_window_hint: get_raw_translation(lang, RawKey::SequenceWindowHint),
            sequence_capturing: get_raw_translation(lang, RawKey::SequenceCapturing),
            sequence_capture_hint: get_raw_translation(lang, RawKey::SequenceCaptureHint),
            sequence_complete: get_raw_translation(lang, RawKey::SequenceComplete),
            sequence_clear_btn: get_raw_translation(lang, RawKey::SequenceClearBtn),
            sequence_example_label: get_raw_translation(lang, RawKey::SequenceExampleLabel),

            // Target Mode
            target_mode_label: get_raw_translation(lang, RawKey::TargetModeLabel),
            target_mode_single: get_raw_translation(lang, RawKey::TargetModeSingle),
            target_mode_multi: get_raw_translation(lang, RawKey::TargetModeMulti),
            target_mode_sequence: get_raw_translation(lang, RawKey::TargetModeSequence),
            target_mode_single_badge: get_raw_translation(lang, RawKey::TargetModeSingleBadge),
            target_mode_multi_badge: get_raw_translation(lang, RawKey::TargetModeMultiBadge),
            target_mode_sequence_badge: get_raw_translation(lang, RawKey::TargetModeSequenceBadge),
            target_mode_multi_explanation: get_raw_translation(
                lang,
                RawKey::TargetModeMultiExplanation,
            ),
            target_mode_sequence_explanation: get_raw_translation(
                lang,
                RawKey::TargetModeSequenceExplanation,
            ),
            target_sequence_output_hint: get_raw_translation(
                lang,
                RawKey::TargetSequenceOutputHint,
            ),

            // Device Manager Dialog
            devices_button: get_raw_translation(lang, RawKey::DevicesBtn),
            device_manager_title: get_raw_translation(lang, RawKey::DeviceManagerTitle),
            connected_devices_title: get_raw_translation(lang, RawKey::ConnectedDevicesTitle),
            refresh_button: get_raw_translation(lang, RawKey::RefreshBtn),
            xinput_controllers_title: get_raw_translation(lang, RawKey::XInputControllersTitle),
            no_controllers_connected: get_raw_translation(lang, RawKey::NoControllersConnected),
            hid_devices_title: get_raw_translation(lang, RawKey::HidDevicesTitle),
            no_hid_devices_detected: get_raw_translation(lang, RawKey::NoHidDevicesDetected),
            slot_label: get_raw_translation(lang, RawKey::SlotLabel),
            hide_button: get_raw_translation(lang, RawKey::HideBtn),
            device_settings_button: get_raw_translation(lang, RawKey::DeviceSettingsBtn),
            vibration_control_title: get_raw_translation(lang, RawKey::VibrationControlTitle),
            left_motor_label: get_raw_translation(lang, RawKey::LeftMotorLabel),
            right_motor_label: get_raw_translation(lang, RawKey::RightMotorLabel),
            power_label: get_raw_translation(lang, RawKey::PowerLabel),
            test_vibration_button: get_raw_translation(lang, RawKey::TestVibrationBtn),
            stop_vibration_button: get_raw_translation(lang, RawKey::StopVibrationBtn),
            deadzone_settings_title: get_raw_translation(lang, RawKey::DeadzoneSettingsTitle),
            stick_label: get_raw_translation(lang, RawKey::StickLabel),
            trigger_label_short: get_raw_translation(lang, RawKey::TriggerLabelShort),
            threshold_label: get_raw_translation(lang, RawKey::ThresholdLabel),
            device_manager_close_button: get_raw_translation(lang, RawKey::Close),
            preferred_api_label: get_raw_translation(lang, RawKey::PreferredApiLabel),
            api_auto: get_raw_translation(lang, RawKey::ApiAuto),
            api_xinput: get_raw_translation(lang, RawKey::ApiXInput),
            api_rawinput: get_raw_translation(lang, RawKey::ApiRawInput),
            reactivate_button: get_raw_translation(lang, RawKey::ReactivateBtn),
            all_devices_filter: get_raw_translation(lang, RawKey::AllDevicesFilter),
            game_devices_only_filter: get_raw_translation(lang, RawKey::GameDevicesOnlyFilter),
            no_game_devices_detected: get_raw_translation(lang, RawKey::NoGameDevicesDetected),

            // Tray Icon
            tray_activate: get_raw_translation(lang, RawKey::TrayActivate),
            tray_pause: get_raw_translation(lang, RawKey::TrayPause),
            tray_show_window: get_raw_translation(lang, RawKey::TrayShowWindow),
            tray_about: get_raw_translation(lang, RawKey::TrayAbout),
            tray_exit: get_raw_translation(lang, RawKey::TrayExit),
            tray_notification_launched: get_raw_translation(lang, RawKey::TrayNotificationLaunched),
            tray_notification_activated: get_raw_translation(
                lang,
                RawKey::TrayNotificationActivated,
            ),
            tray_notification_paused: get_raw_translation(lang, RawKey::TrayNotificationPaused),

            // UI Icons
            sequence_icon: get_raw_translation(lang, RawKey::SequenceIcon),
            target_icon: get_raw_translation(lang, RawKey::TargetIcon),
            arrow_icon: get_raw_translation(lang, RawKey::ArrowIcon),
            delete_icon: get_raw_translation(lang, RawKey::DeleteIcon),
            keys_text: get_raw_translation(lang, RawKey::KeysText),
            targets_text: get_raw_translation(lang, RawKey::TargetsText),
        }
    }
}
