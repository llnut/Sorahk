//! Helper functions for settings dialog display and styling.

use crate::i18n::CachedTranslations;
use crate::state::CaptureMode;

/// Maximum characters to display in button text before truncating.
pub const BUTTON_TEXT_MAX_CHARS: usize = 50;

/// Truncates text safely at UTF-8 boundaries for button display.
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
