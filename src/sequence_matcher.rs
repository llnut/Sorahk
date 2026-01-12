//! Sequence matching for fighting game-style input combos.

use crate::state::InputDevice;
use crate::util::unlikely;
use smallvec::SmallVec;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::time::{Duration, Instant};

const MAX_SEQUENCE_LENGTH: usize = 16;
const DEFAULT_SEQUENCE_WINDOW_MS: u64 = 500;
const HISTORY_BUFFER_SIZE: usize = 32;
const HISTORY_BUFFER_MASK: usize = HISTORY_BUFFER_SIZE - 1;
const DEDUP_THRESHOLD_MS: u64 = 16;

#[derive(Clone, Debug)]
#[repr(C, align(64))]
struct TimedInput {
    device: InputDevice,
    timestamp: Instant,
}

#[allow(clippy::len_without_is_empty)]
#[derive(Clone, Debug)]
pub struct InputSequence {
    inputs: Arc<SmallVec<[InputDevice; MAX_SEQUENCE_LENGTH]>>,
    window_ms: u64,
}

impl InputSequence {
    #[inline]
    pub fn new(inputs: Vec<InputDevice>, window_ms: Option<u64>) -> Self {
        Self {
            inputs: Arc::new(SmallVec::from_vec(inputs)),
            window_ms: window_ms.unwrap_or(DEFAULT_SEQUENCE_WINDOW_MS),
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inputs.len()
    }

    #[inline(always)]
    pub fn inputs(&self) -> &[InputDevice] {
        &self.inputs
    }

    #[inline(always)]
    pub fn inputs_arc(&self) -> Arc<SmallVec<[InputDevice; MAX_SEQUENCE_LENGTH]>> {
        Arc::clone(&self.inputs)
    }

    #[inline(always)]
    pub fn window_ms(&self) -> u64 {
        self.window_ms
    }
}

#[repr(C, align(64))]
struct HistorySlot {
    data: AtomicPtr<TimedInput>,
}

impl HistorySlot {
    #[inline]
    const fn new() -> Self {
        Self {
            data: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    #[inline]
    fn load(&self) -> Option<TimedInput> {
        let ptr = self.data.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            unsafe { Some((*ptr).clone()) }
        }
    }

    #[inline]
    fn store(&self, value: Option<TimedInput>) {
        let old_ptr = if let Some(input) = value {
            let new_ptr = Box::into_raw(Box::new(input));
            self.data.swap(new_ptr, Ordering::Release)
        } else {
            self.data.swap(std::ptr::null_mut(), Ordering::Release)
        };

        if !old_ptr.is_null() {
            unsafe { drop(Box::from_raw(old_ptr)) };
        }
    }
}

impl Drop for HistorySlot {
    fn drop(&mut self) {
        let ptr = self.data.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}

#[repr(C, align(64))]
pub struct SequenceMatcher {
    head: AtomicU64,
    history: [HistorySlot; HISTORY_BUFFER_SIZE],
    sequences: AtomicPtr<Arc<Vec<(InputSequence, InputDevice)>>>,
}

impl SequenceMatcher {
    #[inline]
    pub fn new() -> Self {
        Self {
            head: AtomicU64::new(0),
            history: [const { HistorySlot::new() }; HISTORY_BUFFER_SIZE],
            sequences: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    #[inline]
    pub fn register_sequence(&self, sequence: InputSequence) {
        if let Some(last_device) = sequence.inputs().last().cloned() {
            loop {
                let current_ptr = self.sequences.load(Ordering::Acquire);
                let mut new_seqs = if current_ptr.is_null() {
                    Vec::with_capacity(8)
                } else {
                    unsafe { (**current_ptr).clone() }
                };

                new_seqs.push((sequence.clone(), last_device.clone()));
                new_seqs.sort_unstable_by(|a, b| b.0.len().cmp(&a.0.len()));

                let new_ptr = Box::into_raw(Box::new(Arc::new(new_seqs)));

                match self.sequences.compare_exchange(
                    current_ptr,
                    new_ptr,
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        if !current_ptr.is_null() {
                            unsafe { drop(Box::from_raw(current_ptr)) };
                        }
                        break;
                    }
                    Err(_) => {
                        unsafe { drop(Box::from_raw(new_ptr)) };
                        continue;
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn clear_sequences(&self) {
        let old_ptr = self.sequences.swap(std::ptr::null_mut(), Ordering::Release);
        if !old_ptr.is_null() {
            unsafe { drop(Box::from_raw(old_ptr)) };
        }
    }

    #[inline(always)]
    pub fn record_input(&self, device: InputDevice, timestamp: Instant) {
        let head = self.head.fetch_add(1, Ordering::AcqRel);

        if head > 0 {
            let prev_idx = ((head - 1) as usize) & HISTORY_BUFFER_MASK;
            if let Some(prev_input) = self.history[prev_idx].load()
                && prev_input.device == device
                    && timestamp.duration_since(prev_input.timestamp)
                        < Duration::from_millis(DEDUP_THRESHOLD_MS)
                {
                    let _ = self.head.fetch_sub(1, Ordering::AcqRel);
                    return;
                }
        }

        let idx = (head as usize) & HISTORY_BUFFER_MASK;
        self.history[idx].store(Some(TimedInput { device, timestamp }));
    }

    #[inline(always)]
    pub fn try_match_with_sequence(
        &self,
    ) -> Option<(
        InputDevice,
        Arc<SmallVec<[InputDevice; MAX_SEQUENCE_LENGTH]>>,
    )> {
        let head = self.head.load(Ordering::Acquire);
        if unlikely(head == 0) {
            return None;
        }

        let sequences_ptr = self.sequences.load(Ordering::Acquire);
        if sequences_ptr.is_null() {
            return None;
        }

        let sequences = unsafe { &**sequences_ptr };

        for (sequence, last_device) in sequences.iter() {
            if self.match_sequence(sequence, head) {
                return Some((last_device.clone(), sequence.inputs_arc()));
            }
        }
        None
    }

    #[inline(always)]
    fn match_sequence(&self, sequence: &InputSequence, head: u64) -> bool {
        let seq_len = sequence.len();
        if unlikely(seq_len == 0 || head < seq_len as u64) {
            return false;
        }

        let latest_idx = ((head - 1) as usize) & HISTORY_BUFFER_MASK;
        let latest = match self.history[latest_idx].load() {
            Some(input) => input,
            None => return false,
        };

        let window = Duration::from_millis(sequence.window_ms());
        let cutoff_time = latest.timestamp.checked_sub(window);

        let mut history_offset = 0u64;
        let mut seq_idx = seq_len;

        while seq_idx > 0 {
            seq_idx -= 1;

            if head <= history_offset {
                return false;
            }

            let history_pos = ((head - 1 - history_offset) as usize) & HISTORY_BUFFER_MASK;
            let input = match self.history[history_pos].load() {
                Some(input) => input,
                None => return false,
            };

            if let Some(cutoff) = cutoff_time
                && unlikely(input.timestamp < cutoff) {
                    return false;
                }

            let expected_device = unsafe { sequence.inputs.get_unchecked(seq_idx) };

            if Self::device_matches(&input.device, expected_device) {
                history_offset += 1;
            } else if Self::is_mouse_transition_tolerable(
                &input.device,
                seq_idx,
                sequence.inputs.as_ref(),
            ) {
                history_offset += 1;
                seq_idx += 1;
            } else if Self::is_sequence_transition_skippable(
                &input.device,
                seq_idx,
                sequence.inputs.as_ref(),
            ) {
                continue;
            } else {
                return false;
            }
        }

        true
    }

    #[inline(always)]
    fn is_mouse_transition_tolerable(
        history_device: &InputDevice,
        seq_idx: usize,
        sequence: &[InputDevice],
    ) -> bool {
        // Handle mouse movement transitions
        if let InputDevice::MouseMove(history_dir) = history_device
            && seq_idx + 1 < sequence.len()
                && let (InputDevice::MouseMove(curr_dir), InputDevice::MouseMove(next_dir)) =
                    (&sequence[seq_idx], &sequence[seq_idx + 1])
                {
                    return history_dir.is_transition_between(*curr_dir, *next_dir);
                }

        // Handle XInput stick transitions
        if let InputDevice::XInputCombo {
            device_type: h_dt,
            button_ids: h_ids,
        } = history_device
            && seq_idx + 1 < sequence.len()
                && let (
                    InputDevice::XInputCombo {
                        device_type: c_dt,
                        button_ids: c_ids,
                    },
                    InputDevice::XInputCombo {
                        device_type: n_dt,
                        button_ids: n_ids,
                    },
                ) = (&sequence[seq_idx], &sequence[seq_idx + 1])
                    && h_dt == c_dt
                        && c_dt == n_dt
                        && h_ids.len() == 2
                        && c_ids.len() == 1
                        && n_ids.len() == 1
                    {
                        return Self::is_xinput_transition_between(h_ids, c_ids[0], n_ids[0]);
                    }

        false
    }

    #[inline(always)]
    fn is_sequence_transition_skippable(
        history_device: &InputDevice,
        seq_idx: usize,
        sequence: &[InputDevice],
    ) -> bool {
        // Handle mouse movement transitions
        if let InputDevice::MouseMove(seq_dir) = &sequence[seq_idx]
            && seq_idx > 0 && seq_idx + 1 < sequence.len()
                && let (InputDevice::MouseMove(prev_dir), InputDevice::MouseMove(next_dir)) =
                    (&sequence[seq_idx - 1], &sequence[seq_idx + 1])
                    && seq_dir.is_transition_between(*prev_dir, *next_dir)
                        && let InputDevice::MouseMove(history_dir) = history_device {
                            return history_dir == next_dir;
                        }

        // Handle XInput stick transitions
        if let InputDevice::XInputCombo {
            device_type: s_dt,
            button_ids: s_ids,
        } = &sequence[seq_idx]
            && seq_idx > 0 && seq_idx + 1 < sequence.len() && s_ids.len() == 2
                && let (
                    InputDevice::XInputCombo {
                        device_type: p_dt,
                        button_ids: p_ids,
                    },
                    InputDevice::XInputCombo {
                        device_type: n_dt,
                        button_ids: n_ids,
                    },
                ) = (&sequence[seq_idx - 1], &sequence[seq_idx + 1])
                    && s_dt == p_dt && p_dt == n_dt && p_ids.len() == 1 && n_ids.len() == 1
                        && Self::is_xinput_transition_between(s_ids, p_ids[0], n_ids[0])
                            && let InputDevice::XInputCombo {
                                device_type: h_dt,
                                button_ids: h_ids,
                            } = history_device
                            {
                                return h_dt == n_dt && h_ids.len() == 1 && h_ids[0] == n_ids[0];
                            }

        false
    }

    #[inline(always)]
    fn device_matches(a: &InputDevice, b: &InputDevice) -> bool {
        if std::mem::discriminant(a) != std::mem::discriminant(b) {
            return false;
        }

        match (a, b) {
            (InputDevice::Keyboard(k1), InputDevice::Keyboard(k2)) => k1 == k2,
            (InputDevice::Mouse(m1), InputDevice::Mouse(m2)) => m1 == m2,
            (InputDevice::MouseMove(d1), InputDevice::MouseMove(d2)) => d1 == d2,
            (InputDevice::KeyCombo(c1), InputDevice::KeyCombo(c2)) => c1 == c2,
            (
                InputDevice::XInputCombo {
                    device_type: dt1,
                    button_ids: b1,
                },
                InputDevice::XInputCombo {
                    device_type: dt2,
                    button_ids: b2,
                },
            ) => {
                if dt1 != dt2 {
                    return false;
                }

                // Exact match
                if b1 == b2 {
                    return true;
                }

                // For 2-button diagonals, allow reversed order
                if b1.len() == 2 && b2.len() == 2
                    && Self::is_diagonal_pair(b1) && b1[0] == b2[1] && b1[1] == b2[0] {
                        return true;
                    }

                false
            }
            (
                InputDevice::GenericDevice {
                    device_type: dt1,
                    button_id: bi1,
                },
                InputDevice::GenericDevice {
                    device_type: dt2,
                    button_id: bi2,
                },
            ) => dt1 == dt2 && bi1 == bi2,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline(always)]
    fn is_diagonal_pair(buttons: &[u32]) -> bool {
        if buttons.len() != 2 {
            return false;
        }

        let (b1, b2) = (buttons[0], buttons[1]);

        // LS: 0x10/0x11 (H) + 0x12/0x13 (V)
        if (b1 == 0x10 || b1 == 0x11) && (b2 == 0x12 || b2 == 0x13) {
            return true;
        }
        if (b2 == 0x10 || b2 == 0x11) && (b1 == 0x12 || b1 == 0x13) {
            return true;
        }

        // RS: 0x14/0x15 (H) + 0x16/0x17 (V)
        if (b1 == 0x14 || b1 == 0x15) && (b2 == 0x16 || b2 == 0x17) {
            return true;
        }
        if (b2 == 0x14 || b2 == 0x15) && (b1 == 0x16 || b1 == 0x17) {
            return true;
        }

        // D-Pad: 0x01/0x02 (V) + 0x03/0x04 (H)
        if (b1 == 0x01 || b1 == 0x02) && (b2 == 0x03 || b2 == 0x04) {
            return true;
        }
        if (b2 == 0x01 || b2 == 0x02) && (b1 == 0x03 || b1 == 0x04) {
            return true;
        }

        false
    }

    /// Checks if diagonal contains both prev and next directions
    #[inline(always)]
    fn is_xinput_transition_between(diagonal: &[u32], prev: u32, next: u32) -> bool {
        if diagonal.len() != 2 {
            return false;
        }

        let (d1, d2) = (diagonal[0], diagonal[1]);
        (d1 == prev && d2 == next) || (d1 == next && d2 == prev)
    }

    #[inline(always)]
    pub fn clear_history(&self) {
        for slot in &self.history {
            slot.store(None);
        }
        self.head.store(0, Ordering::Release);
    }

    #[cfg(test)]
    #[inline(always)]
    pub fn head(&self) -> u64 {
        self.head.load(Ordering::Acquire)
    }
}

impl Drop for SequenceMatcher {
    fn drop(&mut self) {
        let ptr = self.sequences.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}

impl Default for SequenceMatcher {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_sequence_string(s: &str, window_ms: Option<u64>) -> Result<InputSequence, String> {
    if s.is_empty() {
        return Err("Sequence cannot be empty".to_string());
    }

    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err("No inputs in sequence".to_string());
    }

    if parts.len() > MAX_SEQUENCE_LENGTH {
        return Err(format!(
            "Sequence too long (max {} inputs)",
            MAX_SEQUENCE_LENGTH
        ));
    }

    let mut inputs = Vec::with_capacity(parts.len());
    for part in parts {
        inputs.push(parse_input_device(part)?);
    }

    Ok(InputSequence::new(inputs, window_ms))
}

fn parse_input_device(s: &str) -> Result<InputDevice, String> {
    use crate::state::{AppState, MouseButton, MouseMoveDirection};

    let upper = s.to_uppercase();

    if (upper.starts_with("GAMEPAD_")
        || upper.starts_with("JOYSTICK_")
        || upper.starts_with("HID_")
        || s.contains('+'))
        && let Some(device) = AppState::parse_input_name(s) {
            return Ok(device);
        }

    match upper.as_str() {
        "MOUSE_UP" => return Ok(InputDevice::MouseMove(MouseMoveDirection::Up)),
        "MOUSE_DOWN" => return Ok(InputDevice::MouseMove(MouseMoveDirection::Down)),
        "MOUSE_LEFT" => return Ok(InputDevice::MouseMove(MouseMoveDirection::Left)),
        "MOUSE_RIGHT" => return Ok(InputDevice::MouseMove(MouseMoveDirection::Right)),
        "MOUSE_UP_LEFT" | "MOUSE_UPLEFT" => {
            return Ok(InputDevice::MouseMove(MouseMoveDirection::UpLeft));
        }
        "MOUSE_UP_RIGHT" | "MOUSE_UPRIGHT" => {
            return Ok(InputDevice::MouseMove(MouseMoveDirection::UpRight));
        }
        "MOUSE_DOWN_LEFT" | "MOUSE_DOWNLEFT" => {
            return Ok(InputDevice::MouseMove(MouseMoveDirection::DownLeft));
        }
        "MOUSE_DOWN_RIGHT" | "MOUSE_DOWNRIGHT" => {
            return Ok(InputDevice::MouseMove(MouseMoveDirection::DownRight));
        }
        "LBUTTON" | "LMB" | "LEFT_MOUSE" => return Ok(InputDevice::Mouse(MouseButton::Left)),
        "RBUTTON" | "RMB" | "RIGHT_MOUSE" => return Ok(InputDevice::Mouse(MouseButton::Right)),
        "MBUTTON" | "MMB" | "MIDDLE_MOUSE" => return Ok(InputDevice::Mouse(MouseButton::Middle)),
        "XBUTTON1" | "X1" | "MOUSE_X1" => return Ok(InputDevice::Mouse(MouseButton::X1)),
        "XBUTTON2" | "X2" | "MOUSE_X2" => return Ok(InputDevice::Mouse(MouseButton::X2)),
        "↑" | "UP" => return Ok(InputDevice::Keyboard(0x26)),
        "↓" | "DOWN" => return Ok(InputDevice::Keyboard(0x28)),
        "←" | "LEFT" => return Ok(InputDevice::Keyboard(0x25)),
        "→" | "RIGHT" => return Ok(InputDevice::Keyboard(0x27)),
        _ => {}
    }

    key_name_to_vk(&upper)
        .map(InputDevice::Keyboard)
        .ok_or_else(|| format!("Unknown input: {}", s))
}

#[inline]
fn key_name_to_vk(name: &str) -> Option<u32> {
    match name {
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
        "UP" => Some(0x26),
        "DOWN" => Some(0x28),
        "LEFT" => Some(0x25),
        "RIGHT" => Some(0x27),
        "SPACE" => Some(0x20),
        "ENTER" => Some(0x0D),
        "ESC" => Some(0x1B),
        "SHIFT" => Some(0x10),
        "CTRL" => Some(0x11),
        "ALT" => Some(0x12),
        "TAB" => Some(0x09),
        "BACKSPACE" => Some(0x08),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::MouseButton;

    #[test]
    fn test_sequence_matching_basic() {
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::Keyboard(0x28),
                InputDevice::Keyboard(0x27),
                InputDevice::Keyboard(0x41),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x28), now);
        matcher.record_input(InputDevice::Keyboard(0x27), now + Duration::from_millis(50));
        matcher.record_input(
            InputDevice::Keyboard(0x41),
            now + Duration::from_millis(100),
        );

        let result = matcher.try_match_with_sequence();
        assert!(result.is_some());
        let (last_device, sequence_inputs) = result.unwrap();
        assert_eq!(last_device, InputDevice::Keyboard(0x41));
        assert_eq!(sequence_inputs.len(), 3);
    }

    #[test]
    fn test_sequence_timeout() {
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![InputDevice::Keyboard(0x28), InputDevice::Keyboard(0x27)],
            Some(100),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x28), now);
        matcher.record_input(
            InputDevice::Keyboard(0x27),
            now + Duration::from_millis(200),
        );

        assert!(matcher.try_match_with_sequence().is_none());
    }

    #[test]
    fn test_parse_sequence() {
        let seq = parse_sequence_string("↓,→,A", Some(500)).unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.window_ms(), 500);
    }

    #[test]
    fn test_input_deduplication() {
        let matcher = SequenceMatcher::new();

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x41), now);
        matcher.record_input(InputDevice::Keyboard(0x41), now + Duration::from_millis(5));
        matcher.record_input(InputDevice::Keyboard(0x41), now + Duration::from_millis(10));

        assert_eq!(
            matcher.head(),
            1,
            "Rapid repeated inputs should be deduplicated"
        );

        matcher.record_input(InputDevice::Keyboard(0x41), now + Duration::from_millis(20));
        assert_eq!(
            matcher.head(),
            2,
            "Input after threshold should be recorded"
        );
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let matcher = SequenceMatcher::new();
        let now = Instant::now();

        for i in 0..(HISTORY_BUFFER_SIZE + 10) {
            matcher.record_input(
                InputDevice::Keyboard(0x41),
                now + Duration::from_millis(i as u64 * 20),
            );
        }

        assert_eq!(matcher.head() as usize, HISTORY_BUFFER_SIZE + 10);
    }

    #[test]
    fn test_mouse_button_sequence() {
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::Mouse(MouseButton::Left),
                InputDevice::Mouse(MouseButton::Right),
                InputDevice::Mouse(MouseButton::Middle),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::Mouse(MouseButton::Left), now);
        matcher.record_input(
            InputDevice::Mouse(MouseButton::Right),
            now + Duration::from_millis(50),
        );
        matcher.record_input(
            InputDevice::Mouse(MouseButton::Middle),
            now + Duration::from_millis(100),
        );

        let result = matcher.try_match_with_sequence();
        assert!(result.is_some());
        let (last_device, _) = result.unwrap();
        assert_eq!(last_device, InputDevice::Mouse(MouseButton::Middle));
    }

    #[test]
    fn test_mouse_movement_sequence() {
        use crate::state::MouseMoveDirection;
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::MouseMove(MouseMoveDirection::Right),
                InputDevice::MouseMove(MouseMoveDirection::Down),
                InputDevice::MouseMove(MouseMoveDirection::Right),
            ],
            Some(300),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::MouseMove(MouseMoveDirection::Right), now);
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Down),
            now + Duration::from_millis(50),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Right),
            now + Duration::from_millis(100),
        );

