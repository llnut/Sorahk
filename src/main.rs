use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, LazyLock, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, SendInput,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::*;

const DW_EXTRA_INFO: usize = 0x4659;
static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

/// 扩展扫描码映射表，确保包含所有常用键
static SCANCODE_MAP: LazyLock<HashMap<u32, u16>> = LazyLock::new(|| {
    [
        // 字母键 (A-Z)
        (0x41, 0x1E),
        (0x42, 0x30),
        (0x43, 0x2E),
        (0x44, 0x20),
        (0x45, 0x12),
        (0x46, 0x21),
        (0x47, 0x22),
        (0x48, 0x23),
        (0x49, 0x17),
        (0x4A, 0x24),
        (0x4B, 0x25),
        (0x4C, 0x26),
        (0x4D, 0x32),
        (0x4E, 0x31),
        (0x4F, 0x18),
        (0x50, 0x19),
        (0x51, 0x10),
        (0x52, 0x13),
        (0x53, 0x1F),
        (0x54, 0x14),
        (0x55, 0x16),
        (0x56, 0x2F),
        (0x57, 0x11),
        (0x58, 0x2D),
        (0x59, 0x15),
        (0x5A, 0x2C),
        // 数字键 (0-9)
        (0x30, 0x0B),
        (0x31, 0x02),
        (0x32, 0x03),
        (0x33, 0x04),
        (0x34, 0x05),
        (0x35, 0x06),
        (0x36, 0x07),
        (0x37, 0x08),
        (0x38, 0x09),
        (0x39, 0x0A),
        // 功能键 (F1-F12)
        (0x70, 0x3B),
        (0x71, 0x3C),
        (0x72, 0x3D),
        (0x73, 0x3E),
        (0x74, 0x3F),
        (0x75, 0x40),
        (0x76, 0x41),
        (0x77, 0x42),
        (0x78, 0x43),
        (0x79, 0x44),
        (0x7A, 0x57),
        (0x7B, 0x58),
        // 特殊键
        (0x1B, 0x01), // ESC
        (0x0D, 0x1C), // ENTER
        (0x09, 0x0F), // TAB
        (0x20, 0x39), // SPACE
        (0x08, 0x0E), // BACKSPACE
        (0x2E, 0x53), // DELETE
        (0x2D, 0x52), // INSERT
        (0x24, 0x47), // HOME
        (0x23, 0x4F), // END
        (0x21, 0x49), // PAGEUP
        (0x22, 0x51), // PAGEDOWN
        (0x26, 0x48), // UP
        (0x28, 0x50), // DOWN
        (0x25, 0x4B), // LEFT
        (0x27, 0x4D), // RIGHT
        // 小键盘
        (0x60, 0x52),
        (0x61, 0x4F),
        (0x62, 0x50),
        (0x63, 0x51),
        (0x64, 0x4B),
        (0x65, 0x4C),
        (0x66, 0x4D),
        (0x67, 0x47),
        (0x68, 0x48),
        (0x69, 0x49),
        (0x6A, 0x37),
        (0x6B, 0x4E),
        (0x6C, 0x53),
        (0x6D, 0x4A),
        (0x6E, 0x52),
        (0x6F, 0x53),
    ]
    .iter()
    .cloned()
    .collect()
});

static SPECIAL_VIRTUAL_KEYS: LazyLock<Vec<(&str, u32)>> = LazyLock::new(|| {
    vec![
        ("ESC", 0x1B),
        ("ENTER", 0x0D),
        ("TAB", 0x09),
        ("CLEAR", 0x0C),
        ("SHIFT", 0x10),
        ("CTRL", 0x11),
        ("ALT", 0x12),
        ("PAUSE", 0x13),
        ("CAPSLOCK", 0x14),
        ("SPACE", 0x20),
        ("BACKSPACE", 0x08),
        ("DELETE", 0x2E),
        ("INSERT", 0x2D),
        ("HOME", 0x24),
        ("END", 0x23),
        ("PAGEUP", 0x21),
        ("PAGEDOWN", 0x22),
        ("UP", 0x26),
        ("DOWN", 0x28),
        ("LEFT", 0x25),
        ("RIGHT", 0x27),
        ("LSHIFT", 0xA0),
        ("RSHIFT", 0xA1),
        ("LCTRL", 0xA2),
        ("RCTRL", 0xA3),
        ("LALT", 0xA4),
        ("RALT", 0xA5),
        ("LWIN", 0x5B),
        ("RWIN", 0x5C),
    ]
});

/// 配置结构
#[derive(Debug, Deserialize)]
struct Config {
    /// 连发开关
    switch_key: String,
    /// 连发设置
    mappings: Vec<KeyMapping>,
    /// 输入超时时间
    input_timeout: u64,
    /// 默认连发间隔
    interval: u64,
    /// 按键事件持续事件
    event_duration: u64,
}

#[derive(Debug, Deserialize)]
struct KeyMapping {
    /// 触发键（如 "S"）
    trigger_key: String,

    /// 目标键（如 "D"）
    target_key: String,

    /// 可选的特定间隔
    #[serde(default)]
    interval: Option<u64>,

