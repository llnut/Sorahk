//! Mouse scroll direction selection dialog component.

use crate::i18n::CachedTranslations;
use eframe::egui;

/// Mouse scroll direction selection dialog
pub struct MouseScrollDialog {
    selected_direction: Option<String>,
}

impl MouseScrollDialog {
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
                    egui::Color32::from_rgb(173, 216, 230),
                    egui::Color32::from_rgb(60, 55, 75),
                    egui::Color32::from_rgb(80, 70, 100),
                    egui::Color32::from_rgb(200, 230, 255),
                    egui::Color32::from_rgb(220, 240, 255),
                )
            } else {
                (
                    egui::Color32::from_rgb(248, 252, 255),
                    egui::Color32::from_rgb(100, 149, 237),
                    egui::Color32::from_rgb(240, 248, 255),
                    egui::Color32::from_rgb(225, 240, 255),
                    egui::Color32::from_rgb(80, 120, 160),
                    egui::Color32::from_rgb(90, 130, 170),
                )
            };

        let mut should_close = false;

        egui::Window::new("mouse_scroll_dialog")
            .id(egui::Id::new("mouse_scroll_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([320.0, 380.0])
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
                        egui::RichText::new(format!("🎡 {} 🎡", t.mouse_scroll_direction_label()))
                            .size(20.0)
                            .strong()
                            .color(title_color),
                    );

                    ui.add_space(30.0);

                    // Scroll Up button
                    let up_btn = render_scroll_button(
                        ui,
                        "⬆",
                        t.mouse_scroll_up(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        text_color,
                        text_hover_color,
                    );

                    if up_btn.clicked() {
                        self.selected_direction = Some("SCROLL_UP".to_string());
                        should_close = true;
                    }

                    ui.add_space(15.0);

                    // Middle Mouse Button
                    let middle_btn = render_scroll_button(
                        ui,
                        "🖱",
                        t.mouse_middle_button(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        text_color,
                        text_hover_color,
                    );

                    if middle_btn.clicked() {
                        self.selected_direction = Some("MBUTTON".to_string());
                        should_close = true;
                    }

                    ui.add_space(15.0);

                    // Scroll Down button
                    let down_btn = render_scroll_button(
                        ui,
                        "⬇",
                        t.mouse_scroll_down(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        text_color,
                        text_hover_color,
                    );

                    if down_btn.clicked() {
                        self.selected_direction = Some("SCROLL_DOWN".to_string());
                        should_close = true;
                    }

                    ui.add_space(25.0);

                    // Cancel button
                    if ui
                        .add_sized(
                            [180.0, 32.0],
                            egui::Button::new(
                                egui::RichText::new(t.cancel_close_button())
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(147, 197, 253))
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

/// Render a scroll direction button with hover effects
#[allow(clippy::too_many_arguments)]
fn render_scroll_button(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    dark_mode: bool,
    button_bg: egui::Color32,
    button_hover_bg: egui::Color32,
    text_color: egui::Color32,
    text_hover_color: egui::Color32,
) -> egui::Response {
    let (desired_size, corner_radius) = ([220.0, 50.0], 12.0);
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
                egui::Color32::from_rgb(200, 230, 255)
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
            format!("{} {}", icon, label),
            egui::FontId::proportional(16.0),
            fg_color,
        );

        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    response
}
