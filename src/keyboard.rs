use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::state::{AppState, KeyEvent};

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
        let (event_tx, event_rx) = channel();
        self.state.set_event_sender(event_tx);

        let main_thread_id = unsafe { GetCurrentThreadId() };

        // 强制创建消息队列
        unsafe {
            let mut msg = MSG::default();
            let result = PeekMessageA(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE);
            println!("PeekMessage result: {:?}", result);
        }

        // 启动连发处理线程
        let state_clone = self.state.clone();
        thread::spawn(move || Self::turbo_worker(main_thread_id, state_clone, event_rx));

        // 主线程消息循环
        unsafe {
            let mut msg = MSG::default();
            loop {
                let result = GetMessageA(&mut msg, None, 0, 0);

                if result.0 == 0 {
                    // 收到 WM_QUIT，退出循环
                    println!("Main thread received WM_QUIT");
                    break;
                } else if result.0 == -1 {
                    // 错误处理
                    eprintln!("GetMessage error");
                    break;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            // 清理钩子
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

        // 跳过模拟的按键事件
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

    fn turbo_worker(main_thread_id: u32, state: Arc<AppState>, event_rx: Receiver<KeyEvent>) {
        let mut key_states = HashMap::new();

        while !state.should_exit() {
            if state.is_paused() {
                if !key_states.is_empty() {
                    key_states.clear();
                }
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            match event_rx.recv_timeout(Duration::from_millis(state.input_timeout())) {
                Ok(event) => Self::handle_key_event(&state, &mut key_states, event),
                Err(_) => Self::handle_timeout(&state, &mut key_states),
            }
        }
        // 向主线程发送退出信号
        unsafe {
            let result = PostThreadMessageA(main_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            if result.is_ok() {
                println!("WM_QUIT message sent successfully");
            } else {
                eprintln!("Failed to send WM_QUIT message: {:?}", result);
                // Alternative solution: Use ctrl+c
                windows::Win32::System::Console::GenerateConsoleCtrlEvent(
                    windows::Win32::System::Console::CTRL_C_EVENT,
                    0,
                )
                .unwrap();
            }
        }

        println!("Turbo worker thread sent quit message");
    }

    fn handle_key_event(state: &AppState, key_states: &mut HashMap<u32, Instant>, event: KeyEvent) {
        match event {
            KeyEvent::Pressed(trigger_key) => {
                let now = Instant::now();

                if let Some(last_time) = key_states.get_mut(&trigger_key) {
                    if let Some(mapping) = state.get_key_mapping(&trigger_key)
                        && now.duration_since(*last_time) >= Duration::from_millis(mapping.interval)
                    {
                        state.simulate_key_press(mapping.target_scancode, mapping.event_duration);
                        *last_time = now;
                    }
                } else {
                    key_states.insert(trigger_key, now);
                    if let Some(mapping) = state.get_key_mapping(&trigger_key) {
                        state.simulate_key_press(mapping.target_scancode, mapping.event_duration);
                    }
                }
            }
            KeyEvent::Released(trigger_key) => {
                key_states.remove(&trigger_key);
            }
        }
    }

    fn handle_timeout(state: &AppState, key_states: &mut HashMap<u32, Instant>) {
        let now = Instant::now();

        for (trigger_key, last_time) in key_states.iter_mut() {
            if let Some(mapping) = state.get_key_mapping(trigger_key)
                && now.duration_since(*last_time) >= Duration::from_millis(mapping.interval)
            {
                state.simulate_key_press(mapping.target_scancode, mapping.event_duration);
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
