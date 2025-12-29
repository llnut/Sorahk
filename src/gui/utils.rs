//! GUI utility functions.

use eframe::egui;

/// Converts key name string to Windows VK code.
pub fn string_to_vk(key_name: &str) -> Option<u32> {
    let key_upper = key_name.to_uppercase();
    match key_upper.as_str() {
        // A-Z (0x41-0x5A)
        "A" => Some(0x41),
        "B" => Some(0x42),
        "C" => Some(0x43),
        "D" => Some(0x44),
        "E" => Some(0x45),
        "F" => Some(0x46),
        "G" => Some(0x47),
        "H" => Some(0x48),
        "I" => Some(0x49),
        "J" => Some(0x4A),
        "K" => Some(0x4B),
        "L" => Some(0x4C),
        "M" => Some(0x4D),
        "N" => Some(0x4E),
        "O" => Some(0x4F),
        "P" => Some(0x50),
        "Q" => Some(0x51),
        "R" => Some(0x52),
        "S" => Some(0x53),
        "T" => Some(0x54),
        "U" => Some(0x55),
        "V" => Some(0x56),
        "W" => Some(0x57),
        "X" => Some(0x58),
        "Y" => Some(0x59),
        "Z" => Some(0x5A),
        // 0-9 (0x30-0x39)
        "0" => Some(0x30),
        "1" => Some(0x31),
        "2" => Some(0x32),
        "3" => Some(0x33),
        "4" => Some(0x34),
        "5" => Some(0x35),
        "6" => Some(0x36),
        "7" => Some(0x37),
        "8" => Some(0x38),
        "9" => Some(0x39),
        // F1-F24 (0x70-0x87)
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        "F13" => Some(0x7C),
        "F14" => Some(0x7D),
        "F15" => Some(0x7E),
        "F16" => Some(0x7F),
        "F17" => Some(0x80),
        "F18" => Some(0x81),
        "F19" => Some(0x82),
        "F20" => Some(0x83),
        "F21" => Some(0x84),
        "F22" => Some(0x85),
        "F23" => Some(0x86),
        "F24" => Some(0x87),
        // Navigation and editing keys
        "DELETE" => Some(0x2E),
        "INSERT" => Some(0x2D),
        "HOME" => Some(0x24),
        "END" => Some(0x23),
        "PAGEUP" => Some(0x21),
        "PAGEDOWN" => Some(0x22),
        "SPACE" => Some(0x20),
        "TAB" => Some(0x09),
        "ESCAPE" | "ESC" => Some(0x1B),
        "RETURN" | "ENTER" => Some(0x0D),
        "BACK" | "BACKSPACE" => Some(0x08),
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),
        _ => None,
    }
}

/// Loads embedded application icon.
pub fn create_icon() -> egui::IconData {
    const ICON_BYTES: &[u8] = include_bytes!("../../resources/sorahk.ico");

    let icon_dir = ico::IconDir::read(std::io::Cursor::new(ICON_BYTES))
        .expect("Failed to parse embedded icon");

    let entry = icon_dir
        .entries()
        .iter()
        .filter(|e| e.width() >= 32)
        .max_by_key(|e| e.width())
        .or_else(|| icon_dir.entries().first())
        .expect("No icon entries found");

    let image = entry.decode().expect("Failed to decode icon");
    let rgba_data = image.rgba_data().to_vec();

    egui::IconData {
        rgba: rgba_data,
        width: image.width(),
        height: image.height(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_vk_letters() {
        assert_eq!(string_to_vk("A"), Some(0x41));
        assert_eq!(string_to_vk("Z"), Some(0x5A));
        assert_eq!(string_to_vk("m"), Some(0x4D)); // Case insensitive
    }

    #[test]
    fn test_string_to_vk_numbers() {
        assert_eq!(string_to_vk("0"), Some(0x30));
        assert_eq!(string_to_vk("5"), Some(0x35));
        assert_eq!(string_to_vk("9"), Some(0x39));
    }

    #[test]
    fn test_string_to_vk_function_keys_f1_to_f24() {
        // F1-F12
        assert_eq!(string_to_vk("F1"), Some(0x70));
        assert_eq!(string_to_vk("F12"), Some(0x7B));
        // F13-F24
        assert_eq!(string_to_vk("F13"), Some(0x7C));
        assert_eq!(string_to_vk("F24"), Some(0x87));
        // Case insensitive
        assert_eq!(string_to_vk("f14"), Some(0x7D));
    }

    #[test]
    fn test_string_to_vk_special_keys() {
        assert_eq!(string_to_vk("DELETE"), Some(0x2E));
        assert_eq!(string_to_vk("INSERT"), Some(0x2D));
        assert_eq!(string_to_vk("HOME"), Some(0x24));
        assert_eq!(string_to_vk("SPACE"), Some(0x20));
        assert_eq!(string_to_vk("ENTER"), Some(0x0D));
        assert_eq!(string_to_vk("RETURN"), Some(0x0D)); // Alias
        assert_eq!(string_to_vk("delete"), Some(0x2E)); // Case insensitive
    }

    #[test]
    fn test_string_to_vk_navigation_keys() {
        assert_eq!(string_to_vk("UP"), Some(0x26));
        assert_eq!(string_to_vk("DOWN"), Some(0x28));
        assert_eq!(string_to_vk("LEFT"), Some(0x25));
        assert_eq!(string_to_vk("RIGHT"), Some(0x27));
    }

    #[test]
    fn test_string_to_vk_unsupported() {
        assert_eq!(string_to_vk("INVALID"), None);
        assert_eq!(string_to_vk(""), None);
        assert_eq!(string_to_vk("F25"), None);
        assert_eq!(string_to_vk("ABC"), None);
    }

    #[test]
    fn test_create_icon_basic() {
        // Test that create_icon returns valid IconData
        let icon = create_icon();

        // Basic validation
        assert!(icon.width > 0, "Icon width should be positive");
        assert!(icon.height > 0, "Icon height should be positive");
        assert!(!icon.rgba.is_empty(), "Icon RGBA data should not be empty");

        // Verify RGBA data size matches dimensions
        assert_eq!(
            icon.rgba.len(),
            (icon.width * icon.height * 4) as usize,
            "RGBA data size should match width * height * 4"
        );
    }
}
