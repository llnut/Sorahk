// Main window implementation

use crate::gui::SorahkGui;
use crate::gui::about_dialog::render_about_dialog;
use crate::gui::utils::string_to_key;
use crate::state::NotificationEvent;
use eframe::egui;

// Implementation of main window update logic for SorahkGui
impl eframe::App for SorahkGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if exit was requested at the very beginning
        if self.app_state.should_exit() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Apply anime theme
        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        // Anime style settings
        visuals.widgets.inactive.rounding = egui::Rounding::same(15.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(15.0);
        visuals.widgets.active.rounding = egui::Rounding::same(15.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(10.0);
        visuals.widgets.open.rounding = egui::Rounding::same(15.0);
        visuals.selection.stroke.width = 1.5;
        visuals.widgets.inactive.bg_stroke.width = 1.0;
        visuals.widgets.hovered.bg_stroke.width = 1.5;
        visuals.widgets.active.bg_stroke.width = 1.5;

        // Colors
        if !self.dark_mode {
            visuals.window_fill = egui::Color32::from_rgb(250, 250, 255);
            visuals.panel_fill = egui::Color32::from_rgb(248, 250, 255);
            visuals.faint_bg_color = egui::Color32::from_rgb(245, 248, 255);
        } else {
            visuals.window_fill = egui::Color32::from_rgb(30, 32, 40);
            visuals.panel_fill = egui::Color32::from_rgb(35, 37, 45);
            visuals.faint_bg_color = egui::Color32::from_rgb(40, 42, 50);
        }

        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 18.0,
            spread: 0.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 25),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 12.0,
            spread: 0.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 20),
        };

        ctx.set_visuals(visuals);
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
        self.handle_close_dialog(ctx);

        // Show dialogs
        if self.show_settings_dialog {
            self.render_settings_dialog(ctx);
        }

        if self.show_about_dialog {
            render_about_dialog(ctx, self.dark_mode, &mut self.show_about_dialog);
        }

        // Handle keyboard input
        self.handle_keyboard_input(ctx);

        // Render main content
        self.render_main_content(ctx);

        // Check exit
        if self.app_state.should_exit() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_state.exit();
    }
}

impl SorahkGui {
    // Handle close dialog logic
    fn handle_close_dialog(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.app_state.should_exit() {
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

    // Render close confirmation dialog
    fn render_close_dialog(&mut self, ctx: &egui::Context) {
        let should_highlight = self
            .dialog_highlight_until
            .map(|until| std::time::Instant::now() < until)
            .unwrap_or(false);

        if should_highlight {
            ctx.request_repaint();
        }

        let mut window = egui::Window::new("")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([380.0, 280.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

        if should_highlight {
            window = window.frame(
                egui::Frame::window(&ctx.style())
                    .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 200, 0))),
            );
        }

        window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("❓ Close Window")
                        .size(20.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        }),
                );

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("What would you like to do?")
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(25.0);

                let button_width = 320.0;
                let button_height = 32.0;
                let tray_enabled = self.config.show_tray_icon;

