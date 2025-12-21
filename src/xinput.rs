//! XInput API integration for Xbox controllers.
//!
//! Provides support for Xbox 360, Xbox One, and Xbox Series controllers.
//! Handles button input, analog sticks, and triggers with deadzone filtering.

use crate::input_ownership::{DeviceOwnership, InputSource};
use crate::state::{AppState, DeviceType, InputDevice, InputEvent};
use crate::util::{likely, unlikely};
use smallvec::SmallVec;
use std::sync::Arc;
use windows::Win32::UI::Input::XboxController::*;

/// XInput gamepad VID (Microsoft)
const XBOX_VID: u16 = 0x045E;

/// Analog stick deadzone threshold.
const STICK_DEADZONE: i16 = 7849; // ~24% of max range

/// Trigger deadzone threshold.
const TRIGGER_THRESHOLD: u8 = 30;

/// Maximum number of simultaneous inputs (buttons + sticks + triggers)
const MAX_INPUTS: usize = 20;

/// Maximum number of capture frames to record
const CAPTURE_FRAMES: usize = 16;

/// Captured input frame with timestamp.
#[derive(Clone)]
struct CaptureFrame {
    button_id: u64,
    timestamp: u64,
    raw_inputs: SmallVec<[u32; 12]>,
}

impl CaptureFrame {
    #[inline(always)]
    fn new() -> Self {
        Self {
            button_id: 0,
            timestamp: 0,
            raw_inputs: SmallVec::new(),
        }
    }
}

/// XInput device state for change detection.
#[derive(Clone)]
struct XInputDeviceState {
    packet_number: u32,
    last_state: XINPUT_GAMEPAD,
    last_button_id: Option<u64>,
    vid_pid: (u16, u16),
    capture_frames: [CaptureFrame; CAPTURE_FRAMES],
    capture_frame_count: u8,
    active_combo: Option<Vec<u32>>,
}

/// XInput handler for Xbox controller input.
pub struct XInputHandler {
    state: Arc<AppState>,
    ownership: DeviceOwnership,
    device_states: [Option<XInputDeviceState>; XUSER_MAX_COUNT as usize],
}

impl XInputHandler {
    /// Creates a new XInput handler.
    pub fn new(state: Arc<AppState>, ownership: DeviceOwnership) -> Self {
        Self {
            state,
            ownership,
            device_states: [None, None, None, None],
        }
    }

    /// Initializes XInput devices and claims ownership.
    pub fn initialize(&mut self) {
        for user_index in 0..XUSER_MAX_COUNT {
            let mut state = XINPUT_STATE::default();
            let result = unsafe { XInputGetState(user_index, &mut state) };
            if result == 0 {
                // Device connected
                let vid_pid = Self::detect_vid_pid_static(user_index).unwrap_or((XBOX_VID, 0x028E));
                if self
                    .ownership
                    .claim_device(vid_pid, InputSource::XInput(user_index))
                {
                    self.device_states[user_index as usize] = Some(XInputDeviceState {
                        packet_number: state.dwPacketNumber,
                        last_state: state.Gamepad,
                        last_button_id: None,
                        vid_pid,
                        capture_frames: std::array::from_fn(|_| CaptureFrame::new()),
                        capture_frame_count: 0,
                        active_combo: None,
                    });

                    // Register device display info
                    let stable_device_id = Self::hash_vid_pid_static(vid_pid) as u64;
                    let display_info = crate::rawinput::DeviceDisplayInfo {
                        vendor_id: vid_pid.0,
                        product_id: vid_pid.1,
                        serial_number: None, // XInput does not provide serial numbers
                    };
                    crate::rawinput::register_device_display_info(
                        stable_device_id & 0xFFFFFFFF,
                        display_info,
                    );
                }
            }
        }
    }

    /// Polls all XInput devices for state changes.
    pub fn poll(&mut self) {
        // Continue polling in capture mode even when paused
        let is_capturing = self.state.is_raw_input_capture_active();
        let is_paused = self.state.is_paused();

        if is_paused && !is_capturing {
            return;
        }

        for user_index in 0..XUSER_MAX_COUNT {
            self.poll_device(user_index);
        }
    }

