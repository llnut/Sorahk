use smallvec::SmallVec;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::types::*;
use super::AppState;

impl AppState {
    #[inline]
    pub fn simulate_action(&self, action: OutputAction, duration: u64) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut press_flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(scancode) {
                        press_flags |= KEYEVENTF_EXTENDEDKEY;
                    }
                    let mut input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: scancode,
                                dwFlags: press_flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(duration));
                    input.Anonymous.ki.dwFlags = press_flags | KEYEVENTF_KEYUP;
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    let (down_flag, up_flag) = match button {
                        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
                        MouseButton::X1 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP),
                        MouseButton::X2 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP),
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };
                    let mut input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: down_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(duration));
                    input.Anonymous.mi.dwFlags = up_flag;
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseMove(direction, speed) => {
                    let (dx, dy) = match direction {
                        MouseMoveDirection::Up => (0, -speed),
                        MouseMoveDirection::Down => (0, speed),
                        MouseMoveDirection::Left => (-speed, 0),
                        MouseMoveDirection::Right => (speed, 0),
                        MouseMoveDirection::UpLeft => (-speed, -speed),
                        MouseMoveDirection::UpRight => (speed, -speed),
                        MouseMoveDirection::DownLeft => (-speed, speed),
                        MouseMoveDirection::DownRight => (speed, speed),
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx,
                                dy,
                                mouseData: 0,
                                dwFlags: MOUSEEVENTF_MOVE,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseScroll(direction, speed) => {
                    let wheel_delta = match direction {
                        MouseScrollDirection::Up => speed,
                        MouseScrollDirection::Down => -speed,
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: wheel_delta as u32,
                                dwFlags: MOUSEEVENTF_WHEEL,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };

                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::KeyCombo(scancodes) => {
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::with_capacity(scancodes.len());
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }
                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    inputs.clear();
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }
                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect press events for batch processing
                    let mut press_inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_press_inputs(&actions, &mut press_inputs);
                    if !press_inputs.is_empty() {
                        SendInput(&press_inputs, std::mem::size_of::<INPUT>() as i32);
                    }

                    // Hold duration
                    std::thread::sleep(std::time::Duration::from_millis(duration));

                    // Collect release events for batch processing
                    let mut release_inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_release_inputs(&actions, &mut release_inputs);
                    if !release_inputs.is_empty() {
                        SendInput(&release_inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
                OutputAction::SequentialActions(actions, interval_ms) => {
                    for (idx, action) in actions.iter().enumerate() {
                        self.simulate_action(action.clone(), duration);
                        if idx < actions.len() - 1 {
                            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub(super) fn is_extended_scancode(scancode: u16) -> bool {
        const EXTENDED_KEYS_BITMAP: u128 = (1u128 << 0x1D)
            | (1u128 << 0x38)
            | (1u128 << 0x47)
            | (1u128 << 0x48)
            | (1u128 << 0x49)
            | (1u128 << 0x4B)
            | (1u128 << 0x4D)
            | (1u128 << 0x4F)
            | (1u128 << 0x50)
            | (1u128 << 0x51)
            | (1u128 << 0x52)
            | (1u128 << 0x53)
            | (1u128 << 0x5B)
            | (1u128 << 0x5C);

        scancode < 128 && (EXTENDED_KEYS_BITMAP & (1u128 << scancode)) != 0
    }

    #[inline(always)]
    pub fn simulate_press(&self, action: &OutputAction) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    let down_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
                        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XDOWN,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: down_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseMove(_, _) => {
                    // Mouse movement doesn't have press/release states
                }
                OutputAction::MouseScroll(_, _) => {
                    // Mouse scroll doesn't have press/release states
                }
                OutputAction::KeyCombo(scancodes) => {
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::with_capacity(scancodes.len());
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }
                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect and send all press events in a single call
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_press_inputs(actions, &mut inputs);
                    if !inputs.is_empty() {
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
                OutputAction::SequentialActions(actions, _) => {
                    // For sequential actions, only press the first action
                    if let Some(first_action) = actions.first() {
                        self.simulate_press(first_action);
                    }
                }
            }
        }
    }

    /// Simulates only the release event for an action
    #[inline(always)]
    pub fn simulate_release(&self, action: &OutputAction) {
        unsafe {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseButton(button) => {
                    let up_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTUP,
                        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XUP,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: up_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    };
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MouseMove(_, _) => {
                    // Mouse movement doesn't have press/release states
                }
                OutputAction::MouseScroll(_, _) => {
                    // Mouse scroll doesn't have press/release states
                }
                OutputAction::KeyCombo(scancodes) => {
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::with_capacity(scancodes.len());
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }
                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                }
                OutputAction::MultipleActions(actions) => {
                    // Collect and send all release events in a single call
                    let mut inputs: SmallVec<[INPUT; 8]> = SmallVec::new();
                    self.collect_release_inputs(actions, &mut inputs);
                    if !inputs.is_empty() {
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
                OutputAction::SequentialActions(actions, _) => {
                    // For sequential actions, only release the first action
                    if let Some(first_action) = actions.first() {
                        self.simulate_release(first_action);
                    }
                }
            }
        }
    }

    /// Collects press INPUT events from actions into a buffer
    #[inline(always)]
    pub(super) fn collect_press_inputs(
        &self,
        actions: &SmallVec<[OutputAction; 4]>,
        inputs: &mut SmallVec<[INPUT; 8]>,
    ) {
        for action in actions.iter() {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::MouseButton(button) => {
                    let down_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
                        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XDOWN,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    inputs.push(INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: down_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter() {
                        let mut flags = KEYEVENTF_SCANCODE;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }

                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                }
                OutputAction::MultipleActions(nested_actions) => {
                    // Recursively collect nested actions
                    self.collect_press_inputs(nested_actions, inputs);
                }
                OutputAction::SequentialActions(nested_actions, _) => {
                    // Recursively collect nested actions (press all for collect)
                    self.collect_press_inputs(nested_actions, inputs);
                }
                OutputAction::MouseMove(_, _) | OutputAction::MouseScroll(_, _) => {
                    // Skip actions without press state
                }
            }
        }
    }

    /// Collects release INPUT events from actions into a buffer
    #[inline(always)]
    pub(super) fn collect_release_inputs(
        &self,
        actions: &SmallVec<[OutputAction; 4]>,
        inputs: &mut SmallVec<[INPUT; 8]>,
    ) {
        for action in actions.iter() {
            match action {
                OutputAction::KeyboardKey(scancode) => {
                    let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    if Self::is_extended_scancode(*scancode) {
                        flags |= KEYEVENTF_EXTENDEDKEY;
                    }

                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: *scancode,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::MouseButton(button) => {
                    let up_flag = match button {
                        MouseButton::Left => MOUSEEVENTF_LEFTUP,
                        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
                        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
                        MouseButton::X1 | MouseButton::X2 => MOUSEEVENTF_XUP,
                    };

                    let mouse_data = match button {
                        MouseButton::X1 => 1,
                        MouseButton::X2 => 2,
                        _ => 0,
                    };

                    inputs.push(INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: up_flag,
                                time: 0,
                                dwExtraInfo: SIMULATED_EVENT_MARKER,
                            },
                        },
                    });
                }
                OutputAction::KeyCombo(scancodes) => {
                    for &scancode in scancodes.iter().rev() {
                        let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        if Self::is_extended_scancode(scancode) {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }

                        inputs.push(INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(0),
                                    wScan: scancode,
                                    dwFlags: flags,
                                    time: 0,
                                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                                },
                            },
                        });
                    }
                }
                OutputAction::MultipleActions(nested_actions) => {
                    // Recursively collect nested actions
                    self.collect_release_inputs(nested_actions, inputs);
                }
                OutputAction::SequentialActions(nested_actions, _) => {
                    // Recursively collect nested actions (release all for collect)
                    self.collect_release_inputs(nested_actions, inputs);
                }
                OutputAction::MouseMove(_, _) | OutputAction::MouseScroll(_, _) => {
                    // Skip actions without release state
                }
            }
        }
    }
}
