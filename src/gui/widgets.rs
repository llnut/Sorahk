//! Themed egui widget factories and dimension constants.

#![allow(dead_code)]

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, epaint::Shadow};
use crate::gui::theme;

/// Semantic text sizes shared across the GUI.
pub mod text_size {
    pub const SMALL: f32 = 11.0;
    pub const COMPACT: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const NORMAL: f32 = 14.0;
    pub const SUBTITLE: f32 = 16.0;
    pub const SECTION: f32 = 18.0;
    pub const TITLE: f32 = 20.0;
    pub const HERO: f32 = 28.0;
}

pub mod spacing {
    pub const TIGHT: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const NORMAL: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const SECTION: f32 = 20.0;
}

pub mod radius {
    pub const PILL: u8 = 8;
    pub const BUTTON: u8 = 12;
    pub const CARD: u8 = 16;
    pub const DIALOG: u8 = 20;
}

/// Selects fill from the active theme's accent palette.
#[derive(Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
    Pink,
    Neutral,
}

/// Builds a styled button with white text on themed fill.
#[inline]
pub fn themed_button(text: &str, kind: ButtonKind, dark_mode: bool) -> egui::Button<'_> {
    let c = theme::colors(dark_mode);
    let fill = match kind {
        ButtonKind::Primary => c.accent_primary,
        ButtonKind::Secondary => c.accent_secondary,
        ButtonKind::Danger => c.accent_danger,
        ButtonKind::Success => c.accent_success,
        ButtonKind::Warning => c.accent_warning,
        ButtonKind::Pink => c.accent_pink,
        ButtonKind::Neutral => c.bg_card_hover,
    };
    egui::Button::new(
        RichText::new(text.to_string())
            .size(text_size::BODY)
            .color(c.fg_inverse),
    )
    .fill(fill)
    .corner_radius(radius::BUTTON)
}

/// Theme toggle button. Gold-warmth in dark mode, lilac in light mode.
#[inline]
pub fn theme_toggle_button(text: &str, dark_mode: bool) -> egui::Button<'_> {
    let c = theme::colors(dark_mode);
    let fill = if dark_mode { c.accent_warning } else { c.accent_secondary };
    egui::Button::new(
        RichText::new(text.to_string())
            .size(text_size::BODY)
            .color(c.fg_inverse),
    )
    .fill(fill)
    .corner_radius(radius::BUTTON)
}

/// Card frame with rounded fill and no stroke.
#[inline]
pub fn card_frame(dark_mode: bool) -> Frame {
    let c = theme::colors(dark_mode);
    Frame::NONE
        .fill(c.bg_card)
        .corner_radius(CornerRadius::same(radius::CARD))
        .inner_margin(Margin::same(spacing::LARGE as i8))
}

/// Modal dialog frame: card frame plus window shadow.
#[inline]
pub fn dialog_frame(dark_mode: bool) -> Frame {
    card_frame(dark_mode)
        .corner_radius(CornerRadius::same(radius::DIALOG))
        .shadow(Shadow {
            offset: [0, 4],
            blur: 18,
            spread: 0,
            color: theme::overlay::SHADOW_LIGHT,
        })
}

/// Pill frame for sequence and target tags.
#[inline]
pub fn pill_frame(fill: Color32) -> Frame {
    Frame::NONE
        .fill(fill)
        .corner_radius(CornerRadius::same(radius::PILL))
        .inner_margin(Margin::symmetric(8, 4))
}

/// Maps a key string to its themed pill background color.
#[inline]
pub fn pill_color(key: &str, dark_mode: bool) -> Color32 {
    let c = theme::colors(dark_mode);
    let bytes = key.as_bytes();
    let starts_with_ci = |p: &[u8]| -> bool {
        bytes.len() >= p.len() && bytes[..p.len()].eq_ignore_ascii_case(p)
    };

    if starts_with_ci(b"MOUSE_") {
        c.pill_mouse_movement
    } else if key.eq_ignore_ascii_case("LBUTTON")
        || key.eq_ignore_ascii_case("RBUTTON")
        || key.eq_ignore_ascii_case("MBUTTON")
        || starts_with_ci(b"XBUTTON")
    {
        c.pill_mouse_button
    } else if starts_with_ci(b"GAMEPAD_") || starts_with_ci(b"JOYSTICK_") {
        c.pill_gamepad
    } else {
        c.pill_keyboard
    }
}

