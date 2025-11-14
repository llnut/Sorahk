use crate::config::{AppConfig, KeyMapping};
use crate::state::{AppState, NotificationEvent};
use eframe::egui;
use std::sync::Arc;

// Enum to track which input is waiting for key press
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyCaptureMode {
    None,
    ToggleKey,
    MappingTrigger(usize), // Index of mapping being edited
    MappingTarget(usize),
    NewMappingTrigger,
    NewMappingTarget,
}

pub struct SorahkGui {
    app_state: Arc<AppState>,
    config: AppConfig,
    show_close_dialog: bool,
    show_settings_dialog: bool,
    show_about_dialog: bool,
    minimize_on_close: bool,
    dark_mode: bool,
    // Temporary settings for editing
    temp_config: Option<AppConfig>,
    // UI state for editing
    new_mapping_trigger: String,
    new_mapping_target: String,
    new_mapping_interval: String,
    new_mapping_duration: String,
    new_process_name: String,
    // Key capture state
    key_capture_mode: KeyCaptureMode,
    // Close dialog highlight effect
    dialog_highlight_until: Option<std::time::Instant>,
}

impl SorahkGui {
    pub fn new(app_state: Arc<AppState>, config: AppConfig) -> Self {
        // Load theme from config (defaults to light theme if not set)
        let dark_mode = config.dark_mode;
        println!(
            "Loaded theme from config: {}",
            if dark_mode { "dark" } else { "light" }
        );

        Self {
            app_state,
            config,
            show_close_dialog: false,
            show_settings_dialog: false,
            show_about_dialog: false,
            minimize_on_close: true, // Default: minimize instead of exit
            dialog_highlight_until: None,
            dark_mode,
            temp_config: None,
            new_mapping_trigger: String::new(),
            new_mapping_target: String::new(),
            new_mapping_interval: String::new(),
            new_mapping_duration: String::new(),
            new_process_name: String::new(),
            key_capture_mode: KeyCaptureMode::None,
        }
    }

    // Helper function to convert egui::Key to virtual key code string
    fn key_to_string(key: egui::Key) -> Option<String> {
        let key_name = match key {
            egui::Key::A => "A",
            egui::Key::B => "B",
            egui::Key::C => "C",
            egui::Key::D => "D",
            egui::Key::E => "E",
            egui::Key::F => "F",
            egui::Key::G => "G",
            egui::Key::H => "H",
            egui::Key::I => "I",
            egui::Key::J => "J",
            egui::Key::K => "K",
            egui::Key::L => "L",
            egui::Key::M => "M",
            egui::Key::N => "N",
            egui::Key::O => "O",
            egui::Key::P => "P",
            egui::Key::Q => "Q",
            egui::Key::R => "R",
            egui::Key::S => "S",
            egui::Key::T => "T",
            egui::Key::U => "U",
            egui::Key::V => "V",
            egui::Key::W => "W",
            egui::Key::X => "X",
            egui::Key::Y => "Y",
            egui::Key::Z => "Z",
            egui::Key::Num0 => "0",
            egui::Key::Num1 => "1",
            egui::Key::Num2 => "2",
            egui::Key::Num3 => "3",
            egui::Key::Num4 => "4",
            egui::Key::Num5 => "5",
            egui::Key::Num6 => "6",
            egui::Key::Num7 => "7",
            egui::Key::Num8 => "8",
            egui::Key::Num9 => "9",
            egui::Key::F1 => "F1",
            egui::Key::F2 => "F2",
            egui::Key::F3 => "F3",
            egui::Key::F4 => "F4",
            egui::Key::F5 => "F5",
            egui::Key::F6 => "F6",
            egui::Key::F7 => "F7",
            egui::Key::F8 => "F8",
            egui::Key::F9 => "F9",
            egui::Key::F10 => "F10",
            egui::Key::F11 => "F11",
            egui::Key::F12 => "F12",
            egui::Key::Delete => "DELETE",
            egui::Key::Insert => "INSERT",
            egui::Key::Home => "HOME",
            egui::Key::End => "END",
            egui::Key::PageUp => "PAGEUP",
            egui::Key::PageDown => "PAGEDOWN",
            egui::Key::Space => "SPACE",
            egui::Key::Tab => "TAB",
            egui::Key::Escape => "ESCAPE",
            egui::Key::Enter => "RETURN",
            egui::Key::Backspace => "BACK",
            egui::Key::ArrowLeft => "LEFT",
            egui::Key::ArrowRight => "RIGHT",
            egui::Key::ArrowUp => "UP",
            egui::Key::ArrowDown => "DOWN",
            _ => return None,
        };
        Some(key_name.to_string())
    }

    // Helper function to convert virtual key code string to egui::Key
    fn string_to_key(key_name: &str) -> Option<egui::Key> {
        let key_upper = key_name.to_uppercase();
        match key_upper.as_str() {
            "A" => Some(egui::Key::A),
            "B" => Some(egui::Key::B),
            "C" => Some(egui::Key::C),
            "D" => Some(egui::Key::D),
            "E" => Some(egui::Key::E),
            "F" => Some(egui::Key::F),
            "G" => Some(egui::Key::G),
            "H" => Some(egui::Key::H),
            "I" => Some(egui::Key::I),
            "J" => Some(egui::Key::J),
            "K" => Some(egui::Key::K),
            "L" => Some(egui::Key::L),
            "M" => Some(egui::Key::M),
            "N" => Some(egui::Key::N),
            "O" => Some(egui::Key::O),
            "P" => Some(egui::Key::P),
            "Q" => Some(egui::Key::Q),
            "R" => Some(egui::Key::R),
            "S" => Some(egui::Key::S),
            "T" => Some(egui::Key::T),
            "U" => Some(egui::Key::U),
            "V" => Some(egui::Key::V),
            "W" => Some(egui::Key::W),
            "X" => Some(egui::Key::X),
            "Y" => Some(egui::Key::Y),
            "Z" => Some(egui::Key::Z),
            "0" => Some(egui::Key::Num0),
            "1" => Some(egui::Key::Num1),
            "2" => Some(egui::Key::Num2),
            "3" => Some(egui::Key::Num3),
            "4" => Some(egui::Key::Num4),
            "5" => Some(egui::Key::Num5),
            "6" => Some(egui::Key::Num6),
            "7" => Some(egui::Key::Num7),
            "8" => Some(egui::Key::Num8),
            "9" => Some(egui::Key::Num9),
            "F1" => Some(egui::Key::F1),
            "F2" => Some(egui::Key::F2),
            "F3" => Some(egui::Key::F3),
            "F4" => Some(egui::Key::F4),
            "F5" => Some(egui::Key::F5),
            "F6" => Some(egui::Key::F6),
            "F7" => Some(egui::Key::F7),
            "F8" => Some(egui::Key::F8),
            "F9" => Some(egui::Key::F9),
            "F10" => Some(egui::Key::F10),
            "F11" => Some(egui::Key::F11),
            "F12" => Some(egui::Key::F12),
            "DELETE" => Some(egui::Key::Delete),
            "INSERT" => Some(egui::Key::Insert),
            "HOME" => Some(egui::Key::Home),
            "END" => Some(egui::Key::End),
            "PAGEUP" => Some(egui::Key::PageUp),
            "PAGEDOWN" => Some(egui::Key::PageDown),
            "SPACE" => Some(egui::Key::Space),
            "TAB" => Some(egui::Key::Tab),
            "ESCAPE" | "ESC" => Some(egui::Key::Escape),
            "RETURN" | "ENTER" => Some(egui::Key::Enter),
            "BACK" | "BACKSPACE" => Some(egui::Key::Backspace),
            "LEFT" => Some(egui::Key::ArrowLeft),
            "RIGHT" => Some(egui::Key::ArrowRight),
            "UP" => Some(egui::Key::ArrowUp),
            "DOWN" => Some(egui::Key::ArrowDown),
            _ => None,
        }
    }

