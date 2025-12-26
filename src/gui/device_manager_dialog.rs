//! Device manager dialog for XInput and HID devices.
//!
//! Provides a GUI interface for managing connected controllers,
//! including vibration testing, deadzone configuration, and API selection.

use crate::config::DeviceApiPreference;
use crate::gui::device_info::{get_device_model, get_hid_device_type, get_vendor_name};
use crate::i18n::CachedTranslations;
use eframe::egui;

/// Information about an XInput device.
#[derive(Clone, Debug)]
pub struct XInputDeviceInfo {
    pub user_index: u32,
    pub vid: u16,
    pub pid: u16,
    pub device_type: String,
}

/// Information about a HID device.
#[derive(Clone, Debug)]
pub struct HidDeviceInfo {
    pub vid: u16,
    pub pid: u16,
    pub device_name: String,
    pub usage_page: u16,
    pub usage: u16,
}

/// Device manager dialog state.
pub struct DeviceManagerDialog {
    /// XInput devices list
    xinput_devices: Vec<XInputDeviceInfo>,
    /// HID devices list  
    hid_devices: Vec<HidDeviceInfo>,
    /// Selected device for detailed view
    selected_device: Option<(u16, u16)>,
    /// Vibration intensity for left motor (0-65535)
    vibration_left: u16,
    /// Vibration intensity for right motor (0-65535)
    vibration_right: u16,
    /// Analog stick deadzone (0-32767)
    stick_deadzone: i16,
    /// Trigger activation threshold (0-255)
    trigger_threshold: u8,
    /// Vibration test duration timer
    vibration_test_until: Option<std::time::Instant>,
    /// Preferred API for each device (VID:PID -> API preference)
    device_api_preference: std::collections::HashMap<(u16, u16), DeviceApiPreference>,
    /// API preferences that changed and need to be saved
    changed_preferences: std::collections::HashMap<(u16, u16), DeviceApiPreference>,
    /// Last device refresh timestamp
    last_refresh: std::time::Instant,
    /// Cached preference lookups for current frame
    preference_cache: [Option<(u16, u16, DeviceApiPreference)>; 8],
    /// Cache hit counter for optimization metrics
    cache_hits: u8,
    /// Show all HID devices including keyboard and mouse
    show_all_hid_devices: bool,
    /// Devices to clear activation for
    devices_to_reactivate: Vec<(u16, u16)>,
}

/// Parses VID:PID device key into numeric tuple.
///
/// Expects hexadecimal format "VVVV:PPPP". Returns None on parse failure.
#[inline]
fn parse_device_key(key: &str) -> Option<(u16, u16)> {
    let (vid_str, pid_str) = key.split_once(':')?;
    let vid = u16::from_str_radix(vid_str, 16).ok()?;
    let pid = u16::from_str_radix(pid_str, 16).ok()?;
    Some((vid, pid))
}

/// Formats VID:PID tuple into string key for persistence.
///
/// Generates uppercase hexadecimal format "VVVV:PPPP".
#[inline]
fn format_device_key(vid: u16, pid: u16) -> String {
    // Pre-allocate exact size (4+1+4 = 9 chars)
    let mut result = String::with_capacity(9);
    use std::fmt::Write;
    let _ = write!(&mut result, "{:04X}:{:04X}", vid, pid);
    result
}

impl Default for DeviceManagerDialog {
    fn default() -> Self {
        Self {
            xinput_devices: Vec::new(),
            hid_devices: Vec::new(),
            selected_device: None,
            vibration_left: 32767,
            vibration_right: 32767,
            stick_deadzone: 7849,
            trigger_threshold: 30,
            vibration_test_until: None,
            device_api_preference: std::collections::HashMap::new(),
            changed_preferences: std::collections::HashMap::new(),
            last_refresh: std::time::Instant::now() - std::time::Duration::from_secs(1),
            preference_cache: [None; 8],
            cache_hits: 0,
            show_all_hid_devices: false,
            devices_to_reactivate: Vec::new(),
        }
    }
}

impl DeviceManagerDialog {
    /// Creates a new device manager dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads API preferences from configuration.
    pub fn load_preferences(
        &mut self,
        preferences: &std::collections::HashMap<String, DeviceApiPreference>,
    ) {
        self.device_api_preference.clear();
        for (key, pref) in preferences {
            if let Some((vid, pid)) = parse_device_key(key) {
                self.device_api_preference.insert((vid, pid), *pref);
            }
        }
    }

