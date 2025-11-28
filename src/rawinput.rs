//! Raw Input API integration for HID devices.
//!
//! Provides support for gamepads, joysticks, and other HID controllers through
//! the Windows Raw Input API. Implements single-shot triggering where each
//! input event generates one output action.
//!
//! ## Cache Architecture
//!
//! Uses a three-tier caching system for device information:
//!
//! 1. **Thread-local cache** (`LAST_DEVICE_CACHE`)
//!    - Stores the most recently accessed device per thread
//!    - Zero-cost access for repeated inputs from the same device
//!    - Invalidated on device removal events
//!
//! 2. **Global device cache** (`device_cache`)
//!    - Lock-free concurrent HashMap for all connected devices
//!    - Shared across all threads
//!    - Invalidated on device removal events
//!
//! 3. **Windows API** (`GetRawInputDeviceInfoW`)
//!    - Authoritative source for device information
//!    - Only queried on cache misses or after invalidation
//!
//! ## Cache Consistency
//!
//! Cache invalidation occurs automatically on `WM_INPUT_DEVICE_CHANGE` messages
//! when devices are disconnected, ensuring stale data is not reused if a device
//! reconnects with the same handle.

use std::cell::UnsafeCell;
use std::sync::{Arc, OnceLock};
use windows::Win32::Foundation::{GetLastError, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

use crate::state::{AppState, DeviceType, InputDevice, InputEvent};

/// Branch prediction helper for error paths.
#[inline(always)]
#[cold]
fn cold() {}

/// Indicates that the condition is unlikely to be true.
#[inline(always)]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

/// Indicates that the condition is likely to be true.
#[inline(always)]
fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

/// Number of buffers in the thread-local pool.
const BUFFER_POOL_SIZE: usize = 8;
/// Maximum buffer size before falling back to heap allocation.
const MAX_BUFFER_SIZE: usize = 256;

/// HID usage page for generic desktop controls.
const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
/// HID usage ID for gamepad devices.
const HID_USAGE_GAMEPAD: u16 = 0x05;
/// HID usage ID for joystick devices.
const HID_USAGE_JOYSTICK: u16 = 0x04;
/// HID usage ID for multi-axis controllers.
const HID_USAGE_MULTI_AXIS: u16 = 0x08;

/// Minimum valid HID report size in bytes.
const MIN_HID_DATA_SIZE: usize = 10;

thread_local! {
    /// Thread-local buffer pool for Raw Input data.
    static BUFFER_POOL: UnsafeCell<RingBufferPool> = const { UnsafeCell::new(RingBufferPool::new()) };
    /// Thread-local cache for most recently accessed device.
    static LAST_DEVICE_CACHE: UnsafeCell<Option<(isize, CachedDeviceInfo)>> = const { UnsafeCell::new(None) };
}

/// Ring buffer pool for reusing Raw Input data buffers.
struct RingBufferPool {
    buffers: [Vec<u8>; BUFFER_POOL_SIZE],
    current: usize,
}

impl RingBufferPool {
    const fn new() -> Self {
        const EMPTY_VEC: Vec<u8> = Vec::new();
        Self {
            buffers: [EMPTY_VEC; BUFFER_POOL_SIZE],
            current: 0,
        }
    }

    /// Returns the next available buffer, expanding if necessary.
    #[inline]
    fn get_buffer(&mut self, size: usize) -> &mut Vec<u8> {
        let idx = self.current;
        self.current = (self.current + 1) % BUFFER_POOL_SIZE;

        let buffer = &mut self.buffers[idx];
        buffer.clear();
        if buffer.capacity() < size {
            buffer.reserve(size.saturating_sub(buffer.capacity()));
        }
        buffer
    }
}

/// Global Raw Input handler instance.
static RAW_INPUT_HANDLER: OnceLock<RawInputHandler> = OnceLock::new();

/// Global cache for device display information.
static DEVICE_DISPLAY_INFO: OnceLock<scc::HashMap<u64, DeviceDisplayInfo>> = OnceLock::new();

/// Display information for HID devices.
#[derive(Debug, Clone)]
pub struct DeviceDisplayInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
}