    // Load icon from sorahk.ico file, with fallback to programmatic generation
    fn create_icon() -> egui::IconData {
        // Try to load from resources/sorahk.ico first
        if let Ok(icon_data) = Self::load_icon_from_file("resources/sorahk.ico") {
            return icon_data;
        }

        // Fallback: Create programmatically generated icon
        Self::create_fallback_icon()
    }

    // Load icon from .ico file
    fn load_icon_from_file(path: &str) -> Result<egui::IconData, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let icon_dir = ico::IconDir::read(file)?;

        // Try to get the best quality icon (prefer 32x32 or larger)
        let entry = icon_dir
            .entries()
            .iter()
            .filter(|e| e.width() >= 32)
            .max_by_key(|e| e.width())
            .or_else(|| icon_dir.entries().first())
            .ok_or("No icon entries found")?;

        let image = entry.decode()?;
        let rgba_data = image.rgba_data().to_vec();

        Ok(egui::IconData {
            rgba: rgba_data,
            width: image.width(),
            height: image.height(),
        })
    }

    // Fallback: programmatically generated icon (sora theme colors)
    fn create_fallback_icon() -> egui::IconData {
        const SIZE: usize = 32;
        let mut rgba = vec![0u8; SIZE * SIZE * 4];

        for y in 0..SIZE {
            for x in 0..SIZE {
                let idx = (y * SIZE + x) * 4;

                // Calculate distance from center for circular shape
                let dx = x as f32 - SIZE as f32 / 2.0;
                let dy = y as f32 - SIZE as f32 / 2.0;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = SIZE as f32 / 2.0;

                if dist < radius {
                    // Sky blue background gradient (sora - sky blue)
                    let gradient = 1.0 - (dist / radius) * 0.3;
                    rgba[idx] = (176.0 * gradient) as u8; // R
                    rgba[idx + 1] = (224.0 * gradient) as u8; // G
                    rgba[idx + 2] = (230.0 * gradient) as u8; // B
                    rgba[idx + 3] = 255; // A

                    // Draw a simple "S" shape in the center (silver white)
                    let in_s = (10..=25).contains(&y)
                        && (
                            ((10..=13).contains(&y) && (12..=20).contains(&x)) || // top bar
                        ((13..=16).contains(&y) && (12..=15).contains(&x)) || // left part
                        ((16..=19).contains(&y) && (12..=20).contains(&x)) || // middle bar
                        ((19..=22).contains(&y) && (17..=20).contains(&x)) || // right part
                        ((22..=25).contains(&y) && (12..=20).contains(&x))
                            // bottom bar
                        );

                    if in_s {
                        rgba[idx] = 232; // R - silver white
                        rgba[idx + 1] = 239; // G
                        rgba[idx + 2] = 245; // B
                        rgba[idx + 3] = 255; // A
                    }
                } else {
                    // Transparent outside circle
                    rgba[idx + 3] = 0;
                }
            }
        }

        egui::IconData {
            rgba,
            width: SIZE as u32,
            height: SIZE as u32,
        }
    }

    pub fn run(app_state: Arc<AppState>, config: AppConfig) -> anyhow::Result<()> {
        // Load icon from resources/sorahk.ico or use fallback
        let icon = Self::create_icon();

        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([580.0, 530.0])
            .with_min_inner_size([500.0, 480.0])
            .with_resizable(true)
            .with_title("Sorahk - Auto Key Press Tool")
            .with_icon(icon)
            .with_taskbar(false); // Hide from taskbar when minimized

        // Apply always on top setting based on config
        if config.always_on_top {
            viewport = viewport.with_always_on_top();
        }

        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        eframe::run_native(
            "Sorahk",
            options,
            Box::new(|_cc| Ok(Box::new(SorahkGui::new(app_state, config)))),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
    }

    /// Show error dialog (anime style)
    pub fn show_error(error_msg: &str) -> anyhow::Result<()> {
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

        let icon = Self::create_icon();
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
}

impl eframe::App for SorahkGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if exit was requested at the very beginning
        // This ensures smooth exit without black screen
        if self.app_state.should_exit() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return; // Stop immediately, don't render anything
        }

        // Apply anime theme with custom styling
        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        // Anime style: large rounded corners + soft colors
        visuals.widgets.inactive.rounding = egui::Rounding::same(15.0); // Larger rounding
        visuals.widgets.hovered.rounding = egui::Rounding::same(15.0);
        visuals.widgets.active.rounding = egui::Rounding::same(15.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(10.0); // TextEdit rounding
        visuals.widgets.open.rounding = egui::Rounding::same(15.0);

        // Set selection rounding for text inputs
        visuals.selection.stroke.width = 1.5;

        // Ensure all widgets including TextEdit have consistent rounding
        // TextEdit uses inactive/hovered/active states when focused
        visuals.widgets.inactive.bg_stroke.width = 1.0;
        visuals.widgets.hovered.bg_stroke.width = 1.5;
        visuals.widgets.active.bg_stroke.width = 1.5;

        // Soft background colors (sora theme)
        if !self.dark_mode {
            // Light: very pale lavender
            visuals.window_fill = egui::Color32::from_rgb(250, 250, 255);
            visuals.panel_fill = egui::Color32::from_rgb(248, 250, 255);
            visuals.faint_bg_color = egui::Color32::from_rgb(245, 248, 255);
        } else {
            // Dark: soft dark blue-gray
            visuals.window_fill = egui::Color32::from_rgb(30, 32, 40);
            visuals.panel_fill = egui::Color32::from_rgb(35, 37, 45);
            visuals.faint_bg_color = egui::Color32::from_rgb(40, 42, 50);
        }

        // Soft shadow effects
        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 18.0,
            spread: 0.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 25),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 12.0,
            spread: 0.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 20),
        };

        ctx.set_visuals(visuals);

        // Only repaint when needed (every 100ms) to save CPU
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Check if window should be shown (from tray menu or double-click)
        if self.app_state.check_and_clear_show_window_request() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Check if about dialog should be shown (from tray menu)
        if self.app_state.check_and_clear_show_about_request() {
            self.show_about_dialog = true;
        }

        // Intercept window close request (but not if exit was already requested)
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.app_state.should_exit() {
                // Exit already requested, allow window to close immediately
            } else if self.minimize_on_close {
                // Always cancel close and show dialog
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

                if !self.show_close_dialog {
                    // First time: show dialog
                    self.show_close_dialog = true;
                    self.dialog_highlight_until = None;
                } else {
                    // Subsequent clicks: highlight the dialog to remind user
                    self.dialog_highlight_until =
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
                    // Request repaint to show the highlight effect
                    ctx.request_repaint();
                }
            }
        }

        // Show close confirmation dialog with modern design
        if self.show_close_dialog {
            // Check if we should highlight the dialog
            let should_highlight = self
                .dialog_highlight_until
                .map(|until| std::time::Instant::now() < until)
                .unwrap_or(false);

            // If highlighting, request continuous repaint
            if should_highlight {
                ctx.request_repaint();
            }

            let mut window = egui::Window::new("")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size([380.0, 280.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

            // Add highlight effect with stroke
            if should_highlight {
                window = window.frame(egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(
                    3.0,
                    egui::Color32::from_rgb(255, 200, 0), // Orange/yellow highlight
                )));
            }

            window.show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Title with icon
                    ui.label(
                        egui::RichText::new("❓ Close Window")
                            .size(20.0)
                            .strong()
                            .color(if self.dark_mode {
                                egui::Color32::from_rgb(255, 255, 255)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            }),
                    );

                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new("What would you like to do?")
                            .size(14.0)
                            .color(egui::Color32::GRAY),
                    );

                    ui.add_space(25.0);

                    // Anime-style button styling
                    let button_width = 320.0;
                    let button_height = 32.0; // Reduced from 36.0

                    // Minimize button - Sky Blue (anime style) - only show if tray icon is enabled
                    let tray_enabled = self.config.show_tray_icon;

                    if tray_enabled {
                        let minimize_btn = egui::Button::new(
                            egui::RichText::new("🗕  Minimize to Tray")
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(135, 206, 235))
                        .rounding(15.0);

                        if ui
                            .add_sized([button_width, button_height], minimize_btn)
                            .clicked()
                        {
                            self.show_close_dialog = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }

                        ui.add_space(12.0);
                    }

                    // Exit button - Soft Pink (anime style)
                    let exit_btn = egui::Button::new(
                        egui::RichText::new("🚪  Exit Program")
                            .size(14.0) // Slightly reduced from 15.0
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink
                    .rounding(15.0);

                    if ui
                        .add_sized([button_width, button_height], exit_btn)
                        .clicked()
                    {
                        self.show_close_dialog = false;
                        self.app_state.exit(); // This sets the should_exit flag
                    }

                    ui.add_space(12.0);

                    // Cancel button - Subtle Gray
                    let cancel_btn =
                        egui::Button::new(egui::RichText::new("Cancel").size(13.0).color(
                            // Slightly reduced from 14.0
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(80, 80, 80)
                            },
                        ))
                        .fill(if self.dark_mode {
                            egui::Color32::from_rgb(60, 60, 60)
                        } else {
                            egui::Color32::from_rgb(230, 230, 230)
                        })
                        .rounding(10.0);

                    if ui
                        .add_sized([button_width, button_height], cancel_btn)
                        .clicked()
                    {
                        self.show_close_dialog = false;
                    }

                    ui.add_space(15.0);
                });
            });
        }

        // Show settings dialog
        if self.show_settings_dialog {
            self.render_settings_dialog(ctx);
        }

        // Show about dialog
        if self.show_about_dialog {
            self.render_about_dialog(ctx);
        }

        // Handle keyboard input for switch key and mapped keys
        ctx.input(|i| {
            // Check for configured switch key press
            if let Some(switch_key) = Self::string_to_key(&self.config.switch_key)
                && i.key_pressed(switch_key)
            {
                println!(
                    "GUI: {} key pressed, toggling state",
                    self.config.switch_key
                );
                let was_paused = self.app_state.toggle_paused();
                // was_paused is the OLD state, so if it was paused, it's now active
                if was_paused {
                    // Send notification if enabled
                    if let Some(sender) = self.app_state.get_notification_sender() {
                        let _ =
                            sender.send(NotificationEvent::Info("Sorahk activiting".to_string()));
                    }
                } else {
                    // Send notification if enabled
                    if let Some(sender) = self.app_state.get_notification_sender() {
                        let _ = sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
                    }
                }
            }

            // Handle mapped keys - forward to keyboard hook by checking raw events
            // Note: egui consumes keyboard events, preventing the hook from seeing them
            // when the GUI window has focus. We detect the keys here instead.

            if !self.app_state.is_paused() {
                // Check for mapped keys: A, B, F1, F2, LSHIFT
                let mut trigger_keys = vec![];

                if i.key_down(egui::Key::A) {
                    trigger_keys.push(('A', 0x41));
                }
                if i.key_down(egui::Key::B) {
                    trigger_keys.push(('B', 0x42));
                }
                if i.key_down(egui::Key::F1) {
                    trigger_keys.push(('F', 0x70));
                } // F1
                if i.key_down(egui::Key::F2) {
                    trigger_keys.push(('F', 0x71));
                } // F2

                // Note: egui doesn't have direct LSHIFT detection, we'd need raw input
                // For now, the keyboard hook will handle keys when GUI doesn't have focus

                if !trigger_keys.is_empty() {}
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Anime-style title bar
            ui.horizontal(|ui| {
                ui.add_space(15.0);

                // Decorative title (Sora theme)
                ui.label(
                    egui::RichText::new("🌸 Sorahk ~ Auto Key Press Tool ~")
                        .size(18.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(176, 224, 230) // Sky blue
                        } else {
                            egui::Color32::from_rgb(135, 206, 235) // Deeper sky blue
                        }),
                );

                // Push buttons to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);

                    let theme_icon = if self.dark_mode { "☀" } else { "🌙" };
                    let theme_text = if self.dark_mode { "Light" } else { "Dark" };

                    // Theme toggle button (colorful style to match settings button)
                    let theme_btn = egui::Button::new(
                        egui::RichText::new(format!("{}  {}", theme_icon, theme_text))
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(if self.dark_mode {
                        egui::Color32::from_rgb(255, 200, 100) // Warm orange for light mode toggle
                    } else {
                        egui::Color32::from_rgb(100, 100, 180) // Deep blue for dark mode toggle
                    })
                    .rounding(12.0);

                    if ui.add(theme_btn).clicked() {
                        self.dark_mode = !self.dark_mode;
                        println!(
                            "Theme switched to: {}",
                            if self.dark_mode { "Dark" } else { "Light" }
                        );

                        // Update config and save to file
                        self.config.dark_mode = self.dark_mode;
                        if let Err(e) = self.config.save_to_file("Config.toml") {
                            eprintln!("Failed to save theme preference: {}", e);
                        } 

                        // If settings dialog is open, sync the temp_config
                        if let Some(temp_config) = &mut self.temp_config {
                            temp_config.dark_mode = self.dark_mode;
                        }
                    }

                    ui.add_space(8.0);

                    // Settings button (sky blue)
                    let settings_btn = egui::Button::new(
                        egui::RichText::new("⚙  Settings")
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(135, 206, 235)) // Sky blue
                    .rounding(12.0);

                    if ui.add(settings_btn).clicked() {
                        self.show_settings_dialog = true;
                        self.temp_config = Some(self.config.clone());
                    }

                    ui.add_space(8.0);

                    // About button (lavender/purple anime style)
                    let about_btn = egui::Button::new(
                        egui::RichText::new("❤  About")
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(216, 191, 216)) // Lavender/thistle
                    .rounding(12.0);

                    if ui.add(about_btn).clicked() {
                        self.show_about_dialog = true;
                    }
                });
            });
            ui.add_space(15.0);

            // ✨ Status card (anime style)
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());

                // Status title and status on same line
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("✨ Status:").size(16.0).strong().color(
                        if self.dark_mode {
                            egui::Color32::from_rgb(255, 182, 193) // Soft pink
                        } else {
                            egui::Color32::from_rgb(220, 20, 60) // Crimson - darker for contrast
                        },
                    ));

                    ui.add_space(10.0);

                    let is_paused = self.app_state.is_paused();
                    let (status_icon, status_text, status_color) = if is_paused {
                        ("⏸", "Paused", egui::Color32::from_rgb(255, 140, 0)) // Darker orange for contrast
                    } else {
                        ("▶", "Running", egui::Color32::from_rgb(34, 139, 34)) // Forest green for contrast
                    };

                    ui.label(
                        egui::RichText::new(status_icon)
                            .size(18.0)
                            .color(status_color),
                    );
                    ui.label(
                        egui::RichText::new(status_text)
                            .size(15.0)
                            .color(status_color)
                            .strong(),
                    );

                    // Add spacing and worker count on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let worker_count = self.app_state.get_actual_worker_count();
                        if worker_count > 0 {
                            ui.label(
                                egui::RichText::new(format!("⚡ {} Worker(s)", worker_count))
                                    .size(13.0)
                                    .color(if self.dark_mode {
                                        egui::Color32::from_rgb(135, 206, 235) // Sky blue
                                    } else {
                                        egui::Color32::from_rgb(70, 130, 180) // Steel blue
                                    }),
                            );
                        }
                    });
                });

                ui.add_space(15.0);

                // Button group (anime style)
                ui.horizontal(|ui| {
                    let button_width = 140.0;
                    let button_height = 32.0; // Reduced from 36.0

                    // Start/Pause button (soft colors)
                    let is_paused = self.app_state.is_paused();
                    let (button_text, button_color) = if is_paused {
                        ("▶  Start", egui::Color32::from_rgb(144, 238, 144)) // Soft green
                    } else {
                        ("⏸  Pause", egui::Color32::from_rgb(255, 218, 185)) // Soft orange
                    };

                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(button_text)
                            .size(14.0) // Slightly reduced from 15.0
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(button_color)
                    .rounding(15.0); // Large rounded corners

                    if ui
                        .add_sized([button_width, button_height], toggle_btn)
                        .clicked()
                    {
                        let was_paused = self.app_state.toggle_paused();
                        // was_paused is the OLD state, so if it was paused, it's now active
                        if was_paused {
                            // Send notification if enabled
                            println!(
                                "GUI Button: Sending activation notification (was_paused=true)"
                            );
                            if let Some(sender) = self.app_state.get_notification_sender() {
                                let _ = sender
                                    .send(NotificationEvent::Info("Sorahk activiting".to_string()));
                            }
                        } else {
                            // Send notification if enabled
                            println!("GUI Button: Sending pause notification (was_paused=false)");
                            if let Some(sender) = self.app_state.get_notification_sender() {
                                let _ = sender
                                    .send(NotificationEvent::Info("Sorahk paused".to_string()));
                            }
                        }
                    }

                    ui.add_space(15.0);

                    // Exit button (soft pink)
                    let exit_btn = egui::Button::new(
                        egui::RichText::new("❌  Exit")
                            .size(14.0) // Slightly reduced from 15.0
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink
                    .rounding(15.0); // Large rounded corners

                    if ui
                        .add_sized([button_width, button_height], exit_btn)
                        .clicked()
                    {
                        self.app_state.exit();
                        std::process::exit(0);
                    }
                });
            });

            ui.add_space(18.0);

            // 🎯 Hotkey Settings card (anime style)
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());

                // Decorative title
                ui.label(
                    egui::RichText::new("🎯 Hotkey Settings")
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(173, 216, 230) // Soft blue
                        } else {
                            egui::Color32::from_rgb(30, 90, 180) // Darker blue for better contrast
                        }),
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Toggle Key:").size(14.0).color(
                        if self.dark_mode {
                            egui::Color32::from_rgb(200, 200, 200)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40) // Much darker for contrast
                        },
                    ));
                    ui.label(
                        egui::RichText::new(&self.config.switch_key)
                            .size(15.0)
                            .color(if self.dark_mode {
                                egui::Color32::from_rgb(135, 206, 235)
                            } else {
                                egui::Color32::from_rgb(0, 100, 200) // Darker sky blue for contrast
                            })
                            .strong(),
                    );
                });
            });

            ui.add_space(15.0);

            // ⚙ Global Configuration card (anime style)
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());

                // Decorative title
                ui.label(
                    egui::RichText::new("⚙ Global Configuration")
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(255, 218, 185) // Soft orange
                        } else {
                            egui::Color32::from_rgb(200, 100, 0) // Darker orange for contrast
                        }),
                );

                ui.add_space(8.0);

                let available = ui.available_width();
                egui::Grid::new("config_grid")
                    .num_columns(2)
                    .spacing([30.0, 8.0])
                    .min_col_width(available * 0.4)
                    .striped(false)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Input Timeout:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(format!("{} ms", self.config.input_timeout))
                                .size(14.0)
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(135, 206, 235)
                                } else {
                                    egui::Color32::from_rgb(0, 100, 200)
                                }),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Default Interval:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(format!("{} ms", self.config.interval))
                                .size(14.0)
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(135, 206, 235)
                                } else {
                                    egui::Color32::from_rgb(0, 100, 200)
                                }),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Default Duration:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(format!("{} ms", self.config.event_duration))
                                .size(14.0)
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(135, 206, 235)
                                } else {
                                    egui::Color32::from_rgb(0, 100, 200)
                                }),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Show Tray Icon:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(if self.config.show_tray_icon {
                                "Yes"
                            } else {
                                "No"
                            })
                            .size(14.0)
                            .color(if self.config.show_tray_icon {
                                if self.dark_mode {
                                    egui::Color32::from_rgb(144, 238, 144)
                                } else {
                                    egui::Color32::from_rgb(34, 139, 34)
                                }
                            } else if self.dark_mode {
                                egui::Color32::from_rgb(255, 182, 193)
                            } else {
                                egui::Color32::from_rgb(220, 20, 60)
                            }),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Show Notifications:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(if self.config.show_notifications {
                                "Yes"
                            } else {
                                "No"
                            })
                            .size(14.0)
                            .color(
                                if self.config.show_notifications {
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(144, 238, 144)
                                    } else {
                                        egui::Color32::from_rgb(34, 139, 34)
                                    }
                                } else if self.dark_mode {
                                    egui::Color32::from_rgb(255, 182, 193)
                                } else {
                                    egui::Color32::from_rgb(220, 20, 60)
                                },
                            ),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Always On Top:").size(14.0).color(
                            if self.dark_mode {
                                egui::Color32::from_rgb(200, 200, 200)
                            } else {
                                egui::Color32::from_rgb(40, 40, 40)
                            },
                        ));
                        ui.label(
                            egui::RichText::new(if self.config.always_on_top {
                                "Yes"
                            } else {
                                "No"
                            })
                            .size(14.0)
                            .color(if self.config.always_on_top {
                                if self.dark_mode {
                                    egui::Color32::from_rgb(144, 238, 144)
                                } else {
                                    egui::Color32::from_rgb(34, 139, 34)
                                }
                            } else if self.dark_mode {
                                egui::Color32::from_rgb(255, 182, 193)
                            } else {
                                egui::Color32::from_rgb(220, 20, 60)
                            }),
                        );
                        ui.end_row();
                    });
            });

            ui.add_space(18.0);

            // 🔄 Key Mappings card (anime style)
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());

                // Decorative title
                ui.label(
                    egui::RichText::new("🔄 Key Mappings")
                        .size(16.0)
                        .strong()
                        .color(if self.dark_mode {
                            egui::Color32::from_rgb(152, 251, 152) // Soft green
                        } else {
                            egui::Color32::from_rgb(0, 120, 0) // Darker forest green for contrast
                        }),
                );
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let available = ui.available_width();
                        egui::Grid::new("mappings_grid")
                            .num_columns(4)
                            .spacing([20.0, 6.0])
                            .min_col_width(available * 0.2)
                            .striped(true)
                            .show(ui, |ui| {
                                // Header
                                ui.label(egui::RichText::new("Trigger").strong().color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    } else {
                                        egui::Color32::from_rgb(40, 40, 40)
                                    },
                                ));
                                ui.label(egui::RichText::new("Target").strong().color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    } else {
                                        egui::Color32::from_rgb(40, 40, 40)
                                    },
                                ));
                                ui.label(egui::RichText::new("Interval(ms)").strong().color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    } else {
                                        egui::Color32::from_rgb(40, 40, 40)
                                    },
                                ));
                                ui.label(egui::RichText::new("Duration(ms)").strong().color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    } else {
                                        egui::Color32::from_rgb(40, 40, 40)
                                    },
                                ));
                                ui.end_row();

                                // Mappings
                                for mapping in &self.config.mappings {
                                    ui.label(egui::RichText::new(&mapping.trigger_key).color(
                                        if self.dark_mode {
                                            egui::Color32::from_rgb(255, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(180, 80, 0) // Darker for contrast
                                        },
                                    ));
                                    ui.label(egui::RichText::new(&mapping.target_key).color(
                                        if self.dark_mode {
                                            egui::Color32::from_rgb(100, 200, 255)
                                        } else {
                                            egui::Color32::from_rgb(0, 80, 180) // Darker for contrast
                                        },
                                    ));
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}",
                                            mapping.interval.unwrap_or(self.config.interval)
                                        ))
                                        .color(
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(200, 200, 200)
                                            } else {
                                                egui::Color32::from_rgb(60, 60, 60)
                                            },
                                        ),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}",
                                            mapping
                                                .event_duration
                                                .unwrap_or(self.config.event_duration)
                                        ))
                                        .color(
                                            if self.dark_mode {
                                                egui::Color32::from_rgb(200, 200, 200)
                                            } else {
                                                egui::Color32::from_rgb(60, 60, 60)
                                            },
                                        ),
                                    );
                                    ui.end_row();
                                }
                            });
                    });
            });
        });

        // Check if should exit
        if self.app_state.should_exit() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_state.exit();
    }
}

