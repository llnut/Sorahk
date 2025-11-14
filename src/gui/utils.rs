// GUI utility functions

use eframe::egui;

/// Convert egui::Key to virtual key code string
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

/// Convert virtual key code string to egui::Key
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

/// Load embedded application icon from binary
pub fn create_icon() -> egui::IconData {
    // Embed icon data directly into binary at compile time
    const ICON_BYTES: &[u8] = include_bytes!("../../resources/sorahk.ico");

    // Parse the embedded icon data
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(ICON_BYTES))
        .expect("Failed to parse embedded icon data");

    // Select the best quality icon (prefer 32x32 or larger)
    let entry = icon_dir
        .entries()
        .iter()
        .filter(|e| e.width() >= 32)
        .max_by_key(|e| e.width())
        .or_else(|| icon_dir.entries().first())
        .expect("No icon entries found in embedded icon");

    let image = entry.decode().expect("Failed to decode embedded icon");
    let rgba_data = image.rgba_data().to_vec();

    egui::IconData {
        rgba: rgba_data,
        width: image.width(),
        height: image.height(),
    }
}
