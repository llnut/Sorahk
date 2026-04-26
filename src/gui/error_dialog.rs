//! Error dialog for displaying critical errors.

use crate::gui::theme::{self, ThemeCache};
use crate::gui::widgets::{self, ButtonKind, text_size};
use crate::gui::fonts;
use crate::gui::utils::create_icon;
use crate::i18n::{CachedTranslations, Language};
use eframe::egui;

/// Error dialog structure for displaying configuration errors.
struct ErrorDialog {
    /// Error message text
    error_msg: String,
    /// Cached translations
    translations: CachedTranslations,
    /// Pre-computed dark theme visuals.
    theme_cache: ThemeCache,
}

impl eframe::App for ErrorDialog {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = &self.translations;
        let c = theme::colors(true);

        ctx.set_visuals(self.theme_cache.dark.clone());

        // Bottom-anchored close button so its position is independent of
        // the message card's content height.
        egui::TopBottomPanel::bottom("error_close_panel")
            .frame(egui::Frame::NONE.fill(c.bg_window))
            .show_separator_line(false)
            .min_height(64.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    let btn =
                        widgets::themed_button(t.error_close_button(), ButtonKind::Pink, true);
                    if ui.add_sized([120.0, 36.0], btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

        // Central content with symmetric horizontal padding applied via
        // the panel frame inner_margin. More reliable than outer_margin
        // when the parent layout is left-aligned.
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(c.bg_window)
                    .inner_margin(egui::Margin::symmetric(20, 0)),
            )
            .show(ctx, |ui| {
                ui.add_space(15.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(t.error_title())
                            .size(text_size::TITLE)
                            .color(c.accent_danger)
                            .strong(),
                    );
                });

                ui.add_space(15.0);

                // Subtract the card_frame inner margin plus a small buffer
                // so the bottom corner_radius stays visible.
                let card_inner_margin_total = widgets::spacing::LARGE * 2.0;
                let remaining = (ui.available_height() - 5.0 - card_inner_margin_total).max(0.0);
                widgets::card_frame(true)
                    .fill(c.bg_card_hover)
                    .show(ui, |ui| {
                        ui.set_min_height(remaining);
                        ui.label(
                            egui::RichText::new(&self.error_msg)
                                .size(text_size::NORMAL)
                                .color(c.fg_primary),
                        );
                    });
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

    // Error dialog always uses English for better compatibility
    let language = Language::English;

    eframe::run_native(
        "Sorahk Error",
        options,
        Box::new(move |cc| {
            // Load fonts for proper text rendering
            fonts::load_fonts(&cc.egui_ctx, language);

            Ok(Box::new(ErrorDialog {
                error_msg: error_msg.to_string(),
                translations: CachedTranslations::new(language),
                theme_cache: ThemeCache::new(),
            }))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to show error dialog: {}", e))
}
