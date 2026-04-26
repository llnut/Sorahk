//! Settings dialog "Existing mapping list" section. Renders the list of
//! existing key mappings inside the Key Mappings card. Implemented as a
//! free function so the caller can split-borrow `SorahkGui` fields
//! disjointly with the parent scroll-area closure. The sibling
//! `mapping_editor` module renders the "Add New Mapping" form.

use super::helpers::{BUTTON_TEXT_MAX_CHARS, truncate_text_safe};
use crate::config::AppConfig;
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

// Local alias matching the parent module. Fields are mapping index,
// current target keys, current hold indices, and current append keys.
type RulePropsRequest = (usize, Vec<String>, Vec<u8>, Vec<String>);

/// Renders the existing-mapping list inside the Key Mappings card.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_mapping_list_section(
    ui: &mut egui::Ui,
    temp_config: &mut AppConfig,
    key_capture_mode: &mut KeyCaptureMode,
    capture_initial_pressed: &mut HashSet<u32>,
    capture_pressed_keys: &mut HashSet<u32>,
    just_captured_input: &mut bool,
    editing_target_seq_idx: &mut Option<usize>,
    editing_target_seq_list: &mut Vec<String>,
    sequence_last_mouse_pos: &mut Option<egui::Pos2>,
    sequence_last_mouse_direction: &mut Option<String>,
    sequence_mouse_delta: &mut egui::Vec2,
    mouse_direction_dialog: &mut Option<MouseDirectionDialog>,
    mouse_direction_mapping_idx: &mut Option<usize>,
    mouse_scroll_dialog: &mut Option<MouseScrollDialog>,
    mouse_scroll_mapping_idx: &mut Option<usize>,
    rule_properties_dialog: &mut Option<RulePropertiesDialog>,
    rule_props_editing_idx: &mut Option<usize>,
    app_state: &Arc<AppState>,
    dark_mode: bool,
    translations: CachedTranslations,
) {
    let t = translations;
    ui.add_space(2.0);
    let hint_bg = if dark_mode {
        egui::Color32::from_rgba_premultiplied(60, 50, 70, 180)
    } else {
        egui::Color32::from_rgba_premultiplied(220, 200, 235, 220)
    };
    ui.horizontal(|ui| {
        egui::Frame::NONE
            .fill(hint_bg)
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::CollapsingHeader::new(
                    egui::RichText::new(t.diagonal_hint_title())
                        .size(12.0)
                        .color(if dark_mode {
                            egui::Color32::from_rgb(
                                255, 180, 220,
                            )
                        } else {
                            egui::Color32::from_rgb(
                                200, 100, 150,
                            )
                        }),
                )
                .default_open(true)
                .show(ui, |ui| {
                    ui.add_space(2.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(
                                t.diagonal_hint(),
                            )
                            .size(11.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(
                                    230, 230, 230,
                                )
                            } else {
                                egui::Color32::from_rgb(
                                    180, 100, 50,
                                )
                            }),
                        )
                        .wrap(),
                    );
                });
            });
    });
    ui.add_space(4.0);

    // Existing mappings
    let mut to_remove = None;
    // Deferred open request for the rule-properties dialog. Collected
    // inside the iter_mut loop so the dialog opens only after the
    // mutable borrow on `temp_config.mappings` is released.
    let mut open_rule_props: Option<RulePropsRequest> =
        None;
    for (idx, mapping) in
        temp_config.mappings.iter_mut().enumerate()
    {
        // Mapping card with card-style layout
        let mapping_card_bg = if dark_mode {
            egui::Color32::from_rgb(50, 52, 62)
        } else {
            egui::Color32::from_rgb(255, 250, 255)
        };

        egui::Frame::NONE
            .fill(mapping_card_bg)
            .corner_radius(egui::CornerRadius::same(16))
            .inner_margin(egui::Margin::same(14))
            .stroke(if dark_mode {
                egui::Stroke::NONE
            } else {
                egui::Stroke::new(
                    1.5,
                    egui::Color32::from_rgba_premultiplied(255, 182, 193, 80)
                )
            })
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Header with mapping number
                ui.horizontal(|ui| {
                    let num_str = (idx + 1).to_string();
                    let mut header_str = String::with_capacity(1 + num_str.len());
                    header_str.push('#');
                    header_str.push_str(&num_str);
                    ui.label(
                        egui::RichText::new(&header_str)
                            .size(16.0)
                            .strong()
                            .color(if dark_mode {
                                egui::Color32::from_rgb(255, 182, 193)
                            } else {
                                egui::Color32::from_rgb(255, 105, 180)
                            }),
                    );
                    ui.add_space(10.0);
                    // Show trigger type badge
                    let (badge_text, badge_color_bg, badge_color_text) = if mapping.is_sequence_trigger() {
                        (
                            t.trigger_mode_sequence_badge(),
                            if dark_mode {
                                egui::Color32::from_rgb(255, 182, 193)
                            } else {
                                egui::Color32::from_rgb(255, 218, 224)
                            },
                            if dark_mode {
                                egui::Color32::from_rgb(80, 20, 40)
                            } else {
                                egui::Color32::from_rgb(220, 80, 120)
                            }
                        )
                    } else {
                        (
                            t.trigger_mode_single_badge(),
                            if dark_mode {
                                egui::Color32::from_rgb(135, 206, 235)
                            } else {
                                egui::Color32::from_rgb(173, 216, 230)
                            },
                            if dark_mode {
                                egui::Color32::from_rgb(20, 60, 80)
                            } else {
                                egui::Color32::from_rgb(40, 80, 120)
                            }
                        )
                    };
                    egui::Frame::NONE
                        .fill(badge_color_bg)
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(badge_text)
                                    .size(11.0)
                                    .strong()
                                    .color(badge_color_text),
                            );
                        });
                });
                ui.add_space(10.0);

                // Trigger mode selection
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t.trigger_mode_label())
                            .size(13.0)
                            .color(if dark_mode {
                                egui::Color32::from_rgb(200, 200, 220)
                            } else {
                                egui::Color32::from_rgb(80, 80, 100)
                            }),
                    );
                    ui.add_space(8.0);
                    let is_sequence = mapping.is_sequence_trigger();
                    // Single Key Button - matches Single badge colors
                    let single_btn = egui::Button::new(
                        egui::RichText::new(t.trigger_mode_single())
                            .size(11.0)
                            .color(if !is_sequence {
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
                    .fill(if !is_sequence {
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
                    .corner_radius(8.0);
                    if ui.add(single_btn).clicked() && is_sequence {
                        mapping.trigger_sequence = None;
                        if let Some(seq_str) = mapping.sequence_string() {
                            let keys: Vec<&str> = seq_str.split(',').collect();
                            if let Some(first_key) = keys.first() {
                                mapping.trigger_key = first_key.trim().to_string();
                            }
                        }
                    }
                    ui.add_space(6.0);
                    // Sequence Button - matches Sequence badge colors
                    let seq_btn = egui::Button::new(
                        egui::RichText::new(t.trigger_mode_sequence())
                            .size(11.0)
                            .color(if is_sequence {
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
                    .fill(if is_sequence {
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
                    .corner_radius(8.0);
                    if ui.add(seq_btn).clicked() && !is_sequence {
                        if !mapping.trigger_key.is_empty() {
                            mapping.trigger_sequence = Some(mapping.trigger_key.clone());
                        } else {
                            mapping.trigger_sequence = Some(String::new());
                            mapping.trigger_key.clear();
                        }
                    }
                });
                ui.add_space(8.0);

                // Trigger key section
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
                let is_capturing_trigger = (*key_capture_mode)
                    == KeyCaptureMode::MappingTrigger(idx);
                // For single key mode, show normal capture button
                if !mapping.is_sequence_trigger() {
                    let trigger_display = if is_capturing_trigger {
                        t.press_any_key().to_string()
                    } else if mapping.trigger_key.is_empty() {
                        t.click_to_set_trigger().to_string()
                    } else {
                        truncate_text_safe(&mapping.trigger_key, BUTTON_TEXT_MAX_CHARS)
                    };

                    let trigger_btn = egui::Button::new(
                        egui::RichText::new(&trigger_display)
                            .size(14.0)
                            .color(
                                if is_capturing_trigger {
                                    egui::Color32::from_rgb(255, 215, 0)
                                } else if dark_mode {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(40, 40, 40)
                                },
                            ),
                    )
                    .fill(if is_capturing_trigger {
                        egui::Color32::from_rgb(70, 130, 180)
                    } else if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(245, 245, 250)
                    })
                    .corner_radius(10.0);

                    let mut trigger_response = ui
                        .add_sized([ui.available_width(), 30.0], trigger_btn);
                    if !is_capturing_trigger && !mapping.trigger_key.is_empty() && mapping.trigger_key.chars().count() > BUTTON_TEXT_MAX_CHARS {
                        trigger_response = trigger_response.on_hover_text(&mapping.trigger_key);
                    }
                    if trigger_response.clicked() && !(*just_captured_input) {
                        *key_capture_mode =
                            KeyCaptureMode::MappingTrigger(idx);
                        capture_pressed_keys.clear();
                        *capture_initial_pressed =
                            crate::gui::SorahkGui::poll_all_pressed_keys();
                        app_state.set_raw_input_capture_mode(true);
                        *just_captured_input = true;
                    }
                } else {
                    // For sequence mode, show add button
                    ui.horizontal(|ui| {
                        let add_seq_btn = egui::Button::new(
                            egui::RichText::new(t.add_button_text())
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(100, 180, 240)
                        } else {
                            egui::Color32::from_rgb(150, 200, 250)
                        })
                        .corner_radius(10.0);
                        if ui.add(add_seq_btn).on_hover_text(t.add_sequence_key_hover()).clicked() && !(*just_captured_input) {
                            *key_capture_mode = KeyCaptureMode::MappingTrigger(idx);
                            capture_pressed_keys.clear();
                            *capture_initial_pressed = crate::gui::SorahkGui::poll_all_pressed_keys();
                            app_state.set_raw_input_capture_mode(true);
                            *just_captured_input = true;
                            // Initialize sequence capture state for mouse movement
                            *sequence_last_mouse_pos = None;
                            *sequence_last_mouse_direction = None;
                            *sequence_mouse_delta = egui::Vec2::ZERO;
                        }
                    });
                }
                // Handle captured input for sequence mode
                if is_capturing_trigger && mapping.is_sequence_trigger() {
                    ui.add_space(6.0);
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
                                if let Some(seq_str) = mapping.sequence_string() {
                                    let seq_keys: Vec<&str> = seq_str.split(',')
                                        .map(|s| s.trim())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    if !seq_keys.is_empty() {
                                        ui.add_space(4.0);
                                        let count = seq_keys.len();
                                        let preview = if count <= 3 {
                                            seq_keys.join(" → ")
                                        } else {
                                            let last = seq_keys.last().unwrap_or(&"");
                                            format!("{} keys: ... → {}", count, last)
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
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                            let done_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_complete())
                                .size(12.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(120, 220, 140)
                        } else {
                            egui::Color32::from_rgb(140, 230, 150)
                        })
                        .corner_radius(8.0);
                        if ui.add_sized([100.0, 26.0], done_btn).clicked() {
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
                        ui.add_space(6.0);
                            let clear_btn = egui::Button::new(
                            egui::RichText::new(t.sequence_clear_btn())
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(230, 100, 100)
                        } else {
                            egui::Color32::from_rgb(250, 150, 150)
                        })
                        .corner_radius(8.0);
                        if ui.add_sized([90.0, 26.0], clear_btn).clicked() {
                            mapping.trigger_sequence = Some(String::new());
                            mapping.trigger_key.clear();
                        }
                    });
                }
                // Display sequence keys with horizontal flow layout
                if mapping.is_sequence_trigger()
                    && let Some(seq_str) = mapping.sequence_string() {
                        let seq_keys: Vec<String> = seq_str.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !seq_keys.is_empty() {
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
                                    // Header with sequence icon
                                    ui.horizontal(|ui| {
                                        let t = &translations;
                                        ui.label(
                                            egui::RichText::new(t.sequence_icon())
                                                .size(12.0)
                                        );
                                        ui.label(
                                            egui::RichText::new(t.format_keys_count(seq_keys.len()))
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
                                    let scroll_id = idx;
                                    egui::ScrollArea::vertical()
                                        .id_salt(scroll_id << 32 | 0x1)
                                        .max_height(120.0)
                                        .show(ui, |ui| {
                                            ui.set_min_width(layout_width);
                                            let mut keys_to_remove = Vec::new();
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
                                                        let (icon, display_name) = widgets::pill_icon_and_label(key);
                                                        let tag_color = widgets::pill_color(key, dark_mode);
                                                        let text_color = if dark_mode {
                                                            egui::Color32::from_rgb(40, 30, 50)
                                                        } else {
                                                            egui::Color32::from_rgb(60, 40, 70)
                                                        };
                                                        // Pill-shaped tag
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
                                                                    // Icon
                                                                    ui.label(
                                                                        egui::RichText::new(icon)
                                                                            .size(12.0)
                                                                            .color(text_color)
                                                                    );
                                                                    // Display name, truncated when long.
                                                                    let short_name = if display_name.len() > 12 {
                                                                        format!("{}...", &display_name[..9])
                                                                    } else {
                                                                        display_name
                                                                    };
                                                                    ui.label(
                                                                        egui::RichText::new(&short_name)
                                                                            .size(10.0)
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
                                                        // Arrow separator between items
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
                                            if !keys_to_remove.is_empty() {
                                                let mut updated_keys = seq_keys.clone();
                                                for &key_idx in keys_to_remove.iter().rev() {
                                                    updated_keys.remove(key_idx);
                                                }
                                                if !updated_keys.is_empty() {
                                                    let new_seq = updated_keys.join(",");
                                                    mapping.trigger_sequence = Some(new_seq);
                                                    mapping.trigger_key = updated_keys[0].clone();
                                                } else {
                                                    mapping.trigger_sequence = Some(String::new());
                                                    mapping.trigger_key.clear();
                                                }
                                            }
                                        });
                                });
                        }
                    }

                ui.add_space(12.0);

                // Target key section with mode selection
                let target_mode = mapping.target_mode;
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
                        mapping.target_mode = 0;
                        if mapping.target_keys.len() > 1 {
                            let first = mapping.target_keys[0].clone();
                            mapping.target_keys.clear();
                            mapping.target_keys.push(first);
                        }
                        // Clear editing state if switching away from sequence
                        if target_mode == 2 && *editing_target_seq_idx == Some(idx) {
                            editing_target_seq_list.clear();
                            *editing_target_seq_idx = None;
                            *key_capture_mode = KeyCaptureMode::None;
                            app_state.set_raw_input_capture_mode(false);
                        }
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
                        mapping.target_mode = 1;
                        // Deduplicate target keys when switching from Sequence to Multi
                        if target_mode == 2 {
                            let mut seen = std::collections::HashSet::new();
                            mapping.target_keys.retain(|k| seen.insert(k.clone()));
                            // Clear editing state if switching away from sequence
                            if *editing_target_seq_idx == Some(idx) {
                                editing_target_seq_list.clear();
                                *editing_target_seq_idx = None;
                                *key_capture_mode = KeyCaptureMode::None;
                                app_state.set_raw_input_capture_mode(false);
                            }
                        }
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
                        mapping.target_mode = 2;
                    }

                    // Right-aligned Rule Props button. Sits on the
                    // target-mode row so one entry point serves Single,
                    // Multi, and Sequence without growing the card.
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
                            let props_btn =
                                egui::Button::new(
                                    egui::RichText::new(
                                        t.rule_props_button(),
                                    )
                                    .size(11.0)
                                    .color(props_text),
                                )
                                .fill(props_fill)
                                .corner_radius(10.0);
                            if ui.add(props_btn).clicked() {
                                // Prefer the live in-progress list while a
                                // sequence is being captured for this mapping.
                                // Fall back to the persisted target_keys.
                                let keys: Vec<String> = if mapping
                                    .target_mode
                                    == 2
                                    && *editing_target_seq_idx
                                        == Some(idx)
                                    && !editing_target_seq_list
                                        .is_empty()
                                {
                                    editing_target_seq_list
                                        .clone()
                                } else {
                                    mapping
                                        .target_keys
                                        .iter()
                                        .cloned()
                                        .collect()
                                };
                                let hold: Vec<u8> = mapping
                                    .hold_indices
                                    .as_ref()
                                    .map(|v| v.to_vec())
                                    .unwrap_or_default();
                                let append: Vec<String> =
                                    mapping
                                        .append_keys
                                        .as_ref()
                                        .map(|v| v.to_vec())
                                        .unwrap_or_default();
                                open_rule_props = Some((
                                    idx, keys, hold, append,
                                ));
                            }
                        },
                    );
                });
                ui.add_space(6.0);
                let is_capturing_target = (*key_capture_mode)
                    == KeyCaptureMode::MappingTarget(idx);
                // For single/multi mode, show normal capture button
                if target_mode != 2 {
                    let target_full_text = mapping.target_keys_display();
                    let target_display_text = if is_capturing_target {
                        t.press_any_key().to_string()
                    } else if target_full_text.is_empty() {
                        t.click_to_set_target().to_string()
                    } else {
                        truncate_text_safe(&target_full_text, BUTTON_TEXT_MAX_CHARS)
                    };

                    let target_btn = egui::Button::new(
                        egui::RichText::new(&target_display_text)
                            .size(14.0)
                            .color(
                                if is_capturing_target {
                                    egui::Color32::from_rgb(255, 215, 0)
                                } else if dark_mode {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(40, 40, 40)
                                },
                            ),
                    )
                    .fill(if is_capturing_target {
                        egui::Color32::from_rgb(70, 130, 180)
                    } else if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(245, 245, 250)
                    })
                    .corner_radius(10.0);

                    let target_width =
                        ui.available_width().max(120.0);
                    let mut target_response = ui
                        .add_sized([target_width, 30.0], target_btn);
                    if !is_capturing_target
                        && !target_full_text.is_empty()
                        && target_full_text.chars().count()
                            > BUTTON_TEXT_MAX_CHARS
                    {
                        target_response = target_response
                            .on_hover_text(&target_full_text);
                    }
                    if target_response.clicked()
                        && !(*just_captured_input)
                    {
                        *key_capture_mode =
                            KeyCaptureMode::MappingTarget(idx);
                        capture_pressed_keys.clear();
                        *capture_initial_pressed =
                            crate::gui::SorahkGui::poll_all_pressed_keys();
                    }
                } else {
                    // For sequence mode
                    if is_capturing_target {
                        // Show capture UI with preview from editing list
                        let preview_keys = &editing_target_seq_list;
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
                                    if !preview_keys.is_empty() {
                                        let count = preview_keys.len();
                                        let preview = if count <= 3 {
                                            preview_keys.join(" → ")
                                        } else {
                                            let last = preview_keys.last().map(|s| s.as_str()).unwrap_or("");
                                            format!("{} keys: ... → {}", count, last)
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
                        // Track button clicks
                        let mut should_finish = false;
                        let mut should_clear = false;
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
                                should_finish = true;
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
                                should_clear = true;
                            }
                        });
                        // Handle button actions after UI
                        if should_finish {
                            // Sync editing list back to mapping
                            mapping.target_keys = editing_target_seq_list.iter().cloned().collect();
                            // Drop hold indices that no longer point into
                            // the new body. Pill-tag deletions already
                            // shift indices live, but a fresh capture may
                            // shorten the list past that path.
                            let new_len =
                                mapping.target_keys.len();
                            if let Some(holds) =
                                mapping.hold_indices.as_mut()
                            {
                                holds.retain(|i| {
                                    (*i as usize) < new_len
                                });
                                if holds.is_empty() {
                                    mapping.hold_indices =
                                        None;
                                }
                            }
                            editing_target_seq_list.clear();
                            *editing_target_seq_idx = None;
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
                        if should_clear {
                            editing_target_seq_list.clear();
                        }
                    } else {
                        // Idle state. Mirrors the Single/Multi target
                        // button and keeps a static label so the row
                        // width stays stable regardless of content.
                        let seq_btn = egui::Button::new(
                            egui::RichText::new(
                                t.click_to_set_target(),
                            )
                            .size(14.0)
                            .color(if dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(
                                    40, 40, 40,
                                )
                            }),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(60, 62, 72)
                        } else {
                            egui::Color32::from_rgb(
                                245, 245, 250,
                            )
                        })
                        .corner_radius(10.0);
                        let target_width =
                            ui.available_width().max(120.0);
                        if ui
                            .add_sized(
                                [target_width, 30.0],
                                seq_btn,
                            )
                            .clicked()
                            && !(*just_captured_input)
                        {
                            *editing_target_seq_list =
                                mapping
                                    .target_keys
                                    .iter()
                                    .cloned()
                                    .collect();
                            *editing_target_seq_idx =
                                Some(idx);
                            *key_capture_mode =
                                KeyCaptureMode::MappingTarget(
                                    idx,
                                );
                            capture_pressed_keys.clear();
                            *capture_initial_pressed =
                                crate::gui::SorahkGui::poll_all_pressed_keys();
                            app_state
                                .set_raw_input_capture_mode(
                                    true,
                                );
                            *just_captured_input = true;
                        }
                    }
                }
                // Show target keys as pill tags when multiple targets exist or in sequence mode
                // During sequence capture, show from editing list for real-time updates
                let display_keys: Vec<String> = if is_capturing_target && target_mode == 2 {
                    editing_target_seq_list.clone()
                } else {
                    mapping.get_target_keys().to_vec()
                };
                if display_keys.len() > 1 || target_mode == 2 {
                    ui.add_space(8.0);
                    let full_width = ui.available_width();
                    let inner_margin = 10.0;
                    let keys_list = display_keys;
                    let keys_count = keys_list.len();
                    let mut key_to_remove: Option<usize> = None;

                    // Sequence mode uses pink tint, Multi uses blue tint
                    let bg_color = if target_mode == 2 {
                        if dark_mode {
                            egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                        } else {
                            egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                        }
                    } else if dark_mode {
                        egui::Color32::from_rgba_premultiplied(135, 206, 235, 25)
                    } else {
                        egui::Color32::from_rgba_premultiplied(173, 216, 230, 120)
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
                                let t = &translations;
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
                                .id_salt(idx << 32 | 0x2)
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
                                                            if del_btn.clicked() { key_to_remove = Some(key_idx); }
                                                        });
                                                    });
                                                tag_response.response.on_hover_text(key);
                                                if key_idx < keys_list.len() - 1 {
                                                    // Sequence mode uses pink arrow, Multi uses blue "+"
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
                    // Apply deferred deletion
                    if let Some(remove_idx) = key_to_remove {
                        if is_capturing_target && target_mode == 2 {
                            // During sequence capture, remove from editing list
                            if remove_idx < editing_target_seq_list.len() {
                                editing_target_seq_list.remove(remove_idx);
                            }
                        } else {
                            // Normal mode: remove from mapping by index
                            mapping.remove_target_key_at(remove_idx);
                            // Rewrite the hold-indices set so marks do
                            // not silently migrate to the wrong key.
                            if let Some(existing) =
                                mapping.hold_indices.as_mut()
                            {
                                let removed = remove_idx as u8;
                                existing.retain(|&mut i| i != removed);
                                for i in existing.iter_mut() {
                                    if *i > removed {
                                        *i -= 1;
                                    }
                                }
                                if existing.is_empty() {
                                    mapping.hold_indices = None;
                                }
                            }
                        }
                    }
                }

                ui.add_space(12.0);

                // Check all target keys to determine what to show
                let target_keys = mapping.get_target_keys();
                let has_mouse_move = target_keys.iter().any(|k| is_mouse_move_target(k));
                let has_mouse_scroll = target_keys.iter().any(|k| is_mouse_scroll_target(k));
                let has_key_or_click = target_keys.iter().any(|k| !is_mouse_move_target(k) && !is_mouse_scroll_target(k));

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
                    let mut interval_str = mapping
                        .interval
                        .unwrap_or(temp_config.interval)
                        .to_string();

                    let interval_edit = egui::TextEdit::singleline(
                        &mut interval_str,
                    )
                    .background_color(if dark_mode {
                        egui::Color32::from_rgb(60, 62, 72)
                    } else {
                        egui::Color32::from_rgb(240, 240, 245)
                    })
                    .desired_width(55.0)
                    .font(egui::TextStyle::Button);

                    if ui
                        .add_sized([55.0, 28.0], interval_edit)
                        .changed()
                        && let Ok(val) = interval_str.parse::<u64>()
                    {
                        mapping.interval = Some(val.max(5));
                    }

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
                        let mut duration_str = mapping
                            .event_duration
                            .unwrap_or(temp_config.event_duration)
                            .to_string();

                        let duration_edit = egui::TextEdit::singleline(
                            &mut duration_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(60, 62, 72)
                        } else {
                            egui::Color32::from_rgb(240, 240, 245)
                        })
                        .desired_width(55.0)
                        .font(egui::TextStyle::Button);

                        if ui
                            .add_sized([55.0, 28.0], duration_edit)
                            .changed()
                            && let Ok(val) = duration_str.parse::<u64>()
                        {
                            mapping.event_duration = Some(val.max(2));
                        }
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
                        let mut speed_str = mapping
                            .move_speed
                            .to_string();

                        let speed_edit = egui::TextEdit::singleline(
                            &mut speed_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(60, 62, 72)
                        } else {
                            egui::Color32::from_rgb(240, 240, 245)
                        })
                        .desired_width(55.0)
                        .font(egui::TextStyle::Button);

                        let max_val = if has_mouse_scroll { 1200 } else { 100 };
                        if ui
                            .add_sized([55.0, 28.0], speed_edit)
                            .changed()
                            && let Ok(val) = speed_str.parse::<i32>()
                        {
                            mapping.move_speed = val.clamp(1, max_val);
                        }
                    }
                    // Show sequence window if is sequence trigger
                    if mapping.is_sequence_trigger() {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(t.sequence_window_label())
                                .size(12.0)
                                .color(if dark_mode {
                                    egui::Color32::from_rgb(170, 170, 190)
                                } else {
                                    egui::Color32::from_rgb(100, 100, 120)
                                }),
                        );
                        let mut window_str = mapping.sequence_window_ms.to_string();
                        let window_edit = egui::TextEdit::singleline(
                            &mut window_str,
                        )
                        .background_color(if dark_mode {
                            egui::Color32::from_rgb(60, 62, 72)
                        } else {
                            egui::Color32::from_rgb(240, 240, 245)
                        })
                        .desired_width(55.0)
                        .font(egui::TextStyle::Button);
                        if ui
                            .add_sized([55.0, 28.0], window_edit)
                            .changed()
                            && let Ok(val) = window_str.parse::<u64>()
                        {
                            mapping.sequence_window_ms = val.max(50);
                        }
                    }
                });

                ui.add_space(12.0);

                // Action buttons row
                ui.horizontal(|ui| {
                    let button_height = 28.0;
                    let button_width = 36.0;

                    // Add target key with capture
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
                    .corner_radius(16.0);

                    if ui
                        .add_sized([button_width, button_height], add_target_btn)
                        .on_hover_text(t.add_target_key_hover())
                        .clicked()
                    {
                        *key_capture_mode = KeyCaptureMode::MappingTarget(idx);
                        capture_pressed_keys.clear();
                        *capture_initial_pressed = crate::gui::SorahkGui::poll_all_pressed_keys();
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
                    .corner_radius(16.0);

                    if ui
                        .add_sized([button_width, button_height], clear_trigger_btn)
                        .on_hover_text(t.clear_all_trigger_keys_hover())
                        .clicked()
                    {
                        // Clear trigger keys based on mode
                        if mapping.is_sequence_trigger() {
                            mapping.trigger_sequence = Some(String::new());
                            mapping.trigger_key.clear();
                        } else {
                            mapping.trigger_key.clear();
                        }
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
                    .corner_radius(16.0);

                    if ui
                        .add_sized([button_width, button_height], clear_btn)
                        .on_hover_text(t.clear_all_target_keys_hover())
                        .clicked()
                    {
                        mapping.clear_target_keys();
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
                    .corner_radius(16.0);

                    if ui
                        .add_sized([button_width, button_height], move_btn)
                        .on_hover_text(t.set_mouse_direction_hover())
                        .clicked()
                    {
                        *mouse_direction_dialog = Some(
                            crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                        );
                        *mouse_direction_mapping_idx = Some(idx);
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
                    .corner_radius(16.0);

                    if ui
                        .add_sized([button_width, button_height], scroll_btn)
                        .on_hover_text(t.set_mouse_scroll_direction_hover())
                        .clicked()
                    {
                        *mouse_scroll_dialog = Some(
                            crate::gui::mouse_scroll_dialog::MouseScrollDialog::new(),
                        );
                        *mouse_scroll_mapping_idx = Some(idx);
                    }

                    ui.add_space(4.0);

                    // Turbo toggle
                    let turbo_enabled = mapping.turbo_enabled;
                    let turbo_color = if turbo_enabled {
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

                    let turbo_icon =
                        if turbo_enabled { "⚡" } else { "○" };
                    let turbo_btn = egui::Button::new(
                        egui::RichText::new(turbo_icon)
                            .color(egui::Color32::WHITE)
                            .size(16.0),
                    )
                    .fill(turbo_color)
                    .corner_radius(16.0)
                    .sense(egui::Sense::click());

                    let hover_text = if turbo_enabled {
                        translations.turbo_on_hover()
                    } else {
                        translations.turbo_off_hover()
                    };

                    if ui
                        .add_sized([36.0, button_height], turbo_btn)
                        .on_hover_text(hover_text)
                        .clicked()
                    {
                        mapping.turbo_enabled =
                            !mapping.turbo_enabled;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let t = &translations;
                        let delete_btn = egui::Button::new(
                            egui::RichText::new(t.delete_icon())
                                .size(15.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(240, 120, 170)
                        } else {
                            egui::Color32::from_rgb(255, 180, 210)
                        })
                        .corner_radius(16.0);

                        if ui
                            .add_sized([36.0, button_height], delete_btn)
                            .clicked()
                        {
                            to_remove = Some(idx);
                        }
                    });
                });
            });

        ui.add_space(10.0);
    }

    if let Some(idx) = to_remove {
        temp_config.mappings.remove(idx);
    }

    if let Some((idx, keys, hold, append)) =
        open_rule_props
    {
        *rule_props_editing_idx = Some(idx);
        *rule_properties_dialog = Some(
            crate::gui::rule_properties_dialog::RulePropertiesDialog::new(
                &keys,
                &hold,
                &append,
            ),
        );
    }
}
