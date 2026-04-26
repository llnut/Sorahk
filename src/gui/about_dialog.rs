//! About dialog implementation.

use crate::gui::theme;
use crate::gui::widgets::{self, text_size};
use crate::i18n::CachedTranslations;
use eframe::egui;

/// Renders the about dialog showing application information.
pub fn render_about_dialog(
    ctx: &egui::Context,
    dark_mode: bool,
    show_about_dialog: &mut bool,
    translations: &CachedTranslations,
) {
    let t = translations;
    let c = theme::colors(dark_mode);

    egui::Window::new("about_sorahk")
        .id(egui::Id::new("about_dialog_window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size([500.0, 550.0])
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
                ui.add_space(25.0);

                // Main title.
                ui.label(
                    egui::RichText::new("🌸 Sorahk 🌸")
                        .size(text_size::HERO)
                        .strong()
                        .color(c.accent_pink),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("~ Auto Key Press Tool ~")
                        .size(text_size::SUBTITLE)
                        .italics()
                        .color(c.accent_secondary),
                );
                ui.add_space(30.0);

                // Version card.
                ui.scope(|ui| {
                    ui.set_max_width(460.0);
                    widgets::card_frame(dark_mode)
                        .fill(c.bg_card_hover)
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(t.about_version())
                                            .size(text_size::SECTION)
                                            .strong()
                                            .color(c.accent_success),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(t.about_description_line1())
                                            .size(text_size::BODY)
                                            .color(c.fg_muted),
                                    );
                                    ui.label(
                                        egui::RichText::new(t.about_description_line2())
                                            .size(text_size::BODY)
                                            .color(c.fg_muted),
                                    );
                                },
                            );
                        });
                });
                ui.add_space(25.0);

                // Info section.
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    ui.set_max_width(420.0);

                    egui::Grid::new("about_info_grid")
                        .num_columns(2)
                        .spacing([12.0, 12.0])
                        .show(ui, |ui| {
                            // Author
                            ui.label(
                                egui::RichText::new(t.about_author())
                                    .size(text_size::NORMAL)
                                    .strong()
                                    .color(c.accent_primary),
                            );
                            ui.label(
                                egui::RichText::new("llnut")
                                    .size(text_size::NORMAL)
                                    .color(c.fg_primary),
                            );
                            ui.end_row();

                            // GitHub
                            ui.label(
                                egui::RichText::new(t.about_github())
                                    .size(text_size::NORMAL)
                                    .strong()
                                    .color(c.accent_primary),
                            );
                            ui.hyperlink_to(
                                egui::RichText::new("https://github.com/llnut/Sorahk")
                                    .size(text_size::NORMAL)
                                    .color(c.accent_primary),
                                "https://github.com/llnut/Sorahk",
                            );
                            ui.end_row();

                            // License
                            ui.label(
                                egui::RichText::new(t.about_license())
                                    .size(text_size::NORMAL)
                                    .strong()
                                    .color(c.accent_primary),
                            );
                            ui.label(
                                egui::RichText::new(t.about_mit_license())
                                    .size(text_size::NORMAL)
                                    .color(c.fg_primary),
                            );
                            ui.end_row();

                            // Built with
                            ui.label(
                                egui::RichText::new(t.about_built_with())
                                    .size(text_size::NORMAL)
                                    .strong()
                                    .color(c.accent_primary),
                            );
                            ui.label(
                                egui::RichText::new(t.about_rust_egui())
                                    .size(text_size::NORMAL)
                                    .color(c.fg_primary),
                            );
                            ui.end_row();
                        });
                });
                ui.add_space(30.0);

                // Inspired note.
                ui.label(
                    egui::RichText::new(t.about_inspired())
                        .size(text_size::COMPACT)
                        .italics()
                        .color(c.fg_muted),
                );
                ui.add_space(25.0);

                // Close button.
                let btn = egui::Button::new(
                    egui::RichText::new(t.error_close_button())
                        .size(15.0)
                        .color(c.fg_inverse)
                        .strong(),
                )
                .fill(c.accent_secondary)
                .corner_radius(15.0);
                if ui.add_sized([200.0, 32.0], btn).clicked() {
                    *show_about_dialog = false;
                }
                ui.add_space(20.0);
            });
        });
}
