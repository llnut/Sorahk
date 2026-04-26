//! Settings dialog implementation.

mod general;
mod helpers;
mod mapping_editor;
mod mapping_list;
mod process_list;

use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;
use eframe::egui;

/// Resets capture mode and clears per-session sequence scratch state.
///
/// Implemented as a macro instead of a `&mut self` method so it can run
/// inside closures that already hold a disjoint sub-field borrow on
/// `self`. The expanded field accesses are tracked independently under
/// Rust split-borrow rules, while a method call would require a full
/// mutable borrow.
macro_rules! finalize_sequence_capture {
    ($this:ident) => {{
        $this.key_capture_mode = KeyCaptureMode::None;
        $this.app_state.set_raw_input_capture_mode(false);
        $this.capture_pressed_keys.clear();
        $this.just_captured_input = true;
        $this.sequence_last_mouse_pos = None;
        $this.sequence_last_mouse_direction = None;
        $this.sequence_mouse_delta = egui::Vec2::ZERO;
    }};
}

// `mod capture;` is declared after the macro so capture.rs can invoke
// `finalize_sequence_capture!`. Textual macro_rules! scope only
// forwards into submodules declared after the macro is in scope.
mod capture;

impl SorahkGui {
    /// Renders the settings dialog for configuration management.
    pub(super) fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_cancel = false;

        self.poll_capture(ctx);


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
                    // `CachedTranslations` is a Copy newtype over a static
                    // pointer. Binding by value drops the borrow on
                    // `self.translations`, so the closure can still call
                    // `&mut self` helpers like `finalize_sequence_capture!`.
                    let t = self.translations;

                    ui.horizontal(|ui| {
                        ui.add_space(15.0);

                        ui.label(
                            egui::RichText::new(t.settings_dialog_title())
                                .size(18.0)
                                .strong()
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(176, 224, 230)
                                } else {
                                    egui::Color32::from_rgb(135, 206, 235)
                                }),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0);

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

                    let temp_config = self.temp_config.as_mut().unwrap();
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(12, 0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(500.0)
                                .show(ui, |ui| {
                                    general::render_general_section(
                                        ui,
                                        temp_config,
                                        &mut self.key_capture_mode,
                                        &mut self.capture_initial_pressed,
                                        &mut self.capture_pressed_keys,
                                        &mut self.just_captured_input,
                                        &self.app_state,
                                        self.dark_mode,
                                        self.translations,
                                    );

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

                                            mapping_list::render_mapping_list_section(
                                                ui,
                                                temp_config,
                                                &mut self.key_capture_mode,
                                                &mut self.capture_initial_pressed,
                                                &mut self.capture_pressed_keys,
                                                &mut self.just_captured_input,
                                                &mut self.editing_target_seq_idx,
                                                &mut self.editing_target_seq_list,
                                                &mut self.sequence_last_mouse_pos,
                                                &mut self.sequence_last_mouse_direction,
                                                &mut self.sequence_mouse_delta,
                                                &mut self.mouse_direction_dialog,
                                                &mut self.mouse_direction_mapping_idx,
                                                &mut self.mouse_scroll_dialog,
                                                &mut self.mouse_scroll_mapping_idx,
                                                &mut self.rule_properties_dialog,
                                                &mut self.rule_props_editing_idx,
                                                &self.app_state,
                                                self.dark_mode,
                                                self.translations,
                                            );

                                            ui.add_space(12.0);
                                            ui.separator();
                                            ui.add_space(12.0);


                                            mapping_editor::render_mapping_editor_section(
                                                ui,
                                                temp_config,
                                                &mut self.key_capture_mode,
                                                &mut self.capture_initial_pressed,
                                                &mut self.capture_pressed_keys,
                                                &mut self.just_captured_input,
                                                &mut self.duplicate_mapping_error,
                                                &mut self.sequence_last_mouse_pos,
                                                &mut self.sequence_last_mouse_direction,
                                                &mut self.sequence_mouse_delta,
                                                &mut self.sequence_capture_list,
                                                &mut self.target_sequence_capture_list,
                                                &mut self.mouse_direction_dialog,
                                                &mut self.mouse_direction_mapping_idx,
                                                &mut self.mouse_scroll_dialog,
                                                &mut self.mouse_scroll_mapping_idx,
                                                &mut self.rule_properties_dialog,
                                                &mut self.rule_props_editing_idx,
                                                &mut self.new_mapping_trigger,
                                                &mut self.new_mapping_target,
                                                &mut self.new_mapping_target_keys,
                                                &mut self.new_mapping_target_mode,
                                                &mut self.new_mapping_interval,
                                                &mut self.new_mapping_duration,
                                                &mut self.new_mapping_move_speed,
                                                &mut self.new_mapping_sequence_window,
                                                &mut self.new_mapping_is_sequence_mode,
                                                &mut self.new_mapping_turbo,
                                                &mut self.new_mapping_hold_indices,
                                                &mut self.new_mapping_append_keys,
                                                &self.app_state,
                                                self.dark_mode,
                                                self.translations,
                                            );

                                    ui.add_space(8.0);

                                    process_list::render_process_list_section(
                                        ui,
                                        temp_config,
                                        &mut self.new_process_name,
                                        &mut self.duplicate_process_error,
                                        self.dark_mode,
                                        self.translations,
                                    );
                                }); // End of ScrollArea
                        }); // End of Frame

                    ui.separator();

                    // Action buttons. Pinned to the bottom outside the scroll area.
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
                            .fill(crate::gui::theme::colors(self.dark_mode).accent_success)
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
                            .fill(crate::gui::theme::colors(self.dark_mode).accent_secondary)
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
                    // Reload configuration into AppState. Takes effect immediately.
                    let _ = self.app_state.reload_config(temp_config.clone());

                    // Update GUI's config
                    self.config = temp_config.clone();

                    // The GUI polls the switch key via GetAsyncKeyState
                    // using a cached parse result. Rebuild it from the new
                    // config so the polling loop picks up the new hotkey
                    // without an app restart.
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
            // Clear rule-properties draft so a leftover selection from a
            // previous session doesn't auto-apply to the next draft.
            self.new_mapping_hold_indices.clear();
            self.new_mapping_append_keys.clear();
            // Drop any in-flight rule-properties dialog so it can't outlive
            // the settings session and write into a stale temp_config.
            self.rule_properties_dialog = None;
            self.rule_props_editing_idx = None;

            // Restore previous paused state after exiting settings
            if let Some(was_paused) = self.was_paused_before_settings.take()
                && !was_paused
            {
                // Resume key repeat silently, no notification.
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
            self.new_mapping_hold_indices.clear();
            self.new_mapping_append_keys.clear();
            self.rule_properties_dialog = None;
            self.rule_props_editing_idx = None;
            self.sequence_last_mouse_pos = None;
            self.sequence_last_mouse_direction = None;
            self.sequence_mouse_delta = egui::Vec2::ZERO;

            // Restore previous paused state after exiting settings
            if let Some(was_paused) = self.was_paused_before_settings.take()
                && !was_paused
            {
                // Resume key repeat silently, no notification.
                self.app_state.set_paused(false);
            }
        }
        });
    }
}
