use std::time::Instant;

use scc::{Guard, Shared, Tag};
use smallvec::SmallVec;

use windows::Win32::UI::WindowsAndMessaging::*;

use crate::util::{likely, unlikely};

use super::AppState;
use super::types::*;

/// Returns how much of a mouse-move delta lands in the half-plane opposite
/// to the given direction. Screen coordinates have Y pointing down. Only
/// positive values are meaningful; a negative result means the delta is in
/// the same hemisphere as the direction and should be ignored by the
/// caller. Diagonal directions are scaled by 1/sqrt(2) to give the
/// projection onto the unit vector, so thresholds stay comparable across
/// all eight directions.
#[inline(always)]
fn opposite_projection(dx: i32, dy: i32, dir: MouseMoveDirection) -> f32 {
    const FRAC_1_SQRT_2: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let dxf = dx as f32;
    let dyf = dy as f32;
    match dir {
        MouseMoveDirection::Up => dyf,
        MouseMoveDirection::Down => -dyf,
        MouseMoveDirection::Left => dxf,
        MouseMoveDirection::Right => -dxf,
        MouseMoveDirection::UpLeft => (dxf + dyf) * FRAC_1_SQRT_2,
        MouseMoveDirection::UpRight => (-dxf + dyf) * FRAC_1_SQRT_2,
        MouseMoveDirection::DownLeft => (dxf - dyf) * FRAC_1_SQRT_2,
        MouseMoveDirection::DownRight => (-dxf - dyf) * FRAC_1_SQRT_2,
    }
}

impl AppState {
    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn handle_key_event(&self, message: u32, vk_code: u32) -> bool {
        use std::sync::atomic::Ordering;

        let mut should_block = false;

        if matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN) {
            let _ = self.pressed_keys.insert_sync(vk_code);

            let kb_vk = self.switch_key_cache.keyboard_vk.load(Ordering::Relaxed);

            if unlikely(kb_vk != 0 && vk_code == kb_vk) {
                self.handle_switch_key_toggle();
                return true;
            }

            if unlikely(kb_vk == 0) {
                let guard = Guard::new();
                let device_ptr = self
                    .switch_key_cache
                    .full_device
                    .load(Ordering::Acquire, &guard);
                if let Some(device) = device_ptr.as_ref()
                    && let InputDevice::KeyCombo(keys) = device
                    && keys.contains(&vk_code)
                {
                    let mut all_pressed = true;
                    for k in keys.iter() {
                        if !self.pressed_keys.contains_sync(k) {
                            all_pressed = false;
                            break;
                        }
                    }

                    if all_pressed {
                        self.handle_switch_key_toggle();
                        return true;
                    }
                }
            }
        }

        if matches!(message, WM_KEYUP | WM_SYSKEYUP) {
            let _ = self.pressed_keys.remove_sync(&vk_code);
        }

        if unlikely(self.is_paused() || !self.is_process_whitelisted()) {
            return should_block;
        }

        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if unlikely(self.is_in_active_combo(vk_code)) {
                    let is_main_key_no_turbo = self.is_main_key_in_active_combo_no_turbo(vk_code);
                    if !is_main_key_no_turbo {
                        return true;
                    }
                }

                let current_device = InputDevice::Keyboard(vk_code);
                let guard = Guard::new();
                let last_inputs_loaded = self.last_sequence_inputs.load(Ordering::Acquire, &guard);
                let should_check_interrupt = if let Some(last_inputs) = last_inputs_loaded.as_ref()
                {
                    !last_inputs.is_empty() && !last_inputs.contains(&current_device)
                } else {
                    false
                };

                if should_check_interrupt {
                    let guard = Guard::new();
                    let last_seq = self.last_sequence_device.load(Ordering::Acquire, &guard);
                    if let Some(seq_device) = last_seq.as_ref().map(|device| device.clone()) {
                        let should_interrupt =
                            if let Some(mapping_info) = self.get_input_mapping(&seq_device) {
                                matches!(
                                    mapping_info.target_action,
                                    OutputAction::MouseMove(..) | OutputAction::MouseScroll(..)
                                )
                            } else {
                                false
                            };

                        if should_interrupt {
                            let _ = self
                                .last_sequence_device
                                .swap((None, Tag::None), Ordering::Release);
                            let _ = self
                                .last_sequence_inputs
                                .swap((None, Tag::None), Ordering::Release);
                            if let Some(pool) = self.worker_pool.get() {
                                pool.dispatch(InputEvent::Released(seq_device));
                            }
                        }
                    }
                }