                if tray_enabled {
                    let minimize_btn = egui::Button::new(
                        egui::RichText::new("🗕  Minimize to Tray")
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(135, 206, 235))
                    .rounding(15.0);

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
                    egui::RichText::new("🚪  Exit Program")
                        .size(14.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                )
                .fill(egui::Color32::from_rgb(255, 182, 193))
                .rounding(15.0);

                if ui
                    .add_sized([button_width, button_height], exit_btn)
                    .clicked()
                {
                    self.show_close_dialog = false;
                    self.app_state.exit();
                }

                ui.add_space(12.0);

                let cancel_btn = egui::Button::new(egui::RichText::new("Cancel").size(13.0).color(
                    if self.dark_mode {
                        egui::Color32::from_rgb(200, 200, 200)
                    } else {
                        egui::Color32::from_rgb(80, 80, 80)
                    },
                ))
                .fill(if self.dark_mode {
                    egui::Color32::from_rgb(60, 60, 60)
                } else {
                    egui::Color32::from_rgb(230, 230, 230)
                })
                .rounding(10.0);

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

    // Handle keyboard input events
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

    // Render main content panel
    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_title_bar(ui);
            ui.add_space(15.0);
            self.render_status_card(ui);
            ui.add_space(18.0);
            self.render_hotkey_card(ui);
            ui.add_space(15.0);
            self.render_config_card(ui);
            ui.add_space(18.0);
            self.render_mappings_card(ui);
        });
    }

    // Render title bar with theme toggle and menu buttons
    fn render_title_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(15.0);

            ui.label(
                egui::RichText::new("🌸 Sorahk ~ Auto Key Press Tool ~")
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
                let theme_text = if self.dark_mode { "Light" } else { "Dark" };

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
                .rounding(12.0);

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
                    egui::RichText::new("⚙  Settings")
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(135, 206, 235))
                .rounding(12.0);

                if ui.add(settings_btn).clicked() {
                    self.show_settings_dialog = true;
                    self.temp_config = Some(self.config.clone());
                }

                ui.add_space(8.0);

                let about_btn = egui::Button::new(
                    egui::RichText::new("❤  About")
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(216, 191, 216))
                .rounding(12.0);

                if ui.add(about_btn).clicked() {
                    self.show_about_dialog = true;
                }
            });
        });
    }

    // Render status card with control buttons
    fn render_status_card(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✨ Status:").size(16.0).strong().color(
                    if self.dark_mode {
                        egui::Color32::from_rgb(255, 182, 193)
                    } else {
                        egui::Color32::from_rgb(220, 20, 60)
                    },
                ));

                ui.add_space(10.0);

                let is_paused = self.app_state.is_paused();
                let (icon, text, color) = if is_paused {
                    ("⏸", "Paused", egui::Color32::from_rgb(255, 140, 0))
                } else {
                    ("▶", "Running", egui::Color32::from_rgb(34, 139, 34))
                };

                ui.label(egui::RichText::new(icon).size(18.0).color(color));
                ui.label(egui::RichText::new(text).size(15.0).color(color).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let worker_count = self.app_state.get_actual_worker_count();
                    if worker_count > 0 {
                        ui.label(
                            egui::RichText::new(format!("⚡ {} Worker(s)", worker_count))
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

                let is_paused = self.app_state.is_paused();
                let (text, color) = if is_paused {
                    ("▶  Start", egui::Color32::from_rgb(144, 238, 144))
                } else {
                    ("⏸  Pause", egui::Color32::from_rgb(255, 218, 185))
                };

                let toggle_btn = egui::Button::new(
                    egui::RichText::new(text)
                        .size(14.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                )
                .fill(color)
                .rounding(15.0);

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
                    egui::RichText::new("❌  Exit")
                        .size(14.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                )
                .fill(egui::Color32::from_rgb(255, 182, 193))
                .rounding(15.0);

                if ui.add_sized([width, height], exit_btn).clicked() {
                    self.app_state.exit();
                    std::process::exit(0);
                }
            });
        });
    }

    // Render hotkey settings card
    fn render_hotkey_card(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new("🎯 Hotkey Settings")
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
                ui.label(
                    egui::RichText::new("Toggle Key:")
                        .size(14.0)
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(200, 200, 200)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        }),
                );
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

    // Render global configuration card
    fn render_config_card(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new("⚙ Global Configuration")
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
                        "Input Timeout:",
                        &format!("{} ms", self.config.input_timeout),
                    );
                    self.render_config_row(
                        ui,
                        "Default Interval:",
                        &format!("{} ms", self.config.interval),
                    );
                    self.render_config_row(
                        ui,
                        "Default Duration:",
                        &format!("{} ms", self.config.event_duration),
                    );
                    self.render_bool_row(ui, "Show Tray Icon:", self.config.show_tray_icon);
                    self.render_bool_row(ui, "Show Notifications:", self.config.show_notifications);
                    self.render_bool_row(ui, "Always On Top:", self.config.always_on_top);
                });
        });
    }

    // Helper: render configuration row
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

    // Helper: render boolean configuration row
    fn render_bool_row(&self, ui: &mut egui::Ui, label: &str, value: bool) {
        ui.label(
            egui::RichText::new(label)
                .size(14.0)
                .color(if self.dark_mode {
                    egui::Color32::from_rgb(200, 200, 200)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                }),
        );
        let text = if value { "Yes" } else { "No" };
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

    // Render key mappings card
    fn render_mappings_card(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new("🔄 Key Mappings")
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
                .max_height(200.0)
                .show(ui, |ui| {
                    let available = ui.available_width();
                    egui::Grid::new("mappings_grid")
                        .num_columns(4)
                        .spacing([20.0, 6.0])
                        .min_col_width(available * 0.2)
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
                                ui.end_row();
                            }
                        });
                });
        });
    }

    // Helper: render mapping table header
    fn render_mapping_header(&self, ui: &mut egui::Ui) {
        let headers = ["Trigger", "Target", "Interval(ms)", "Duration(ms)"];
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
