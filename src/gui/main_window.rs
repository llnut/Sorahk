//! Main window implementation and rendering logic.

use crate::gui::SorahkGui;
use crate::gui::about_dialog::render_about_dialog;
use crate::gui::utils::string_to_key;
use crate::state::NotificationEvent;
use eframe::egui;

/// Cached frame state to avoid repeated atomic operations.
struct FrameState {
    is_paused: bool,
    worker_count: usize,
    should_exit: bool,
}

impl eframe::App for SorahkGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Cache frame state at the beginning to avoid repeated atomic operations
        let frame_state = FrameState {
            is_paused: self.app_state.is_paused(),
            worker_count: self.app_state.get_actual_worker_count(),
            should_exit: self.app_state.should_exit(),
        };

        // Check if exit was requested at the very beginning
        if frame_state.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Check for HID device activation requests
        if self.hid_activation_dialog.is_none() {
            let requests = self.app_state.poll_hid_activation_requests();
            if let Some((device_handle, device_name)) = requests.first() {
                self.hid_activation_dialog =
                    Some(crate::gui::hid_activation_dialog::HidActivationDialog::new(
                        device_name.clone(),
                        *device_handle,
                    ));
                // Record creation time for 100ms debounce
                self.hid_activation_creation_time = Some(std::time::Instant::now());
            }
        }

        // Render HID activation dialog if present
        if let Some(dialog) = &mut self.hid_activation_dialog {
            let is_debouncing = if let Some(creation_time) = self.hid_activation_creation_time {
                creation_time.elapsed().as_millis() < 200
            } else {
                false
            };

            if is_debouncing {
                while self
                    .app_state
                    .try_recv_hid_activation_data(dialog.device_handle())
                    .is_some()
                {
                    // Discard all data during debounce period
                }
            } else {
                while let Some(hid_data) = self
                    .app_state
                    .try_recv_hid_activation_data(dialog.device_handle())
                {
                    dialog.handle_hid_data(&hid_data);
                }
            }

            let should_close = dialog.render(ctx, self.dark_mode, &self.translations);

            if should_close {
                // Save baseline to config if successful
                if let Some(baseline) = dialog.get_baseline() {
                    crate::rawinput::activate_hid_device(dialog.device_handle(), baseline.clone());

                    // Get device info for stable identifier (VID:PID:Serial)
                    if let Some((vid, pid, serial)) =
                        crate::rawinput::get_device_info_for_handle(dialog.device_handle())
                    {
                        // Create device ID string using format: "VID:PID" or "VID:PID:Serial"
                        let device_id = if let Some(ref serial) = serial {
                            format!("{:04X}:{:04X}:{}", vid, pid, serial)
                        } else {
                            format!("{:04X}:{:04X}", vid, pid)
                        };

                        // Check if device already exists in config (avoid duplicates)
                        if !self
                            .config
                            .hid_baselines
                            .iter()
                            .any(|b| b.device_id == device_id)
                        {
                            // Add to config for persistence
                            self.config
                                .hid_baselines
                                .push(crate::config::HidDeviceBaseline {
                                    device_id,
                                    baseline_data: baseline,
                                });

                            // Save config
                            let _ = self.config.save_to_file("Config.toml");
                        }
                    }
                }

                // Clear activating device handle
                self.app_state.clear_activating_device();
                self.hid_activation_dialog = None;
                self.hid_activation_creation_time = None;
            }
        }

        // Apply cached visuals based on theme
        let visuals = if self.dark_mode {
            &self.cached_dark_visuals
        } else {
            &self.cached_light_visuals
        };
        ctx.set_visuals(visuals.clone());

        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Handle window visibility requests
        if self.app_state.check_and_clear_show_window_request() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        if self.app_state.check_and_clear_show_about_request() {
            self.show_about_dialog = true;
        }

        // Handle close dialog
        self.handle_close_dialog(ctx, &frame_state);

        // Show dialogs
        if self.show_settings_dialog {
            self.render_settings_dialog(ctx);
        }

        if self.show_about_dialog {
            render_about_dialog(
                ctx,
                self.dark_mode,
                &mut self.show_about_dialog,
                &self.translations,
            );
        }

        // Handle mouse direction dialog
        if let Some(dialog) = &mut self.mouse_direction_dialog {
            let should_close = dialog.render(ctx, self.dark_mode, &self.translations);

            if should_close {
                if let Some(selected) = dialog.get_selected_direction() {
                    // Apply the selected direction
                    if let Some(idx) = self.mouse_direction_mapping_idx {
                        // Editing existing mapping
                        if let Some(temp_config) = &mut self.temp_config
                            && let Some(mapping) = temp_config.mappings.get_mut(idx)
                        {
                            mapping.target_key = selected;
                        }
                    } else {
                        // New mapping
                        self.new_mapping_target = selected;
                    }
                }
                self.mouse_direction_dialog = None;
                self.mouse_direction_mapping_idx = None;
            }
        }

        // Handle keyboard input
        self.handle_keyboard_input(ctx);

        // Render main content
        self.render_main_content(ctx, &frame_state);

        // Check exit
        if frame_state.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_state.exit();
    }
}