        assert!(matcher.try_match_with_sequence().is_some());
    }

    #[test]
    fn test_multiple_sequences() {
        let matcher = SequenceMatcher::new();

        let seq1 = InputSequence::new(
            vec![InputDevice::Keyboard(0x28), InputDevice::Keyboard(0x41)],
            Some(500),
        );
        let seq2 = InputSequence::new(
            vec![InputDevice::Keyboard(0x26), InputDevice::Keyboard(0x42)],
            Some(500),
        );
        matcher.register_sequence(seq1);
        matcher.register_sequence(seq2);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x28), now);
        matcher.record_input(InputDevice::Keyboard(0x41), now + Duration::from_millis(50));

        let result = matcher.try_match_with_sequence();
        assert!(result.is_some());
        let (last_device, _) = result.unwrap();
        assert_eq!(last_device, InputDevice::Keyboard(0x41));

        matcher.clear_history();
        matcher.record_input(InputDevice::Keyboard(0x26), now);
        matcher.record_input(InputDevice::Keyboard(0x42), now + Duration::from_millis(50));

        let result = matcher.try_match_with_sequence();
        assert!(result.is_some());
        let (last_device, _) = result.unwrap();
        assert_eq!(last_device, InputDevice::Keyboard(0x42));
    }

    #[test]
    fn test_partial_sequence_no_match() {
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::Keyboard(0x28),
                InputDevice::Keyboard(0x27),
                InputDevice::Keyboard(0x41),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x28), now);
        matcher.record_input(InputDevice::Keyboard(0x27), now + Duration::from_millis(50));

        assert!(matcher.try_match_with_sequence().is_none());
    }

    #[test]
    fn test_wrong_order_no_match() {
        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![InputDevice::Keyboard(0x28), InputDevice::Keyboard(0x27)],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x27), now);
        matcher.record_input(InputDevice::Keyboard(0x28), now + Duration::from_millis(50));

        assert!(matcher.try_match_with_sequence().is_none());
    }

    #[test]
    fn test_clear_history() {
        let matcher = SequenceMatcher::new();

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x41), now);
        matcher.record_input(InputDevice::Keyboard(0x42), now + Duration::from_millis(50));

        matcher.clear_history();
        assert_eq!(matcher.head(), 0);
    }

    #[test]
    fn test_parse_sequence_string() {
        let seq = parse_sequence_string("A,B,C", Some(300)).unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.window_ms(), 300);

        let seq = parse_sequence_string("↓,→,A", Some(500)).unwrap();
        assert_eq!(seq.len(), 3);

        let seq = parse_sequence_string("MOUSE_UP,MOUSE_RIGHT", Some(200)).unwrap();
        assert_eq!(seq.len(), 2);

        assert!(parse_sequence_string("", Some(500)).is_err());
    }

    #[test]
    fn test_longest_sequence_first() {
        let matcher = SequenceMatcher::new();

        let short_seq = InputSequence::new(
            vec![InputDevice::Keyboard(0x28), InputDevice::Keyboard(0x41)],
            Some(500),
        );
        matcher.register_sequence(short_seq);

        let long_seq = InputSequence::new(
            vec![
                InputDevice::Keyboard(0x28),
                InputDevice::Keyboard(0x27),
                InputDevice::Keyboard(0x41),
            ],
            Some(500),
        );
        matcher.register_sequence(long_seq);

        let now = Instant::now();
        matcher.record_input(InputDevice::Keyboard(0x28), now);
        matcher.record_input(InputDevice::Keyboard(0x27), now + Duration::from_millis(50));
        matcher.record_input(
            InputDevice::Keyboard(0x41),
            now + Duration::from_millis(100),
        );

        let result = matcher.try_match_with_sequence();
        assert!(result.is_some());
        let (_, sequence_inputs) = result.unwrap();
        assert_eq!(
            sequence_inputs.len(),
            3,
            "Should match longer sequence first"
        );
    }

    #[test]
    fn test_mouse_direction_transition_tolerance() {
        use crate::state::MouseMoveDirection;

        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::MouseMove(MouseMoveDirection::Down),
                InputDevice::MouseMove(MouseMoveDirection::Left),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::MouseMove(MouseMoveDirection::Down), now);
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::DownLeft),
            now + Duration::from_millis(30),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Left),
            now + Duration::from_millis(60),
        );

        assert!(
            matcher.try_match_with_sequence().is_some(),
            "Should match DOWN->LEFT even with DOWN_LEFT transition"
        );
    }

    #[test]
    fn test_mouse_direction_non_transition_rejected() {
        use crate::state::MouseMoveDirection;

        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::MouseMove(MouseMoveDirection::Down),
                InputDevice::MouseMove(MouseMoveDirection::Left),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::MouseMove(MouseMoveDirection::Down), now);
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::UpLeft),
            now + Duration::from_millis(30),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Left),
            now + Duration::from_millis(60),
        );

        assert!(
            matcher.try_match_with_sequence().is_none(),
            "Should NOT match when intermediate direction is not a valid transition"
        );
    }

    #[test]
    fn test_mouse_direction_multiple_transitions() {
        use crate::state::MouseMoveDirection;

        let matcher = SequenceMatcher::new();

        let sequence = InputSequence::new(
            vec![
                InputDevice::MouseMove(MouseMoveDirection::Down),
                InputDevice::MouseMove(MouseMoveDirection::Left),
                InputDevice::MouseMove(MouseMoveDirection::Up),
            ],
            Some(500),
        );
        matcher.register_sequence(sequence);

        let now = Instant::now();
        matcher.record_input(InputDevice::MouseMove(MouseMoveDirection::Down), now);
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::DownLeft),
            now + Duration::from_millis(20),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Left),
            now + Duration::from_millis(40),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::UpLeft),
            now + Duration::from_millis(60),
        );
        matcher.record_input(
            InputDevice::MouseMove(MouseMoveDirection::Up),
            now + Duration::from_millis(80),
        );

        assert!(
            matcher.try_match_with_sequence().is_some(),
            "Should match sequence with multiple transition tolerances"
        );
    }
}
