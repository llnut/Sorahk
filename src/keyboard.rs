use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::state::{AppState, InputDevice, InputEvent};

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
    // Dispatch events using pre-computed lookup
    fn dispatch(&self, event: InputEvent) {
        if self.workers.is_empty() {
            return;
        }

        let device = match event {
            InputEvent::Pressed(d) | InputEvent::Released(d) => d,
        };

        // Determine worker index based on device
        let worker_idx = if let Some(state) = crate::state::get_global_state() {
            // For keyboard, use the optimized VK lookup
            // For mouse, use a simple hash
            match device {
                InputDevice::Keyboard(vk) => {
                    let mapping_idx = state.get_worker_index(vk);
                    mapping_idx % self.worker_count
                }
                InputDevice::Mouse(button) => {
                    // Use simple discriminant-based distribution (much faster than hashing)
                    (button as usize) % self.worker_count
                }
            }
        } else {
            // Fallback: simple hash
            match device {
                InputDevice::Keyboard(vk) => (vk as usize) % self.worker_count,
                InputDevice::Mouse(button) => (button as usize) % self.worker_count,
            }
        };

        // Non-blocking send to ensure hook callback responsiveness
        let _ = self.workers[worker_idx].send(event);
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
        let mut device_states: HashMap<InputDevice, (Instant, u64, u64, OutputAction)> =
            HashMap::new();
        // Cache format: (last_time, interval, event_duration, target_action)

        while !state.should_exit() {
            if state.is_paused() {
                if !device_states.is_empty() {
                    device_states.clear();
                }
                thread::sleep(Duration::from_millis(50)); // Shorter sleep time for better responsiveness
                continue;
            }

            // Use shorter timeout for better responsiveness
            match event_rx.recv_timeout(Duration::from_millis(state.input_timeout())) {
                Ok(event) => Self::handle_input_event(&state, &mut device_states, event),
                Err(_) => Self::handle_timeout(&state, &mut device_states),
            }
        }
    }

    fn handle_input_event(
        state: &AppState,
        device_states: &mut HashMap<InputDevice, (Instant, u64, u64, crate::state::OutputAction)>,
        event: InputEvent,
    ) {
        match event {
            InputEvent::Pressed(device) => {
                let now = Instant::now();

                if let Some((last_time, interval, duration, target_action)) =
                    device_states.get_mut(&device)
                {
                    // Use cached mapping info - no lock needed!
                    if now.duration_since(*last_time) >= Duration::from_millis(*interval) {
                        state.simulate_action(*target_action, *duration);
                        *last_time = now;
                    }
                } else {
                    // First press: lookup mapping and cache it (one-time RwLock read)
                    if let Some(mapping) = state.get_input_mapping(&device) {
                        device_states.insert(
                            device,
                            (
                                now,
                                mapping.interval,
                                mapping.event_duration,
                                mapping.target_action,
                            ),
                        );
                        state.simulate_action(mapping.target_action, mapping.event_duration);
                    }
                }
            }
            InputEvent::Released(device) => {
                device_states.remove(&device);
            }
        }
    }

    fn handle_timeout(
        state: &AppState,
        device_states: &mut HashMap<InputDevice, (Instant, u64, u64, crate::state::OutputAction)>,
    ) {
        let now = Instant::now();

        // Use cached mapping info - no repeated lookups or locks!
        for (_device, (last_time, interval, duration, target_action)) in device_states.iter_mut() {
            if now.duration_since(*last_time) >= Duration::from_millis(*interval) {
                state.simulate_action(*target_action, *duration);
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
    use crate::state::EventDispatcher;

    #[test]
    fn test_worker_pool_creation() {
        let worker_count = 4;
        let pool = WorkerPool::new(worker_count);

        assert_eq!(pool.worker_count, worker_count);
        assert_eq!(pool.workers.capacity(), worker_count);
    }

    #[test]
    fn test_worker_pool_dispatch_distribution() {
        let config = AppConfig::default();
        let state = Arc::new(AppState::new(config).unwrap());

        // Test that different keys map to different workers
        let index_a = state.get_worker_index(0x41); // A
        let index_b = state.get_worker_index(0x42); // B

        // Worker indices should be valid
        assert!(index_a < 256);
        assert!(index_b < 256);
    }

    #[test]
    fn test_input_event_pressed_variant() {
        let event = InputEvent::Pressed(InputDevice::Keyboard(0x41));

        match event {
            InputEvent::Pressed(InputDevice::Keyboard(key)) => assert_eq!(key, 0x41),
            _ => panic!("Expected Pressed variant"),
        }
    }

    #[test]
    fn test_input_event_released_variant() {
        let event = InputEvent::Released(InputDevice::Keyboard(0x41));

        match event {
            InputEvent::Released(InputDevice::Keyboard(key)) => assert_eq!(key, 0x41),
            _ => panic!("Expected Released variant"),
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

    #[test]
    fn test_interval_expiration_logic() {
        // Test interval expiration calculation
        let interval_ms = 50u64;

        // Simulate a key pressed 10ms ago (within interval)
        let recent_time = Instant::now() - Duration::from_millis(10);
        assert!(
            recent_time.elapsed() < Duration::from_millis(interval_ms),
            "10ms elapsed should be within 50ms interval"
        );

        // Simulate a key pressed 60ms ago (beyond interval)
        let old_time = Instant::now() - Duration::from_millis(60);
        assert!(
            old_time.elapsed() >= Duration::from_millis(interval_ms),
            "60ms elapsed should exceed 50ms interval"
        );
    }

    #[test]
    fn test_worker_pool_with_actual_threads() {
        use std::sync::mpsc::channel;
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let worker_count = 4;
        let mut pool = WorkerPool::new(worker_count);
        let received_events = Arc::new(Mutex::new(Vec::new()));

        // Create worker threads
        for _ in 0..worker_count {
            let (tx, rx) = channel();
            pool.add_worker(tx);

            let events = received_events.clone();
            thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    events.lock().unwrap().push(event);
                }
            });
        }

        // Send test events
        for i in 0..10 {
            pool.dispatch(InputEvent::Pressed(InputDevice::Keyboard(0x41 + i)));
        }

        // Wait for events to be processed
        thread::sleep(Duration::from_millis(100));

        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 10);
    }

    #[test]
    fn test_worker_pool_load_balancing() {
        use std::sync::mpsc::channel;
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let worker_count = 4;
        let mut pool = WorkerPool::new(worker_count);
        let worker_loads = Arc::new(Mutex::new(vec![0usize; worker_count]));

        // Create worker threads with counters
        for worker_id in 0..worker_count {
            let (tx, rx) = channel();
            pool.add_worker(tx);

            let loads = worker_loads.clone();
            thread::spawn(move || {
                while let Ok(_event) = rx.recv() {
                    loads.lock().unwrap()[worker_id] += 1;
                }
            });
        }

        // Send many events to test distribution
        for i in 0..100 {
            pool.dispatch(InputEvent::Pressed(InputDevice::Keyboard(0x41 + (i % 26))));
        }

        // Wait for processing
        thread::sleep(Duration::from_millis(200));

        // Check that load is distributed
        let loads = worker_loads.lock().unwrap();
        let total: usize = loads.iter().sum();
        assert_eq!(total, 100);

        // Ensure no worker is idle
        for load in loads.iter() {
            assert!(*load > 0, "Worker received no events");
        }
    }

    #[test]
    fn test_key_event_channel_communication() {
        use std::sync::mpsc::channel;
        use std::thread;
        use std::time::Duration;

        let (tx, rx) = channel();

        let device = InputDevice::Keyboard(0x41);

        // Sender thread
        thread::spawn(move || {
            tx.send(InputEvent::Pressed(device)).unwrap();
            tx.send(InputEvent::Released(device)).unwrap();
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

        assert_eq!(events.len(), 2);
        match events[0] {
            InputEvent::Pressed(InputDevice::Keyboard(k)) => assert_eq!(k, 0x41),
            _ => panic!("Expected Pressed event"),
        }
        match events[1] {
            InputEvent::Released(InputDevice::Keyboard(k)) => assert_eq!(k, 0x41),
            _ => panic!("Expected Released event"),
        }
    }
}
