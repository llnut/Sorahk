//! XInput API integration for Xbox controllers.
//!
//! Provides support for Xbox 360, Xbox One, and Xbox Series controllers.
//! Handles button input, analog sticks, and triggers with deadzone filtering.

use crate::input_ownership::{DeviceOwnership, InputSource};
use crate::state::{AppState, DeviceType, InputDevice, InputEvent};
use crate::util::{likely, unlikely};
use smallvec::SmallVec;
use std::sync::{Arc, atomic::Ordering};
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

/// Bitset representation of combo for fast matching
#[derive(Clone, Debug)]
struct ComboMask {
    required_bits: u32,
    combo_size: u8,
    button_ids: SmallVec<[u32; 4]>,
}

type ButtonIds = SmallVec<[u32; 2]>;
type TwoKeyEntry = ((u32, u32), ButtonIds);
type MultiKeyEntry = (SmallVec<[u32; 4]>, ButtonIds);

/// Layered combo index for different combo sizes
#[derive(Clone, Debug)]
struct LayeredComboIndex {
    single_keys: SmallVec<[(u32, ButtonIds); 16]>,
    two_keys: SmallVec<[TwoKeyEntry; 16]>,
    multi_keys: SmallVec<[MultiKeyEntry; 8]>,
}

impl LayeredComboIndex {
    #[inline(always)]
    fn new() -> Self {
        Self {
            single_keys: SmallVec::new(),
            two_keys: SmallVec::new(),
            multi_keys: SmallVec::new(),
        }
    }
}