                let mut pressed_snapshot: SmallVec<[u32; 16]> = SmallVec::new();
                self.pressed_keys.iter_sync(|key| {
                    pressed_snapshot.push(*key);
                    true
                });

                let matched_device = self
                    .find_matching_combo(&pressed_snapshot, vk_code)
                    .or(Some(InputDevice::Keyboard(vk_code)));

                if let Some(device) = matched_device {
                    let now = Instant::now();
                    let sequence_match_result = self.record_and_match_sequence(device.clone(), now);

                    if let Some((matched_device, sequence_inputs)) = sequence_match_result {
                        // Sequence matched - check if it's a sequence-only mapping
                        if let Some(mapping_info) = self.get_input_mapping(&matched_device)
                            && mapping_info.is_sequence
                        {
                            let shared_device = Shared::new(matched_device.clone());
                            let _ = self
                                .last_sequence_device
                                .swap((Some(shared_device), Tag::None), Ordering::Release);

                            let shared_inputs = Shared::new(sequence_inputs);
                            let _ = self
                                .last_sequence_inputs
                                .swap((Some(shared_inputs), Tag::None), Ordering::Release);

                            if let Some(pool) = self.worker_pool.get() {
                                pool.dispatch(InputEvent::Pressed(matched_device));
                                should_block = true;
                            }
                        }
                    } else {
                        // No sequence matched - check if holding sequence trigger key
                        let guard = Guard::new();
                        let last_seq_device =
                            self.last_sequence_device.load(Ordering::Acquire, &guard);
                        let last_seq_inputs =
                            self.last_sequence_inputs.load(Ordering::Acquire, &guard);

                        if let (Some(seq_device), Some(seq_inputs)) =
                            (last_seq_device.as_ref(), last_seq_inputs.as_ref())
                        {
                            if seq_inputs.contains(&device) {
                                // User is holding a key from the sequence - continue turbo
                                if let Some(pool) = self.worker_pool.get() {
                                    pool.dispatch(InputEvent::Pressed(seq_device.clone()));
                                    should_block = true;
                                }
                            } else if let Some(mapping_info) = self.get_input_mapping(&device) {
                                // Not part of sequence, handle normally if not sequence-only
                                if !mapping_info.is_sequence {
                                    should_block = self.dispatch_normal_key(&device, vk_code);
                                }
                            }
                        } else if let Some(mapping_info) = self.get_input_mapping(&device) {
                            // No active sequence, handle normally if not sequence-only
                            if !mapping_info.is_sequence {
                                should_block = self.dispatch_normal_key(&device, vk_code);
                            }
                        }
                    }
                }
            }

            WM_KEYUP | WM_SYSKEYUP => {
                let removed_combos = self.cleanup_released_combos();

                if !removed_combos.is_empty()
                    && let Some(pool) = self.worker_pool.get()
                {
                    for combo in removed_combos {
                        pool.dispatch(InputEvent::Released(combo));
                    }
                    should_block = true;
                }

                let device = self.find_device_for_release(vk_code);
                if let Some(dev) = device
                    && let Some(pool) = self.worker_pool.get()
                {
                    let guard = Guard::new();
                    let last_seq_inputs = self.last_sequence_inputs.load(Ordering::Acquire, &guard);

                    if let Some(seq_inputs) = last_seq_inputs.as_ref()
                        && let Some(last_input) = seq_inputs.last()
                        && last_input == &dev
                    {
                        let _ = self
                            .last_sequence_device
                            .swap((None, Tag::None), Ordering::Release);
                        let _ = self
                            .last_sequence_inputs
                            .swap((None, Tag::None), Ordering::Release);
                    }

                    pool.dispatch(InputEvent::Released(dev));
                    should_block = true;
                }
            }

            _ => {}
        }

        should_block
    }

    /// Dispatch a normal (non-sequence) key press event to the worker pool.
    #[inline(always)]
    fn dispatch_normal_key(&self, device: &InputDevice, _vk_code: u32) -> bool {
        let already_active = self.is_combo_active(device);
        if already_active && self.is_turbo_enabled(device) {
            return true;
        }

        if let InputDevice::KeyCombo(keys) = device {
            self.add_active_combo(device.clone(), keys.iter().copied().collect());
        }

        if let Some(pool) = self.worker_pool.get() {
            pool.dispatch(InputEvent::Pressed(device.clone()));
            return true;
        }
        false
    }

    #[inline(always)]
    pub(super) fn find_matching_combo(
        &self,
        pressed_keys: &SmallVec<[u32; 16]>,
        main_key: u32,
    ) -> Option<InputDevice> {
        self.cached_combo_index
            .read_sync(&main_key, |_, combos| {
                for device in combos {
                    if let InputDevice::KeyCombo(combo_keys) = device {
                        let all_pressed = combo_keys.iter().all(|&k| pressed_keys.contains(&k));
                        if likely(all_pressed) {
                            return Some(device.clone());
                        }
                    }
                }
                None
            })
            .flatten()
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn handle_mouse_event(
        &self,
        message: u32,
        mouse_data: u32,
        mouse_x: i32,
        mouse_y: i32,
    ) -> bool {
        use std::sync::atomic::Ordering;

        // Short-circuit the entire mouse pipeline when paused or the
        // foreground app is not whitelisted. The whitelist check also
        // trips the transition detector which releases held keys.
        if unlikely(self.is_paused() || !self.is_process_whitelisted()) {
            return false;
        }

        let mut should_block = false;

        if message == WM_MOUSEMOVE {
            let last_x = self.last_mouse_x.load(Ordering::Acquire);
            let last_y = self.last_mouse_y.load(Ordering::Acquire);

            if last_x != 0 || last_y != 0 {
                let delta_x = mouse_x - last_x;
                let delta_y = mouse_y - last_y;
                let delta_mag_sq =
                    (delta_x as i64) * (delta_x as i64) + (delta_y as i64) * (delta_y as i64);

                // Per-event speed gate. Events whose single-frame delta
                // falls under this threshold are treated as hardware noise
                // or unintentional drift and are dropped entirely: they do
                // not contribute to the motion accumulator, the reverse
                // accumulator, or trigger any logic. Deliberate motion
                // produces events above the threshold.
                let per_event_min_sq =
                    self.mouse_move_per_event_min_sq.load(Ordering::Acquire) as i64;
                let qualifies = delta_mag_sq >= per_event_min_sq;

                let opp_rearm_px = self.mouse_move_rearm_px.load(Ordering::Acquire);
                let last_dir_now =
                    MouseMoveDirection::from_u8(self.last_mouse_direction.load(Ordering::Acquire));

                // Default to existing accumulator for non-qualifying events
                // so no stored state changes when this frame is dropped.
                let mut accum_dx = self.mouse_accum_dx.load(Ordering::Acquire);
                let mut accum_dy = self.mouse_accum_dy.load(Ordering::Acquire);

                if qualifies {
                    // Qualifying events feed the accumulator so slow but
                    // deliberate motion still eventually crosses the
                    // threshold, and feed the reverse accumulator used to
                    // re-arm same-direction firing.
                    accum_dx = accum_dx.saturating_add(delta_x);
                    accum_dy = accum_dy.saturating_add(delta_y);
                    self.mouse_accum_dx.store(accum_dx, Ordering::Release);
                    self.mouse_accum_dy.store(accum_dy, Ordering::Release);

                    if let Some(ld) = last_dir_now {
                        let opp = opposite_projection(delta_x, delta_y, ld);
                        // The reverse projection itself must clear the same
                        // per-event speed gate as the full delta. Without
                        // this, a mostly-forward event with a small lateral
                        // reverse component would quietly drip pixels into
                        // opp_distance and eventually re-arm without any
                        // real return motion.
                        let per_event_min_px =
                            self.mouse_move_per_event_min_px.load(Ordering::Acquire);
                        if opp >= per_event_min_px as f32 {
                            let add = opp.round().max(0.0) as u32;
                            if add > 0 {
                                let cur = self.mouse_opp_distance_px.load(Ordering::Acquire);
                                self.mouse_opp_distance_px
                                    .store(cur.saturating_add(add), Ordering::Release);
                            }
                        }
                    }
                }

                let accum_mag_sq =
                    (accum_dx as i64) * (accum_dx as i64) + (accum_dy as i64) * (accum_dy as i64);
                let min_trigger_sq = self.mouse_move_min_trigger_sq.load(Ordering::Acquire) as i64;
                if qualifies && accum_mag_sq >= min_trigger_sq {
                    // Direction is read from the accumulated vector, so
                    // slight zig-zag within one "swipe unit" still resolves
                    // to the net direction.
                    let angle = (accum_dy as f32).atan2(accum_dx as f32).to_degrees();

                    // Determine direction (8 directions)
                    // Note: Screen coordinates have Y-axis pointing DOWN
                    let direction = if (-22.5..22.5).contains(&angle) {
                        MouseMoveDirection::Right
                    } else if (22.5..67.5).contains(&angle) {
                        MouseMoveDirection::DownRight
                    } else if (67.5..112.5).contains(&angle) {
                        MouseMoveDirection::Down
                    } else if (112.5..157.5).contains(&angle) {
                        MouseMoveDirection::DownLeft
                    } else if !(-157.5..157.5).contains(&angle) {
                        MouseMoveDirection::Left
                    } else if (-157.5..-112.5).contains(&angle) {
                        MouseMoveDirection::UpLeft
                    } else if (-112.5..-67.5).contains(&angle) {
                        MouseMoveDirection::Up
                    } else {
                        MouseMoveDirection::UpRight
                    };

                    // Direction-change records are unconditional. Same-direction
                    // records are gated by either the reverse-distance
                    // accumulator or the pause reset above, so a continuous
                    // swipe does not spam records while a double-flick still
                    // re-arms the next record.
                    let last_dir_u8 = self.last_mouse_direction.load(Ordering::Acquire);
                    let last_dir = MouseMoveDirection::from_u8(last_dir_u8);

                    let mut rearm_dispatch: Option<InputDevice> = None;
                    let (should_record, should_stop_previous) = if last_dir != Some(direction) {
                        let should_stop = last_dir.is_some();
                        self.last_mouse_direction
                            .store(direction.to_u8(), Ordering::Release);
                        self.mouse_opp_distance_px.store(0, Ordering::Release);
                        (true, should_stop)
                    } else if self.mouse_opp_distance_px.load(Ordering::Acquire) >= opp_rearm_px {
                        // Same-direction re-arm: emit a `RetapHold` so the
                        // worker releases, waits the mapping's interval,
                        // then presses again. That intermediate keyup is
                        // what a polling-based game samples to register the
                        // double-tap. Skip the record/dispatch path below
                        // because `last_sequence_device` is already set to
                        // this direction and the worker will restart its
                        // hold cycle.
                        self.mouse_opp_distance_px.store(0, Ordering::Release);
                        rearm_dispatch = Some(InputDevice::MouseMove(direction));
                        self.sequence_matcher.clear_history();
                        (false, false)
                    } else {
                        (false, false)
                    };

                    if let Some(dev) = rearm_dispatch
                        && let Some(pool) = self.worker_pool.get()
                    {
                        pool.dispatch(InputEvent::RetapHold(dev));
                    }

                    // Clear the previous mouse-move sequence state when the
                    // direction actually changed, but defer the `Released`
                    // dispatch until *after* the new `Pressed` fires below.
                    // The worker processes events FIFO, so sending Pressed
                    // first lets ref counts for any shared scancodes climb
                    // to >= 2 before the release drops them back to 1 — the
                    // overlapping keys stay held all the way through the
                    // transition (smooth cardinal → diagonal handoff).
                    let mut deferred_release: Option<InputDevice> = None;
                    if should_stop_previous {
                        let guard = Guard::new();
                        let last_seq = self.last_sequence_device.load(Ordering::Acquire, &guard);
                        if let Some(seq_device) = last_seq.as_ref().map(|device| device.clone())
                            && matches!(seq_device, InputDevice::MouseMove(_))
                        {
                            let _ = self
                                .last_sequence_device
                                .swap((None, Tag::None), Ordering::Release);
                            let _ = self
                                .last_sequence_inputs
                                .swap((None, Tag::None), Ordering::Release);

                            deferred_release = Some(seq_device);
                        }
                    }

                    if should_record {
                        let device = InputDevice::MouseMove(direction);
                        let now = Instant::now();

                        let sequence_match_result =
                            self.record_and_match_sequence(device.clone(), now);

                        if let Some((matched_device, sequence_inputs)) = sequence_match_result
                            && let Some(mapping_info) = self.get_input_mapping(&matched_device)
                            && mapping_info.is_sequence
                        {
                            self.sequence_matcher.clear_history();

                            let shared_device = Shared::new(matched_device.clone());
                            let _ = self
                                .last_sequence_device
                                .swap((Some(shared_device), Tag::None), Ordering::Release);

                            let shared_inputs = Shared::new(sequence_inputs);
                            let _ = self
                                .last_sequence_inputs
                                .swap((Some(shared_inputs), Tag::None), Ordering::Release);

                            if let Some(pool) = self.worker_pool.get() {
                                pool.dispatch(InputEvent::Pressed(matched_device));
                            }
                        }
                    } else {
                        // Direction unchanged - check if holding sequence trigger direction
                        let guard = Guard::new();
                        let last_seq_device =
                            self.last_sequence_device.load(Ordering::Acquire, &guard);
                        let last_seq_inputs =
                            self.last_sequence_inputs.load(Ordering::Acquire, &guard);

                        if let (Some(seq_device), Some(seq_inputs)) =
                            (last_seq_device.as_ref(), last_seq_inputs.as_ref())
                        {
                            let device = InputDevice::MouseMove(direction);
                            // Check if current direction is part of the last matched sequence
                            if seq_inputs.contains(&device) {
                                // User is continuing in same direction
                                // Continue dispatching Pressed events for turbo
                                if let Some(pool) = self.worker_pool.get() {
                                    pool.dispatch(InputEvent::Pressed(seq_device.clone()));
                                }
                            }
                        }
                    }

                    // Deferred release fires AFTER the new Pressed above,
                    // so shared scancodes keep ref >= 1 during the swap.
                    if let Some(seq_device) = deferred_release
                        && let Some(pool) = self.worker_pool.get()
                    {
                        pool.dispatch(InputEvent::Released(seq_device));
                    }

                    // Threshold reached this event: reset the motion
                    // accumulator so the next record requires a fresh
                    // threshold's worth of motion. This avoids a single long
                    // swipe firing the same direction repeatedly and lets
                    // `opp_distance` alone gate same-direction re-records.
                    self.mouse_accum_dx.store(0, Ordering::Release);
                    self.mouse_accum_dy.store(0, Ordering::Release);
                }
            }

            // Update last position after detecting direction (lock-free atomic stores)
            self.last_mouse_x.store(mouse_x, Ordering::Release);
            self.last_mouse_y.store(mouse_y, Ordering::Release);

            // Check if there's an active sequence device to interrupt
            // Check if there's an active sequence device (mouse movement)
            let guard = Guard::new();
            let last_seq = self.last_sequence_device.load(Ordering::Acquire, &guard);
            if let Some(seq_device_clone) = last_seq.as_ref().map(|device| device.clone()) {
                // Only interrupt if it's a mouse move action
                if let Some(mapping_info) = self.get_input_mapping(&seq_device_clone)
                    && matches!(mapping_info.target_action, OutputAction::MouseMove(..))
                {
                    // Clear last sequence device and inputs (lock-free atomic operations)
                    let _ = self
                        .last_sequence_device
                        .swap((None, Tag::None), Ordering::Release);
                    let _ = self
                        .last_sequence_inputs
                        .swap((None, Tag::None), Ordering::Release);
                    // Send Release to stop the continuous mouse movement
                    if let Some(pool) = self.worker_pool.get() {
                        pool.dispatch(InputEvent::Released(seq_device_clone));
                    }
                }
            }
            return false; // Don't block real mouse movement
        }

        // Parse mouse button from message
        let button_opt = match message {
            WM_LBUTTONDOWN | WM_LBUTTONUP => Some(MouseButton::Left),
            WM_RBUTTONDOWN | WM_RBUTTONUP => Some(MouseButton::Right),
            WM_MBUTTONDOWN | WM_MBUTTONUP => Some(MouseButton::Middle),
            WM_XBUTTONDOWN | WM_XBUTTONUP => {
                // Extract X button identifier from high word of mouseData
                // XBUTTON1 = 1, XBUTTON2 = 2
                let x_button = (mouse_data >> 16) & 0xFFFF;
                match x_button {
                    1 => Some(MouseButton::X1),
                    2 => Some(MouseButton::X2),
                    _ => None, // Unknown X button
                }
            }
            _ => None,
        };

        if let Some(button) = button_opt {
            let device = InputDevice::Mouse(button);

            match message {
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
                    if !self.is_paused() && self.is_process_whitelisted() =>
                {
                    let now = Instant::now();

                    let sequence_match_result = self.record_and_match_sequence(device.clone(), now);

                    if let Some((matched_device, sequence_inputs)) = sequence_match_result {
                        if let Some(mapping_info) = self.get_input_mapping(&matched_device)
                            && mapping_info.is_sequence
                        {
                            self.sequence_matcher.clear_history();

                            let shared_device = Shared::new(matched_device.clone());
                            let _ = self
                                .last_sequence_device
                                .swap((Some(shared_device), Tag::None), Ordering::Release);

                            let shared_inputs = Shared::new(sequence_inputs);
                            let _ = self
                                .last_sequence_inputs
                                .swap((Some(shared_inputs), Tag::None), Ordering::Release);

                            if let Some(pool) = self.worker_pool.get() {
                                pool.dispatch(InputEvent::Pressed(matched_device));
                                should_block = true;
                            }
                        }
                    } else {
                        // No sequence matched - check if holding sequence trigger key
                        let guard = Guard::new();
                        let last_seq_device =
                            self.last_sequence_device.load(Ordering::Acquire, &guard);
                        let last_seq_inputs =
                            self.last_sequence_inputs.load(Ordering::Acquire, &guard);

                        if let (Some(seq_device), Some(seq_inputs)) =
                            (last_seq_device.as_ref(), last_seq_inputs.as_ref())
                        {
                            // Check if current device is part of the last matched sequence
                            if seq_inputs.contains(&device) {
                                // User is holding a key from the sequence
                                // Continue dispatching Pressed events for turbo
                                if let Some(pool) = self.worker_pool.get() {
                                    pool.dispatch(InputEvent::Pressed(seq_device.clone()));
                                    should_block = true;
                                }
                            } else if let Some(mapping_info) = self.get_input_mapping(&device) {
                                // Not part of sequence, handle normally
                                if !mapping_info.is_sequence
                                    && let Some(pool) = self.worker_pool.get()
                                {
                                    pool.dispatch(InputEvent::Pressed(device));
                                    should_block = true;
                                }
                            }
                        } else if let Some(mapping_info) = self.get_input_mapping(&device) {
                            // No active sequence, handle normally
                            if !mapping_info.is_sequence
                                && let Some(pool) = self.worker_pool.get()
                            {
                                pool.dispatch(InputEvent::Pressed(device));
                                should_block = true;
                            }
                        }
                    }
                }
                WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
                    let guard = Guard::new();
                    let last_seq_inputs = self.last_sequence_inputs.load(Ordering::Acquire, &guard);

                    let is_last_sequence_input = if let Some(seq_inputs) = last_seq_inputs.as_ref()
                    {
                        if let Some(last_input) = seq_inputs.last() {
                            matches!(last_input, InputDevice::Mouse(btn) if *btn == button)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_last_sequence_input {
                        // Clear sequence state when last input is released
                        let _ = self
                            .last_sequence_device
                            .swap((None, Tag::None), Ordering::Release);
                        let _ = self
                            .last_sequence_inputs
                            .swap((None, Tag::None), Ordering::Release);

                        if let Some(pool) = self.worker_pool.get() {
                            pool.dispatch(InputEvent::Released(device.clone()));
                        }
                    } else {
                        // Not a sequence device
                        if let Some(mapping_info) = self.get_input_mapping(&device) {
                            // Has mapping - check if it's sequence-only
                            if !mapping_info.is_sequence {
                                // Normal mapping (non-sequence)
                                if let Some(pool) = self.worker_pool.get() {
                                    pool.dispatch(InputEvent::Released(device));
                                }
                            }
                            // Else: sequence-only mapping - don't dispatch release
                        } else {
                            // No mapping configured - dispatch as normal input
                            if let Some(pool) = self.worker_pool.get() {
                                pool.dispatch(InputEvent::Released(device));
                            }
                        }
                    }
                    should_block = false;
                }
                _ => {}
            }
        }

        // Handle mouse wheel scroll
        const WM_MOUSEWHEEL: u32 = 0x020A;
        if message == WM_MOUSEWHEEL {
            // Mouse wheel is not intercepted by default - always return false
            return false;
        }

        should_block
    }
}