    /// 可选的按键事件持续事件
    #[serde(default)]
    event_duration: Option<u64>,
}

/// 全局状态
#[derive(Debug)]
struct AppState {
    switch_key: u32,
    should_pause: AtomicBool,
    /// 输入超时时间
    input_timeout: u64,
    /// 按键映射表：触发键 -> (目标键, 间隔, 按下持续时间)
    key_mappings: HashMap<u32, (u16, u64, u64)>,
    /// 使用通道进行线程间通信
    key_event_tx: Sender<KeyEvent>,
}

/// 按键事件类型
#[derive(Debug, Clone, Copy)]
enum KeyEvent {
    /// 按下虚拟键
    Pressed(u32),
    /// 放开虚拟键
    Released(u32),
}

impl AppState {
    fn new(
        switch_key: u32,
        input_timeout: u64,
        key_event_tx: Sender<KeyEvent>,
        key_mappings: HashMap<u32, (u16, u64, u64)>,
    ) -> Self {
        Self {
            switch_key,
            should_pause: AtomicBool::new(false),
            input_timeout,
            key_mappings,
            key_event_tx,
        }
    }
}

fn main() -> windows::core::Result<()> {
    // 检查并提升管理员权限
    if !is_elevated() {
        if let Err(e) = elevate_privileges() {
            eprintln!("Failed to elevate privileges: {}", e);
            return Ok(());
        }
        // 如果成功请求提升权限，当前进程应该退出
        return Ok(());
    }

    println!("Running with administrator privileges");
    let config = load_config().expect("failed to get config");

    let switch_key = key_name_to_vk(&config.switch_key)
        .ok_or(std::io::Error::other("Failed to set switch key"))?;

    let input_timeout = config.input_timeout.max(2);

    let key_mappings = create_key_mappings(&config);
    println!("Loaded {} key mappings", key_mappings.len());

    let (key_event_tx, key_event_rx) = channel();

    let app_state = Arc::new(AppState::new(
        switch_key,
        input_timeout,
        key_event_tx,
        key_mappings,
    ));
    APP_STATE.set(app_state.clone()).unwrap();

    let state_clone = Arc::clone(&app_state);
    thread::spawn(move || {
        // 存储每个触发键的按下状态和最后连发时间
        let mut key_states: HashMap<u32, Instant> = HashMap::new();

        loop {
            if state_clone.should_pause.load(Ordering::Relaxed) {
                if !key_states.is_empty() {
                    key_states.clear();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            match key_event_rx.recv_timeout(Duration::from_millis(state_clone.input_timeout)) {
                Ok(event) => {
                    match event {
                        KeyEvent::Pressed(trigger_key) => {
                            let now = Instant::now();
                            if let Some(last_time) = key_states.get_mut(&trigger_key) {
                                if let Some((target_scancode, interval, event_duration)) =
                                    state_clone.key_mappings.get(&trigger_key)
                                    && now.duration_since(*last_time)
                                        >= Duration::from_millis(*interval)
                                {
                                    simulate_key_press(*target_scancode, *event_duration);
                                    *last_time = now;
                                }
                            } else {
                                key_states.insert(trigger_key, now);
                                // 立即发送一次目标键
                                if let Some((target_scancode, _, event_duration)) =
                                    state_clone.key_mappings.get(&trigger_key)
                                {
                                    simulate_key_press(*target_scancode, *event_duration);
                                }
                            }
                        }
                        KeyEvent::Released(trigger_key) => {
                            key_states.remove(&trigger_key);
                        }
                    }
                }
                Err(_) => {
                    // 超时，检查所有需要连发的触发键
                    let now = Instant::now();

                    for (trigger_key, last_time) in key_states.iter_mut() {
                        if let Some((target_scancode, interval, event_duration)) =
                            state_clone.key_mappings.get(trigger_key)
                            && now.duration_since(*last_time) >= Duration::from_millis(*interval)
                        {
                            simulate_key_press(*target_scancode, *event_duration);
                            *last_time = now;
                        }
                    }
                }
            }
        }
    });

    // 设置键盘钩子
    unsafe {
        let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)?;

        if hook.0.is_null() {
            eprintln!("Failed to set keyboard hook");
            return Ok(());
        }

        // 消息循环
        let mut msg = MSG::default();
        while GetMessageA(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        // 清理钩子
        let _ = UnhookWindowsHookEx(hook);
    }

    Ok(())
}

/// 键盘钩子回调
/// 模拟按键直接执行，不新增事件
unsafe extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = unsafe { &*(l_param.0 as *mut KBDLLHOOKSTRUCT) };
        // 若是模拟的按键，直接执行
        if kb_struct.dwExtraInfo == DW_EXTRA_INFO {
            return unsafe { CallNextHookEx(None, code, w_param, l_param) };
        }

        // 手动按下的按键，检查是否是触发键
        if let Some(app_state) = APP_STATE.get() {
            let should_pause = app_state.should_pause.load(Ordering::Relaxed);
            if !should_pause && app_state.key_mappings.contains_key(&kb_struct.vkCode) {
                match w_param.0 as u32 {
                    #[allow(non_snake_case)]
                    WM_KEYDOWN | WM_SYSKEYDOWN => {
                        let _ = app_state
                            .key_event_tx
                            .send(KeyEvent::Pressed(kb_struct.vkCode));
                        return LRESULT(1); //直接忽略所有手动的按下效果，只有模拟按下效果会生效
                    }
                    #[allow(non_snake_case)]
                    WM_KEYUP | WM_SYSKEYUP => {
                        let _ = app_state
                            .key_event_tx
                            .send(KeyEvent::Released(kb_struct.vkCode));
                        return LRESULT(1); //直接忽略所有手动的弹起效果，只有模拟弹起效果会生效
                    }
                    _ => {}
                }
            }

            // 开关连发
            if kb_struct.vkCode == app_state.switch_key {
                match w_param.0 as u32 {
                    #[allow(non_snake_case)]
                    WM_KEYUP | WM_SYSKEYUP => {
                        if should_pause {
                            println!("AHK activiting");
                            app_state.should_pause.store(false, Ordering::Relaxed);
                        } else {
                            println!("AHK paused");
                            app_state.should_pause.store(true, Ordering::Relaxed);
                        }
                    }
                    _ => {}
                }
                return LRESULT(1);
            }
        }
    }

    unsafe { CallNextHookEx(None, code, w_param, l_param) }
}

