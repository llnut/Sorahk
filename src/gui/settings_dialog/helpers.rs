//! Helper functions for settings dialog display and styling.

use crate::i18n::CachedTranslations;
use crate::state::CaptureMode;
use eframe::egui;

/// Maximum characters to display in button text before truncating
pub const BUTTON_TEXT_MAX_CHARS: usize = 50;

/// Truncates text safely at UTF-8 boundaries for button display
#[inline]
pub fn truncate_text_safe(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }
    let take_count = max_chars.saturating_sub(3);
    if take_count == 0 {
        "...".chars().take(max_chars).collect()
    } else {
        let mut result = String::with_capacity(max_chars);
        result.extend(text.chars().take(take_count));
        result.push_str("...");
        result
    }
}

/// Gets localized display name for a capture mode.
#[inline]
pub fn get_capture_mode_display_name(t: &CachedTranslations, mode: CaptureMode) -> &str {
    match mode {
        CaptureMode::MostSustained => t.capture_mode_most_sustained(),
        CaptureMode::AdaptiveIntelligent => t.capture_mode_adaptive_intelligent(),
        CaptureMode::MaxChangedBits => t.capture_mode_max_changed_bits(),
        CaptureMode::MaxSetBits => t.capture_mode_max_set_bits(),
        CaptureMode::LastStable => t.capture_mode_last_stable(),
        CaptureMode::HatSwitchOptimized => t.capture_mode_hat_switch_optimized(),
        CaptureMode::AnalogOptimized => t.capture_mode_analog_optimized(),
    }
}

/// Check if target key is a mouse movement action.
#[inline]
pub fn is_mouse_move_target(target: &str) -> bool {
    let upper = target.to_uppercase();
    matches!(
        upper.as_str(),
        "MOUSE_UP"
            | "MOUSE_DOWN"
            | "MOUSE_LEFT"
            | "MOUSE_RIGHT"
            | "MOUSE_UP_LEFT"
            | "MOUSE_UP_RIGHT"
            | "MOUSE_DOWN_LEFT"
            | "MOUSE_DOWN_RIGHT"
            | "MOUSEUP"
            | "MOUSEDOWN"
            | "MOUSELEFT"
            | "MOUSERIGHT"
            | "MOVE_UP"
            | "MOVE_DOWN"
            | "MOVE_LEFT"
            | "MOVE_RIGHT"
            | "M_UP"
            | "M_DOWN"
            | "M_LEFT"
            | "M_RIGHT"
            | "MOUSEUPLEFT"
            | "MOUSEUPRIGHT"
            | "MOUSEDOWNLEFT"
            | "MOUSEDOWNRIGHT"
            | "M_UP_LEFT"
            | "M_UP_RIGHT"
            | "M_DOWN_LEFT"
            | "M_DOWN_RIGHT"
    )
}

/// Check if target key is a mouse scroll action.
#[inline]
pub fn is_mouse_scroll_target(target: &str) -> bool {
    let upper = target.to_uppercase();
    matches!(
        upper.as_str(),
        "SCROLL_UP"
            | "SCROLLUP"
            | "WHEEL_UP"
            | "WHEELUP"
            | "SCROLL_DOWN"
            | "SCROLLDOWN"
            | "WHEEL_DOWN"
            | "WHEELDOWN"
    )
}

/// Converts mouse delta to 8-directional movement string.
/// Screen coordinates: Y+ = down, Y- = up.
#[inline]
pub fn calculate_mouse_direction(delta: egui::Vec2, threshold: f32) -> Option<&'static str> {
    let mag_sq = delta.x * delta.x + delta.y * delta.y;
    if mag_sq < threshold * threshold {
        return None;
    }

    let angle = delta.y.atan2(delta.x).to_degrees();

    Some(if (-22.5..22.5).contains(&angle) {
        "MOUSE_RIGHT"
    } else if (22.5..67.5).contains(&angle) {
        "MOUSE_DOWN_RIGHT"
    } else if (67.5..112.5).contains(&angle) {
        "MOUSE_DOWN"
    } else if (112.5..157.5).contains(&angle) {
        "MOUSE_DOWN_LEFT"
    } else if !(-157.5..157.5).contains(&angle) {
        "MOUSE_LEFT"
    } else if (-157.5..-112.5).contains(&angle) {
        "MOUSE_UP_LEFT"
    } else if (-112.5..-67.5).contains(&angle) {
        "MOUSE_UP"
    } else {
        "MOUSE_UP_RIGHT"
    })
}

/// Returns (icon, display_name) tuple for a sequence key.
#[inline]
pub fn get_sequence_key_display(key: &str) -> (&'static str, String) {
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

/// Returns the background color for a sequence key tag.
#[inline]
pub fn get_sequence_key_color(key: &str, dark_mode: bool) -> egui::Color32 {
    let upper = key.to_uppercase();
    if upper.starts_with("MOUSE_UP")
        || upper.starts_with("MOUSE_DOWN")
        || upper.starts_with("MOUSE_LEFT")
        || upper.starts_with("MOUSE_RIGHT")
    {
        if dark_mode {
            egui::Color32::from_rgb(180, 140, 220)
        } else {
            egui::Color32::from_rgb(220, 190, 255)
        }
    } else if upper.contains("BUTTON") {
        if dark_mode {
            egui::Color32::from_rgb(140, 180, 220)
        } else {
            egui::Color32::from_rgb(180, 210, 255)
        }
    } else if upper.starts_with("GAMEPAD_") || upper.starts_with("JOYSTICK_") {
        if dark_mode {
            egui::Color32::from_rgb(140, 200, 160)
        } else {
            egui::Color32::from_rgb(180, 235, 200)
        }
    } else if dark_mode {
        egui::Color32::from_rgb(255, 182, 193)
    } else {
        egui::Color32::from_rgb(255, 210, 220)
    }
}

/// Estimates the display width of a sequence key pill tag.
#[inline]
pub fn estimate_pill_width(key: &str) -> f32 {
    let (_icon, display_name) = get_sequence_key_display(key);
    let base_width = 53.0; // padding + index + spacing + delete button
    let text_width = display_name.chars().count().min(20) as f32 * 6.5;
    base_width + text_width
}

/// Estimates the width of an arrow separator.
#[inline]
pub const fn estimate_arrow_width() -> f32 {
    18.0
}

/// Returns the background color for a target key tag.
#[inline]
pub fn get_target_key_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(135, 206, 235)
    } else {
        egui::Color32::from_rgb(173, 216, 230)
    }
}

/// Estimates the display width of a target key pill tag.
#[inline]
pub fn estimate_target_pill_width(key: &str) -> f32 {
    let base_width = 53.0; // padding + index + spacing + delete button
    let text_width = key.chars().count().min(20) as f32 * 6.5;
    base_width + text_width
}
