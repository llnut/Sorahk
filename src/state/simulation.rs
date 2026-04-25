use std::sync::atomic::Ordering;

use smallvec::SmallVec;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::AppState;
use super::types::*;

impl AppState {
    /// Full press + hold + release cycle. Dispatches to the right primitive
    /// based on action variant.
    #[inline]
    pub fn simulate_action(&self, action: OutputAction, duration: u64) {
        match action {
            OutputAction::MouseMove(direction, speed) => {
                Self::send_mouse_move(direction, speed);
            }
            OutputAction::MouseScroll(direction, speed) => {
                Self::send_mouse_scroll(direction, speed);
            }
            OutputAction::SequentialActions(actions, interval_ms) => {
                let last = actions.len().saturating_sub(1);
                for (idx, a) in actions.iter().enumerate() {
                    self.simulate_action(a.clone(), duration);
                    if idx < last {
                        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    }
                }
            }
            OutputAction::MappingHold {
                actions,
                interval_ms,
                hold_mask,
                append,
                sequential,
            } => {
                if sequential {
                    // Sequential playback (Sequence target mode). Held
                    // indices only press; non-held indices run a full
                    // press + hold + release cycle on their own.
                    let last = actions.len().saturating_sub(1);
                    for (idx, a) in actions.iter().enumerate() {
                        if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                            self.simulate_initial_press(a);
                        } else {
                            self.simulate_action(a.clone(), duration);
                        }
                        if idx < last {
                            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                        }
                    }
                } else {
                    // Simultaneous playback (Single / Multi target mode).
                    // Press everything at once, sleep `duration`, then
                    // release only the non-held indices. Held indices
                    // stay pressed for the append phase.
                    for a in actions.iter() {
                        self.simulate_initial_press(a);
                    }
                    if duration > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(duration));
                    }
                    for (idx, a) in actions.iter().enumerate().rev() {
                        let held = idx < 16 && (hold_mask & (1u16 << idx)) != 0;
                        if !held {
                            self.simulate_release(a);
                        }
                    }
                }
                // Append phase identical for both playback shapes. Holdable
                // items stay held; edge-event items (MouseMove / Scroll)
                // pulse once.
                for a in append.iter() {
                    if Self::is_holdable_primitive(a) {
                        self.simulate_initial_press(a);
                    } else {
                        self.simulate_action(a.clone(), 0);
                    }
                }
            }
            _ => {
                // Held-style actions: KeyboardKey / KeyCombo / MouseButton /
                // MultipleActions. The press + release halves go through the
                // ref-counted helpers so overlapping triggers can share keys
                // without releasing each other's holds prematurely.
                self.simulate_initial_press(&action);
                std::thread::sleep(std::time::Duration::from_millis(duration));
                self.simulate_release(&action);
            }
        }
    }

    /// Presses every primitive in `action`, incrementing per-key ref counts.
    /// Only emits `KEYDOWN` / `MOUSEDOWN` on a 0 -> 1 transition, so a key
    /// already held by another active trigger stays held.
    #[inline]
    pub fn simulate_initial_press(&self, action: &OutputAction) {
        let mut scs: SmallVec<[u16; 8]> = SmallVec::new();
        let mut btns: SmallVec<[MouseButton; 4]> = SmallVec::new();
        Self::collect_primitives(action, &mut scs, &mut btns);

        let mut inputs: SmallVec<[INPUT; 12]> = SmallVec::new();
        for &scancode in &scs {
            let idx = scancode as usize;
            if idx >= 256 {
                continue;
            }
            if self.held_scancodes[idx].fetch_add(1, Ordering::AcqRel) == 0 {
                inputs.push(Self::build_key_input(scancode, false));
            }
        }
        for &btn in &btns {
            if self.held_mouse_buttons[btn as usize].fetch_add(1, Ordering::AcqRel) == 0 {
                inputs.push(Self::build_mouse_input(btn, false));
            }
        }

        if !inputs.is_empty() {
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    /// Sends `KEYDOWN` for every held primitive in `action` without touching
    /// the ref counts. Used to drive Windows' own auto-repeat stream for a
    /// held key; every call looks to the OS like a fresh keydown that
    /// produces one auto-repeat event.
    #[inline]
    pub fn simulate_repeat_press(&self, action: &OutputAction) {
        let mut scs: SmallVec<[u16; 8]> = SmallVec::new();
        let mut btns: SmallVec<[MouseButton; 4]> = SmallVec::new();
        Self::collect_primitives(action, &mut scs, &mut btns);

        let mut inputs: SmallVec<[INPUT; 12]> = SmallVec::new();
        for &scancode in &scs {
            inputs.push(Self::build_key_input(scancode, false));
        }
        for &btn in &btns {
            inputs.push(Self::build_mouse_input(btn, false));
        }

        if !inputs.is_empty() {
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    /// Releases every primitive in `action`, decrementing per-key ref counts.
    /// Only emits `KEYUP` / `MOUSEUP` on the N -> 0 transition, so a key
    /// still held by another active trigger remains pressed.
    #[inline]
    pub fn simulate_release(&self, action: &OutputAction) {
        let mut scs: SmallVec<[u16; 8]> = SmallVec::new();
        let mut btns: SmallVec<[MouseButton; 4]> = SmallVec::new();
        Self::collect_primitives(action, &mut scs, &mut btns);

        let mut inputs: SmallVec<[INPUT; 12]> = SmallVec::new();
        // Reverse order matches the historical KeyCombo release-in-reverse
        // behavior so modifiers outlive their main key.
        for &scancode in scs.iter().rev() {
            let idx = scancode as usize;
            if idx >= 256 {
                debug_assert!(
                    false,
                    "scancode >= 256 should never be generated by parsing"
                );
                continue;
            }
            if Self::ref_decrement_saturating(&self.held_scancodes[idx]) {
                inputs.push(Self::build_key_input(scancode, true));
            }
        }
        for &btn in btns.iter().rev() {
            if Self::ref_decrement_saturating(&self.held_mouse_buttons[btn as usize]) {
                inputs.push(Self::build_mouse_input(btn, true));
            }
        }

        if !inputs.is_empty() {
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    /// Atomically decrements a non-negative ref count, returning `true` when
    /// the transition was `1 → 0` (the caller should emit the release event).
    /// Uses a compare-exchange loop so concurrent `fetch_add` on the same
    /// counter never observes a transient negative value. Ref counts that are
    /// already at 0 stay at 0 and the call returns `false`.
    #[inline(always)]
    fn ref_decrement_saturating(cell: &std::sync::atomic::AtomicI32) -> bool {
        let mut curr = cell.load(Ordering::Acquire);
        loop {
            if curr <= 0 {
                // Already released by a concurrent reset or underflow path;
                // don't emit and don't let the counter go negative.
                return false;
            }
            match cell.compare_exchange_weak(curr, curr - 1, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return curr == 1,
                Err(actual) => curr = actual,
            }
        }
    }

    /// Walks the action tree and collects every keyboard scancode and mouse
    /// button primitive. Movement / scroll primitives are intentionally
    /// dropped: they have no persistent held state to ref-count.
    fn collect_primitives(
        action: &OutputAction,
        scs: &mut SmallVec<[u16; 8]>,
        btns: &mut SmallVec<[MouseButton; 4]>,
    ) {
        match action {
            OutputAction::KeyboardKey(scancode) => scs.push(*scancode),
            OutputAction::KeyCombo(scancodes) => {
                for &s in scancodes.iter() {
                    scs.push(s);
                }
            }
            OutputAction::MouseButton(button) => btns.push(*button),
            OutputAction::MultipleActions(nested) => {
                for a in nested.iter() {
                    Self::collect_primitives(a, scs, btns);
                }
            }
            OutputAction::SequentialActions(nested, _) => {
                for a in nested.iter() {
                    Self::collect_primitives(a, scs, btns);
                }
            }
            OutputAction::MappingHold {
                actions,
                hold_mask,
                append,
                ..
            } => {
                // Only collect primitives that outlive the sequence pass:
                // the held subset plus the append list. Non-held body
                // actions self-balance their press+release inside
                // simulate_action and leave ref counts at zero.
                for (idx, a) in actions.iter().enumerate() {
                    if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                        Self::collect_primitives(a, scs, btns);
                    }
                }
                for a in append.iter() {
                    Self::collect_primitives(a, scs, btns);
                }
            }
            OutputAction::MouseMove(..) | OutputAction::MouseScroll(..) => {
                // Edge-event primitives: no held state.
            }
        }
    }

    /// Classifies whether an action produces held keyboard / mouse button
    /// primitives. Used by MappingHold to split append actions into
    /// "press and keep held" vs "fire once as edge event".
    #[inline(always)]
    fn is_holdable_primitive(action: &OutputAction) -> bool {
        match action {
            OutputAction::KeyboardKey(_)
            | OutputAction::KeyCombo(_)
            | OutputAction::MouseButton(_) => true,
            OutputAction::MultipleActions(nested) => nested.iter().all(Self::is_holdable_primitive),
            _ => false,
        }
    }

    /// Re-emits KEYDOWN / MOUSEDOWN for the held subset of a
    /// `MappingHold`, without touching ref counts. Called on every
    /// Windows auto-repeat tick (or synthetic repeat for non-keyboard
    /// triggers) so the host OS keeps the held keys "warm" without
    /// replaying the whole sequence.
    #[inline]
    pub fn simulate_hold_repeat(&self, action: &OutputAction) {
        if let OutputAction::MappingHold {
            actions,
            hold_mask,
            append,
            ..
        } = action
        {
            let mut scs: SmallVec<[u16; 8]> = SmallVec::new();
            let mut btns: SmallVec<[MouseButton; 4]> = SmallVec::new();
            for (idx, a) in actions.iter().enumerate() {
                if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                    Self::collect_primitives(a, &mut scs, &mut btns);
                }
            }
            for a in append.iter() {
                Self::collect_primitives(a, &mut scs, &mut btns);
            }

            if scs.is_empty() && btns.is_empty() {
                return;
            }

            let mut inputs: SmallVec<[INPUT; 12]> = SmallVec::new();
            for &sc in &scs {
                inputs.push(Self::build_key_input(sc, false));
            }
            for &btn in &btns {
                inputs.push(Self::build_mouse_input(btn, false));
            }
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    /// Runs one full press + hold + release cycle over the hold subset
    /// and the append list of a `MappingHold`. Used by the turbo
    /// worker so turbo-mode MappingHold targets pulse only the
    /// user-configured "sequence properties" keys rather than looping
    /// the whole sequence body. Body actions outside the hold subset
    /// are ignored, which is what enables "double-tap to run then turbo
    /// the held portion" style configurations.
    ///
    /// `duration` is the hold time; the caller controls the gap
    /// between cycles via its own interval pacing.
    #[inline]
    pub fn simulate_hold_cycle(&self, action: &OutputAction, duration: u64) {
        if let OutputAction::MappingHold {
            actions,
            hold_mask,
            append,
            ..
        } = action
        {
            // Press phase.
            for (idx, a) in actions.iter().enumerate() {
                if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                    self.simulate_initial_press(a);
                }
            }
            for a in append.iter() {
                if Self::is_holdable_primitive(a) {
                    self.simulate_initial_press(a);
                } else {
                    // Edge-event append (MouseMove / MouseScroll) fires
                    // once per cycle — no held state to release later.
                    self.simulate_action(a.clone(), 0);
                }
            }

            if duration > 0 {
                std::thread::sleep(std::time::Duration::from_millis(duration));
            }

            // Release phase: body held set first (reversed), then append
            // in reverse. Append items are typically modifier-like extras
            // (LSHIFT, LCTRL) that users add to qualify the body, so they
            // should outlive the body keys on release — releasing body
            // first guarantees "Shift+A" tails with Shift still down for
            // one tick, matching the ergonomic expectation.
            for (idx, a) in actions.iter().enumerate().rev() {
                if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                    self.simulate_release(a);
                }
            }
            for a in append.iter().rev() {
                if Self::is_holdable_primitive(a) {
                    self.simulate_release(a);
                }
            }
        }
    }

    /// Decrements the ref count for every action held by a
    /// `MappingHold` and emits KEYUP / MOUSEUP on the final release.
    /// Body held items release first (reversed), then append in reverse,
    /// so modifier-like append keys outlive their body main key.
    #[inline]
    pub fn simulate_hold_release(&self, action: &OutputAction) {
        if let OutputAction::MappingHold {
            actions,
            hold_mask,
            append,
            ..
        } = action
        {
            for (idx, a) in actions.iter().enumerate().rev() {
                if idx < 16 && (hold_mask & (1u16 << idx)) != 0 {
                    self.simulate_release(a);
                }
            }
            for a in append.iter().rev() {
                if Self::is_holdable_primitive(a) {
                    self.simulate_release(a);
                }
            }
        }
    }

    #[inline(always)]
    pub(super) fn build_key_input(scancode: u16, up: bool) -> INPUT {
        let mut flags = KEYEVENTF_SCANCODE;
        if Self::is_extended_scancode(scancode) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        if up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
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
        }
    }

    #[inline(always)]
    pub(super) fn build_mouse_input(button: MouseButton, up: bool) -> INPUT {
        let (down_flag, up_flag) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            MouseButton::X1 | MouseButton::X2 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP),
        };
        let mouse_data = match button {
            MouseButton::X1 => 1u32,
            MouseButton::X2 => 2u32,
            _ => 0,
        };
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data,
                    dwFlags: if up { up_flag } else { down_flag },
                    time: 0,
                    dwExtraInfo: SIMULATED_EVENT_MARKER,
                },
            },
        }
    }

    #[inline(always)]
    fn send_mouse_move(direction: MouseMoveDirection, speed: i32) {
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
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    #[inline(always)]
    fn send_mouse_scroll(direction: MouseScrollDirection, speed: i32) {
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
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
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
}