// 使用扫描码模拟按键按下和释放
fn simulate_key_press(scancode: u16, event_duration: u64) {
    unsafe {
        // 按下键
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0), // 不使用虚拟键码
                    wScan: scancode,
                    dwFlags: KEYEVENTF_SCANCODE,
                    time: 0,
                    dwExtraInfo: DW_EXTRA_INFO, // 标记为模拟事件
                },
            },
        };

        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        std::thread::sleep(std::time::Duration::from_millis(event_duration));

        // 释放键
        input.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

// 加载配置文件
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(toml::from_str(&fs::read_to_string("Config.toml")?)?)
}

// 创建按键映射表
fn create_key_mappings(config: &Config) -> HashMap<u32, (u16, u64, u64)> {
    let mut mappings = HashMap::new();

    for mapping in &config.mappings {
        if let (Some(trigger_vk), Some(target_vk)) = (
            key_name_to_vk(&mapping.trigger_key),
            key_name_to_vk(&mapping.target_key),
        ) {
            let target_scancode = vk_to_scancode(target_vk);
            let interval = mapping.interval.unwrap_or(config.interval).max(5);
            let event_duration = mapping
                .event_duration
                .unwrap_or(config.event_duration)
                .max(5);

            if target_scancode != 0 {
                mappings.insert(trigger_vk, (target_scancode, interval, event_duration));
                println!(
                    "Mapping: {} (VK:{:X}) -> {} (SC:{:X}) (interval:{}ms, duration:{}ms)",
                    mapping.trigger_key,
                    trigger_vk,
                    mapping.target_key,
                    target_scancode,
                    interval,
                    event_duration
                );
            } else {
                eprintln!(
                    "Invalid target key mapping: {} -> {} (unknown scancode)",
                    mapping.trigger_key, mapping.target_key
                );
            }
        } else {
            eprintln!(
                "Invalid key mapping: {} -> {}",
                mapping.trigger_key, mapping.target_key
            );
        }
    }

    mappings
}

// 将键名转换为虚拟键码
fn key_name_to_vk(key_name: &str) -> Option<u32> {
    let key = key_name.to_uppercase();

    // 字母键
    if key.len() == 1
        && let Some(c) = key.chars().next()
        && c.is_ascii_alphabetic()
    {
        return Some(c as u32);
    }

    // 数字键
    if key.len() == 1
        && let Some(c) = key.chars().next()
        && c.is_ascii_digit()
    {
        return Some(c as u32);
    }

    // 功能键
    if key.starts_with('F')
        && key.len() > 1
        && let Ok(num) = key[1..].parse::<u32>()
        && (1..=24).contains(&num)
    {
        return Some(0x70 + num - 1); // VK_F1 = 0x70
    }

    // 特殊键映射
    for (name, vk) in &*SPECIAL_VIRTUAL_KEYS {
        if key == *name {
            return Some(*vk);
        }
    }

    eprintln!("Unknown key name: {}", key_name);
    None
}

// 确保使用正确的扫描码映射
fn vk_to_scancode(vk_code: u32) -> u16 {
    *SCANCODE_MAP.get(&vk_code).unwrap_or(&0)
}

// 检查当前进程是否以管理员权限运行
fn is_elevated() -> bool {
    unsafe {
        let mut token = std::mem::zeroed();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;

        if GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_err()
        {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

// 提升权限 - 使用ShellExecute以管理员权限重新启动程序
fn elevate_privileges() -> std::io::Result<()> {
    let exe_path = env::current_exe()?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| std::io::Error::other("Invalid executable path"))?;

    // 使用PowerShell的Start-Process命令请求提升权限
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Hidden",
                exe_str
            ),
        ])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        Err(std::io::Error::other(format!(
            "Failed to elevate privileges: {}",
            err_msg
        )))
    }
}
