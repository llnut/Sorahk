//! Settings dialog "Add new mapping" editor section. Renders the form
//! below the existing-mappings list inside the Key Mappings card.
//! Implemented as a free function so the caller can split-borrow
//! `SorahkGui` fields disjointly with the parent scroll-area closure.

use super::helpers::{BUTTON_TEXT_MAX_CHARS, truncate_text_safe};
use crate::config::{AppConfig, KeyMapping};
use crate::gui::mouse_direction_dialog::MouseDirectionDialog;
use crate::gui::mouse_scroll_dialog::MouseScrollDialog;
use crate::gui::rule_properties_dialog::RulePropertiesDialog;
use crate::gui::theme;
use crate::gui::types::KeyCaptureMode;
use crate::gui::utils::{is_mouse_move_target, is_mouse_scroll_target};
use crate::gui::widgets;
use crate::i18n::CachedTranslations;
use crate::state::AppState;
use eframe::egui;
use std::collections::HashSet;
use std::sync::Arc;

/// Renders the "Add New Mapping" form inside the Key Mappings card.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_mapping_editor_section(
    ui: &mut egui::Ui,
    temp_config: &mut AppConfig,
    key_capture_mode: &mut KeyCaptureMode,
    capture_initial_pressed: &mut HashSet<u32>,
    capture_pressed_keys: &mut HashSet<u32>,
    just_captured_input: &mut bool,
    duplicate_mapping_error: &mut Option<String>,
    sequence_last_mouse_pos: &mut Option<egui::Pos2>,
    sequence_last_mouse_direction: &mut Option<String>,
    sequence_mouse_delta: &mut egui::Vec2,
    sequence_capture_list: &mut Vec<String>,
    target_sequence_capture_list: &mut Vec<String>,
    mouse_direction_dialog: &mut Option<MouseDirectionDialog>,
    mouse_direction_mapping_idx: &mut Option<usize>,
    mouse_scroll_dialog: &mut Option<MouseScrollDialog>,
    mouse_scroll_mapping_idx: &mut Option<usize>,
    rule_properties_dialog: &mut Option<RulePropertiesDialog>,
    rule_props_editing_idx: &mut Option<usize>,
    new_mapping_trigger: &mut String,
    new_mapping_target: &mut String,
    new_mapping_target_keys: &mut Vec<String>,
    new_mapping_target_mode: &mut u8,
    new_mapping_interval: &mut String,
    new_mapping_duration: &mut String,
    new_mapping_move_speed: &mut String,
    new_mapping_sequence_window: &mut String,
    new_mapping_is_sequence_mode: &mut bool,
    new_mapping_turbo: &mut bool,
    new_mapping_hold_indices: &mut Vec<u8>,
    new_mapping_append_keys: &mut Vec<String>,
    app_state: &Arc<AppState>,
    dark_mode: bool,
    translations: CachedTranslations,
) {
    let t = translations;
    // Add new mapping section with card layout
    ui.label(
        egui::RichText::new(t.add_new_mapping_title())
            .size(15.0)
            .strong()
            .color(if dark_mode {
                egui::Color32::from_rgb(255, 182, 193)
            } else {
                egui::Color32::from_rgb(255, 105, 180)
            }),
    );
    ui.add_space(10.0);

    let new_mapping_card_bg = if dark_mode {
        egui::Color32::from_rgb(50, 52, 62)
    } else {
        egui::Color32::from_rgb(255, 250, 255)
    };

    egui::Frame::NONE
        .fill(new_mapping_card_bg)
        .corner_radius(egui::CornerRadius::same(16))
        .inner_margin(egui::Margin::same(14))
        .stroke(if dark_mode {
            egui::Stroke::NONE
        } else {
            egui::Stroke::new(
                1.5,
                egui::Color32::from_rgba_premultiplied(147, 197, 253, 80)
            )
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            // Trigger mode selector. Sits above the trigger input row.
            let is_sequence_mode = *new_mapping_is_sequence_mode;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t.trigger_mode_label())
                        .size(13.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(255, 182, 193)
                        } else {
                            egui::Color32::from_rgb(255, 105, 180)
                        }),
                );
                ui.add_space(8.0);
                // Single Key Button - matches Single badge colors
                let single_btn = egui::Button::new(
                    egui::RichText::new(t.trigger_mode_single())
                        .size(12.0)
                        .color(if !is_sequence_mode {
                            if dark_mode {
                                egui::Color32::from_rgb(20, 60, 80)
                            } else {
                                egui::Color32::from_rgb(40, 80, 120)
                            }
                        } else if dark_mode {
                            egui::Color32::from_rgb(180, 180, 200)
                        } else {
                            egui::Color32::from_rgb(100, 100, 120)
                        }),
                )
                .fill(if !is_sequence_mode {
                    if dark_mode {
                        egui::Color32::from_rgb(135, 206, 235)
                    } else {
                        egui::Color32::from_rgb(173, 216, 230)
                    }
                } else if dark_mode {
                    egui::Color32::from_rgb(50, 52, 62)
                } else {
                    egui::Color32::from_rgb(240, 240, 245)
                })
                .corner_radius(10.0);
                if ui.add(single_btn).clicked() && is_sequence_mode {
                    // Switch to single key mode - clear sequence
                    *new_mapping_is_sequence_mode = false;
                    sequence_capture_list.clear();
                    *sequence_last_mouse_pos = None;
                    *sequence_last_mouse_direction = None;
                    *sequence_mouse_delta = egui::Vec2::ZERO;
                    if (*new_mapping_trigger).contains(',') {
                        (*new_mapping_trigger).clear();
                    }
                }
                ui.add_space(6.0);
                // Sequence Button - matches Sequence badge colors
                let seq_btn = egui::Button::new(
                    egui::RichText::new(t.trigger_mode_sequence())
                        .size(12.0)
                        .color(if is_sequence_mode {
                            if dark_mode {
                                egui::Color32::from_rgb(80, 20, 40)
                            } else {
                                egui::Color32::from_rgb(220, 80, 120)
                            }
                        } else if dark_mode {
                            egui::Color32::from_rgb(180, 180, 200)
                        } else {
                            egui::Color32::from_rgb(100, 100, 120)
                        }),
                )
                .fill(if is_sequence_mode {
                    if dark_mode {
                        egui::Color32::from_rgb(255, 182, 193)
                    } else {
                        egui::Color32::from_rgb(255, 218, 224)
                    }
                } else if dark_mode {
                    egui::Color32::from_rgb(50, 52, 62)
                } else {
                    egui::Color32::from_rgb(240, 240, 245)
                })
                .corner_radius(10.0);
                if ui.add(seq_btn).clicked() && !is_sequence_mode {
                    // Switch to sequence mode - clear single trigger
                    *new_mapping_is_sequence_mode = true;
                    (*new_mapping_trigger).clear();
                }
            });
            ui.add_space(8.0);
            // Inline hint shown only in sequence mode.
            if is_sequence_mode {
                egui::Frame::NONE
                    .fill(if dark_mode {
                        egui::Color32::from_rgba_premultiplied(255, 182, 193, 30)
                    } else {
                        egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                    })
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(t.sequence_trigger_explanation())
                                .size(11.0)
                                .italics()
                                .color(if dark_mode {
                                    egui::Color32::from_rgb(255, 200, 210)
                                } else {
                                    egui::Color32::from_rgb(220, 80, 120)
                                }),
                        );
                    });
                ui.add_space(8.0);
                // Time window setting for sequence
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t.sequence_window_label())
                            .size(12.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(200, 200, 220)
                            } else {
                                egui::Color32::from_rgb(80, 80, 100)
                            }),
                    );
                    let window_edit = egui::TextEdit::singleline(
                        &mut *new_mapping_sequence_window,
                    )
                    .background_color(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(240, 240, 245)
                    })
                    .hint_text("300")
                    .desired_width(70.0)
                    .font(egui::TextStyle::Button);
                    ui.add_sized([70.0, 26.0], window_edit);
                    let hint = t.sequence_window_hint();
                    let mut hint_text = String::with_capacity(3 + hint.len() + 3);
                    hint_text.push('(');
                    hint_text.push_str(hint);
                    hint_text.push(')');
                    hint_text.push(' ');
                    hint_text.push_str(t.sequence_icon());
                    ui.label(
                        egui::RichText::new(&hint_text)
                            .size(11.0)
                            .italics()
                            .color(if dark_mode {
                                egui::Color32::from_rgb(150, 150, 170)
                            } else {
                                egui::Color32::from_rgb(120, 120, 140)
                            }),
                    );
                });
                ui.add_space(8.0);
            }
            ui.add_space(4.0);

            // Trigger section
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t.trigger_short())
                        .size(13.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(200, 200, 220)
                        } else {
                            egui::Color32::from_rgb(80, 80, 100)
                        }),
                );
            });
            ui.add_space(6.0);
            if !is_sequence_mode {
                // Single key trigger mode
                let is_capturing_new_trigger = *key_capture_mode
                    == KeyCaptureMode::NewMappingTrigger;
                let new_trigger_display = if is_capturing_new_trigger {
                    t.press_any_key().to_string()
                } else if (*new_mapping_trigger).is_empty() {
                    t.click_to_set_trigger().to_string()
                } else {
                    truncate_text_safe(&*new_mapping_trigger, BUTTON_TEXT_MAX_CHARS)
                };

                let new_trigger_btn = egui::Button::new(
                    egui::RichText::new(&new_trigger_display)
                        .size(14.0)
                        .color(
                            if is_capturing_new_trigger {
                                egui::Color32::from_rgb(255, 215, 0)
                            } else if dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ),
                )
                .fill(if is_capturing_new_trigger {
                    egui::Color32::from_rgb(70, 130, 180)
                } else if dark_mode {
                    egui::Color32::from_rgb(60, 62, 72)
                } else {
                    egui::Color32::from_rgb(245, 245, 250)
                })
                .corner_radius(10.0);

                let mut new_trigger_response = ui
                    .add_sized([ui.available_width(), 30.0], new_trigger_btn);
                // Show full text on hover if truncated
                if !is_capturing_new_trigger && !(*new_mapping_trigger).is_empty() && (*new_mapping_trigger).chars().count() > BUTTON_TEXT_MAX_CHARS {
                    new_trigger_response = new_trigger_response.on_hover_text(&*new_mapping_trigger);
                }
                if new_trigger_response.clicked() && !*just_captured_input {
                    *key_capture_mode =
                        KeyCaptureMode::NewMappingTrigger;
                    capture_pressed_keys.clear();
                    *capture_initial_pressed =
                        crate::gui::SorahkGui::poll_all_pressed_keys();
                    app_state.set_raw_input_capture_mode(true);
                    *just_captured_input = true;
                    *duplicate_mapping_error = None;
                }
                // Display full trigger text with wrapping for long device names
                if !is_capturing_new_trigger && !(*new_mapping_trigger).is_empty() && (*new_mapping_trigger).chars().count() > BUTTON_TEXT_MAX_CHARS {
                    ui.add_space(6.0);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .fill(if dark_mode {
                            egui::Color32::from_rgba_unmultiplied(60, 62, 72, 100)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(240, 240, 245, 150)
                        })
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.set_max_width(ui.available_width());
                            ui.add(egui::Label::new(
                                egui::RichText::new(&*new_mapping_trigger)
                                    .size(11.0)
                                    .color(if dark_mode {
                                        egui::Color32::from_rgb(180, 180, 200)
                                    } else {
                                        egui::Color32::from_rgb(80, 80, 100)
                                    })
                            ).wrap());
                        });
                }
            } else {
                // Sequence trigger mode - display captured sequence
                let is_capturing_sequence = *key_capture_mode == KeyCaptureMode::NewMappingTrigger;
                if is_capturing_sequence {
                    // Currently capturing - show animated prompt
                    egui::Frame::NONE
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(255, 200, 130)
                        } else {
                            egui::Color32::from_rgb(255, 235, 180)
                        })
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(t.sequence_capturing())
                                        .size(14.0)
                                        .strong()
                                        .color(if dark_mode {
                                            egui::Color32::from_rgb(80, 60, 20)
                                        } else {
                                            egui::Color32::from_rgb(100, 80, 20)
                                        }),
                                );
                                ui.add_space(4.0);
                                if !sequence_capture_list.is_empty() {
                                    let count = sequence_capture_list.len();
                                    let last_key = sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                    let preview = if count <= 3 {
                                        sequence_capture_list.join(" → ")
                                    } else {
                                        format!("{} keys: ... → {}", count, last_key)
                                    };
                                    ui.label(
                                        egui::RichText::new(&preview)
                                            .size(13.0)
                                            .color(if dark_mode {
                                                egui::Color32::from_rgb(60, 40, 10)
                                            } else {
                                                egui::Color32::from_rgb(80, 60, 20)
                                            }),
                                    );
                                }
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(t.sequence_capture_hint())
                                        .size(11.0)
                                        .italics()
                                        .color(if dark_mode {
                                            egui::Color32::from_rgb(100, 80, 40)
                                        } else {
                                            egui::Color32::from_rgb(120, 100, 60)
                                        }),
                                );
                            });
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                            let done_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_complete())
                                .size(13.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(120, 220, 140)
                        } else {
                            egui::Color32::from_rgb(140, 230, 150)
                        })
                        .corner_radius(10.0);
                        if ui.add_sized([120.0, 28.0], done_btn).clicked() {
                            // Finish sequence capture
                            if !sequence_capture_list.is_empty() {
                                *new_mapping_trigger = sequence_capture_list.join(",");
                            }
                            {
                                *key_capture_mode = KeyCaptureMode::None;
                                app_state.set_raw_input_capture_mode(false);
                                capture_pressed_keys.clear();
                                *just_captured_input = true;
                                *sequence_last_mouse_pos = None;
                                *sequence_last_mouse_direction = None;
                                *sequence_mouse_delta = egui::Vec2::ZERO;
                            }
                        }
                        ui.add_space(8.0);
                            let clear_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_clear_btn())
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(230, 100, 100)
                        } else {
                            egui::Color32::from_rgb(250, 150, 150)
                        })
                        .corner_radius(10.0);
                        if ui.add_sized([100.0, 28.0], clear_btn).clicked() {
                            sequence_capture_list.clear();
                            *sequence_last_mouse_pos = None;
                            *sequence_last_mouse_direction = None;
                            *sequence_mouse_delta = egui::Vec2::ZERO;
                        }
                    });
                } else {
                    let seq_display = if sequence_capture_list.is_empty() {
                        let label = t.sequence_example_label();
                        let icon = t.sequence_icon();
                        let mut s = String::with_capacity(icon.len() + 1 + label.len() + 1 + icon.len());
                        s.push_str(icon);
                        s.push(' ');
                        s.push_str(label);
                        s.push(' ');
                        s.push_str(icon);
                        s
                    } else {
                        // Show simplified text for button: count + last few keys
                        let count = sequence_capture_list.len();
                        if count <= 3 {
                            let arrow = t.arrow_icon();
                            sequence_capture_list.join(&[' ', arrow.chars().next().unwrap_or('→'), ' '].iter().collect::<String>())
                        } else {
                            let last = sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                            format!("{} keys: ... → {}", count, last)
                        }
                    };
                    let seq_btn = egui::Button::new(
                        egui::RichText::new(&seq_display)
                            .size(14.0)
                            .color(if dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            }),
                    )
                    .fill(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(245, 245, 250)
                    })
                    .corner_radius(10.0);
                    if ui.add_sized([ui.available_width(), 30.0], seq_btn).clicked() && !*just_captured_input {
                        // Start sequence capture
                        *key_capture_mode = KeyCaptureMode::NewMappingTrigger;
                        sequence_capture_list.clear();
                        *sequence_last_mouse_pos = None;
                        *sequence_last_mouse_direction = None;
                        *sequence_mouse_delta = egui::Vec2::ZERO;
                        capture_pressed_keys.clear();
                        *capture_initial_pressed = crate::gui::SorahkGui::poll_all_pressed_keys();
                        app_state.set_raw_input_capture_mode(true);
                        *just_captured_input = true;
                        *duplicate_mapping_error = None;
                    }
                }
                // Display captured sequence keys with a horizontal flow layout.
                if !sequence_capture_list.is_empty() {
                    ui.add_space(8.0);
                    // Get full available width before Frame consumes it
                    let full_width = ui.available_width();
                    let inner_margin = 10.0;
                    egui::Frame::NONE
                        .fill(if dark_mode {
                            egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                        } else {
                            egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                        })
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(inner_margin as i8, inner_margin as i8))
                        .show(ui, |ui| {
                            // Subtract the symmetric inner margin from the outer width.
                            let content_width = full_width - inner_margin * 2.0;
                            ui.set_min_width(content_width);
                            ui.set_max_width(content_width);
                            // Header
                            ui.horizontal(|ui| {
                                let t = &translations;
                                ui.label(
                                    egui::RichText::new(t.sequence_icon())
                                        .size(12.0)
                                );
                                ui.label(
                                    egui::RichText::new(t.format_keys_count(sequence_capture_list.len()))
                                        .size(10.0)
                                        .italics()
                                        .color(if dark_mode {
                                            egui::Color32::from_rgb(200, 150, 160)
                                        } else {
                                            egui::Color32::from_rgb(180, 100, 120)
                                        })
                                );
                            });
                            ui.add_space(6.0);
                            // Manual flow layout to avoid horizontal_wrapped staircase bug
                            // Use content_width for pill layout calculation
                            let layout_width = content_width;
                            egui::ScrollArea::vertical()
                                .id_salt(0xDEADBEEFu64)
                                .max_height(120.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(layout_width);
                                    let mut keys_to_remove = Vec::new();
                                    let seq_keys: Vec<_> = sequence_capture_list.to_vec();
                                    let available_width = layout_width;

                                    // Pre-calculate rows to avoid staircase effect
                                    let mut rows: Vec<Vec<usize>> = Vec::new();
                                    let mut current_row: Vec<usize> = Vec::new();
                                    let mut current_width = 0.0f32;

                                    for (key_idx, key) in seq_keys.iter().enumerate() {
                                        let pill_width = widgets::estimate_pill_width_editor(key);
                                        let arrow_width = if key_idx < seq_keys.len() - 1 { widgets::arrow_separator_width() } else { 0.0 };
                                        let total_width = pill_width + arrow_width;

                                        if current_width + total_width > available_width && !current_row.is_empty() {
                                            rows.push(std::mem::take(&mut current_row));
                                            current_width = 0.0;
                                        }
                                        current_row.push(key_idx);
                                        current_width += total_width + 4.0; // item spacing
                                    }
                                    if !current_row.is_empty() {
                                        rows.push(current_row);
                                    }

                                    // Render each row
                                    for row in &rows {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                                            for &key_idx in row {
                                                let key = &seq_keys[key_idx];
                                                let (_icon, display_name) = widgets::pill_icon_and_label(key);
                                                let tag_color = widgets::pill_color(key, dark_mode);
                                                let text_color = if dark_mode {
                                                    egui::Color32::from_rgb(40, 30, 50)
                                                } else {
                                                    egui::Color32::from_rgb(60, 40, 70)
                                                };
                                                // Pill-shaped tag, matching the target pill style.
                                                let tag_response = egui::Frame::NONE
                                                    .fill(tag_color)
                                                    .corner_radius(egui::CornerRadius::same(12))
                                                    .inner_margin(egui::Margin::symmetric(8, 4))
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.spacing_mut().item_spacing.x = 3.0;
                                                            // Index badge
                                                            ui.label(
                                                                egui::RichText::new(format!("{}", key_idx + 1))
                                                                    .size(9.0)
                                                                    .strong()
                                                                    .color(text_color)
                                                            );
                                                            // Display name, sized to match the target pill.
                                                            let short_name = if display_name.len() > 15 {
                                                                format!("{}...", &display_name[..12])
                                                            } else {
                                                                display_name
                                                            };
                                                            ui.label(
                                                                egui::RichText::new(&short_name)
                                                                    .size(11.0)
                                                                    .color(text_color)
                                                            );
                                                            // Delete button
                                                            let del_btn = ui.add(
                                                                egui::Button::new(
                                                                    egui::RichText::new("🗑")
                                                                        .size(11.0)
                                                                        .color(if dark_mode {
                                                                            egui::Color32::from_rgb(180, 80, 100)
                                                                        } else {
                                                                            egui::Color32::from_rgb(200, 60, 80)
                                                                        })
                                                                )
                                                                .fill(egui::Color32::TRANSPARENT)
                                                                .frame(false)
                                                                .corner_radius(8.0)
                                                            );
                                                            if del_btn.clicked() {
                                                                keys_to_remove.push(key_idx);
                                                            }
                                                        });
                                                    });
                                                tag_response.response.on_hover_text(key);
                                                // Arrow separator
                                                if key_idx < seq_keys.len() - 1 {
                                                    ui.label(
                                                        egui::RichText::new("→")
                                                            .size(14.0)
                                                            .color(if dark_mode {
                                                                egui::Color32::from_rgb(255, 150, 170)
                                                            } else {
                                                                egui::Color32::from_rgb(255, 120, 150)
                                                            })
                                                    );
                                                }
                                            }
                                        });
                                        ui.add_space(6.0);
                                    }
                                    // Apply deletions
                                    for &idx in keys_to_remove.iter().rev() {
                                        sequence_capture_list.remove(idx);
                                    }
                                    if sequence_capture_list.is_empty() {
                                        (*new_mapping_trigger).clear();
                                    }
                                });
                        });
                }
            }

            ui.add_space(12.0);

            // Target mode selector. 0 = Single, 1 = Multi, 2 = Sequence.
            let target_mode = *new_mapping_target_mode;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t.target_mode_label())
                        .size(13.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(135, 206, 235)
                        } else {
                            egui::Color32::from_rgb(70, 130, 180)
                        }),
                );
                ui.add_space(8.0);

                // Helper for inactive button styling
                let inactive_fill = if dark_mode {
                    egui::Color32::from_rgb(50, 52, 62)
                } else {
                    egui::Color32::from_rgb(240, 240, 245)
                };
                let inactive_text = if dark_mode {
                    egui::Color32::from_rgb(180, 180, 200)
                } else {
                    egui::Color32::from_rgb(100, 100, 120)
                };

                // Single Button - matches Single badge colors
                let single_btn = egui::Button::new(
                    egui::RichText::new(t.target_mode_single())
                        .size(11.0)
                        .color(if target_mode == 0 {
                            if dark_mode {
                                egui::Color32::from_rgb(20, 60, 80)
                            } else {
                                egui::Color32::from_rgb(40, 80, 120)
                            }
                        } else {
                            inactive_text
                        })
                ).fill(if target_mode == 0 {
                    if dark_mode {
                        egui::Color32::from_rgb(135, 206, 235)
                    } else {
                        egui::Color32::from_rgb(173, 216, 230)
                    }
                } else {
                    inactive_fill
                }).corner_radius(10.0);
                if ui.add(single_btn).clicked() && target_mode != 0 {
                    *new_mapping_target_mode = 0;
                    if new_mapping_target_keys.len() > 1 {
                        let first = new_mapping_target_keys[0].clone();
                        new_mapping_target_keys.clear();
                        new_mapping_target_keys.push(first.clone());
                        *new_mapping_target = first;
                    }
                    target_sequence_capture_list.clear();
                }
                ui.add_space(4.0);

                // Multi Button - matches Single badge colors
                let multi_btn = egui::Button::new(
                    egui::RichText::new(t.target_mode_multi())
                        .size(11.0)
                        .color(if target_mode == 1 {
                            if dark_mode {
                                egui::Color32::from_rgb(20, 60, 80)
                            } else {
                                egui::Color32::from_rgb(40, 80, 120)
                            }
                        } else {
                            inactive_text
                        })
                ).fill(if target_mode == 1 {
                    if dark_mode {
                        egui::Color32::from_rgb(135, 206, 235)
                    } else {
                        egui::Color32::from_rgb(173, 216, 230)
                    }
                } else {
                    inactive_fill
                }).corner_radius(10.0);
                if ui.add(multi_btn).clicked() && target_mode != 1 {
                    *new_mapping_target_mode = 1;
                    target_sequence_capture_list.clear();
                }
                ui.add_space(4.0);

                // Sequence Button - matches Sequence badge colors
                let seq_btn = egui::Button::new(
                    egui::RichText::new(t.target_mode_sequence())
                        .size(11.0)
                        .color(if target_mode == 2 {
                            if dark_mode {
                                egui::Color32::from_rgb(80, 20, 40)
                            } else {
                                egui::Color32::from_rgb(220, 80, 120)
                            }
                        } else {
                            inactive_text
                        })
                ).fill(if target_mode == 2 {
                    if dark_mode {
                        egui::Color32::from_rgb(255, 182, 193)
                    } else {
                        egui::Color32::from_rgb(255, 218, 224)
                    }
                } else {
                    inactive_fill
                }).corner_radius(10.0);
                if ui.add(seq_btn).clicked() && target_mode != 2 {
                    *new_mapping_target_mode = 2;
                    // Initialize sequence list from existing keys
                    if !new_mapping_target_keys.is_empty() {
                        *target_sequence_capture_list = new_mapping_target_keys.clone();
                    }
                }

                // Right-aligned Rule Props button. Mirrors the slot used
                // on the edit path for visual consistency.
                ui.with_layout(
                    egui::Layout::right_to_left(
                        egui::Align::Center,
                    ),
                    |ui| {
                        let (props_fill, props_text) =
                            if dark_mode {
                                (
                                    egui::Color32::from_rgb(
                                        255, 140, 170,
                                    ),
                                    egui::Color32::WHITE,
                                )
                            } else {
                                (
                                    egui::Color32::from_rgb(
                                        255, 170, 190,
                                    ),
                                    egui::Color32::WHITE,
                                )
                            };
                        let props_btn = egui::Button::new(
                            egui::RichText::new(
                                t.rule_props_button(),
                            )
                            .size(11.0)
                            .color(props_text),
                        )
                        .fill(props_fill)
                        .corner_radius(10.0);
                        if ui.add(props_btn).clicked() {
                            let keys: Vec<String> =
                                if target_mode == 2
                                    && !target_sequence_capture_list
                                        .is_empty()
                                {
                                    target_sequence_capture_list
                                        .clone()
                                } else {
                                    new_mapping_target_keys
                                        .clone()
                                };
                            let hold = new_mapping_hold_indices
                                .clone();
                            let append = new_mapping_append_keys
                                .clone();
                            *rule_props_editing_idx = None;
                            *rule_properties_dialog = Some(
                                crate::gui::rule_properties_dialog::RulePropertiesDialog::new(
                                    &keys,
                                    &hold,
                                    &append,
                                ),
                            );
                        }
                    },
                );
            });

            // Mode explanation
            ui.add_space(4.0);
            let explanation = match target_mode {
                1 => t.target_mode_multi_explanation(),
                2 => t.target_mode_sequence_explanation(),
                _ => "",
            };
            if !explanation.is_empty() {
                ui.label(
                    egui::RichText::new(explanation)
                        .size(11.0)
                        .italics()
                        .color(if dark_mode {
                            egui::Color32::from_rgb(150, 180, 200)
                        } else {
                            egui::Color32::from_rgb(100, 140, 180)
                        }),
                );
                // Show additional hint for sequence target mode
                if target_mode == 2 {
                    ui.label(
                        egui::RichText::new(t.target_sequence_output_hint())
                            .size(10.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(120, 150, 180)
                            } else {
                                egui::Color32::from_rgb(130, 160, 190)
                            }),
                    );
                }
            }
            ui.add_space(8.0);

            // Target section label
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t.target_short())
                        .size(13.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(200, 200, 220)
                        } else {
                            egui::Color32::from_rgb(80, 80, 100)
                        }),
                );
            });
            ui.add_space(6.0);

            // Determine which list to use based on mode
            let is_capturing_new_target = *key_capture_mode == KeyCaptureMode::NewMappingTarget;
            let display_keys = if target_mode == 2 {
                &target_sequence_capture_list
            } else {
                &new_mapping_target_keys
            };
            let separator = if target_mode == 2 { " → " } else { " + " };

            // Defer mutations until the display_keys borrow is dropped.
            let mut should_finish_target_seq = false;
            let mut should_clear_target_seq = false;

            // Sequence mode reuses the trigger-style capture UI.
            if target_mode == 2 {
                if is_capturing_new_target {
                    // Currently capturing. Animated prompt matches the trigger style.
                    egui::Frame::NONE
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(255, 200, 130)
                        } else {
                            egui::Color32::from_rgb(255, 235, 180)
                        })
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(t.sequence_capturing())
                                        .size(14.0)
                                        .strong()
                                        .color(if dark_mode {
                                            egui::Color32::from_rgb(80, 60, 20)
                                        } else {
                                            egui::Color32::from_rgb(100, 80, 20)
                                        }),
                                );
                                ui.add_space(4.0);
                                if !target_sequence_capture_list.is_empty() {
                                    let count = target_sequence_capture_list.len();
                                    let last_key = target_sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                    let preview = if count <= 3 {
                                        target_sequence_capture_list.join(" → ")
                                    } else {
                                        format!("{} keys: ... → {}", count, last_key)
                                    };
                                    ui.label(
                                        egui::RichText::new(&preview)
                                            .size(13.0)
                                            .color(if dark_mode {
                                                egui::Color32::from_rgb(60, 40, 10)
                                            } else {
                                                egui::Color32::from_rgb(80, 60, 20)
                                            }),
                                    );
                                }
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(t.sequence_capture_hint())
                                        .size(11.0)
                                        .italics()
                                        .color(if dark_mode {
                                            egui::Color32::from_rgb(100, 80, 40)
                                        } else {
                                            egui::Color32::from_rgb(120, 100, 60)
                                        }),
                                );
                            });
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        // Done button
                        let done_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_complete())
                                .size(13.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(120, 220, 140)
                        } else {
                            egui::Color32::from_rgb(140, 230, 150)
                        })
                        .corner_radius(10.0);
                        if ui.add_sized([120.0, 28.0], done_btn).clicked() {
                            should_finish_target_seq = true;
                        }
                        ui.add_space(8.0);
                        // Clear button
                        let clear_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_clear_btn())
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(230, 100, 100)
                        } else {
                            egui::Color32::from_rgb(250, 150, 150)
                        })
                        .corner_radius(10.0);
                        if ui.add_sized([100.0, 28.0], clear_btn).clicked() {
                            should_clear_target_seq = true;
                        }
                    });
                } else {
                    // Idle state. Mirrors the Single/Multi target button
                    // and keeps a static label so the row width stays
                    // stable. Clicking re-enters capture mode and opens
                    // the amber preview frame above.
                    let seq_btn = egui::Button::new(
                        egui::RichText::new(
                            t.click_to_set_target(),
                        )
                        .size(14.0)
                        .color(if dark_mode {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        }),
                    )
                    .fill(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(245, 245, 250)
                    })
                    .corner_radius(10.0);
                    let target_width =
                        ui.available_width().max(120.0);
                    if ui
                        .add_sized([target_width, 30.0], seq_btn)
                        .clicked()
                        && !*just_captured_input
                    {
                        *key_capture_mode =
                            KeyCaptureMode::NewMappingTarget;
                        capture_pressed_keys.clear();
                        *capture_initial_pressed =
                            crate::gui::SorahkGui::poll_all_pressed_keys();
                        app_state
                            .set_raw_input_capture_mode(true);
                        *just_captured_input = true;
                    }
                }
            } else {
                // Single/Multi mode - original button style
                let target_btn_display = if is_capturing_new_target {
                    t.press_any_key().to_string()
                } else if display_keys.is_empty() {
                    t.click_to_set_target().to_string()
                } else if target_mode == 0 {
                    truncate_text_safe(&*new_mapping_target, BUTTON_TEXT_MAX_CHARS)
                } else {
                    let count = display_keys.len();
                    if count <= 2 {
                        display_keys.join(separator)
                    } else {
                        let last = display_keys.last().map(|s| s.as_str()).unwrap_or("");
                        format!("{} keys: ...{}{}", count, separator.trim(), last)
                    }
                };

                let target_btn = egui::Button::new(
                    egui::RichText::new(&target_btn_display)
                        .size(14.0)
                        .color(if is_capturing_new_target {
                            egui::Color32::from_rgb(255, 215, 0)
                        } else if dark_mode {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        }),
                )
                .fill(if is_capturing_new_target {
                    egui::Color32::from_rgb(70, 130, 180)
                } else if dark_mode {
                    egui::Color32::from_rgb(60, 62, 72)
                } else {
                    egui::Color32::from_rgb(245, 245, 250)
                })
                .corner_radius(10.0);

                let target_width =
                    ui.available_width().max(120.0);
                if ui
                    .add_sized([target_width, 30.0], target_btn)
                    .clicked()
                    && !*just_captured_input
                {
                    *key_capture_mode =
                        KeyCaptureMode::NewMappingTarget;
                    capture_pressed_keys.clear();
                    *capture_initial_pressed =
                        crate::gui::SorahkGui::poll_all_pressed_keys();
                }
            }

            // Show target keys as pill tags for Multi/Sequence modes
            // Clone keys list and count to avoid borrow conflict
            let keys_list: Vec<_> = display_keys.to_vec();
            let keys_count = keys_list.len();
            let mut target_key_to_remove: Option<usize> = None;
            if target_mode > 0 && keys_count > 0 {
                ui.add_space(8.0);
                let full_width = ui.available_width();
                let inner_margin = 10.0;
                // Sequence mode uses trigger-style pink tint, Multi uses blue tint
                let bg_color = if target_mode == 2 {
                    if dark_mode {
                        egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                    } else {
                        egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                    }
                } else {
                    // Multi: blue tint
                    if dark_mode {
                        egui::Color32::from_rgba_premultiplied(135, 206, 235, 25)
                    } else {
                        egui::Color32::from_rgba_premultiplied(173, 216, 230, 120)
                    }
                };
                egui::Frame::NONE
                    .fill(bg_color)
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(egui::Margin::symmetric(inner_margin as i8, inner_margin as i8))
                    .show(ui, |ui| {
                        let content_width = full_width - inner_margin * 2.0;
                        ui.set_min_width(content_width);
                        ui.set_max_width(content_width);
                        // Header
                        ui.horizontal(|ui| {
                            let icon = if target_mode == 2 { "🎬" } else { t.target_icon() };
                            ui.label(egui::RichText::new(icon).size(12.0));
                            ui.label(
                                egui::RichText::new(t.format_targets_count(keys_count))
                                    .size(10.0)
                                    .italics()
                                    .color(if dark_mode {
                                        egui::Color32::from_rgb(150, 180, 200)
                                    } else {
                                        egui::Color32::from_rgb(100, 140, 180)
                                    })
                            );
                        });
                        ui.add_space(6.0);
                        // Pill tags layout
                        let layout_width = content_width;
                        egui::ScrollArea::vertical()
                            .id_salt(0xCAFEBABEu64)
                            .max_height(120.0)
                            .show(ui, |ui| {
                                ui.set_min_width(layout_width);
                                let available_width = layout_width;
                                let sep_char = if target_mode == 2 { "→" } else { "+" };
                                let sep_width = if target_mode == 2 { widgets::arrow_separator_width() } else { 24.0 };

                                // Pre-calculate rows
                                let mut rows: Vec<Vec<usize>> = Vec::new();
                                let mut current_row: Vec<usize> = Vec::new();
                                let mut current_width = 0.0f32;

                                for (key_idx, key) in keys_list.iter().enumerate() {
                                    let pill_width = widgets::estimate_pill_width_editor(key);
                                    let s_width = if key_idx < keys_list.len() - 1 { sep_width } else { 0.0 };
                                    let total_width = pill_width + s_width;
                                    if current_width + total_width > available_width && !current_row.is_empty() {
                                        rows.push(std::mem::take(&mut current_row));
                                        current_width = 0.0;
                                    }
                                    current_row.push(key_idx);
                                    current_width += total_width + 4.0;
                                }
                                if !current_row.is_empty() { rows.push(current_row); }

                                // Render each row
                                for row in &rows {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                                        for &key_idx in row {
                                            let key = &keys_list[key_idx];
                                            // Sequence mode uses trigger-style colors
                                            let (tag_color, text_color) = if target_mode == 2 {
                                                let color = widgets::pill_color(key, dark_mode);
                                                let text = if dark_mode {
                                                    egui::Color32::from_rgb(40, 30, 50)
                                                } else {
                                                    egui::Color32::from_rgb(60, 40, 70)
                                                };
                                                (color, text)
                                            } else {
                                                let color = theme::colors(dark_mode).pill_target;
                                                let text = if dark_mode {
                                                    egui::Color32::from_rgb(20, 60, 80)
                                                } else {
                                                    egui::Color32::from_rgb(40, 100, 140)
                                                };
                                                (color, text)
                                            };
                                            // Get display name for sequence mode
                                            let display_name = if target_mode == 2 {
                                                let (_, name) = widgets::pill_icon_and_label(key);
                                                name
                                            } else {
                                                key.clone()
                                            };
                                            let tag_response = egui::Frame::NONE
                                                .fill(tag_color)
                                                .corner_radius(egui::CornerRadius::same(12))
                                                .inner_margin(egui::Margin::symmetric(8, 4))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.spacing_mut().item_spacing.x = 3.0;
                                                        ui.label(egui::RichText::new(format!("{}", key_idx + 1)).size(9.0).strong().color(text_color));
                                                        let short_name = if display_name.len() > 15 { format!("{}...", &display_name[..12]) } else { display_name };
                                                        ui.label(egui::RichText::new(&short_name).size(11.0).color(text_color));
                                                        let del_btn = ui.add(egui::Button::new(egui::RichText::new("🗑").size(11.0).color(
                                                            if dark_mode { egui::Color32::from_rgb(180, 80, 100) }
                                                            else { egui::Color32::from_rgb(200, 60, 80) }
                                                        )).fill(egui::Color32::TRANSPARENT).frame(false).corner_radius(8.0));
                                                        if del_btn.clicked() { target_key_to_remove = Some(key_idx); }
                                                    });
                                                });
                                            tag_response.response.on_hover_text(key);
                                            if key_idx < keys_list.len() - 1 {
                                                // Sequence mode uses pink arrow like trigger
                                                let sep_color = if target_mode == 2 {
                                                    if dark_mode { egui::Color32::from_rgb(255, 150, 170) }
                                                    else { egui::Color32::from_rgb(255, 120, 150) }
                                                } else if dark_mode { egui::Color32::from_rgb(135, 206, 235) }
                                                else { egui::Color32::from_rgb(70, 130, 180) };
                                                ui.label(egui::RichText::new(sep_char).size(14.0).color(sep_color));
                                            }
                                        }
                                    });
                                    ui.add_space(6.0);
                                }
                            });
                    });
            }
            // Apply deferred target key deletion outside the inner closures.
            if let Some(idx) = target_key_to_remove {
                if target_mode == 2 {
                    target_sequence_capture_list.remove(idx);
                    *new_mapping_target_keys = target_sequence_capture_list.clone();
                } else {
                    new_mapping_target_keys.remove(idx);
                }
                if new_mapping_target_keys.is_empty() {
                    (*new_mapping_target).clear();
                } else {
                    *new_mapping_target = new_mapping_target_keys[0].clone();
                }
                // Keep the rule-properties draft aligned with the new
                // target layout. Drop the mark on the deleted index and
                // shift higher indices down by one.
                let removed = idx as u8;
                new_mapping_hold_indices
                    .retain(|&i| i != removed);
                for i in new_mapping_hold_indices.iter_mut() {
                    if *i > removed {
                        *i -= 1;
                    }
                }
            }
            // Apply deferred finish/clear for sequence target capture
            if should_finish_target_seq {
                *key_capture_mode = KeyCaptureMode::None;
                app_state.set_raw_input_capture_mode(false);
                capture_pressed_keys.clear();
            }
            if should_clear_target_seq {
                target_sequence_capture_list.clear();
                new_mapping_target_keys.clear();
                (*new_mapping_target).clear();
            }

            ui.add_space(12.0);

            // Check all target keys to determine what to show
            let has_mouse_move = new_mapping_target_keys.iter().any(|k| is_mouse_move_target(k));
            let has_mouse_scroll = new_mapping_target_keys.iter().any(|k| is_mouse_scroll_target(k));
            let has_key_or_click = new_mapping_target_keys.iter().any(|k| !is_mouse_move_target(k) && !is_mouse_scroll_target(k));

            // Parameters row
            ui.horizontal(|ui| {
                // Always show interval
                ui.label(
                    egui::RichText::new(t.interval_short())
                        .size(12.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(170, 170, 190)
                        } else {
                            egui::Color32::from_rgb(100, 100, 120)
                        }),
                );
                let interval_edit = egui::TextEdit::singleline(
                    &mut *new_mapping_interval,
                )
                .background_color(if dark_mode {
                    egui::Color32::from_rgb(60, 62, 72)
                } else {
                    egui::Color32::from_rgb(240, 240, 245)
                })
                .hint_text("5")
                .desired_width(55.0)
                .font(egui::TextStyle::Button);
                ui.add_sized([55.0, 28.0], interval_edit);

                // Show duration if has key press/mouse click
                if has_key_or_click {
                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new(t.duration_short())
                            .size(12.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(170, 170, 190)
                            } else {
                                egui::Color32::from_rgb(100, 100, 120)
                            }),
                    );
                    let duration_edit = egui::TextEdit::singleline(
                        &mut *new_mapping_duration,
                    )
                    .background_color(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(240, 240, 245)
                    })
                    .hint_text("5")
                    .desired_width(55.0)
                    .font(egui::TextStyle::Button);
                    ui.add_sized([55.0, 28.0], duration_edit);
                }

                // Show move speed if has mouse move/scroll
                if has_mouse_move || has_mouse_scroll {
                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new(t.speed_label())
                            .size(12.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(170, 170, 190)
                            } else {
                                egui::Color32::from_rgb(100, 100, 120)
                            }),
                    );
                    let hint = if has_mouse_scroll { "120" } else { "5" };
                    let speed_edit = egui::TextEdit::singleline(
                        &mut *new_mapping_move_speed,
                    )
                    .background_color(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(240, 240, 245)
                    })
                    .hint_text(hint)
                    .desired_width(55.0)
                    .font(egui::TextStyle::Button);
                    ui.add_sized([55.0, 28.0], speed_edit);
                }
            });

            ui.add_space(12.0);

            // Action buttons row
            ui.horizontal(|ui| {
                let button_height = 30.0;
                let button_width = 36.0;

                // Add target key
                let add_target_btn = egui::Button::new(
                    egui::RichText::new("+")
                        .color(egui::Color32::WHITE)
                        .size(18.0),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(100, 180, 240)
                } else {
                    egui::Color32::from_rgb(150, 200, 250)
                })
                .corner_radius(12.0);

                if ui
                    .add_sized([button_width, button_height], add_target_btn)
                    .on_hover_text(t.add_target_key_hover())
                    .clicked()
                {
                    *key_capture_mode = KeyCaptureMode::NewMappingTarget;
                    capture_pressed_keys.clear();
                    *capture_initial_pressed = crate::gui::SorahkGui::poll_all_pressed_keys();
                    *just_captured_input = true;
                }

                // Clear all trigger keys
                let clear_trigger_btn = egui::Button::new(
                    egui::RichText::new("✖")
                        .color(egui::Color32::WHITE)
                        .size(16.0),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(220, 160, 100)
                } else {
                    egui::Color32::from_rgb(255, 200, 130)
                })
                .corner_radius(12.0);

                if ui
                    .add_sized([button_width, button_height], clear_trigger_btn)
                    .on_hover_text(t.clear_all_trigger_keys_hover())
                    .clicked()
                {
                    // Clear trigger keys for new mapping
                    sequence_capture_list.clear();
                    (*new_mapping_trigger).clear();
                }

                // Clear all target keys
                let clear_btn = egui::Button::new(
                    egui::RichText::new("✖")
                        .color(egui::Color32::WHITE)
                        .size(14.0),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(230, 100, 100)
                } else {
                    egui::Color32::from_rgb(250, 150, 150)
                })
                .corner_radius(12.0);

                if ui
                    .add_sized([button_width, button_height], clear_btn)
                    .on_hover_text(t.clear_all_target_keys_hover())
                    .clicked()
                {
                    new_mapping_target_keys.clear();
                    (*new_mapping_target).clear();
                    target_sequence_capture_list.clear();
                }

                // Mouse movement direction
                let move_btn = egui::Button::new(
                    egui::RichText::new("⌖")
                        .color(egui::Color32::WHITE)
                        .size(16.0),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(160, 130, 240)
                } else {
                    egui::Color32::from_rgb(180, 150, 250)
                })
                .corner_radius(12.0);

                if ui
                    .add_sized([button_width, button_height], move_btn)
                    .on_hover_text(t.set_mouse_direction_hover())
                    .clicked()
                {
                    *mouse_direction_dialog = Some(
                        crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                    );
                    *mouse_direction_mapping_idx = None;
                }

                // Mouse scroll direction
                let scroll_btn = egui::Button::new(
                    egui::RichText::new("🎡")
                        .color(egui::Color32::WHITE)
                        .size(16.0),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(100, 220, 180)
                } else {
                    egui::Color32::from_rgb(120, 240, 200)
                })
                .corner_radius(12.0);

                if ui
                    .add_sized([button_width, button_height], scroll_btn)
                    .on_hover_text(t.set_mouse_scroll_direction_hover())
                    .clicked()
                {
                    *mouse_scroll_dialog = Some(
                        crate::gui::mouse_scroll_dialog::MouseScrollDialog::new(),
                    );
                    *mouse_scroll_mapping_idx = None;
                }

                ui.add_space(4.0);

                // Turbo toggle for new mapping
                let new_turbo_enabled = *new_mapping_turbo;
                let new_turbo_color = if new_turbo_enabled {
                    if dark_mode {
                        egui::Color32::from_rgb(250, 200, 80)
                    } else {
                        egui::Color32::from_rgb(255, 220, 120)
                    }
                } else if dark_mode {
                    egui::Color32::from_rgb(100, 100, 120)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };

                let new_turbo_icon =
                    if new_turbo_enabled { "⚡" } else { "○" };

                let new_turbo_btn = egui::Button::new(
                    egui::RichText::new(new_turbo_icon)
                        .color(egui::Color32::WHITE)
                        .size(16.0),
                )
                .fill(new_turbo_color)
                .corner_radius(12.0)
                .sense(egui::Sense::click());

                let new_hover_text = if new_turbo_enabled {
                    translations.turbo_on_hover()
                } else {
                    translations.turbo_off_hover()
                };

                if ui
                    .add_sized([36.0, button_height], new_turbo_btn)
                    .on_hover_text(new_hover_text)
                    .clicked()
                {
                    *new_mapping_turbo =
                        !*new_mapping_turbo;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = egui::Button::new(
                        egui::RichText::new(t.add_button_text())
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(if dark_mode {
                        egui::Color32::from_rgb(100, 220, 180)
                    } else {
                        egui::Color32::from_rgb(120, 240, 200)
                    })
                    .corner_radius(12.0);

                    let has_trigger = !(*new_mapping_trigger).is_empty() || !sequence_capture_list.is_empty();
                    if ui.add_sized([80.0, button_height], add_btn).clicked()
                        && has_trigger
                        && !new_mapping_target_keys.is_empty()
                    {
                        let is_sequence_mode = !sequence_capture_list.is_empty();
                        let (trigger_key, trigger_sequence) = if is_sequence_mode {
                            // Sequence mode. First key becomes trigger_key, full list becomes trigger_sequence.
                            let trigger = sequence_capture_list[0].to_uppercase();
                            let sequence = sequence_capture_list.iter()
                                .map(|k| k.to_uppercase())
                                .collect::<Vec<_>>()
                                .join(",");
                            (trigger, Some(sequence))
                        } else {
                            // Single key mode
                            ((*new_mapping_trigger).to_uppercase(), None)
                        };

                        // Check for duplicate trigger key
                        let is_duplicate = temp_config
                            .mappings
                            .iter()
                            .any(|m| m.trigger_key == trigger_key);

                        if is_duplicate {
                *duplicate_mapping_error = Some(
                    t.duplicate_trigger_error().to_string(),
                );
            } else {
                // Clear any previous error
                *duplicate_mapping_error = None;

                let interval = new_mapping_interval
                    .parse::<u64>()
                    .ok()
                    .map(|v| v.max(5));
                let duration = new_mapping_duration
                    .parse::<u64>()
                    .ok()
                    .map(|v| v.max(2));
                let move_speed = new_mapping_move_speed
                    .parse::<i32>()
                    .unwrap_or(5)
                    .clamp(1, 100);
                let sequence_window = new_mapping_sequence_window
                    .parse::<u64>()
                    .unwrap_or(300)
                    .max(50);

                let turbo_enabled = *new_mapping_turbo;

                // Flush the rule-property draft when hold indices or
                // append keys are configured. Applies to every target
                // mode since the Rule Props dialog is universal.
                let (new_hold, new_append) =
                    if !new_mapping_hold_indices.is_empty()
                        || !new_mapping_append_keys.is_empty()
                    {
                        let hold = if new_mapping_hold_indices
                            .is_empty()
                        {
                            None
                        } else {
                            Some(
                                smallvec::SmallVec::from_vec(
                                    new_mapping_hold_indices
                                        .clone(),
                                ),
                            )
                        };
                        let append = if new_mapping_append_keys
                            .is_empty()
                        {
                            None
                        } else {
                            Some(
                                smallvec::SmallVec::from_vec(
                                    new_mapping_append_keys
                                        .clone(),
                                ),
                            )
                        };
                        (hold, append)
                    } else {
                        (None, None)
                    };

                temp_config.mappings.push(KeyMapping {
                    trigger_key,
                    trigger_sequence,
                    sequence_window_ms: sequence_window,
                    target_keys: new_mapping_target_keys.iter()
                        .map(|k| k.to_uppercase())
                        .collect(),
                    interval,
                    event_duration: duration,
                    turbo_enabled,
                    move_speed,
                    target_mode: (*new_mapping_target_mode),
                    hold_indices: new_hold,
                    append_keys: new_append,
                });

                // Clear input fields
                (*new_mapping_trigger).clear();
                (*new_mapping_target).clear();
                new_mapping_target_keys.clear();
                (*new_mapping_interval).clear();
                (*new_mapping_duration).clear();
                *new_mapping_move_speed = "5".to_string();
                *new_mapping_turbo = true; // Reset to default
                sequence_capture_list.clear();
                *new_mapping_sequence_window = "300".to_string();
                *new_mapping_is_sequence_mode = false;
                *new_mapping_target_mode = 0;
                target_sequence_capture_list.clear();
                new_mapping_hold_indices.clear();
                new_mapping_append_keys.clear();
                *sequence_last_mouse_pos = None;
                *sequence_last_mouse_direction = None;
                *sequence_mouse_delta = egui::Vec2::ZERO;
            }
        }
    });

            // Display duplicate trigger error if exists
            if let Some(ref error_msg) = *duplicate_mapping_error {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(error_msg)
                        .color(egui::Color32::from_rgb(
                            255, 100, 100,
                        ))
                        .size(13.0),
                );
            }
        });

    ui.add_space(8.0);
});
}
