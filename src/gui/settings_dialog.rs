//! Settings dialog implementation.

use crate::config::KeyMapping;
use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;
use eframe::egui;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

const TEXT_TRUNCATE_LEN: usize = 9;

/// Safely truncate a string by character count (not byte count) to avoid UTF-8 boundary issues.
fn truncate_text_safe(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count > max_chars {
        let truncated: String = text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}

/// Check if target key is a mouse movement action.
fn is_mouse_move_target(target: &str) -> bool {
    let upper = target.to_uppercase();
    matches!(
        upper.as_str(),
        "MOUSE_UP"
            | "MOUSE_DOWN"
            | "MOUSE_LEFT"
            | "MOUSE_RIGHT"
            | "MOUSE_UP_LEFT"
            | "MOUSE_UP_RIGHT"
            | "MOUSE_DOWN_LEFT"
            | "MOUSE_DOWN_RIGHT"
            | "MOUSEUP"
            | "MOUSEDOWN"
            | "MOUSELEFT"
            | "MOUSERIGHT"
            | "MOVE_UP"
            | "MOVE_DOWN"
            | "MOVE_LEFT"
            | "MOVE_RIGHT"
            | "M_UP"
            | "M_DOWN"
            | "M_LEFT"
            | "M_RIGHT"
            | "MOUSEUPLEFT"
            | "MOUSEUPRIGHT"
            | "MOUSEDOWNLEFT"
            | "MOUSEDOWNRIGHT"
            | "M_UP_LEFT"
            | "M_UP_RIGHT"
            | "M_DOWN_LEFT"
            | "M_DOWN_RIGHT"
    )
}

impl SorahkGui {
    /// Renders the settings dialog for configuration management.
    pub(super) fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_cancel = false;

        let t = &self.translations;

        // Handle key and mouse capture if in capture mode
        // Priority: Keyboard > Mouse > Raw Input (gamepad/joystick)
        if self.key_capture_mode != KeyCaptureMode::None {
            let mut captured_input: Option<String> = None;

            // Check keyboard input first
            let current_pressed = Self::poll_all_pressed_keys();

            // Filter out keys that were already pressed when capture started (noise baseline)
            let new_pressed: std::collections::HashSet<u32> = current_pressed
                .difference(&self.capture_initial_pressed)
                .copied()
                .collect();

            // Update tracking set with newly pressed keys only
            for vk in &new_pressed {
                self.capture_pressed_keys.insert(*vk);
            }

            // Check if any tracked key has been released
            let any_released = self
                .capture_pressed_keys
                .iter()
                .any(|vk| !current_pressed.contains(vk));

            if any_released && !self.capture_pressed_keys.is_empty() {
                // Capture complete: format the result
                captured_input = Self::format_captured_keys(&self.capture_pressed_keys);
            }

            // Check mouse button input ONLY if keyboard not captured
            // Skip mouse capture on the first frame after entering capture mode (just_captured_input flag)
            // to avoid capturing the click on the "Capture" button itself
            if captured_input.is_none() && !self.just_captured_input {
                ctx.input(|i| {
                    if i.pointer.button_clicked(egui::PointerButton::Primary) {
                        captured_input = Some("LBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Secondary) {
                        captured_input = Some("RBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Middle) {
                        captured_input = Some("MBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Extra1) {
                        captured_input = Some("XBUTTON1".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Extra2) {
                        captured_input = Some("XBUTTON2".to_string());
                    }
                });
            }

            // Check Raw Input (gamepad/joystick) ONLY if neither keyboard nor mouse captured
            if captured_input.is_none() {
                let should_check_raw_input = matches!(
                    self.key_capture_mode,
                    KeyCaptureMode::ToggleKey
                        | KeyCaptureMode::MappingTrigger(_)
                        | KeyCaptureMode::NewMappingTrigger
                );

                if should_check_raw_input
                    && let Some(device) = self.app_state.try_recv_raw_input_capture()
                {
                    captured_input = Some(device.to_string());
                }
            }

            if let Some(input_name) = captured_input {
                // Update the appropriate field
                if let Some(temp_config) = &mut self.temp_config {
                    match self.key_capture_mode {
                        KeyCaptureMode::ToggleKey => {
                            temp_config.switch_key = input_name.clone();
                        }
                        KeyCaptureMode::MappingTrigger(idx) => {
                            if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                mapping.trigger_key = input_name.clone();
                            }
                        }
                        KeyCaptureMode::MappingTarget(idx) => {
                            if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                mapping.target_key = input_name.clone();
                            }
                        }
                        KeyCaptureMode::NewMappingTrigger => {
                            self.new_mapping_trigger = input_name.clone();
                        }
                        KeyCaptureMode::NewMappingTarget => {
                            self.new_mapping_target = input_name.clone();
                        }
                        KeyCaptureMode::None => {}
                    }
                }
                // Exit capture mode and clear capture state
                self.key_capture_mode = KeyCaptureMode::None;
                // Flag will be cleared after capture completes, used to ignore button click on next frame
                self.just_captured_input = true;
                self.capture_pressed_keys.clear();
                self.app_state.set_raw_input_capture_mode(false);
            } else {
                // In capture mode but no input captured yet - clear the flag after first frame
                if self.just_captured_input {
                    self.just_captured_input = false;
                }
            }
        } else {
            // Not in capture mode: ensure state is clean
            self.capture_pressed_keys.clear();
            // Clear flag when not in capture mode
            if self.just_captured_input {
                self.just_captured_input = false;
            }
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

                                                if ui.add_sized([180.0, 28.0], button).clicked()
                                                    && !self.just_captured_input
                                                {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::ToggleKey;
                                                    self.capture_pressed_keys.clear();
                                                    self.capture_initial_pressed =
                                                        Self::poll_all_pressed_keys();
                                                    self.app_state.set_raw_input_capture_mode(true);
                                                    // Set flag to skip mouse capture on this frame
                                                    self.just_captured_input = true;
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
                                                        temp_config.event_duration = val.max(2);
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
                                                    let full_trigger_text = &mapping.trigger_key;
                                                    let trigger_text = if is_capturing_trigger {
                                                        "⌨..."
                                                    } else {
                                                        full_trigger_text
                                                    };
                                                    // Truncate text to fit in button
                                                    let display_text = truncate_text_safe(
                                                        trigger_text,
                                                        TEXT_TRUNCATE_LEN,
                                                    );
                                                    // Create button with truncated text
                                                    let trigger_btn = egui::Button::new(
                                                        egui::RichText::new(&display_text).color(
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
                                                    let mut response =
                                                        ui.add_sized([80.0, 24.0], trigger_btn);
                                                    // Show full text on hover if truncated
                                                    if !is_capturing_trigger
                                                        && trigger_text.len() > TEXT_TRUNCATE_LEN
                                                    {
                                                        response =
                                                            response.on_hover_text(trigger_text);
                                                    }
                                                    if response.clicked()
                                                        && !self.just_captured_input
                                                    {
                                                        self.key_capture_mode =
                                                            KeyCaptureMode::MappingTrigger(idx);
                                                        self.capture_pressed_keys.clear();
                                                        self.capture_initial_pressed =
                                                            Self::poll_all_pressed_keys();
                                                        self.app_state
                                                            .set_raw_input_capture_mode(true);
                                                        // Set flag to skip mouse capture on this frame
                                                        self.just_captured_input = true;
                                                    }

                                                    ui.label(t.target_short());
                                                    let is_capturing_target = self.key_capture_mode
                                                        == KeyCaptureMode::MappingTarget(idx);
                                                    let target_text = if is_capturing_target {
                                                        "⌨..."
                                                    } else {
                                                        &mapping.target_key
                                                    };
                                                    let display_target_text = if is_capturing_target {
                                                        target_text.to_string()
                                                    } else {
                                                        truncate_text_safe(
                                                            target_text,
                                                            TEXT_TRUNCATE_LEN,
                                                        )
                                                    };
                                                    let target_btn = egui::Button::new(
                                                        egui::RichText::new(&display_target_text).color(
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
                                                    let mut target_response = ui
                                                        .add_sized([80.0, 24.0], target_btn);
                                                    if !is_capturing_target
                                                        && target_text.len() > TEXT_TRUNCATE_LEN
                                                    {
                                                        target_response =
                                                            target_response.on_hover_text(target_text);
                                                    }
                                                    if target_response.clicked()
                                                        && !self.just_captured_input
                                                    {
                                                        self.key_capture_mode =
                                                            KeyCaptureMode::MappingTarget(idx);
                                                        self.capture_pressed_keys.clear();
                                                        self.capture_initial_pressed =
                                                            Self::poll_all_pressed_keys();
                                                    }

                                                    // Check if this mapping is for mouse movement
                                                    let is_mouse_move_mapping = is_mouse_move_target(&mapping.target_key);

                                                    if is_mouse_move_mapping {
                                                        // Show move speed for mouse movement
                                                        ui.label(t.mouse_move_speed_label());
                                                        let mut speed_str = mapping
                                                            .move_speed
                                                            .to_string();

                                                        let speed_edit = egui::TextEdit::singleline(
                                                            &mut speed_str,
                                                        )
                                                        .background_color(if self.dark_mode {
                                                            egui::Color32::from_rgb(50, 50, 50)
                                                        } else {
                                                            egui::Color32::from_rgb(220, 220, 220)
                                                        })
                                                        .desired_width(45.0)
                                                        .font(egui::TextStyle::Button);

                                                        if ui
                                                            .add_sized([75.0, 24.0], speed_edit)
                                                            .changed()
                                                            && let Ok(val) = speed_str.parse::<i32>()
                                                        {
                                                            mapping.move_speed = val.clamp(1, 100);
                                                        }
                                                    } else {
                                                        // Show interval and duration for regular keys
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
                                                        .desired_width(45.0)
                                                        .font(egui::TextStyle::Button);

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
                                                        .desired_width(45.0)
                                                        .font(egui::TextStyle::Button);

                                                        if ui
                                                            .add_sized([45.0, 24.0], duration_edit)
                                                            .changed()
                                                            && let Ok(val) = duration_str.parse::<u64>()
                                                        {
                                                            mapping.event_duration = Some(val.max(2));
                                                        }
                                                    }

                                                    // Mouse direction selector button
                                                    let mouse_btn = egui::Button::new(
                                                        egui::RichText::new("🖱")
                                                            .color(egui::Color32::WHITE)
                                                            .size(16.0),
                                                    )
                                                    .fill(if self.dark_mode {
                                                        egui::Color32::from_rgb(177, 156, 217) // Soft lavender
                                                    } else {
                                                        egui::Color32::from_rgb(216, 191, 216) // Pale thistle
                                                    })
                                                    .corner_radius(12.0);

                                                    if ui
                                                        .add_sized([32.0, 24.0], mouse_btn)
                                                        .on_hover_text(t.set_mouse_direction_hover())
                                                        .clicked()
                                                    {
                                                        self.mouse_direction_dialog = Some(
                                                            crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                                                        );
                                                        self.mouse_direction_mapping_idx = Some(idx);
                                                    }

                                                    // Turbo toggle
                                                    let turbo_enabled = mapping.turbo_enabled;
                                                    let turbo_color = if turbo_enabled {
                                                        if self.dark_mode {
                                                            egui::Color32::from_rgb(147, 197, 253)
                                                        } else {
                                                            egui::Color32::from_rgb(96, 165, 250)
                                                        }
                                                    } else if self.dark_mode {
                                                        egui::Color32::from_rgb(75, 85, 99)
                                                    } else {
                                                        egui::Color32::from_rgb(156, 163, 175)
                                                    };

                                                    let turbo_icon =
                                                        if turbo_enabled { "⚡" } else { "○" };
                                                    let turbo_btn = egui::Button::new(
                                                        egui::RichText::new(turbo_icon)
                                                            .color(egui::Color32::WHITE)
                                                            .size(16.0),
                                                    )
                                                    .fill(turbo_color)
                                                    .corner_radius(12.0)
                                                    .sense(egui::Sense::click());

                                                    let hover_text = if turbo_enabled {
                                                        self.translations.turbo_on_hover()
                                                    } else {
                                                        self.translations.turbo_off_hover()
                                                    };

                                                    if ui
                                                        .add_sized([32.0, 24.0], turbo_btn)
                                                        .on_hover_text(hover_text)
                                                        .clicked()
                                                    {
                                                        mapping.turbo_enabled =
                                                            !mapping.turbo_enabled;
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
                                                let full_new_trigger_text =
                                                    if is_capturing_new_trigger {
                                                        t.press_any_key()
                                                    } else if self.new_mapping_trigger.is_empty() {
                                                        t.click_text()
                                                    } else {
                                                        &self.new_mapping_trigger
                                                    };
                                                // Truncate text to fit in button
                                                let new_display_text = truncate_text_safe(
                                                    full_new_trigger_text,
                                                    TEXT_TRUNCATE_LEN,
                                                );
                                                let new_trigger_btn = egui::Button::new(
                                                    egui::RichText::new(&new_display_text).color(
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
                                                let mut new_trigger_response =
                                                    ui.add_sized([80.0, 24.0], new_trigger_btn);
                                                // Show full text on hover if truncated
                                                if !is_capturing_new_trigger
                                                    && !self.new_mapping_trigger.is_empty()
                                                    && full_new_trigger_text.len()
                                                        > TEXT_TRUNCATE_LEN
                                                {
                                                    new_trigger_response = new_trigger_response
                                                        .on_hover_text(full_new_trigger_text);
                                                }
                                                if new_trigger_response.clicked()
                                                    && !self.just_captured_input
                                                {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::NewMappingTrigger;
                                                    self.capture_pressed_keys.clear();
                                                    self.capture_initial_pressed =
                                                        Self::poll_all_pressed_keys();
                                                    self.app_state.set_raw_input_capture_mode(true);
                                                    // Set flag to skip mouse capture on this frame
                                                    self.just_captured_input = true;
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
                                                let display_new_target_text = if is_capturing_new_target
                                                    || self.new_mapping_target.is_empty()
                                                {
                                                    new_target_text.to_string()
                                                } else {
                                                    truncate_text_safe(
                                                        new_target_text,
                                                        TEXT_TRUNCATE_LEN,
                                                    )
                                                };
                                                let new_target_btn = egui::Button::new(
                                                    egui::RichText::new(&display_new_target_text).color(
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
                                                let mut new_target_response = ui
                                                    .add_sized([80.0, 24.0], new_target_btn);
                                                if !is_capturing_new_target
                                                    && !self.new_mapping_target.is_empty()
                                                    && new_target_text.len() > TEXT_TRUNCATE_LEN
                                                {
                                                    new_target_response = new_target_response
                                                        .on_hover_text(new_target_text);
                                                }
                                                if new_target_response.clicked()
                                                    && !self.just_captured_input
                                                {
                                                    self.key_capture_mode =
                                                        KeyCaptureMode::NewMappingTarget;
                                                    self.capture_pressed_keys.clear();
                                                    self.capture_initial_pressed =
                                                        Self::poll_all_pressed_keys();
                                                }

                                                // Check if target is mouse movement
                                                let is_mouse_move = is_mouse_move_target(&self.new_mapping_target);

                                                if is_mouse_move {
                                                    // Show move speed for mouse movement
                                                    ui.label(t.mouse_move_speed_label());
                                                    let speed_edit = egui::TextEdit::singleline(
                                                        &mut self.new_mapping_move_speed,
                                                    )
                                                    .background_color(if self.dark_mode {
                                                        egui::Color32::from_rgb(50, 50, 50)
                                                    } else {
                                                        egui::Color32::from_rgb(220, 220, 220)
                                                    })
                                                    .hint_text("10")
                                                    .desired_width(45.0)
                                                    .font(egui::TextStyle::Button);
                                                    ui.add_sized([45.0, 24.0], speed_edit);
                                                } else {
                                                    // Show interval and duration for regular keys
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
                                                }

                                                // Mouse direction selector button for new mapping
                                                let new_mouse_btn = egui::Button::new(
                                                    egui::RichText::new("🖱")
                                                        .color(egui::Color32::WHITE)
                                                        .size(16.0),
                                                )
                                                .fill(if self.dark_mode {
                                                    egui::Color32::from_rgb(177, 156, 217) // Soft lavender
                                                } else {
                                                    egui::Color32::from_rgb(216, 191, 216) // Pale thistle
                                                })
                                                .corner_radius(12.0);

                                                if ui
                                                    .add_sized([32.0, 24.0], new_mouse_btn)
                                                    .on_hover_text(t.set_mouse_direction_hover())
                                                    .clicked()
                                                {
                                                    self.mouse_direction_dialog = Some(
                                                        crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                                                    );
                                                    self.mouse_direction_mapping_idx = None;
                                                }

                                                // Turbo toggle for new mapping
                                                let new_turbo_enabled = self.new_mapping_turbo;
                                                let new_turbo_color = if new_turbo_enabled {
                                                    if self.dark_mode {
                                                        egui::Color32::from_rgb(147, 197, 253)
                                                    } else {
                                                        egui::Color32::from_rgb(96, 165, 250)
                                                    }
                                                } else if self.dark_mode {
                                                    egui::Color32::from_rgb(75, 85, 99)
                                                } else {
                                                    egui::Color32::from_rgb(156, 163, 175)
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
                                                    self.translations.turbo_on_hover()
                                                } else {
                                                    self.translations.turbo_off_hover()
                                                };

                                                if ui
                                                    .add_sized([32.0, 24.0], new_turbo_btn)
                                                    .on_hover_text(new_hover_text)
                                                    .clicked()
                                                {
                                                    self.new_mapping_turbo =
                                                        !self.new_mapping_turbo;
                                                }

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
                                                            .map(|v| v.max(2));
                                                        let move_speed = self
                                                            .new_mapping_move_speed
                                                            .parse::<i32>()
                                                            .unwrap_or(10)
                                                            .clamp(1, 100);

                                                        let turbo_enabled = self.new_mapping_turbo;

                                                        temp_config.mappings.push(KeyMapping {
                                                            trigger_key: trigger_upper,
                                                            target_key: self
                                                                .new_mapping_target
                                                                .to_uppercase(),
                                                            interval,
                                                            event_duration: duration,
                                                            turbo_enabled,
                                                            move_speed,
                                                        });

                                                        // Clear input fields
                                                        self.new_mapping_trigger.clear();
                                                        self.new_mapping_target.clear();
                                                        self.new_mapping_interval.clear();
                                                        self.new_mapping_duration.clear();
                                                        self.new_mapping_move_speed = "10".to_string();
                                                        self.new_mapping_turbo = true; // Reset to default
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

        // Handle save/cancel outside the window closure
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
            self.app_state.set_raw_input_capture_mode(false);

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
            self.app_state.set_raw_input_capture_mode(false);
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

    /// Polls all currently pressed keys using Windows API
    /// Returns a set of VK codes
    #[inline]
    fn poll_all_pressed_keys() -> std::collections::HashSet<u32> {
        let mut pressed_keys = std::collections::HashSet::new();

        unsafe {
            // Modifiers
            let modifiers = [
                0xA0, 0xA1, // LSHIFT, RSHIFT
                0xA2, 0xA3, // LCTRL, RCTRL
                0xA4, 0xA5, // LALT, RALT
                0x5B, 0x5C, // LWIN, RWIN
            ];
            for &vk in &modifiers {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            // A-Z (0x41-0x5A)
            for vk in 0x41..=0x5A {
                if GetAsyncKeyState(vk) < 0 {
                    pressed_keys.insert(vk as u32);
                }
            }

            // 0-9 (0x30-0x39)
            for vk in 0x30..=0x39 {
                if GetAsyncKeyState(vk) < 0 {
                    pressed_keys.insert(vk as u32);
                }
            }

            // Numpad 0-9 (0x60-0x69)
            for vk in 0x60..=0x69 {
                if GetAsyncKeyState(vk) < 0 {
                    pressed_keys.insert(vk as u32);
                }
            }

            // F1-F24 (0x70-0x87)
            for vk in 0x70..=0x87 {
                if GetAsyncKeyState(vk) < 0 {
                    pressed_keys.insert(vk as u32);
                }
            }

            // Navigation and editing keys
            let navigation = [
                0x20, // SPACE
                0x0D, // ENTER
                0x09, // TAB
                0x1B, // ESC
                0x08, // BACKSPACE
                0x2E, // DELETE
                0x2D, // INSERT
                0x24, // HOME
                0x23, // END
                0x21, // PAGEUP
                0x22, // PAGEDOWN
                0x26, // UP
                0x28, // DOWN
                0x25, // LEFT
                0x27, // RIGHT
            ];
            for &vk in &navigation {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            // Lock and special keys
            let lock_keys = [
                0x14, // CAPSLOCK
                0x90, // NUMLOCK
                0x91, // SCROLL LOCK
                0x13, // PAUSE
                0x2C, // PRINT SCREEN
            ];
            for &vk in &lock_keys {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            // Numpad operators
            let numpad_ops = [
                0x6A, // MULTIPLY
                0x6B, // ADD
                0x6C, // SEPARATOR
                0x6D, // SUBTRACT
                0x6E, // DECIMAL
                0x6F, // DIVIDE
            ];
            for &vk in &numpad_ops {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            // OEM keys (punctuation and symbols)
            let oem_keys = [
                0xBA, // OEM_1 (;:)
                0xBB, // OEM_PLUS (=+)
                0xBC, // OEM_COMMA (,<)
                0xBD, // OEM_MINUS (-_)
                0xBE, // OEM_PERIOD (.>)
                0xBF, // OEM_2 (/?)
                0xC0, // OEM_3 (`~)
                0xDB, // OEM_4 ([{)
                0xDC, // OEM_5 (\|)
                0xDD, // OEM_6 (]})
                0xDE, // OEM_7 ('")
                0xDF, // OEM_8
                0xE2, // OEM_102 (<>)
            ];
            for &vk in &oem_keys {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            // Mouse buttons
            let mouse_buttons = [
                0x01, // LBUTTON
                0x02, // RBUTTON
                0x04, // MBUTTON
                0x05, // XBUTTON1
                0x06, // XBUTTON2
            ];
            for &vk in &mouse_buttons {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }
        }

        pressed_keys
    }

    /// Formats a set of VK codes into a key combination string
    #[inline]
    fn format_captured_keys(vk_codes: &std::collections::HashSet<u32>) -> Option<String> {
        if vk_codes.is_empty() {
            return None;
        }

        // Separate modifiers and main keys
        let mut modifiers: Vec<u32> = Vec::new();
        let mut main_keys: Vec<u32> = Vec::new();

        for &vk in vk_codes {
            match vk {
                0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0x5B | 0x5C => {
                    modifiers.push(vk);
                }
                _ => {
                    main_keys.push(vk);
                }
            }
        }

        // Build the key combination string
        let mut parts: Vec<String> = Vec::new();

        // Add modifiers in consistent order
        for &vk in &modifiers {
            let name = match vk {
                0xA2 => "LCTRL",
                0xA3 => "RCTRL",
                0xA4 => "LALT",
                0xA5 => "RALT",
                0xA0 => "LSHIFT",
                0xA1 => "RSHIFT",
                0x5B => "LWIN",
                0x5C => "RWIN",
                _ => continue,
            };
            parts.push(name.to_string());
        }

        // Add main key (only use the first one if multiple)
        if let Some(&vk) = main_keys.first() {
            let name = Self::vk_to_string(vk)?;
            parts.push(name);
        }

        if !parts.is_empty() {
            Some(parts.join("+"))
        } else {
            None
        }
    }

    /// Converts VK code to key name string
    #[inline]
    fn vk_to_string(vk: u32) -> Option<String> {
        match vk {
            // A-Z
            0x41..=0x5A => Some(char::from_u32(vk).unwrap().to_string()),
            // 0-9
            0x30..=0x39 => Some(char::from_u32(vk).unwrap().to_string()),
            // Numpad 0-9
            0x60..=0x69 => Some(format!("NUMPAD{}", vk - 0x60)),
            // F1-F24
            0x70..=0x87 => Some(format!("F{}", vk - 0x70 + 1)),
            // Navigation keys
            0x20 => Some("SPACE".to_string()),
            0x0D => Some("RETURN".to_string()),
            0x09 => Some("TAB".to_string()),
            0x1B => Some("ESCAPE".to_string()),
            0x08 => Some("BACK".to_string()),
            0x2E => Some("DELETE".to_string()),
            0x2D => Some("INSERT".to_string()),
            0x24 => Some("HOME".to_string()),
            0x23 => Some("END".to_string()),
            0x21 => Some("PAGEUP".to_string()),
            0x22 => Some("PAGEDOWN".to_string()),
            0x26 => Some("UP".to_string()),
            0x28 => Some("DOWN".to_string()),
            0x25 => Some("LEFT".to_string()),
            0x27 => Some("RIGHT".to_string()),
            // Lock and special keys
            0x14 => Some("CAPITAL".to_string()),
            0x90 => Some("NUMLOCK".to_string()),
            0x91 => Some("SCROLL".to_string()),
            0x13 => Some("PAUSE".to_string()),
            0x2C => Some("SNAPSHOT".to_string()),
            // Numpad operators
            0x6A => Some("MULTIPLY".to_string()),
            0x6B => Some("ADD".to_string()),
            0x6C => Some("SEPARATOR".to_string()),
            0x6D => Some("SUBTRACT".to_string()),
            0x6E => Some("DECIMAL".to_string()),
            0x6F => Some("DIVIDE".to_string()),
            // OEM keys
            0xBA => Some("OEM_1".to_string()),
            0xBB => Some("OEM_PLUS".to_string()),
            0xBC => Some("OEM_COMMA".to_string()),
            0xBD => Some("OEM_MINUS".to_string()),
            0xBE => Some("OEM_PERIOD".to_string()),
            0xBF => Some("OEM_2".to_string()),
            0xC0 => Some("OEM_3".to_string()),
            0xDB => Some("OEM_4".to_string()),
            0xDC => Some("OEM_5".to_string()),
            0xDD => Some("OEM_6".to_string()),
            0xDE => Some("OEM_7".to_string()),
            0xDF => Some("OEM_8".to_string()),
            0xE2 => Some("OEM_102".to_string()),
            // Modifiers
            0xA2 => Some("LCTRL".to_string()),
            0xA3 => Some("RCTRL".to_string()),
            0xA4 => Some("LALT".to_string()),
            0xA5 => Some("RALT".to_string()),
            0xA0 => Some("LSHIFT".to_string()),
            0xA1 => Some("RSHIFT".to_string()),
            0x5B => Some("LWIN".to_string()),
            0x5C => Some("RWIN".to_string()),
            // Mouse buttons
            0x01 => Some("LBUTTON".to_string()),
            0x02 => Some("RBUTTON".to_string()),
            0x04 => Some("MBUTTON".to_string()),
            0x05 => Some("XBUTTON1".to_string()),
            0x06 => Some("XBUTTON2".to_string()),
            _ => None,
        }
    }
}
