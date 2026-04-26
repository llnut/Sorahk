//! HID device activation dialog for establishing device baseline.

use crate::gui::device_info::{get_device_model, get_hid_device_type, get_vendor_name};
use crate::gui::theme;
use crate::gui::widgets::{self, text_size};
use crate::i18n::CachedTranslations;
use eframe::egui;
use std::time::Instant;

/// State for HID device activation process
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationState {
    WaitingForPress,   // Waiting for user to press a button
    WaitingForRelease, // Waiting for user to release the button
    Success,           // Activation successful
    Failed(String),    // Activation failed with error message
}

/// HID device activation dialog
pub struct HidActivationDialog {
    device_name: String,
    device_handle: isize,
    vid: u16,
    pid: u16,
    usage_page: u16,
    usage: u16,
    pub state: ActivationState,
    pub pressed_data: Option<Vec<u8>>,
    pub released_data: Option<Vec<u8>>,
    success_time: Option<Instant>,
    animation_progress: f32,
}

impl HidActivationDialog {
    #[inline]
    pub fn new(
        device_name: String,
        device_handle: isize,
        vid: u16,
        pid: u16,
        usage_page: u16,
        usage: u16,
    ) -> Self {
        Self {
            device_name,
            device_handle,
            vid,
            pid,
            usage_page,
            usage,
            state: ActivationState::WaitingForPress,
            pressed_data: None,
            released_data: None,
            success_time: None,
            animation_progress: 0.0,
        }
    }

    #[inline(always)]
    pub fn device_handle(&self) -> isize {
        self.device_handle
    }

