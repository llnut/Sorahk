//! Settings dialog implementation.

mod capture;
mod helpers;

use crate::config::KeyMapping;
use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;
use crate::state::CaptureMode;
use eframe::egui;

use std::str::FromStr;

use helpers::{
    calculate_mouse_direction, estimate_arrow_width, estimate_pill_width, estimate_target_pill_width,
    get_capture_mode_display_name, get_sequence_key_color, get_sequence_key_display,
    get_target_key_color, is_mouse_move_target, is_mouse_scroll_target, truncate_text_safe,
    BUTTON_TEXT_MAX_CHARS,
};

impl SorahkGui {
    /// Renders the settings dialog for configuration management.
    pub(super) fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_cancel = false;

        // Handle key and mouse capture if in capture mode
        // Priority: Keyboard > Mouse > Raw Input (gamepad/joystick)
        // Skip input handling when HID activation dialog is active
        if self.key_capture_mode != KeyCaptureMode::None && self.hid_activation_dialog.is_none() {
            let mut captured_input: Option<String> = None;

            // Determine if we're in sequence capture mode (trigger or target)
            let is_sequence_capture = match self.key_capture_mode {
                KeyCaptureMode::NewMappingTrigger => self.new_mapping_is_sequence_mode,
                KeyCaptureMode::NewMappingTarget => self.new_mapping_target_mode == 2,
                KeyCaptureMode::MappingTrigger(idx) => {
                    if let Some(temp_config) = &self.temp_config {
                        temp_config
                            .mappings
                            .get(idx)
                            .map(|m| m.is_sequence_trigger())
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                KeyCaptureMode::MappingTarget(idx) => {
                    if let Some(temp_config) = &self.temp_config {
                        temp_config
                            .mappings
                            .get(idx)
                            .map(|m| m.target_mode == 2)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // Check mouse movement and button clicks for sequence capture
            if is_sequence_capture && !self.just_captured_input {
                // Check if pointer is over UI BEFORE entering ctx.input() to avoid deadlock
                let pointer_over_ui = ctx.is_pointer_over_area();

                ctx.input(|i| {
                    // Initialize mouse position if not set
                    if self.sequence_last_mouse_pos.is_none() {
                        self.sequence_last_mouse_pos =
                            Some(i.pointer.hover_pos().unwrap_or_default());
                        self.sequence_mouse_delta = egui::Vec2::ZERO;
                    }

                    // Check mouse button clicks first (only if not over UI)
                    // Additional check: don't capture if button is already in sequence_last_mouse_direction (deduplication)
                    if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Primary)
                        && self.sequence_last_mouse_direction.as_deref() != Some("LBUTTON")
                    {
                        captured_input = Some("LBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("LBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Secondary)
                        && self.sequence_last_mouse_direction.as_deref() != Some("RBUTTON")
                    {
                        captured_input = Some("RBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("RBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Middle)
                        && self.sequence_last_mouse_direction.as_deref() != Some("MBUTTON")
                    {
                        captured_input = Some("MBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("MBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Extra1)
                        && self.sequence_last_mouse_direction.as_deref() != Some("XBUTTON1")
                    {
                        captured_input = Some("XBUTTON1".to_string());
                        self.sequence_last_mouse_direction = Some("XBUTTON1".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Extra2)
                        && self.sequence_last_mouse_direction.as_deref() != Some("XBUTTON2")
                    {
                        captured_input = Some("XBUTTON2".to_string());
                        self.sequence_last_mouse_direction = Some("XBUTTON2".to_string());
                    }

                    // Check mouse movement if no button clicked
                    if captured_input.is_none()
                        && let Some(current_pos) = i.pointer.hover_pos()
                            && let Some(last_pos) = self.sequence_last_mouse_pos {
                                // Accumulate delta
                                let frame_delta = current_pos - last_pos;
                                self.sequence_mouse_delta += frame_delta;

                                // Check if movement threshold reached (30 pixels accumulated)
                                if let Some(direction) =
                                    calculate_mouse_direction(self.sequence_mouse_delta, 30.0)
                                {
                                    captured_input = Some(direction.to_string());
                                    self.sequence_last_mouse_direction =
                                        Some(direction.to_string());
                                    self.sequence_mouse_delta = egui::Vec2::ZERO;
                                }

                                self.sequence_last_mouse_pos = Some(current_pos);
                            }
                });
            }

            if captured_input.is_none() && !self.just_captured_input {
                let current_pressed = Self::poll_all_pressed_keys();

                let new_keys: std::collections::HashSet<u32> = current_pressed
                    .iter()
                    .filter(|&vk| !self.capture_initial_pressed.contains(vk))
                    .copied()
                    .collect();

                let has_new_key = new_keys
                    .iter()
                    .any(|vk| !self.capture_pressed_keys.contains(vk));

                if has_new_key && !self.capture_pressed_keys.is_empty() {
                    captured_input = Self::format_captured_keys(&self.capture_pressed_keys);
                } else {
                    new_keys.iter().for_each(|&vk| {
                        self.capture_pressed_keys.insert(vk);
                    });

                    let any_released = self
                        .capture_pressed_keys
                        .iter()
                        .any(|vk| !current_pressed.contains(vk));

                    if any_released {
                        captured_input = Self::format_captured_keys(&self.capture_pressed_keys);
                    }
                }
            }

            // Check mouse button input ONLY if keyboard not captured
            // Skip mouse capture on the first frame after entering capture mode (just_captured_input flag)
            // to avoid capturing the click on the "Capture" button itself
            // ALSO skip mouse capture in sequence mode - sequences should only capture keyboard keys
            if captured_input.is_none() && !self.just_captured_input && !is_sequence_capture {
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
            // In sequence mode, allow raw input capture for gamepad sequences
            if captured_input.is_none() {
                let should_check_raw_input = matches!(
                    self.key_capture_mode,
                    KeyCaptureMode::ToggleKey
                        | KeyCaptureMode::MappingTrigger(_)
                        | KeyCaptureMode::MappingTarget(_)
                        | KeyCaptureMode::NewMappingTrigger
                        | KeyCaptureMode::NewMappingTarget
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
                                if mapping.is_sequence_trigger() {
                                    // Sequence mode: add to existing sequence
                                    if let Some(seq_str) = &mapping.trigger_sequence {
                                        if seq_str.is_empty() {
                                            mapping.trigger_sequence = Some(input_name.clone());
                                            mapping.trigger_key = input_name.clone();
                                        } else {
                                            let mut new_seq = String::with_capacity(
                                                seq_str.len() + 1 + input_name.len(),
                                            );
                                            new_seq.push_str(seq_str);
                                            new_seq.push(',');
                                            new_seq.push_str(&input_name);
                                            mapping.trigger_sequence = Some(new_seq);
                                        }
                                    } else {
                                        mapping.trigger_sequence = Some(input_name.clone());
                                        mapping.trigger_key = input_name.clone();
                                    }
                                    // Clear capture state but keep capturing
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                } else {
                                    // Single key mode: replace trigger key
                                    mapping.trigger_key = input_name.clone();
                                }
                            }
                        }
                        KeyCaptureMode::MappingTarget(idx) => {
                            if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                if mapping.target_mode == 2 {
                                    // Sequence mode: add to editing list (like new mapping)
                                    self.editing_target_seq_list.push(input_name.clone());
                                    // Clear capture state for next key
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                } else {
                                    // Single/Multi mode: add directly to mapping
                                    mapping.add_target_key(input_name.clone());
                                }
                            }
                        }
                        KeyCaptureMode::NewMappingTrigger => {
                            if is_sequence_capture {
                                // Sequence mode: add to list and continue capturing
                                // Allow duplicates for combo moves like S→A→S→D
                                self.sequence_capture_list.push(input_name);
                                // Clear capture state but DON'T exit capture mode
                                self.capture_pressed_keys.clear();
                                self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                // DON'T return - let UI render normally
                            } else {
                                // Single key mode: normal behavior
                                self.new_mapping_trigger = input_name.clone();
                            }
                        }
                        KeyCaptureMode::NewMappingTarget => {
                            match self.new_mapping_target_mode {
                                0 => {
                                    // Single mode: just set single target
                                    self.new_mapping_target = input_name.clone();
                                    self.new_mapping_target_keys.clear();
                                    self.new_mapping_target_keys.push(input_name);
                                }
                                1 => {
                                    // Multi mode: add to list (no duplicates)
                                    self.new_mapping_target = input_name.clone();
                                    if !self.new_mapping_target_keys.contains(&input_name) {
                                        self.new_mapping_target_keys.push(input_name);
                                    }
                                }
                                2 => {
                                    // Sequence mode: add to sequence list (allow duplicates)
                                    self.target_sequence_capture_list.push(input_name.clone());
                                    // Sync to target_keys
                                    self.new_mapping_target_keys =
                                        self.target_sequence_capture_list.clone();
                                    self.new_mapping_target = input_name;
                                    // Clear capture state but DON'T exit capture mode
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                }
                                _ => {}
                            }
                        }
                        KeyCaptureMode::None => {}
                    }
                }

                // Exit capture mode and clear capture state
                // BUT: Keep capture mode active for sequence capture (new mapping or existing mapping)
                let is_existing_mapping_sequence =
                    if let KeyCaptureMode::MappingTrigger(idx) = self.key_capture_mode {
                        if let Some(temp_config) = &self.temp_config {
                            temp_config
                                .mappings
                                .get(idx)
                                .map(|m| m.is_sequence_trigger())
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                // Also keep capture mode for existing mapping target sequence
                let is_existing_target_sequence =
                    if let KeyCaptureMode::MappingTarget(idx) = self.key_capture_mode {
                        if let Some(temp_config) = &self.temp_config {
                            temp_config
                                .mappings
                                .get(idx)
                                .map(|m| m.target_mode == 2)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                if !is_sequence_capture
                    && !is_existing_mapping_sequence
                    && !is_existing_target_sequence
                {
                    self.key_capture_mode = KeyCaptureMode::None;
                    self.capture_pressed_keys.clear();
                    self.app_state.set_raw_input_capture_mode(false);
                }
                // Always clear just_captured_input flag after processing input
                self.just_captured_input = false;
            } else {
                // In capture mode but no input captured yet - clear the flag after first frame
                if self.just_captured_input {
                    self.just_captured_input = false;
                }
            }
        } else {
            // Not in capture mode: ensure state is clean
            self.capture_pressed_keys.clear();
            self.just_captured_input = false;
        }

        let dialog_bg = if self.dark_mode {
            egui::Color32::from_rgb(30, 32, 42)
        } else {
            egui::Color32::from_rgb(252, 248, 255)
        };

        egui::Window::new("")
            .title_bar(false)
            .collapsible(false)
            .resizable(true)
            .default_size([750.0, 530.0])
            .min_size([750.0, 530.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .id(egui::Id::new("settings_dialog_window"))
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(dialog_bg)
                    .corner_radius(egui::CornerRadius::same(20))
                    .stroke(egui::Stroke::NONE)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 5],
                        blur: 22,
                        spread: 2,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 45),
                    }),
            )
            .show(ctx, |ui| {
                ui.push_id("settings_dialog_scope", |ui| {
                    let t = &self.translations;

                    // Custom title bar (matching main window style)
                    ui.horizontal(|ui| {
                        ui.add_space(15.0);

                        // Settings title
                        ui.label(
                            egui::RichText::new(t.settings_dialog_title())
                                .size(18.0)
                                .strong()
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(176, 224, 230) // Light cyan
                                } else {
                                    egui::Color32::from_rgb(135, 206, 235) // Sky blue
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
                        .inner_margin(egui::Margin::symmetric(12, 0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(500.0)
                                .show(ui, |ui| {
                                    // Toggle Key Section
                                    let card_bg = if self.dark_mode {
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
                                                egui::RichText::new(t.toggle_key())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(200, 180, 255)
                                                    } else {
                                                        egui::Color32::from_rgb(100, 120, 200)
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
                                                    .color(if self.dark_mode {
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
                                                    let current_mode_name = get_capture_mode_display_name(t, current_mode);
                                                    egui::ComboBox::from_id_salt("rawinput_capture_mode")
                                                        .selected_text(current_mode_name)
                                                        .width(180.0)
                                                        .show_ui(ui, |ui| {
                                                            for &mode in CaptureMode::all_modes() {
                                                                let mode_name = get_capture_mode_display_name(t, mode);
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
                                                egui::RichText::new(t.key_mappings_title())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(200, 180, 255)
                                                    } else {
                                                        egui::Color32::from_rgb(80, 150, 90)
                                                    }),
                                            );
                                            ui.add_space(6.0);

                                            ui.add_space(2.0);
                                            let hint_bg = if self.dark_mode {
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
                                                                .color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
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
                                            for (idx, mapping) in
                                                temp_config.mappings.iter_mut().enumerate()
                                            {
                                                // Mapping card with card-style layout
                                                let mapping_card_bg = if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 52, 62)
                                                } else {
                                                    egui::Color32::from_rgb(255, 250, 255)
                                                };

                                                egui::Frame::NONE
                                                    .fill(mapping_card_bg)
                                                    .corner_radius(egui::CornerRadius::same(16))
                                                    .inner_margin(egui::Margin::same(14))
                                                    .stroke(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
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
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(255, 182, 193)
                                                                    } else {
                                                                        egui::Color32::from_rgb(255, 218, 224)
                                                                    },
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(80, 20, 40)
                                                                    } else {
                                                                        egui::Color32::from_rgb(220, 80, 120)
                                                                    }
                                                                )
                                                            } else {
                                                                (
                                                                    t.trigger_mode_single_badge(),
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(135, 206, 235)
                                                                    } else {
                                                                        egui::Color32::from_rgb(173, 216, 230)
                                                                    },
                                                                    if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
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
                                                                        if self.dark_mode {
                                                                            egui::Color32::from_rgb(20, 60, 80)
                                                                        } else {
                                                                            egui::Color32::from_rgb(40, 80, 120)
                                                                        }
                                                                    } else if self.dark_mode {
                                                                        egui::Color32::from_rgb(180, 180, 200)
                                                                    } else {
                                                                        egui::Color32::from_rgb(100, 100, 120)
                                                                    }),
                                                            )
                                                            .fill(if !is_sequence {
                                                                if self.dark_mode {
                                                                    egui::Color32::from_rgb(135, 206, 235)
                                                                } else {
                                                                    egui::Color32::from_rgb(173, 216, 230)
                                                                }
                                                            } else if self.dark_mode {
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
                                                                        if self.dark_mode {
                                                                            egui::Color32::from_rgb(80, 20, 40)
                                                                        } else {
                                                                            egui::Color32::from_rgb(220, 80, 120)
                                                                        }
                                                                    } else if self.dark_mode {
                                                                        egui::Color32::from_rgb(180, 180, 200)
                                                                    } else {
                                                                        egui::Color32::from_rgb(100, 100, 120)
                                                                    }),
                                                            )
                                                            .fill(if is_sequence {
                                                                if self.dark_mode {
                                                                    egui::Color32::from_rgb(255, 182, 193)
                                                                } else {
                                                                    egui::Color32::from_rgb(255, 218, 224)
                                                                }
                                                            } else if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(200, 200, 220)
                                                                    } else {
                                                                        egui::Color32::from_rgb(80, 80, 100)
                                                                    }),
                                                            );
                                                        });
                                                        ui.add_space(6.0);
                                                        let is_capturing_trigger = self.key_capture_mode
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
                                                            if trigger_response.clicked() && !self.just_captured_input {
                                                                self.key_capture_mode =
                                                                    KeyCaptureMode::MappingTrigger(idx);
                                                                self.capture_pressed_keys.clear();
                                                                self.capture_initial_pressed =
                                                                    Self::poll_all_pressed_keys();
                                                                self.app_state.set_raw_input_capture_mode(true);
                                                                self.just_captured_input = true;
                                                            }
                                                        } else {
                                                            // For sequence mode, show add button
                                                            ui.horizontal(|ui| {
                                                                let add_seq_btn = egui::Button::new(
                                                                    egui::RichText::new(t.add_button_text())
                                                                        .size(13.0)
                                                                        .color(egui::Color32::WHITE),
                                                                )
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(100, 180, 240)
                                                                } else {
                                                                    egui::Color32::from_rgb(150, 200, 250)
                                                                })
                                                                .corner_radius(10.0);
                                                                if ui.add(add_seq_btn).on_hover_text(t.add_sequence_key_hover()).clicked() && !self.just_captured_input {
                                                                    self.key_capture_mode = KeyCaptureMode::MappingTrigger(idx);
                                                                    self.capture_pressed_keys.clear();
                                                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                                    self.app_state.set_raw_input_capture_mode(true);
                                                                    self.just_captured_input = true;
                                                                    // Initialize sequence capture state for mouse movement
                                                                    self.sequence_last_mouse_pos = None;
                                                                    self.sequence_last_mouse_direction = None;
                                                                    self.sequence_mouse_delta = egui::Vec2::ZERO;
                                                                }
                                                            });
                                                        }
                                                        // Handle captured input for sequence mode
                                                        if is_capturing_trigger && mapping.is_sequence_trigger() {
                                                            ui.add_space(6.0);
                                                            egui::Frame::NONE
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(255, 200, 130)
                                                                } else {
                                                                    egui::Color32::from_rgb(255, 235, 180)
                                                                })
                                                                .corner_radius(egui::CornerRadius::same(10))
                                                                .inner_margin(egui::Margin::symmetric(10, 8))
                                                                .show(ui, |ui| {
                                                                    ui.label(
                                                                        egui::RichText::new(t.sequence_capturing())
                                                                            .size(13.0)
                                                                            .strong()
                                                                            .color(if self.dark_mode {
                                                                                egui::Color32::from_rgb(80, 60, 20)
                                                                            } else {
                                                                                egui::Color32::from_rgb(100, 80, 20)
                                                                            }),
                                                                    );
                                                                    // Show immediate preview of captured keys
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
                                                                                    .size(12.0)
                                                                                    .color(if self.dark_mode {
                                                                                        egui::Color32::from_rgb(60, 40, 10)
                                                                                    } else {
                                                                                        egui::Color32::from_rgb(80, 60, 20)
                                                                                    }),
                                                                            );
                                                                        }
                                                                    }
                                                                });
                                                            ui.add_space(6.0);
                                                            ui.horizontal(|ui| {
                                                                // Done button
                                                                let done_btn = egui::Button::new(
                                                                    egui::RichText::new(t.sequence_complete())
                                                                        .size(12.0)
                                                                        .color(egui::Color32::WHITE)
                                                                        .strong(),
                                                                )
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(120, 220, 140)
                                                                } else {
                                                                    egui::Color32::from_rgb(140, 230, 150)
                                                                })
                                                                .corner_radius(8.0);
                                                                if ui.add_sized([100.0, 26.0], done_btn).clicked() {
                                                                    self.key_capture_mode = KeyCaptureMode::None;
                                                                    self.app_state.set_raw_input_capture_mode(false);
                                                                    self.capture_pressed_keys.clear();
                                                                    self.just_captured_input = true;
                                                                }
                                                                ui.add_space(6.0);
                                                                // Clear button
                                                                let clear_btn = egui::Button::new(
                                                                    egui::RichText::new(t.sequence_clear_btn())
                                                                        .size(12.0)
                                                                        .color(egui::Color32::WHITE),
                                                                )
                                                                .fill(if self.dark_mode {
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
                                                                        .fill(if self.dark_mode {
                                                                            egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                                                                        } else {
                                                                            egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                                                                        })
                                                                        .corner_radius(egui::CornerRadius::same(12))
                                                                        .inner_margin(egui::Margin::symmetric(inner_margin as i8, inner_margin as i8))
                                                                        .show(ui, |ui| {
                                                                            // Fill the entire available width (content width = full_width - 2*inner_margin)
                                                                            let content_width = full_width - inner_margin * 2.0;
                                                                            ui.set_min_width(content_width);
                                                                            ui.set_max_width(content_width);
                                                                            // Header with cute icon
                                                                            ui.horizontal(|ui| {
                                                                                let t = &self.translations;
                                                                                ui.label(
                                                                                    egui::RichText::new(t.sequence_icon())
                                                                                        .size(12.0)
                                                                                );
                                                                                ui.label(
                                                                                    egui::RichText::new(t.format_keys_count(seq_keys.len()))
                                                                                        .size(10.0)
                                                                                        .italics()
                                                                                        .color(if self.dark_mode {
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
                                                                                        let pill_width = estimate_pill_width(key);
                                                                                        let arrow_width = if key_idx < seq_keys.len() - 1 { estimate_arrow_width() } else { 0.0 };
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
                                                                                                let (icon, display_name) = get_sequence_key_display(key);
                                                                                                let tag_color = get_sequence_key_color(key, self.dark_mode);
                                                                                                let text_color = if self.dark_mode {
                                                                                                    egui::Color32::from_rgb(40, 30, 50)
                                                                                                } else {
                                                                                                    egui::Color32::from_rgb(60, 40, 70)
                                                                                                };
                                                                                                // Cute pill-shaped tag
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
                                                                                                            // Display name (truncated for long names)
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
                                                                                                            // Delete button (×)
                                                                                                            let del_btn = ui.add(
                                                                                                                egui::Button::new(
                                                                                                                    egui::RichText::new("×")
                                                                                                                        .size(11.0)
                                                                                                                        .color(if self.dark_mode {
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
                                                                                                            .color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(135, 206, 235)
                                                                    } else {
                                                                        egui::Color32::from_rgb(70, 130, 180)
                                                                    }),
                                                            );
                                                            ui.add_space(8.0);

                                                            // Helper for inactive button styling
                                                            let inactive_fill = if self.dark_mode {
                                                                egui::Color32::from_rgb(50, 52, 62)
                                                            } else {
                                                                egui::Color32::from_rgb(240, 240, 245)
                                                            };
                                                            let inactive_text = if self.dark_mode {
                                                                egui::Color32::from_rgb(180, 180, 200)
                                                            } else {
                                                                egui::Color32::from_rgb(100, 100, 120)
                                                            };

                                                            // Single Button - matches Single badge colors
                                                            let single_btn = egui::Button::new(
                                                                egui::RichText::new(t.target_mode_single())
                                                                    .size(11.0)
                                                                    .color(if target_mode == 0 {
                                                                        if self.dark_mode {
                                                                            egui::Color32::from_rgb(20, 60, 80)
                                                                        } else {
                                                                            egui::Color32::from_rgb(40, 80, 120)
                                                                        }
                                                                    } else {
                                                                        inactive_text
                                                                    })
                                                            ).fill(if target_mode == 0 {
                                                                if self.dark_mode {
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
                                                                if target_mode == 2 && self.editing_target_seq_idx == Some(idx) {
                                                                    self.editing_target_seq_list.clear();
                                                                    self.editing_target_seq_idx = None;
                                                                    self.key_capture_mode = KeyCaptureMode::None;
                                                                    self.app_state.set_raw_input_capture_mode(false);
                                                                }
                                                            }
                                                            ui.add_space(4.0);

                                                            // Multi Button - matches Single badge colors
                                                            let multi_btn = egui::Button::new(
                                                                egui::RichText::new(t.target_mode_multi())
                                                                    .size(11.0)
                                                                    .color(if target_mode == 1 {
                                                                        if self.dark_mode {
                                                                            egui::Color32::from_rgb(20, 60, 80)
                                                                        } else {
                                                                            egui::Color32::from_rgb(40, 80, 120)
                                                                        }
                                                                    } else {
                                                                        inactive_text
                                                                    })
                                                            ).fill(if target_mode == 1 {
                                                                if self.dark_mode {
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
                                                                    if self.editing_target_seq_idx == Some(idx) {
                                                                        self.editing_target_seq_list.clear();
                                                                        self.editing_target_seq_idx = None;
                                                                        self.key_capture_mode = KeyCaptureMode::None;
                                                                        self.app_state.set_raw_input_capture_mode(false);
                                                                    }
                                                                }
                                                            }
                                                            ui.add_space(4.0);

                                                            // Sequence Button - matches Sequence badge colors
                                                            let seq_btn = egui::Button::new(
                                                                egui::RichText::new(t.target_mode_sequence())
                                                                    .size(11.0)
                                                                    .color(if target_mode == 2 {
                                                                        if self.dark_mode {
                                                                            egui::Color32::from_rgb(80, 20, 40)
                                                                        } else {
                                                                            egui::Color32::from_rgb(220, 80, 120)
                                                                        }
                                                                    } else {
                                                                        inactive_text
                                                                    })
                                                            ).fill(if target_mode == 2 {
                                                                if self.dark_mode {
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
                                                        });
                                                        ui.add_space(6.0);
                                                        let is_capturing_target = self.key_capture_mode
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
                                                                egui::Color32::from_rgb(60, 62, 72)
                                                            } else {
                                                                egui::Color32::from_rgb(245, 245, 250)
                                                            })
                                                            .corner_radius(10.0);

                                                            let mut target_response = ui
                                                                .add_sized([ui.available_width(), 30.0], target_btn);
                                                            // Show full text on hover if truncated
                                                            if !is_capturing_target && !target_full_text.is_empty() && target_full_text.chars().count() > BUTTON_TEXT_MAX_CHARS {
                                                                target_response = target_response.on_hover_text(&target_full_text);
                                                            }
                                                            if target_response.clicked() && !self.just_captured_input {
                                                                self.key_capture_mode =
                                                                    KeyCaptureMode::MappingTarget(idx);
                                                                self.capture_pressed_keys.clear();
                                                                self.capture_initial_pressed =
                                                                    Self::poll_all_pressed_keys();
                                                            }
                                                        } else {
                                                            // For sequence mode
                                                            if is_capturing_target {
                                                                // Show capture UI with preview from editing list
                                                                let preview_keys = &self.editing_target_seq_list;
                                                                egui::Frame::NONE
                                                                    .fill(if self.dark_mode {
                                                                        egui::Color32::from_rgb(255, 200, 130)
                                                                    } else {
                                                                        egui::Color32::from_rgb(255, 235, 180)
                                                                    })
                                                                    .corner_radius(egui::CornerRadius::same(12))
                                                                    .inner_margin(egui::Margin::symmetric(12, 10))
                                                                    .show(ui, |ui| {
                                                                        ui.set_min_width(ui.available_width());
                                                                        ui.vertical(|ui| {
                                                                            ui.horizontal(|ui| {
                                                                                ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                                                                                ui.label(
                                                                                    egui::RichText::new(t.sequence_capturing())
                                                                                        .size(14.0)
                                                                                        .strong()
                                                                                        .color(if self.dark_mode {
                                                                                            egui::Color32::from_rgb(80, 60, 20)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(100, 80, 20)
                                                                                        }),
                                                                                );
                                                                            });
                                                                            ui.add_space(4.0);
                                                                            // Show current keys preview
                                                                            if !preview_keys.is_empty() {
                                                                                let count = preview_keys.len();
                                                                                let preview = if count <= 3 {
                                                                                    preview_keys.join(" → ")
                                                                                } else {
                                                                                    let last = preview_keys.last().map(|s| s.as_str()).unwrap_or("");
                                                                                    format!("{} keys: ... → {}", count, last)
                                                                                };
                                                                                ui.horizontal(|ui| {
                                                                                    ui.add_space(8.0);
                                                                                    ui.label(
                                                                                        egui::RichText::new(&preview)
                                                                                            .size(13.0)
                                                                                            .color(if self.dark_mode {
                                                                                                egui::Color32::from_rgb(60, 40, 10)
                                                                                            } else {
                                                                                                egui::Color32::from_rgb(80, 60, 20)
                                                                                            }),
                                                                                    );
                                                                                });
                                                                            }
                                                                            ui.add_space(4.0);
                                                                            ui.horizontal(|ui| {
                                                                                ui.add_space(8.0);
                                                                                ui.label(
                                                                                    egui::RichText::new(t.sequence_capture_hint())
                                                                                        .size(11.0)
                                                                                        .italics()
                                                                                        .color(if self.dark_mode {
                                                                                            egui::Color32::from_rgb(100, 80, 40)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(120, 100, 60)
                                                                                        }),
                                                                                );
                                                                            });
                                                                        });
                                                                    });
                                                                ui.add_space(8.0);
                                                                // Track button clicks
                                                                let mut should_finish = false;
                                                                let mut should_clear = false;
                                                                ui.horizontal(|ui| {
                                                                    // Done button
                                                                    let done_btn = egui::Button::new(
                                                                        egui::RichText::new(t.sequence_complete())
                                                                            .size(13.0)
                                                                            .color(egui::Color32::WHITE)
                                                                            .strong(),
                                                                    )
                                                                    .fill(if self.dark_mode {
                                                                        egui::Color32::from_rgb(120, 220, 140)
                                                                    } else {
                                                                        egui::Color32::from_rgb(140, 230, 150)
                                                                    })
                                                                    .corner_radius(10.0);
                                                                    if ui.add_sized([120.0, 28.0], done_btn).clicked() {
                                                                        should_finish = true;
                                                                    }
                                                                    ui.add_space(8.0);
                                                                    // Clear button
                                                                    let clear_btn = egui::Button::new(
                                                                        egui::RichText::new(t.sequence_clear_btn())
                                                                            .size(13.0)
                                                                            .color(egui::Color32::WHITE),
                                                                    )
                                                                    .fill(if self.dark_mode {
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
                                                                    mapping.target_keys = self.editing_target_seq_list.iter().cloned().collect();
                                                                    self.editing_target_seq_list.clear();
                                                                    self.editing_target_seq_idx = None;
                                                                    self.key_capture_mode = KeyCaptureMode::None;
                                                                    self.app_state.set_raw_input_capture_mode(false);
                                                                    self.capture_pressed_keys.clear();
                                                                    self.just_captured_input = true;
                                                                }
                                                                if should_clear {
                                                                    self.editing_target_seq_list.clear();
                                                                }
                                                            } else {
                                                                // Not capturing - show +Add button
                                                                ui.horizontal(|ui| {
                                                                    let add_seq_btn = egui::Button::new(
                                                                        egui::RichText::new(t.add_button_text())
                                                                            .size(13.0)
                                                                            .color(egui::Color32::WHITE),
                                                                    )
                                                                    .fill(if self.dark_mode {
                                                                        egui::Color32::from_rgb(255, 140, 170)
                                                                    } else {
                                                                        egui::Color32::from_rgb(255, 170, 190)
                                                                    })
                                                                    .corner_radius(10.0);
                                                                    if ui.add(add_seq_btn).on_hover_text(t.add_target_key_hover()).clicked() && !self.just_captured_input {
                                                                        // Initialize editing list from current target keys
                                                                        self.editing_target_seq_list = mapping.target_keys.iter().cloned().collect();
                                                                        self.editing_target_seq_idx = Some(idx);
                                                                        self.key_capture_mode = KeyCaptureMode::MappingTarget(idx);
                                                                        self.capture_pressed_keys.clear();
                                                                        self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                                        self.app_state.set_raw_input_capture_mode(true);
                                                                        self.just_captured_input = true;
                                                                    }
                                                                });
                                                            }
                                                        }
                                                        // Show target keys as pill tags when multiple targets exist or in sequence mode
                                                        // During sequence capture, show from editing list for real-time updates
                                                        let display_keys: Vec<String> = if is_capturing_target && target_mode == 2 {
                                                            self.editing_target_seq_list.clone()
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
                                                                if self.dark_mode {
                                                                    egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                                                                } else {
                                                                    egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                                                                }
                                                            } else if self.dark_mode {
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
                                                                        let t = &self.translations;
                                                                        let icon = if target_mode == 2 { "🎬" } else { t.target_icon() };
                                                                        ui.label(egui::RichText::new(icon).size(12.0));
                                                                        ui.label(
                                                                            egui::RichText::new(t.format_targets_count(keys_count))
                                                                                .size(10.0)
                                                                                .italics()
                                                                                .color(if self.dark_mode {
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
                                                                            let sep_width = if target_mode == 2 { estimate_arrow_width() } else { 24.0 };

                                                                            // Pre-calculate rows
                                                                            let mut rows: Vec<Vec<usize>> = Vec::new();
                                                                            let mut current_row: Vec<usize> = Vec::new();
                                                                            let mut current_width = 0.0f32;

                                                                            for (key_idx, key) in keys_list.iter().enumerate() {
                                                                                let pill_width = if target_mode == 2 { estimate_pill_width(key) } else { estimate_target_pill_width(key) };
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
                                                                                            let color = get_sequence_key_color(key, self.dark_mode);
                                                                                            let text = if self.dark_mode {
                                                                                                egui::Color32::from_rgb(40, 30, 50)
                                                                                            } else {
                                                                                                egui::Color32::from_rgb(60, 40, 70)
                                                                                            };
                                                                                            (color, text)
                                                                                        } else {
                                                                                            let color = get_target_key_color(self.dark_mode);
                                                                                            let text = if self.dark_mode {
                                                                                                egui::Color32::from_rgb(20, 60, 80)
                                                                                            } else {
                                                                                                egui::Color32::from_rgb(40, 100, 140)
                                                                                            };
                                                                                            (color, text)
                                                                                        };
                                                                                        // Get display name for sequence mode
                                                                                        let display_name = if target_mode == 2 {
                                                                                            let (_, name) = get_sequence_key_display(key);
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
                                                                                                    let del_btn = ui.add(egui::Button::new(egui::RichText::new("×").size(11.0).color(
                                                                                                        if self.dark_mode { egui::Color32::from_rgb(180, 80, 100) }
                                                                                                        else { egui::Color32::from_rgb(200, 60, 80) }
                                                                                                    )).fill(egui::Color32::TRANSPARENT).frame(false).corner_radius(8.0));
                                                                                                    if del_btn.clicked() { key_to_remove = Some(key_idx); }
                                                                                                });
                                                                                            });
                                                                                        tag_response.response.on_hover_text(key);
                                                                                        if key_idx < keys_list.len() - 1 {
                                                                                            // Sequence mode uses pink arrow, Multi uses blue "+"
                                                                                            let sep_color = if target_mode == 2 {
                                                                                                if self.dark_mode { egui::Color32::from_rgb(255, 150, 170) }
                                                                                                else { egui::Color32::from_rgb(255, 120, 150) }
                                                                                            } else if self.dark_mode { egui::Color32::from_rgb(135, 206, 235) }
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
                                                                    if remove_idx < self.editing_target_seq_list.len() {
                                                                        self.editing_target_seq_list.remove(remove_idx);
                                                                    }
                                                                } else {
                                                                    // Normal mode: remove from mapping by index
                                                                    mapping.remove_target_key_at(remove_idx);
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
                                                                    .color(if self.dark_mode {
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
                                                            .background_color(if self.dark_mode {
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
                                                                        .color(if self.dark_mode {
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
                                                                .background_color(if self.dark_mode {
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
                                                                        .color(if self.dark_mode {
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
                                                                .background_color(if self.dark_mode {
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
                                                                        .color(if self.dark_mode {
                                                                            egui::Color32::from_rgb(170, 170, 190)
                                                                        } else {
                                                                            egui::Color32::from_rgb(100, 100, 120)
                                                                        }),
                                                                );
                                                                let mut window_str = mapping.sequence_window_ms.to_string();
                                                                let window_edit = egui::TextEdit::singleline(
                                                                    &mut window_str,
                                                                )
                                                                .background_color(if self.dark_mode {
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
                                                            .fill(if self.dark_mode {
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
                                                                self.key_capture_mode = KeyCaptureMode::MappingTarget(idx);
                                                                self.capture_pressed_keys.clear();
                                                                self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                            }

                                                            // Clear all trigger keys
                                                            let clear_trigger_btn = egui::Button::new(
                                                                egui::RichText::new("✖")
                                                                    .color(egui::Color32::WHITE)
                                                                    .size(16.0),
                                                            )
                                                            .fill(if self.dark_mode {
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
                                                            .fill(if self.dark_mode {
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
                                                            .fill(if self.dark_mode {
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
                                                                self.mouse_direction_dialog = Some(
                                                                    crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                                                                );
                                                                self.mouse_direction_mapping_idx = Some(idx);
                                                            }

                                                            // Mouse scroll direction
                                                            let scroll_btn = egui::Button::new(
                                                                egui::RichText::new("🎡")
                                                                    .color(egui::Color32::WHITE)
                                                                    .size(16.0),
                                                            )
                                                            .fill(if self.dark_mode {
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
                                                                self.mouse_scroll_dialog = Some(
                                                                    crate::gui::mouse_scroll_dialog::MouseScrollDialog::new(),
                                                                );
                                                                self.mouse_scroll_mapping_idx = Some(idx);
                                                            }

                                                            ui.add_space(4.0);

                                                            // Turbo toggle
                                                            let turbo_enabled = mapping.turbo_enabled;
                                                            let turbo_color = if turbo_enabled {
                                                                if self.dark_mode {
                                                                    egui::Color32::from_rgb(250, 200, 80)
                                                                } else {
                                                                    egui::Color32::from_rgb(255, 220, 120)
                                                                }
                                                            } else if self.dark_mode {
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
                                                                self.translations.turbo_on_hover()
                                                            } else {
                                                                self.translations.turbo_off_hover()
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
                                                                let t = &self.translations;
                                                                let delete_btn = egui::Button::new(
                                                                    egui::RichText::new(t.delete_icon())
                                                                        .size(15.0)
                                                                        .color(egui::Color32::WHITE),
                                                                )
                                                                .fill(if self.dark_mode {
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

                                            ui.add_space(12.0);
                                            ui.separator();
                                            ui.add_space(12.0);

                                            // Add new mapping section with card layout
                                            ui.label(
                                                egui::RichText::new(t.add_new_mapping_title())
                                                    .size(15.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(255, 182, 193)
                                                    } else {
                                                        egui::Color32::from_rgb(255, 105, 180)
                                                    }),
                                            );
                                            ui.add_space(10.0);

                                            let new_mapping_card_bg = if self.dark_mode {
                                                egui::Color32::from_rgb(50, 52, 62)
                                            } else {
                                                egui::Color32::from_rgb(255, 250, 255)
                                            };

                                            egui::Frame::NONE
                                                .fill(new_mapping_card_bg)
                                                .corner_radius(egui::CornerRadius::same(16))
                                                .inner_margin(egui::Margin::same(14))
                                                .stroke(if self.dark_mode {
                                                    egui::Stroke::NONE
                                                } else {
                                                    egui::Stroke::new(
                                                        1.5,
                                                        egui::Color32::from_rgba_premultiplied(147, 197, 253, 80)
                                                    )
                                                })
                                                .show(ui, |ui| {
                                                    ui.set_min_width(ui.available_width());
                                                    // Trigger Mode Selection - Place BEFORE trigger input! ✨
                                                    let is_sequence_mode = self.new_mapping_is_sequence_mode;
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(t.trigger_mode_label())
                                                                .size(13.0)
                                                                .color(if self.dark_mode {
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
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(20, 60, 80)
                                                                    } else {
                                                                        egui::Color32::from_rgb(40, 80, 120)
                                                                    }
                                                                } else if self.dark_mode {
                                                                    egui::Color32::from_rgb(180, 180, 200)
                                                                } else {
                                                                    egui::Color32::from_rgb(100, 100, 120)
                                                                }),
                                                        )
                                                        .fill(if !is_sequence_mode {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(135, 206, 235)
                                                            } else {
                                                                egui::Color32::from_rgb(173, 216, 230)
                                                            }
                                                        } else if self.dark_mode {
                                                            egui::Color32::from_rgb(50, 52, 62)
                                                        } else {
                                                            egui::Color32::from_rgb(240, 240, 245)
                                                        })
                                                        .corner_radius(10.0);
                                                        if ui.add(single_btn).clicked() && is_sequence_mode {
                                                            // Switch to single key mode - clear sequence
                                                            self.new_mapping_is_sequence_mode = false;
                                                            self.sequence_capture_list.clear();
                                                            self.sequence_last_mouse_pos = None;
                                                            self.sequence_last_mouse_direction = None;
                                                            self.sequence_mouse_delta = egui::Vec2::ZERO;
                                                            if self.new_mapping_trigger.contains(',') {
                                                                self.new_mapping_trigger.clear();
                                                            }
                                                        }
                                                        ui.add_space(6.0);
                                                        // Sequence Button - matches Sequence badge colors
                                                        let seq_btn = egui::Button::new(
                                                            egui::RichText::new(t.trigger_mode_sequence())
                                                                .size(12.0)
                                                                .color(if is_sequence_mode {
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(80, 20, 40)
                                                                    } else {
                                                                        egui::Color32::from_rgb(220, 80, 120)
                                                                    }
                                                                } else if self.dark_mode {
                                                                    egui::Color32::from_rgb(180, 180, 200)
                                                                } else {
                                                                    egui::Color32::from_rgb(100, 100, 120)
                                                                }),
                                                        )
                                                        .fill(if is_sequence_mode {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(255, 182, 193)
                                                            } else {
                                                                egui::Color32::from_rgb(255, 218, 224)
                                                            }
                                                        } else if self.dark_mode {
                                                            egui::Color32::from_rgb(50, 52, 62)
                                                        } else {
                                                            egui::Color32::from_rgb(240, 240, 245)
                                                        })
                                                        .corner_radius(10.0);
                                                        if ui.add(seq_btn).clicked() && !is_sequence_mode {
                                                            // Switch to sequence mode - clear single trigger
                                                            self.new_mapping_is_sequence_mode = true;
                                                            self.new_mapping_trigger.clear();
                                                        }
                                                    });
                                                    ui.add_space(8.0);
                                                    // Show explanation for sequence mode ♡
                                                    if is_sequence_mode {
                                                        egui::Frame::NONE
                                                            .fill(if self.dark_mode {
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
                                                                        .color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(200, 200, 220)
                                                                    } else {
                                                                        egui::Color32::from_rgb(80, 80, 100)
                                                                    }),
                                                            );
                                                            let window_edit = egui::TextEdit::singleline(
                                                                &mut self.new_mapping_sequence_window,
                                                            )
                                                            .background_color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
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
                                                                .color(if self.dark_mode {
                                                                    egui::Color32::from_rgb(200, 200, 220)
                                                                } else {
                                                                    egui::Color32::from_rgb(80, 80, 100)
                                                                }),
                                                        );
                                                    });
                                                    ui.add_space(6.0);
                                                    // Check if we should use sequence mode for UI
                                                    // Use the same is_sequence_mode from above to determine UI mode
                                                    if !is_sequence_mode {
                                                        // Single key trigger mode (existing behavior)
                                                        let is_capturing_new_trigger = self.key_capture_mode
                                                            == KeyCaptureMode::NewMappingTrigger;
                                                        let new_trigger_display = if is_capturing_new_trigger {
                                                            t.press_any_key().to_string()
                                                        } else if self.new_mapping_trigger.is_empty() {
                                                            t.click_to_set_trigger().to_string()
                                                        } else {
                                                            truncate_text_safe(&self.new_mapping_trigger, BUTTON_TEXT_MAX_CHARS)
                                                        };

                                                        let new_trigger_btn = egui::Button::new(
                                                            egui::RichText::new(&new_trigger_display)
                                                                .size(14.0)
                                                                .color(
                                                                    if is_capturing_new_trigger {
                                                                        egui::Color32::from_rgb(255, 215, 0)
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
                                                            egui::Color32::from_rgb(60, 62, 72)
                                                        } else {
                                                            egui::Color32::from_rgb(245, 245, 250)
                                                        })
                                                        .corner_radius(10.0);

                                                        let mut new_trigger_response = ui
                                                            .add_sized([ui.available_width(), 30.0], new_trigger_btn);
                                                        // Show full text on hover if truncated
                                                        if !is_capturing_new_trigger && !self.new_mapping_trigger.is_empty() && self.new_mapping_trigger.chars().count() > BUTTON_TEXT_MAX_CHARS {
                                                            new_trigger_response = new_trigger_response.on_hover_text(&self.new_mapping_trigger);
                                                        }
                                                        if new_trigger_response.clicked() && !self.just_captured_input {
                                                            self.key_capture_mode =
                                                                KeyCaptureMode::NewMappingTrigger;
                                                            self.capture_pressed_keys.clear();
                                                            self.capture_initial_pressed =
                                                                Self::poll_all_pressed_keys();
                                                            self.app_state.set_raw_input_capture_mode(true);
                                                            self.just_captured_input = true;
                                                            self.duplicate_mapping_error = None;
                                                        }
                                                        // Display full trigger text with wrapping for long device names
                                                        if !is_capturing_new_trigger && !self.new_mapping_trigger.is_empty() && self.new_mapping_trigger.chars().count() > BUTTON_TEXT_MAX_CHARS {
                                                            ui.add_space(6.0);
                                                            egui::Frame::NONE
                                                                .inner_margin(egui::Margin::symmetric(8, 4))
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgba_unmultiplied(60, 62, 72, 100)
                                                                } else {
                                                                    egui::Color32::from_rgba_unmultiplied(240, 240, 245, 150)
                                                                })
                                                                .corner_radius(6.0)
                                                                .show(ui, |ui| {
                                                                    ui.set_max_width(ui.available_width());
                                                                    ui.add(egui::Label::new(
                                                                        egui::RichText::new(&self.new_mapping_trigger)
                                                                            .size(11.0)
                                                                            .color(if self.dark_mode {
                                                                                egui::Color32::from_rgb(180, 180, 200)
                                                                            } else {
                                                                                egui::Color32::from_rgb(80, 80, 100)
                                                                            })
                                                                    ).wrap());
                                                                });
                                                        }
                                                    } else {
                                                        // Sequence trigger mode - display captured sequence
                                                        let is_capturing_sequence = self.key_capture_mode == KeyCaptureMode::NewMappingTrigger;
                                                        if is_capturing_sequence {
                                                            // Currently capturing - show animated prompt
                                                            egui::Frame::NONE
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(255, 200, 130)
                                                                } else {
                                                                    egui::Color32::from_rgb(255, 235, 180)
                                                                })
                                                                .corner_radius(egui::CornerRadius::same(12))
                                                                .inner_margin(egui::Margin::symmetric(12, 10))
                                                                .show(ui, |ui| {
                                                                    ui.set_min_width(ui.available_width());
                                                                    // Use vertical layout instead of vertical_centered to avoid staircase effect
                                                                    ui.vertical(|ui| {
                                                                        ui.horizontal(|ui| {
                                                                            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                                                                            ui.label(
                                                                                egui::RichText::new(t.sequence_capturing())
                                                                                    .size(14.0)
                                                                                    .strong()
                                                                                    .color(if self.dark_mode {
                                                                                        egui::Color32::from_rgb(80, 60, 20)
                                                                                    } else {
                                                                                        egui::Color32::from_rgb(100, 80, 20)
                                                                                    }),
                                                                            );
                                                                        });
                                                                        ui.add_space(4.0);
                                                                        if !self.sequence_capture_list.is_empty() {
                                                                            // Show simplified preview: count + last key
                                                                            let count = self.sequence_capture_list.len();
                                                                            let last_key = self.sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                                                            let preview = if count <= 3 {
                                                                                self.sequence_capture_list.join(" → ")
                                                                            } else {
                                                                                format!("{} keys: ... → {}", count, last_key)
                                                                            };
                                                                            ui.horizontal(|ui| {
                                                                                ui.add_space(8.0);
                                                                                ui.label(
                                                                                    egui::RichText::new(&preview)
                                                                                        .size(13.0)
                                                                                        .color(if self.dark_mode {
                                                                                            egui::Color32::from_rgb(60, 40, 10)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(80, 60, 20)
                                                                                        }),
                                                                                );
                                                                            });
                                                                        }
                                                                        ui.add_space(4.0);
                                                                        ui.horizontal(|ui| {
                                                                            ui.add_space(8.0);
                                                                            ui.label(
                                                                                egui::RichText::new(t.sequence_capture_hint())
                                                                                    .size(11.0)
                                                                                    .italics()
                                                                                    .color(if self.dark_mode {
                                                                                        egui::Color32::from_rgb(100, 80, 40)
                                                                                    } else {
                                                                                        egui::Color32::from_rgb(120, 100, 60)
                                                                                    }),
                                                                            );
                                                                        });
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
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(120, 220, 140)
                                                                } else {
                                                                    egui::Color32::from_rgb(140, 230, 150)
                                                                })
                                                                .corner_radius(10.0);
                                                                if ui.add_sized([120.0, 28.0], done_btn).clicked() {
                                                                    // Exit capture mode FIRST to prevent capturing Done button click
                                                                    self.key_capture_mode = KeyCaptureMode::None;
                                                                    self.app_state.set_raw_input_capture_mode(false);
                                                                    // Finish sequence capture
                                                                    if !self.sequence_capture_list.is_empty() {
                                                                        self.new_mapping_trigger = self.sequence_capture_list.join(",");
                                                                    }
                                                                    // Clear capture state
                                                                    self.capture_pressed_keys.clear();
                                                                    self.sequence_last_mouse_pos = None;
                                                                    self.sequence_last_mouse_direction = None;
                                                                    self.sequence_mouse_delta = egui::Vec2::ZERO;
                                                                    // Set flag to prevent capturing the click on Done button
                                                                    self.just_captured_input = true;
                                                                }
                                                                ui.add_space(8.0);
                                                                // Clear button
                                                                let clear_btn = egui::Button::new(
                                                                    egui::RichText::new(t.sequence_clear_btn())
                                                                        .size(13.0)
                                                                        .color(egui::Color32::WHITE),
                                                                )
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(230, 100, 100)
                                                                } else {
                                                                    egui::Color32::from_rgb(250, 150, 150)
                                                                })
                                                                .corner_radius(10.0);
                                                                if ui.add_sized([100.0, 28.0], clear_btn).clicked() {
                                                                    self.sequence_capture_list.clear();
                                                                    self.sequence_last_mouse_pos = None;
                                                                    self.sequence_last_mouse_direction = None;
                                                                    self.sequence_mouse_delta = egui::Vec2::ZERO;
                                                                }
                                                            });
                                                        } else {
                                                            let seq_display = if self.sequence_capture_list.is_empty() {
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
                                                                let count = self.sequence_capture_list.len();
                                                                if count <= 3 {
                                                                    let arrow = t.arrow_icon();
                                                                    self.sequence_capture_list.join(&[' ', arrow.chars().next().unwrap_or('→'), ' '].iter().collect::<String>())
                                                                } else {
                                                                    let last = self.sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                                                    format!("{} keys: ... → {}", count, last)
                                                                }
                                                            };
                                                            let seq_btn = egui::Button::new(
                                                                egui::RichText::new(&seq_display)
                                                                    .size(14.0)
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::WHITE
                                                                    } else {
                                                                        egui::Color32::from_rgb(40, 40, 40)
                                                                    }),
                                                            )
                                                            .fill(if self.dark_mode {
                                                                egui::Color32::from_rgb(60, 62, 72)
                                                            } else {
                                                                egui::Color32::from_rgb(245, 245, 250)
                                                            })
                                                            .corner_radius(10.0);
                                                            if ui.add_sized([ui.available_width(), 30.0], seq_btn).clicked() && !self.just_captured_input {
                                                                // Start sequence capture
                                                                self.key_capture_mode = KeyCaptureMode::NewMappingTrigger;
                                                                self.sequence_capture_list.clear();
                                                                self.sequence_last_mouse_pos = None;
                                                                self.sequence_last_mouse_direction = None;
                                                                self.sequence_mouse_delta = egui::Vec2::ZERO;
                                                                self.capture_pressed_keys.clear();
                                                                self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                                self.app_state.set_raw_input_capture_mode(true);
                                                                self.just_captured_input = true;
                                                                self.duplicate_mapping_error = None;
                                                            }
                                                        }
                                                        // Display captured sequence keys with horizontal flow layout (including during capture)
                                                        if !self.sequence_capture_list.is_empty() {
                                                            ui.add_space(8.0);
                                                            // Get full available width before Frame consumes it
                                                            let full_width = ui.available_width();
                                                            let inner_margin = 10.0;
                                                            egui::Frame::NONE
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                                                                } else {
                                                                    egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                                                                })
                                                                .corner_radius(egui::CornerRadius::same(12))
                                                                .inner_margin(egui::Margin::symmetric(inner_margin as i8, inner_margin as i8))
                                                                .show(ui, |ui| {
                                                                    // Fill the entire available width (content width = full_width - 2*inner_margin)
                                                                    let content_width = full_width - inner_margin * 2.0;
                                                                    ui.set_min_width(content_width);
                                                                    ui.set_max_width(content_width);
                                                                    // Header
                                                                    ui.horizontal(|ui| {
                                                                        let t = &self.translations;
                                                                        ui.label(
                                                                            egui::RichText::new(t.sequence_icon())
                                                                                .size(12.0)
                                                                        );
                                                                        ui.label(
                                                                            egui::RichText::new(t.format_keys_count(self.sequence_capture_list.len()))
                                                                                .size(10.0)
                                                                                .italics()
                                                                                .color(if self.dark_mode {
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
                                                                            let seq_keys: Vec<_> = self.sequence_capture_list.to_vec();
                                                                            let available_width = layout_width;

                                                                            // Pre-calculate rows to avoid staircase effect
                                                                            let mut rows: Vec<Vec<usize>> = Vec::new();
                                                                            let mut current_row: Vec<usize> = Vec::new();
                                                                            let mut current_width = 0.0f32;

                                                                            for (key_idx, key) in seq_keys.iter().enumerate() {
                                                                                let pill_width = estimate_pill_width(key);
                                                                                let arrow_width = if key_idx < seq_keys.len() - 1 { estimate_arrow_width() } else { 0.0 };
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
                                                                                        let (_icon, display_name) = get_sequence_key_display(key);
                                                                                        let tag_color = get_sequence_key_color(key, self.dark_mode);
                                                                                        let text_color = if self.dark_mode {
                                                                                            egui::Color32::from_rgb(40, 30, 50)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(60, 40, 70)
                                                                                        };
                                                                                        // Cute pill-shaped tag (unified style with target pills)
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
                                                                                                    // Display name (unified size with target)
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
                                                                                                            egui::RichText::new("×")
                                                                                                                .size(11.0)
                                                                                                                .color(if self.dark_mode {
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
                                                                                                    .color(if self.dark_mode {
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
                                                                                self.sequence_capture_list.remove(idx);
                                                                            }
                                                                            if self.sequence_capture_list.is_empty() {
                                                                                self.new_mapping_trigger.clear();
                                                                            }
                                                                        });
                                                                });
                                                        }
                                                    }

                                                    ui.add_space(12.0);

                                                    // Target Mode Selection (0=Single, 1=Multi, 2=Sequence)
                                                    let target_mode = self.new_mapping_target_mode;
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(t.target_mode_label())
                                                                .size(13.0)
                                                                .color(if self.dark_mode {
                                                                    egui::Color32::from_rgb(135, 206, 235)
                                                                } else {
                                                                    egui::Color32::from_rgb(70, 130, 180)
                                                                }),
                                                        );
                                                        ui.add_space(8.0);

                                                        // Helper for inactive button styling
                                                        let inactive_fill = if self.dark_mode {
                                                            egui::Color32::from_rgb(50, 52, 62)
                                                        } else {
                                                            egui::Color32::from_rgb(240, 240, 245)
                                                        };
                                                        let inactive_text = if self.dark_mode {
                                                            egui::Color32::from_rgb(180, 180, 200)
                                                        } else {
                                                            egui::Color32::from_rgb(100, 100, 120)
                                                        };

                                                        // Single Button - matches Single badge colors
                                                        let single_btn = egui::Button::new(
                                                            egui::RichText::new(t.target_mode_single())
                                                                .size(11.0)
                                                                .color(if target_mode == 0 {
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(20, 60, 80)
                                                                    } else {
                                                                        egui::Color32::from_rgb(40, 80, 120)
                                                                    }
                                                                } else {
                                                                    inactive_text
                                                                })
                                                        ).fill(if target_mode == 0 {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(135, 206, 235)
                                                            } else {
                                                                egui::Color32::from_rgb(173, 216, 230)
                                                            }
                                                        } else {
                                                            inactive_fill
                                                        }).corner_radius(10.0);
                                                        if ui.add(single_btn).clicked() && target_mode != 0 {
                                                            self.new_mapping_target_mode = 0;
                                                            if self.new_mapping_target_keys.len() > 1 {
                                                                let first = self.new_mapping_target_keys[0].clone();
                                                                self.new_mapping_target_keys.clear();
                                                                self.new_mapping_target_keys.push(first.clone());
                                                                self.new_mapping_target = first;
                                                            }
                                                            self.target_sequence_capture_list.clear();
                                                        }
                                                        ui.add_space(4.0);

                                                        // Multi Button - matches Single badge colors
                                                        let multi_btn = egui::Button::new(
                                                            egui::RichText::new(t.target_mode_multi())
                                                                .size(11.0)
                                                                .color(if target_mode == 1 {
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(20, 60, 80)
                                                                    } else {
                                                                        egui::Color32::from_rgb(40, 80, 120)
                                                                    }
                                                                } else {
                                                                    inactive_text
                                                                })
                                                        ).fill(if target_mode == 1 {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(135, 206, 235)
                                                            } else {
                                                                egui::Color32::from_rgb(173, 216, 230)
                                                            }
                                                        } else {
                                                            inactive_fill
                                                        }).corner_radius(10.0);
                                                        if ui.add(multi_btn).clicked() && target_mode != 1 {
                                                            self.new_mapping_target_mode = 1;
                                                            self.target_sequence_capture_list.clear();
                                                        }
                                                        ui.add_space(4.0);

                                                        // Sequence Button - matches Sequence badge colors
                                                        let seq_btn = egui::Button::new(
                                                            egui::RichText::new(t.target_mode_sequence())
                                                                .size(11.0)
                                                                .color(if target_mode == 2 {
                                                                    if self.dark_mode {
                                                                        egui::Color32::from_rgb(80, 20, 40)
                                                                    } else {
                                                                        egui::Color32::from_rgb(220, 80, 120)
                                                                    }
                                                                } else {
                                                                    inactive_text
                                                                })
                                                        ).fill(if target_mode == 2 {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(255, 182, 193)
                                                            } else {
                                                                egui::Color32::from_rgb(255, 218, 224)
                                                            }
                                                        } else {
                                                            inactive_fill
                                                        }).corner_radius(10.0);
                                                        if ui.add(seq_btn).clicked() && target_mode != 2 {
                                                            self.new_mapping_target_mode = 2;
                                                            // Initialize sequence list from existing keys
                                                            if !self.new_mapping_target_keys.is_empty() {
                                                                self.target_sequence_capture_list = self.new_mapping_target_keys.clone();
                                                            }
                                                        }
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
                                                                .color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
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
                                                                .color(if self.dark_mode {
                                                                    egui::Color32::from_rgb(200, 200, 220)
                                                                } else {
                                                                    egui::Color32::from_rgb(80, 80, 100)
                                                                }),
                                                        );
                                                    });
                                                    ui.add_space(6.0);

                                                    // Determine which list to use based on mode
                                                    let is_capturing_new_target = self.key_capture_mode == KeyCaptureMode::NewMappingTarget;
                                                    let display_keys = if target_mode == 2 {
                                                        &self.target_sequence_capture_list
                                                    } else {
                                                        &self.new_mapping_target_keys
                                                    };
                                                    let separator = if target_mode == 2 { " → " } else { " + " };

                                                    // Use flags to defer mutations until after display_keys reference is no longer used
                                                    let mut should_finish_target_seq = false;
                                                    let mut should_clear_target_seq = false;

                                                    // Sequence mode (target_mode == 2) uses trigger-style capture UI
                                                    if target_mode == 2 {
                                                        if is_capturing_new_target {
                                                            // Currently capturing - show animated prompt (same style as trigger)
                                                            egui::Frame::NONE
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(255, 200, 130)
                                                                } else {
                                                                    egui::Color32::from_rgb(255, 235, 180)
                                                                })
                                                                .corner_radius(egui::CornerRadius::same(12))
                                                                .inner_margin(egui::Margin::symmetric(12, 10))
                                                                .show(ui, |ui| {
                                                                    ui.set_min_width(ui.available_width());
                                                                    ui.vertical(|ui| {
                                                                        ui.horizontal(|ui| {
                                                                            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                                                                            ui.label(
                                                                                egui::RichText::new(t.sequence_capturing())
                                                                                    .size(14.0)
                                                                                    .strong()
                                                                                    .color(if self.dark_mode {
                                                                                        egui::Color32::from_rgb(80, 60, 20)
                                                                                    } else {
                                                                                        egui::Color32::from_rgb(100, 80, 20)
                                                                                    }),
                                                                            );
                                                                        });
                                                                        ui.add_space(4.0);
                                                                        if !self.target_sequence_capture_list.is_empty() {
                                                                            // Show simplified preview: count + last key
                                                                            let count = self.target_sequence_capture_list.len();
                                                                            let last_key = self.target_sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                                                            let preview = if count <= 3 {
                                                                                self.target_sequence_capture_list.join(" → ")
                                                                            } else {
                                                                                format!("{} keys: ... → {}", count, last_key)
                                                                            };
                                                                            ui.horizontal(|ui| {
                                                                                ui.add_space(8.0);
                                                                                ui.label(
                                                                                    egui::RichText::new(&preview)
                                                                                        .size(13.0)
                                                                                        .color(if self.dark_mode {
                                                                                            egui::Color32::from_rgb(60, 40, 10)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(80, 60, 20)
                                                                                        }),
                                                                                );
                                                                            });
                                                                        }
                                                                        ui.add_space(4.0);
                                                                        ui.horizontal(|ui| {
                                                                            ui.add_space(8.0);
                                                                            ui.label(
                                                                                egui::RichText::new(t.sequence_capture_hint())
                                                                                    .size(11.0)
                                                                                    .italics()
                                                                                    .color(if self.dark_mode {
                                                                                        egui::Color32::from_rgb(100, 80, 40)
                                                                                    } else {
                                                                                        egui::Color32::from_rgb(120, 100, 60)
                                                                                    }),
                                                                            );
                                                                        });
                                                                    });
                                                                });
                                                            ui.add_space(8.0);
                                                            ui.horizontal(|ui| {
                                                                // Done button (green)
                                                                let done_btn = egui::Button::new(
                                                                    egui::RichText::new(t.sequence_complete())
                                                                        .size(13.0)
                                                                        .color(egui::Color32::WHITE)
                                                                        .strong(),
                                                                )
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(120, 220, 140)
                                                                } else {
                                                                    egui::Color32::from_rgb(140, 230, 150)
                                                                })
                                                                .corner_radius(10.0);
                                                                if ui.add_sized([120.0, 28.0], done_btn).clicked() {
                                                                    should_finish_target_seq = true;
                                                                }
                                                                ui.add_space(8.0);
                                                                // Clear button (red)
                                                                let clear_btn = egui::Button::new(
                                                                    egui::RichText::new(t.sequence_clear_btn())
                                                                        .size(13.0)
                                                                        .color(egui::Color32::WHITE),
                                                                )
                                                                .fill(if self.dark_mode {
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
                                                            // Not capturing - show sequence capture button
                                                            let seq_display = if self.target_sequence_capture_list.is_empty() {
                                                                t.click_to_set_target().to_string()
                                                            } else {
                                                                let count = self.target_sequence_capture_list.len();
                                                                if count <= 3 {
                                                                    self.target_sequence_capture_list.join(" → ")
                                                                } else {
                                                                    let last = self.target_sequence_capture_list.last().map(|s| s.as_str()).unwrap_or("");
                                                                    format!("{} keys: ... → {}", count, last)
                                                                }
                                                            };
                                                            ui.horizontal(|ui| {
                                                                let seq_btn = egui::Button::new(
                                                                    egui::RichText::new(&seq_display)
                                                                        .size(14.0)
                                                                        .color(if self.dark_mode {
                                                                            egui::Color32::WHITE
                                                                        } else {
                                                                            egui::Color32::from_rgb(40, 40, 40)
                                                                        }),
                                                                )
                                                                .fill(if self.dark_mode {
                                                                    egui::Color32::from_rgb(60, 62, 72)
                                                                } else {
                                                                    egui::Color32::from_rgb(245, 245, 250)
                                                                })
                                                                .corner_radius(10.0);
                                                                if ui.add_sized([ui.available_width() - 70.0, 30.0], seq_btn).clicked() && !self.just_captured_input {
                                                                    self.key_capture_mode = KeyCaptureMode::NewMappingTarget;
                                                                    self.capture_pressed_keys.clear();
                                                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                                    self.app_state.set_raw_input_capture_mode(true);
                                                                    self.just_captured_input = true;
                                                                }
                                                                // +Add button for adding more keys
                                                                if !self.target_sequence_capture_list.is_empty() {
                                                                    ui.add_space(4.0);
                                                                    let add_btn = egui::Button::new(
                                                                        egui::RichText::new(t.add_button_text())
                                                                            .size(12.0)
                                                                            .color(egui::Color32::WHITE),
                                                                    )
                                                                    .fill(if self.dark_mode {
                                                                        egui::Color32::from_rgb(255, 140, 170)
                                                                    } else {
                                                                        egui::Color32::from_rgb(255, 170, 190)
                                                                    })
                                                                    .corner_radius(8.0);
                                                                    if ui.add(add_btn).clicked() && !self.just_captured_input {
                                                                        self.key_capture_mode = KeyCaptureMode::NewMappingTarget;
                                                                        self.capture_pressed_keys.clear();
                                                                        self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                                        self.app_state.set_raw_input_capture_mode(true);
                                                                        self.just_captured_input = true;
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    } else {
                                                        // Single/Multi mode - original button style
                                                        let target_btn_display = if is_capturing_new_target {
                                                            t.press_any_key().to_string()
                                                        } else if display_keys.is_empty() {
                                                            t.click_to_set_target().to_string()
                                                        } else if target_mode == 0 {
                                                            truncate_text_safe(&self.new_mapping_target, BUTTON_TEXT_MAX_CHARS)
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
                                                                } else if self.dark_mode {
                                                                    egui::Color32::WHITE
                                                                } else {
                                                                    egui::Color32::from_rgb(40, 40, 40)
                                                                }),
                                                        )
                                                        .fill(if is_capturing_new_target {
                                                            egui::Color32::from_rgb(70, 130, 180)
                                                        } else if self.dark_mode {
                                                            egui::Color32::from_rgb(60, 62, 72)
                                                        } else {
                                                            egui::Color32::from_rgb(245, 245, 250)
                                                        })
                                                        .corner_radius(10.0);

                                                        if ui.add_sized([ui.available_width(), 30.0], target_btn).clicked() && !self.just_captured_input {
                                                            self.key_capture_mode = KeyCaptureMode::NewMappingTarget;
                                                            self.capture_pressed_keys.clear();
                                                            self.capture_initial_pressed = Self::poll_all_pressed_keys();
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
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgba_premultiplied(255, 182, 193, 25)
                                                            } else {
                                                                egui::Color32::from_rgba_premultiplied(255, 218, 224, 120)
                                                            }
                                                        } else {
                                                            // Multi: blue tint
                                                            if self.dark_mode {
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
                                                                            .color(if self.dark_mode {
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
                                                                        let sep_width = if target_mode == 2 { estimate_arrow_width() } else { 24.0 };

                                                                        // Pre-calculate rows
                                                                        let mut rows: Vec<Vec<usize>> = Vec::new();
                                                                        let mut current_row: Vec<usize> = Vec::new();
                                                                        let mut current_width = 0.0f32;

                                                                        for (key_idx, key) in keys_list.iter().enumerate() {
                                                                            let pill_width = if target_mode == 2 { estimate_pill_width(key) } else { estimate_target_pill_width(key) };
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
                                                                                        let color = get_sequence_key_color(key, self.dark_mode);
                                                                                        let text = if self.dark_mode {
                                                                                            egui::Color32::from_rgb(40, 30, 50)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(60, 40, 70)
                                                                                        };
                                                                                        (color, text)
                                                                                    } else {
                                                                                        let color = get_target_key_color(self.dark_mode);
                                                                                        let text = if self.dark_mode {
                                                                                            egui::Color32::from_rgb(20, 60, 80)
                                                                                        } else {
                                                                                            egui::Color32::from_rgb(40, 100, 140)
                                                                                        };
                                                                                        (color, text)
                                                                                    };
                                                                                    // Get display name for sequence mode
                                                                                    let display_name = if target_mode == 2 {
                                                                                        let (_, name) = get_sequence_key_display(key);
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
                                                                                                let del_btn = ui.add(egui::Button::new(egui::RichText::new("×").size(11.0).color(
                                                                                                    if self.dark_mode { egui::Color32::from_rgb(180, 80, 100) }
                                                                                                    else { egui::Color32::from_rgb(200, 60, 80) }
                                                                                                )).fill(egui::Color32::TRANSPARENT).frame(false).corner_radius(8.0));
                                                                                                if del_btn.clicked() { target_key_to_remove = Some(key_idx); }
                                                                                            });
                                                                                        });
                                                                                    tag_response.response.on_hover_text(key);
                                                                                    if key_idx < keys_list.len() - 1 {
                                                                                        // Sequence mode uses pink arrow like trigger
                                                                                        let sep_color = if target_mode == 2 {
                                                                                            if self.dark_mode { egui::Color32::from_rgb(255, 150, 170) }
                                                                                            else { egui::Color32::from_rgb(255, 120, 150) }
                                                                                        } else if self.dark_mode { egui::Color32::from_rgb(135, 206, 235) }
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
                                                    // Apply deferred target key deletion (outside closures to avoid borrow conflict)
                                                    if let Some(idx) = target_key_to_remove {
                                                        if target_mode == 2 {
                                                            self.target_sequence_capture_list.remove(idx);
                                                            self.new_mapping_target_keys = self.target_sequence_capture_list.clone();
                                                        } else {
                                                            self.new_mapping_target_keys.remove(idx);
                                                        }
                                                        if self.new_mapping_target_keys.is_empty() {
                                                            self.new_mapping_target.clear();
                                                        } else {
                                                            self.new_mapping_target = self.new_mapping_target_keys[0].clone();
                                                        }
                                                    }
                                                    // Apply deferred finish/clear for sequence target capture
                                                    if should_finish_target_seq {
                                                        self.key_capture_mode = KeyCaptureMode::None;
                                                        self.app_state.set_raw_input_capture_mode(false);
                                                        self.capture_pressed_keys.clear();
                                                    }
                                                    if should_clear_target_seq {
                                                        self.target_sequence_capture_list.clear();
                                                        self.new_mapping_target_keys.clear();
                                                        self.new_mapping_target.clear();
                                                    }

                                                    ui.add_space(12.0);

                                                    // Check all target keys to determine what to show
                                                    let has_mouse_move = self.new_mapping_target_keys.iter().any(|k| is_mouse_move_target(k));
                                                    let has_mouse_scroll = self.new_mapping_target_keys.iter().any(|k| is_mouse_scroll_target(k));
                                                    let has_key_or_click = self.new_mapping_target_keys.iter().any(|k| !is_mouse_move_target(k) && !is_mouse_scroll_target(k));

                                                    // Parameters row
                                                    ui.horizontal(|ui| {
                                                        // Always show interval
                                                        ui.label(
                                                            egui::RichText::new(t.interval_short())
                                                                .size(12.0)
                                                                .color(if self.dark_mode {
                                                                    egui::Color32::from_rgb(170, 170, 190)
                                                                } else {
                                                                    egui::Color32::from_rgb(100, 100, 120)
                                                                }),
                                                        );
                                                        let interval_edit = egui::TextEdit::singleline(
                                                            &mut self.new_mapping_interval,
                                                        )
                                                        .background_color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(170, 170, 190)
                                                                    } else {
                                                                        egui::Color32::from_rgb(100, 100, 120)
                                                                    }),
                                                            );
                                                            let duration_edit = egui::TextEdit::singleline(
                                                                &mut self.new_mapping_duration,
                                                            )
                                                            .background_color(if self.dark_mode {
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
                                                                    .color(if self.dark_mode {
                                                                        egui::Color32::from_rgb(170, 170, 190)
                                                                    } else {
                                                                        egui::Color32::from_rgb(100, 100, 120)
                                                                    }),
                                                            );
                                                            let hint = if has_mouse_scroll { "120" } else { "5" };
                                                            let speed_edit = egui::TextEdit::singleline(
                                                                &mut self.new_mapping_move_speed,
                                                            )
                                                            .background_color(if self.dark_mode {
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
                                                        .fill(if self.dark_mode {
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
                                                            self.key_capture_mode = KeyCaptureMode::NewMappingTarget;
                                                            self.capture_pressed_keys.clear();
                                                            self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                                            self.just_captured_input = true;
                                                        }

                                                        // Clear all trigger keys
                                                        let clear_trigger_btn = egui::Button::new(
                                                            egui::RichText::new("✖")
                                                                .color(egui::Color32::WHITE)
                                                                .size(16.0),
                                                        )
                                                        .fill(if self.dark_mode {
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
                                                            self.sequence_capture_list.clear();
                                                            self.new_mapping_trigger.clear();
                                                        }

                                                        // Clear all target keys
                                                        let clear_btn = egui::Button::new(
                                                            egui::RichText::new("✖")
                                                                .color(egui::Color32::WHITE)
                                                                .size(14.0),
                                                        )
                                                        .fill(if self.dark_mode {
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
                                                            self.new_mapping_target_keys.clear();
                                                            self.new_mapping_target.clear();
                                                            self.target_sequence_capture_list.clear();
                                                        }

                                                        // Mouse movement direction
                                                        let move_btn = egui::Button::new(
                                                            egui::RichText::new("⌖")
                                                                .color(egui::Color32::WHITE)
                                                                .size(16.0),
                                                        )
                                                        .fill(if self.dark_mode {
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
                                                            self.mouse_direction_dialog = Some(
                                                                crate::gui::mouse_direction_dialog::MouseDirectionDialog::new(),
                                                            );
                                                            self.mouse_direction_mapping_idx = None;
                                                        }

                                                        // Mouse scroll direction
                                                        let scroll_btn = egui::Button::new(
                                                            egui::RichText::new("🎡")
                                                                .color(egui::Color32::WHITE)
                                                                .size(16.0),
                                                        )
                                                        .fill(if self.dark_mode {
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
                                                            self.mouse_scroll_dialog = Some(
                                                                crate::gui::mouse_scroll_dialog::MouseScrollDialog::new(),
                                                            );
                                                            self.mouse_scroll_mapping_idx = None;
                                                        }

                                                        ui.add_space(4.0);

                                                        // Turbo toggle for new mapping
                                                        let new_turbo_enabled = self.new_mapping_turbo;
                                                        let new_turbo_color = if new_turbo_enabled {
                                                            if self.dark_mode {
                                                                egui::Color32::from_rgb(250, 200, 80)
                                                            } else {
                                                                egui::Color32::from_rgb(255, 220, 120)
                                                            }
                                                        } else if self.dark_mode {
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
                                                            self.translations.turbo_on_hover()
                                                        } else {
                                                            self.translations.turbo_off_hover()
                                                        };

                                                        if ui
                                                            .add_sized([36.0, button_height], new_turbo_btn)
                                                            .on_hover_text(new_hover_text)
                                                            .clicked()
                                                        {
                                                            self.new_mapping_turbo =
                                                                !self.new_mapping_turbo;
                                                        }

                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            let add_btn = egui::Button::new(
                                                                egui::RichText::new(t.add_button_text())
                                                                    .size(14.0)
                                                                    .color(egui::Color32::WHITE)
                                                                    .strong(),
                                                            )
                                                            .fill(if self.dark_mode {
                                                                egui::Color32::from_rgb(100, 220, 180)
                                                            } else {
                                                                egui::Color32::from_rgb(120, 240, 200)
                                                            })
                                                            .corner_radius(12.0);

                                                            let has_trigger = !self.new_mapping_trigger.is_empty() || !self.sequence_capture_list.is_empty();
                                                            if ui.add_sized([80.0, button_height], add_btn).clicked()
                                                                && has_trigger
                                                                && !self.new_mapping_target_keys.is_empty()
                                                            {
                                                                let is_sequence_mode = !self.sequence_capture_list.is_empty();
                                                                let (trigger_key, trigger_sequence) = if is_sequence_mode {
                                                                    // Sequence mode: use first key as trigger_key, full sequence as trigger_sequence
                                                                    let trigger = self.sequence_capture_list[0].to_uppercase();
                                                                    let sequence = self.sequence_capture_list.iter()
                                                                        .map(|k| k.to_uppercase())
                                                                        .collect::<Vec<_>>()
                                                                        .join(",");
                                                                    (trigger, Some(sequence))
                                                                } else {
                                                                    // Single key mode
                                                                    (self.new_mapping_trigger.to_uppercase(), None)
                                                                };

                                                                // Check for duplicate trigger key
                                                                let is_duplicate = temp_config
                                                                    .mappings
                                                                    .iter()
                                                                    .any(|m| m.trigger_key == trigger_key);

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
                                                            .unwrap_or(5)
                                                            .clamp(1, 100);
                                                        let sequence_window = self
                                                            .new_mapping_sequence_window
                                                            .parse::<u64>()
                                                            .unwrap_or(300)
                                                            .max(50);

                                                        let turbo_enabled = self.new_mapping_turbo;

                                                        temp_config.mappings.push(KeyMapping {
                                                            trigger_key,
                                                            trigger_sequence,
                                                            sequence_window_ms: sequence_window,
                                                            target_keys: self.new_mapping_target_keys.iter()
                                                                .map(|k| k.to_uppercase())
                                                                .collect(),
                                                            interval,
                                                            event_duration: duration,
                                                            turbo_enabled,
                                                            move_speed,
                                                            target_mode: self.new_mapping_target_mode,
                                                        });

                                                        // Clear input fields
                                                        self.new_mapping_trigger.clear();
                                                        self.new_mapping_target.clear();
                                                        self.new_mapping_target_keys.clear();
                                                        self.new_mapping_interval.clear();
                                                        self.new_mapping_duration.clear();
                                                        self.new_mapping_move_speed = "5".to_string();
                                                        self.new_mapping_turbo = true; // Reset to default
                                                        self.sequence_capture_list.clear();
                                                        self.new_mapping_sequence_window = "300".to_string();
                                                        self.new_mapping_is_sequence_mode = false;
                                                        self.new_mapping_target_mode = 0;
                                                        self.target_sequence_capture_list.clear();
                                                        self.sequence_last_mouse_pos = None;
                                                        self.sequence_last_mouse_direction = None;
                                                        self.sequence_mouse_delta = egui::Vec2::ZERO;
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
                                        });

                                    ui.add_space(8.0);

                                    // Process Whitelist Section
                                    let card_bg = if self.dark_mode {
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
                                                egui::RichText::new(t.process_whitelist_hint())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(if self.dark_mode {
                                                        egui::Color32::from_rgb(200, 180, 255)
                                                    } else {
                                                        egui::Color32::from_rgb(150, 100, 200)
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
                                                                    let t = &self.translations;
                                                                    let del_btn = egui::Button::new(
                                                                    egui::RichText::new(t.delete_icon())
                                                                        .color(egui::Color32::WHITE)
                                                                        .size(11.0),
                                                                )
                                                                .fill(egui::Color32::from_rgb(
                                                                    255, 182, 193,
                                                                ))
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
                                                .fill(if self.dark_mode {
                                                    egui::Color32::from_rgb(100, 220, 180)
                                                } else {
                                                    egui::Color32::from_rgb(120, 240, 200)
                                                })
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
                                                .fill(if self.dark_mode {
                                                    egui::Color32::from_rgb(180, 160, 230)
                                                } else {
                                                    egui::Color32::from_rgb(210, 190, 240)
                                                })
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
                                    .size(14.0)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(if self.dark_mode {
                                egui::Color32::from_rgb(120, 220, 140)
                            } else {
                                egui::Color32::from_rgb(140, 230, 150)
                            })
                            .corner_radius(15.0);

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
                            .fill(if self.dark_mode {
                                egui::Color32::from_rgb(220, 180, 210)
                            } else {
                                egui::Color32::from_rgb(230, 200, 220)
                            })
                            .corner_radius(15.0);

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

                    // Re-parse switch key after configuration update
                    self.parsed_switch_key = Self::parse_switch_key(&self.config.switch_key);

                    // Apply theme change immediately
                    if dark_mode_changed {
                        self.dark_mode = self.config.dark_mode;
                    }

                    if language_changed {
                        self.update_translations(self.config.language);
                        crate::gui::fonts::load_fonts(ctx, self.config.language);
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
            self.new_mapping_target_keys.clear();
            self.new_mapping_interval.clear();
            self.new_mapping_duration.clear();
            self.sequence_capture_list.clear();
            self.new_mapping_is_sequence_mode = false;
            self.new_mapping_target_mode = 0;
            self.target_sequence_capture_list.clear();
            self.editing_target_seq_list.clear();
            self.editing_target_seq_idx = None;
            self.sequence_last_mouse_pos = None;
            self.sequence_last_mouse_direction = None;
            self.sequence_mouse_delta = egui::Vec2::ZERO;

            // Restore previous paused state after exiting settings
            if let Some(was_paused) = self.was_paused_before_settings.take()
                && !was_paused
            {
                // Resume key repeat without notification (silent resume)
                self.app_state.set_paused(false);
            }
        }
        });
    }
}
