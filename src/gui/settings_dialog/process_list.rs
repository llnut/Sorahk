//! Settings dialog "Process Whitelist" section. Implemented as a free
//! function so the caller can split-borrow `SorahkGui` fields disjointly
//! with the parent scroll-area closure.

use crate::config::AppConfig;
use crate::i18n::CachedTranslations;
use eframe::egui;

/// Renders the Process Whitelist card with hint text, entry list, and
/// the input row for adding a new entry.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_process_list_section(
    ui: &mut egui::Ui,
    temp_config: &mut AppConfig,
    new_process_name: &mut String,
    duplicate_process_error: &mut Option<String>,
    dark_mode: bool,
    translations: CachedTranslations,
) {
    let t = translations;

    // Process Whitelist Section
    let card_bg = if dark_mode {
        egui::Color32::from_rgb(40, 40, 50)
    } else {
        egui::Color32::from_rgb(250, 240, 255)
    };

    egui::Frame::NONE
        .fill(card_bg)
        .corner_radius(egui::CornerRadius::same(15))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(t.process_whitelist_hint())
                    .size(16.0)
                    .strong()
                    .color(if dark_mode {
                        egui::Color32::from_rgb(200, 180, 255)
                    } else {
                        egui::Color32::from_rgb(150, 100, 200)
                    }),
            );
            ui.add_space(6.0);

            // Process list
            egui::ScrollArea::vertical().max_height(80.0).show(
                ui,
                |ui| {
                    let mut to_remove: Option<usize> = None;
                    for (idx, process) in temp_config
                        .process_whitelist
                        .iter()
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(process)
                                    .size(13.0)
                                    .color(if dark_mode {
                                        egui::Color32::from_rgb(
                                            200, 200, 255,
                                        )
                                    } else {
                                        egui::Color32::from_rgb(
                                            60, 60, 120,
                                        )
                                    }),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(
                                    egui::Align::Center,
                                ),
                                |ui| {
                                    let t = &translations;
                                    let del_btn = egui::Button::new(
                                    egui::RichText::new(t.delete_icon())
                                        .color(egui::Color32::WHITE)
                                        .size(11.0),
                                )
                                .fill(egui::Color32::from_rgb(
                                    255, 182, 193,
                                ))
                                .corner_radius(8.0);

                                    if ui
                                        .add_sized(
                                            [24.0, 20.0],
                                            del_btn,
                                        )
                                        .clicked()
                                    {
                                        to_remove = Some(idx);
                                    }
                                },
                            );
                        });
                    }

                    if let Some(idx) = to_remove {
                        temp_config.process_whitelist.remove(idx);
                    }
                },
            );

            ui.add_space(6.0);

            // Add new process
            ui.horizontal(|ui| {
                let process_edit = egui::TextEdit::singleline(
                    &mut (*new_process_name),
                )
                .background_color(if dark_mode {
                    egui::Color32::from_rgb(50, 50, 50)
                } else {
                    egui::Color32::from_rgb(220, 220, 220)
                })
                .hint_text(t.process_example())
                .desired_width(200.0);
                ui.add(process_edit);

                let add_btn = egui::Button::new(
                    egui::RichText::new(t.add_button_text())
                        .color(egui::Color32::WHITE)
                        .size(12.0)
                        .strong(),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(100, 220, 180)
                } else {
                    egui::Color32::from_rgb(120, 240, 200)
                })
                .corner_radius(10.0);

                if ui.add_sized([70.0, 24.0], add_btn).clicked() {
                    let process_name = (*new_process_name).trim();
                    if !process_name.is_empty() {
                        // Check for duplicate process
                        if temp_config
                            .process_whitelist
                            .contains(&process_name.to_string())
                        {
                            *duplicate_process_error = Some(
                                t.duplicate_process_error()
                                    .to_string(),
                            );
                        } else {
                            // Clear any previous error
                            *duplicate_process_error = None;
                            temp_config
                                .process_whitelist
                                .push(process_name.to_string());
                            (*new_process_name).clear();
                        }
                    }
                }

                ui.add_space(8.0);

                // Browse button for selecting process
                let browse_btn = egui::Button::new(
                    egui::RichText::new(t.browse_button())
                        .color(egui::Color32::WHITE)
                        .size(12.0)
                        .strong(),
                )
                .fill(if dark_mode {
                    egui::Color32::from_rgb(180, 160, 230)
                } else {
                    egui::Color32::from_rgb(210, 190, 240)
                })
                .corner_radius(10.0);

                if ui.add_sized([85.0, 24.0], browse_btn).clicked()
                {
                    // Open file dialog to select executable
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Executable", &["exe"])
                        .set_title("Select Process")
                        .pick_file()
                        && let Some(filename) = path.file_name()
                    {
                        let process_name =
                            filename.to_string_lossy().to_string();
                        // Check for duplicate process
                        if temp_config
                            .process_whitelist
                            .contains(&process_name)
                        {
                            *duplicate_process_error = Some(
                                t.duplicate_process_error()
                                    .to_string(),
                            );
                        } else {
                            // Clear any previous error
                            *duplicate_process_error = None;
                            temp_config
                                .process_whitelist
                                .push(process_name);
                        }
                    }
                }
            });

            // Display duplicate process error if exists
            if let Some(ref error_msg) = *duplicate_process_error {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(error_msg)
                        .color(egui::Color32::from_rgb(
                            255, 100, 100,
                        ))
                        .size(13.0),
                );
            }
        });
}