/// Cached device information with optimized memory layout.
#[derive(Debug, Clone)]
#[repr(C)]
struct CachedDeviceInfo {
    device_type: DeviceType,
    vendor_id: u16,
    product_id: u16,
    usage_page: u16,
    usage: u16,
    serial_number: Option<String>,
}

/// Capture state for a single device during GUI button capture.
#[derive(Debug, Clone)]
struct DeviceCaptureState {
    captured: bool,
}

impl DeviceCaptureState {
    fn new() -> Self {
        Self { captured: false }
    }
}

/// Handler for Raw Input API messages from HID devices.
pub struct RawInputHandler {
    state: Arc<AppState>,
    /// Lock-free cache for device information.
    device_cache: scc::HashMap<isize, CachedDeviceInfo>,
    /// Capture state tracking during GUI button capture mode.
    capture_states: std::sync::Mutex<std::collections::HashMap<isize, DeviceCaptureState>>,
}

impl RawInputHandler {
    /// Invalidates all device information caches.
    ///
    /// Called automatically when a device is disconnected to ensure
    /// stale data is not used if the device reconnects with the same handle.
    fn invalidate_caches(&self) {
        // Clear global device information cache
        self.device_cache.clear_sync();

        // Clear thread-local last-device cache
        LAST_DEVICE_CACHE.with(|cache| unsafe {
            *cache.get() = None;
        });

        // Note: DEVICE_DISPLAY_INFO is not cleared here as it's used for
        // display formatting only and doesn't affect input processing.
        // It will be cleared on the next capture mode entry.
    }

    /// Provides manual cache invalidation for testing or error recovery.
    #[allow(dead_code)]
    pub fn force_invalidate_all_caches(&self) {
        self.invalidate_caches();
        clear_device_display_info_cache();
    }
}

/// Window class name for the Raw Input message-only window.
const RAWINPUT_WINDOW_CLASS: &str = "SorahkRawInputWindow";

/// Handle to the Raw Input processing thread.
pub struct RawInputThread {
    _handle: std::thread::JoinHandle<()>,
}

