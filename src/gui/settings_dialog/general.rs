//! Settings dialog "General" section. Renders the hotkeys card and the
//! global configuration card. Implemented as a free function so the
//! caller can split-borrow `SorahkGui` fields disjointly with the parent
//! scroll-area closure.

use super::helpers::get_capture_mode_display_name;
use crate::config::AppConfig;
use crate::gui::types::KeyCaptureMode;
use crate::i18n::CachedTranslations;
use crate::state::{AppState, CaptureMode};
use eframe::egui;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

/// Renders the General settings section, hotkeys and global config.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_general_section(
    ui: &mut egui::Ui,
    temp_config: &mut AppConfig,
    key_capture_mode: &mut KeyCaptureMode,
    capture_initial_pressed: &mut HashSet<u32>,
    capture_pressed_keys: &mut HashSet<u32>,
    just_captured_input: &mut bool,
    app_state: &Arc<AppState>,
    dark_mode: bool,
    translations: CachedTranslations,
) {
    let t = translations;

    // Toggle Key Section
    let card_bg = if dark_mode {
        egui::Color32::from_rgb(45, 47, 58)
    } else {
        egui::Color32::from_rgb(255, 250, 255)
    };

    egui::Frame::NONE
        .fill(card_bg)
        .corner_radius(egui::CornerRadius::same(15))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(t.hotkeys_section_title())
                    .size(16.0)
                    .strong()
                    .color(if dark_mode {
                        egui::Color32::from_rgb(200, 180, 255)
                    } else {
                        egui::Color32::from_rgb(100, 120, 200)
                    }),
            );
            ui.add_space(6.0);

            // Two-column grid keeps capture buttons aligned across rows
            // regardless of per-language label width.
            let available = ui.available_width();
            egui::Grid::new("hotkeys_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .min_col_width(available * 0.35)
                .show(ui, |ui| {
                    // Toggle hotkey row
                    ui.label(t.toggle_key_label());
                    let is_capturing = (*key_capture_mode)
                        == KeyCaptureMode::ToggleKey;
                    let button_text = if is_capturing {
                        t.press_any_key()
                    } else if temp_config.switch_key.is_empty() {
                        t.click_to_set()
                    } else {
                        &temp_config.switch_key
                    };
                    let button = egui::Button::new(
                        egui::RichText::new(button_text).color(
                            if is_capturing {
                                egui::Color32::from_rgb(255, 200, 0)
                            } else if dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ),
                    )
                    .fill(if is_capturing {
                        egui::Color32::from_rgb(70, 130, 180)
                    } else if dark_mode {
                        egui::Color32::from_rgb(60, 60, 60)
                    } else {
                        egui::Color32::from_rgb(220, 220, 220)
                    })
                    .corner_radius(10.0);
                    if ui.add_sized([180.0, 28.0], button).clicked()
                        && !(*just_captured_input)
                    {
                        *key_capture_mode =
                            KeyCaptureMode::ToggleKey;
                        capture_pressed_keys.clear();
                        *capture_initial_pressed =
                            crate::gui::SorahkGui::poll_all_pressed_keys();
                        app_state
                            .set_raw_input_capture_mode(true);
                        *just_captured_input = true;
                    }
                    ui.end_row();

                    // Sequence-finalize hotkey row
                    ui.label(t.sequence_finalize_row_label());
                    let is_capturing = (*key_capture_mode)
                        == KeyCaptureMode::SequenceFinalizeKey;
                    let button_text = if is_capturing {
                        t.press_any_key()
                    } else if temp_config
                        .sequence_finalize_key
                        .is_empty()
                    {
                        t.click_to_set()
                    } else {
                        &temp_config.sequence_finalize_key
                    };
                    let button = egui::Button::new(
                        egui::RichText::new(button_text).color(
                            if is_capturing {
                                egui::Color32::from_rgb(255, 200, 0)
                            } else if dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ),
                    )
                    .fill(if is_capturing {
                        egui::Color32::from_rgb(70, 130, 180)
                    } else if dark_mode {
                        egui::Color32::from_rgb(60, 60, 60)
                    } else {
                        egui::Color32::from_rgb(220, 220, 220)
                    })
                    .corner_radius(10.0);
                    if ui.add_sized([180.0, 28.0], button).clicked()
                        && !(*just_captured_input)
                    {
                        *key_capture_mode =
                            KeyCaptureMode::SequenceFinalizeKey;
                        capture_pressed_keys.clear();
                        *capture_initial_pressed =
                            crate::gui::SorahkGui::poll_all_pressed_keys();
                        app_state
                            .set_raw_input_capture_mode(false);
                        *just_captured_input = true;
                    }
                    ui.end_row();
                });

            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(t.sequence_finalize_hint())
                    .size(11.0)
                    .italics()
                    .color(if dark_mode {
                        egui::Color32::from_rgb(170, 170, 185)
                    } else {
                        egui::Color32::from_rgb(120, 120, 140)
                    }),
            );
        });

    ui.add_space(8.0);

    // Global Configuration Section
    let card_bg = if dark_mode {
        egui::Color32::from_rgb(40, 40, 50)
    } else {
        egui::Color32::from_rgb(250, 240, 255)
    };

    egui::Frame::NONE
        .fill(card_bg)
        .corner_radius(egui::CornerRadius::same(15))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(t.global_config_title())
                    .size(16.0)
                    .strong()
                    .color(if dark_mode {
                        egui::Color32::from_rgb(200, 180, 255)
                    } else {
                        egui::Color32::from_rgb(200, 120, 80)
                    }),
            );
            ui.add_space(6.0);

            let available = ui.available_width();
            egui::Grid::new("config_edit_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .min_col_width(available * 0.35)
                .show(ui, |ui| {
                    // Language
                    ui.label(t.language());
                    egui::ComboBox::from_id_salt(
                        "language_selector",
                    )
                    .selected_text(
                        temp_config.language.display_name(),
                    )
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        use crate::i18n::Language;
                        for lang in Language::all() {
                            ui.selectable_value(
                                &mut temp_config.language,
                                *lang,
                                lang.display_name(),
                            );
                        }
                    });
                    ui.end_row();

                    // Raw Input Capture Mode selector
                    ui.label(t.rawinput_capture_mode_label());
                    let current_mode_str = &temp_config.rawinput_capture_mode;
                    let current_mode = CaptureMode::from_str(current_mode_str).unwrap();
                    let current_mode_name = get_capture_mode_display_name(&t, current_mode);
                    egui::ComboBox::from_id_salt("rawinput_capture_mode")
                        .selected_text(current_mode_name)
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for &mode in CaptureMode::all_modes() {
                                let mode_name = get_capture_mode_display_name(&t, mode);
                                let is_selected = temp_config.rawinput_capture_mode == mode.as_str();
                                if ui.selectable_label(is_selected, mode_name).clicked() {
                                    temp_config.rawinput_capture_mode = mode.as_str().to_string();
                                }
                            }
                        });
                    ui.end_row();

                    // XInput Capture Mode selector
                    ui.label(t.xinput_capture_mode_label());
                    let current_mode_str = &temp_config.xinput_capture_mode;
                    let current_mode = crate::config::XInputCaptureMode::from_str(current_mode_str).unwrap();
                    let current_mode_name = match current_mode {
                        crate::config::XInputCaptureMode::MostSustained => t.capture_mode_most_sustained(),
                        crate::config::XInputCaptureMode::LastStable => t.capture_mode_last_stable(),
                        crate::config::XInputCaptureMode::DiagonalPriority => t.capture_mode_diagonal_priority(),
                    };
                    egui::ComboBox::from_id_salt("xinput_capture_mode")
                        .selected_text(current_mode_name)
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for &mode in crate::config::XInputCaptureMode::all_modes() {
                                let mode_name = match mode {
                                    crate::config::XInputCaptureMode::MostSustained => t.capture_mode_most_sustained(),
                                    crate::config::XInputCaptureMode::LastStable => t.capture_mode_last_stable(),
                                    crate::config::XInputCaptureMode::DiagonalPriority => t.capture_mode_diagonal_priority(),
                                };
                                let is_selected = temp_config.xinput_capture_mode == mode.as_str();
                                if ui.selectable_label(is_selected, mode_name).clicked() {
                                    temp_config.xinput_capture_mode = mode.as_str().to_string();
                                }
                            }
                        });
                    ui.end_row();

                    ui.label(t.input_timeout_label());
                    let mut timeout_str =
                        temp_config.input_timeout.to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(
                            &mut timeout_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(50, 50, 50)
                        } else {
                            egui::Color32::from_rgb(220, 220, 220)
                        }),
                    );
                    if let Ok(val) = timeout_str.parse::<u64>() {
                        temp_config.input_timeout = val;
                    }
                    ui.end_row();

                    ui.label(t.default_interval_label());
                    let mut interval_str =
                        temp_config.interval.to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(
                            &mut interval_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(50, 50, 50)
                        } else {
                            egui::Color32::from_rgb(220, 220, 220)
                        }),
                    );
                    if let Ok(val) = interval_str.parse::<u64>() {
                        temp_config.interval = val.max(5);
                    }
                    ui.end_row();

                    ui.label(t.default_duration_label());
                    let mut duration_str =
                        temp_config.event_duration.to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(
                            &mut duration_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(50, 50, 50)
                        } else {
                            egui::Color32::from_rgb(220, 220, 220)
                        }),
                    );
                    if let Ok(val) = duration_str.parse::<u64>() {
                        temp_config.event_duration = val.max(2);
                    }
                    ui.end_row();

                    ui.label(t.mouse_move_per_event_min_label())
                        .on_hover_text(
                            t.mouse_move_per_event_min_hint(),
                        );
                    let mut per_event_str = temp_config
                        .mouse_move_per_event_min_px
                        .to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(
                            &mut per_event_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(50, 50, 50)
                        } else {
                            egui::Color32::from_rgb(220, 220, 220)
                        }),
                    );
                    if let Ok(val) = per_event_str.parse::<u32>() {
                        temp_config.mouse_move_per_event_min_px =
                            val.max(1);
                    }
                    ui.end_row();

                    ui.label(t.mouse_move_min_trigger_label())
                        .on_hover_text(
                            t.mouse_move_min_trigger_hint(),
                        );
                    let mut min_trigger_str = temp_config
                        .mouse_move_min_trigger_px
                        .to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(
                            &mut min_trigger_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(50, 50, 50)
                        } else {
                            egui::Color32::from_rgb(220, 220, 220)
                        }),
                    );
                    if let Ok(val) =
                        min_trigger_str.parse::<u32>()
                    {
                        temp_config.mouse_move_min_trigger_px =
                            val.max(1);
                    }
                    ui.end_row();

                    ui.label(t.mouse_move_rearm_label())
                        .on_hover_text(t.mouse_move_rearm_hint());
                    let mut rearm_str =
                        temp_config.mouse_move_rearm_px.to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(&mut rearm_str)
                            .background_color(if dark_mode {
                                egui::Color32::from_rgb(50, 50, 50)
                            } else {
                                egui::Color32::from_rgb(
                                    220, 220, 220,
                                )
                            }),
                    );
                    if let Ok(val) = rearm_str.parse::<u32>() {
                        temp_config.mouse_move_rearm_px =
                            val.max(1);
                    }
                    ui.end_row();

                    ui.label(t.worker_count_label());
                    let mut worker_str =
                        temp_config.worker_count.to_string();
                    ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(&mut worker_str)
                            .hint_text("0 = auto")
                            .background_color(if dark_mode {
                                egui::Color32::from_rgb(50, 50, 50)
                            } else {
                                egui::Color32::from_rgb(
                                    220, 220, 220,
                                )
                            }),
                    );
                    if let Ok(val) = worker_str.parse::<usize>() {
                        temp_config.worker_count = val;
                    }
                    ui.end_row();

                    ui.label(t.show_tray_icon());
                    ui.checkbox(
                        &mut temp_config.show_tray_icon,
                        "",
                    );
                    ui.end_row();

                    ui.label(t.show_notifications());
                    ui.checkbox(
                        &mut temp_config.show_notifications,
                        "",
                    );
                    ui.end_row();

                    ui.label(t.always_on_top());
                    ui.checkbox(&mut temp_config.always_on_top, "");
                    ui.end_row();

                    ui.label(t.dark_mode());
                    ui.checkbox(&mut temp_config.dark_mode, "");
                    ui.end_row();
                });
        });
}