impl SorahkGui {
    fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_cancel = false;

        // Handle key capture if in capture mode
        if self.key_capture_mode != KeyCaptureMode::None {
            ctx.input(|i| {
                for key in i.keys_down.iter() {
                    if let Some(key_name) = Self::key_to_string(*key) {
                        // Capture the key and update the appropriate field
                        if let Some(temp_config) = &mut self.temp_config {
                            match self.key_capture_mode {
                                KeyCaptureMode::ToggleKey => {
                                    temp_config.switch_key = key_name.clone();
                                }
                                KeyCaptureMode::MappingTrigger(idx) => {
                                    if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                        mapping.trigger_key = key_name.clone();
                                    }
                                }
                                KeyCaptureMode::MappingTarget(idx) => {
                                    if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                        mapping.target_key = key_name.clone();
                                    }
                                }
                                KeyCaptureMode::NewMappingTrigger => {
                                    self.new_mapping_trigger = key_name.clone();
                                }
                                KeyCaptureMode::NewMappingTarget => {
                                    self.new_mapping_target = key_name.clone();
                                }
                                KeyCaptureMode::None => {
                                    //
                                }
                            }
                        }
                        // Exit capture mode
                        self.key_capture_mode = KeyCaptureMode::None;
                        break;
                    }
                }
            });
        }

        egui::Window::new("")
            .title_bar(false) // Remove default title bar
            .collapsible(false)
            .resizable(false)
            .fixed_size([680.0, 580.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .id(egui::Id::new("settings_dialog_window")) // Unique ID to avoid conflicts
            .show(ctx, |ui| {
                // Push a unique ID scope for this entire settings dialog
                ui.push_id("settings_dialog_scope", |ui| {
                    // Custom title bar (matching main window style)
                    ui.horizontal(|ui| {
                        ui.add_space(15.0);

                        // Settings title
                        ui.label(
                            egui::RichText::new("⚙ Settings ~ Configuration Panel ~")
                                .size(18.0)
                                .strong()
                                .color(if self.dark_mode {
                                    egui::Color32::from_rgb(176, 224, 230) // Sky blue
                                } else {
                                    egui::Color32::from_rgb(135, 206, 235) // Deeper sky blue
                                }),
                        );

                        // Push close button to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0);

                            // Close button (matching style)
                            let close_btn =
                                egui::Button::new(egui::RichText::new("x").size(16.0).color(
                                    if self.dark_mode {
                                        egui::Color32::from_rgb(255, 182, 193)
                                    } else {
                                        egui::Color32::from_rgb(220, 20, 60)
                                    },
                                ))
                                .rounding(12.0)
                                .frame(false);

                            if ui.add(close_btn).clicked() {
                                should_cancel = true;
                            }
                        });
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(10.0);
                    let temp_config = self.temp_config.as_mut().unwrap();
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            // Toggle Key Section
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new("⌨ Toggle Key")
                                        .size(16.0)
                                        .strong()
                                        .color(if self.dark_mode {
                                            egui::Color32::from_rgb(100, 200, 255)
                                        } else {
                                            egui::Color32::from_rgb(20, 100, 200)
                                        }),
                                );
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    ui.label("Key:");
                                    ui.add_space(5.0);

                                    let is_capturing =
                                        self.key_capture_mode == KeyCaptureMode::ToggleKey;
                                    let button_text = if is_capturing {
                                        "⌨ Press any key..."
                                    } else if temp_config.switch_key.is_empty() {
                                        "Click to set key"
                                    } else {
                                        &temp_config.switch_key
                                    };

                                    let button = egui::Button::new(
                                        egui::RichText::new(button_text).color(if is_capturing {
                                            egui::Color32::from_rgb(255, 200, 0)
                                        } else if self.dark_mode {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::from_rgb(40, 40, 40)
                                        }),
                                    )
                                    .fill(if is_capturing {
                                        egui::Color32::from_rgb(70, 130, 180)
                                    } else if self.dark_mode {
                                        egui::Color32::from_rgb(60, 60, 60)
                                    } else {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    })
                                    .rounding(10.0); // Increased rounding to match buttons

                                    if ui.add_sized([180.0, 28.0], button).clicked() {
                                        self.key_capture_mode = KeyCaptureMode::ToggleKey;
                                    }
                                });
                            });

                            ui.add_space(8.0);

                            // Global Configuration Section
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new("⚙ Global Configuration")
                                        .size(16.0)
                                        .strong()
                                        .color(if self.dark_mode {
                                            egui::Color32::from_rgb(255, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(200, 100, 0)
                                        }),
                                );
                                ui.add_space(6.0);

                                let available = ui.available_width();
                                egui::Grid::new("config_edit_grid")
                                    .num_columns(2)
                                    .spacing([20.0, 8.0])
                                    .min_col_width(available * 0.35)
                                    .show(ui, |ui| {
                                        ui.label("Input Timeout (ms):");
                                        let mut timeout_str = temp_config.input_timeout.to_string();
                                        ui.add_sized(
                                            [120.0, 24.0],
                                            egui::TextEdit::singleline(&mut timeout_str)
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                }),
                                        );
                                        if let Ok(val) = timeout_str.parse::<u64>() {
                                            temp_config.input_timeout = val;
                                        }
                                        ui.end_row();

                                        ui.label("Default Interval (ms):");
                                        let mut interval_str = temp_config.interval.to_string();
                                        ui.add_sized(
                                            [120.0, 24.0],
                                            egui::TextEdit::singleline(&mut interval_str)
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                }),
                                        );
                                        if let Ok(val) = interval_str.parse::<u64>() {
                                            temp_config.interval = val.max(5);
                                        }
                                        ui.end_row();

                                        ui.label("Default Duration (ms):");
                                        let mut duration_str =
                                            temp_config.event_duration.to_string();
                                        ui.add_sized(
                                            [120.0, 24.0],
                                            egui::TextEdit::singleline(&mut duration_str)
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                }),
                                        );
                                        if let Ok(val) = duration_str.parse::<u64>() {
                                            temp_config.event_duration = val.max(5);
                                        }
                                        ui.end_row();

                                        ui.label("Worker Count:");
                                        let mut worker_str = temp_config.worker_count.to_string();
                                        ui.add_sized(
                                            [120.0, 24.0],
                                            egui::TextEdit::singleline(&mut worker_str)
                                                .hint_text("0 = auto")
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                }),
                                        );
                                        if let Ok(val) = worker_str.parse::<usize>() {
                                            temp_config.worker_count = val;
                                        }
                                        ui.end_row();

                                        ui.label("Show Tray Icon:");
                                        ui.checkbox(&mut temp_config.show_tray_icon, "");
                                        ui.end_row();

                                        ui.label("Show Notifications:");
                                        ui.checkbox(&mut temp_config.show_notifications, "");
                                        ui.end_row();

                                        ui.label("Always On Top:");
                                        ui.checkbox(&mut temp_config.always_on_top, "");
                                        ui.end_row();

                                        ui.label("Dark Mode:");
                                        ui.checkbox(&mut temp_config.dark_mode, "");
                                        ui.end_row();
                                    });
                            });

                            ui.add_space(8.0);

                            // Key Mappings Section
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new("🔄 Key Mappings")
                                        .size(16.0)
                                        .strong()
                                        .color(if self.dark_mode {
                                            egui::Color32::from_rgb(150, 255, 150)
                                        } else {
                                            egui::Color32::from_rgb(0, 150, 0)
                                        }),
                                );
                                ui.add_space(6.0);

                                // Existing mappings
                                let mut to_remove = None;
                                for (idx, mapping) in temp_config.mappings.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", idx + 1));

                                        ui.label("Trigger:");
                                        let is_capturing_trigger = self.key_capture_mode
                                            == KeyCaptureMode::MappingTrigger(idx);
                                        let trigger_text = if is_capturing_trigger {
                                            "⌨..."
                                        } else {
                                            &mapping.trigger_key
                                        };
                                        let trigger_btn = egui::Button::new(
                                            egui::RichText::new(trigger_text).color(
                                                if is_capturing_trigger {
                                                    egui::Color32::from_rgb(255, 200, 0)
                                                } else if self.dark_mode {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::from_rgb(40, 40, 40)
                                                },
                                            ),
                                        )
                                        .fill(if is_capturing_trigger {
                                            egui::Color32::from_rgb(70, 130, 180)
                                        } else if self.dark_mode {
                                            egui::Color32::from_rgb(50, 50, 50)
                                        } else {
                                            egui::Color32::from_rgb(220, 220, 220)
                                        })
                                        .rounding(4.0);
                                        if ui.add_sized([80.0, 24.0], trigger_btn).clicked() {
                                            self.key_capture_mode =
                                                KeyCaptureMode::MappingTrigger(idx);
                                        }

                                        ui.label("Target:");
                                        let is_capturing_target = self.key_capture_mode
                                            == KeyCaptureMode::MappingTarget(idx);
                                        let target_text = if is_capturing_target {
                                            "⌨..."
                                        } else {
                                            &mapping.target_key
                                        };
                                        let target_btn = egui::Button::new(
                                            egui::RichText::new(target_text).color(
                                                if is_capturing_target {
                                                    egui::Color32::from_rgb(255, 200, 0)
                                                } else if self.dark_mode {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::from_rgb(40, 40, 40)
                                                },
                                            ),
                                        )
                                        .fill(if is_capturing_target {
                                            egui::Color32::from_rgb(70, 130, 180)
                                        } else if self.dark_mode {
                                            egui::Color32::from_rgb(50, 50, 50)
                                        } else {
                                            egui::Color32::from_rgb(220, 220, 220)
                                        })
                                        .rounding(4.0);
                                        if ui.add_sized([80.0, 24.0], target_btn).clicked() {
                                            self.key_capture_mode =
                                                KeyCaptureMode::MappingTarget(idx);
                                        }

                                        ui.label("Int:");
                                        let mut interval_str = mapping
                                            .interval
                                            .unwrap_or(temp_config.interval)
                                            .to_string();

                                        let interval_edit =
                                            egui::TextEdit::singleline(&mut interval_str)
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .desired_width(45.0) // Shorter width
                                                .font(egui::TextStyle::Button); // Match button style

                                        if ui.add_sized([45.0, 24.0], interval_edit).changed()
                                            && let Ok(val) = interval_str.parse::<u64>()
                                        {
                                            mapping.interval = Some(val.max(5));
                                        }

                                        ui.label("Dur:");
                                        let mut duration_str = mapping
                                            .event_duration
                                            .unwrap_or(temp_config.event_duration)
                                            .to_string();

                                        let duration_edit =
                                            egui::TextEdit::singleline(&mut duration_str)
                                                .background_color(if self.dark_mode {
                                                    egui::Color32::from_rgb(50, 50, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220)
                                                })
                                                .desired_width(45.0) // Shorter width
                                                .font(egui::TextStyle::Button); // Match button style

                                        if ui.add_sized([45.0, 24.0], duration_edit).changed()
                                            && let Ok(val) = duration_str.parse::<u64>()
                                        {
                                            mapping.event_duration = Some(val.max(5));
                                        }

                                        let delete_btn = egui::Button::new(
                                            egui::RichText::new("🗑").color(egui::Color32::WHITE),
                                        )
                                        .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink (anime style)
                                        .rounding(10.0); // Larger rounding for anime style

                                        if ui.add_sized([32.0, 24.0], delete_btn).clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                    ui.add_space(4.0);
                                }

                                if let Some(idx) = to_remove {
                                    temp_config.mappings.remove(idx);
                                }

                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                // Add new mapping
                                ui.label(
                                    egui::RichText::new("➕ Add New Mapping")
                                        .size(14.0)
                                        .strong(),
                                );
                                ui.add_space(5.0);

                                ui.horizontal(|ui| {
                                    ui.label("Trigger:");
                                    let is_capturing_new_trigger =
                                        self.key_capture_mode == KeyCaptureMode::NewMappingTrigger;
                                    let new_trigger_text = if is_capturing_new_trigger {
                                        "⌨ Press..."
                                    } else if self.new_mapping_trigger.is_empty() {
                                        "Click"
                                    } else {
                                        &self.new_mapping_trigger
                                    };
                                    let new_trigger_btn = egui::Button::new(
                                        egui::RichText::new(new_trigger_text).color(
                                            if is_capturing_new_trigger {
                                                egui::Color32::from_rgb(255, 200, 0)
                                            } else if self.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::from_rgb(40, 40, 40)
                                            },
                                        ),
                                    )
                                    .fill(if is_capturing_new_trigger {
                                        egui::Color32::from_rgb(70, 130, 180)
                                    } else if self.dark_mode {
                                        egui::Color32::from_rgb(50, 50, 50)
                                    } else {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    })
                                    .rounding(4.0);
                                    if ui.add_sized([80.0, 24.0], new_trigger_btn).clicked() {
                                        self.key_capture_mode = KeyCaptureMode::NewMappingTrigger;
                                    }

                                    ui.label("Target:");
                                    let is_capturing_new_target =
                                        self.key_capture_mode == KeyCaptureMode::NewMappingTarget;
                                    let new_target_text = if is_capturing_new_target {
                                        "⌨ Press..."
                                    } else if self.new_mapping_target.is_empty() {
                                        "Click"
                                    } else {
                                        &self.new_mapping_target
                                    };
                                    let new_target_btn = egui::Button::new(
                                        egui::RichText::new(new_target_text).color(
                                            if is_capturing_new_target {
                                                egui::Color32::from_rgb(255, 200, 0)
                                            } else if self.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::from_rgb(40, 40, 40)
                                            },
                                        ),
                                    )
                                    .fill(if is_capturing_new_target {
                                        egui::Color32::from_rgb(70, 130, 180)
                                    } else if self.dark_mode {
                                        egui::Color32::from_rgb(50, 50, 50)
                                    } else {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    })
                                    .rounding(4.0);
                                    if ui.add_sized([80.0, 24.0], new_target_btn).clicked() {
                                        self.key_capture_mode = KeyCaptureMode::NewMappingTarget;
                                    }

                                    ui.label("Int:");
                                    let interval_edit =
                                        egui::TextEdit::singleline(&mut self.new_mapping_interval)
                                            .background_color(if self.dark_mode {
                                                egui::Color32::from_rgb(50, 50, 50)
                                            } else {
                                                egui::Color32::from_rgb(220, 220, 220)
                                            })
                                            .hint_text("5")
                                            .desired_width(45.0)
                                            .font(egui::TextStyle::Button);
                                    ui.add_sized([45.0, 24.0], interval_edit);

                                    ui.label("Dur:");
                                    let duration_edit =
                                        egui::TextEdit::singleline(&mut self.new_mapping_duration)
                                            .background_color(if self.dark_mode {
                                                egui::Color32::from_rgb(50, 50, 50)
                                            } else {
                                                egui::Color32::from_rgb(220, 220, 220)
                                            })
                                            .hint_text("5")
                                            .desired_width(45.0)
                                            .font(egui::TextStyle::Button);
                                    ui.add_sized([45.0, 24.0], duration_edit);

                                    let add_btn = egui::Button::new(
                                        egui::RichText::new("➕ Add")
                                            .color(egui::Color32::WHITE)
                                            .strong(),
                                    )
                                    .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green (anime style)
                                    .rounding(10.0); // Larger rounding for anime style

                                    if ui.add_sized([70.0, 24.0], add_btn).clicked()
                                        && !self.new_mapping_trigger.is_empty()
                                        && !self.new_mapping_target.is_empty()
                                    {
                                        let interval = self
                                            .new_mapping_interval
                                            .parse::<u64>()
                                            .ok()
                                            .map(|v| v.max(5));
                                        let duration = self
                                            .new_mapping_duration
                                            .parse::<u64>()
                                            .ok()
                                            .map(|v| v.max(5));

                                        temp_config.mappings.push(KeyMapping {
                                            trigger_key: self.new_mapping_trigger.to_uppercase(),
                                            target_key: self.new_mapping_target.to_uppercase(),
                                            interval,
                                            event_duration: duration,
                                        });

                                        // Clear input fields
                                        self.new_mapping_trigger.clear();
                                        self.new_mapping_target.clear();
                                        self.new_mapping_interval.clear();
                                        self.new_mapping_duration.clear();
                                    }
                                });
                            });

                            ui.add_space(8.0);

                            // 🎯 Process Whitelist Section (anime style)
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new(
                                        "🎯 Process Whitelist (Empty = All Enabled)",
                                    )
                                    .size(16.0)
                                    .strong()
                                    .color(if self.dark_mode {
                                        egui::Color32::from_rgb(186, 149, 230) // Soft purple
                                    } else {
                                        egui::Color32::from_rgb(100, 50, 150) // Darker purple for contrast
                                    }),
                                );
                                ui.add_space(6.0);

                                // Process list
                                egui::ScrollArea::vertical()
                                    .max_height(80.0)
                                    .show(ui, |ui| {
                                        let mut to_remove: Option<usize> = None;
                                        for (idx, process) in
                                            temp_config.process_whitelist.iter().enumerate()
                                        {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(process).size(13.0).color(
                                                        if self.dark_mode {
                                                            egui::Color32::from_rgb(200, 200, 255)
                                                        } else {
                                                            egui::Color32::from_rgb(60, 60, 120)
                                                        },
                                                    ),
                                                );

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        let del_btn = egui::Button::new(
                                                            egui::RichText::new("🗑")
                                                                .color(egui::Color32::WHITE)
                                                                .size(11.0),
                                                        )
                                                        .fill(egui::Color32::from_rgb(
                                                            255, 182, 193,
                                                        )) // Soft pink
                                                        .rounding(8.0);

                                                        if ui
                                                            .add_sized([24.0, 20.0], del_btn)
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
                                    });

                                ui.add_space(6.0);

                                // Add new process
                                ui.horizontal(|ui| {
                                    let process_edit =
                                        egui::TextEdit::singleline(&mut self.new_process_name)
                                            .background_color(if self.dark_mode {
                                                egui::Color32::from_rgb(50, 50, 50)
                                            } else {
                                                egui::Color32::from_rgb(220, 220, 220)
                                            })
                                            .hint_text("e.g., notepad.exe")
                                            .desired_width(200.0);
                                    ui.add(process_edit);

                                    let add_btn = egui::Button::new(
                                        egui::RichText::new("➕ Add")
                                            .color(egui::Color32::WHITE)
                                            .size(12.0)
                                            .strong(),
                                    )
                                    .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green
                                    .rounding(10.0);

                                    if ui.add_sized([70.0, 24.0], add_btn).clicked() {
                                        let process_name = self.new_process_name.trim();
                                        if !process_name.is_empty()
                                            && !temp_config
                                                .process_whitelist
                                                .contains(&process_name.to_string())
                                        {
                                            temp_config
                                                .process_whitelist
                                                .push(process_name.to_string());
                                            self.new_process_name.clear();
                                        }
                                    }

                                    ui.add_space(8.0);

                                    // Browse button for selecting process
                                    let browse_btn = egui::Button::new(
                                        egui::RichText::new("📁 Browse")
                                            .color(egui::Color32::WHITE)
                                            .size(12.0)
                                            .strong(),
                                    )
                                    .fill(egui::Color32::from_rgb(135, 206, 235)) // Sky blue
                                    .rounding(10.0);

                                    if ui.add_sized([85.0, 24.0], browse_btn).clicked() {
                                        // Open file dialog to select executable
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Executable", &["exe"])
                                            .set_title("Select Process")
                                            .pick_file()
                                            && let Some(filename) = path.file_name()
                                        {
                                            let process_name =
                                                filename.to_string_lossy().to_string();
                                            if !temp_config
                                                .process_whitelist
                                                .contains(&process_name)
                                            {
                                                temp_config.process_whitelist.push(process_name);
                                            }
                                        }
                                    }
                                });
                            });
                        });
                }); // End of ScrollArea

                ui.separator();
                ui.add_space(6.0);

                // Action buttons - centered (outside ScrollArea, fixed at bottom)
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        // Calculate total width of buttons and spacing
                        let button_width = 240.0;
                        let spacing = 15.0;
                        let total_buttons_width = button_width * 2.0 + spacing;
                        let available_width = ui.available_width();

                        // Add left padding to center the buttons
                        if available_width > total_buttons_width {
                            ui.add_space((available_width - total_buttons_width) / 2.0);
                        }

                        let save_btn = egui::Button::new(
                            egui::RichText::new("💾  Save Changes")
                                .size(14.0) // Slightly smaller for consistency
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(144, 238, 144)) // Soft green (anime style)
                        .rounding(15.0); // Larger rounding for anime style

                        if ui.add_sized([button_width, 32.0], save_btn).clicked() {
                            should_save = true;
                        }

                        ui.add_space(spacing);

                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("❌  Cancel")
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(255, 182, 193)) // Soft pink (anime style)
                        .rounding(15.0); // Larger rounding for anime style

                        if ui.add_sized([button_width, 32.0], cancel_btn).clicked() {
                            should_cancel = true;
                        }
                    });
                });

                ui.add_space(6.0);

                // Hint
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("* Changes will take effect immediately after saving")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(100, 220, 100))
                            .italics(),
                    );
                });
            });

        // Handle save/cancel outside the window closure to avoid borrow conflicts
        if should_save {
            if let Some(temp_config) = &self.temp_config {
                // Check if always_on_top changed
                let always_on_top_changed = temp_config.always_on_top != self.config.always_on_top;
                // Check if dark_mode changed
                let dark_mode_changed = temp_config.dark_mode != self.config.dark_mode;

                // Save to file
                if let Err(e) = temp_config.save_to_file("Config.toml") {
                    eprintln!("Failed to save config: {}", e);
                } else {
                    // Reload configuration into AppState (takes effect immediately)
                    if let Err(e) = self.app_state.reload_config(temp_config.clone()) {
                        eprintln!("Failed to reload config into AppState: {}", e);
                    } 

                    // Update GUI's config
                    self.config = temp_config.clone();

                    // Apply theme change immediately
                    if dark_mode_changed {
                        self.dark_mode = self.config.dark_mode;
                        println!(
                            "Theme changed to: {}",
                            if self.dark_mode { "dark" } else { "light" }
                        );
                    }

                    // Apply always_on_top change immediately
                    if always_on_top_changed {
                        if self.config.always_on_top {
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::AlwaysOnTop,
                            ));
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::Normal,
                            ));
                        }
                    }
                }
            }
            self.show_settings_dialog = false;
            self.temp_config = None;
            self.key_capture_mode = KeyCaptureMode::None;
        }

        if should_cancel {
            self.show_settings_dialog = false;
            self.temp_config = None;
            self.key_capture_mode = KeyCaptureMode::None;
            // Clear input fields
            self.new_mapping_trigger.clear();
            self.new_mapping_target.clear();
            self.new_mapping_interval.clear();
            self.new_mapping_duration.clear();
        }
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
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
        ) = if self.dark_mode {
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

        egui::Window::new("about_sorahk")
            .id(egui::Id::new("about_dialog_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 550.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
                        egui::Frame::none()
                            .fill(card_bg)
                            .rounding(15.0)
                            .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                            .show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new("✨ Version 0.1.2")
                                            .size(18.0)
                                            .strong()
                                            .color(version_color),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(
                                            "A lightweight, efficient auto key press tool",
                                        )
                                        .size(13.0)
                                        .color(text_secondary),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "with beautiful anime-inspired interface",
                                        )
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
                                    egui::RichText::new("👤 Author:")
                                        .size(14.0)
                                        .strong()
                                        .color(label_color),
                                );
                                ui.label(egui::RichText::new("llnut").size(14.0).color(text_color));
                                ui.end_row();

                                // GitHub
                                ui.label(
                                    egui::RichText::new("🔗 GitHub:")
                                        .size(14.0)
                                        .strong()
                                        .color(label_color),
                                );
                                ui.label(
                                    egui::RichText::new("github.com/llnut/sorahk")
                                        .size(14.0)
                                        .color(text_color),
                                );
                                ui.end_row();

                                // License
                                ui.label(
                                    egui::RichText::new("📜 License:")
                                        .size(14.0)
                                        .strong()
                                        .color(label_color),
                                );
                                ui.label(
                                    egui::RichText::new("MIT License")
                                        .size(14.0)
                                        .color(text_color),
                                );
                                ui.end_row();

                                // Built with
                                ui.label(
                                    egui::RichText::new("⚙ Built with:")
                                        .size(14.0)
                                        .strong()
                                        .color(label_color),
                                );
                                ui.label(
                                    egui::RichText::new("Rust + egui")
                                        .size(14.0)
                                        .color(text_color),
                                );
                                ui.end_row();
                            });
                    });
                    ui.add_space(30.0);

                    // Inspired note
                    ui.label(
                        egui::RichText::new("💫 Inspired by Kasugano Sora")
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
                                egui::RichText::new("✨ Close")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(216, 191, 216))
                            .rounding(15.0),
                        )
                        .clicked()
                    {
                        self.show_about_dialog = false;
                    }
                    ui.add_space(20.0);
                });
            });
    }
}
