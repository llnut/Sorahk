//! GUI utility functions.

use eframe::egui;

/// Converts egui::Key to virtual key code string.
pub fn key_to_string(key: egui::Key) -> Option<String> {
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

/// Converts virtual key code string to egui::Key.
pub fn string_to_key(key_name: &str) -> Option<egui::Key> {
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
    fn test_key_to_string_letters() {
        assert_eq!(key_to_string(egui::Key::A), Some("A".to_string()));
        assert_eq!(key_to_string(egui::Key::M), Some("M".to_string()));
        assert_eq!(key_to_string(egui::Key::Z), Some("Z".to_string()));
    }

    #[test]
    fn test_key_to_string_numbers() {
        assert_eq!(key_to_string(egui::Key::Num0), Some("0".to_string()));
        assert_eq!(key_to_string(egui::Key::Num5), Some("5".to_string()));
        assert_eq!(key_to_string(egui::Key::Num9), Some("9".to_string()));
    }

    #[test]
    fn test_key_to_string_function_keys() {
        assert_eq!(key_to_string(egui::Key::F1), Some("F1".to_string()));
        assert_eq!(key_to_string(egui::Key::F6), Some("F6".to_string()));
        assert_eq!(key_to_string(egui::Key::F12), Some("F12".to_string()));
    }

    #[test]
    fn test_key_to_string_special_keys() {
        assert_eq!(key_to_string(egui::Key::Space), Some("SPACE".to_string()));
        assert_eq!(key_to_string(egui::Key::Enter), Some("RETURN".to_string()));
        assert_eq!(key_to_string(egui::Key::Escape), Some("ESCAPE".to_string()));
        assert_eq!(key_to_string(egui::Key::Tab), Some("TAB".to_string()));
        assert_eq!(
            key_to_string(egui::Key::Backspace),
            Some("BACK".to_string())
        );
    }

    #[test]
    fn test_key_to_string_navigation_keys() {
        assert_eq!(key_to_string(egui::Key::ArrowUp), Some("UP".to_string()));
        assert_eq!(
            key_to_string(egui::Key::ArrowDown),
            Some("DOWN".to_string())
        );
        assert_eq!(
            key_to_string(egui::Key::ArrowLeft),
            Some("LEFT".to_string())
        );
        assert_eq!(
            key_to_string(egui::Key::ArrowRight),
            Some("RIGHT".to_string())
        );
        assert_eq!(key_to_string(egui::Key::Home), Some("HOME".to_string()));
        assert_eq!(key_to_string(egui::Key::End), Some("END".to_string()));
        assert_eq!(key_to_string(egui::Key::PageUp), Some("PAGEUP".to_string()));
        assert_eq!(
            key_to_string(egui::Key::PageDown),
            Some("PAGEDOWN".to_string())
        );
    }

    #[test]
    fn test_key_to_string_edit_keys() {
        assert_eq!(key_to_string(egui::Key::Delete), Some("DELETE".to_string()));
        assert_eq!(key_to_string(egui::Key::Insert), Some("INSERT".to_string()));
    }

    #[test]
    fn test_string_to_key_letters() {
        assert_eq!(string_to_key("A"), Some(egui::Key::A));
        assert_eq!(string_to_key("a"), Some(egui::Key::A)); // Case insensitive
        assert_eq!(string_to_key("Z"), Some(egui::Key::Z));
        assert_eq!(string_to_key("m"), Some(egui::Key::M));
    }

    #[test]
    fn test_string_to_key_numbers() {
        assert_eq!(string_to_key("0"), Some(egui::Key::Num0));
        assert_eq!(string_to_key("5"), Some(egui::Key::Num5));
        assert_eq!(string_to_key("9"), Some(egui::Key::Num9));
    }

    #[test]
    fn test_string_to_key_function_keys() {
        assert_eq!(string_to_key("F1"), Some(egui::Key::F1));
        assert_eq!(string_to_key("f6"), Some(egui::Key::F6)); // Case insensitive
        assert_eq!(string_to_key("F12"), Some(egui::Key::F12));
    }

    #[test]
    fn test_string_to_key_special_keys() {
        assert_eq!(string_to_key("SPACE"), Some(egui::Key::Space));
        assert_eq!(string_to_key("space"), Some(egui::Key::Space));
        assert_eq!(string_to_key("TAB"), Some(egui::Key::Tab));
        assert_eq!(string_to_key("ESCAPE"), Some(egui::Key::Escape));
        assert_eq!(string_to_key("ESC"), Some(egui::Key::Escape)); // Alias
    }

    #[test]
    fn test_string_to_key_enter_aliases() {
        assert_eq!(string_to_key("RETURN"), Some(egui::Key::Enter));
        assert_eq!(string_to_key("ENTER"), Some(egui::Key::Enter));
        assert_eq!(string_to_key("enter"), Some(egui::Key::Enter));
    }

    #[test]
    fn test_string_to_key_backspace_aliases() {
        assert_eq!(string_to_key("BACK"), Some(egui::Key::Backspace));
        assert_eq!(string_to_key("BACKSPACE"), Some(egui::Key::Backspace));
        assert_eq!(string_to_key("backspace"), Some(egui::Key::Backspace));
    }

    #[test]
    fn test_string_to_key_navigation_keys() {
        assert_eq!(string_to_key("UP"), Some(egui::Key::ArrowUp));
        assert_eq!(string_to_key("DOWN"), Some(egui::Key::ArrowDown));
        assert_eq!(string_to_key("LEFT"), Some(egui::Key::ArrowLeft));
        assert_eq!(string_to_key("RIGHT"), Some(egui::Key::ArrowRight));
        assert_eq!(string_to_key("HOME"), Some(egui::Key::Home));
        assert_eq!(string_to_key("END"), Some(egui::Key::End));
    }

    #[test]
    fn test_string_to_key_invalid() {
        assert_eq!(string_to_key("INVALID"), None);
        assert_eq!(string_to_key(""), None);
        assert_eq!(string_to_key("F13"), None); // Not supported
        assert_eq!(string_to_key("ABC"), None);
    }

    #[test]
    fn test_roundtrip_conversion_letters() {
        // Test that converting back and forth preserves the value
        for key in [egui::Key::A, egui::Key::M, egui::Key::Z] {
            let string = key_to_string(key).unwrap();
            let converted_back = string_to_key(&string).unwrap();
            assert_eq!(key, converted_back);
        }
    }

    #[test]
    fn test_roundtrip_conversion_numbers() {
        for key in [egui::Key::Num0, egui::Key::Num5, egui::Key::Num9] {
            let string = key_to_string(key).unwrap();
            let converted_back = string_to_key(&string).unwrap();
            assert_eq!(key, converted_back);
        }
    }

    #[test]
    fn test_roundtrip_conversion_function_keys() {
        for key in [egui::Key::F1, egui::Key::F6, egui::Key::F12] {
            let string = key_to_string(key).unwrap();
            let converted_back = string_to_key(&string).unwrap();
            assert_eq!(key, converted_back);
        }
    }

    #[test]
    fn test_roundtrip_conversion_special_keys() {
        let keys = [
            egui::Key::Space,
            egui::Key::Tab,
            egui::Key::Enter,
            egui::Key::Backspace,
            egui::Key::Delete,
            egui::Key::Insert,
            egui::Key::Home,
            egui::Key::End,
        ];

        for key in keys {
            let string = key_to_string(key).unwrap();
            let converted_back = string_to_key(&string).unwrap();
            assert_eq!(key, converted_back);
        }
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(string_to_key("space"), string_to_key("SPACE"));
        assert_eq!(string_to_key("escape"), string_to_key("ESCAPE"));
        assert_eq!(string_to_key("f5"), string_to_key("F5"));
        assert_eq!(string_to_key("delete"), string_to_key("DELETE"));
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
