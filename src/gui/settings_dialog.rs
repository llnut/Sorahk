//! Settings dialog implementation.

use crate::config::KeyMapping;
use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;
use crate::gui::utils::key_to_string;
use eframe::egui;

impl SorahkGui {
    /// Renders the settings dialog for configuration management.
    pub(super) fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_cancel = false;

        // Use cached translations if temp config language matches current, otherwise create new one
        let temp_lang = self
            .temp_config
            .as_ref()
            .map(|c| c.language)
            .unwrap_or(self.config.language);
        let t = if temp_lang == self.config.language {
            &self.translations
        } else {
            // Language is being previewed in settings but not saved yet, use current cached translations
            &self.translations
        };

        // Handle key capture if in capture mode
        if self.key_capture_mode != KeyCaptureMode::None {
            ctx.input(|i| {
                for key in i.keys_down.iter() {
                    if let Some(key_name) = key_to_string(*key) {
                        // Capture the key and update the appropriate field
                        if let Some(temp_config) = &mut self.temp_config {
                            match self.key_capture_mode {
                                KeyCaptureMode::ToggleKey => {
                                    temp_config.switch_key = key_name.clone();
                                }
                                KeyCaptureMode::MappingTrigger(idx) => {
                                    if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                        mapping.trigger_key = key_name.clone();
                                    }
                                }
                                KeyCaptureMode::MappingTarget(idx) => {
                                    if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                        mapping.target_key = key_name.clone();
                                    }
                                }
                                KeyCaptureMode::NewMappingTrigger => {
                                    self.new_mapping_trigger = key_name.clone();
                                }
                                KeyCaptureMode::NewMappingTarget => {
                                    self.new_mapping_target = key_name.clone();
                                }
                                KeyCaptureMode::None => {
                                    //
                                }
                            }
                        }
                        // Exit capture mode
                        self.key_capture_mode = KeyCaptureMode::None;
                        break;
                    }
                }
            });
        }

        let dialog_bg = if self.dark_mode {
            egui::Color32::from_rgb(32, 34, 45)
        } else {
            egui::Color32::from_rgb(245, 240, 252)
        };

        egui::Window::new("")
            .title_bar(false) // Remove default title bar
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 530.0])
            .min_size([600.0, 530.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .id(egui::Id::new("settings_dialog_window")) // Unique ID to avoid conflicts
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(dialog_bg)
                    .corner_radius(egui::CornerRadius::same(20))
                    .stroke(egui::Stroke::NONE)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4],
                        blur: 10,
                        spread: 2,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 40),
                    }),
            )
            .show(ctx, |ui| {
                ui.push_id("settings_dialog_scope", |ui| {
                    // Custom title bar (matching main window style)
                    ui.horizontal(|ui| {
                        ui.add_space(15.0);

                        // Settings title
                        ui.label(
                            egui::RichText::new(t.settings_dialog_title())
                                .size(18.0)
                                .strong()
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(176, 224, 230) // Sky blue
                                } else {
                                    egui::Color32::from_rgb(135, 206, 235) // Deeper sky blue
                                }),
                        );

                        // Push close button to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0);

                            // Close button (matching style)
                            let close_btn =
                                egui::Button::new(egui::RichText::new("x").size(16.0).color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(255, 182, 193)
                                    } else {
                                        egui::Color32::from_rgb(220, 20, 60)
                                    },
                                ))
                                .corner_radius(12.0)
                                .frame(false);

                            if ui.add(close_btn).clicked() {
                                should_cancel = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // Wrap ScrollArea in a frame with padding (like main window)
                    let temp_config = self.temp_config.as_mut().unwrap();
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(8, 0)) // Left and right padding
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(400.0) // Adjusted for new layout
                                .show(ui, |ui| {
                                    // Toggle Key Section
                                    let card_bg = if self.dark_mode {
                                        egui::Color32::from_rgb(35, 42, 50)
                                    } else {
                                        egui::Color32::from_rgb(240, 248, 255)
                                    };

                                    egui::Frame::NONE
                                        .fill(card_bg)
                                        .corner_radius(egui::CornerRadius::same(14))
                                        .inner_margin(egui::Margin::same(14))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(
                                                egui::RichText::new(t.toggle_key())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(100, 200, 255)
                                                    } else {
                                                        egui::Color32::from_rgb(20, 100, 200)
                                                    }),
                                            );
                                            ui.add_space(6.0);

                                            ui.horizontal(|ui| {
                                                ui.label(t.key_label());
                                                ui.add_space(5.0);

                                                let is_capturing = self.key_capture_mode
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
                                                        } else if self.dark_mode {
                                                            egui::Color32::WHITE
                                                        } else {
                                                            egui::Color32::from_rgb(40, 40, 40)
                                                        },
                                                    ),
                                                )
                                                .fill(if is_capturing {
                                                    egui::Color32::from_rgb(70, 130, 180)
                                                } else if self.dark_mode {
                                                    egui::Color32::from_rgb(60, 60, 60)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .corner_radius(10.0); // Increased rounding to match buttons

                                                if ui.add_sized([180.0, 28.0], button).clicked() {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::ToggleKey;
                                                }
                                            });
                                        });

                                    ui.add_space(8.0);

                                    // Global Configuration Section
                                    let card_bg = if self.dark_mode {
                                        egui::Color32::from_rgb(48, 42, 38)
                                    } else {
                                        egui::Color32::from_rgb(255, 248, 240)
                                    };

                                    egui::Frame::NONE
                                        .fill(card_bg)
                                        .corner_radius(egui::CornerRadius::same(14))
                                        .inner_margin(egui::Margin::same(14))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(
                                                egui::RichText::new(t.global_config_title())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(255, 200, 100)
                                                    } else {
                                                        egui::Color32::from_rgb(200, 100, 0)
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

                                                    ui.label(t.input_timeout_label());
                                                    let mut timeout_str =
                                                        temp_config.input_timeout.to_string();
                                                    ui.add_sized(
                                                        [120.0, 24.0],
                                                        egui::TextEdit::singleline(
                                                            &mut timeout_str,
                                                        )
                                                        .background_color(if self.dark_mode {
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
                                                        .background_color(if self.dark_mode {
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
                                                        .background_color(if self.dark_mode {
                                                            egui::Color32::from_rgb(50, 50, 50)
                                                        } else {
                                                            egui::Color32::from_rgb(220, 220, 220)
                                                        }),
                                                    );
                                                    if let Ok(val) = duration_str.parse::<u64>() {
                                                        temp_config.event_duration = val.max(5);
                                                    }
                                                    ui.end_row();

                                                    ui.label(t.worker_count_label());
                                                    let mut worker_str =
                                                        temp_config.worker_count.to_string();
                                                    ui.add_sized(
                                                        [120.0, 24.0],
                                                        egui::TextEdit::singleline(&mut worker_str)
                                                            .hint_text("0 = auto")
                                                            .background_color(if self.dark_mode {
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

                                    ui.add_space(8.0);

                                    // Key Mappings Section
                                    let card_bg = if self.dark_mode {
                                        egui::Color32::from_rgb(35, 45, 40)
                                    } else {
                                        egui::Color32::from_rgb(240, 255, 245)
                                    };

                                    egui::Frame::NONE
                                        .fill(card_bg)
                                        .corner_radius(egui::CornerRadius::same(14))
                                        .inner_margin(egui::Margin::same(14))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(
                                                egui::RichText::new(t.key_mappings_title())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(150, 255, 150)
                                                    } else {
                                                        egui::Color32::from_rgb(0, 150, 0)
                                                    }),
                                            );
                                            ui.add_space(6.0);

                                            // Existing mappings
                                            let mut to_remove = None;
                                            for (idx, mapping) in
                                                temp_config.mappings.iter_mut().enumerate()
                                            {
                                                ui.horizontal(|ui| {
                                                    // Fixed-width label for numbering to ensure alignment
                                                    ui.add_sized(
                                                        [26.0, 24.0],
                                                        egui::Label::new(
                                                            egui::RichText::new(format!(
                                                                "{}.",
                                                                idx + 1
                                                            ))
                                                            .color(if self.dark_mode {
                                                                egui::Color32::from_rgb(
                                                                    200, 200, 200,
                                                                )
                                                            } else {
                                                                egui::Color32::from_rgb(80, 80, 80)
                                                            }),
                                                        ),
                                                    );

                                                    ui.label(t.trigger_short());
                                                    let is_capturing_trigger = self
                                                        .key_capture_mode
                                                        == KeyCaptureMode::MappingTrigger(idx);
                                                    let trigger_text = if is_capturing_trigger {
                                                        "⌨..."
                                                    } else {
                                                        &mapping.trigger_key
                                                    };
                                                    let trigger_btn = egui::Button::new(
                                                        egui::RichText::new(trigger_text).color(
                                                            if is_capturing_trigger {
                                                                egui::Color32::from_rgb(255, 200, 0)
                                                            } else if self.dark_mode {
                                                                egui::Color32::WHITE
                                                            } else {
                                                                egui::Color32::from_rgb(40, 40, 40)
                                                            },
                                                        ),
                                                    )
                                                    .fill(if is_capturing_trigger {
                                                        egui::Color32::from_rgb(70, 130, 180)
                                                    } else if self.dark_mode {
                                                        egui::Color32::from_rgb(50, 50, 50)
                                                    } else {
                                                        egui::Color32::from_rgb(220, 220, 220)
                                                    })
                                                    .corner_radius(4.0);
                                                    if ui
                                                        .add_sized([80.0, 24.0], trigger_btn)
                                                        .clicked()
                                                    {
                                                        self.key_capture_mode =
                                                            KeyCaptureMode::MappingTrigger(idx);
                                                    }

                                                    ui.label(t.target_short());
                                                    let is_capturing_target = self.key_capture_mode
                                                        == KeyCaptureMode::MappingTarget(idx);
                                                    let target_text = if is_capturing_target {
                                                        "⌨..."
                                                    } else {
                                                        &mapping.target_key
                                                    };
                                                    let target_btn = egui::Button::new(
                                                        egui::RichText::new(target_text).color(
                                                            if is_capturing_target {
                                                                egui::Color32::from_rgb(255, 200, 0)
                                                            } else if self.dark_mode {
                                                                egui::Color32::WHITE
                                                            } else {
                                                                egui::Color32::from_rgb(40, 40, 40)
                                                            },
                                                        ),
                                                    )
                                                    .fill(if is_capturing_target {
                                                        egui::Color32::from_rgb(70, 130, 180)
                                                    } else if self.dark_mode {
                                                        egui::Color32::from_rgb(50, 50, 50)
                                                    } else {
                                                        egui::Color32::from_rgb(220, 220, 220)
                                                    })
                                                    .corner_radius(4.0);
                                                    if ui
                                                        .add_sized([80.0, 24.0], target_btn)
                                                        .clicked()
                                                    {
                                                        self.key_capture_mode =
                                                            KeyCaptureMode::MappingTarget(idx);
                                                    }

                                                    ui.label(t.interval_short());
                                                    let mut interval_str = mapping
                                                        .interval
                                                        .unwrap_or(temp_config.interval)
                                                        .to_string();

                                                    let interval_edit = egui::TextEdit::singleline(
                                                        &mut interval_str,
                                                    )
                                                    .background_color(if self.dark_mode {
                                                        egui::Color32::from_rgb(50, 50, 50)
                                                    } else {
                                                        egui::Color32::from_rgb(220, 220, 220)
                                                    })
                                                    .desired_width(45.0) // Shorter width
                                                    .font(egui::TextStyle::Button); // Match button style

                                                    if ui
                                                        .add_sized([45.0, 24.0], interval_edit)
                                                        .changed()
                                                        && let Ok(val) = interval_str.parse::<u64>()
                                                    {
                                                        mapping.interval = Some(val.max(5));
                                                    }

                                                    ui.label(t.duration_short());
                                                    let mut duration_str = mapping
                                                        .event_duration
                                                        .unwrap_or(temp_config.event_duration)
                                                        .to_string();

                                                    let duration_edit = egui::TextEdit::singleline(
                                                        &mut duration_str,
                                                    )
                                                    .background_color(if self.dark_mode {
                                                        egui::Color32::from_rgb(50, 50, 50)
                                                    } else {
                                                        egui::Color32::from_rgb(220, 220, 220)
                                                    })
                                                    .desired_width(45.0) // Shorter width
                                                    .font(egui::TextStyle::Button); // Match button style

                                                    if ui
                                                        .add_sized([45.0, 24.0], duration_edit)
                                                        .changed()
                                                        && let Ok(val) = duration_str.parse::<u64>()
                                                    {
                                                        mapping.event_duration = Some(val.max(5));
                                                    }

                                                    let delete_btn = egui::Button::new(
                                                        egui::RichText::new("🗑")
                                                            .color(egui::Color32::WHITE),
                                                    )
                                                    .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink (anime style)
                                                    .corner_radius(10.0); // Larger rounding for anime style

                                                    if ui
                                                        .add_sized([32.0, 24.0], delete_btn)
                                                        .clicked()
                                                    {
                                                        to_remove = Some(idx);
                                                    }
                                                });
                                                ui.add_space(4.0);
                                            }

                                            if let Some(idx) = to_remove {
                                                temp_config.mappings.remove(idx);
                                            }

                                            ui.add_space(8.0);
                                            ui.separator();
                                            ui.add_space(8.0);

                                            // Add new mapping
                                            ui.label(
                                                egui::RichText::new(t.add_new_mapping_title())
                                                    .size(14.0)
                                                    .strong(),
                                            );
                                            ui.add_space(5.0);

                                            ui.horizontal(|ui| {
                                                ui.label(t.trigger_short());
                                                let is_capturing_new_trigger = self
                                                    .key_capture_mode
                                                    == KeyCaptureMode::NewMappingTrigger;
                                                let new_trigger_text = if is_capturing_new_trigger {
                                                    t.press_any_key()
                                                } else if self.new_mapping_trigger.is_empty() {
                                                    t.click_text()
                                                } else {
                                                    &self.new_mapping_trigger
                                                };
                                                let new_trigger_btn = egui::Button::new(
                                                    egui::RichText::new(new_trigger_text).color(
                                                        if is_capturing_new_trigger {
                                                            egui::Color32::from_rgb(255, 200, 0)
                                                        } else if self.dark_mode {
                                                            egui::Color32::WHITE
                                                        } else {
                                                            egui::Color32::from_rgb(40, 40, 40)
                                                        },
                                                    ),
                                                )
                                                .fill(if is_capturing_new_trigger {
                                                    egui::Color32::from_rgb(70, 130, 180)
                                                } else if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .corner_radius(4.0);
                                                if ui
                                                    .add_sized([80.0, 24.0], new_trigger_btn)
                                                    .clicked()
                                                {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::NewMappingTrigger;
                                                    // Clear error when user starts to modify trigger
                                                    self.duplicate_mapping_error = None;
                                                }

                                                ui.label(t.target_short());
                                                let is_capturing_new_target = self.key_capture_mode
                                                    == KeyCaptureMode::NewMappingTarget;
                                                let new_target_text = if is_capturing_new_target {
                                                    t.press_any_key()
                                                } else if self.new_mapping_target.is_empty() {
                                                    t.click_text()
                                                } else {
                                                    &self.new_mapping_target
                                                };
                                                let new_target_btn = egui::Button::new(
                                                    egui::RichText::new(new_target_text).color(
                                                        if is_capturing_new_target {
                                                            egui::Color32::from_rgb(255, 200, 0)
                                                        } else if self.dark_mode {
                                                            egui::Color32::WHITE
                                                        } else {
                                                            egui::Color32::from_rgb(40, 40, 40)
                                                        },
                                                    ),
                                                )
                                                .fill(if is_capturing_new_target {
                                                    egui::Color32::from_rgb(70, 130, 180)
                                                } else if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .corner_radius(4.0);
                                                if ui
                                                    .add_sized([80.0, 24.0], new_target_btn)
                                                    .clicked()
                                                {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::NewMappingTarget;
                                                }

                                                ui.label(t.interval_short());
                                                let interval_edit = egui::TextEdit::singleline(
                                                    &mut self.new_mapping_interval,
                                                )
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .hint_text("5")
                                                .desired_width(45.0)
                                                .font(egui::TextStyle::Button);
                                                ui.add_sized([45.0, 24.0], interval_edit);

                                                ui.label(t.duration_short());
                                                let duration_edit = egui::TextEdit::singleline(
                                                    &mut self.new_mapping_duration,
                                                )
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .hint_text("5")
                                                .desired_width(45.0)
                                                .font(egui::TextStyle::Button);
                                                ui.add_sized([45.0, 24.0], duration_edit);

                                                let add_btn = egui::Button::new(
                                                    egui::RichText::new(t.add_button_text())
                                                        .color(egui::Color32::WHITE)
                                                        .strong(),
                                                )
                                                .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green (anime style)
                                                .corner_radius(10.0); // Larger rounding for anime style

                                                if ui.add_sized([70.0, 24.0], add_btn).clicked()
                                                    && !self.new_mapping_trigger.is_empty()
                                                    && !self.new_mapping_target.is_empty()
                                                {
                                                    let trigger_upper =
                                                        self.new_mapping_trigger.to_uppercase();

                                                    // Check for duplicate trigger key
                                                    let is_duplicate = temp_config
                                                        .mappings
                                                        .iter()
                                                        .any(|m| m.trigger_key == trigger_upper);

                                                    if is_duplicate {
                                                        self.duplicate_mapping_error = Some(
                                                            t.duplicate_trigger_error().to_string(),
                                                        );
                                                    } else {
                                                        // Clear any previous error
                                                        self.duplicate_mapping_error = None;

                                                        let interval = self
                                                            .new_mapping_interval
                                                            .parse::<u64>()
                                                            .ok()
                                                            .map(|v| v.max(5));
                                                        let duration = self
                                                            .new_mapping_duration
                                                            .parse::<u64>()
                                                            .ok()
                                                            .map(|v| v.max(5));

                                                        temp_config.mappings.push(KeyMapping {
                                                            trigger_key: trigger_upper,
                                                            target_key: self
                                                                .new_mapping_target
                                                                .to_uppercase(),
                                                            interval,
                                                            event_duration: duration,
                                                        });

                                                        // Clear input fields
                                                        self.new_mapping_trigger.clear();
                                                        self.new_mapping_target.clear();
                                                        self.new_mapping_interval.clear();
                                                        self.new_mapping_duration.clear();
                                                    }
                                                }
                                            });

                                            // Display duplicate trigger error if exists
                                            if let Some(ref error_msg) =
                                                self.duplicate_mapping_error
                                            {
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

                                    // Process Whitelist Section
                                    let card_bg = if self.dark_mode {
                                        egui::Color32::from_rgb(45, 35, 50)
                                    } else {
                                        egui::Color32::from_rgb(255, 245, 250)
                                    };

                                    egui::Frame::NONE
                                        .fill(card_bg)
                                        .corner_radius(egui::CornerRadius::same(14))
                                        .inner_margin(egui::Margin::same(14))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(
                                                egui::RichText::new(t.process_whitelist_hint())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(186, 149, 230) // Soft purple
                                                    } else {
                                                        egui::Color32::from_rgb(100, 50, 150) // Darker purple for contrast
                                                    }),
                                            );
                                            ui.add_space(6.0);

                                            // Process list
                                            egui::ScrollArea::vertical().max_height(80.0).show(
                                                ui,
                                                |ui| {
                                                    let mut to_remove: Option<usize> = None;
                                                    for (idx, process) in temp_config
                                                        .process_whitelist
                                                        .iter()
                                                        .enumerate()
                                                    {
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(process)
                                                                    .size(13.0)
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(
                                                                            200, 200, 255,
                                                                        )
                                                                    } else {
                                                                        egui::Color32::from_rgb(
                                                                            60, 60, 120,
                                                                        )
                                                                    }),
                                                            );

                                                            ui.with_layout(
                                                                egui::Layout::right_to_left(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    let del_btn = egui::Button::new(
                                                                    egui::RichText::new("🗑")
                                                                        .color(egui::Color32::WHITE)
                                                                        .size(11.0),
                                                                )
                                                                .fill(egui::Color32::from_rgb(
                                                                    255, 182, 193,
                                                                )) // Soft pink
                                                                .corner_radius(8.0);

                                                                    if ui
                                                                        .add_sized(
                                                                            [24.0, 20.0],
                                                                            del_btn,
                                                                        )
                                                                        .clicked()
                                                                    {
                                                                        to_remove = Some(idx);
                                                                    }
                                                                },
                                                            );
                                                        });
                                                    }

                                                    if let Some(idx) = to_remove {
                                                        temp_config.process_whitelist.remove(idx);
                                                    }
                                                },
                                            );

                                            ui.add_space(6.0);

                                            // Add new process
                                            ui.horizontal(|ui| {
                                                let process_edit = egui::TextEdit::singleline(
                                                    &mut self.new_process_name,
                                                )
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .hint_text(t.process_example())
                                                .desired_width(200.0);
                                                ui.add(process_edit);

                                                let add_btn = egui::Button::new(
                                                    egui::RichText::new(t.add_button_text())
                                                        .color(egui::Color32::WHITE)
                                                        .size(12.0)
                                                        .strong(),
                                                )
                                                .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green
                                                .corner_radius(10.0);

                                                if ui.add_sized([70.0, 24.0], add_btn).clicked() {
                                                    let process_name = self.new_process_name.trim();
                                                    if !process_name.is_empty() {
                                                        // Check for duplicate process
                                                        if temp_config
                                                            .process_whitelist
                                                            .contains(&process_name.to_string())
                                                        {
                                                            self.duplicate_process_error = Some(
                                                                t.duplicate_process_error()
                                                                    .to_string(),
                                                            );
                                                        } else {
                                                            // Clear any previous error
                                                            self.duplicate_process_error = None;
                                                            temp_config
                                                                .process_whitelist
                                                                .push(process_name.to_string());
                                                            self.new_process_name.clear();
                                                        }
                                                    }
                                                }

                                                ui.add_space(8.0);

                                                // Browse button for selecting process
                                                let browse_btn = egui::Button::new(
                                                    egui::RichText::new(t.browse_button())
                                                        .color(egui::Color32::WHITE)
                                                        .size(12.0)
                                                        .strong(),
                                                )
                                                .fill(egui::Color32::from_rgb(135, 206, 235)) // Sky blue
                                                .corner_radius(10.0);

                                                if ui.add_sized([85.0, 24.0], browse_btn).clicked()
                                                {
                                                    // Open file dialog to select executable
                                                    if let Some(path) = rfd::FileDialog::new()
                                                        .add_filter("Executable", &["exe"])
                                                        .set_title("Select Process")
                                                        .pick_file()
                                                        && let Some(filename) = path.file_name()
                                                    {
                                                        let process_name =
                                                            filename.to_string_lossy().to_string();
                                                        // Check for duplicate process
                                                        if temp_config
                                                            .process_whitelist
                                                            .contains(&process_name)
                                                        {
                                                            self.duplicate_process_error = Some(
                                                                t.duplicate_process_error()
                                                                    .to_string(),
                                                            );
                                                        } else {
                                                            // Clear any previous error
                                                            self.duplicate_process_error = None;
                                                            temp_config
                                                                .process_whitelist
                                                                .push(process_name);
                                                        }
                                                    }
                                                }
                                            });

                                            // Display duplicate process error if exists
                                            if let Some(ref error_msg) =
                                                self.duplicate_process_error
                                            {
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
                                }); // End of ScrollArea
                        }); // End of Frame

                    ui.separator();

                    // Action buttons - centered (outside ScrollArea, fixed at bottom)
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            // Calculate total width of buttons and spacing
                            let button_width = 240.0;
                            let spacing = 15.0;
                            let total_buttons_width = button_width * 2.0 + spacing;
                            let available_width = ui.available_width();

                            // Add left padding to center the buttons
                            if available_width > total_buttons_width {
                                ui.add_space((available_width - total_buttons_width) / 2.0);
                            }

                            let save_btn = egui::Button::new(
                                egui::RichText::new(t.save())
                                    .size(14.0) // Slightly smaller for consistency
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green (anime style)
                            .corner_radius(15.0); // Larger rounding for anime style

                            if ui.add_sized([button_width, 32.0], save_btn).clicked() {
                                should_save = true;
                            }

                            ui.add_space(spacing);

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(t.cancel())
                                    .size(14.0)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink (anime style)
                            .corner_radius(15.0); // Larger rounding for anime style

                            if ui.add_sized([button_width, 32.0], cancel_btn).clicked() {
                                should_cancel = true;
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // Hint
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(t.changes_take_effect_hint())
                                .size(12.0)
                                .color(egui::Color32::from_rgb(100, 220, 100))
                                .italics(),
                        );
                    });
                }); // End of ui.push_id
            }); // End of egui::Window

        // Handle save/cancel outside the window closure to avoid borrow conflicts
        if should_save {
            if let Some(temp_config) = &self.temp_config {
                // Check if always_on_top changed
                let always_on_top_changed = temp_config.always_on_top != self.config.always_on_top;
                // Check if dark_mode changed
                let dark_mode_changed = temp_config.dark_mode != self.config.dark_mode;
                // Check if language changed
                let language_changed = temp_config.language != self.config.language;

                // Save to file
                if temp_config.save_to_file("Config.toml").is_ok() {
                    // Reload configuration into AppState (takes effect immediately)
                    let _ = self.app_state.reload_config(temp_config.clone());

                    // Update GUI's config
                    self.config = temp_config.clone();

                    // Apply theme change immediately
                    if dark_mode_changed {
                        self.dark_mode = self.config.dark_mode;
                    }

                    if language_changed {
                        self.update_translations(self.config.language);
                        use crate::gui::fonts;
                        fonts::load_fonts(ctx, self.config.language);
                    }

                    // Apply always_on_top change immediately
                    if always_on_top_changed {
                        if self.config.always_on_top {
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::AlwaysOnTop,
                            ));
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::Normal,
                            ));
                        }
                    }
                }
            }
            self.show_settings_dialog = false;
            self.temp_config = None;
            self.key_capture_mode = KeyCaptureMode::None;
            self.duplicate_mapping_error = None;
            self.duplicate_process_error = None;

            // Restore previous paused state after exiting settings
            if let Some(was_paused) = self.was_paused_before_settings.take()
                && !was_paused
            {
                // Resume key repeat without notification (silent resume)
                self.app_state.set_paused(false);
            }
        }

        if should_cancel {
            self.show_settings_dialog = false;
            self.temp_config = None;
            self.key_capture_mode = KeyCaptureMode::None;
            self.duplicate_mapping_error = None;
            self.duplicate_process_error = None;
            // Clear input fields
            self.new_mapping_trigger.clear();
            self.new_mapping_target.clear();
            self.new_mapping_interval.clear();
            self.new_mapping_duration.clear();

            // Restore previous paused state after exiting settings
            if let Some(was_paused) = self.was_paused_before_settings.take()
                && !was_paused
            {
                // Resume key repeat without notification (silent resume)
                self.app_state.set_paused(false);
            }
        }
    }
}