/// XInput device state for change detection.
#[derive(Clone)]
struct XInputDeviceState {
    packet_number: u32,
    last_state: XINPUT_GAMEPAD,
    vid_pid: (u16, u16),
    capture_frames: [CaptureFrame; CAPTURE_FRAMES],
    capture_frame_count: u8,
    active_inputs: SmallVec<[u32; MAX_INPUTS]>,
    active_combos: SmallVec<[Vec<u32>; 4]>,
    last_input_bits: u32,
    combo_masks: SmallVec<[ComboMask; 16]>,
    layered_index: LayeredComboIndex,
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
                        vid_pid,
                        capture_frames: std::array::from_fn(|_| CaptureFrame::new()),
                        capture_frame_count: 0,
                        active_inputs: SmallVec::new(),
                        active_combos: SmallVec::new(),
                        last_input_bits: 0,
                        combo_masks: SmallVec::new(),
                        layered_index: LayeredComboIndex::new(),
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
        // Check and clear cache if configuration was reloaded
        if unlikely(self.state.check_and_reset_xinput_cache_invalid()) {
            for device_state in self.device_states.iter_mut().flatten() {
                device_state.combo_masks.clear();
                device_state.layered_index = LayeredComboIndex::new();
                device_state.active_combos.clear();
                device_state.last_input_bits = 0;
            }
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

                if let Some(device_state) = &mut self.device_states[idx] {
                    let vid_pid = device_state.vid_pid;
                    let stable_device_id = Self::hash_vid_pid_static(vid_pid) as u64;
                    let gamepad = state.Gamepad;

                    let mut current_inputs = SmallVec::<[u32; MAX_INPUTS]>::new();

                    Self::check_buttons_fast(&gamepad, &mut current_inputs);
                    Self::check_analog_sticks_fast(&gamepad, &mut current_inputs);
                    Self::check_triggers_fast(&gamepad, &mut current_inputs);

                    let inputs_changed = current_inputs != device_state.active_inputs;

                    device_state.packet_number = state.dwPacketNumber;
                    device_state.last_state = gamepad;

                    if unlikely(inputs_changed) {
                        let is_capturing = self.state.is_raw_input_capture_active();

                        if unlikely(is_capturing) {
                            Self::handle_capture_mode_xinput(
                                device_state,
                                &current_inputs,
                                vid_pid,
                                &self.state,
                            );
                        } else if let Some(pool) = self.state.get_worker_pool() {
                            Self::handle_normal_mode_xinput(
                                device_state,
                                &current_inputs,
                                stable_device_id,
                                vid_pid,
                                pool,
                                &self.state,
                            );
                        }

                        device_state.active_inputs = current_inputs;
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
                            vid_pid,
                            capture_frames: std::array::from_fn(|_| CaptureFrame::new()),
                            capture_frame_count: 0,
                            active_inputs: SmallVec::new(),
                            active_combos: SmallVec::new(),
                            last_input_bits: 0,
                            combo_masks: SmallVec::new(),
                            layered_index: LayeredComboIndex::new(),
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

    /// Handles capture mode: records frames for later selection
    #[inline]
    fn handle_capture_mode_xinput(
        device_state: &mut XInputDeviceState,
        current_inputs: &SmallVec<[u32; MAX_INPUTS]>,
        vid_pid: (u16, u16),
        state: &Arc<AppState>,
    ) {
        let capture_mode = state.get_xinput_capture_mode();
        let current_button_id = if !current_inputs.is_empty() {
            let stable_device_id = Self::hash_vid_pid_static(vid_pid) as u64;
            Some(Self::hash_inputs_fast(current_inputs, stable_device_id))
        } else {
            None
        };

        if let Some(current_id) = current_button_id {
            if (device_state.capture_frame_count as usize) < CAPTURE_FRAMES {
                let idx = device_state.capture_frame_count as usize;
                let raw_inputs: SmallVec<[u32; 12]> = current_inputs.iter().copied().collect();

                device_state.capture_frames[idx] = CaptureFrame {
                    button_id: current_id,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    raw_inputs,
                };
                device_state.capture_frame_count += 1;
            }
        } else if device_state.capture_frame_count > 0 {
            let best_frame_idx = match capture_mode {
                crate::config::XInputCaptureMode::MostSustained => {
                    Self::get_most_sustained_frame_idx(
                        &device_state.capture_frames,
                        device_state.capture_frame_count,
                    )
                }
                crate::config::XInputCaptureMode::LastStable => Self::get_last_stable_frame_idx(
                    &device_state.capture_frames,
                    device_state.capture_frame_count,
                ),
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
                let button_ids: Vec<u32> = frame.raw_inputs.iter().copied().collect();

                let device = InputDevice::XInputCombo {
                    device_type,
                    button_ids,
                };
                let _ = state.get_raw_input_capture_sender().send(device);
            }

            device_state.capture_frame_count = 0;
        }
    }

    /// Handles normal mode: finds all matching combos using optimized bitset matching
    #[inline]
    fn handle_normal_mode_xinput(
        device_state: &mut XInputDeviceState,
        current_inputs: &SmallVec<[u32; MAX_INPUTS]>,
        _stable_device_id: u64,
        vid_pid: (u16, u16),
        pool: &Arc<dyn crate::state::EventDispatcher>,
        state: &Arc<AppState>,
    ) {
        let device_type = DeviceType::Gamepad(vid_pid.0);

        let current_bits = Self::inputs_to_bitset(current_inputs);

        let xinput_mask = state
            .switch_key_cache
            .xinput_button_mask
            .load(Ordering::Relaxed);
        if unlikely(xinput_mask != 0) {
            let device_hash = state
                .switch_key_cache
                .xinput_device_hash
                .load(Ordering::Relaxed);
            let current_device_hash = AppState::hash_device_type(&device_type);

            if likely(device_hash == current_device_hash) {
                let switch_active = (current_bits & xinput_mask) == xinput_mask;
                let was_active = (device_state.last_input_bits & xinput_mask) == xinput_mask;

                if unlikely(!was_active && switch_active) {
                    state.handle_switch_key_toggle();
                }
            }
        }

        if likely(current_bits == device_state.last_input_bits) {
            return;
        }

        // Check paused state after switch key detection
        if unlikely(state.is_paused()) {
            device_state.last_input_bits = current_bits;
            return;
        }

        if unlikely(device_state.combo_masks.is_empty()) {
            let all_combos = state.get_xinput_combos_for_device(&device_type);
            Self::build_combo_masks(&all_combos, &mut device_state.combo_masks);
            Self::build_layered_index(&all_combos, &mut device_state.layered_index);
        }

        let mut new_active_combos = SmallVec::<[Vec<u32>; 4]>::new();

        if device_state.combo_masks.len() <= 8 {
            Self::match_combos_bitset(
                current_bits,
                &device_state.combo_masks,
                &mut new_active_combos,
            );
        } else if device_state.combo_masks.len() <= 16 {
            // Medium set: use layered index
            Self::match_combos_layered(
                current_inputs,
                &device_state.layered_index,
                &mut new_active_combos,
            );
        } else {
            // Large set: use AVX2 SIMD batch matching
            #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
            {
                Self::match_combos_simd(
                    current_bits,
                    &device_state.combo_masks,
                    &mut new_active_combos,
                );
            }
            #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
            {
                Self::match_combos_bitset(
                    current_bits,
                    &device_state.combo_masks,
                    &mut new_active_combos,
                );
            }
        }

        // Compare with previously active combos
        let prev_active = &device_state.active_combos;

        // Find newly activated combos
        for combo in &new_active_combos {
            if !Self::contains_combo(prev_active, combo) {
                let device = InputDevice::XInputCombo {
                    device_type,
                    button_ids: combo.clone(),
                };
                pool.dispatch(InputEvent::Pressed(device));
            }
        }

        // Find deactivated combos
        for combo in prev_active {
            if !Self::contains_combo(&new_active_combos, combo) {
                let device = InputDevice::XInputCombo {
                    device_type,
                    button_ids: combo.clone(),
                };
                pool.dispatch(InputEvent::Released(device));
            }
        }

        device_state.active_combos = new_active_combos;
        device_state.last_input_bits = current_bits;
    }

    /// Converts input IDs to bitset representation
    /// Each button gets a unique bit position (0-31 for XInput's 26 buttons)
    #[inline(always)]
    fn inputs_to_bitset(inputs: &[u32]) -> u32 {
        let mut bits = 0u32;
        for &input_id in inputs {
            if likely(input_id < 32) {
                bits |= 1u32 << input_id;
            }
        }
        bits
    }

    /// Builds bitset masks for all combos
    #[inline]
    fn build_combo_masks(combos: &[Vec<u32>], masks: &mut SmallVec<[ComboMask; 16]>) {
        masks.clear();
        for combo in combos {
            let required_bits = Self::inputs_to_bitset(combo);
            masks.push(ComboMask {
                required_bits,
                combo_size: combo.len() as u8,
                button_ids: combo.iter().copied().collect(),
            });
        }
        // Sort by combo size (larger combos first for priority matching)
        masks.sort_unstable_by(|a, b| b.combo_size.cmp(&a.combo_size));
    }

    /// Builds layered index for fast lookup
    #[inline]
    fn build_layered_index(combos: &[Vec<u32>], index: &mut LayeredComboIndex) {
        index.single_keys.clear();
        index.two_keys.clear();
        index.multi_keys.clear();

        for combo in combos {
            match combo.len() {
                1 => {
                    let key = combo[0];
                    if let Some(entry) = index.single_keys.iter_mut().find(|(k, _)| *k == key) {
                        entry.1 = combo.iter().copied().collect();
                    } else {
                        index
                            .single_keys
                            .push((key, combo.iter().copied().collect()));
                    }
                }
                2 => {
                    let mut sorted = [combo[0], combo[1]];
                    sorted.sort_unstable();
                    let key = (sorted[0], sorted[1]);
                    if let Some(entry) = index.two_keys.iter_mut().find(|(k, _)| *k == key) {
                        entry.1 = combo.iter().copied().collect();
                    } else {
                        index.two_keys.push((key, combo.iter().copied().collect()));
                    }
                }
                _ => {
                    index.multi_keys.push((
                        combo.iter().copied().collect(),
                        combo.iter().copied().collect(),
                    ));
                }
            }
        }
    }

    /// Bitset-based combo matching
    #[inline(always)]
    fn match_combos_bitset(
        current_bits: u32,
        masks: &[ComboMask],
        matched: &mut SmallVec<[Vec<u32>; 4]>,
    ) {
        for mask in masks {
            if (current_bits & mask.required_bits) == mask.required_bits {
                matched.push(mask.button_ids.to_vec());
            }
        }
    }

    /// Layered index matching
    #[inline]
    fn match_combos_layered(
        current_inputs: &[u32],
        index: &LayeredComboIndex,
        matched: &mut SmallVec<[Vec<u32>; 4]>,
    ) {
        // Match single keys - O(n) where n is number of pressed keys
        for &key in current_inputs {
            for (k, combo) in &index.single_keys {
                if *k == key {
                    matched.push(combo.to_vec());
                }
            }
        }

        // Match two-key combos - O(n²) but n ≤ 5 typically
        if current_inputs.len() >= 2 {
            for i in 0..current_inputs.len() {
                for j in (i + 1)..current_inputs.len() {
                    let mut sorted = [current_inputs[i], current_inputs[j]];
                    sorted.sort_unstable();
                    let key = (sorted[0], sorted[1]);

                    for (k, combo) in &index.two_keys {
                        if *k == key {
                            matched.push(combo.to_vec());
                        }
                    }
                }
            }
        }

        // Match multi-key combos - brute force (rare case)
        for (combo_keys, combo) in &index.multi_keys {
            if combo_keys.iter().all(|&key| current_inputs.contains(&key)) {
                matched.push(combo.to_vec());
            }
        }
    }

    /// AVX2 batch matching
    /// Processes 8 combos in parallel using AVX2
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[inline]
    fn match_combos_simd(
        current_bits: u32,
        masks: &[ComboMask],
        matched: &mut SmallVec<[Vec<u32>; 4]>,
    ) {
        use std::arch::x86_64::*;

        let mut i = 0;
        let current_vec = unsafe { _mm256_set1_epi32(current_bits as i32) };

        // Process 8 masks at a time
        while i + 8 <= masks.len() {
            unsafe {
                // Load 8 required_bits masks
                let mask_bits: [i32; 8] = [
                    masks[i].required_bits as i32,
                    masks[i + 1].required_bits as i32,
                    masks[i + 2].required_bits as i32,
                    masks[i + 3].required_bits as i32,
                    masks[i + 4].required_bits as i32,
                    masks[i + 5].required_bits as i32,
                    masks[i + 6].required_bits as i32,
                    masks[i + 7].required_bits as i32,
                ];

                let masks_vec = _mm256_loadu_si256(mask_bits.as_ptr() as *const __m256i);

                // Perform (current & mask) == mask check
                let anded = _mm256_and_si256(current_vec, masks_vec);
                let cmp = _mm256_cmpeq_epi32(anded, masks_vec);
                let result_mask = _mm256_movemask_epi8(cmp);

                // Check which combos matched (every 4 bytes = 1 result)
                for j in 0..8 {
                    let bit_offset = j * 4;
                    if (result_mask & (0x0F << bit_offset)) == (0x0F << bit_offset) {
                        matched.push(masks[i + j].button_ids.to_vec());
                    }
                }
            }

            i += 8;
        }

        // Handle remaining masks (< 8)
        for mask in &masks[i..] {
            if (current_bits & mask.required_bits) == mask.required_bits {
                matched.push(mask.button_ids.to_vec());
            }
        }
    }

    /// Checks if a combo list contains a specific combo (order-independent)
    #[inline(always)]
    fn contains_combo(combos: &[Vec<u32>], target: &[u32]) -> bool {
        if unlikely(combos.is_empty()) {
            return false;
        }

        for combo in combos {
            if likely(combo.len() == target.len()) && Self::combo_equals(combo, target) {
                return true;
            }
        }

        false
    }

    /// Fast comparison of two combos (order-independent)
    #[inline(always)]
    fn combo_equals(a: &[u32], b: &[u32]) -> bool {
        if unlikely(a.len() != b.len()) {
            return false;
        }

        // Fast path for small combos
        if likely(a.len() <= 4) {
            return a.iter().all(|k| b.contains(k));
        }

        // For larger combos, use sorted comparison (rare case)
        let mut a_sorted: SmallVec<[u32; 8]> = a.iter().copied().collect();
        let mut b_sorted: SmallVec<[u32; 8]> = b.iter().copied().collect();
        a_sorted.sort_unstable();
        b_sorted.sort_unstable();
        a_sorted == b_sorted
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
    pub fn hash_vid_pid_static(vid_pid: (u16, u16)) -> u32 {
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

    #[test]
    fn test_contains_combo_true() {
        let combos = vec![vec![0x01], vec![0x01, 0x0B], vec![0x0C, 0x0D]];
        let target = vec![0x01, 0x0B];
        assert!(XInputHandler::contains_combo(&combos, &target));
    }

    #[test]
    fn test_contains_combo_false() {
        let combos = vec![vec![0x01], vec![0x01, 0x0B]];
        let target = vec![0x0C, 0x0D];
        assert!(!XInputHandler::contains_combo(&combos, &target));
    }

    #[test]
    fn test_contains_combo_order_independent() {
        let combos = vec![vec![0x01, 0x0B, 0x0C]];
        let target = vec![0x0C, 0x01, 0x0B]; // Different order
        assert!(XInputHandler::contains_combo(&combos, &target));
    }

    #[test]
    fn test_combo_equals_same_order() {
        let a = vec![0x01, 0x0B, 0x0C];
        let b = vec![0x01, 0x0B, 0x0C];
        assert!(XInputHandler::combo_equals(&a, &b));
    }

    #[test]
    fn test_combo_equals_diff_order() {
        let a = vec![0x01, 0x0B, 0x0C];
        let b = vec![0x0C, 0x01, 0x0B];
        assert!(XInputHandler::combo_equals(&a, &b));
    }

    #[test]
    fn test_combo_equals_different() {
        let a = vec![0x01, 0x0B];
        let b = vec![0x0C, 0x0D];
        assert!(!XInputHandler::combo_equals(&a, &b));
    }

    #[test]
    fn test_inputs_to_bitset() {
        let inputs = vec![0x01, 0x0B, 0x0D]; // DPAD_UP, A, X
        let bits = XInputHandler::inputs_to_bitset(&inputs);

        assert_eq!(bits & (1 << 0x01), 1 << 0x01);
        assert_eq!(bits & (1 << 0x0B), 1 << 0x0B);
        assert_eq!(bits & (1 << 0x0D), 1 << 0x0D);
        assert_eq!(bits & (1 << 0x0C), 0); // B not pressed
    }

    #[test]
    fn test_build_combo_masks() {
        let combos = vec![
            vec![0x01, 0x0B], // DPAD_UP + A
            vec![0x0C, 0x0D], // B + X
            vec![0x0E],       // Y
        ];

        let mut masks = SmallVec::new();
        XInputHandler::build_combo_masks(&combos, &mut masks);

        assert_eq!(masks.len(), 3);

        // Check first combo mask
        let expected_bits = (1 << 0x01) | (1 << 0x0B);
        assert_eq!(masks[0].required_bits, expected_bits);
        assert_eq!(masks[0].combo_size, 2);
    }

    #[test]
    fn test_match_combos_bitset() {
        let mut masks: SmallVec<[ComboMask; 16]> = SmallVec::new();
        masks.push(ComboMask {
            required_bits: (1 << 0x01) | (1 << 0x0B),
            combo_size: 2,
            button_ids: SmallVec::from_slice(&[0x01, 0x0B]),
        });
        masks.push(ComboMask {
            required_bits: 1 << 0x0C,
            combo_size: 1,
            button_ids: SmallVec::from_slice(&[0x0C]),
        });

        let current_bits = (1 << 0x01) | (1 << 0x0B) | (1 << 0x0C);
        let mut matched: SmallVec<[Vec<u32>; 4]> = SmallVec::new();

        XInputHandler::match_combos_bitset(current_bits, &masks, &mut matched);

        // Both combos should match
        assert_eq!(matched.len(), 2);
        assert!(matched.contains(&vec![0x01, 0x0B]));
        assert!(matched.contains(&vec![0x0C]));
    }

    #[test]
    fn test_match_combos_layered() {
        let mut index = LayeredComboIndex::new();

        // Single key: A
        index
            .single_keys
            .push((0x0B, SmallVec::from_slice(&[0x0B])));

        // Two keys: DPAD_UP + A
        index
            .two_keys
            .push(((0x01, 0x0B), SmallVec::from_slice(&[0x01, 0x0B])));

        let current_inputs = vec![0x01, 0x0B]; // DPAD_UP + A
        let mut matched = SmallVec::new();

        XInputHandler::match_combos_layered(&current_inputs, &index, &mut matched);

        // Both should match
        assert_eq!(matched.len(), 2);
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[test]
    fn test_match_combos_simd() {
        // Create 10 combo masks
        let mut masks: SmallVec<[ComboMask; 16]> = SmallVec::new();
        for i in 0..10 {
            masks.push(ComboMask {
                required_bits: 1 << i,
                combo_size: 1,
                button_ids: SmallVec::from_slice(&[i]),
            });
        }

        // Press buttons 0, 2, 4
        let current_bits = (1 << 0) | (1 << 2) | (1 << 4);
        let mut matched: SmallVec<[Vec<u32>; 4]> = SmallVec::new();

        XInputHandler::match_combos_simd(current_bits, &masks, &mut matched);

        // Should match 3 combos
        assert_eq!(matched.len(), 3);
        assert!(matched.contains(&vec![0]));
        assert!(matched.contains(&vec![2]));
        assert!(matched.contains(&vec![4]));
    }

    #[test]
    fn test_incremental_update_no_change() {
        // Simulate no change scenario (most common)
        let inputs = vec![0x01, 0x0B];
        let bits1 = XInputHandler::inputs_to_bitset(&inputs);
        let bits2 = XInputHandler::inputs_to_bitset(&inputs);

        // Bits should be identical, allowing early exit
        assert_eq!(bits1, bits2);
    }

    #[test]
    fn test_combo_priority_by_size() {
        let combos = vec![
            vec![0x01],             // Single key
            vec![0x01, 0x0B, 0x0C], // Three keys
            vec![0x01, 0x0B],       // Two keys
        ];

        let mut masks = SmallVec::new();
        XInputHandler::build_combo_masks(&combos, &mut masks);

        // Larger combos should come first (sorted by combo_size descending)
        assert_eq!(masks[0].combo_size, 3);
        assert_eq!(masks[1].combo_size, 2);
        assert_eq!(masks[2].combo_size, 1);
    }

    // ==================== Performance Benchmarks ====================

    #[test]
    #[ignore] // Run with: cargo test --release -- --ignored --nocapture
    fn benchmark_matching_strategies() {
        use std::time::Instant;

        // Setup: 20 combos (realistic game scenario)
        let combos: Vec<Vec<u32>> = vec![
            vec![0x01],             // DPAD_UP
            vec![0x02],             // DPAD_DOWN
            vec![0x03],             // DPAD_LEFT
            vec![0x04],             // DPAD_RIGHT
            vec![0x0B],             // A
            vec![0x0C],             // B
            vec![0x0D],             // X
            vec![0x0E],             // Y
            vec![0x01, 0x0B],       // UP + A
            vec![0x01, 0x0C],       // UP + B
            vec![0x02, 0x0B],       // DOWN + A
            vec![0x03, 0x0D],       // LEFT + X
            vec![0x04, 0x0E],       // RIGHT + Y
            vec![0x01, 0x0B, 0x0D], // UP + A + X
            vec![0x02, 0x0C, 0x0E], // DOWN + B + Y
            vec![0x09, 0x0A],       // LB + RB
            vec![0x18],             // LT
            vec![0x19],             // RT
            vec![0x10, 0x12],       // LS_Right + LS_Up
            vec![0x14, 0x16],       // RS_Right + RS_Up
        ];

        let mut masks = SmallVec::new();
        XInputHandler::build_combo_masks(&combos, &mut masks);

        let mut layered_index = LayeredComboIndex::new();
        XInputHandler::build_layered_index(&combos, &mut layered_index);

        // Test case: pressing UP + A + X (should match 4 combos)
        let inputs = vec![0x01, 0x0B, 0x0D];
        let bits = XInputHandler::inputs_to_bitset(&inputs);

        const ITERATIONS: usize = 100_000;

        // Benchmark 1: Bitset matching
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let mut matched = SmallVec::<[Vec<u32>; 4]>::new();
            XInputHandler::match_combos_bitset(bits, &masks, &mut matched);
        }
        let bitset_time = start.elapsed();

        // Benchmark 2: Layered index matching
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let mut matched = SmallVec::<[Vec<u32>; 4]>::new();
            XInputHandler::match_combos_layered(&inputs, &layered_index, &mut matched);
        }
        let layered_time = start.elapsed();

        // Benchmark 3: AVX2 SIMD matching (if compiled with AVX2)
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        let simd_time = {
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let mut matched = SmallVec::<[Vec<u32>; 4]>::new();
                XInputHandler::match_combos_simd(bits, &masks, &mut matched);
            }
            Some(start.elapsed())
        };

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        let simd_time: Option<std::time::Duration> = None;

        println!("\n===== Performance Benchmark Results =====");
        println!("Iterations: {}", ITERATIONS);
        println!("Combos: {}", combos.len());
        println!("Inputs: {:?}", inputs);
        println!();
        println!(
            "Bitset matching:  {:?} ({:.2} ns/iter)",
            bitset_time,
            bitset_time.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "Layered matching: {:?} ({:.2} ns/iter)",
            layered_time,
            layered_time.as_nanos() as f64 / ITERATIONS as f64
        );

        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        if let Some(simd_time) = simd_time {
            println!(
                "AVX2 SIMD matching: {:?} ({:.2} ns/iter)",
                simd_time,
                simd_time.as_nanos() as f64 / ITERATIONS as f64
            );
        }

        println!("\nSpeedup vs Layered:");
        println!(
            "  Bitset: {:.2}x",
            layered_time.as_nanos() as f64 / bitset_time.as_nanos() as f64
        );

        #[cfg(target_arch = "x86_64")]
        if let Some(simd_time) = simd_time {
            println!(
                "  SIMD:   {:.2}x",
                layered_time.as_nanos() as f64 / simd_time.as_nanos() as f64
            );
        }

        println!("=========================================\n");
    }

    #[test]
    #[ignore]
    fn benchmark_incremental_update() {
        use std::time::Instant;

        let inputs = vec![0x01, 0x0B, 0x0D];
        let bits = XInputHandler::inputs_to_bitset(&inputs);

        const ITERATIONS: usize = 1_000_000;

        // Scenario 1: No change (95% of frames in typical gameplay)
        let start = Instant::now();
        let mut last_bits = bits;
        for _ in 0..ITERATIONS {
            let current_bits = bits; // Same as last frame
            if current_bits == last_bits {
                continue; // Early exit
            }
            last_bits = current_bits;
        }
        let no_change_time = start.elapsed();

        // Scenario 2: Always changing (worst case, 5% of frames)
        let start = Instant::now();
        let mut last_bits = bits;
        for i in 0..ITERATIONS {
            let current_bits = bits ^ (i as u32 & 1); // Toggle a bit
            if current_bits == last_bits {
                continue;
            }
            last_bits = current_bits;
            // Would do full matching here
        }
        let always_change_time = start.elapsed();

        println!("\n===== Incremental Update Benchmark =====");
        println!(
            "No change (95%):  {:?} ({:.2} ns/iter)",
            no_change_time,
            no_change_time.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "Changed (5%):     {:?} ({:.2} ns/iter)",
            always_change_time,
            always_change_time.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "Speedup: {:.2}x for static frames",
            always_change_time.as_nanos() as f64 / no_change_time.as_nanos() as f64
        );
        println!("=========================================\n");
    }
}
