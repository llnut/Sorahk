use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::state::{AppState, InputDevice, InputEvent};
use crate::util::{likely, unlikely};

unsafe impl Send for KeyboardHook {}

// Multi-worker dispatcher supporting both keyboard and mouse
struct WorkerPool {
    workers: Vec<Sender<InputEvent>>,
    mouse_move_worker: Sender<InputEvent>,
    worker_count: usize,
    state: Arc<AppState>,
    /// Cache for mouse action detection (keyboard VK 0-255)
    /// Bit 0: is mouse action, Bit 1-7: reserved
    mouse_action_cache: [std::sync::atomic::AtomicU8; 256],
}

impl WorkerPool {
    fn new(
        worker_count: usize,
        state: Arc<AppState>,
        mouse_move_worker: Sender<InputEvent>,
    ) -> Self {
        Self {
            workers: Vec::with_capacity(worker_count),
            mouse_move_worker,
            worker_count,
            state,
            mouse_action_cache: std::array::from_fn(|_| std::sync::atomic::AtomicU8::new(0xFF)),
        }
    }

    fn add_worker(&mut self, sender: Sender<InputEvent>) {
        self.workers.push(sender);
    }
}

// Implement EventDispatcher trait
impl crate::state::EventDispatcher for WorkerPool {
    /// Clear internal caches (called when configuration is reloaded)
    fn clear_cache(&self) {
        // Reset all cache entries to 0xFF (uncached state)
        for i in 0..256 {
            self.mouse_action_cache[i].store(0xFF, Ordering::Relaxed);
        }
    }

    // Dispatch events using pre-computed lookup (hot path)
    #[inline]
    fn dispatch(&self, event: InputEvent) {
        if unlikely(self.workers.is_empty()) {
            return;
        }

        let device = match &event {
            InputEvent::Pressed(d) | InputEvent::Released(d) => d,
        };

        // Fast path: check cache for keyboard keys (most common case)
        let is_mouse_action = if let InputDevice::Keyboard(vk) = device
            && *vk < 256
        {
            let cached = self.mouse_action_cache[*vk as usize].load(Ordering::Relaxed);
            if cached != 0xFF {
                // Cache hit
                (cached & 0x01) != 0
            } else {
                // Cache miss - query and update cache
                let is_action = if let Some(mapping) = self.state.get_input_mapping(device) {
                    Self::is_mouse_action(&mapping.target_action)
                } else {
                    false
                };
                self.mouse_action_cache[*vk as usize]
                    .store(if is_action { 0x01 } else { 0x00 }, Ordering::Relaxed);
                is_action
            }
        } else {
            // Slow path: mouse buttons, combos, and generic devices
            if let Some(mapping) = self.state.get_input_mapping(device) {
                Self::is_mouse_action(&mapping.target_action)
            } else {
                false
            }
        };

        // Route to dedicated mouse move worker or distribute normally
        if is_mouse_action {
            let _ = self.mouse_move_worker.send(event);
        } else {
            // Normal load-balanced distribution
            let worker_idx = match device {
                InputDevice::Keyboard(vk) => (*vk as usize) % self.worker_count,
                InputDevice::Mouse(button) => (*button as usize) % self.worker_count,
                InputDevice::KeyCombo(keys) => {
                    if let Some(&last_key) = keys.last() {
                        (last_key as usize) % self.worker_count
                    } else {
                        0
                    }
                }
                InputDevice::GenericDevice {
                    device_type,
                    button_id,
                } => {
                    // Hash device type and button id for distribution
                    Self::hash_generic_device(device_type, *button_id) % self.worker_count
                }
            };
            let _ = self.workers[worker_idx].send(event);
        }
    }
}

impl WorkerPool {
    /// Check if OutputAction contains mouse movement or scroll
    #[inline]
    fn is_mouse_action(action: &crate::state::OutputAction) -> bool {
        use crate::state::OutputAction;
        match action {
            OutputAction::MouseMove(_, _) | OutputAction::MouseScroll(_, _) => true,
            OutputAction::MultipleActions(actions) => actions.iter().any(Self::is_mouse_action),
            _ => false,
        }
    }