impl RawInputHandler {
    /// Creates a new Raw Input handler and registers HID devices.
    fn new(hwnd: HWND, state: Arc<AppState>) -> anyhow::Result<Self> {
        Self::register_devices(hwnd)?;
        Ok(Self {
            state,
            device_cache: scc::HashMap::new(),
            capture_states: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    /// Starts the Raw Input handler in a dedicated thread.
    pub fn start_thread(state: Arc<AppState>) -> RawInputThread {
        let handle = std::thread::Builder::new()
            .name("rawinput_thread".to_string())
            .spawn(move || {
                if let Err(e) = Self::run_message_loop(state) {
                    eprintln!("Raw Input thread error: {}", e);
                }
            })
            .expect("Failed to spawn Raw Input thread");

        RawInputThread { _handle: handle }
    }

    /// Runs the Windows message loop for Raw Input processing.
    fn run_message_loop(state: Arc<AppState>) -> anyhow::Result<()> {
        unsafe {
            let class_name = Self::to_wstring(RAWINPUT_WINDOW_CLASS);
            let h_instance = GetModuleHandleW(None)?;

            let wc = WNDCLASSW {
                lpfnWndProc: Some(Self::window_proc),
                hInstance: HINSTANCE(h_instance.0),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };

            if RegisterClassW(&wc) == 0 {
                let last_error = GetLastError();
                if last_error.0 != 1410 {
                    return Err(anyhow::anyhow!(
                        "Failed to register window class: {:?}",
                        last_error
                    ));
                }
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(class_name.as_ptr()),
                windows::core::w!("Sorahk Raw Input Window"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(HINSTANCE(h_instance.0)),
                None,
            )?;

            let handler = Self::new(hwnd, state)?;
            let _ = RAW_INPUT_HANDLER.set(handler);

            let mut msg = MSG::default();
            loop {
                let result = GetMessageW(&mut msg, None, 0, 0);

                if result.0 == 0 || result.0 == -1 {
                    break;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = DestroyWindow(hwnd);
            UnregisterClassW(PCWSTR(class_name.as_ptr()), Some(HINSTANCE(h_instance.0)))?;
        }

        Ok(())
    }

    /// Window procedure for Raw Input messages
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        const WM_INPUT_DEVICE_CHANGE: u32 = 0x00FE;
        const GIDC_ARRIVAL: usize = 1;
        const GIDC_REMOVAL: usize = 2;

        match msg {
            WM_INPUT => unsafe {
                if let Some(handler) = RAW_INPUT_HANDLER.get() {
                    handler.handle_raw_input(l_param);
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            },
            WM_INPUT_DEVICE_CHANGE => unsafe {
                // Device arrival or removal
                match w_param.0 {
                    GIDC_REMOVAL => {
                        // Device disconnected - invalidate caches
                        if let Some(handler) = RAW_INPUT_HANDLER.get() {
                            handler.invalidate_caches();
                        }
                    }
                    GIDC_ARRIVAL => {
                        // Device connected - caches will update on next input
                    }
                    _ => {}
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            },
            WM_CLOSE | WM_DESTROY => unsafe {
                PostQuitMessage(0);
                LRESULT(0)
            },
            _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
        }
    }

    /// Converts a string to null-terminated UTF-16 for Windows APIs.
    fn to_wstring(s: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Registers HID device types with the Raw Input API.
    fn register_devices(hwnd: HWND) -> anyhow::Result<()> {
        unsafe {
            let devices = [
                RAWINPUTDEVICE {
                    usUsagePage: 0x01,
                    usUsage: 0x05,
                    dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
                    hwndTarget: hwnd,
                },
                RAWINPUTDEVICE {
                    usUsagePage: 0x01,
                    usUsage: 0x04,
                    dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
                    hwndTarget: hwnd,
                },
                RAWINPUTDEVICE {
                    usUsagePage: 0x01,
                    usUsage: 0x08,
                    dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
                    hwndTarget: hwnd,
                },
            ];

            match RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Processes a WM_INPUT message from the Windows message loop.
    #[inline]
    pub fn handle_raw_input(&self, l_param: LPARAM) -> bool {
        unsafe {
            let mut size = 0u32;

            let result = GetRawInputData(
                HRAWINPUT(l_param.0 as _),
                RID_INPUT,
                None,
                &mut size,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            );

            if unlikely(result != 0) {
                return false;
            }

            // Size check for common buffer optimization
            if unlikely(size as usize > MAX_BUFFER_SIZE) {
                // Fallback for large buffers
                let mut buffer = vec![0u8; size as usize];
                let result = GetRawInputData(
                    HRAWINPUT(l_param.0 as _),
                    RID_INPUT,
                    Some(buffer.as_mut_ptr() as _),
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                if unlikely(result as u32 != size) {
                    return false;
                }

                let raw = &*(buffer.as_ptr() as *const RAWINPUT);
                return self.process_hid_input_fast(raw);
            }

            // Use thread-local buffer pool for typical sizes
            BUFFER_POOL.with(|pool| {
                let pool = &mut *pool.get();
                let buffer = pool.get_buffer(size as usize);
                buffer.resize(size as usize, 0);

                let result = GetRawInputData(
                    HRAWINPUT(l_param.0 as _),
                    RID_INPUT,
                    Some(buffer.as_mut_ptr() as _),
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                if unlikely(result as u32 != size) {
                    return false;
                }

                let raw = &*(buffer.as_ptr() as *const RAWINPUT);
                self.process_hid_input_fast(raw)
            })
        }
    }

    /// Processes HID input data with performance optimizations.
    #[inline(always)]
    fn process_hid_input_fast(&self, raw: &RAWINPUT) -> bool {
        unsafe {
            let hid = &raw.data.hid;

            // Early size validation using const
            let raw_data_size = hid.dwSizeHid as usize;
            let raw_data_count = hid.dwCount as usize;

            if unlikely(raw_data_size < MIN_HID_DATA_SIZE || raw_data_count == 0) {
                return false;
            }

            let device_handle = raw.header.hDevice;
            let handle_key = device_handle.0 as isize;

            // Ultra-fast path: check thread-local last device cache (most common case)
            let device_info = LAST_DEVICE_CACHE.with(|cache| {
                let cache_ptr = cache.get();
                if let Some((cached_handle, ref cached_info)) = *cache_ptr
                    && likely(cached_handle == handle_key)
                {
                    return Some(cached_info.clone());
                }

                // Fast path: check global device info cache
                if let Some(info) = self.device_cache.get_sync(&handle_key) {
                    let info_clone = info.get().clone();
                    // Update thread-local cache
                    *cache_ptr = Some((handle_key, info_clone.clone()));
                    return Some(info_clone);
                }

                // Slow path: fetch and cache device info
                if let Some(info) = self.get_device_info(device_handle) {
                    // Update both caches
                    *cache_ptr = Some((handle_key, info.clone()));
                    Some(info)
                } else {
                    None
                }
            });

            let device_info = match device_info {
                Some(info) => info,
                None => return false,
            };

            // Fast filter check using const (usage_page is always 0x01 for allowed devices)
            if unlikely(device_info.usage_page != HID_USAGE_PAGE_GENERIC) {
                return false;
            }

            // Inline usage check using const (most common: gamepad = 0x05)
            let usage = device_info.usage;
            if unlikely(
                usage != HID_USAGE_GAMEPAD
                    && usage != HID_USAGE_JOYSTICK
                    && usage != HID_USAGE_MULTI_AXIS,
            ) {
                return false;
            }

            let is_capturing = self.state.is_raw_input_capture_active();

            let data_ptr = hid.bRawData.as_ptr();
            let data_slice = std::slice::from_raw_parts(data_ptr, raw_data_size * raw_data_count);

            // === CAPTURE MODE ===
            if unlikely(is_capturing) {
                return self.handle_capture_mode(handle_key, device_info, data_slice);
            }

            // === NORMAL MODE - Trigger when matching HID data is received ===

            // Fast paused check
            if unlikely(self.state.is_paused()) {
                return false;
            }

            // Generate device identifier (inline for speed)
            let stable_device_id = if let Some(ref serial) = device_info.serial_number {
                Self::hash_vid_pid_serial(device_info.vendor_id, device_info.product_id, serial)
            } else {
                handle_key as u64
            };

            Self::update_device_display_info(stable_device_id, &device_info);

            let data_hash = Self::hash_hid_data_fast(data_slice);
            let button_id = (stable_device_id << 32) | (data_hash as u64);

            let device = InputDevice::GenericDevice {
                device_type: device_info.device_type,
                button_id,
            };

            // Fast mapping check and dispatch
            if likely(self.state.get_input_mapping(&device).is_some())
                && let Some(pool) = self.state.get_worker_pool()
            {
                pool.dispatch(InputEvent::Pressed(device));
                return true;
            }

            false
        }
    }

    /// Captures the first HID input received during GUI button capture mode.
    fn handle_capture_mode(
        &self,
        handle_key: isize,
        device_info: CachedDeviceInfo,
        current_data: &[u8],
    ) -> bool {
        let mut capture_states = self.capture_states.lock().unwrap();

        let capture_state = capture_states
            .entry(handle_key)
            .or_insert_with(DeviceCaptureState::new);

        // If already captured, ignore subsequent data
        if capture_state.captured {
            return false;
        }

        capture_state.captured = true;

        // Generate device identifier
        let stable_device_id = Self::generate_stable_device_id(handle_key, &device_info);
        Self::update_device_display_info(stable_device_id, &device_info);

        let data_hash = Self::hash_hid_data(current_data);
        let button_id = (stable_device_id << 32) | (data_hash as u64);

        let device = InputDevice::GenericDevice {
            device_type: device_info.device_type,
            button_id,
        };

        // Send to GUI
        let _ = self.state.get_raw_input_capture_sender().send(device);

        // Clear capture state for this device
        capture_states.remove(&handle_key);

        false
    }

    /// Extract serial number from device path
    fn extract_serial_from_path(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('#').collect();
        if parts.len() >= 3 {
            let serial = parts[2].trim();
            if !serial.is_empty() && serial.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(serial.to_string());
            }
        }
        None
    }

    /// Get device serial number from device handle
    fn get_device_serial_number(device_handle: HANDLE) -> Option<String> {
        unsafe {
            let mut size = 0u32;
            let result =
                GetRawInputDeviceInfoW(Some(device_handle), RIDI_DEVICENAME, None, &mut size);

            if result != 0 || size == 0 {
                return None;
            }

            let mut path_buf = vec![0u16; size as usize];
            let result = GetRawInputDeviceInfoW(
                Some(device_handle),
                RIDI_DEVICENAME,
                Some(path_buf.as_mut_ptr() as _),
                &mut size,
            );

            if result == u32::MAX {
                return None;
            }

            let path = String::from_utf16_lossy(&path_buf);
            Self::extract_serial_from_path(&path)
        }
    }

    /// Get device info for a given device handle
    #[inline]
    fn get_device_info(&self, device_handle: HANDLE) -> Option<CachedDeviceInfo> {
        let handle_key = device_handle.0 as isize;

        // Fast path: try async read first (lock-free)
        if let Some(device_info) = self.device_cache.read_sync(&handle_key, |_, v| v.clone()) {
            return Some(device_info);
        }

        unsafe {
            let mut size = 0u32;
            let result =
                GetRawInputDeviceInfoW(Some(device_handle), RIDI_DEVICEINFO, None, &mut size);

            if result != 0 {
                return None;
            }

            let mut buffer = vec![0u8; size as usize];
            let result = GetRawInputDeviceInfoW(
                Some(device_handle),
                RIDI_DEVICEINFO,
                Some(buffer.as_mut_ptr() as _),
                &mut size,
            );

            if result == u32::MAX {
                return None;
            }

            let device_info = &*(buffer.as_ptr() as *const RID_DEVICE_INFO);

            let cached_info = match device_info.dwType {
                t if t == RIM_TYPEHID => {
                    let hid_info = &device_info.Anonymous.hid;
                    let usage_page = hid_info.usUsagePage;
                    let usage = hid_info.usUsage;
                    let vendor_id = hid_info.dwVendorId as u16;
                    let product_id = hid_info.dwProductId as u16;

                    let device_type = match (usage_page, usage) {
                        (0x01, 0x05) => DeviceType::Gamepad(vendor_id),
                        (0x01, 0x04) => DeviceType::Joystick(vendor_id),
                        (0x01, 0x08) => DeviceType::Gamepad(vendor_id),
                        _ => DeviceType::HidDevice { usage_page, usage },
                    };

                    let serial_number = Self::get_device_serial_number(device_handle);

                    CachedDeviceInfo {
                        device_type,
                        vendor_id,
                        product_id,
                        serial_number,
                        usage_page,
                        usage,
                    }
                }
                _ => return None,
            };

            let _ = self
                .device_cache
                .upsert_sync(handle_key, cached_info.clone());

            Some(cached_info)
        }
    }

    /// Generate stable device ID for consistent identification
    #[inline]
    fn generate_stable_device_id(handle_key: isize, device_info: &CachedDeviceInfo) -> u64 {
        if let Some(ref serial) = device_info.serial_number {
            Self::hash_vid_pid_serial(device_info.vendor_id, device_info.product_id, serial)
        } else {
            handle_key as u64
        }
    }

    /// Computes a hash for device identification using VID, PID, and serial number.
    #[inline(always)]
    fn hash_vid_pid_serial(vendor_id: u16, product_id: u16, serial: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        vendor_id.hash(&mut hasher);
        product_id.hash(&mut hasher);
        serial.hash(&mut hasher);
        hasher.finish()
    }

    /// Computes FNV-1a hash of HID data with loop unrolling.
    #[inline(always)]
    fn hash_hid_data_fast(data: &[u8]) -> u32 {
        const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
        const FNV_PRIME: u32 = 0x01000193;

        let mut hash = FNV_OFFSET_BASIS;

        let chunks = data.chunks_exact(8);
        let remainder = chunks.remainder();

        for chunk in chunks {
            hash = (hash ^ chunk[0] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[1] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[2] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[3] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[4] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[5] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[6] as u32).wrapping_mul(FNV_PRIME);
            hash = (hash ^ chunk[7] as u32).wrapping_mul(FNV_PRIME);
        }

        for &byte in remainder {
            hash = (hash ^ byte as u32).wrapping_mul(FNV_PRIME);
        }

        hash
    }

    /// Fallback hash function for compatibility.
    #[inline]
    fn hash_hid_data(data: &[u8]) -> u32 {
        Self::hash_hid_data_fast(data)
    }

    /// Updates the global device display information cache.
    #[inline]
    fn update_device_display_info(stable_device_id: u64, device_info: &CachedDeviceInfo) {
        let display_info = DeviceDisplayInfo {
            vendor_id: device_info.vendor_id,
            product_id: device_info.product_id,
            serial_number: device_info.serial_number.clone(),
        };

        let cache = DEVICE_DISPLAY_INFO.get_or_init(scc::HashMap::new);
        let _ = cache.upsert_sync(stable_device_id, display_info);
    }
}

/// Retrieves cached display information for a device.
pub fn get_device_display_info(stable_device_id: u64) -> Option<DeviceDisplayInfo> {
    let cache = DEVICE_DISPLAY_INFO.get()?;
    cache
        .get_sync(&stable_device_id)
        .map(|entry| entry.get().clone())
}

/// Clears the device display information cache.
pub fn clear_device_display_info_cache() {
    if let Some(cache) = DEVICE_DISPLAY_INFO.get() {
        cache.clear_sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv_hash_consistency() {
        let data = b"test_hid_data_12345";
        let hash1 = RawInputHandler::hash_hid_data_fast(data);
        let hash2 = RawInputHandler::hash_hid_data_fast(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv_hash_different_data() {
        let data1 = b"test_data_1";
        let data2 = b"test_data_2";
        let hash1 = RawInputHandler::hash_hid_data_fast(data1);
        let hash2 = RawInputHandler::hash_hid_data_fast(data2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_fnv_hash_empty_data() {
        let data = b"";
        let hash = RawInputHandler::hash_hid_data_fast(data);
        assert_eq!(hash, 0x811c9dc5);
    }

    #[test]
    fn test_fnv_hash_unrolled_vs_remainder() {
        let data_8 = b"12345678";
        let data_9 = b"123456789";
        let hash_8 = RawInputHandler::hash_hid_data_fast(data_8);
        let hash_9 = RawInputHandler::hash_hid_data_fast(data_9);
        assert_ne!(hash_8, hash_9);
    }

    #[test]
    fn test_vid_pid_serial_hash() {
        let vendor_id: u16 = 0x045E;
        let product_id: u16 = 0x0B05;
        let serial = "ABC123";

        let hash1 = RawInputHandler::hash_vid_pid_serial(vendor_id, product_id, serial);
        let hash2 = RawInputHandler::hash_vid_pid_serial(vendor_id, product_id, serial);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_vid_pid_serial_hash_different() {
        let vendor_id: u16 = 0x045E;
        let product_id: u16 = 0x0B05;

        let hash1 = RawInputHandler::hash_vid_pid_serial(vendor_id, product_id, "ABC123");
        let hash2 = RawInputHandler::hash_vid_pid_serial(vendor_id, product_id, "ABC124");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_device_capture_state() {
        let state = DeviceCaptureState::new();
        assert!(!state.captured);
    }

    #[test]
    fn test_device_display_info_cache_clear() {
        let cache = DEVICE_DISPLAY_INFO.get_or_init(scc::HashMap::new);

        let info = DeviceDisplayInfo {
            vendor_id: 0x045E,
            product_id: 0x0B05,
            serial_number: Some("TEST123".to_string()),
        };

        let _ = cache.insert_sync(12345, info);
        assert!(cache.contains_sync(&12345));

        clear_device_display_info_cache();
        assert!(!cache.contains_sync(&12345));
    }

    #[test]
    fn test_thread_local_cache_isolation() {
        // Thread-local cache should be isolated per thread
        LAST_DEVICE_CACHE.with(|cache| {
            unsafe {
                *cache.get() = Some((
                    999,
                    CachedDeviceInfo {
                        device_type: DeviceType::Gamepad(0x045E),
                        vendor_id: 0x045E,
                        product_id: 0x0B05,
                        usage_page: 0x01,
                        usage: 0x05,
                        serial_number: None,
                    },
                ));

                assert!((*cache.get()).is_some());

                // Clear
                *cache.get() = None;
                assert!((*cache.get()).is_none());
            }
        });
    }
}