    /// Polls a single XInput device.
    #[inline]
    fn poll_device(&mut self, user_index: u32) {
        let mut state = XINPUT_STATE::default();

        match unsafe { XInputGetState(user_index, &mut state) } {
            0 => {
                let idx = user_index as usize;

                let needs_update = if let Some(device_state) = &self.device_states[idx] {
                    state.dwPacketNumber != device_state.packet_number
                } else {
                    false
                };

                if likely(needs_update) {
                    if let Some(device_state) = &mut self.device_states[idx] {
                        let vid_pid = device_state.vid_pid;
                        let stable_device_id = Self::hash_vid_pid_static(vid_pid) as u64;
                        let gamepad = state.Gamepad;

                        let mut current_inputs = SmallVec::<[u32; MAX_INPUTS]>::new();

                        Self::check_buttons_fast(&gamepad, &mut current_inputs);
                        Self::check_analog_sticks_fast(&gamepad, &mut current_inputs);
                        Self::check_triggers_fast(&gamepad, &mut current_inputs);

                        let current_button_id = if unlikely(!current_inputs.is_empty()) {
                            Some(Self::hash_inputs_fast(&current_inputs, stable_device_id))
                        } else {
                            None
                        };

                        let button_id_changed = current_button_id != device_state.last_button_id;

                        if unlikely(button_id_changed) {
                            device_state.last_button_id = current_button_id;
                        }

                        device_state.packet_number = state.dwPacketNumber;
                        device_state.last_state = gamepad;

                        if unlikely(button_id_changed) {
                            let is_capturing = self.state.is_raw_input_capture_active();

                            if unlikely(is_capturing) {
                                // Capture mode: supports MostSustained and LastStable
                                let capture_mode = self.state.get_xinput_capture_mode();

                                if let Some(current_id) = current_button_id {
                                    if (device_state.capture_frame_count as usize) < CAPTURE_FRAMES
                                    {
                                        let idx = device_state.capture_frame_count as usize;
                                        let raw_inputs: SmallVec<[u32; 12]> =
                                            current_inputs.iter().copied().collect();

                                        device_state.capture_frames[idx] = CaptureFrame {
                                            button_id: current_id,
                                            timestamp: std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis()
                                                as u64,
                                            raw_inputs,
                                        };
                                        device_state.capture_frame_count += 1;
                                    }
                                } else if device_state.capture_frame_count > 0 {
                                    // Released: select best frame based on capture mode
                                    let best_frame_idx = match capture_mode {
                                        crate::config::XInputCaptureMode::MostSustained => {
                                            Self::get_most_sustained_frame_idx(
                                                &device_state.capture_frames,
                                                device_state.capture_frame_count,
                                            )
                                        }
                                        crate::config::XInputCaptureMode::LastStable => {
                                            Self::get_last_stable_frame_idx(
                                                &device_state.capture_frames,
                                                device_state.capture_frame_count,
                                            )
                                        }
                                        crate::config::XInputCaptureMode::DiagonalPriority => {
                                            Self::get_diagonal_priority_frame_idx(
                                                &device_state.capture_frames,
                                                device_state.capture_frame_count,
                                            )
                                        }
                                    };

                                    if let Some(idx) = best_frame_idx {
                                        let frame = &device_state.capture_frames[idx];
                                        let device_type = DeviceType::Gamepad(vid_pid.0);

                                        // Extract button IDs from frame
                                        let button_ids: Vec<u32> =
                                            frame.raw_inputs.iter().copied().collect();

                                        let device = InputDevice::XInputCombo {
                                            device_type,
                                            button_ids,
                                        };
                                        let _ =
                                            self.state.get_raw_input_capture_sender().send(device);
                                    }

                                    // Clear capture state
                                    device_state.capture_frame_count = 0;
                                }
                            } else if let Some(pool) = self.state.get_worker_pool() {
                                let device_type = DeviceType::Gamepad(vid_pid.0);
                                let button_ids: Vec<u32> = current_inputs.iter().copied().collect();

                                // Check if combo state changed
                                let combo_changed =
                                    match (&device_state.active_combo, !button_ids.is_empty()) {
                                        (Some(active), true) => active != &button_ids,
                                        (Some(_), false) => true, // Released
                                        (None, true) => true,     // New press
                                        (None, false) => false,   // Still released
                                    };

                                if combo_changed {
                                    // First, send release event for previous combo if it exists
                                    if let Some(active_button_ids) =
                                        device_state.active_combo.take()
                                    {
                                        let release_device = InputDevice::XInputCombo {
                                            device_type,
                                            button_ids: active_button_ids,
                                        };
                                        if self.state.get_input_mapping(&release_device).is_some() {
                                            pool.dispatch(InputEvent::Released(release_device));
                                        }
                                    }

                                    // Then, send press event for new combo if buttons are pressed
                                    if !button_ids.is_empty() {
                                        let press_device = InputDevice::XInputCombo {
                                            device_type,
                                            button_ids: button_ids.clone(),
                                        };
                                        if self.state.get_input_mapping(&press_device).is_some() {
                                            pool.dispatch(InputEvent::Pressed(press_device));
                                            device_state.active_combo = Some(button_ids);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if self.device_states[idx].is_none() {
                    let vid_pid =
                        Self::detect_vid_pid_static(user_index).unwrap_or((XBOX_VID, 0x028E));

                    if self
                        .ownership
                        .claim_device(vid_pid, InputSource::XInput(user_index))
                    {
                        self.device_states[idx] = Some(XInputDeviceState {
                            packet_number: state.dwPacketNumber,
                            last_state: state.Gamepad,
                            last_button_id: None,
                            vid_pid,
                            capture_frames: std::array::from_fn(|_| CaptureFrame::new()),
                            capture_frame_count: 0,
                            active_combo: None,
                        });
                    }
                }
            }
            _ => {
                // Device disconnected
                if let Some(device_state) = self.device_states[user_index as usize].take() {
                    self.ownership.release_device(device_state.vid_pid);
                }
            }
        }
    }

    /// Checks button states and records active buttons.
    #[inline(always)]
    fn check_buttons_fast(gamepad: &XINPUT_GAMEPAD, active: &mut SmallVec<[u32; MAX_INPUTS]>) {
        let buttons = gamepad.wButtons.0;

        const BUTTON_MAP: [(u16, u32); 14] = [
            (0x0001, 0x01), // DPAD_UP
            (0x0002, 0x02), // DPAD_DOWN
            (0x0004, 0x03), // DPAD_LEFT
            (0x0008, 0x04), // DPAD_RIGHT
            (0x0010, 0x05), // START
            (0x0020, 0x06), // BACK
            (0x0040, 0x07), // LEFT_THUMB
            (0x0080, 0x08), // RIGHT_THUMB
            (0x0100, 0x09), // LEFT_SHOULDER
            (0x0200, 0x0A), // RIGHT_SHOULDER
            (0x1000, 0x0B), // A
            (0x2000, 0x0C), // B
            (0x4000, 0x0D), // X
            (0x8000, 0x0E), // Y
        ];

        for &(mask, id) in &BUTTON_MAP {
            if unlikely(buttons & mask != 0) {
                active.push(id);
            }
        }
    }

    /// Checks analog stick states and records active directions.
    #[inline(always)]
    fn check_analog_sticks_fast(
        gamepad: &XINPUT_GAMEPAD,
        active: &mut SmallVec<[u32; MAX_INPUTS]>,
    ) {
        // Left stick X-axis
        let lx = gamepad.sThumbLX;
        if unlikely(lx > STICK_DEADZONE) {
            active.push(0x10); // Left stick right
        } else if unlikely(lx < -STICK_DEADZONE) {
            active.push(0x11); // Left stick left
        }

        // Left stick Y-axis
        let ly = gamepad.sThumbLY;
        if unlikely(ly > STICK_DEADZONE) {
            active.push(0x12); // Left stick up
        } else if unlikely(ly < -STICK_DEADZONE) {
            active.push(0x13); // Left stick down
        }

        // Right stick X-axis
        let rx = gamepad.sThumbRX;
        if unlikely(rx > STICK_DEADZONE) {
            active.push(0x14); // Right stick right
        } else if unlikely(rx < -STICK_DEADZONE) {
            active.push(0x15); // Right stick left
        }

        // Right stick Y-axis
        let ry = gamepad.sThumbRY;
        if unlikely(ry > STICK_DEADZONE) {
            active.push(0x16); // Right stick up
        } else if unlikely(ry < -STICK_DEADZONE) {
            active.push(0x17); // Right stick down
        }
    }

    /// Checks trigger states and records active triggers.
    #[inline(always)]
    fn check_triggers_fast(gamepad: &XINPUT_GAMEPAD, active: &mut SmallVec<[u32; MAX_INPUTS]>) {
        if unlikely(gamepad.bLeftTrigger > TRIGGER_THRESHOLD) {
            active.push(0x18); // Left trigger
        }
        if unlikely(gamepad.bRightTrigger > TRIGGER_THRESHOLD) {
            active.push(0x19); // Right trigger
        }
    }

    /// Checks if the given inputs form a diagonal direction.
    #[inline(always)]
    fn is_diagonal_direction(inputs: &[u32]) -> bool {
        if inputs.len() != 2 {
            return false;
        }

        let (a, b) = (inputs[0], inputs[1]);

        // Left stick diagonal: horizontal (0x10/0x11) + vertical (0x12/0x13)
        let left_stick = matches!(
            (a, b),
            (0x10, 0x12)
                | (0x10, 0x13)
                | (0x11, 0x12)
                | (0x11, 0x13)
                | (0x12, 0x10)
                | (0x12, 0x11)
                | (0x13, 0x10)
                | (0x13, 0x11)
        );

        // Right stick diagonal: horizontal (0x14/0x15) + vertical (0x16/0x17)
        let right_stick = matches!(
            (a, b),
            (0x14, 0x16)
                | (0x14, 0x17)
                | (0x15, 0x16)
                | (0x15, 0x17)
                | (0x16, 0x14)
                | (0x16, 0x15)
                | (0x17, 0x14)
                | (0x17, 0x15)
        );

        // D-Pad diagonal: horizontal (0x03/0x04) + vertical (0x01/0x02)
        let dpad = matches!(
            (a, b),
            (0x01, 0x03)
                | (0x01, 0x04)
                | (0x02, 0x03)
                | (0x02, 0x04)
                | (0x03, 0x01)
                | (0x03, 0x02)
                | (0x04, 0x01)
                | (0x04, 0x02)
        );

        left_stick || right_stick || dpad
    }

    /// Checks if an input ID is a direction (D-Pad or stick).
    #[inline(always)]
    pub fn is_direction_input(input_id: u32) -> bool {
        matches!(
            input_id,
            0x01..=0x04 |  // D-Pad
            0x10..=0x17    // Left/Right sticks
        )
    }

    /// Converts input ID to readable button name.
    #[inline(always)]
    pub fn input_id_to_name(input_id: u32) -> &'static str {
        match input_id {
            // D-Pad
            0x01 => "DPad_Up",
            0x02 => "DPad_Down",
            0x03 => "DPad_Left",
            0x04 => "DPad_Right",
            // Buttons
            0x05 => "Start",
            0x06 => "Back",
            0x07 => "LS_Click",
            0x08 => "RS_Click",
            0x09 => "LB",
            0x0A => "RB",
            0x0B => "A",
            0x0C => "B",
            0x0D => "X",
            0x0E => "Y",
            // Left Stick
            0x10 => "LS_Right",
            0x11 => "LS_Left",
            0x12 => "LS_Up",
            0x13 => "LS_Down",
            // Right Stick
            0x14 => "RS_Right",
            0x15 => "RS_Left",
            0x16 => "RS_Up",
            0x17 => "RS_Down",
            // Triggers
            0x18 => "LT",
            0x19 => "RT",
            _ => "Unknown",
        }
    }

    /// Converts button name to input ID.
    #[inline(always)]
    pub fn name_to_input_id(name: &str) -> Option<u32> {
        match name {
            // D-Pad
            "DPad_Up" | "DPAD_UP" => Some(0x01),
            "DPad_Down" | "DPAD_DOWN" => Some(0x02),
            "DPad_Left" | "DPAD_LEFT" => Some(0x03),
            "DPad_Right" | "DPAD_RIGHT" => Some(0x04),
            // Buttons
            "Start" | "START" => Some(0x05),
            "Back" | "BACK" => Some(0x06),
            "LS_Click" | "LS_CLICK" => Some(0x07),
            "RS_Click" | "RS_CLICK" => Some(0x08),
            "LB" => Some(0x09),
            "RB" => Some(0x0A),
            "A" => Some(0x0B),
            "B" => Some(0x0C),
            "X" => Some(0x0D),
            "Y" => Some(0x0E),
            // Left Stick
            "LS_Right" | "LS_RIGHT" => Some(0x10),
            "LS_Left" | "LS_LEFT" => Some(0x11),
            "LS_Up" | "LS_UP" => Some(0x12),
            "LS_Down" | "LS_DOWN" => Some(0x13),
            // Right Stick
            "RS_Right" | "RS_RIGHT" => Some(0x14),
            "RS_Left" | "RS_LEFT" => Some(0x15),
            "RS_Up" | "RS_UP" => Some(0x16),
            "RS_Down" | "RS_DOWN" => Some(0x17),
            // Triggers
            "LT" => Some(0x18),
            "RT" => Some(0x19),
            // Diagonal combinations
            "LS_RightUp" | "LS_RIGHTUP" => None, // Special: needs to return [0x10, 0x12]
            "LS_RightDown" | "LS_RIGHTDOWN" => None,
            "LS_LeftUp" | "LS_LEFTUP" => None,
            "LS_LeftDown" | "LS_LEFTDOWN" => None,
            "RS_RightUp" | "RS_RIGHTUP" => None,
            "RS_RightDown" | "RS_RIGHTDOWN" => None,
            "RS_LeftUp" | "RS_LEFTUP" => None,
            "RS_LeftDown" | "RS_LEFTDOWN" => None,
            "DPad_UpRight" | "DPAD_UPRIGHT" => None,
            "DPad_UpLeft" | "DPAD_UPLEFT" => None,
            "DPad_DownRight" | "DPAD_DOWNRIGHT" => None,
            "DPad_DownLeft" | "DPAD_DOWNLEFT" => None,
            _ => None,
        }
    }

    /// Parses diagonal name to input IDs (always returns exactly 2 IDs).
    #[inline(always)]
    pub fn parse_diagonal_name(name: &str) -> Option<[u32; 2]> {
        match name {
            // Left Stick diagonals
            "LS_RightUp" | "LS_RIGHTUP" => Some([0x10, 0x12]),
            "LS_RightDown" | "LS_RIGHTDOWN" => Some([0x10, 0x13]),
            "LS_LeftUp" | "LS_LEFTUP" => Some([0x11, 0x12]),
            "LS_LeftDown" | "LS_LEFTDOWN" => Some([0x11, 0x13]),
            // Right Stick diagonals
            "RS_RightUp" | "RS_RIGHTUP" => Some([0x14, 0x16]),
            "RS_RightDown" | "RS_RIGHTDOWN" => Some([0x14, 0x17]),
            "RS_LeftUp" | "RS_LEFTUP" => Some([0x15, 0x16]),
            "RS_LeftDown" | "RS_LEFTDOWN" => Some([0x15, 0x17]),
            // D-Pad diagonals
            "DPad_UpRight" | "DPAD_UPRIGHT" => Some([0x01, 0x04]),
            "DPad_UpLeft" | "DPAD_UPLEFT" => Some([0x01, 0x03]),
            "DPad_DownRight" | "DPAD_DOWNRIGHT" => Some([0x02, 0x04]),
            "DPad_DownLeft" | "DPAD_DOWNLEFT" => Some([0x02, 0x03]),
            _ => None,
        }
    }

    /// Checks if two direction inputs form a diagonal.
    #[inline(always)]
    pub fn try_combine_diagonal(inputs: &[u32]) -> Option<&'static str> {
        if inputs.len() != 2 {
            return None;
        }

        let (a, b) = (inputs[0], inputs[1]);

        // Left stick diagonals
        match (a, b) {
            (0x10, 0x12) | (0x12, 0x10) => return Some("LS_RightUp"),
            (0x10, 0x13) | (0x13, 0x10) => return Some("LS_RightDown"),
            (0x11, 0x12) | (0x12, 0x11) => return Some("LS_LeftUp"),
            (0x11, 0x13) | (0x13, 0x11) => return Some("LS_LeftDown"),
            _ => {}
        }

        // Right stick diagonals
        match (a, b) {
            (0x14, 0x16) | (0x16, 0x14) => return Some("RS_RightUp"),
            (0x14, 0x17) | (0x17, 0x14) => return Some("RS_RightDown"),
            (0x15, 0x16) | (0x16, 0x15) => return Some("RS_LeftUp"),
            (0x15, 0x17) | (0x17, 0x15) => return Some("RS_LeftDown"),
            _ => {}
        }

        // D-Pad diagonals
        match (a, b) {
            (0x01, 0x04) | (0x04, 0x01) => return Some("DPad_UpRight"),
            (0x01, 0x03) | (0x03, 0x01) => return Some("DPad_UpLeft"),
            (0x02, 0x04) | (0x04, 0x02) => return Some("DPad_DownRight"),
            (0x02, 0x03) | (0x03, 0x02) => return Some("DPad_DownLeft"),
            _ => {}
        }

        None
    }

    /// Finds the most sustained frame index from captured frames.
    /// Prioritizes frames with more inputs (diagonal directions), then duration.
    #[inline(always)]
    fn get_most_sustained_frame_idx(
        frames: &[CaptureFrame; CAPTURE_FRAMES],
        frame_count: u8,
    ) -> Option<usize> {
        if frame_count == 0 {
            return None;
        }

        if frame_count == 1 {
            return Some(0);
        }

        let mut best_idx = 0usize;
        let mut max_input_count = frames[0].raw_inputs.len();
        let mut max_duration = 0u64;

        let mut i = 0;
        while i < frame_count as usize {
            let current_id = frames[i].button_id;
            let current_input_count = frames[i].raw_inputs.len();
            let start_time = frames[i].timestamp;

            // Find consecutive frames with same button_id
            let mut j = i;
            while j + 1 < frame_count as usize && frames[j + 1].button_id == current_id {
                j += 1;
            }

            // Calculate duration
            let end_time = frames[j].timestamp;
            let duration = end_time.saturating_sub(start_time);

            // Prioritize input_count (diagonal has more inputs), then duration
            if current_input_count > max_input_count
                || (current_input_count == max_input_count && duration > max_duration)
            {
                max_input_count = current_input_count;
                max_duration = duration;
                best_idx = i;
            }

            i = j + 1;
        }

        Some(best_idx)
    }

    /// Finds the last stable frame index from captured frames.
    /// Prioritizes frames with more inputs (diagonal directions) when comparing stable sequences.
    #[inline(always)]
    fn get_last_stable_frame_idx(
        frames: &[CaptureFrame; CAPTURE_FRAMES],
        frame_count: u8,
    ) -> Option<usize> {
        if frame_count == 0 {
            return None;
        }

        if frame_count == 1 {
            return Some(0);
        }

        // Find the last sequence of at least 2 consecutive frames with same button_id
        let mut i = (frame_count as usize).saturating_sub(1);
        let last_button_id = frames[i].button_id;
        let last_input_count = frames[i].raw_inputs.len();

        // Count how many consecutive frames at the end have the same button_id
        let mut stable_count = 1;
        while i > 0 && frames[i - 1].button_id == last_button_id {
            i -= 1;
            stable_count += 1;
        }

        // If last state was held for at least 2 frames, it's stable
        if stable_count >= 2 {
            // Check if there's a previous stable sequence with more inputs
            if i > 0 {
                let mut prev_i = i.saturating_sub(1);
                let prev_button_id = frames[prev_i].button_id;
                let prev_input_count = frames[prev_i].raw_inputs.len();
                let mut prev_stable_count = 1;

                while prev_i > 0 && frames[prev_i - 1].button_id == prev_button_id {
                    prev_i -= 1;
                    prev_stable_count += 1;
                }

                // Prefer previous sequence if it has more inputs and is also stable
                if prev_stable_count >= 2 && prev_input_count > last_input_count {
                    return Some(prev_i);
                }
            }
            Some(i)
        } else {
            // Otherwise, look for the previous stable sequence
            let mut prev_i = i.saturating_sub(1);
            if prev_i == 0 {
                return Some(0);
            }

            let prev_button_id = frames[prev_i].button_id;
            let mut prev_stable_count = 1;
            while prev_i > 0 && frames[prev_i - 1].button_id == prev_button_id {
                prev_i -= 1;
                prev_stable_count += 1;
            }

            if prev_stable_count >= 2 {
                Some(prev_i)
            } else {
                // No stable sequence found, return most recent
                Some(i)
            }
        }
    }

    /// Diagonal priority capture: returns best frame index.
    #[inline(always)]
    fn get_diagonal_priority_frame_idx(
        frames: &[CaptureFrame; CAPTURE_FRAMES],
        frame_count: u8,
    ) -> Option<usize> {
        if frame_count == 0 {
            return None;
        }

        // Find the best frame: prioritize frames with both direction and other buttons
        let mut best_frame_idx: Option<usize> = None;
        let mut best_priority = 0u8;

        for i in 0..frame_count as usize {
            let frame = &frames[i];
            let inputs = &frame.raw_inputs[..];

            let mut has_direction = false;
            let mut has_diagonal = false;
            let mut has_other = false;

            let mut dir_inputs = SmallVec::<[u32; 4]>::new();
            for &input_id in inputs {
                if input_id == 0 {
                    break;
                }
                if Self::is_direction_input(input_id) {
                    has_direction = true;
                    dir_inputs.push(input_id);
                } else {
                    has_other = true;
                }
            }

            if dir_inputs.len() == 2 && Self::is_diagonal_direction(&dir_inputs) {
                has_diagonal = true;
            }

            let priority = match (has_diagonal, has_direction, has_other) {
                (true, _, true) => 5,
                (false, true, true) => 4,
                (true, _, false) => 3,
                (false, true, false) => 2,
                (false, false, true) => 1,
                _ => 0,
            };

            let should_select = if let Some(current_idx) = best_frame_idx {
                let current_frame = &frames[current_idx];
                priority > best_priority
                    || (priority == best_priority
                        && frame.raw_inputs.len() > current_frame.raw_inputs.len())
                    || (priority == best_priority
                        && frame.raw_inputs.len() == current_frame.raw_inputs.len())
            } else {
                true
            };

            if should_select {
                best_priority = priority;
                best_frame_idx = Some(i);
            }
        }

        best_frame_idx
    }

    /// Generates stable device ID from VID:PID.
    #[inline(always)]
    fn hash_vid_pid_static(vid_pid: (u16, u16)) -> u32 {
        use crate::util::{fnv1a_hash_u32, fnv32};

        let mut hash = fnv32::OFFSET_BASIS;
        hash = fnv1a_hash_u32(hash, vid_pid.0 as u32);
        hash = fnv1a_hash_u32(hash, vid_pid.1 as u32);
        hash
    }

    /// Generates button ID from input combination.
    #[inline(always)]
    fn hash_inputs_fast(inputs: &[u32], stable_device_id: u64) -> u64 {
        use crate::util::{fnv1a_hash_u32, fnv32};

        let mut hash = fnv32::OFFSET_BASIS;

        match inputs.len() {
            0 => {}
            1 => hash = fnv1a_hash_u32(hash, inputs[0]),
            2 => {
                hash = fnv1a_hash_u32(hash, inputs[0]);
                hash = fnv1a_hash_u32(hash, inputs[1]);
            }
            3 => {
                hash = fnv1a_hash_u32(hash, inputs[0]);
                hash = fnv1a_hash_u32(hash, inputs[1]);
                hash = fnv1a_hash_u32(hash, inputs[2]);
            }
            _ => {
                // General case for rare multi-input combinations
                for &input in inputs {
                    hash = fnv1a_hash_u32(hash, input);
                }
            }
        }

        // Combine: [32-bit stable_device_id][32-bit input_hash]
        ((stable_device_id & 0xFFFFFFFF) << 32) | (hash as u64)
    }

    /// Attempts to detect VID:PID for an XInput device.
    fn detect_vid_pid_static(user_index: u32) -> Option<(u16, u16)> {
        // Try to get capabilities (contains subtype info)
        let mut caps = XINPUT_CAPABILITIES::default();
        if unsafe { XInputGetCapabilities(user_index, XINPUT_FLAG_GAMEPAD, &mut caps) } == 0 {
            // Map subtype to known VID:PID
            match caps.SubType {
                XINPUT_DEVSUBTYPE_GAMEPAD => Some((XBOX_VID, 0x028E)),
                XINPUT_DEVSUBTYPE_WHEEL => Some((XBOX_VID, 0x028F)),
                XINPUT_DEVSUBTYPE_ARCADE_STICK => Some((XBOX_VID, 0x02D1)),
                XINPUT_DEVSUBTYPE_FLIGHT_STICK => Some((XBOX_VID, 0x02DD)),
                XINPUT_DEVSUBTYPE_DANCE_PAD => Some((XBOX_VID, 0x02E3)),
                XINPUT_DEVSUBTYPE_GUITAR => Some((XBOX_VID, 0x02EA)),
                XINPUT_DEVSUBTYPE_DRUM_KIT => Some((XBOX_VID, 0x02FD)),
                _ => Some((XBOX_VID, 0x028E)), // Default to Xbox 360
            }
        } else {
            None
        }
    }

    /// Enumerates all connected XInput devices.
    pub fn enumerate_devices() -> Vec<crate::gui::device_manager_dialog::XInputDeviceInfo> {
        let mut devices = Vec::new();

        for user_index in 0..XUSER_MAX_COUNT {
            let mut state = XINPUT_STATE::default();
            if unsafe { XInputGetState(user_index, &mut state) } == 0 {
                let mut caps = XINPUT_CAPABILITIES::default();
                let _ =
                    unsafe { XInputGetCapabilities(user_index, XINPUT_FLAG_GAMEPAD, &mut caps) };

                let (vid, pid) =
                    Self::detect_vid_pid_static(user_index).unwrap_or((XBOX_VID, 0x028E));

                let device_type = match caps.SubType {
                    XINPUT_DEVSUBTYPE_GAMEPAD => "Xbox Controller",
                    XINPUT_DEVSUBTYPE_WHEEL => "Racing Wheel",
                    XINPUT_DEVSUBTYPE_ARCADE_STICK => "Arcade Stick",
                    XINPUT_DEVSUBTYPE_FLIGHT_STICK => "Flight Stick",
                    XINPUT_DEVSUBTYPE_DANCE_PAD => "Dance Pad",
                    XINPUT_DEVSUBTYPE_GUITAR => "Guitar Controller",
                    XINPUT_DEVSUBTYPE_DRUM_KIT => "Drum Kit",
                    _ => "XInput Device",
                };

                devices.push(crate::gui::device_manager_dialog::XInputDeviceInfo {
                    user_index,
                    vid,
                    pid,
                    device_type: device_type.to_string(),
                });
            }
        }

        devices
    }

    /// Sets vibration for an XInput device.
    ///
    /// # Arguments
    /// * `user_index` - XInput user index (0-3)
    /// * `left_motor` - Left motor speed (0-65535)
    /// * `right_motor` - Right motor speed (0-65535)
    ///
    /// # Returns
    /// `true` if vibration was set successfully, `false` otherwise
    pub fn set_vibration(user_index: u32, left_motor: u16, right_motor: u16) -> bool {
        use windows::Win32::UI::Input::XboxController::*;

        let vibration = XINPUT_VIBRATION {
            wLeftMotorSpeed: left_motor,
            wRightMotorSpeed: right_motor,
        };

        unsafe { XInputSetState(user_index, &vibration) == 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_vid_pid_static() {
        let vid_pid = (0x045E, 0x028E);
        let hash = XInputHandler::hash_vid_pid_static(vid_pid);

        assert_ne!(hash, 0);

        let same_hash = XInputHandler::hash_vid_pid_static(vid_pid);
        assert_eq!(hash, same_hash);

        let different_vid = (0x046D, 0x028E);
        let different_hash = XInputHandler::hash_vid_pid_static(different_vid);
        assert_ne!(hash, different_hash);
    }

    #[test]
    fn test_hash_inputs_fast_empty() {
        let stable_device_id = 0x12345678u64;
        let inputs: [u32; 0] = [];
        let hash = XInputHandler::hash_inputs_fast(&inputs, stable_device_id);

        let expected_upper = (stable_device_id & 0xFFFFFFFF) << 32;
        assert_eq!(hash & 0xFFFFFFFF00000000, expected_upper);
    }

    #[test]
    fn test_hash_inputs_fast_single() {
        let stable_device_id = 0x12345678u64;
        let inputs = [0x01u32];
        let hash = XInputHandler::hash_inputs_fast(&inputs, stable_device_id);

        assert_ne!(hash, 0);
        let same_hash = XInputHandler::hash_inputs_fast(&inputs, stable_device_id);
        assert_eq!(hash, same_hash);

        let different_input = [0x02u32];
        let different_hash = XInputHandler::hash_inputs_fast(&different_input, stable_device_id);
        assert_ne!(hash, different_hash);
    }

    #[test]
    fn test_hash_inputs_fast_multiple() {
        let stable_device_id = 0x12345678u64;
        let inputs = [0x01u32, 0x02u32];
        let hash = XInputHandler::hash_inputs_fast(&inputs, stable_device_id);

        let reversed_inputs = [0x02u32, 0x01u32];
        let reversed_hash = XInputHandler::hash_inputs_fast(&reversed_inputs, stable_device_id);
        assert_ne!(hash, reversed_hash);
    }

    #[test]
    fn test_hash_inputs_fast_many_inputs() {
        let stable_device_id = 0x12345678u64;
        let inputs = [0x01u32, 0x02u32, 0x03u32, 0x04u32, 0x05u32];
        let hash = XInputHandler::hash_inputs_fast(&inputs, stable_device_id);

        assert_ne!(hash, 0);

        let subset = [0x01u32, 0x02u32, 0x03u32];
        let subset_hash = XInputHandler::hash_inputs_fast(&subset, stable_device_id);
        assert_ne!(hash, subset_hash);
    }

    #[test]
    fn test_check_buttons_fast_no_buttons() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_buttons_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_check_buttons_fast_single_button() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x1000), // A button
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_buttons_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], 0x0B); // A button ID
    }

    #[test]
    fn test_check_buttons_fast_multiple_buttons() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x3001), // DPAD_UP + A + B
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_buttons_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 3);
        assert!(active.contains(&0x01)); // DPAD_UP
        assert!(active.contains(&0x0B)); // A
        assert!(active.contains(&0x0C)); // B
    }

