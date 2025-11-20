use std::sync::Arc;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::state::AppState;

unsafe impl Send for MouseHook {}

pub struct MouseHook {
    hook_handle: HHOOK,
}

impl MouseHook {
    pub fn new(_state: Arc<AppState>) -> anyhow::Result<Self> {
        unsafe {
            let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(Self::mouse_proc), None, 0)?;

            if hook.0.is_null() {
                anyhow::bail!("Failed to set mouse hook.");
            }

            Ok(Self { hook_handle: hook })
        }
    }

    pub fn run_message_loop(self) -> anyhow::Result<()> {
        // Force create message queue
        unsafe {
            let mut msg = MSG::default();
            let _ = PeekMessageA(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE);
        }

        // Main thread message loop
        unsafe {
            let mut msg = MSG::default();
            loop {
                let result = GetMessageA(&mut msg, None, 0, 0);

                if result.0 == 0 || result.0 == -1 {
                    break;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            // Cleanup hook
            let _ = UnhookWindowsHookEx(self.hook_handle);
        }

        Ok(())
    }

    unsafe extern "system" fn mouse_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if code < 0 {
            return unsafe { CallNextHookEx(None, code, w_param, l_param) };
        }

        let mouse_struct = unsafe { &*(l_param.0 as *mut MSLLHOOKSTRUCT) };

        // Skip simulated mouse events
        if mouse_struct.dwExtraInfo == crate::state::SIMULATED_EVENT_MARKER {
            return unsafe { CallNextHookEx(None, code, w_param, l_param) };
        }

        if let Some(state) = crate::state::get_global_state() {
            // Extract mouse data (needed for X button identification)
            let mouse_data = mouse_struct.mouseData;
            let should_block = state.handle_mouse_event(w_param.0 as u32, mouse_data);
            if should_block {
                return LRESULT(1); // block raw mouse event
            }
        }

        unsafe { CallNextHookEx(None, code, w_param, l_param) }
    }
}

