//! Semantic color roles plus cached `Visuals` for both themes.

#![allow(dead_code)]

use eframe::egui::{self, Color32, Visuals, epaint::Shadow};

/// Semantic color roles for the GUI palette.
#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_window: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_input: Color32,

    pub fg_primary: Color32,
    pub fg_muted: Color32,
    pub fg_inverse: Color32,
    pub fg_link: Color32,

    pub title_primary: Color32,

    pub accent_primary: Color32,
    pub accent_secondary: Color32,
    pub accent_danger: Color32,
    pub accent_success: Color32,
    pub accent_warning: Color32,
    pub accent_pink: Color32,

    pub pill_keyboard: Color32,
    pub pill_mouse_button: Color32,
    pub pill_mouse_movement: Color32,
    pub pill_gamepad: Color32,
    pub pill_target: Color32,

    pub status_active: Color32,
    pub status_paused: Color32,

    pub divider: Color32,
}

pub static DARK: ThemeColors = ThemeColors {
    bg_window: Color32::from_rgb(25, 27, 35),
    bg_card: Color32::from_rgb(38, 40, 50),
    bg_card_hover: Color32::from_rgb(48, 50, 60),
    bg_input: Color32::from_rgb(42, 44, 55),

    fg_primary: Color32::from_rgb(220, 220, 220),
    fg_muted: Color32::from_rgb(170, 170, 190),
    fg_inverse: Color32::WHITE,
    fg_link: Color32::from_rgb(130, 190, 255),

    title_primary: Color32::from_rgb(170, 225, 255),

    accent_primary: Color32::from_rgb(181, 217, 252),
    accent_secondary: Color32::from_rgb(220, 196, 250),
    accent_danger: Color32::from_rgb(252, 170, 167),
    accent_success: Color32::from_rgb(181, 233, 199),
    accent_warning: Color32::from_rgb(251, 215, 158),
    accent_pink: Color32::from_rgb(255, 182, 193),

    pill_keyboard: Color32::from_rgb(255, 182, 193),
    pill_mouse_button: Color32::from_rgb(140, 180, 220),
    pill_mouse_movement: Color32::from_rgb(180, 140, 220),
    pill_gamepad: Color32::from_rgb(140, 200, 160),
    pill_target: Color32::from_rgb(135, 206, 235),

    status_active: Color32::from_rgb(181, 233, 199),
    status_paused: Color32::from_rgb(251, 215, 158),

    divider: Color32::from_rgb(60, 62, 72),
};

pub static LIGHT: ThemeColors = ThemeColors {
    bg_window: Color32::from_rgb(240, 235, 245),
    bg_card: Color32::from_rgb(250, 245, 255),
    bg_card_hover: Color32::from_rgb(252, 250, 255),
    bg_input: Color32::from_rgb(238, 233, 243),

    fg_primary: Color32::from_rgb(40, 40, 40),
    fg_muted: Color32::from_rgb(100, 100, 120),
    fg_inverse: Color32::WHITE,
    fg_link: Color32::from_rgb(60, 145, 235),

    title_primary: Color32::from_rgb(60, 145, 235),

    accent_primary: Color32::from_rgb(148, 191, 234),
    accent_secondary: Color32::from_rgb(192, 168, 226),
    accent_danger: Color32::from_rgb(240, 145, 142),
    accent_success: Color32::from_rgb(138, 215, 167),
    accent_warning: Color32::from_rgb(243, 191, 124),
    accent_pink: Color32::from_rgb(255, 150, 170),

    pill_keyboard: Color32::from_rgb(255, 210, 220),
    pill_mouse_button: Color32::from_rgb(180, 210, 255),
    pill_mouse_movement: Color32::from_rgb(220, 190, 255),
    pill_gamepad: Color32::from_rgb(180, 235, 200),
    pill_target: Color32::from_rgb(173, 216, 230),

    status_active: Color32::from_rgb(138, 215, 167),
    status_paused: Color32::from_rgb(243, 191, 124),

    divider: Color32::from_rgb(220, 220, 235),
};

#[inline]
pub fn colors(dark_mode: bool) -> &'static ThemeColors {
    if dark_mode { &DARK } else { &LIGHT }
}

/// Translucent overlay tints used as faint background fills.
pub mod overlay {
    use super::Color32;
    pub const PINK_TINT_DARK:  Color32 = Color32::from_rgba_premultiplied(255, 182, 193, 25);
    pub const PINK_TINT_LIGHT: Color32 = Color32::from_rgba_premultiplied(255, 218, 224, 120);
    pub const BLUE_TINT_DARK:  Color32 = Color32::from_rgba_premultiplied(135, 206, 235, 25);
    pub const BLUE_TINT_LIGHT: Color32 = Color32::from_rgba_premultiplied(173, 216, 230, 120);
    pub const SHADOW_LIGHT:    Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 25);
    pub const SHADOW_HEAVY:    Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 45);
}

fn build_dark_visuals() -> Visuals {
    let mut visuals = egui::Visuals::dark();

    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.selection.stroke.width = 0.0;

    visuals.window_fill = DARK.bg_window;
    visuals.panel_fill = Color32::from_rgb(30, 32, 40);
    visuals.faint_bg_color = Color32::from_rgb(35, 37, 45);
    visuals.widgets.noninteractive.weak_bg_fill = DARK.bg_card;
    visuals.extreme_bg_color = DARK.bg_input;

    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 18,
        spread: 0,
        color: overlay::SHADOW_LIGHT,
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 3],
        blur: 12,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
    };

    visuals
}

fn build_light_visuals() -> Visuals {
    let mut visuals = egui::Visuals::light();

    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.selection.stroke.width = 0.0;

    visuals.window_fill = LIGHT.bg_window;
    visuals.panel_fill = LIGHT.bg_input;
    visuals.faint_bg_color = LIGHT.bg_card_hover;
    visuals.widgets.noninteractive.weak_bg_fill = LIGHT.bg_card;
    visuals.extreme_bg_color = Color32::from_rgb(235, 230, 245);

    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 18,
        spread: 0,
        color: overlay::SHADOW_LIGHT,
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 3],
        blur: 12,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
    };

    visuals
}

/// Owned by `SorahkGui`; pre-computed `Visuals` for both themes.
pub struct ThemeCache {
    pub dark: Visuals,
    pub light: Visuals,
}

impl ThemeCache {
    pub fn new() -> Self {
        Self {
            dark: build_dark_visuals(),
            light: build_light_visuals(),
        }
    }

    #[inline]
    pub fn visuals(&self, dark_mode: bool) -> &Visuals {
        if dark_mode { &self.dark } else { &self.light }
    }
}

impl Default for ThemeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_dispatches_by_dark_mode() {
        assert!(std::ptr::eq(colors(true), &DARK));
        assert!(std::ptr::eq(colors(false), &LIGHT));
    }

    #[test]
    fn dark_and_light_have_distinct_window_fills() {
        assert_ne!(DARK.bg_window, LIGHT.bg_window);
    }
}