    #[test]
    fn test_check_analog_sticks_fast_neutral() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_check_analog_sticks_fast_left_stick_right() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 20000,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], 0x10); // Left stick right
    }

    #[test]
    fn test_check_analog_sticks_fast_left_stick_up() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 20000,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], 0x12); // Left stick up
    }

    #[test]
    fn test_check_analog_sticks_fast_diagonal() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 20000,
            sThumbLY: 20000,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 2);
        assert!(active.contains(&0x10)); // Right
        assert!(active.contains(&0x12)); // Up
    }

    #[test]
    fn test_check_analog_sticks_fast_deadzone() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 1000,
            sThumbLY: 1000,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_check_analog_sticks_fast_right_stick() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: -20000,
            sThumbRY: -20000,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_analog_sticks_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 2);
        assert!(active.contains(&0x15)); // Right stick left
        assert!(active.contains(&0x17)); // Right stick down
    }

    #[test]
    fn test_check_triggers_fast_none() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_triggers_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_check_triggers_fast_left_trigger() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 200,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_triggers_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], 0x18); // Left trigger
    }

    #[test]
    fn test_check_triggers_fast_both_triggers() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 200,
            bRightTrigger: 200,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_triggers_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 2);
        assert!(active.contains(&0x18)); // Left trigger
        assert!(active.contains(&0x19)); // Right trigger
    }

    #[test]
    fn test_check_triggers_fast_deadzone() {
        let gamepad = XINPUT_GAMEPAD {
            wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0x0000),
            bLeftTrigger: 20,
            bRightTrigger: 20,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        };

        let mut active = SmallVec::<[u32; MAX_INPUTS]>::new();

        XInputHandler::check_triggers_fast(&gamepad, &mut active);
        assert_eq!(active.len(), 0);
    }
}