/// Maps a key string to a 1-char display icon and canonicalized label.
pub fn pill_icon_and_label(key: &str) -> (&'static str, String) {
    let upper = key.to_uppercase();
    match upper.as_str() {
        "MOUSE_UP" => ("↑", "MOUSE_UP".to_string()),
        "MOUSE_DOWN" => ("↓", "MOUSE_DOWN".to_string()),
        "MOUSE_LEFT" => ("←", "MOUSE_LEFT".to_string()),
        "MOUSE_RIGHT" => ("→", "MOUSE_RIGHT".to_string()),
        "MOUSE_UP_LEFT" | "MOUSE_UPLEFT" => ("↖", "MOUSE_UP_LEFT".to_string()),
        "MOUSE_UP_RIGHT" | "MOUSE_UPRIGHT" => ("↗", "MOUSE_UP_RIGHT".to_string()),
        "MOUSE_DOWN_LEFT" | "MOUSE_DOWNLEFT" => ("↙", "MOUSE_DOWN_LEFT".to_string()),
        "MOUSE_DOWN_RIGHT" | "MOUSE_DOWNRIGHT" => ("↘", "MOUSE_DOWN_RIGHT".to_string()),
        "LBUTTON" => ("🖱", "LBUTTON".to_string()),
        "RBUTTON" => ("🖱", "RBUTTON".to_string()),
        "MBUTTON" => ("🖱", "MBUTTON".to_string()),
        "XBUTTON1" => ("🖱", "XBUTTON1".to_string()),
        "XBUTTON2" => ("🖱", "XBUTTON2".to_string()),
        _ if upper.starts_with("GAMEPAD_") => ("🎮", key.to_string()),
        _ if upper.starts_with("JOYSTICK_") => ("🕹", key.to_string()),
        _ if upper.starts_with("HID_") => ("🎛", key.to_string()),
        _ => ("⌨", key.to_string()),
    }
}

/// Pill width estimator for read-only display in the mappings card.
#[inline]
pub fn estimate_pill_width_display(key: &str) -> f32 {
    let label_len = pill_icon_and_label(key).1.chars().count().min(20);
    16.0 + 8.0 + (label_len as f32) * 6.5
}

/// Pill width estimator for the editor row, accounting for index and delete button.
#[inline]
pub fn estimate_pill_width_editor(key: &str) -> f32 {
    let label_len = pill_icon_and_label(key).1.chars().count().min(20);
    53.0 + (label_len as f32) * 6.5
}

#[inline]
pub const fn arrow_separator_width() -> f32 {
    18.0
}

/// Renders a bold themed section heading with vertical padding.
pub fn section_header(ui: &mut egui::Ui, text: &str, dark_mode: bool) {
    let c = theme::colors(dark_mode);
    ui.add_space(spacing::SMALL);
    ui.label(
        RichText::new(text.to_string())
            .size(text_size::SECTION)
            .strong()
            .color(c.title_primary),
    );
    ui.add_space(spacing::SMALL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_color_classifies_mouse_movement() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("MOUSE_UP", true), dark.pill_mouse_movement);
        assert_eq!(pill_color("MOUSE_DOWN_LEFT", true), dark.pill_mouse_movement);
        assert_eq!(pill_color("mouse_left", true), dark.pill_mouse_movement);
    }

    #[test]
    fn pill_color_classifies_mouse_buttons() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("LBUTTON", true), dark.pill_mouse_button);
        assert_eq!(pill_color("XBUTTON1", true), dark.pill_mouse_button);
    }

    #[test]
    fn pill_color_classifies_gamepad() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("GAMEPAD_045E_A", true), dark.pill_gamepad);
        assert_eq!(pill_color("JOYSTICK_FOO", true), dark.pill_gamepad);
    }

    #[test]
    fn pill_color_keyboard_default() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("A", true), dark.pill_keyboard);
        assert_eq!(pill_color("F12", true), dark.pill_keyboard);
        assert_eq!(pill_color("LCTRL", true), dark.pill_keyboard);
    }

    #[test]
    fn pill_color_respects_theme() {
        assert_ne!(pill_color("A", true), pill_color("A", false));
    }

    #[test]
    fn pill_icon_and_label_mouse() {
        assert_eq!(pill_icon_and_label("MOUSE_UP"), ("↑", "MOUSE_UP".to_string()));
        assert_eq!(
            pill_icon_and_label("mouse_down_right"),
            ("↘", "MOUSE_DOWN_RIGHT".to_string()),
        );
    }

    #[test]
    fn pill_icon_and_label_gamepad() {
        assert_eq!(
            pill_icon_and_label("GAMEPAD_045E_A"),
            ("🎮", "GAMEPAD_045E_A".to_string()),
        );
    }

    #[test]
    fn pill_icon_and_label_keyboard_fallback() {
        assert_eq!(pill_icon_and_label("F12"), ("⌨", "F12".to_string()));
    }

    #[test]
    fn estimate_pill_width_display_grows_with_label_length() {
        let a = estimate_pill_width_display("A");
        let b = estimate_pill_width_display("MOUSE_DOWN_RIGHT");
        assert!(b > a);
    }

    #[test]
    fn estimate_pill_width_editor_includes_index_padding() {
        let key = "A";
        assert!(estimate_pill_width_editor(key) > estimate_pill_width_display(key));
    }

    #[test]
    fn arrow_separator_width_is_const() {
        assert_eq!(arrow_separator_width(), 18.0);
    }
}
