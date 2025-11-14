// Error dialog

use crate::gui::utils::create_icon;
use eframe::egui;

/// Error dialog structure for displaying configuration errors
struct ErrorDialog {
    error_msg: String,
}

impl eframe::App for ErrorDialog {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply anime theme
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(15.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(15.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(15.0);
        visuals.widgets.active.rounding = egui::Rounding::same(15.0);
        visuals.window_fill = egui::Color32::from_rgb(45, 50, 65);
        visuals.panel_fill = egui::Color32::from_rgb(45, 50, 65);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::Vec2::new(0.0, 8.0),
            blur: 20.0,
            spread: 5.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
        };
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
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

            ui.add_space(15.0);

            // Error message
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(60, 40, 50))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(15.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&self.error_msg)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(255, 200, 220)),
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
                .rounding(15.0);

                if ui.add_sized([120.0, 36.0], close_btn).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.add_space(10.0);
        });
    }
}

/// Display error dialog with anime-style theming
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