    /// Render the activation dialog, returns true if should close
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        dark_mode: bool,
        translations: &CachedTranslations,
    ) -> bool {
        let t = translations;
        let c = theme::colors(dark_mode);

        // Translucent warning tint stays inline; the palette has no clean
        // equivalent for the "outline + soft fill" warning box.
        let warning_bg = if dark_mode {
            egui::Color32::from_rgba_premultiplied(80, 60, 30, 40)
        } else {
            egui::Color32::from_rgba_premultiplied(255, 200, 100, 30)
        };

        let mut should_close = false;

        egui::Window::new("hid_activation_dialog")
            .id(egui::Id::new("hid_activation_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground) // Always on top
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(c.bg_card)
                    .corner_radius(egui::CornerRadius::same(widgets::radius::DIALOG))
                    .stroke(egui::Stroke::NONE)
                    .shadow(egui::epaint::Shadow {
                        // Heavier shadow than other dialogs because this
                        // panel renders on the Foreground order layer.
                        offset: [0, 8],
                        blur: 28,
                        spread: 3,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 60),
                    }),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(30.0);

                    // Title with state-dependent color/animation
                    match &self.state {
                        ActivationState::WaitingForPress | ActivationState::WaitingForRelease => {
                            let time = ui.input(|i| i.time);
                            let bounce = (time * 2.0).sin() * 3.0;
                            ui.add_space(bounce as f32);

                            ui.label(
                                egui::RichText::new(t.hid_activation_title())
                                    .size(text_size::HERO)
                                    .color(c.accent_pink)
                                    .strong(),
                            );
                        }
                        ActivationState::Success => {
                            ui.label(
                                egui::RichText::new(t.hid_activation_success_title())
                                    .size(text_size::HERO)
                                    .color(c.accent_success)
                                    .strong(),
                            );
                        }
                        ActivationState::Failed(_) => {
                            ui.label(
                                egui::RichText::new(t.hid_activation_failed_title())
                                    .size(text_size::HERO)
                                    .color(c.accent_danger)
                                    .strong(),
                            );
                        }
                    }

                    ui.add_space(20.0);

                    // Device-name card. The asymmetric inner_margin is
                    // intentional for this style, so an inline Frame
                    // stays here instead of widgets::card_frame.
                    egui::Frame::NONE
                        .fill(c.bg_card_hover)
                        .corner_radius(egui::CornerRadius::same(widgets::radius::BUTTON))
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 2],
                            blur: 6,
                            spread: 0,
                            color: theme::overlay::SHADOW_LIGHT,
                        })
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                let device_type = get_hid_device_type(self.usage_page, self.usage);
                                let device_icon = match device_type {
                                    "Gamepad" | "Joystick" | "Multi-axis Controller" => "🎮",
                                    "Keyboard" => "⌨",
                                    "Mouse" => "🖱",
                                    _ => "📱",
                                };

                                ui.label(egui::RichText::new(device_icon).size(20.0));
                                ui.add_space(6.0);

                                let display_name =
                                    if let Some(model) = get_device_model(self.vid, self.pid) {
                                        model.to_string()
                                    } else if !self.device_name.is_empty() {
                                        self.device_name.clone()
                                    } else {
                                        device_type.to_string()
                                    };

                                ui.label(
                                    egui::RichText::new(&display_name)
                                        .size(text_size::SUBTITLE)
                                        .strong()
                                        .color(c.fg_primary),
                                );

                                ui.add_space(4.0);

                                let mut info_parts = vec![];
                                if let Some(vendor) = get_vendor_name(self.vid) {
                                    info_parts.push(vendor.to_string());
                                }
                                info_parts.push(format!("{:04X}:{:04X}", self.vid, self.pid));

                                if !info_parts.is_empty() {
                                    ui.label(
                                        egui::RichText::new(info_parts.join(" │ "))
                                            .size(text_size::COMPACT)
                                            .color(c.fg_muted),
                                    );
                                }
                            });
                        });

                    ui.add_space(25.0);

                    // State-specific content.
                    match &self.state {
                        ActivationState::WaitingForPress => {
                            ui.label(
                                egui::RichText::new(t.hid_activation_press_prompt())
                                    .size(text_size::SECTION)
                                    .color(c.accent_pink)
                                    .strong(),
                            );

                            ui.add_space(15.0);

                            // Warning box: translucent fill + warning-color stroke.
                            egui::Frame::NONE
                                .fill(warning_bg)
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::same(15))
                                .stroke(egui::Stroke::new(2.0, c.accent_warning))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(t.hid_activation_warning_title())
                                                .size(text_size::SUBTITLE)
                                                .color(c.accent_warning)
                                                .strong(),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new(t.hid_activation_warning_1())
                                                .size(text_size::BODY)
                                                .color(c.fg_primary),
                                        );
                                        ui.label(
                                            egui::RichText::new(t.hid_activation_warning_2())
                                                .size(text_size::BODY)
                                                .color(c.fg_primary),
                                        );
                                        ui.label(
                                            egui::RichText::new(t.hid_activation_warning_3())
                                                .size(text_size::BODY)
                                                .color(c.fg_primary),
                                        );
                                    });
                                });
                        }

                        ActivationState::WaitingForRelease => {
                            // Pulse color stays inline because the math
                            // interpolates between specific base RGB values.
                            let time = ui.input(|i| i.time);
                            let pulse = ((time * 3.0).sin() + 1.0) / 2.0;
                            let pulse_color = egui::Color32::from_rgb(
                                (144.0 + 111.0 * pulse) as u8,
                                (238.0 + 17.0 * pulse) as u8,
                                (144.0 + 111.0 * pulse) as u8,
                            );

                            ui.label(
                                egui::RichText::new(t.hid_activation_release_prompt())
                                    .size(text_size::SECTION)
                                    .color(pulse_color)
                                    .strong(),
                            );
                        }

                        ActivationState::Success => {
                            if let Some(success_time) = self.success_time {
                                self.animation_progress =
                                    success_time.elapsed().as_secs_f32().min(2.0);
                            } else {
                                self.success_time = Some(Instant::now());
                            }

                            // Decorative stars animation, sized via inline literal.
                            let stars = "✨ ⭐ 💫 🌟 ✨ ⭐ 💫 🌟";
                            ui.label(egui::RichText::new(stars).size(24.0).color(c.accent_success));

                            ui.add_space(10.0);

                            ui.label(
                                egui::RichText::new(t.hid_activation_success_message())
                                    .size(text_size::SUBTITLE)
                                    .color(c.fg_primary),
                            );

                            ui.label(
                                egui::RichText::new(t.hid_activation_success_hint())
                                    .size(text_size::NORMAL)
                                    .color(c.fg_primary),
                            );

                            if self.animation_progress >= 1.0 {
                                ui.add_space(15.0);
                                ui.label(
                                    egui::RichText::new(t.hid_activation_auto_close())
                                        .size(text_size::COMPACT)
                                        .color(c.fg_muted)
                                        .italics(),
                                );

                                if self.animation_progress >= 2.0 {
                                    should_close = true;
                                }
                            }
                        }

                        ActivationState::Failed(error_msg) => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}: {}",
                                    t.hid_activation_error(),
                                    error_msg
                                ))
                                .size(text_size::NORMAL)
                                .color(c.accent_danger),
                            );

                            ui.add_space(15.0);

                            let retry_btn = egui::Button::new(
                                egui::RichText::new(t.hid_activation_retry())
                                    .size(text_size::SUBTITLE)
                                    .color(c.fg_inverse),
                            )
                            .fill(c.accent_primary)
                            .corner_radius(12.0);

                            if ui.add_sized([120.0, 40.0], retry_btn).clicked() {
                                self.state = ActivationState::WaitingForPress;
                                self.pressed_data = None;
                                self.released_data = None;
                            }
                        }
                    }

                    ui.add_space(30.0);

                    // Cancel button only while waiting; success / failed states
                    // close themselves.
                    if matches!(
                        self.state,
                        ActivationState::WaitingForPress | ActivationState::WaitingForRelease
                    ) {
                        ui.add_space(20.0);

                        let cancel_btn = egui::Button::new(
                            egui::RichText::new(t.hid_activation_cancel())
                                .size(text_size::SUBTITLE)
                                .color(c.fg_inverse)
                                .strong(),
                        )
                        .fill(c.accent_secondary)
                        .corner_radius(15.0);

                        if ui.add_sized([260.0, 32.0], cancel_btn).clicked() {
                            should_close = true;
                        }
                    }
                });
            });

        ctx.request_repaint();

        should_close
    }

    /// Handle incoming HID data during activation
    #[inline]
    pub fn handle_hid_data(&mut self, data: &[u8]) {
        match self.state {
            ActivationState::WaitingForPress => {
                self.pressed_data = Some(data.to_vec());
                self.state = ActivationState::WaitingForRelease;
            }
            ActivationState::WaitingForRelease => {
                self.released_data = Some(data.to_vec());

                if let (Some(pressed), Some(released)) = (&self.pressed_data, &self.released_data) {
                    if is_more_idle(released, pressed) {
                        self.state = ActivationState::Success;
                    } else {
                        self.state = ActivationState::Failed(
                            "Abnormal detection! Please press and release a single button"
                                .to_string(),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Get the established baseline data
    #[inline]
    pub fn get_baseline(&self) -> Option<Vec<u8>> {
        if matches!(self.state, ActivationState::Success) {
            self.released_data.clone()
        } else {
            None
        }
    }
}

/// Check if data1 is more "idle" than data2 (fewer active bits)
#[inline]
fn is_more_idle(data1: &[u8], data2: &[u8]) -> bool {
    const SKIP_BYTES: usize = 5; // Skip protocol header and analog axes
    let count1 = data1
        .iter()
        .skip(SKIP_BYTES)
        .map(|b| b.count_ones())
        .sum::<u32>();
    let count2 = data2
        .iter()
        .skip(SKIP_BYTES)
        .map(|b| b.count_ones())
        .sum::<u32>();
    count1 < count2
}
