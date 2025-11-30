//! Mouse direction selection dialog.

use crate::i18n::CachedTranslations;
use eframe::egui;

/// Mouse direction selection dialog
pub struct MouseDirectionDialog {
    selected_direction: Option<String>,
}

impl MouseDirectionDialog {
    pub fn new() -> Self {
        Self {
            selected_direction: None,
        }
    }

    /// Get the selected direction and consume the dialog
    pub fn get_selected_direction(&self) -> Option<String> {
        self.selected_direction.clone()
    }

    /// Render the dialog, returns true if should close
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        dark_mode: bool,
        translations: &CachedTranslations,
    ) -> bool {
        let t = translations;

        let (bg_color, title_color, button_bg, button_hover_bg, text_color, text_hover_color) =
            if dark_mode {
                (
                    egui::Color32::from_rgb(30, 32, 42),
                    egui::Color32::from_rgb(255, 182, 193),
                    egui::Color32::from_rgb(60, 55, 75),
                    egui::Color32::from_rgb(80, 70, 100),
                    egui::Color32::from_rgb(255, 200, 220),
                    egui::Color32::from_rgb(255, 220, 235),
                )
            } else {
                (
                    egui::Color32::from_rgb(252, 248, 255),
                    egui::Color32::from_rgb(219, 112, 147),
                    egui::Color32::from_rgb(255, 240, 250),
                    egui::Color32::from_rgb(255, 225, 245),
                    egui::Color32::from_rgb(140, 80, 120),
                    egui::Color32::from_rgb(160, 90, 130),
                )
            };

        let mut should_close = false;

        egui::Window::new("mouse_direction_dialog")
            .id(egui::Id::new("mouse_direction_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([380.0, 380.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(bg_color)
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
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(20.0);

                    // Title
                    ui.label(
                        egui::RichText::new(format!("✨ {} ✨", t.mouse_move_direction_label()))
                            .size(20.0)
                            .strong()
                            .color(title_color),
                    );

                    ui.add_space(25.0);

                    // Direction grid - centered
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - (100.0 * 3.0 + 8.0 * 2.0)) / 2.0);
                        egui::Grid::new("mouse_direction_grid_dialog")
                            .spacing([8.0, 8.0])
                            .show(ui, |ui| {
                                // Row 1: Up-Left, Up, Up-Right
                                if render_direction_button(
                                    ui,
                                    "↖",
                                    t.mouse_move_up_left(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_UP_LEFT".to_string());
                                    should_close = true;
                                }
                                if render_direction_button(
                                    ui,
                                    "↑",
                                    t.mouse_move_up(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_UP".to_string());
                                    should_close = true;
                                }
                                if render_direction_button(
                                    ui,
                                    "↗",
                                    t.mouse_move_up_right(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_UP_RIGHT".to_string());
                                    should_close = true;
                                }
                                ui.end_row();

                                // Row 2: Left, (Center), Right
                                if render_direction_button(
                                    ui,
                                    "←",
                                    t.mouse_move_left(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_LEFT".to_string());
                                    should_close = true;
                                }

                                // Center mouse icon
                                let (rect, _response) = ui.allocate_exact_size(
                                    egui::vec2(100.0, 60.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "🖱",
                                    egui::FontId::proportional(32.0),
                                    text_color,
                                );

                                if render_direction_button(
                                    ui,
                                    "→",
                                    t.mouse_move_right(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_RIGHT".to_string());
                                    should_close = true;
                                }
                                ui.end_row();

                                // Row 3: Down-Left, Down, Down-Right
                                if render_direction_button(
                                    ui,
                                    "↙",
                                    t.mouse_move_down_left(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_DOWN_LEFT".to_string());
                                    should_close = true;
                                }
                                if render_direction_button(
                                    ui,
                                    "↓",
                                    t.mouse_move_down(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_DOWN".to_string());
                                    should_close = true;
                                }
                                if render_direction_button(
                                    ui,
                                    "↘",
                                    t.mouse_move_down_right(),
                                    dark_mode,
                                    button_bg,
                                    button_hover_bg,
                                    text_color,
                                    text_hover_color,
                                )
                                .clicked()
                                {
                                    self.selected_direction = Some("MOUSE_DOWN_RIGHT".to_string());
                                    should_close = true;
                                }
                                ui.end_row();
                            });
                    });

                    ui.add_space(20.0);

                    // Cancel button
                    if ui
                        .add_sized(
                            [180.0, 32.0],
                            egui::Button::new(
                                egui::RichText::new(t.error_close_button())
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(216, 191, 216))
                            .corner_radius(15.0),
                        )
                        .clicked()
                    {
                        should_close = true;
                    }

                    ui.add_space(15.0);
                });
            });

        should_close
    }
}

/// Renders a direction button with consistent styling
fn render_direction_button(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    dark_mode: bool,
    button_bg: egui::Color32,
    button_hover_bg: egui::Color32,
    text_color: egui::Color32,
    text_hover_color: egui::Color32,
) -> egui::Response {
    let (desired_size, corner_radius) = ([100.0, 60.0], 12.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(desired_size[0], desired_size[1]),
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let is_pressed = response.is_pointer_button_down_on();

        let bg_color = if is_pressed {
            if dark_mode {
                egui::Color32::from_rgb(70, 60, 90)
            } else {
                egui::Color32::from_rgb(255, 215, 240)
            }
        } else if is_hovered {
            button_hover_bg
        } else {
            button_bg
        };

        let fg_color = if is_hovered || is_pressed {
            text_hover_color
        } else {
            text_color
        };

        ui.painter().rect_filled(rect, corner_radius, bg_color);

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}\n{}", icon, label),
            egui::FontId::proportional(13.0),
            fg_color,
        );
    }

    response
}