impl SorahkGui {
    /// Handles close dialog display and interaction logic.
    fn handle_close_dialog(&mut self, ctx: &egui::Context, frame_state: &FrameState) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if frame_state.should_exit {
                // Allow close
            } else if self.minimize_on_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

                if !self.show_close_dialog {
                    self.show_close_dialog = true;
                    self.dialog_highlight_until = None;
                } else {
                    self.dialog_highlight_until =
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
                    ctx.request_repaint();
                }
            }
        }

        if self.show_close_dialog {
            self.render_close_dialog(ctx);
        }
    }

    /// Renders the close confirmation dialog.
    fn render_close_dialog(&mut self, ctx: &egui::Context) {
        let t = &self.translations;
        let should_highlight = self
            .dialog_highlight_until
            .map(|until| std::time::Instant::now() < until)
            .unwrap_or(false);

        if should_highlight {
            ctx.request_repaint();
        }

        let dialog_bg = if self.dark_mode {
            egui::Color32::from_rgb(32, 34, 45)
        } else {
            egui::Color32::from_rgb(245, 240, 252)
        };

        let window = egui::Window::new("")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(dialog_bg)
                    .corner_radius(egui::CornerRadius::same(20))
                    .stroke(if should_highlight {
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 200, 0))
                    } else {
                        egui::Stroke::NONE
                    })
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4],
                        blur: 10,
                        spread: 2,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 40),
                    }),
            );

        window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(25.0);

                ui.label(
                    egui::RichText::new(t.close_window_title())
                        .size(22.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(255, 182, 193)
                        } else {
                            egui::Color32::from_rgb(219, 112, 147)
                        }),
                );

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(t.close_subtitle())
                        .size(13.0)
                        .italics()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(180, 180, 180)
                        } else {
                            egui::Color32::from_rgb(120, 120, 120)
                        }),
                );

                ui.add_space(30.0);

                let button_width = 320.0;
                let button_height = 32.0;
                let tray_enabled = self.config.show_tray_icon;

                if tray_enabled {
                    let minimize_btn = egui::Button::new(
                        egui::RichText::new(t.minimize_to_tray_button())
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(135, 206, 235))
                    .corner_radius(15.0);

                    if ui
                        .add_sized([button_width, button_height], minimize_btn)
                        .clicked()
                    {
                        self.show_close_dialog = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    ui.add_space(12.0);
                }

                let exit_btn = egui::Button::new(
                    egui::RichText::new(t.exit_program_button())
                        .size(14.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                )
                .fill(egui::Color32::from_rgb(255, 182, 193))
                .corner_radius(15.0);

                if ui
                    .add_sized([button_width, button_height], exit_btn)
                    .clicked()
                {
                    self.show_close_dialog = false;
                    self.app_state.exit();
                }

                ui.add_space(12.0);

                let cancel_btn = egui::Button::new(
                    egui::RichText::new(t.cancel_close_button())
                        .size(13.0)
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(200, 200, 200)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        }),
                )
                .fill(if self.dark_mode {
                    egui::Color32::from_rgb(60, 60, 60)
                } else {
                    egui::Color32::from_rgb(230, 230, 230)
                })
                .corner_radius(10.0);

                if ui
                    .add_sized([button_width, button_height], cancel_btn)
                    .clicked()
                {
                    self.show_close_dialog = false;
                }

                ui.add_space(15.0);
            });
        });
    }

    /// Handles global hotkey input events.
    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if let Some(switch_key) = string_to_key(&self.config.switch_key)
                && i.key_pressed(switch_key)
            {
                let was_paused = self.app_state.toggle_paused();
                if let Some(sender) = self.app_state.get_notification_sender() {
                    let msg = if was_paused {
                        "Sorahk activiting"
                    } else {
                        "Sorahk paused"
                    };
                    let _ = sender.send(NotificationEvent::Info(msg.to_string()));
                }
            }
        });
    }

    /// Renders the main content panel with all UI components.
    fn render_main_content(&mut self, ctx: &egui::Context, frame_state: &FrameState) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_title_bar(ui);

            ui.add_space(10.0);

            // Add scroll area for main content to allow vertical scrolling
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    self.render_status_card(ui, frame_state);
                    ui.add_space(10.0);
                    self.render_hotkey_card(ui);
                    ui.add_space(10.0);
                    self.render_config_card(ui);
                    ui.add_space(10.0);
                    self.render_mappings_card(ui);
                    ui.add_space(10.0);
                });
        });
    }

    /// Renders the title bar with theme toggle and menu buttons.
    fn render_title_bar(&mut self, ui: &mut egui::Ui) {
        let t = &self.translations;

        ui.horizontal(|ui| {
            ui.add_space(15.0);

            ui.label(
                egui::RichText::new(t.app_title())
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

                let theme_icon = if self.dark_mode { "☀" } else { "🌙" };
                let theme_text = if self.dark_mode {
                    t.dark_theme()
                } else {
                    t.light_theme()
                };

                let theme_btn = egui::Button::new(
                    egui::RichText::new(format!("{}  {}", theme_icon, theme_text))
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(if self.dark_mode {
                    egui::Color32::from_rgb(255, 200, 100)
                } else {
                    egui::Color32::from_rgb(100, 100, 180)
                })
                .corner_radius(12.0);

                if ui.add(theme_btn).clicked() {
                    self.dark_mode = !self.dark_mode;
                    self.config.dark_mode = self.dark_mode;
                    let _ = self.config.save_to_file("Config.toml");
                    if let Some(temp_config) = &mut self.temp_config {
                        temp_config.dark_mode = self.dark_mode;
                    }
                }

                ui.add_space(8.0);

                let settings_btn = egui::Button::new(
                    egui::RichText::new(t.settings_button())
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(135, 206, 235))
                .corner_radius(12.0);

                if ui.add(settings_btn).clicked() {
                    // Save current paused state before entering settings
                    let was_paused = self.app_state.is_paused();
                    self.was_paused_before_settings = Some(was_paused);

                    // Pause key repeat when entering settings to avoid interference with input
                    if !was_paused {
                        self.app_state.set_paused(true);
                    }

                    self.show_settings_dialog = true;
                    self.temp_config = Some(self.config.clone());
                }

                ui.add_space(8.0);

                let about_btn = egui::Button::new(
                    egui::RichText::new(t.about_button())
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(216, 191, 216))
                .corner_radius(12.0);

                if ui.add(about_btn).clicked() {
                    self.show_about_dialog = true;
                }
            });
        });
    }

    /// Renders the status card with pause/resume and exit controls.
    fn render_status_card(&mut self, ui: &mut egui::Ui, frame_state: &FrameState) {
        let t = &self.translations;
        let card_bg = if self.dark_mode {
            egui::Color32::from_rgb(42, 38, 48)
        } else {
            egui::Color32::from_rgb(255, 245, 250)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(egui::CornerRadius::same(16))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t.status_title())
                            .size(16.0)
                            .strong()
                            .color(if self.dark_mode {
                                egui::Color32::from_rgb(255, 182, 193)
                            } else {
                                egui::Color32::from_rgb(220, 20, 60)
                            }),
                    );

                    ui.add_space(10.0);

                    let (icon, text, color) = if frame_state.is_paused {
                        ("⏸", t.paused_status(), egui::Color32::from_rgb(255, 140, 0))
                    } else {
                        (
                            "▶",
                            t.running_status(),
                            egui::Color32::from_rgb(34, 139, 34),
                        )
                    };

                    ui.label(egui::RichText::new(icon).size(18.0).color(color));
                    ui.label(egui::RichText::new(text).size(15.0).color(color).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if frame_state.worker_count > 0 {
                            ui.label(
                                egui::RichText::new(
                                    t.format_worker_count(frame_state.worker_count),
                                )
                                .size(13.0)
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(135, 206, 235)
                                } else {
                                    egui::Color32::from_rgb(70, 130, 180)
                                }),
                            );
                        }
                    });
                });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    let width = 140.0;
                    let height = 32.0;

                    let (text, color) = if frame_state.is_paused {
                        (t.start_button(), egui::Color32::from_rgb(144, 238, 144))
                    } else {
                        (t.pause_button(), egui::Color32::from_rgb(255, 218, 185))
                    };

                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(text)
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(color)
                    .corner_radius(15.0);

                    if ui.add_sized([width, height], toggle_btn).clicked() {
                        let was_paused = self.app_state.toggle_paused();
                        if let Some(sender) = self.app_state.get_notification_sender() {
                            let msg = if was_paused {
                                "Sorahk activiting"
                            } else {
                                "Sorahk paused"
                            };
                            let _ = sender.send(NotificationEvent::Info(msg.to_string()));
                        }
                    }

                    ui.add_space(15.0);

                    let exit_btn = egui::Button::new(
                        egui::RichText::new(t.exit_button())
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(255, 182, 193))
                    .corner_radius(15.0);

                    if ui.add_sized([width, height], exit_btn).clicked() {
                        self.app_state.exit();
                        std::process::exit(0);
                    }
                });
            });
    }

    /// Renders the hotkey settings card displaying the toggle key.
    fn render_hotkey_card(&self, ui: &mut egui::Ui) {
        let t = &self.translations;
        let card_bg = if self.dark_mode {
            egui::Color32::from_rgb(35, 42, 50)
        } else {
            egui::Color32::from_rgb(240, 248, 255)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(egui::CornerRadius::same(16))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.label(
                    egui::RichText::new(t.hotkey_settings_title())
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(173, 216, 230)
                        } else {
                            egui::Color32::from_rgb(30, 90, 180)
                        }),
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t.toggle_key_label()).size(14.0).color(
                        if self.dark_mode {
                            egui::Color32::from_rgb(200, 200, 200)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        },
                    ));
                    ui.label(
                        egui::RichText::new(&self.config.switch_key)
                            .size(15.0)
                            .color(if self.dark_mode {
                                egui::Color32::from_rgb(135, 206, 235)
                            } else {
                                egui::Color32::from_rgb(0, 100, 200)
                            })
                            .strong(),
                    );
                });
            });
    }

    /// Renders the global configuration card with application settings.
    fn render_config_card(&self, ui: &mut egui::Ui) {
        let t = &self.translations;
        let card_bg = if self.dark_mode {
            egui::Color32::from_rgb(48, 42, 38)
        } else {
            egui::Color32::from_rgb(255, 248, 240)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(egui::CornerRadius::same(16))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.label(
                    egui::RichText::new(t.config_settings_title())
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(255, 218, 185)
                        } else {
                            egui::Color32::from_rgb(200, 100, 0)
                        }),
                );

                ui.add_space(8.0);

                let available = ui.available_width();
                egui::Grid::new("config_grid")
                    .num_columns(2)
                    .spacing([30.0, 8.0])
                    .min_col_width(available * 0.4)
                    .striped(false)
                    .show(ui, |ui| {
                        self.render_config_row(
                            ui,
                            t.input_timeout_display(),
                            &format!("{} ms", self.config.input_timeout),
                        );
                        self.render_config_row(
                            ui,
                            t.default_interval_display(),
                            &format!("{} ms", self.config.interval),
                        );
                        self.render_config_row(
                            ui,
                            t.default_duration_display(),
                            &format!("{} ms", self.config.event_duration),
                        );
                        self.render_bool_row(
                            ui,
                            t.show_tray_icon_display(),
                            self.config.show_tray_icon,
                        );
                        self.render_bool_row(
                            ui,
                            t.show_notifications_display(),
                            self.config.show_notifications,
                        );
                        self.render_bool_row(
                            ui,
                            t.always_on_top_display(),
                            self.config.always_on_top,
                        );
                    });
            });
    }

    /// Renders a single configuration row with label and value.
    fn render_config_row(&self, ui: &mut egui::Ui, label: &str, value: &str) {
        ui.label(
            egui::RichText::new(label)
                .size(14.0)
                .color(if self.dark_mode {
                    egui::Color32::from_rgb(200, 200, 200)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                }),
        );
        ui.label(
            egui::RichText::new(value)
                .size(14.0)
                .color(if self.dark_mode {
                    egui::Color32::from_rgb(135, 206, 235)
                } else {
                    egui::Color32::from_rgb(0, 100, 200)
                }),
        );
        ui.end_row();
    }

    /// Renders a single boolean configuration row with checkmark.
    fn render_bool_row(&self, ui: &mut egui::Ui, label: &str, value: bool) {
        let t = &self.translations;
        ui.label(
            egui::RichText::new(label)
                .size(14.0)
                .color(if self.dark_mode {
                    egui::Color32::from_rgb(200, 200, 200)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                }),
        );
        let text = if value { t.yes() } else { t.no() };
        let color = if value {
            if self.dark_mode {
                egui::Color32::from_rgb(144, 238, 144)
            } else {
                egui::Color32::from_rgb(34, 139, 34)
            }
        } else if self.dark_mode {
            egui::Color32::from_rgb(255, 182, 193)
        } else {
            egui::Color32::from_rgb(220, 20, 60)
        };
        ui.label(egui::RichText::new(text).size(14.0).color(color));
        ui.end_row();
    }

    /// Renders the key mappings card showing all configured mappings.
    fn render_mappings_card(&self, ui: &mut egui::Ui) {
        let t = &self.translations;
        let card_bg = if self.dark_mode {
            egui::Color32::from_rgb(35, 45, 40)
        } else {
            egui::Color32::from_rgb(240, 255, 245)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(egui::CornerRadius::same(16))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.label(
                    egui::RichText::new(t.key_mappings_title())
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(152, 251, 152)
                        } else {
                            egui::Color32::from_rgb(0, 120, 0)
                        }),
                );
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        let available = ui.available_width();
                        egui::Grid::new("mappings_grid")
                            .num_columns(5)
                            .spacing([15.0, 6.0])
                            .min_col_width(available * 0.18)
                            .striped(true)
                            .show(ui, |ui| {
                                // Header
                                self.render_mapping_header(ui);

                                // Mappings
                                for mapping in &self.config.mappings {
                                    ui.label(egui::RichText::new(&mapping.trigger_key).color(
                                        if self.dark_mode {
                                            egui::Color32::from_rgb(255, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(180, 80, 0)
                                        },
                                    ));
                                    ui.label(egui::RichText::new(&mapping.target_key).color(
                                        if self.dark_mode {
                                            egui::Color32::from_rgb(100, 200, 255)
                                        } else {
                                            egui::Color32::from_rgb(0, 80, 180)
                                        },
                                    ));
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}",
                                            mapping.interval.unwrap_or(self.config.interval)
                                        ))
                                        .color(
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(200, 200, 200)
                                            } else {
                                                egui::Color32::from_rgb(60, 60, 60)
                                            },
                                        ),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}",
                                            mapping
                                                .event_duration
                                                .unwrap_or(self.config.event_duration)
                                        ))
                                        .color(
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(200, 200, 200)
                                            } else {
                                                egui::Color32::from_rgb(60, 60, 60)
                                            },
                                        ),
                                    );

                                    // Turbo status
                                    let (turbo_icon, turbo_color) = if mapping.turbo_enabled {
                                        (
                                            "⚡",
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(100, 200, 255)
                                            } else {
                                                egui::Color32::from_rgb(0, 120, 220)
                                            },
                                        )
                                    } else {
                                        (
                                            "○",
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(120, 120, 120)
                                            } else {
                                                egui::Color32::from_rgb(160, 160, 160)
                                            },
                                        )
                                    };
                                    ui.label(
                                        egui::RichText::new(turbo_icon)
                                            .size(16.0)
                                            .color(turbo_color),
                                    );

                                    ui.end_row();
                                }
                            });
                    });
            });
    }

    /// Renders the header row for the key mappings table.
    fn render_mapping_header(&self, ui: &mut egui::Ui) {
        let t = &self.translations;
        let headers = [
            t.trigger_header(),
            t.target_header(),
            t.interval_header(),
            t.duration_header(),
            t.turbo_header(),
        ];
        for header in &headers {
            ui.label(
                egui::RichText::new(*header)
                    .strong()
                    .color(if self.dark_mode {
                        egui::Color32::from_rgb(220, 220, 220)
                    } else {
                        egui::Color32::from_rgb(40, 40, 40)
                    }),
            );
        }
        ui.end_row();
    }
}
