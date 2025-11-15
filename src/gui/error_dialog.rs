//! Error dialog for displaying critical errors.

use crate::gui::utils::create_icon;
use eframe::egui;

/// Error dialog structure for displaying configuration errors.
struct ErrorDialog {
    /// Error message text
    error_msg: String,
}

impl eframe::App for ErrorDialog {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply anime theme with no borders
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        visuals.window_fill = egui::Color32::from_rgb(32, 34, 45);
        visuals.panel_fill = egui::Color32::from_rgb(32, 34, 45);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 10,
            spread: 3,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 60),
        };
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(32, 34, 45)))
            .show(ctx, |ui| {
                ui.add_space(20.0);

                // Error icon and title
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("❌ Configuration Error")
                            .size(24.0)
                            .color(egui::Color32::from_rgb(255, 100, 130))
                            .strong(),
                    );
                });

                ui.add_space(20.0);

                // Error message card - 无边框设计
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(45, 40, 52))
                    .corner_radius(egui::CornerRadius::same(16))
                    .inner_margin(egui::Margin::same(18))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 2],
                        blur: 8,
                        spread: 0,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 35),
                    })
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.error_msg)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(255, 210, 230)),
                        );
                    });

                ui.add_space(20.0);

                // Close button
                ui.vertical_centered(|ui| {
                    let close_btn = egui::Button::new(
                        egui::RichText::new("Close")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(255, 182, 193))
                    .corner_radius(15.0);

                    if ui.add_sized([120.0, 36.0], close_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.add_space(10.0);
            });
    }
}

/// Displays an error dialog in a separate window.
///
/// # Errors
///
/// Returns an error if the GUI framework fails to initialize.
pub fn show_error(error_msg: &str) -> anyhow::Result<()> {
    let icon = create_icon();
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([450.0, 280.0])
        .with_resizable(false)
        .with_title("Sorahk - Error")
        .with_icon(icon)
        .with_always_on_top();

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Sorahk Error",
        options,
        Box::new(|_cc| {
            Ok(Box::new(ErrorDialog {
                error_msg: error_msg.to_string(),
            }))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to show error dialog: {}", e))
}
