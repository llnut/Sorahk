//! Mouse scroll direction selection dialog component.

use crate::gui::theme;
use crate::gui::widgets::{self, text_size};
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
        let c = theme::colors(dark_mode);

        // Hover and pressed bg for scroll buttons stay as inline tints
        // because the palette has no mid-tone entry suited to this
        // tinted-blue picker. Default bg uses c.bg_card_hover.
        let button_bg = c.bg_card_hover;
        let button_hover_bg = if dark_mode {
            egui::Color32::from_rgb(80, 70, 100)
        } else {
            egui::Color32::from_rgb(225, 240, 255)
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
                    .fill(c.bg_card)
                    .corner_radius(egui::CornerRadius::same(widgets::radius::DIALOG))
                    .stroke(egui::Stroke::NONE)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 5],
                        blur: 22,
                        spread: 2,
                        color: theme::overlay::SHADOW_HEAVY,
                    }),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(20.0);

                    // Title.
                    ui.label(
                        egui::RichText::new(t.mouse_scroll_direction_label())
                            .size(text_size::TITLE)
                            .strong()
                            .color(c.title_primary),
                    );

                    ui.add_space(25.0);

                    // Scroll Up button
                    if render_scroll_button(
                        ui,
                        t.mouse_scroll_up(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        c.fg_primary,
                    )
                    .clicked()
                    {
                        self.selected_direction = Some("SCROLL_UP".to_string());
                        should_close = true;
                    }

                    ui.add_space(8.0);

                    // Middle Mouse Button
                    if render_scroll_button(
                        ui,
                        t.mouse_middle_button(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        c.fg_primary,
                    )
                    .clicked()
                    {
                        self.selected_direction = Some("MBUTTON".to_string());
                        should_close = true;
                    }

                    ui.add_space(8.0);

                    // Scroll Down button
                    if render_scroll_button(
                        ui,
                        t.mouse_scroll_down(),
                        dark_mode,
                        button_bg,
                        button_hover_bg,
                        c.fg_primary,
                    )
                    .clicked()
                    {
                        self.selected_direction = Some("SCROLL_DOWN".to_string());
                        should_close = true;
                    }

                    ui.add_space(20.0);

                    // Cancel button.
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(t.cancel_close_button())
                            .size(text_size::NORMAL)
                            .color(c.fg_inverse),
                    )
                    .fill(c.accent_secondary)
                    .corner_radius(15.0);
                    if ui.add_sized([180.0, 32.0], cancel_btn).clicked() {
                        should_close = true;
                    }

                    ui.add_space(15.0);
                });
            });

        should_close
    }
}

/// Custom-painted scroll-direction button. Hover/pressed states use inline
/// RGB tints because the theme palette has no mid-tone equivalent suited
/// to this small picker dialog.
fn render_scroll_button(
    ui: &mut egui::Ui,
    label: &str,
    dark_mode: bool,
    button_bg: egui::Color32,
    button_hover_bg: egui::Color32,
    text_color: egui::Color32,
) -> egui::Response {
    let corner_radius = widgets::radius::BUTTON as f32;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(220.0, 50.0), egui::Sense::click());

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

        ui.painter().rect_filled(rect, corner_radius, bg_color);

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(text_size::SUBTITLE),
            text_color,
        );

        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    response
}
