//! About dialog implementation.

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
    // Pre-calculate all colors based on theme
    let (
        title_color,
        subtitle_color,
        card_bg,
        version_color,
        text_color,
        text_secondary,
        label_color,
        inspired_color,
    ) = if dark_mode {
        (
            egui::Color32::from_rgb(255, 182, 193), // Soft pink
            egui::Color32::from_rgb(200, 200, 255), // Light lavender
            egui::Color32::from_rgb(40, 40, 50),    // Dark card bg
            egui::Color32::from_rgb(144, 238, 144), // Light green
            egui::Color32::from_rgb(220, 220, 220), // Light gray text
            egui::Color32::from_rgb(200, 200, 200), // Secondary text
            egui::Color32::from_rgb(135, 206, 235), // Sky blue
            egui::Color32::from_rgb(180, 180, 180), // Inspired text
        )
    } else {
        (
            egui::Color32::from_rgb(219, 112, 147), // Pale violet red
            egui::Color32::from_rgb(147, 112, 219), // Medium purple
            egui::Color32::from_rgb(250, 240, 255), // Light card bg
            egui::Color32::from_rgb(60, 179, 113),  // Medium sea green
            egui::Color32::from_rgb(60, 60, 60),    // Dark text
            egui::Color32::from_rgb(80, 80, 80),    // Secondary text
            egui::Color32::from_rgb(70, 130, 180),  // Steel blue
            egui::Color32::from_rgb(120, 120, 120), // Inspired text
        )
    };

    let dialog_bg = if dark_mode {
        egui::Color32::from_rgb(30, 32, 42)
    } else {
        egui::Color32::from_rgb(252, 248, 255)
    };

    egui::Window::new("about_sorahk")
        .id(egui::Id::new("about_dialog_window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size([500.0, 550.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
            // Use a simpler layout without excessive centering
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(25.0);

                // Main title - single label
                ui.label(
                    egui::RichText::new("🌸 Sorahk 🌸")
                        .size(32.0)
                        .strong()
                        .color(title_color),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("~ Auto Key Press Tool ~")
                        .size(16.0)
                        .italics()
                        .color(subtitle_color),
                );
                ui.add_space(30.0);

                // Version card - simplified Frame
                ui.scope(|ui| {
                    ui.set_max_width(460.0);
                    egui::Frame::NONE
                        .fill(card_bg)
                        .corner_radius(15.0)
                        .inner_margin(egui::Margin::symmetric(20, 15))
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(t.about_version())
                                        .size(18.0)
                                        .strong()
                                        .color(version_color),
                                );
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(t.about_description_line1())
                                        .size(13.0)
                                        .color(text_secondary),
                                );
                                ui.label(
                                    egui::RichText::new(t.about_description_line2())
                                        .size(13.0)
                                        .color(text_secondary),
                                );
                            });
                        });
                });
                ui.add_space(25.0);

                // Info section - flattened layout
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    ui.set_max_width(420.0);

                    // Use Grid for better performance
                    egui::Grid::new("about_info_grid")
                        .num_columns(2)
                        .spacing([12.0, 12.0])
                        .show(ui, |ui| {
                            // Author
                            ui.label(
                                egui::RichText::new(t.about_author())
                                    .size(14.0)
                                    .strong()
                                    .color(label_color),
                            );
                            ui.label(egui::RichText::new("llnut").size(14.0).color(text_color));
                            ui.end_row();

                            // GitHub
                            ui.label(
                                egui::RichText::new(t.about_github())
                                    .size(14.0)
                                    .strong()
                                    .color(label_color),
                            );
                            ui.hyperlink_to(
                                egui::RichText::new("https://github.com/llnut/Sorahk")
                                    .size(14.0)
                                    .color(label_color),
                                "https://github.com/llnut/Sorahk",
                            );
                            ui.end_row();

                            // License
                            ui.label(
                                egui::RichText::new(t.about_license())
                                    .size(14.0)
                                    .strong()
                                    .color(label_color),
                            );
                            ui.label(
                                egui::RichText::new(t.about_mit_license())
                                    .size(14.0)
                                    .color(text_color),
                            );
                            ui.end_row();

                            // Built with
                            ui.label(
                                egui::RichText::new(t.about_built_with())
                                    .size(14.0)
                                    .strong()
                                    .color(label_color),
                            );
                            ui.label(
                                egui::RichText::new(t.about_rust_egui())
                                    .size(14.0)
                                    .color(text_color),
                            );
                            ui.end_row();
                        });
                });
                ui.add_space(30.0);

                // Inspired note
                ui.label(
                    egui::RichText::new(t.about_inspired())
                        .size(12.0)
                        .italics()
                        .color(inspired_color),
                );
                ui.add_space(25.0);

                // Close button
                if ui
                    .add_sized(
                        [200.0, 32.0],
                        egui::Button::new(
                            egui::RichText::new(t.error_close_button())
                                .size(15.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(216, 191, 216))
                        .corner_radius(15.0),
                    )
                    .clicked()
                {
                    *show_about_dialog = false;
                }
                ui.add_space(20.0);
            });
        });
}
