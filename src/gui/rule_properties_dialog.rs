//! Rule properties dialog.
//!
//! Exposes per-key "hold after playback" toggles plus an append-keys list
//! for any mapping regardless of target mode. The results drive the
//! `hold_indices` / `append_keys` fields on `KeyMapping`, which
//! `state::create_input_mappings` turns into an `OutputAction::MappingHold`.

use std::collections::HashSet;

use eframe::egui;
use smallvec::SmallVec;

use crate::gui::SorahkGui;
use crate::gui::theme;
use crate::gui::widgets::{self, text_size};
use crate::i18n::CachedTranslations;

/// Snapshot returned when the user confirms the dialog.
#[derive(Debug, Clone, Default)]
pub struct RuleProperties {
    pub hold_indices: SmallVec<[u8; 4]>,
    pub append_keys: SmallVec<[String; 4]>,
}

/// Standalone dialog for editing rule properties (hold + append).
pub struct RulePropertiesDialog {
    /// Target keys of the mapping, shown as the body list.
    body_keys: Vec<String>,
    /// Parallel to `body_keys`: whether each index is marked "hold after".
    hold_flags: Vec<bool>,
    /// Keys pressed and held after the mapping body completes.
    append_keys: Vec<String>,
    capturing_append: bool,
    capture_initial_pressed: HashSet<u32>,
    capture_pressed_keys: HashSet<u32>,
    result: Option<RuleProperties>,
}

impl RulePropertiesDialog {
    /// Builds the dialog from the mapping's current target keys, hold
    /// indices, and append list. `existing_hold` values out of range
    /// are silently ignored so stale config never crashes the UI.
    pub fn new(existing_keys: &[String], existing_hold: &[u8], existing_append: &[String]) -> Self {
        let mut hold_flags = vec![false; existing_keys.len()];
        for &idx in existing_hold {
            let ui = idx as usize;
            if ui < hold_flags.len() {
                hold_flags[ui] = true;
            }
        }

        Self {
            body_keys: existing_keys.to_vec(),
            hold_flags,
            append_keys: existing_append.to_vec(),
            capturing_append: false,
            capture_initial_pressed: HashSet::new(),
            capture_pressed_keys: HashSet::new(),
            result: None,
        }
    }

    /// Consumes the confirmed result. Returns `None` if the user cancelled
    /// or the dialog is still open.
    pub fn take_result(&mut self) -> Option<RuleProperties> {
        self.result.take()
    }

    /// Drives the capture state machine. Mirrors the accumulator +
    /// finalize-on-release logic from `settings_dialog::capture` so combos
    /// like `LCTRL+C` resolve as one entry.
    fn drive_capture(&mut self) {
        if !self.capturing_append {
            return;
        }
        let current = SorahkGui::poll_all_pressed_keys();

        for &vk in current.iter() {
            if !self.capture_initial_pressed.contains(&vk) {
                self.capture_pressed_keys.insert(vk);
            }
        }

        let any_released = self
            .capture_pressed_keys
            .iter()
            .any(|vk| !current.contains(vk));

        if any_released {
            if let Some(formatted) = SorahkGui::format_captured_keys(&self.capture_pressed_keys)
                && !self.append_keys.contains(&formatted)
            {
                self.append_keys.push(formatted);
            }
            self.capturing_append = false;
            self.capture_pressed_keys.clear();
            self.capture_initial_pressed.clear();
        }
    }

