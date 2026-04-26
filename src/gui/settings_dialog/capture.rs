//! Key capture functionality for settings dialog.

use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;
use crate::gui::utils::mouse_delta_to_direction;
use crate::util::numpad;
use eframe::egui;
use smallvec::SmallVec;
use std::collections::HashSet;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyState, VK_NUMLOCK};

impl SorahkGui {
    /// Drives the per-frame settings-dialog capture state machine.
    ///
    /// Polls keyboard, mouse, and raw input in priority order and routes
    /// captured keys into the matching field on `self`. Suppressed while
    /// the HID activation dialog or the rule properties dialog is open
    /// to avoid routing one physical release into two collections.
    pub(super) fn poll_capture(&mut self, ctx: &egui::Context) {
        // Capture pipeline runs in priority order: Keyboard, Mouse, Raw
        // Input. Suppressed while the HID activation dialog or the rule
        // properties dialog is open. The rule-properties dialog runs its
        // own capture state machine for append keys, so letting both
        // poll loops fire would route one physical release twice.
        if self.key_capture_mode != KeyCaptureMode::None
            && self.hid_activation_dialog.is_none()
            && self.rule_properties_dialog.is_none()
        {
            let mut captured_input: Option<String> = None;

            let is_sequence_capture = match self.key_capture_mode {
                KeyCaptureMode::NewMappingTrigger => self.new_mapping_is_sequence_mode,
                KeyCaptureMode::NewMappingTarget => self.new_mapping_target_mode == 2,
                KeyCaptureMode::MappingTrigger(idx) => {
                    if let Some(temp_config) = &self.temp_config {
                        temp_config
                            .mappings
                            .get(idx)
                            .map(|m| m.is_sequence_trigger())
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                KeyCaptureMode::MappingTarget(idx) => {
                    if let Some(temp_config) = &self.temp_config {
                        temp_config
                            .mappings
                            .get(idx)
                            .map(|m| m.target_mode == 2)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // Sequence mode uses a user-configurable finalize key, default
            // Enter, to end recording without joining the sequence itself.
            // A Done-button alternative would force extra mouse motion
            // into the trace.
            if is_sequence_capture
                && !self.just_captured_input
                && self.is_sequence_finalize_pressed()
            {
                // The Done-button paths commit captured lists back to the
                // mapping draft. The finalize-key shortcut previously
                // tore down capture state via the macro without the
                // commit, so target-sequence captures that ended with
                // Enter silently dropped every key. Mirror the
                // Done-button sync here for target-mode 2 captures.
                // Trigger-mode sequences mutate the mapping live during
                // capture, so no sync is required there.
                match self.key_capture_mode {
                    KeyCaptureMode::MappingTarget(idx) => {
                        if let Some(temp_config) = &mut self.temp_config
                            && let Some(m) = temp_config.mappings.get_mut(idx)
                            && m.target_mode == 2
                        {
                            m.target_keys = self.editing_target_seq_list.iter().cloned().collect();
                            let new_len = m.target_keys.len();
                            if let Some(holds) = m.hold_indices.as_mut() {
                                holds.retain(|i| (*i as usize) < new_len);
                                if holds.is_empty() {
                                    m.hold_indices = None;
                                }
                            }
                            self.editing_target_seq_list.clear();
                            self.editing_target_seq_idx = None;
                        }
                    }
                    KeyCaptureMode::NewMappingTarget
                        if self.new_mapping_target_mode == 2
                            && !self.target_sequence_capture_list.is_empty() =>
                    {
                        self.new_mapping_target_keys = self.target_sequence_capture_list.clone();
                        if let Some(first) = self.new_mapping_target_keys.first() {
                            self.new_mapping_target = first.clone();
                        }
                        let new_len = self.new_mapping_target_keys.len();
                        self.new_mapping_hold_indices
                            .retain(|i| (*i as usize) < new_len);
                    }
                    _ => {}
                }
                finalize_sequence_capture!(self);
            }

            if is_sequence_capture && !self.just_captured_input {
                // Query pointer-over state before borrowing `ctx.input` to avoid deadlock.
                let pointer_over_ui = ctx.is_pointer_over_area();

                ctx.input(|i| {
                    if self.sequence_last_mouse_pos.is_none() {
                        self.sequence_last_mouse_pos =
                            Some(i.pointer.hover_pos().unwrap_or_default());
                        self.sequence_mouse_delta = egui::Vec2::ZERO;
                    }

                    // Skip captures over UI widgets, and dedupe repeats of the
                    // same button so holding a mouse button does not spam.
                    if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Primary)
                        && self.sequence_last_mouse_direction.as_deref() != Some("LBUTTON")
                    {
                        captured_input = Some("LBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("LBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Secondary)
                        && self.sequence_last_mouse_direction.as_deref() != Some("RBUTTON")
                    {
                        captured_input = Some("RBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("RBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Middle)
                        && self.sequence_last_mouse_direction.as_deref() != Some("MBUTTON")
                    {
                        captured_input = Some("MBUTTON".to_string());
                        self.sequence_last_mouse_direction = Some("MBUTTON".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Extra1)
                        && self.sequence_last_mouse_direction.as_deref() != Some("XBUTTON1")
                    {
                        captured_input = Some("XBUTTON1".to_string());
                        self.sequence_last_mouse_direction = Some("XBUTTON1".to_string());
                    } else if !pointer_over_ui
                        && i.pointer.button_clicked(egui::PointerButton::Extra2)
                        && self.sequence_last_mouse_direction.as_deref() != Some("XBUTTON2")
                    {
                        captured_input = Some("XBUTTON2".to_string());
                        self.sequence_last_mouse_direction = Some("XBUTTON2".to_string());
                    }

                    if captured_input.is_none()
                        && let Some(current_pos) = i.pointer.hover_pos()
                        && let Some(last_pos) = self.sequence_last_mouse_pos
                    {
                        let frame_delta = current_pos - last_pos;
                        self.sequence_mouse_delta += frame_delta;

                        // 30 px accumulated delta commits a direction sample.
                        if let Some(direction) =
                            mouse_delta_to_direction(self.sequence_mouse_delta, 30.0)
                        {
                            captured_input = Some(direction.to_string());
                            self.sequence_last_mouse_direction = Some(direction.to_string());
                            self.sequence_mouse_delta = egui::Vec2::ZERO;
                        }

                        self.sequence_last_mouse_pos = Some(current_pos);
                    }
                });
            }

            if captured_input.is_none() && !self.just_captured_input {
                let current_pressed = Self::poll_all_pressed_keys();

                // Accumulate every newly pressed key that was not held when
                // capture started, then finalize once any accumulated key is
                // released. Waiting for a release is what lets modifier-plus-
                // main-key combos like LCTRL+C be captured as a whole rather
                // than one key at a time.
                current_pressed
                    .iter()
                    .filter(|&vk| !self.capture_initial_pressed.contains(vk))
                    .for_each(|&vk| {
                        self.capture_pressed_keys.insert(vk);
                    });

                let any_released = self
                    .capture_pressed_keys
                    .iter()
                    .any(|vk| !current_pressed.contains(vk));

                if any_released {
                    captured_input = Self::format_captured_keys(&self.capture_pressed_keys);
                }
            }

            // `just_captured_input` suppresses the click that entered capture
            // mode from being captured as the trigger. Sequences and the
            // finalize-key picker exclude mouse buttons by design.
            if captured_input.is_none()
                && !self.just_captured_input
                && !is_sequence_capture
                && self.key_capture_mode != KeyCaptureMode::SequenceFinalizeKey
            {
                ctx.input(|i| {
                    if i.pointer.button_clicked(egui::PointerButton::Primary) {
                        captured_input = Some("LBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Secondary) {
                        captured_input = Some("RBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Middle) {
                        captured_input = Some("MBUTTON".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Extra1) {
                        captured_input = Some("XBUTTON1".to_string());
                    } else if i.pointer.button_clicked(egui::PointerButton::Extra2) {
                        captured_input = Some("XBUTTON2".to_string());
                    }
                });
            }

            if captured_input.is_none() {
                let should_check_raw_input = matches!(
                    self.key_capture_mode,
                    KeyCaptureMode::ToggleKey
                        | KeyCaptureMode::MappingTrigger(_)
                        | KeyCaptureMode::MappingTarget(_)
                        | KeyCaptureMode::NewMappingTrigger
                        | KeyCaptureMode::NewMappingTarget
                );

                if should_check_raw_input
                    && let Some(device) = self.app_state.try_recv_raw_input_capture()
                {
                    captured_input = Some(device.to_string());
                }
            }

            if let Some(input_name) = captured_input {
                // Route the captured key into the matching draft field.
                if let Some(temp_config) = &mut self.temp_config {
                    match self.key_capture_mode {
                        KeyCaptureMode::ToggleKey => {
                            temp_config.switch_key = input_name.clone();
                        }
                        KeyCaptureMode::SequenceFinalizeKey => {
                            temp_config.sequence_finalize_key = input_name.clone();
                        }
                        KeyCaptureMode::MappingTrigger(idx) => {
                            if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                if mapping.is_sequence_trigger() {
                                    if let Some(seq_str) = &mapping.trigger_sequence {
                                        if seq_str.is_empty() {
                                            mapping.trigger_sequence = Some(input_name.clone());
                                            mapping.trigger_key = input_name.clone();
                                        } else {
                                            let mut new_seq = String::with_capacity(
                                                seq_str.len() + 1 + input_name.len(),
                                            );
                                            new_seq.push_str(seq_str);
                                            new_seq.push(',');
                                            new_seq.push_str(&input_name);
                                            mapping.trigger_sequence = Some(new_seq);
                                        }
                                    } else {
                                        mapping.trigger_sequence = Some(input_name.clone());
                                        mapping.trigger_key = input_name.clone();
                                    }
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                } else {
                                    mapping.trigger_key = input_name.clone();
                                }
                            }
                        }
                        KeyCaptureMode::MappingTarget(idx) => {
                            if let Some(mapping) = temp_config.mappings.get_mut(idx) {
                                if mapping.target_mode == 2 {
                                    self.editing_target_seq_list.push(input_name.clone());
                                    self.capture_pressed_keys.clear();
                                    self.capture_initial_pressed = Self::poll_all_pressed_keys();
                                } else {
                                    mapping.add_target_key(input_name.clone());
                                }
                            }
                        }
                        KeyCaptureMode::NewMappingTrigger => {
                            if is_sequence_capture {
                                // Allow duplicates for combo moves like S→A→S→D.
                                self.sequence_capture_list.push(input_name);
                                self.capture_pressed_keys.clear();
                                self.capture_initial_pressed = Self::poll_all_pressed_keys();
                            } else {
                                self.new_mapping_trigger = input_name.clone();
                            }
                        }
                        KeyCaptureMode::NewMappingTarget => match self.new_mapping_target_mode {
                            0 => {
                                self.new_mapping_target = input_name.clone();
                                self.new_mapping_target_keys.clear();
                                self.new_mapping_target_keys.push(input_name);
                            }
                            1 => {
                                self.new_mapping_target = input_name.clone();
                                if !self.new_mapping_target_keys.contains(&input_name) {
                                    self.new_mapping_target_keys.push(input_name);
                                }
                            }
                            2 => {
                                // Sequence: allow duplicates and stay in capture mode.
                                self.target_sequence_capture_list.push(input_name.clone());
                                self.new_mapping_target_keys =
                                    self.target_sequence_capture_list.clone();
                                self.new_mapping_target = input_name;
                                self.capture_pressed_keys.clear();
                                self.capture_initial_pressed = Self::poll_all_pressed_keys();
                            }
                            _ => {}
                        },
                        KeyCaptureMode::None => {}
                    }
                }

                // Stay in capture mode for every sequence variant so the
                // next key adds without re-entering Capture.
                let is_existing_mapping_sequence =
                    if let KeyCaptureMode::MappingTrigger(idx) = self.key_capture_mode {
                        if let Some(temp_config) = &self.temp_config {
                            temp_config
                                .mappings
                                .get(idx)
                                .map(|m| m.is_sequence_trigger())
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                let is_existing_target_sequence =
                    if let KeyCaptureMode::MappingTarget(idx) = self.key_capture_mode {
                        if let Some(temp_config) = &self.temp_config {
                            temp_config
                                .mappings
                                .get(idx)
                                .map(|m| m.target_mode == 2)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                if !is_sequence_capture
                    && !is_existing_mapping_sequence
                    && !is_existing_target_sequence
                {
                    self.key_capture_mode = KeyCaptureMode::None;
                    self.capture_pressed_keys.clear();
                    self.app_state.set_raw_input_capture_mode(false);
                }
                self.just_captured_input = false;
            } else {
                // In capture mode with no input yet. Clear the flag on the next frame.
                if self.just_captured_input {
                    self.just_captured_input = false;
                }
            }
        } else {
            // Not in capture mode. Reset transient state.
            self.capture_pressed_keys.clear();
            self.just_captured_input = false;
        }
    }

    /// Returns true when NumLock is currently enabled.
    #[inline(always)]
    fn is_num_lock_on() -> bool {
        unsafe { (GetKeyState(VK_NUMLOCK.0 as i32) & 0x0001) != 0 }
    }

    /// Detects a freshly pressed finalize-sequence key.
    ///
    /// The key itself is user-configurable via
    /// `AppConfig::sequence_finalize_key`. This reads the cached VK from
    /// `AppState`. Returns `false` when the key was already held when
    /// capture began. That filter avoids treating a stale press as a
    /// finalize signal, for example a leftover press from clicking Done
    /// or from a combo pressed before entering capture.
    ///
    /// When the configured VK is a numpad key and NumLock is off, Windows
    /// translates the physical press into a nav-cluster VK instead. The
    /// check falls back to the paired nav VK so the finalize hotkey keeps
    /// working regardless of NumLock state, mirroring the remap in
    /// `poll_all_pressed_keys`.
    #[inline]
    pub(super) fn is_sequence_finalize_pressed(&self) -> bool {
        let vk = self
            .app_state
            .sequence_finalize_vk
            .load(std::sync::atomic::Ordering::Relaxed);
        if vk == 0 {
            return false;
        }
        if self.capture_initial_pressed.contains(&vk) {
            return false;
        }
        if unsafe { GetAsyncKeyState(vk as i32) < 0 } {
            return true;
        }
        // Numpad fallback. With NumLock off, the physical press lands on
        // the nav-cluster VK instead of VK_NUMPAD*.
        if !Self::is_num_lock_on()
            && let Some(nav_vk) = numpad::to_nav_vk(vk)
            && !self.capture_initial_pressed.contains(&nav_vk)
        {
            return unsafe { GetAsyncKeyState(nav_vk as i32) < 0 };
        }
        false
    }

    #[inline]
    pub(crate) fn poll_all_pressed_keys() -> HashSet<u32> {
        const ALL_VK_CODES: &[u32] = &[
            0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C, 0x20, 0x0D, 0x09, 0x1B, 0x08, 0x2E,
            0x2D, 0x24, 0x23, 0x21, 0x22, 0x26, 0x28, 0x25, 0x27, 0x14, 0x90, 0x91, 0x13, 0x2C,
            0x0C, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0,
            0xDB, 0xDC, 0xDD, 0xDE, 0xDF, 0xE2, 0x01, 0x02, 0x04, 0x05, 0x06,
        ];

        let mut pressed_keys = HashSet::with_capacity(16);
        let numlock_off = !Self::is_num_lock_on();

        unsafe {
            for vk in 0x30u32..=0x5A {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            for vk in 0x60u32..=0x87 {
                if GetAsyncKeyState(vk as i32) < 0 {
                    pressed_keys.insert(vk);
                }
            }

            for &vk in ALL_VK_CODES {
                if GetAsyncKeyState(vk as i32) < 0 {
                    let mapped = if numlock_off {
                        numpad::from_nav_vk(vk)
                    } else {
                        vk
                    };
                    pressed_keys.insert(mapped);
                }
            }
        }

        pressed_keys
    }

    #[inline]
    pub(crate) fn format_captured_keys(vk_codes: &HashSet<u32>) -> Option<String> {
        if vk_codes.is_empty() {
            return None;
        }

        let mut modifiers: SmallVec<[u32; 8]> = SmallVec::new();
        let mut main_key: Option<u32> = None;

        for &vk in vk_codes {
            if matches!(vk, 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0x5B | 0x5C) {
                modifiers.push(vk);
            } else if main_key.is_none() {
                main_key = Some(vk);
            }
        }

        let mut result = String::with_capacity(64);
        let mut first = true;

        for &vk in &modifiers {
            if !first {
                result.push('+');
            }
            first = false;
            result.push_str(match vk {
                0xA2 => "LCTRL",
                0xA3 => "RCTRL",
                0xA4 => "LALT",
                0xA5 => "RALT",
                0xA0 => "LSHIFT",
                0xA1 => "RSHIFT",
                0x5B => "LWIN",
                0x5C => "RWIN",
                _ => continue,
            });
        }

        if let Some(vk) = main_key
            && let Some(name) = Self::vk_to_string(vk)
        {
            if !first {
                result.push('+');
            }
            result.push_str(&name);
            first = false;
        }

        if !first { Some(result) } else { None }
    }

    /// Converts VK code to key name string
    #[inline]
    pub(super) fn vk_to_string(vk: u32) -> Option<String> {
        match vk {
            // A-Z
            0x41..=0x5A => Some(char::from_u32(vk).unwrap().to_string()),
            // 0-9
            0x30..=0x39 => Some(char::from_u32(vk).unwrap().to_string()),
            // Numpad 0-9
            0x60..=0x69 => Some(format!("NUMPAD{}", vk - 0x60)),
            // F1-F24
            0x70..=0x87 => Some(format!("F{}", vk - 0x70 + 1)),
            // Navigation keys
            0x20 => Some("SPACE".to_string()),
            0x0D => Some("RETURN".to_string()),
            0x09 => Some("TAB".to_string()),
            0x1B => Some("ESCAPE".to_string()),
            0x08 => Some("BACK".to_string()),
            0x2E => Some("DELETE".to_string()),
            0x2D => Some("INSERT".to_string()),
            0x24 => Some("HOME".to_string()),
            0x23 => Some("END".to_string()),
            0x21 => Some("PAGEUP".to_string()),
            0x22 => Some("PAGEDOWN".to_string()),
            0x26 => Some("UP".to_string()),
            0x28 => Some("DOWN".to_string()),
            0x25 => Some("LEFT".to_string()),
            0x27 => Some("RIGHT".to_string()),
            // Lock and special keys
            0x14 => Some("CAPITAL".to_string()),
            0x90 => Some("NUMLOCK".to_string()),
            0x91 => Some("SCROLL".to_string()),
            0x13 => Some("PAUSE".to_string()),
            0x2C => Some("SNAPSHOT".to_string()),
            // Numpad operators
            0x6A => Some("MULTIPLY".to_string()),
            0x6B => Some("ADD".to_string()),
            0x6C => Some("SEPARATOR".to_string()),
            0x6D => Some("SUBTRACT".to_string()),
            0x6E => Some("DECIMAL".to_string()),
            0x6F => Some("DIVIDE".to_string()),
            // OEM keys
            0xBA => Some("OEM_1".to_string()),
            0xBB => Some("OEM_PLUS".to_string()),
            0xBC => Some("OEM_COMMA".to_string()),
            0xBD => Some("OEM_MINUS".to_string()),
            0xBE => Some("OEM_PERIOD".to_string()),
            0xBF => Some("OEM_2".to_string()),
            0xC0 => Some("OEM_3".to_string()),
            0xDB => Some("OEM_4".to_string()),
            0xDC => Some("OEM_5".to_string()),
            0xDD => Some("OEM_6".to_string()),
            0xDE => Some("OEM_7".to_string()),
            0xDF => Some("OEM_8".to_string()),
            0xE2 => Some("OEM_102".to_string()),
            // Modifiers
            0xA2 => Some("LCTRL".to_string()),
            0xA3 => Some("RCTRL".to_string()),
            0xA4 => Some("LALT".to_string()),
            0xA5 => Some("RALT".to_string()),
            0xA0 => Some("LSHIFT".to_string()),
            0xA1 => Some("RSHIFT".to_string()),
            0x5B => Some("LWIN".to_string()),
            0x5C => Some("RWIN".to_string()),
            // Mouse buttons
            0x01 => Some("LBUTTON".to_string()),
            0x02 => Some("RBUTTON".to_string()),
            0x04 => Some("MBUTTON".to_string()),
            0x05 => Some("XBUTTON1".to_string()),
            0x06 => Some("XBUTTON2".to_string()),
            _ => None,
        }
    }
}
