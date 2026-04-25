//! Accessor methods on `CachedTranslations`.
//!
//! Each method is a one-liner that reads a `&'static str` field from the
//! currently selected `TranslationCache`.

use super::{
    CachedTranslations, EN_CACHE, JA_CACHE, KO_CACHE, Language, TranslationCache, ZH_CN_CACHE,
    ZH_TW_CACHE,
};

impl CachedTranslations {
    /// Returns the cached translations for the specified language.
    /// A single pointer assignment, no allocation.
    #[inline(always)]
    pub fn new(lang: Language) -> Self {
        let inner: &'static TranslationCache = match lang {
            Language::English => &EN_CACHE,
            Language::SimplifiedChinese => &ZH_CN_CACHE,
            Language::TraditionalChinese => &ZH_TW_CACHE,
            Language::Japanese => &JA_CACHE,
            Language::Korean => &KO_CACHE,
        };
        Self { inner }
    }

    // Main Window - Title Bar
    pub fn app_title(&self) -> &str {
        self.inner.app_title
    }
    pub fn settings_button(&self) -> &str {
        self.inner.settings_button
    }
    pub fn about_button(&self) -> &str {
        self.inner.about_button
    }
    pub fn dark_theme(&self) -> &str {
        self.inner.dark_theme
    }
    pub fn light_theme(&self) -> &str {
        self.inner.light_theme
    }
    pub fn language(&self) -> &str {
        self.inner.language_label
    }
    pub fn dark_mode(&self) -> &str {
        self.inner.dark_mode_label
    }
    pub fn always_on_top(&self) -> &str {
        self.inner.always_on_top_label
    }
    pub fn show_tray_icon(&self) -> &str {
        self.inner.show_tray_icon_label
    }
    pub fn show_notifications(&self) -> &str {
        self.inner.show_notifications_label
    }
    pub fn hotkeys_section_title(&self) -> &str {
        self.inner.toggle_key_section
    }
    pub fn sequence_finalize_row_label(&self) -> &str {
        self.inner.sequence_finalize_row_label
    }
    pub fn sequence_finalize_hint(&self) -> &str {
        self.inner.sequence_finalize_hint
    }
    pub fn save(&self) -> &str {
        self.inner.save_changes_button
    }
    pub fn cancel(&self) -> &str {
        self.inner.cancel_settings_button
    }

    // Main Window - Status Card
    pub fn status_title(&self) -> &str {
        self.inner.status_title
    }
    pub fn paused_status(&self) -> &str {
        self.inner.paused_status
    }

    // Main Window - Hotkey Settings Card
    pub fn hotkey_settings_title(&self) -> &str {
        self.inner.hotkey_settings_title
    }
    pub fn toggle_key_label(&self) -> &str {
        self.inner.toggle_key_label
    }
    pub fn click_to_set(&self) -> &str {
        self.inner.click_to_set
    }

    // Main Window - Config Settings Card
    pub fn config_settings_title(&self) -> &str {
        self.inner.config_settings_title
    }

    // Main Window - Key Mappings Card
    pub fn key_mappings_title(&self) -> &str {
        self.inner.key_mappings_title
    }

    // Settings Dialog - Title
    pub fn settings_dialog_title(&self) -> &str {
        self.inner.settings_dialog_title
    }

    // Settings Dialog - Toggle Key Section
    pub fn press_any_key(&self) -> &str {
        self.inner.press_any_key
    }
    pub fn click_to_set_trigger(&self) -> &str {
        self.inner.click_to_set_trigger
    }
    pub fn click_to_set_target(&self) -> &str {
        self.inner.click_to_set_target
    }

    // Settings Dialog - Global Configuration Section
    pub fn global_config_title(&self) -> &str {
        self.inner.global_config_title
    }
    pub fn input_timeout_label(&self) -> &str {
        self.inner.input_timeout_label
    }
    pub fn default_interval_label(&self) -> &str {
        self.inner.default_interval_label
    }
    pub fn default_duration_label(&self) -> &str {
        self.inner.default_duration_label
    }
    pub fn mouse_move_per_event_min_label(&self) -> &str {
        self.inner.mouse_move_per_event_min_label
    }
    pub fn mouse_move_per_event_min_hint(&self) -> &str {
        self.inner.mouse_move_per_event_min_hint
    }
    pub fn mouse_move_min_trigger_label(&self) -> &str {
        self.inner.mouse_move_min_trigger_label
    }
    pub fn mouse_move_min_trigger_hint(&self) -> &str {
        self.inner.mouse_move_min_trigger_hint
    }
    pub fn mouse_move_rearm_label(&self) -> &str {
        self.inner.mouse_move_rearm_label
    }
    pub fn mouse_move_rearm_hint(&self) -> &str {
        self.inner.mouse_move_rearm_hint
    }

    // Close Dialog
    pub fn close_window_title(&self) -> &str {
        self.inner.close_window_title
    }
    pub fn close_subtitle(&self) -> &str {
        self.inner.close_subtitle
    }
    pub fn minimize_to_tray_button(&self) -> &str {
        self.inner.minimize_to_tray_button
    }
    pub fn exit_program_button(&self) -> &str {
        self.inner.exit_program_button
    }
    pub fn cancel_close_button(&self) -> &str {
        self.inner.cancel_close_button
    }

    // Error Dialog
    pub fn error_title(&self) -> &str {
        self.inner.error_title
    }
    pub fn error_close_button(&self) -> &str {
        self.inner.error_close_button
    }
    pub fn duplicate_trigger_error(&self) -> &str {
        self.inner.duplicate_trigger_error
    }

    pub fn duplicate_process_error(&self) -> &str {
        self.inner.duplicate_process_error
    }

    // About Dialog
    pub fn about_version(&self) -> &str {
        self.inner.about_version
    }
    pub fn about_description_line1(&self) -> &str {
        self.inner.about_description_line1
    }
    pub fn about_description_line2(&self) -> &str {
        self.inner.about_description_line2
    }
    pub fn about_author(&self) -> &str {
        self.inner.about_author
    }
    pub fn about_github(&self) -> &str {
        self.inner.about_github
    }
    pub fn about_license(&self) -> &str {
        self.inner.about_license
    }
    pub fn about_built_with(&self) -> &str {
        self.inner.about_built_with
    }
    pub fn about_mit_license(&self) -> &str {
        self.inner.about_mit_license
    }
    pub fn about_rust_egui(&self) -> &str {
        self.inner.about_rust_egui
    }
    pub fn about_inspired(&self) -> &str {
        self.inner.about_inspired
    }

    // Turbo toggle tooltips
    pub fn turbo_on_hover(&self) -> &str {
        self.inner.turbo_on_hover
    }
    pub fn turbo_off_hover(&self) -> &str {
        self.inner.turbo_off_hover
    }

    // HID Activation Dialog
    pub fn hid_activation_title(&self) -> &str {
        self.inner.hid_activation_title
    }
    pub fn hid_activation_press_prompt(&self) -> &str {
        self.inner.hid_activation_press_prompt
    }
    pub fn hid_activation_release_prompt(&self) -> &str {
        self.inner.hid_activation_release_prompt
    }
    pub fn hid_activation_warning_title(&self) -> &str {
        self.inner.hid_activation_warning_title
    }
    pub fn hid_activation_warning_1(&self) -> &str {
        self.inner.hid_activation_warning_1
    }
    pub fn hid_activation_warning_2(&self) -> &str {
        self.inner.hid_activation_warning_2
    }
    pub fn hid_activation_warning_3(&self) -> &str {
        self.inner.hid_activation_warning_3
    }
    pub fn hid_activation_success_title(&self) -> &str {
        self.inner.hid_activation_success_title
    }
    pub fn hid_activation_success_message(&self) -> &str {
        self.inner.hid_activation_success_message
    }
    pub fn hid_activation_success_hint(&self) -> &str {
        self.inner.hid_activation_success_hint
    }
    pub fn hid_activation_auto_close(&self) -> &str {
        self.inner.hid_activation_auto_close
    }
    pub fn hid_activation_failed_title(&self) -> &str {
        self.inner.hid_activation_failed_title
    }
    pub fn hid_activation_error(&self) -> &str {
        self.inner.hid_activation_error
    }
    pub fn hid_activation_retry(&self) -> &str {
        self.inner.hid_activation_retry
    }
    pub fn hid_activation_cancel(&self) -> &str {
        self.inner.hid_activation_cancel
    }

    // Mouse Movement
    pub fn mouse_move_direction_label(&self) -> &str {
        self.inner.mouse_move_direction_label
    }
    pub fn mouse_move_up(&self) -> &str {
        self.inner.mouse_move_up
    }
    pub fn mouse_move_down(&self) -> &str {
        self.inner.mouse_move_down
    }
    pub fn mouse_move_left(&self) -> &str {
        self.inner.mouse_move_left
    }
    pub fn mouse_move_right(&self) -> &str {
        self.inner.mouse_move_right
    }
    pub fn mouse_move_up_left(&self) -> &str {
        self.inner.mouse_move_up_left
    }
    pub fn mouse_move_up_right(&self) -> &str {
        self.inner.mouse_move_up_right
    }
    pub fn mouse_move_down_left(&self) -> &str {
        self.inner.mouse_move_down_left
    }
    pub fn mouse_move_down_right(&self) -> &str {
        self.inner.mouse_move_down_right
    }
    pub fn set_mouse_direction_hover(&self) -> &str {
        self.inner.set_mouse_direction_hover
    }

    // Mouse Scroll
    pub fn mouse_scroll_direction_label(&self) -> &str {
        self.inner.mouse_scroll_direction_label
    }
    pub fn mouse_scroll_up(&self) -> &str {
        self.inner.mouse_scroll_up
    }
    pub fn mouse_scroll_down(&self) -> &str {
        self.inner.mouse_scroll_down
    }
    pub fn mouse_middle_button(&self) -> &str {
        self.inner.mouse_middle_button
    }
    pub fn set_mouse_scroll_direction_hover(&self) -> &str {
        self.inner.set_mouse_scroll_direction_hover
    }
    pub fn speed_label(&self) -> &str {
        self.inner.speed_label
    }

    // Capture Mode
    pub fn rawinput_capture_mode_label(&self) -> &str {
        self.inner.rawinput_capture_mode_label
    }
    pub fn xinput_capture_mode_label(&self) -> &str {
        self.inner.xinput_capture_mode_label
    }
    pub fn capture_mode_most_sustained(&self) -> &str {
        self.inner.capture_mode_most_sustained
    }
    pub fn capture_mode_adaptive_intelligent(&self) -> &str {
        self.inner.capture_mode_adaptive_intelligent
    }
    pub fn capture_mode_max_changed_bits(&self) -> &str {
        self.inner.capture_mode_max_changed_bits
    }
    pub fn capture_mode_max_set_bits(&self) -> &str {
        self.inner.capture_mode_max_set_bits
    }
    pub fn capture_mode_last_stable(&self) -> &str {
        self.inner.capture_mode_last_stable
    }
    pub fn capture_mode_diagonal_priority(&self) -> &str {
        self.inner.capture_mode_diagonal_priority
    }
    pub fn capture_mode_hat_switch_optimized(&self) -> &str {
        self.inner.capture_mode_hat_switch_optimized
    }
    pub fn capture_mode_analog_optimized(&self) -> &str {
        self.inner.capture_mode_analog_optimized
    }

    // Multi-target key support
    pub fn add_target_key_hover(&self) -> &str {
        self.inner.add_target_key_hover
    }
    pub fn add_sequence_key_hover(&self) -> &str {
        self.inner.add_sequence_key_hover
    }
    pub fn clear_all_trigger_keys_hover(&self) -> &str {
        self.inner.clear_all_trigger_keys_hover
    }
    pub fn clear_all_target_keys_hover(&self) -> &str {
        self.inner.clear_all_target_keys_hover
    }
    pub fn diagonal_hint_title(&self) -> &str {
        self.inner.diagonal_hint_title
    }
    pub fn diagonal_hint(&self) -> &str {
        self.inner.diagonal_hint
    }

    // Sequence Trigger
    pub fn trigger_mode_label(&self) -> &str {
        self.inner.trigger_mode_label
    }
    pub fn trigger_mode_single(&self) -> &str {
        self.inner.trigger_mode_single
    }
    pub fn trigger_mode_sequence(&self) -> &str {
        self.inner.trigger_mode_sequence
    }
    pub fn trigger_mode_single_badge(&self) -> &str {
        self.inner.trigger_mode_single_badge
    }
    pub fn trigger_mode_sequence_badge(&self) -> &str {
        self.inner.trigger_mode_sequence_badge
    }
    pub fn sequence_trigger_explanation(&self) -> &str {
        self.inner.sequence_trigger_explanation
    }
    pub fn sequence_window_label(&self) -> &str {
        self.inner.sequence_window_label
    }
    pub fn sequence_window_hint(&self) -> &str {
        self.inner.sequence_window_hint
    }
    pub fn sequence_capturing(&self) -> &str {
        self.inner.sequence_capturing
    }
    pub fn sequence_capture_hint(&self) -> &str {
        self.inner.sequence_capture_hint
    }
    pub fn sequence_complete(&self) -> &str {
        self.inner.sequence_complete
    }
    pub fn sequence_clear_btn(&self) -> &str {
        self.inner.sequence_clear_btn
    }
    pub fn sequence_example_label(&self) -> &str {
        self.inner.sequence_example_label
    }
    pub fn target_mode_label(&self) -> &str {
        self.inner.target_mode_label
    }
    pub fn target_mode_single(&self) -> &str {
        self.inner.target_mode_single
    }
    pub fn target_mode_multi(&self) -> &str {
        self.inner.target_mode_multi
    }
    pub fn target_mode_sequence(&self) -> &str {
        self.inner.target_mode_sequence
    }
    pub fn target_mode_single_badge(&self) -> &str {
        self.inner.target_mode_single_badge
    }
    pub fn target_mode_multi_badge(&self) -> &str {
        self.inner.target_mode_multi_badge
    }
    pub fn target_mode_sequence_badge(&self) -> &str {
        self.inner.target_mode_sequence_badge
    }
    pub fn target_mode_multi_explanation(&self) -> &str {
        self.inner.target_mode_multi_explanation
    }
    pub fn target_mode_sequence_explanation(&self) -> &str {
        self.inner.target_mode_sequence_explanation
    }
    pub fn target_sequence_output_hint(&self) -> &str {
        self.inner.target_sequence_output_hint
    }

    // Device Manager Dialog
    pub fn devices_button(&self) -> &str {
        self.inner.devices_button
    }
    pub fn device_manager_title(&self) -> &str {
        self.inner.device_manager_title
    }
    pub fn connected_devices_title(&self) -> &str {
        self.inner.connected_devices_title
    }
    pub fn refresh_button(&self) -> &str {
        self.inner.refresh_button
    }
    pub fn xinput_controllers_title(&self) -> &str {
        self.inner.xinput_controllers_title
    }
    pub fn no_controllers_connected(&self) -> &str {
        self.inner.no_controllers_connected
    }
    pub fn hid_devices_title(&self) -> &str {
        self.inner.hid_devices_title
    }
    pub fn no_hid_devices_detected(&self) -> &str {
        self.inner.no_hid_devices_detected
    }
    pub fn slot_label(&self) -> &str {
        self.inner.slot_label
    }
    pub fn hide_button(&self) -> &str {
        self.inner.hide_button
    }
    pub fn device_settings_button(&self) -> &str {
        self.inner.device_settings_button
    }
    pub fn vibration_control_title(&self) -> &str {
        self.inner.vibration_control_title
    }
    pub fn left_motor_label(&self) -> &str {
        self.inner.left_motor_label
    }
    pub fn right_motor_label(&self) -> &str {
        self.inner.right_motor_label
    }
    pub fn power_label(&self) -> &str {
        self.inner.power_label
    }
    pub fn test_vibration_button(&self) -> &str {
        self.inner.test_vibration_button
    }
    pub fn stop_vibration_button(&self) -> &str {
        self.inner.stop_vibration_button
    }
    pub fn deadzone_settings_title(&self) -> &str {
        self.inner.deadzone_settings_title
    }
    pub fn stick_label(&self) -> &str {
        self.inner.stick_label
    }
    pub fn trigger_label_short(&self) -> &str {
        self.inner.trigger_label_short
    }
    pub fn threshold_label(&self) -> &str {
        self.inner.threshold_label
    }
    pub fn device_manager_close_button(&self) -> &str {
        self.inner.device_manager_close_button
    }
    pub fn preferred_api_label(&self) -> &str {
        self.inner.preferred_api_label
    }
    pub fn api_auto(&self) -> &str {
        self.inner.api_auto
    }
    pub fn api_xinput(&self) -> &str {
        self.inner.api_xinput
    }
    pub fn api_rawinput(&self) -> &str {
        self.inner.api_rawinput
    }
    pub fn reactivate_button(&self) -> &str {
        self.inner.reactivate_button
    }
    pub fn all_devices_filter(&self) -> &str {
        self.inner.all_devices_filter
    }
    pub fn game_devices_only_filter(&self) -> &str {
        self.inner.game_devices_only_filter
    }
    pub fn no_game_devices_detected(&self) -> &str {
        self.inner.no_game_devices_detected
    }

    // Additional main window status card
    pub fn running_status(&self) -> &str {
        self.inner.running_status
    }
    pub fn pause_button(&self) -> &str {
        self.inner.pause_button
    }
    pub fn start_button(&self) -> &str {
        self.inner.start_button
    }
    pub fn exit_button(&self) -> &str {
        self.inner.exit_button
    }

    // Main window config display
    pub fn input_timeout_display(&self) -> &str {
        self.inner.input_timeout_display
    }
    pub fn default_interval_display(&self) -> &str {
        self.inner.default_interval_display
    }
    pub fn default_duration_display(&self) -> &str {
        self.inner.default_duration_display
    }
    pub fn show_tray_icon_display(&self) -> &str {
        self.inner.show_tray_icon_display
    }
    pub fn show_notifications_display(&self) -> &str {
        self.inner.show_notifications_display
    }
    pub fn always_on_top_display(&self) -> &str {
        self.inner.always_on_top_display
    }
    pub fn yes(&self) -> &str {
        self.inner.yes
    }
    pub fn no(&self) -> &str {
        self.inner.no
    }

    // Additional settings dialog fields
    pub fn worker_count_label(&self) -> &str {
        self.inner.worker_count_label
    }
    pub fn trigger_short(&self) -> &str {
        self.inner.trigger_short
    }
    pub fn target_short(&self) -> &str {
        self.inner.target_short
    }
    pub fn interval_short(&self) -> &str {
        self.inner.interval_short
    }
    pub fn duration_short(&self) -> &str {
        self.inner.duration_short
    }
    pub fn add_new_mapping_title(&self) -> &str {
        self.inner.add_new_mapping_title
    }
    pub fn add_button_text(&self) -> &str {
        self.inner.add_button_text
    }
    pub fn process_whitelist_hint(&self) -> &str {
        self.inner.process_whitelist_hint
    }
    pub fn process_example(&self) -> &str {
        self.inner.process_example
    }
    pub fn browse_button(&self) -> &str {
        self.inner.browse_button
    }
    pub fn changes_take_effect_hint(&self) -> &str {
        self.inner.changes_take_effect_hint
    }

    // Dynamic worker count formatting (for runtime values)
    pub fn format_worker_count(&self, count: usize) -> String {
        format!("{} {}", self.inner.worker_count_label, count)
    }

    // Tray Icon
    pub fn tray_activate(&self) -> &str {
        self.inner.tray_activate
    }
    pub fn tray_pause(&self) -> &str {
        self.inner.tray_pause
    }
    pub fn tray_show_window(&self) -> &str {
        self.inner.tray_show_window
    }
    pub fn tray_about(&self) -> &str {
        self.inner.tray_about
    }
    pub fn tray_exit(&self) -> &str {
        self.inner.tray_exit
    }
    pub fn tray_notification_launched(&self) -> &str {
        self.inner.tray_notification_launched
    }
    pub fn tray_notification_activated(&self) -> &str {
        self.inner.tray_notification_activated
    }
    pub fn tray_notification_paused(&self) -> &str {
        self.inner.tray_notification_paused
    }

    // UI Icons
    pub fn sequence_icon(&self) -> &str {
        self.inner.sequence_icon
    }
    pub fn target_icon(&self) -> &str {
        self.inner.target_icon
    }
    pub fn arrow_icon(&self) -> &str {
        self.inner.arrow_icon
    }
    pub fn delete_icon(&self) -> &str {
        self.inner.delete_icon
    }
    pub fn keys_text(&self) -> &str {
        self.inner.keys_text
    }
    pub fn targets_text(&self) -> &str {
        self.inner.targets_text
    }

    pub fn rule_props_button(&self) -> &str {
        self.inner.rule_props_button
    }
    pub fn rule_props_dialog_title(&self) -> &str {
        self.inner.rule_props_dialog_title
    }
    pub fn rule_props_hint(&self) -> &str {
        self.inner.rule_props_hint
    }
    pub fn rule_props_hold_column(&self) -> &str {
        self.inner.rule_props_hold_column
    }
    pub fn rule_props_append_label(&self) -> &str {
        self.inner.rule_props_append_label
    }
    pub fn rule_props_add_append(&self) -> &str {
        self.inner.rule_props_add_append
    }
    pub fn rule_props_append_placeholder(&self) -> &str {
        self.inner.rule_props_append_placeholder
    }
    pub fn rule_props_save(&self) -> &str {
        self.inner.rule_props_save
    }
    pub fn rule_props_cancel(&self) -> &str {
        self.inner.rule_props_cancel
    }

    /// Format keys count with localized text
    /// Optimized to minimize allocations with pre-sized capacity
    #[inline]
    pub fn format_keys_count(&self, count: usize) -> String {
        let count_str = count.to_string();
        let mut s = String::with_capacity(count_str.len() + 1 + self.keys_text().len());
        s.push_str(&count_str);
        s.push(' ');
        s.push_str(self.keys_text());
        s
    }

    /// Format targets count with localized text
    /// Optimized to minimize allocations with pre-sized capacity
    #[inline]
    pub fn format_targets_count(&self, count: usize) -> String {
        let count_str = count.to_string();
        let mut s = String::with_capacity(count_str.len() + 1 + self.targets_text().len());
        s.push_str(&count_str);
        s.push(' ');
        s.push_str(self.targets_text());
        s
    }
}