    /// Extracts changed API preferences for persistence.
    #[inline]
    pub fn take_changed_preferences(
        &mut self,
    ) -> std::collections::HashMap<String, DeviceApiPreference> {
        let mut result = std::collections::HashMap::with_capacity(self.changed_preferences.len());
        for ((vid, pid), pref) in self.changed_preferences.drain() {
            result.insert(format_device_key(vid, pid), pref);
        }
        result
    }

    /// Extracts device reactivation requests.
    #[inline]
    pub fn take_devices_to_reactivate(&mut self) -> Vec<(u16, u16)> {
        std::mem::take(&mut self.devices_to_reactivate)
    }

    /// Retrieves API preference with frame-local caching.
    #[inline]
    fn get_preference_cached(&mut self, device_key: (u16, u16)) -> DeviceApiPreference {
        for slot in &self.preference_cache {
            if let Some((vid, pid, pref)) = slot
                && (*vid, *pid) == device_key
            {
                self.cache_hits = self.cache_hits.wrapping_add(1);
                return *pref;
            }
        }

        // Cache miss: query HashMap
        let pref = self
            .device_api_preference
            .get(&device_key)
            .copied()
            .unwrap_or(DeviceApiPreference::Auto);

        // Update cache (simple rotation strategy)
        let cache_idx = (device_key.0 ^ device_key.1) as usize % self.preference_cache.len();
        self.preference_cache[cache_idx] = Some((device_key.0, device_key.1, pref));

        pref
    }

    /// Invalidates preference cache for next frame.
    #[inline(always)]
    fn clear_preference_cache(&mut self) {
        self.preference_cache = [None; 8];
        self.cache_hits = 0;
    }

    /// Refreshes device lists from system APIs.
    pub fn refresh_devices(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_refresh) < std::time::Duration::from_millis(100) {
            return;
        }