    /// Renders the dialog. Returns `true` when the caller should drop the
    /// dialog instance (user clicked Save or Cancel).
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        dark_mode: bool,
        translations: &CachedTranslations,
    ) -> bool {
        self.drive_capture();

        let t = translations;
        let c = theme::colors(dark_mode);

        let mut should_close = false;

        egui::Window::new("rule_properties_dialog")
            .id(egui::Id::new("rule_properties_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([460.0, 520.0])
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
                ui.vertical_centered(|ui| {
                    ui.add_space(18.0);
                    ui.label(
                        egui::RichText::new(t.rule_props_dialog_title())
                            .size(text_size::TITLE)
                            .strong()
                            .color(c.accent_pink),
                    );
                    ui.add_space(8.0);
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(t.rule_props_hint())
                            .size(text_size::COMPACT)
                            .color(c.fg_muted),
                    );
                });

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new(t.rule_props_hold_column())
                        .size(text_size::BODY)
                        .color(c.fg_muted),
                );
                ui.add_space(4.0);

                egui::Frame::NONE
                    .fill(c.bg_card_hover)
                    .corner_radius(egui::CornerRadius::same(widgets::radius::BUTTON))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                if self.body_keys.is_empty() {
                                    ui.label(
                                        egui::RichText::new("—")
                                            .size(text_size::BODY)
                                            .color(c.fg_muted),
                                    );
                                } else {
                                    for (idx, key) in self.body_keys.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            // Pin the index column to a fixed width so single
                                            // digit and double digit labels line up vertically.
                                            // `allocate_exact_size` reserves the column rect
                                            // unconditionally (unlike `allocate_ui_with_layout`,
                                            // which shrinks back to content on closure exit).
                                            // `painter.text` then draws "#N" starting at the
                                            // same origin in every row — alignment comes from
                                            // the shared left_center origin, not from the
                                            // label fitting inside the rect. 18px is deliberately
                                            // tight: "#15" extends a couple pixels past the
                                            // column and egui's default item_spacing (~8px)
                                            // absorbs the overflow before the next widget.
                                            const IDX_COL_WIDTH: f32 = 18.0;
                                            const IDX_COL_HEIGHT: f32 = 20.0;
                                            let (idx_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(IDX_COL_WIDTH, IDX_COL_HEIGHT),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().text(
                                                idx_rect.left_center(),
                                                egui::Align2::LEFT_CENTER,
                                                format!("#{idx}"),
                                                egui::FontId::proportional(text_size::COMPACT),
                                                c.fg_muted,
                                            );
                                            ui.label(
                                                egui::RichText::new(key)
                                                    .size(text_size::NORMAL)
                                                    .strong()
                                                    .color(c.fg_primary),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let label = t.rule_props_hold_column();
                                                    if idx < self.hold_flags.len() {
                                                        ui.checkbox(
                                                            &mut self.hold_flags[idx],
                                                            egui::RichText::new(label)
                                                                .size(text_size::COMPACT)
                                                                .color(c.fg_primary),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                    }
                                }
                            });
                    });

                ui.add_space(14.0);

                ui.label(
                    egui::RichText::new(t.rule_props_append_label())
                        .size(text_size::BODY)
                        .color(c.fg_muted),
                );
                ui.add_space(4.0);

                egui::Frame::NONE
                    .fill(c.bg_card_hover)
                    .corner_radius(egui::CornerRadius::same(widgets::radius::BUTTON))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let mut remove_idx: Option<usize> = None;
                            for (idx, key) in self.append_keys.iter().enumerate() {
                                widgets::pill_frame(c.accent_secondary).show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(key)
                                            .size(text_size::BODY)
                                            .color(c.fg_inverse)
                                            .strong(),
                                    );
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(t.delete_icon())
                                                    .size(text_size::COMPACT)
                                                    .color(c.fg_inverse),
                                            )
                                            .fill(egui::Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        remove_idx = Some(idx);
                                    }
                                });
                            }
                            if let Some(i) = remove_idx {
                                self.append_keys.remove(i);
                            }

                            if self.capturing_append {
                                ui.label(
                                    egui::RichText::new(t.rule_props_append_placeholder())
                                        .size(text_size::COMPACT)
                                        .italics()
                                        .color(c.fg_muted),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(t.cancel_close_button())
                                                .size(text_size::COMPACT)
                                                .color(c.fg_inverse),
                                        )
                                        .fill(c.accent_secondary)
                                        .corner_radius(10.0),
                                    )
                                    .clicked()
                                {
                                    self.capturing_append = false;
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed.clear();
                                }
                            } else if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(t.rule_props_add_append())
                                            .size(text_size::BODY)
                                            .color(c.fg_inverse)
                                            .strong(),
                                    )
                                    .fill(c.accent_secondary)
                                    .corner_radius(10.0),
                                )
                                .clicked()
                            {
                                self.capturing_append = true;
                                self.capture_pressed_keys.clear();
                                self.capture_initial_pressed = SorahkGui::poll_all_pressed_keys();
                            }
                        });
                    });

                ui.add_space(18.0);

                // Save on the left, Cancel on the right. Compact
                // fixed-width buttons, centered horizontally so the
                // footer stays lighter than the global Settings footer.
                const FOOTER_BTN_W: f32 = 120.0;
                const FOOTER_BTN_H: f32 = 32.0;
                const FOOTER_GAP: f32 = 12.0;
                let footer_total = FOOTER_BTN_W * 2.0 + FOOTER_GAP;
                ui.horizontal(|ui| {
                    let pad = ((ui.available_width() - footer_total) / 2.0).max(0.0);
                    ui.add_space(pad);
                    if ui
                        .add_sized(
                            [FOOTER_BTN_W, FOOTER_BTN_H],
                            egui::Button::new(
                                egui::RichText::new(t.rule_props_save())
                                    .size(text_size::NORMAL)
                                    .color(c.fg_inverse)
                                    .strong(),
                            )
                            .fill(c.accent_success)
                            .corner_radius(15.0),
                        )
                        .clicked()
                    {
                        let mut hold_indices: SmallVec<[u8; 4]> = SmallVec::new();
                        for (idx, &held) in self.hold_flags.iter().enumerate() {
                            if held && idx < 16 {
                                hold_indices.push(idx as u8);
                            }
                        }
                        let append_keys: SmallVec<[String; 4]> =
                            self.append_keys.iter().cloned().collect();
                        self.result = Some(RuleProperties {
                            hold_indices,
                            append_keys,
                        });
                        should_close = true;
                    }
                    ui.add_space(FOOTER_GAP);
                    if ui
                        .add_sized(
                            [FOOTER_BTN_W, FOOTER_BTN_H],
                            egui::Button::new(
                                egui::RichText::new(t.rule_props_cancel())
                                    .size(text_size::NORMAL)
                                    .color(c.fg_inverse)
                                    .strong(),
                            )
                            .fill(c.accent_secondary)
                            .corner_radius(15.0),
                        )
                        .clicked()
                    {
                        self.result = None;
                        should_close = true;
                    }
                });

                ui.add_space(12.0);
            });

        if self.capturing_append {
            // Keep repainting while we wait for the user to press and then
            // release keys, otherwise the dialog would freeze on idle.
            ctx.request_repaint();
        }

        should_close
    }
}