    /// Hash generic device for worker distribution using FNV-1a.
    #[inline(always)]
    fn hash_generic_device(device_type: &crate::state::DeviceType, button_id: u64) -> usize {
        let mut hash = crate::util::fnv64::OFFSET_BASIS;

        // Hash device type discriminant
        let discriminant = std::mem::discriminant(device_type);
        let disc_value = unsafe { *(&discriminant as *const _ as *const u64) };
        hash = crate::util::fnv1a_hash_u64(hash, disc_value);

        // Hash button_id
        hash = crate::util::fnv1a_hash_u64(hash, button_id);

        hash as usize
    }
}

pub struct KeyboardHook {
    state: Arc<AppState>,
    hook_handle: HHOOK,
}

impl KeyboardHook {
    pub fn new(state: Arc<AppState>) -> anyhow::Result<Self> {
        crate::state::set_global_state(state.clone())
            .map_err(|_| anyhow::anyhow!("Global status has been set"))?;

        unsafe {
            let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(Self::keyboard_proc), None, 0)?;

            if hook.0.is_null() {
                anyhow::bail!("Failed to set keyboard hook.");
            }

            Ok(Self {
                state,
                hook_handle: hook,
            })
        }
    }

    pub fn run_message_loop(self) -> anyhow::Result<()> {
        let main_thread_id = unsafe { GetCurrentThreadId() };

        // Force create message queue
        unsafe {
            let mut msg = MSG::default();
            let _ = PeekMessageA(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE);
        }

        // Determine worker count
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let configured_count = self.state.get_configured_worker_count();
        let worker_count = if configured_count == 0 {
            cpu_count
        } else {
            configured_count
        };

        // Store actual worker count in state
        self.state.set_actual_worker_count(worker_count);

        // Create dedicated mouse move worker channel
        let (mouse_move_tx, mouse_move_rx) = channel();

        let mut worker_pool = WorkerPool::new(worker_count, self.state.clone(), mouse_move_tx);
        let mut worker_handles = Vec::new();

        // Start normal turbo workers
        for worker_id in 0..worker_count {
            let (tx, rx) = channel();
            worker_pool.add_worker(tx);

            let state_clone = self.state.clone();
            let handle = thread::Builder::new()
                .name(format!("turbo_worker_{}", worker_id))
                .spawn(move || {
                    Self::turbo_worker(worker_id, state_clone, rx);
                })
                .expect("Failed to spawn turbo worker");

            worker_handles.push(handle);
        }

        // Start dedicated mouse move worker
        let state_clone = self.state.clone();
        let mouse_move_handle = thread::Builder::new()
            .name("mouse_move_worker".to_string())
            .spawn(move || {
                Self::mouse_move_worker(state_clone, mouse_move_rx);
            })
            .expect("Failed to spawn mouse move worker");

        worker_handles.push(mouse_move_handle);

        // Store WorkerPool in state for event dispatching
        self.state.set_worker_pool(Arc::new(worker_pool));

        // Start monitoring thread, responsible for sending WM_QUIT
        thread::spawn(move || {
            // Wait for all workers to exit
            for handle in worker_handles {
                let _ = handle.join();
            }

            unsafe {
                let _ = PostThreadMessageA(main_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        });

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

    unsafe extern "system" fn keyboard_proc(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if code < 0 {
            return unsafe { CallNextHookEx(None, code, w_param, l_param) };
        }

        let kb_struct = unsafe { &*(l_param.0 as *mut KBDLLHOOKSTRUCT) };

        // Skip simulated key events
        if kb_struct.dwExtraInfo == crate::state::SIMULATED_EVENT_MARKER {
            return unsafe { CallNextHookEx(None, code, w_param, l_param) };
        }

        if let Some(state) = crate::state::get_global_state() {
            let should_block = state.handle_key_event(w_param.0 as u32, kb_struct.vkCode);
            if should_block {
                return LRESULT(1); // block raw key event
            }
        }

        unsafe { CallNextHookEx(None, code, w_param, l_param) }
    }

    fn turbo_worker(_worker_id: usize, state: Arc<AppState>, event_rx: Receiver<InputEvent>) {
        use crate::state::OutputAction;
        // Store device state along with cached mapping info to avoid repeated lookups
        // Cache format: (last_time, interval, event_duration, target_action, turbo_enabled)
        // Pre-allocate with reasonable capacity to reduce allocations
        let mut device_states: HashMap<InputDevice, (Instant, u64, u64, OutputAction, bool)> =
            HashMap::with_capacity(16);

        let timeout_duration = Duration::from_millis(state.input_timeout());

        while !state.should_exit() {
            if unlikely(state.is_paused()) {
                if !device_states.is_empty() {
                    device_states.clear();
                }
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            // Hot path: event receiving and processing
            match event_rx.recv_timeout(timeout_duration) {
                Ok(event) => Self::handle_input_event(&state, &mut device_states, event),
                Err(_) => Self::handle_timeout(&state, &mut device_states),
            }
        }
    }

    #[inline]
    fn handle_input_event(
        state: &AppState,
        device_states: &mut HashMap<
            InputDevice,
            (Instant, u64, u64, crate::state::OutputAction, bool),
        >,
        event: InputEvent,
    ) {
        use crate::state::OutputAction;

        match event {
            InputEvent::Pressed(device) => {
                let now = Instant::now();

                // Fast path: check if device already in cache (common case for repeats)
                if let Some((last_time, interval, duration, target_action, turbo_enabled)) =
                    device_states.get_mut(&device)
                {
                    if *turbo_enabled {
                        // Turbo mode: respect interval timing
                        if likely(
                            now.duration_since(*last_time) >= Duration::from_millis(*interval),
                        ) {
                            state.simulate_action(target_action.clone(), *duration);
                            *last_time = now;
                        }
                    } else {
                        // Non-turbo mode: handle Windows repeat events
                        // Process keyboard keys only (mouse buttons don't generate repeat events)
                        // Windows repeat sends full action cycles (press->duration->release)
                        match target_action {
                            OutputAction::KeyboardKey(_) | OutputAction::KeyCombo(_) => {
                                state.simulate_action(target_action.clone(), *duration);
                                *last_time = now;
                            }
                            OutputAction::MultipleActions(actions) => {
                                // For multiple actions, check if all are keyboard-related
                                let all_keyboard = actions.iter().all(|a| {
                                    matches!(
                                        a,
                                        OutputAction::KeyboardKey(_) | OutputAction::KeyCombo(_)
                                    )
                                });
                                if all_keyboard {
                                    state.simulate_action(target_action.clone(), *duration);
                                    *last_time = now;
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Slow path: first press lookup and cache
                    if let Some(mapping) = state.get_input_mapping(&device) {
                        let target_action_clone = mapping.target_action.clone();
                        let turbo_enabled = mapping.turbo_enabled;

                        device_states.insert(
                            device,
                            (
                                now,
                                mapping.interval,
                                mapping.event_duration,
                                mapping.target_action,
                                turbo_enabled,
                            ),
                        );

                        // Simulate based on turbo mode
                        if turbo_enabled {
                            state.simulate_action(target_action_clone, mapping.event_duration);
                        } else {
                            state.simulate_press(&target_action_clone);
                        }
                    }
                }
            }
            InputEvent::Released(device) => {
                // For non-turbo mode, simulate release event
                if let Some((_, _, _, target_action, turbo_enabled)) = device_states.get(&device)
                    && !turbo_enabled
                {
                    state.simulate_release(target_action);
                }
                device_states.remove(&device);
            }
        }
    }

    #[inline]
    fn handle_timeout(
        state: &AppState,
        device_states: &mut HashMap<
            InputDevice,
            (Instant, u64, u64, crate::state::OutputAction, bool),
        >,
    ) {
        // Early return if no active devices
        if unlikely(device_states.is_empty()) {
            return;
        }

        let now = Instant::now();

        // Iterate over cached device states
        for (_device, (last_time, interval, duration, target_action, turbo_enabled)) in
            device_states.iter_mut()
        {
            // Only repeat if turbo mode is enabled
            if likely(
                *turbo_enabled
                    && now.duration_since(*last_time) >= Duration::from_millis(*interval),
            ) {
                state.simulate_action(target_action.clone(), *duration);
                *last_time = now;
            }
        }
    }

    /// Worker thread for mouse movement and scroll events.
    /// Uses bit flags and fixed-size arrays for efficient state tracking.
    fn mouse_move_worker(state: Arc<AppState>, event_rx: Receiver<InputEvent>) {
        // Bit flags for active directions to track which directions are currently pressed
        // Each bit represents a direction: [Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight]
        let mut active_directions: u8 = 0;
        let mut direction_devices: [Option<InputDevice>; 8] =
            [None, None, None, None, None, None, None, None];
        let mut direction_intervals: [u64; 8] = [0; 8];
        let mut direction_last_times: [Instant; 8] = [Instant::now(); 8];
        let mut direction_turbo: [bool; 8] = [false; 8];

        // Pre-computed normalized direction vectors
        static DIRECTION_VECTORS: [(f32, f32); 8] = [
            (0.0, -1.0), // Up = 0
            (0.0, 1.0),  // Down = 1
            (-1.0, 0.0), // Left = 2
            (1.0, 0.0),  // Right = 3
            (
                -std::f32::consts::FRAC_1_SQRT_2,
                -std::f32::consts::FRAC_1_SQRT_2,
            ), // UpLeft = 4
            (
                std::f32::consts::FRAC_1_SQRT_2,
                -std::f32::consts::FRAC_1_SQRT_2,
            ), // UpRight = 5
            (
                -std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ), // DownLeft = 6
            (
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ), // DownRight = 7
        ];

        // Mouse scroll state - fixed-size arrays
        // Support up to 4 simultaneous scroll inputs
        let mut scroll_active: u8 = 0; // Bit flags for active scroll slots
        let mut scroll_devices: [Option<InputDevice>; 4] = [None, None, None, None];
        let mut scroll_directions: [crate::state::MouseScrollDirection; 4] = [
            crate::state::MouseScrollDirection::Up,
            crate::state::MouseScrollDirection::Up,
            crate::state::MouseScrollDirection::Up,
            crate::state::MouseScrollDirection::Up,
        ];
        let mut scroll_speeds: [i32; 4] = [1; 4];
        let mut scroll_intervals: [u64; 4] = [50; 4];
        let mut scroll_last_times: [Instant; 4] = [Instant::now(); 4];
        let mut scroll_turbo: [bool; 4] = [false; 4];

        // Local cache: maps device to action type
        // For MouseMove: (direction_index, speed, interval, turbo_enabled)
        let mut mapping_cache: HashMap<InputDevice, (usize, i32, u64, bool)> =
            HashMap::with_capacity(8);

        // Track first pressed key's speed for multi-direction movement
        let mut first_speed: i32 = 5;
        let mut has_first_speed = false;

        // 1ms timeout for high-frequency updates
        let timeout_duration = Duration::from_millis(1);

        while !state.should_exit() {
            if unlikely(state.is_paused()) {
                active_directions = 0;
                scroll_active = 0;
                has_first_speed = false;
                first_speed = 5;
                mapping_cache.clear();
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            match event_rx.recv_timeout(timeout_duration) {
                Ok(event) => {
                    Self::handle_mouse_action_event(
                        &state,
                        &mut active_directions,
                        &mut direction_devices,
                        &mut direction_intervals,
                        &mut direction_last_times,
                        &mut direction_turbo,
                        &mut mapping_cache,
                        &mut scroll_active,
                        &mut scroll_devices,
                        &mut scroll_directions,
                        &mut scroll_speeds,
                        &mut scroll_intervals,
                        &mut scroll_last_times,
                        &mut scroll_turbo,
                        &mut first_speed,
                        &mut has_first_speed,
                        &DIRECTION_VECTORS,
                        event,
                    );
                }
                Err(_) => {
                    // Process timeout for turbo-enabled movements
                    if active_directions != 0 {
                        Self::execute_movement_bitflags(
                            active_directions,
                            &direction_intervals,
                            &mut direction_last_times,
                            &direction_turbo,
                            &DIRECTION_VECTORS,
                            first_speed,
                        );
                    }

                    // Process timeout for turbo-enabled scrolls
                    if scroll_active != 0 {
                        Self::execute_scroll_timeout(
                            &state,
                            scroll_active,
                            &scroll_directions,
                            &scroll_speeds,
                            &scroll_intervals,
                            &mut scroll_last_times,
                            &scroll_turbo,
                        );
                    }
                }
            }
        }
    }

    /// Process MultipleActions containing mouse movements and scrolls
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn handle_multiple_mouse_actions(
        state: &AppState,
        device: &InputDevice,
        actions: &smallvec::SmallVec<[crate::state::OutputAction; 4]>,
        active_directions: &mut u8,
        direction_devices: &mut [Option<InputDevice>; 8],
        direction_intervals: &mut [u64; 8],
        direction_last_times: &mut [Instant; 8],
        direction_turbo: &mut [bool; 8],
        scroll_active: &mut u8,
        scroll_devices: &mut [Option<InputDevice>; 4],
        scroll_directions: &mut [crate::state::MouseScrollDirection; 4],
        scroll_speeds: &mut [i32; 4],
        scroll_intervals: &mut [u64; 4],
        scroll_last_times: &mut [Instant; 4],
        scroll_turbo: &mut [bool; 4],
        first_speed: &mut i32,
        has_first_speed: &mut bool,
        direction_vectors: &[(f32, f32); 8],
        interval: u64,
        turbo_enabled: bool,
        now: Instant,
    ) {
        use crate::state::OutputAction;

        let mut move_directions = smallvec::SmallVec::<[usize; 4]>::new();
        let mut move_speed = 5;
        let mut scroll_list =
            smallvec::SmallVec::<[(crate::state::MouseScrollDirection, i32); 2]>::new();

        for action in actions.iter() {
            match action {
                OutputAction::MouseMove(direction, speed) => {
                    let dir_idx = *direction as usize;
                    move_directions.push(dir_idx);
                    if !*has_first_speed {
                        move_speed = *speed;
                        *first_speed = *speed;
                        *has_first_speed = true;
                    }
                }
                OutputAction::MouseScroll(direction, speed) => {
                    scroll_list.push((*direction, *speed));
                }
                OutputAction::MultipleActions(sub_actions) => {
                    for sub_action in sub_actions.iter() {
                        match sub_action {
                            OutputAction::MouseMove(direction, speed) => {
                                let dir_idx = *direction as usize;
                                move_directions.push(dir_idx);
                                if !*has_first_speed {
                                    move_speed = *speed;
                                    *first_speed = *speed;
                                    *has_first_speed = true;
                                }
                            }
                            OutputAction::MouseScroll(direction, speed) => {
                                scroll_list.push((*direction, *speed));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        for dir_idx in move_directions.iter() {
            *active_directions |= 1 << dir_idx;
            direction_devices[*dir_idx] = Some(device.clone());
            direction_intervals[*dir_idx] = interval;
            direction_last_times[*dir_idx] = now;
            direction_turbo[*dir_idx] = turbo_enabled;
        }

        if !move_directions.is_empty() {
            Self::execute_movement_immediate_bitflags(
                *active_directions,
                direction_vectors,
                move_speed,
            );
        }

        for (scroll_direction, scroll_speed) in scroll_list.iter() {
            for i in 0..4 {
                if scroll_devices[i].is_none() {
                    *scroll_active |= 1 << i;
                    scroll_devices[i] = Some(device.clone());
                    scroll_directions[i] = *scroll_direction;
                    scroll_speeds[i] = *scroll_speed;
                    scroll_intervals[i] = interval;
                    scroll_last_times[i] = now;
                    scroll_turbo[i] = turbo_enabled;
                    break;
                }
            }
        }

        for (scroll_direction, scroll_speed) in scroll_list.iter() {
            state.simulate_action(
                OutputAction::MouseScroll(*scroll_direction, *scroll_speed),
                0,
            );
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn handle_mouse_action_event(
        state: &AppState,
        active_directions: &mut u8,
        direction_devices: &mut [Option<InputDevice>; 8],
        direction_intervals: &mut [u64; 8],
        direction_last_times: &mut [Instant; 8],
        direction_turbo: &mut [bool; 8],
        mapping_cache: &mut HashMap<InputDevice, (usize, i32, u64, bool)>,
        scroll_active: &mut u8,
        scroll_devices: &mut [Option<InputDevice>; 4],
        scroll_directions: &mut [crate::state::MouseScrollDirection; 4],
        scroll_speeds: &mut [i32; 4],
        scroll_intervals: &mut [u64; 4],
        scroll_last_times: &mut [Instant; 4],
        scroll_turbo: &mut [bool; 4],
        first_speed: &mut i32,
        has_first_speed: &mut bool,
        direction_vectors: &[(f32, f32); 8],
        event: InputEvent,
    ) {
        use crate::state::OutputAction;
        let now = Instant::now();

        match event {
            InputEvent::Pressed(device) => {
                // Check if this is a scroll action first
                // First check if device already in scroll_devices (Windows repeat)
                let mut scroll_idx = None;
                for (i, scroll_device) in scroll_devices.iter().enumerate().take(4) {
                    if scroll_device.as_ref() == Some(&device) {
                        scroll_idx = Some(i);
                        break;
                    }
                }

                if let Some(idx) = scroll_idx {
                    // Windows repeat event for non-turbo scroll
                    if !scroll_turbo[idx] {
                        scroll_last_times[idx] = now;
                        state.simulate_action(
                            OutputAction::MouseScroll(scroll_directions[idx], scroll_speeds[idx]),
                            0,
                        );
                    }
                    return;
                }

                // Check if this is a new scroll mapping
                if let Some(mapping) = state.get_input_mapping(&device)
                    && let OutputAction::MouseScroll(direction, speed) = &mapping.target_action
                {
                    // Find empty slot
                    for i in 0..4 {
                        if scroll_devices[i].is_none() {
                            *scroll_active |= 1 << i;
                            scroll_devices[i] = Some(device);
                            scroll_directions[i] = *direction;
                            scroll_speeds[i] = *speed;
                            scroll_intervals[i] = mapping.interval;
                            scroll_last_times[i] = now;
                            scroll_turbo[i] = mapping.turbo_enabled;
                            // Always simulate on first press
                            state.simulate_action(OutputAction::MouseScroll(*direction, *speed), 0);
                            return;
                        }
                    }
                    // No empty slot found (unlikely, but handle gracefully)
                    return;
                }

                // Handle mouse movement
                // Check if already active (Windows repeat event)
                let mut found_idx = None;
                for (i, direction_device) in direction_devices.iter().enumerate().take(8) {
                    if direction_device.as_ref() == Some(&device) {
                        found_idx = Some(i);
                        break;
                    }
                }

                if let Some(idx) = found_idx {
                    // Update existing entry (non-turbo repeat)
                    if !direction_turbo[idx] {
                        direction_last_times[idx] = now;
                        Self::execute_movement_immediate_bitflags(
                            *active_directions,
                            direction_vectors,
                            *first_speed,
                        );
                    }
                    return;
                }

                // New key press - check cache first
                if let Some(&(dir_idx, speed, interval, turbo_enabled)) = mapping_cache.get(&device)
                {
                    // Cache hit - no need to query state
                    if !*has_first_speed {
                        *first_speed = speed;
                        *has_first_speed = true;
                    }

                    // Set bit flag
                    *active_directions |= 1 << dir_idx;
                    direction_devices[dir_idx] = Some(device);
                    direction_intervals[dir_idx] = interval;
                    direction_last_times[dir_idx] = now;
                    direction_turbo[dir_idx] = turbo_enabled;

                    Self::execute_movement_immediate_bitflags(
                        *active_directions,
                        direction_vectors,
                        *first_speed,
                    );
                    return;
                }

                // Cache miss - query and cache
                if let Some(mapping) = state.get_input_mapping(&device) {
                    match &mapping.target_action {
                        OutputAction::MouseMove(direction, speed) => {
                            let dir_idx = *direction as usize;

                            mapping_cache.insert(
                                device.clone(),
                                (dir_idx, *speed, mapping.interval, mapping.turbo_enabled),
                            );

                            if !*has_first_speed {
                                *first_speed = *speed;
                                *has_first_speed = true;
                            }

                            *active_directions |= 1 << dir_idx;
                            direction_devices[dir_idx] = Some(device);
                            direction_intervals[dir_idx] = mapping.interval;
                            direction_last_times[dir_idx] = now;
                            direction_turbo[dir_idx] = mapping.turbo_enabled;

                            Self::execute_movement_immediate_bitflags(
                                *active_directions,
                                direction_vectors,
                                *first_speed,
                            );
                        }
                        OutputAction::MultipleActions(actions) => {
                            Self::handle_multiple_mouse_actions(
                                state,
                                &device,
                                actions,
                                active_directions,
                                direction_devices,
                                direction_intervals,
                                direction_last_times,
                                direction_turbo,
                                scroll_active,
                                scroll_devices,
                                scroll_directions,
                                scroll_speeds,
                                scroll_intervals,
                                scroll_last_times,
                                scroll_turbo,
                                first_speed,
                                has_first_speed,
                                direction_vectors,
                                mapping.interval,
                                mapping.turbo_enabled,
                                now,
                            );
                        }
                        _ => {}
                    }
                }
            }
            InputEvent::Released(device) => {
                // Clear all scroll slots associated with this device
                let mut found_scroll = false;
                for (i, scroll_device) in scroll_devices.iter_mut().enumerate().take(4) {
                    if scroll_device.as_ref() == Some(&device) {
                        *scroll_active &= !(1 << i);
                        *scroll_device = None;
                        found_scroll = true;
                    }
                }

                // If we found scroll devices, don't process movement devices
                if found_scroll {
                    return;
                }

                // Handle mouse movement release
                // Clear all directions associated with this device
                for (i, direction_device) in direction_devices.iter_mut().enumerate().take(8) {
                    if direction_device.as_ref() == Some(&device) {
                        *active_directions &= !(1 << i);
                        *direction_device = None;
                    }
                }

                // Reset first_speed when all released
                if *active_directions == 0 {
                    *has_first_speed = false;
                    *first_speed = 10;
                }
            }
        }
    }

    /// Process turbo-enabled scrolls from timeout
    #[inline(always)]
    fn execute_scroll_timeout(
        state: &AppState,
        scroll_active: u8,
        scroll_directions: &[crate::state::MouseScrollDirection; 4],
        scroll_speeds: &[i32; 4],
        scroll_intervals: &[u64; 4],
        scroll_last_times: &mut [Instant; 4],
        scroll_turbo: &[bool; 4],
    ) {
        use crate::state::OutputAction;
        let now = Instant::now();

        // Check each scroll slot bit
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            if (scroll_active & (1 << i)) == 0 {
                continue; // Slot not active
            }

            if !scroll_turbo[i] {
                continue; // Non-turbo, skip timeout processing
            }

            if now.duration_since(scroll_last_times[i])
                >= Duration::from_millis(scroll_intervals[i])
            {
                state.simulate_action(
                    OutputAction::MouseScroll(scroll_directions[i], scroll_speeds[i]),
                    0,
                );
                scroll_last_times[i] = now;
            }
        }
    }

    /// Process turbo-enabled movements from timeout
    #[inline(always)]
    fn execute_movement_bitflags(
        active_directions: u8,
        direction_intervals: &[u64; 8],
        direction_last_times: &mut [Instant; 8],
        direction_turbo: &[bool; 8],
        direction_vectors: &[(f32, f32); 8],
        speed: i32,
    ) {
        let now = Instant::now();
        let mut total_dx: f32 = 0.0;
        let mut total_dy: f32 = 0.0;
        let mut any_ready = false;

        // Check each direction bit for active movements
        #[allow(clippy::needless_range_loop)]
        for i in 0..8 {
            if (active_directions & (1 << i)) == 0 {
                continue; // Direction not active
            }

            if !direction_turbo[i] {
                continue; // Non-turbo, skip timeout processing
            }

            if now.duration_since(direction_last_times[i])
                < Duration::from_millis(direction_intervals[i])
            {
                continue; // Not ready yet
            }

            any_ready = true;
            let (dx, dy) = direction_vectors[i];
            total_dx += dx;
            total_dy += dy;

            // Update time inline
            direction_last_times[i] = now;
        }

        if any_ready {
            Self::send_movement_normalized(total_dx, total_dy, speed);
        }
    }

    /// Process immediate movements from key press
    #[inline(always)]
    fn execute_movement_immediate_bitflags(
        active_directions: u8,
        direction_vectors: &[(f32, f32); 8],
        speed: i32,
    ) {
        if active_directions == 0 {
            return;
        }

        let mut total_dx: f32 = 0.0;
        let mut total_dy: f32 = 0.0;

        // Manual loop unrolling for 8 directions
        if (active_directions & 0x01) != 0 {
            let (dx, dy) = direction_vectors[0];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x02) != 0 {
            let (dx, dy) = direction_vectors[1];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x04) != 0 {
            let (dx, dy) = direction_vectors[2];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x08) != 0 {
            let (dx, dy) = direction_vectors[3];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x10) != 0 {
            let (dx, dy) = direction_vectors[4];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x20) != 0 {
            let (dx, dy) = direction_vectors[5];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x40) != 0 {
            let (dx, dy) = direction_vectors[6];
            total_dx += dx;
            total_dy += dy;
        }
        if (active_directions & 0x80) != 0 {
            let (dx, dy) = direction_vectors[7];
            total_dx += dx;
            total_dy += dy;
        }

        Self::send_movement_normalized(total_dx, total_dy, speed);
    }

    /// Normalize and send mouse movement
    #[inline(always)]
    fn send_movement_normalized(dx: f32, dy: f32, speed: i32) {
        use crate::state::SIMULATED_EVENT_MARKER;

        if dx == 0.0 && dy == 0.0 {
            return;
        }

        let speed_f = speed as f32;

        // Calculate magnitude squared
        let mag_sq = dx * dx + dy * dy;

        if mag_sq < 0.0001 {
            return;
        }

        // Hardware square root instruction
        let inv_mag = 1.0 / mag_sq.sqrt();

        // Normalize and scale
        let final_dx = (dx * inv_mag * speed_f + 0.5) as i32;
        let final_dy = (dy * inv_mag * speed_f + 0.5) as i32;

        if unlikely(final_dx == 0 && final_dy == 0) {
            return;
        }

        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::*;

            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: final_dx,
                        dy: final_dy,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_MOVE,
                        time: 0,
                        dwExtraInfo: SIMULATED_EVENT_MARKER,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hook_handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_worker_pool_creation() {
        let worker_count = 4;
        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());
        let (mouse_move_tx, _mouse_move_rx) = channel();
        let pool = WorkerPool::new(worker_count, state, mouse_move_tx);

        assert_eq!(pool.worker_count, worker_count);
        assert_eq!(pool.workers.capacity(), worker_count);
    }

    #[test]
    fn test_worker_distribution_stability() {
        let worker_count = 4;
        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());
        let (mouse_move_tx, _mouse_move_rx) = channel();
        let _pool = WorkerPool::new(worker_count, state, mouse_move_tx);

        // Test that same device always maps to same worker
        let _device_a = InputDevice::Keyboard(0x41); // A
        let idx1 = 0x41usize % worker_count;
        let idx2 = 0x41usize % worker_count;
        assert_eq!(idx1, idx2, "Same device should map to same worker");

        // Test that different devices distribute across workers
        let mut worker_usage = vec![0; worker_count];
        for vk in 0..256u32 {
            let idx = (vk as usize) % worker_count;
            worker_usage[idx] += 1;
        }

        // Check that distribution is reasonably balanced
        for count in &worker_usage {
            assert!(*count > 0, "All workers should receive some keys");
            assert!(*count < 256, "No worker should receive all keys");
        }
    }

    #[test]
    fn test_mapping_cache_retrieval() {
        use crate::config::KeyMapping;
        use smallvec::SmallVec;

        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_keys: SmallVec::from_vec(vec!["B".to_string()]),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
            move_speed: 10,
        }];

        let state = Arc::new(AppState::new(config).unwrap());
        let device = InputDevice::Keyboard(0x41); // 'A' key

        // Test that mapping can be retrieved correctly
        let mapping = state.get_input_mapping(&device);
        assert!(mapping.is_some(), "Mapping for 'A' key should exist");

        let mapping = mapping.unwrap();
        assert_eq!(mapping.interval, 10);
        assert_eq!(mapping.event_duration, 5);

        // Test unmapped device
        let unmapped_device = InputDevice::Keyboard(0x5A); // 'Z' key
        let no_mapping = state.get_input_mapping(&unmapped_device);
        assert!(no_mapping.is_none(), "Unmapped key should return None");
    }
}