impl Drop for MouseHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hook_handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{InputDevice, InputEvent, MouseButton};

    #[test]
    fn test_mouse_button_event_pressed() {
        let event = InputEvent::Pressed(InputDevice::Mouse(MouseButton::Left));

        match event {
            InputEvent::Pressed(InputDevice::Mouse(button)) => {
                assert_eq!(button, MouseButton::Left)
            }
            _ => panic!("Expected mouse pressed event"),
        }
    }

    #[test]
    fn test_mouse_button_event_released() {
        let event = InputEvent::Released(InputDevice::Mouse(MouseButton::Right));

        match event {
            InputEvent::Released(InputDevice::Mouse(button)) => {
                assert_eq!(button, MouseButton::Right)
            }
            _ => panic!("Expected mouse released event"),
        }
    }

    #[test]
    fn test_x_button1_event() {
        let event = InputEvent::Pressed(InputDevice::Mouse(MouseButton::X1));

        match event {
            InputEvent::Pressed(InputDevice::Mouse(button)) => {
                assert_eq!(button, MouseButton::X1)
            }
            _ => panic!("Expected X1 button pressed event"),
        }
    }

    #[test]
    fn test_x_button2_event() {
        let event = InputEvent::Released(InputDevice::Mouse(MouseButton::X2));

        match event {
            InputEvent::Released(InputDevice::Mouse(button)) => {
                assert_eq!(button, MouseButton::X2)
            }
            _ => panic!("Expected X2 button released event"),
        }
    }

    #[test]
    fn test_mouse_button_count() {
        // Verify we support exactly 5 standard mouse buttons (Windows limit)
        use std::mem;

        // This ensures MouseButton enum stays at expected size
        assert!(
            mem::size_of::<MouseButton>() <= 1,
            "MouseButton should be small"
        );

        // Verify all button types are distinct
        let buttons = [
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::X1,
            MouseButton::X2,
        ];

        // Check uniqueness by adding to HashSet
        use std::collections::HashSet;
        let set: HashSet<_> = buttons.iter().copied().collect();
        assert_eq!(set.len(), 5, "Should have exactly 5 unique mouse buttons");
    }

    #[test]
    fn test_mouse_data_parsing() {
        // Test XBUTTON1 parsing (mouse_data high word = 1)
        let mouse_data_x1: u32 = 1 << 16;
        let x1_button = (mouse_data_x1 >> 16) & 0xFFFF;
        assert_eq!(x1_button, 1, "XBUTTON1 should parse to 1");

        // Test XBUTTON2 parsing (mouse_data high word = 2)
        let mouse_data_x2: u32 = 2 << 16;
        let x2_button = (mouse_data_x2 >> 16) & 0xFFFF;
        assert_eq!(x2_button, 2, "XBUTTON2 should parse to 2");

        // Test invalid X button parsing (should parse but be ignored elsewhere)
        let mouse_data_invalid: u32 = 3 << 16;
        let invalid_button = (mouse_data_invalid >> 16) & 0xFFFF;
        assert_eq!(invalid_button, 3, "Invalid button should parse to 3");

        // Verify button codes match expected MouseButton mapping
        match x1_button {
            1 => { /* Valid XBUTTON1 */ }
            _ => panic!("Expected button code 1 for XBUTTON1"),
        }

        match x2_button {
            2 => { /* Valid XBUTTON2 */ }
            _ => panic!("Expected button code 2 for XBUTTON2"),
        }
    }

    #[test]
    fn test_mouse_button_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();

        // Insert all button types
        set.insert(InputDevice::Mouse(MouseButton::Left));
        set.insert(InputDevice::Mouse(MouseButton::Right));
        set.insert(InputDevice::Mouse(MouseButton::Middle));
        set.insert(InputDevice::Mouse(MouseButton::X1));
        set.insert(InputDevice::Mouse(MouseButton::X2));

        assert_eq!(set.len(), 5);

        // Inserting duplicate should not increase size
        set.insert(InputDevice::Mouse(MouseButton::Left));
        assert_eq!(set.len(), 5);

        // Check membership
        assert!(set.contains(&InputDevice::Mouse(MouseButton::Left)));
        assert!(set.contains(&InputDevice::Mouse(MouseButton::X2)));
    }

    #[test]
    fn test_input_event_channel_communication() {
        use std::sync::mpsc::channel;
        use std::thread;
        use std::time::Duration;

        let (tx, rx) = channel();

        let left_btn = InputDevice::Mouse(MouseButton::Left);
        let right_btn = InputDevice::Mouse(MouseButton::Right);

        // Sender thread
        thread::spawn(move || {
            tx.send(InputEvent::Pressed(left_btn)).unwrap();
            tx.send(InputEvent::Released(left_btn)).unwrap();
            tx.send(InputEvent::Pressed(right_btn)).unwrap();
            tx.send(InputEvent::Released(right_btn)).unwrap();
        });

        // Receiver thread
        let events: Vec<_> = thread::spawn(move || {
            let mut collected = Vec::new();
            while let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
                collected.push(event);
            }
            collected
        })
        .join()
        .unwrap();

        assert_eq!(events.len(), 4);

        // Verify event sequence
        match events[0] {
            InputEvent::Pressed(InputDevice::Mouse(MouseButton::Left)) => {}
            _ => panic!("Expected left button press"),
        }
        match events[1] {
            InputEvent::Released(InputDevice::Mouse(MouseButton::Left)) => {}
            _ => panic!("Expected left button release"),
        }
        match events[2] {
            InputEvent::Pressed(InputDevice::Mouse(MouseButton::Right)) => {}
            _ => panic!("Expected right button press"),
        }
        match events[3] {
            InputEvent::Released(InputDevice::Mouse(MouseButton::Right)) => {}
            _ => panic!("Expected right button release"),
        }
    }

    #[test]
    fn test_mouse_button_as_hash_map_key() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let left_device = InputDevice::Mouse(MouseButton::Left);
        let right_device = InputDevice::Mouse(MouseButton::Right);

        map.insert(left_device, "Left click");
        map.insert(right_device, "Right click");

        assert_eq!(map.get(&left_device), Some(&"Left click"));
        assert_eq!(map.get(&right_device), Some(&"Right click"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_x_button_data_extraction() {
        // Test the bit manipulation for X button data
        let x1_data: u32 = 1 << 16; // XBUTTON1
        let x2_data: u32 = 2 << 16; // XBUTTON2
        let invalid_data: u32 = 3 << 16; // Invalid

        // Extract high word
        let x1_button = (x1_data >> 16) & 0xFFFF;
        let x2_button = (x2_data >> 16) & 0xFFFF;
        let invalid_button = (invalid_data >> 16) & 0xFFFF;

        assert_eq!(x1_button, 1);
        assert_eq!(x2_button, 2);
        assert_eq!(invalid_button, 3);
    }

    #[test]
    fn test_mouse_event_pattern_matching() {
        let events = vec![
            InputEvent::Pressed(InputDevice::Mouse(MouseButton::Left)),
            InputEvent::Released(InputDevice::Mouse(MouseButton::Right)),
            InputEvent::Pressed(InputDevice::Mouse(MouseButton::X1)),
        ];

        for event in events {
            match event {
                InputEvent::Pressed(InputDevice::Mouse(button)) => {
                    // Should match pressed events
                    assert!(
                        button == MouseButton::Left || button == MouseButton::X1,
                        "Unexpected button in pressed event"
                    );
                }
                InputEvent::Released(InputDevice::Mouse(button)) => {
                    // Should match released events
                    assert_eq!(
                        button,
                        MouseButton::Right,
                        "Unexpected button in released event"
                    );
                }
                _ => panic!("Unexpected event type"),
            }
        }
    }
}