        self.xinput_devices = crate::xinput::XInputHandler::enumerate_devices();
        self.hid_devices = crate::rawinput::enumerate_hid_devices();
        self.last_refresh = now;
        self.clear_preference_cache();
    }

    /// Renders device manager dialog UI.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        dark_mode: bool,
        translations: &CachedTranslations,
        activated_devices: &std::collections::HashSet<(u16, u16)>,
    ) -> bool {
        self.clear_preference_cache();

        let mut should_close = false;
        let t = translations;

        // Auto-stop vibration after test duration
        if let Some(test_until) = self.vibration_test_until
            && std::time::Instant::now() > test_until
        {
            self.stop_all_vibrations();
            self.vibration_test_until = None;
        }

        // Theme colors
        let (bg_color, title_color, _text_color) = if dark_mode {
            (
                egui::Color32::from_rgb(30, 32, 42),
                egui::Color32::from_rgb(255, 182, 193),
                egui::Color32::from_rgb(220, 220, 220),
            )
        } else {
            (
                egui::Color32::from_rgb(252, 248, 255),
                egui::Color32::from_rgb(219, 112, 147),
                egui::Color32::from_rgb(60, 60, 60),
            )
        };

        egui::Window::new("device_manager")
            .id(egui::Id::new("device_manager_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([700.0, 530.0])
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
                    ui.add_space(25.0);

                    // Title
                    ui.label(
                        egui::RichText::new(t.device_manager_title())
                            .size(28.0)
                            .strong()
                            .color(title_color),
                    );

                    ui.add_space(20.0);
                });

                // Header with refresh button
                let header_bg = if dark_mode {
                    egui::Color32::from_rgb(40, 40, 50)
                } else {
                    egui::Color32::from_rgb(250, 240, 255)
                };

                egui::Frame::NONE
                    .fill(header_bg)
                    .corner_radius(15.0)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(t.connected_devices_title())
                                    .size(16.0)
                                    .strong()
                                    .color(if dark_mode {
                                        egui::Color32::from_rgb(200, 180, 255)
                                    } else {
                                        egui::Color32::from_rgb(150, 100, 200)
                                    }),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let refresh_btn = egui::Button::new(
                                        egui::RichText::new(t.refresh_button())
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(if dark_mode {
                                        egui::Color32::from_rgb(120, 100, 180)
                                    } else {
                                        egui::Color32::from_rgb(180, 150, 230)
                                    })
                                    .corner_radius(12.0);

                                    if ui.add(refresh_btn).clicked() {
                                        self.refresh_devices();
                                    }
                                },
                            );
                        });
                    });

                ui.add_space(12.0);

                // Main content in scroll area with fixed height
                egui::ScrollArea::vertical()
                    .max_height(420.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_xinput_section(ui, dark_mode, t, activated_devices);
                        ui.add_space(15.0);
                        self.render_hid_section(ui, dark_mode, t, activated_devices);
                    });

                ui.add_space(18.0);

                // Close button - centered at bottom (about dialog style)
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            [260.0, 32.0],
                            egui::Button::new(
                                egui::RichText::new(t.device_manager_close_button())
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(216, 191, 216))
                            .corner_radius(15.0),
                        )
                        .clicked()
                    {
                        self.stop_all_vibrations();
                        should_close = true;
                    }

                    ui.add_space(10.0);
                });
            });

        should_close
    }

    /// Renders the XInput devices section.
    fn render_xinput_section(
        &mut self,
        ui: &mut egui::Ui,
        dark_mode: bool,
        t: &CachedTranslations,
        activated_devices: &std::collections::HashSet<(u16, u16)>,
    ) {
        let section_bg = if dark_mode {
            egui::Color32::from_rgb(40, 42, 50)
        } else {
            egui::Color32::from_rgb(245, 238, 252)
        };

        egui::Frame::NONE
            .fill(section_bg)
            .corner_radius(18.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(t.xinput_controllers_title())
                        .size(16.0)
                        .strong()
                        .color(if dark_mode {
                            egui::Color32::from_rgb(150, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 120, 200)
                        }),
                );

                ui.add_space(10.0);

                if self.xinput_devices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("(´・ω・`)")
                                .size(32.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(t.no_controllers_connected())
                                .size(14.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                        ui.add_space(20.0);
                    });
                } else {
                    let devices = self.xinput_devices.clone();
                    for device in &devices {
                        self.render_xinput_device(ui, device, dark_mode, t, activated_devices);
                        ui.add_space(8.0);
                    }
                }
            });
    }

    /// Renders a single XInput device card.
    fn render_xinput_device(
        &mut self,
        ui: &mut egui::Ui,
        device: &XInputDeviceInfo,
        dark_mode: bool,
        t: &CachedTranslations,
        activated_devices: &std::collections::HashSet<(u16, u16)>,
    ) {
        let card_bg = if dark_mode {
            egui::Color32::from_rgb(48, 50, 60)
        } else {
            egui::Color32::from_rgb(245, 238, 252)
        };

        let accent_color = if dark_mode {
            egui::Color32::from_rgb(150, 180, 255)
        } else {
            egui::Color32::from_rgb(120, 80, 180)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(15.0)
            .inner_margin(egui::Margin::same(14))
            .stroke(egui::Stroke::new(
                2.0,
                if dark_mode {
                    egui::Color32::from_rgba_premultiplied(100, 120, 200, 30)
                } else {
                    egui::Color32::from_rgb(150, 120, 200)
                },
            ))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Cute icon
                    ui.label(egui::RichText::new("🎮").size(24.0));

                    ui.add_space(8.0);

                    // Device info with cute styling
                    ui.vertical(|ui| {
                        // Title with model name if available
                        let title = if let Some(model) = get_device_model(device.vid, device.pid) {
                            model.to_string()
                        } else {
                            device.device_type.clone()
                        };

                        ui.label(
                            egui::RichText::new(&title)
                                .size(15.0)
                                .strong()
                                .color(accent_color),
                        );

                        // Subtitle with vendor and technical info
                        let mut info_parts = Vec::with_capacity(4);

                        if let Some(vendor) = get_vendor_name(device.vid) {
                            info_parts.push(format!("🏭 {}", vendor));
                        }

                        info_parts.push(format!("{} {}", t.slot_label(), device.user_index));
                        info_parts.push(format!("{:04X}:{:04X}", device.vid, device.pid));

                        ui.label(
                            egui::RichText::new(info_parts.join(" │ "))
                                .size(11.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_text = if self.selected_device == Some((device.vid, device.pid)) {
                            t.hide_button()
                        } else {
                            t.device_settings_button()
                        };

                        let settings_btn = egui::Button::new(
                            egui::RichText::new(btn_text)
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if dark_mode {
                            egui::Color32::from_rgb(100, 120, 180)
                        } else {
                            egui::Color32::from_rgb(180, 160, 230)
                        })
                        .corner_radius(10.0);

                        if ui.add(settings_btn).clicked() {
                            if self.selected_device == Some((device.vid, device.pid)) {
                                self.selected_device = None;
                            } else {
                                self.selected_device = Some((device.vid, device.pid));
                            }
                        }

                        // Reactivation button
                        let device_key = (device.vid, device.pid);
                        let current_pref = self.get_preference_cached(device_key);
                        let is_activated = activated_devices.contains(&device_key);

                        // Show button only for RawInput API with activation
                        if current_pref == DeviceApiPreference::RawInput && is_activated {
                            ui.add_space(4.0);

                            let reactivate_btn = egui::Button::new(
                                egui::RichText::new(t.reactivate_button())
                                    .size(12.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(if dark_mode {
                                egui::Color32::from_rgb(100, 200, 150)
                            } else {
                                egui::Color32::from_rgb(150, 230, 180)
                            })
                            .corner_radius(10.0);

                            if ui.add(reactivate_btn).clicked() {
                                self.devices_to_reactivate.push(device_key);
                            }
                        }
                    });
                });

                // Expandable settings panel
                if self.selected_device == Some((device.vid, device.pid)) {
                    ui.add_space(12.0);

                    ui.add(egui::Separator::default().spacing(0.0));

                    ui.add_space(12.0);

                    self.render_xinput_settings(ui, device, dark_mode, t);
                }
            });
    }

    /// Renders XInput device settings panel.
    fn render_xinput_settings(
        &mut self,
        ui: &mut egui::Ui,
        device: &XInputDeviceInfo,
        dark_mode: bool,
        t: &CachedTranslations,
    ) {
        let panel_bg = if dark_mode {
            egui::Color32::from_rgb(40, 42, 52)
        } else {
            egui::Color32::from_rgb(242, 235, 250)
        };

        egui::Frame::NONE
            .fill(panel_bg)
            .corner_radius(12.0)
            .inner_margin(egui::Margin::symmetric(16, 14))
            .show(ui, |ui| {
                // Horizontal layout for vibration and deadzone settings
                ui.horizontal(|ui| {
                    // Left panel - Vibration control
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width() * 0.48);

                        ui.label(
                            egui::RichText::new(t.vibration_control_title())
                                .size(15.0)
                                .strong()
                                .color(if dark_mode {
                                    egui::Color32::from_rgb(200, 150, 255)
                                } else {
                                    egui::Color32::from_rgb(130, 80, 180)
                                }),
                        );
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(60.0, 20.0),
                                egui::Label::new(
                                    egui::RichText::new(t.left_motor_label())
                                        .size(13.0)
                                        .strong(),
                                ),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.vibration_left, 0..=65535)
                                    .text(t.power_label())
                                    .show_value(true),
                            );
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(60.0, 20.0),
                                egui::Label::new(
                                    egui::RichText::new(t.right_motor_label())
                                        .size(13.0)
                                        .strong(),
                                ),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.vibration_right, 0..=65535)
                                    .text(t.power_label())
                                    .show_value(true),
                            );
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let test_btn = egui::Button::new(
                                egui::RichText::new(t.test_vibration_button())
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(if dark_mode {
                                egui::Color32::from_rgb(100, 200, 150)
                            } else {
                                egui::Color32::from_rgb(150, 230, 180)
                            })
                            .corner_radius(8.0);

                            if ui.add(test_btn).clicked() {
                                self.test_vibration(device.user_index);
                            }

                            let stop_btn = egui::Button::new(
                                egui::RichText::new(t.stop_vibration_button())
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(if dark_mode {
                                egui::Color32::from_rgb(200, 100, 120)
                            } else {
                                egui::Color32::from_rgb(255, 150, 170)
                            })
                            .corner_radius(8.0);

                            if ui.add(stop_btn).clicked() {
                                self.stop_vibration(device.user_index);
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // Right panel - Deadzone settings
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());

                        ui.label(
                            egui::RichText::new(t.deadzone_settings_title())
                                .size(15.0)
                                .strong()
                                .color(if dark_mode {
                                    egui::Color32::from_rgb(150, 200, 255)
                                } else {
                                    egui::Color32::from_rgb(80, 120, 180)
                                }),
                        );
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(60.0, 20.0),
                                egui::Label::new(
                                    egui::RichText::new(t.stick_label()).size(13.0).strong(),
                                ),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.stick_deadzone, 0..=32767)
                                    .text(t.threshold_label())
                                    .show_value(true),
                            );
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(60.0, 20.0),
                                egui::Label::new(
                                    egui::RichText::new(t.trigger_label_short())
                                        .size(13.0)
                                        .strong(),
                                ),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.trigger_threshold, 0..=255)
                                    .text(t.threshold_label())
                                    .show_value(true),
                            );
                        });
                    });
                });

                ui.add_space(10.0);

                // API preference section
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t.preferred_api_label())
                            .size(14.0)
                            .strong()
                            .color(if dark_mode {
                                egui::Color32::from_rgb(200, 180, 255)
                            } else {
                                egui::Color32::from_rgb(120, 80, 180)
                            }),
                    );

                    let device_key = (device.vid, device.pid);
                    let current_pref = self.get_preference_cached(device_key);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if ui
                            .add(egui::RadioButton::new(
                                current_pref == DeviceApiPreference::Auto,
                                t.api_auto(),
                            ))
                            .clicked()
                        {
                            self.device_api_preference
                                .insert(device_key, DeviceApiPreference::Auto);
                            self.changed_preferences
                                .insert(device_key, DeviceApiPreference::Auto);
                            self.clear_preference_cache();
                        }

                        if ui
                            .add(egui::RadioButton::new(
                                current_pref == DeviceApiPreference::XInput,
                                t.api_xinput(),
                            ))
                            .clicked()
                        {
                            self.device_api_preference
                                .insert(device_key, DeviceApiPreference::XInput);
                            self.changed_preferences
                                .insert(device_key, DeviceApiPreference::XInput);
                            self.clear_preference_cache();
                        }

                        if ui
                            .add(egui::RadioButton::new(
                                current_pref == DeviceApiPreference::RawInput,
                                t.api_rawinput(),
                            ))
                            .clicked()
                        {
                            self.device_api_preference
                                .insert(device_key, DeviceApiPreference::RawInput);
                            self.changed_preferences
                                .insert(device_key, DeviceApiPreference::RawInput);
                            self.clear_preference_cache();
                        }
                    });
                });
            });
    }

    /// Renders the HID devices section.
    fn render_hid_section(
        &mut self,
        ui: &mut egui::Ui,
        dark_mode: bool,
        t: &CachedTranslations,
        activated_devices: &std::collections::HashSet<(u16, u16)>,
    ) {
        let section_bg = if dark_mode {
            egui::Color32::from_rgb(40, 42, 50)
        } else {
            egui::Color32::from_rgb(238, 248, 242)
        };

        egui::Frame::NONE
            .fill(section_bg)
            .corner_radius(18.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Header with title and filter toggle
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t.hid_devices_title())
                            .size(16.0)
                            .strong()
                            .color(if dark_mode {
                                egui::Color32::from_rgb(200, 255, 150)
                            } else {
                                egui::Color32::from_rgb(80, 150, 90)
                            }),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Cute filter toggle button
                        let filter_bg = if self.show_all_hid_devices {
                            if dark_mode {
                                egui::Color32::from_rgb(120, 200, 140)
                            } else {
                                egui::Color32::from_rgb(150, 230, 170)
                            }
                        } else if dark_mode {
                            egui::Color32::from_rgb(80, 80, 90)
                        } else {
                            egui::Color32::from_rgb(200, 200, 210)
                        };

                        let filter_text = if self.show_all_hid_devices {
                            t.all_devices_filter()
                        } else {
                            t.game_devices_only_filter()
                        };

                        let filter_btn = egui::Button::new(
                            egui::RichText::new(filter_text)
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(filter_bg)
                        .corner_radius(12.0);

                        if ui.add(filter_btn).clicked() {
                            self.show_all_hid_devices = !self.show_all_hid_devices;
                        }
                    });
                });

                ui.add_space(10.0);

                // Filter devices based on toggle state
                let filtered_devices: Vec<_> = if self.show_all_hid_devices {
                    self.hid_devices.clone()
                } else {
                    self.hid_devices
                        .iter()
                        .filter(|device| {
                            // Only show gaming devices (gamepads, joysticks)
                            matches!(
                                (device.usage_page, device.usage),
                                (0x0001, 0x0004) | // Joystick
                                (0x0001, 0x0005) | // Gamepad
                                (0x0001, 0x0008) // Multi-axis Controller
                            )
                        })
                        .cloned()
                        .collect()
                };

                if filtered_devices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("(｡•́︿•̀｡)")
                                .size(32.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                        ui.add_space(8.0);
                        let empty_text = if self.show_all_hid_devices {
                            t.no_hid_devices_detected()
                        } else {
                            t.no_game_devices_detected()
                        };
                        ui.label(
                            egui::RichText::new(empty_text)
                                .size(14.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                        ui.add_space(20.0);
                    });
                } else {
                    for device in filtered_devices {
                        self.render_hid_device(ui, &device, dark_mode, activated_devices, t);
                        ui.add_space(8.0);
                    }
                }
            });
    }

    /// Renders a single HID device card.
    fn render_hid_device(
        &mut self,
        ui: &mut egui::Ui,
        device: &HidDeviceInfo,
        dark_mode: bool,
        activated_devices: &std::collections::HashSet<(u16, u16)>,
        t: &CachedTranslations,
    ) {
        let card_bg = if dark_mode {
            egui::Color32::from_rgb(48, 50, 60)
        } else {
            egui::Color32::from_rgb(238, 248, 242)
        };

        let accent_color = if dark_mode {
            egui::Color32::from_rgb(150, 255, 180)
        } else {
            egui::Color32::from_rgb(60, 140, 80)
        };

        egui::Frame::NONE
            .fill(card_bg)
            .corner_radius(15.0)
            .inner_margin(egui::Margin::same(14))
            .stroke(egui::Stroke::new(
                2.0,
                if dark_mode {
                    egui::Color32::from_rgba_premultiplied(100, 200, 120, 30)
                } else {
                    egui::Color32::from_rgb(120, 200, 140)
                },
            ))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Cute icon
                    ui.label(egui::RichText::new("🕹").size(24.0));

                    ui.add_space(8.0);

                    // Device info
                    ui.vertical(|ui| {
                        // Title with model name if available, otherwise device name
                        let title = if let Some(model) = get_device_model(device.vid, device.pid) {
                            model.to_string()
                        } else {
                            device.device_name.clone()
                        };

                        // Add device type tag
                        let device_type = get_hid_device_type(device.usage_page, device.usage);
                        let display_title = if !title.is_empty() {
                            format!("{} ✨ {}", title, device_type)
                        } else {
                            device_type.to_string()
                        };

                        ui.label(
                            egui::RichText::new(&display_title)
                                .size(15.0)
                                .strong()
                                .color(accent_color),
                        );

                        // Subtitle with vendor and technical info
                        let mut info_parts = vec![];

                        if let Some(vendor) = get_vendor_name(device.vid) {
                            info_parts.push(format!("🏭 {}", vendor));
                        }

                        info_parts.push(format!("{:04X}:{:04X}", device.vid, device.pid));
                        info_parts.push(format!(
                            "Usage {:04X}:{:04X}",
                            device.usage_page, device.usage
                        ));

                        ui.label(
                            egui::RichText::new(info_parts.join(" │ "))
                                .size(11.0)
                                .color(ui.style().visuals.weak_text_color()),
                        );
                    });

                    // Reactivation button - always use right_to_left layout for consistent width
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let device_key = (device.vid, device.pid);
                        let is_activated = activated_devices.contains(&device_key);

                        if is_activated {
                            let reactivate_btn = egui::Button::new(
                                egui::RichText::new(t.reactivate_button())
                                    .size(12.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(if dark_mode {
                                egui::Color32::from_rgb(100, 200, 150)
                            } else {
                                egui::Color32::from_rgb(150, 230, 180)
                            })
                            .corner_radius(10.0);

                            if ui.add(reactivate_btn).clicked() {
                                self.devices_to_reactivate.push(device_key);
                            }
                        }
                    });
                });
            });
    }

    /// Tests vibration with given intensity.
    fn test_vibration(&mut self, user_index: u32) {
        crate::xinput::XInputHandler::set_vibration(
            user_index,
            self.vibration_left,
            self.vibration_right,
        );
        self.vibration_test_until =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
    }

    /// Stops vibration for a specific device.
    fn stop_vibration(&mut self, user_index: u32) {
        crate::xinput::XInputHandler::set_vibration(user_index, 0, 0);
        self.vibration_test_until = None;
    }

    /// Stops all vibrations.
    fn stop_all_vibrations(&mut self) {
        for device in &self.xinput_devices {
            crate::xinput::XInputHandler::set_vibration(device.user_index, 0, 0);
        }
        self.vibration_test_until = None;
    }
}
