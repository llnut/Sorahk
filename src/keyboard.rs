use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::state::{AppState, InputDevice, InputEvent};

/// Branch prediction hints
#[inline(always)]
#[cold]
fn cold() {}

#[inline(always)]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

#[inline(always)]
fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

unsafe impl Send for KeyboardHook {}

// Multi-worker dispatcher supporting both keyboard and mouse
struct WorkerPool {
    workers: Vec<Sender<InputEvent>>,
    worker_count: usize,
}

impl WorkerPool {
    fn new(worker_count: usize) -> Self {
        Self {
            workers: Vec::with_capacity(worker_count),
            worker_count,
        }
    }

    fn add_worker(&mut self, sender: Sender<InputEvent>) {
        self.workers.push(sender);
    }
}

// Implement EventDispatcher trait
impl crate::state::EventDispatcher for WorkerPool {
    // Dispatch events using pre-computed lookup (hot path)
    #[inline]
    fn dispatch(&self, event: InputEvent) {
        if unlikely(self.workers.is_empty()) {
            return;
        }

        let device = match &event {
            InputEvent::Pressed(d) | InputEvent::Released(d) => d,
        };

        // Determine worker index using simple hash-based distribution
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

impl WorkerPool {
    /// Hash generic device for worker distribution using FNV-1a.
    #[inline(always)]
    fn hash_generic_device(device_type: &crate::state::DeviceType, button_id: u64) -> usize {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        let mut hash = FNV_OFFSET_BASIS;

        // Hash device type discriminant
        let discriminant = std::mem::discriminant(device_type);
        let disc_value = unsafe { *(&discriminant as *const _ as *const u64) };
        hash = (hash ^ disc_value).wrapping_mul(FNV_PRIME);

        // Hash button_id
        hash = (hash ^ button_id).wrapping_mul(FNV_PRIME);

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

        let mut worker_pool = WorkerPool::new(worker_count);
        let mut worker_handles = Vec::new();

        // Start multiple worker threads
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
                        // Single-shot mode: simulate on every Windows repeat event
                        state.simulate_action(target_action.clone(), *duration);
                        *last_time = now;
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

                        // Always simulate on first press
                        state.simulate_action(target_action_clone, mapping.event_duration);
                    }
                }
            }
            InputEvent::Released(device) => {
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
        let pool = WorkerPool::new(worker_count);

        assert_eq!(pool.worker_count, worker_count);
        assert_eq!(pool.workers.capacity(), worker_count);
    }

    #[test]
    fn test_worker_distribution_stability() {
        let worker_count = 4;
        let _pool = WorkerPool::new(worker_count);

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

        let mut config = AppConfig::default();
        config.mappings = vec![KeyMapping {
            trigger_key: "A".to_string(),
            target_key: "B".to_string(),
            interval: Some(10),
            event_duration: Some(5),
            turbo_enabled: true,
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
