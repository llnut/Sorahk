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
//!    - Caches most recently accessed device per thread
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
//! ## Cache Management
//!
//! Device caches are automatically cleaned up when devices are disconnected.
//! The `WM_INPUT_DEVICE_CHANGE` removal event triggers cleanup of all associated
//! cache entries, including device info, HID states, and capture states.

use smallvec::SmallVec;
use std::cell::UnsafeCell;
use std::sync::{Arc, OnceLock, atomic::Ordering};
use std::time::Instant;
use windows::Win32::Foundation::{GetLastError, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

use crate::state::{AppState, DeviceType, InputDevice, InputEvent};
use crate::util::{
    fnv1a_hash_bytes, fnv1a_hash_u32, fnv1a_hash_u64, fnv32, fnv64, likely, unlikely,
};

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

/// Skip HID report bytes
const SKIP_BYTES: usize = 5;

const DEVICE_CAPTURE_FRAMES: usize = 32;
const FRAME_BUFFER_SIZE: usize = 256;

thread_local! {
    /// Thread-local buffer pool for Raw Input data.
    static BUFFER_POOL: UnsafeCell<RingBufferPool> = const { UnsafeCell::new(RingBufferPool::new()) };
    /// Thread-local cache for most recently accessed device.
    static LAST_DEVICE_CACHE: UnsafeCell<Option<(isize, CachedDeviceInfo)>> = const { UnsafeCell::new(None) };
    /// Thread-local HID activation data pool.
    static HID_DATA_POOL: UnsafeCell<HidDataPool> = const { UnsafeCell::new(HidDataPool::new()) };
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

/// HID data memory pool for activation process.
const HID_DATA_POOL_SIZE: usize = 4; // Small pool for activation (typically 1-2 devices)

struct HidDataPool {
    buffers: [Vec<u8>; HID_DATA_POOL_SIZE],
    current: usize,
}

impl HidDataPool {
    const fn new() -> Self {
        const EMPTY_VEC: Vec<u8> = Vec::new();
        Self {
            buffers: [EMPTY_VEC; HID_DATA_POOL_SIZE],
            current: 0,
        }
    }

    /// Returns pooled Vec<u8> with data copied.
    #[inline]
    fn copy_to_vec(&mut self, data: &[u8]) -> Vec<u8> {
        let idx = self.current;
        self.current = (self.current + 1) % HID_DATA_POOL_SIZE;

        let buffer = &mut self.buffers[idx];
        buffer.clear();
        if buffer.capacity() < data.len() {
            buffer.reserve(data.len().saturating_sub(buffer.capacity()));
        }
        buffer.extend_from_slice(data);
        buffer.clone()
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

/// Cached device information
#[derive(Debug, Clone)]
struct CachedDeviceInfo {
    device_type: DeviceType,
    vendor_id: u16,
    product_id: u16,
    usage_page: u16,
    usage: u16,
    serial_number: Option<String>,
}

/// Capture state for a single device during GUI button capture.
/// Tracks frame timestamps to calculate true sustained duration.
#[derive(Debug, Clone, Copy)]
struct DeviceCaptureState {
    /// Pre-allocated inline storage for captured frames (32 frames × 256 bytes)
    frames: [FrameRecord; DEVICE_CAPTURE_FRAMES],
    /// Number of frames currently stored
    frame_count: u8,
}

#[derive(Debug, Clone, Copy)]
struct FrameRecord {
    /// Frame data buffer
    data: [u8; FRAME_BUFFER_SIZE],
    /// Actual frame length
    len: u16,
    /// Timestamp when each frame was received (in milliseconds since epoch)
    timestamp: u64,
}

impl FrameRecord {
    #[inline(always)]
    fn new() -> Self {
        Self {
            data: [0; FRAME_BUFFER_SIZE],
            len: 0,
            timestamp: 0,
        }
    }
}

impl DeviceCaptureState {
    #[inline(always)]
    fn new() -> Self {
        Self {
            frames: [FrameRecord::new(); DEVICE_CAPTURE_FRAMES],
            frame_count: 0,
        }
    }

    #[inline(always)]
    fn with_frame(data: &[u8], timestamp_ms: u64) -> Self {
        let mut state = Self::new();
        let len = data.len().min(FRAME_BUFFER_SIZE);
        let idx = 0;

        let frame_record = &mut state.frames[idx];
        frame_record.data[..len].copy_from_slice(&data[..len]);
        frame_record.len = len as u16;
        frame_record.timestamp = timestamp_ms;
        state.frame_count = 1;
        state
    }

    /// Adds a captured frame with timestamp.
    #[inline(always)]
    fn add_frame(&mut self, data: &[u8], timestamp_ms: u64) {
        if unlikely(self.frame_count >= DEVICE_CAPTURE_FRAMES as u8) {
            return;
        }

        let len = data.len().min(FRAME_BUFFER_SIZE);
        let idx = self.frame_count as usize;
        let data = &data[..len];

        if idx > 0 {
            let last_idx = idx - 1;
            let last_frame_record = &mut self.frames[last_idx];
            let last_frame = &last_frame_record.data[..last_frame_record.len as usize];
            if Self::is_equal_fast(last_frame, data) {
                return;
            }
        }

        let frame_record = &mut self.frames[idx];
        frame_record.data[..len].copy_from_slice(data);
        frame_record.len = len as u16;
        frame_record.timestamp = timestamp_ms;
        self.frame_count += 1;
    }

    /// Returns the frame with the longest sustained duration.
    ///
    /// `now`: current timestamp in milliseconds.
    /// Duration of a stable segment is measured from its first frame's timestamp to:
    ///   - the timestamp of the first different frame that follows, OR
    ///   - `now` if it is the final segment.
    #[inline(always)]
    fn get_most_sustained_frame(&self, now: u64) -> Option<&[u8]> {
        if unlikely(self.frame_count == 0) {
            return None;
        }

        if self.frame_count == 1 {
            let len = self.frames[0].len as usize;
            return Some(&self.frames[0].data[..len]);
        }

        let mut best_idx = 0usize;
        let mut max_duration = 0u64;

        let mut i = 0;
        while i < self.frame_count as usize {
            let seg_start_time = self.frames[i].timestamp;
            let seg_frame = &self.frames[i].data[..self.frames[i].len as usize];

            // Extend segment as far as frames are equal
            let mut j = i;
            while j + 1 < self.frame_count as usize {
                let next_frame = &self.frames[j + 1].data[..self.frames[j + 1].len as usize];
                if Self::is_equal_fast(seg_frame, next_frame) {
                    j += 1;
                } else {
                    break;
                }
            }

            // Compute segment duration
            let seg_end_time = if j + 1 < self.frame_count as usize {
                // Next different frame exists → segment ends at its timestamp
                self.frames[j + 1].timestamp
            } else {
                // Last segment → ends at current time
                now
            };

            let duration = seg_end_time.saturating_sub(seg_start_time);
            if duration > max_duration {
                max_duration = duration;
                best_idx = i; // representative: first frame of the longest segment
            }

            // Jump to next distinct segment
            i = j + 1;
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Fast equality check for byte slices with AVX2 optimization.
    #[inline(always)]
    fn is_equal_fast(a: &[u8], b: &[u8]) -> bool {
        if unlikely(a.len() != b.len()) {
            return false;
        }

        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            return Self::is_equal_avx2(a, b);
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            a == b
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[inline(always)]
    fn is_equal_avx2(a: &[u8], b: &[u8]) -> bool {
        use std::arch::x86_64::*;

        let len = a.len();
        let mut offset = 0;

        unsafe {
            // Process 32-byte chunks with AVX2
            while offset + 32 <= len {
                let va = _mm256_loadu_si256(a.as_ptr().add(offset) as *const __m256i);
                let vb = _mm256_loadu_si256(b.as_ptr().add(offset) as *const __m256i);
                let cmp = _mm256_cmpeq_epi8(va, vb);
                let mask = _mm256_movemask_epi8(cmp);

                if mask != -1 {
                    return false;
                }
                offset += 32;
            }
        }

        // Process remaining bytes
        a[offset..].iter().zip(&b[offset..]).all(|(x, y)| x == y)
    }

    /// Selects the best frame based on the specified capture mode.
    fn get_best_frame(&self, baseline: &[u8], mode: crate::state::CaptureMode) -> Option<&[u8]> {
        use crate::state::CaptureMode;

        if unlikely(self.frame_count == 0) {
            return None;
        }

        match mode {
            CaptureMode::MostSustained => self.get_most_sustained_frame(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            ),
            CaptureMode::AdaptiveIntelligent => self.get_adaptive_intelligent_frame(baseline),
            CaptureMode::MaxChangedBits => self.get_max_changed_bits_frame(baseline),
            CaptureMode::MaxSetBits => self.get_max_set_bits_frame(),
            CaptureMode::LastStable => self.get_last_stable_frame(baseline),
            CaptureMode::HatSwitchOptimized => self.get_hat_switch_optimized_frame(baseline),
            CaptureMode::AnalogOptimized => self.get_analog_optimized_frame(baseline),
        }
    }

    /// Max Changed Bits: Selects frame with most bits changed from baseline.
    #[inline(always)]
    fn get_max_changed_bits_frame(&self, baseline: &[u8]) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }

        let mut max_changed = 0u32;
        let mut best_idx = 0usize;

        for i in 0..self.frame_count as usize {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let changed = Self::count_changed_bits(frame, baseline);
            if changed > max_changed {
                max_changed = changed;
                best_idx = i;
            }
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Max Set Bits: Selects frame with most bits set to 1.
    #[inline(always)]
    fn get_max_set_bits_frame(&self) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }

        let mut max_set = 0u32;
        let mut best_idx = 0usize;

        for i in 0..self.frame_count as usize {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let set_count: u32 = frame[SKIP_BYTES..].iter().map(|b| b.count_ones()).sum();

            if set_count > max_set {
                max_set = set_count;
                best_idx = i;
            }
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Last Stable: Finds last frame that's significantly different from baseline.
    #[inline(always)]
    fn get_last_stable_frame(&self, baseline: &[u8]) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }

        for i in (0..self.frame_count as usize).rev() {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let changed = Self::count_changed_bits(frame, baseline);
            if changed > 0 {
                return Some(frame);
            }
        }

        let idx = (self.frame_count - 1) as usize;
        let len = self.frames[idx].len as usize;
        Some(&self.frames[idx].data[..len])
    }

    /// Hat Switch Optimized: Prioritizes numeric deviation over bit count.
    #[inline(always)]
    fn get_hat_switch_optimized_frame(&self, baseline: &[u8]) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }

        let mut max_score = 0u32;
        let mut best_idx = 0usize;

        for i in 0..self.frame_count as usize {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let mut score = 0u32;
            for byte_idx in SKIP_BYTES..len.min(baseline.len()) {
                let val = frame[byte_idx];
                let base = baseline[byte_idx];
                if val != base {
                    let numeric_diff = val.abs_diff(base) as u32;
                    score += numeric_diff * 100;
                }
            }

            if score > max_score {
                max_score = score;
                best_idx = i;
            }
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Analog Optimized: Prioritizes magnitude of deviation.
    #[inline(always)]
    fn get_analog_optimized_frame(&self, baseline: &[u8]) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }

        let mut max_deviation = 0u32;
        let mut best_idx = 0usize;

        for i in 0..self.frame_count as usize {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let mut deviation = 0u32;
            for byte_idx in SKIP_BYTES..len.min(baseline.len()) {
                let val = frame[byte_idx];
                let base = baseline[byte_idx];
                deviation += val.abs_diff(base) as u32;
            }

            if deviation > max_deviation {
                max_deviation = deviation;
                best_idx = i;
            }
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Adaptive Intelligent: Uses smart scoring based on encoding detection.
    #[inline(always)]
    fn get_adaptive_intelligent_frame(&self, baseline: &[u8]) -> Option<&[u8]> {
        if self.frame_count == 0 {
            return None;
        }
        let mut max_score = 0u32;
        let mut best_idx = 0usize;

        for i in 0..self.frame_count as usize {
            let frame_record = &self.frames[i];
            let len = frame_record.len as usize;
            let frame = &frame_record.data[..len];

            let mut score = 0u32;
            for byte_idx in SKIP_BYTES..len.min(baseline.len()) {
                let val = frame[byte_idx];
                let base = baseline[byte_idx];
                if val != base {
                    let numeric_diff = val.abs_diff(base) as u32;
                    let hamming_dist = (val ^ base).count_ones();

                    // Adaptive weighting based on change pattern
                    if numeric_diff <= 16 && hamming_dist >= 2 {
                        // Likely bitmask: prioritize Hamming distance
                        score += hamming_dist * 150;
                    } else if numeric_diff > 32 {
                        // Likely analog: prioritize numeric diff
                        score += numeric_diff * 100;
                    } else {
                        // Mixed: use both
                        score += numeric_diff * 80 + hamming_dist * 80;
                    }
                }
            }

            if score > max_score {
                max_score = score;
                best_idx = i;
            }
        }

        let len = self.frames[best_idx].len as usize;
        Some(&self.frames[best_idx].data[..len])
    }

    /// Helper: count changed bits between data and baseline.
    #[inline(always)]
    fn count_changed_bits(data: &[u8], baseline: &[u8]) -> u32 {
        data[SKIP_BYTES..]
            .iter()
            .zip(&baseline[SKIP_BYTES..])
            .map(|(d, b)| (d ^ b).count_ones())
            .sum()
    }
}

/// HID device state for button change detection.
#[derive(Debug, Clone)]
struct DeviceHidState {
    /// Whether baseline is established (hot field, placed first)
    baseline_ready: bool,
    /// Baseline HID data (idle state with no buttons pressed)
    baseline_data: Vec<u8>,
    /// Last received HID data for change detection
    last_data: Vec<u8>,
    /// Last update timestamp
    last_update: Instant,
    /// Last generated button_id (for proper release tracking)
    last_button_id: Option<u64>,
}

impl DeviceHidState {
    #[inline]
    fn with_baseline(baseline: Vec<u8>) -> Self {
        Self {
            baseline_ready: true,
            baseline_data: baseline.clone(),
            last_data: baseline,
            last_update: Instant::now(),
            last_button_id: None,
        }
    }
}

/// Handler for Raw Input API messages from HID devices.
pub struct RawInputHandler {
    state: Arc<AppState>,
    /// Lock-free cache for device information.
    device_cache: scc::HashMap<isize, CachedDeviceInfo>,
    /// Capture state tracking during GUI button capture mode (lock-free).
    capture_states: scc::HashMap<isize, DeviceCaptureState>,
    /// Device HID state tracking for button change detection (lock-free).
    device_states: scc::HashMap<isize, DeviceHidState>,
    /// Config baselines keyed by stable device ID (hash of VID:PID:Serial).
    config_baselines: scc::HashMap<u64, Vec<u8>>,
    /// Device ownership manager.
    ownership: crate::input_ownership::DeviceOwnership,
}

impl RawInputHandler {
    /// Removes cached data for a specific device.
    ///
    /// Called when a device is disconnected to free associated resources.
    fn remove_device_caches(&self, handle_key: isize) {
        if let Some(device_info) = self.device_cache.read_sync(&handle_key, |_, v| v.clone()) {
            let vid_pid = (device_info.vendor_id, device_info.product_id);
            if let Some(owner) = self.ownership.get_owner(vid_pid)
                && matches!(owner, crate::input_ownership::InputSource::RawInput(_))
            {
                self.ownership.release_device(vid_pid);
            }
        }

        self.device_cache.remove_sync(&handle_key);
        self.device_states.remove_sync(&handle_key);
        self.capture_states.remove_sync(&handle_key);

        // Clear thread-local cache if it references this device
        LAST_DEVICE_CACHE.with(|cache| unsafe {
            if let Some((cached_handle, _)) = *cache.get()
                && cached_handle == handle_key
            {
                *cache.get() = None;
            }
        });
    }

    /// Resets all HID device states to baseline (idle state).
    /// Called when entering capture mode to ensure clean state detection.
    #[inline]
    pub fn reset_device_states_to_baseline(&self) {
        self.device_states.retain_sync(|_handle, state| {
            if state.baseline_ready {
                state.last_data.copy_from_slice(&state.baseline_data);
            }
            true // Keep all entries
        });
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
    fn new(
        hwnd: HWND,
        state: Arc<AppState>,
        hid_baselines: Vec<crate::config::HidDeviceBaseline>,
        ownership: crate::input_ownership::DeviceOwnership,
    ) -> anyhow::Result<Self> {
        Self::register_devices(hwnd)?;

        // Load baselines into lock-free HashMap
        let config_baselines = scc::HashMap::new();
        for baseline in hid_baselines {
            // Parse device_id to extract VID, PID, Serial and compute hash
            if let Some((vid, pid, serial)) = Self::parse_device_id(&baseline.device_id) {
                let stable_id = if let Some(ref serial) = serial {
                    const SERIAL_FORBIDDEN_CHARS: [char; 3] = ['&', ':', '.'];
                    let is_real_serial = serial.len() > 4
                        && !serial.chars().any(|c| SERIAL_FORBIDDEN_CHARS.contains(&c));

                    if is_real_serial {
                        Self::hash_vid_pid_serial(vid, pid, serial)
                    } else {
                        Self::hash_vid_pid(vid, pid)
                    }
                } else {
                    Self::hash_vid_pid(vid, pid)
                };
                let _ = config_baselines.insert_sync(stable_id, baseline.baseline_data);
            }
        }

        Ok(Self {
            state,
            device_cache: scc::HashMap::new(),
            capture_states: scc::HashMap::new(),
            device_states: scc::HashMap::new(),
            config_baselines,
            ownership,
        })
    }

    /// Starts the Raw Input handler in a dedicated thread.
    pub fn start_thread(
        state: Arc<AppState>,
        hid_baselines: Vec<crate::config::HidDeviceBaseline>,
        ownership: crate::input_ownership::DeviceOwnership,
    ) -> RawInputThread {
        let handle = std::thread::Builder::new()
            .name("rawinput_thread".to_string())
            .spawn(move || {
                if let Err(e) = Self::run_message_loop(state, hid_baselines, ownership) {
                    eprintln!("Raw Input thread error: {}", e);
                }
            })
            .expect("Failed to spawn Raw Input thread");

        RawInputThread { _handle: handle }
    }

    /// Runs the Windows message loop for Raw Input processing.
    fn run_message_loop(
        state: Arc<AppState>,
        hid_baselines: Vec<crate::config::HidDeviceBaseline>,
        ownership: crate::input_ownership::DeviceOwnership,
    ) -> anyhow::Result<()> {
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

            let handler = Self::new(hwnd, state, hid_baselines, ownership)?;
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
    #[allow(non_snake_case)]
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
                match w_param.0 {
                    GIDC_REMOVAL => {
                        // Device disconnected - remove its caches
                        // l_param contains the device handle as isize
                        let handle_key = l_param.0;

                        if let Some(handler) = RAW_INPUT_HANDLER.get() {
                            handler.remove_device_caches(handle_key);
                        }
                    }
                    GIDC_ARRIVAL => {
                        // Device connected - caches will populate on first input
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

            // Thread-local cache lookup for most recent device
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

            let vid_pid = (device_info.vendor_id, device_info.product_id);
            if self.ownership.is_claimed_by_higher_priority(
                vid_pid,
                &crate::input_ownership::InputSource::RawInput(handle_key),
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

            // === NORMAL MODE - Detect button changes and dispatch events ===

            // Generate device identifier (needed for both activation and normal processing)
            let stable_device_id = Self::generate_stable_device_id(&device_info);

            Self::update_device_display_info(stable_device_id, &device_info);

            // Check if device is activated (has baseline)
            // This check MUST be before paused check to allow activation even when paused
            let has_baseline = if let Some(baseline_ready) = self
                .device_states
                .read_sync(&handle_key, |_, state| state.baseline_ready)
            {
                baseline_ready
            } else {
                // New device detected - try to load baseline from config
                if let Some(baseline) = self
                    .config_baselines
                    .read_sync(&stable_device_id, |_, v| v.clone())
                {
                    // Found baseline in config - load it
                    let _ = self
                        .device_states
                        .insert_sync(handle_key, DeviceHidState::with_baseline(baseline));
                    true
                } else {
                    false
                }
            };

            if unlikely(!has_baseline) {
                // Device not activated - handle activation regardless of paused state
                if likely(self.state.is_device_activating(handle_key)) {
                    // Send HID data to activation dialog
                    let pooled_data =
                        HID_DATA_POOL.with(|pool| (*pool.get()).copy_to_vec(data_slice));
                    self.state.send_hid_activation_data(handle_key, pooled_data);
                    return false;
                } else {
                    // Request activation for first time
                    self.request_device_activation(handle_key, &device_info);
                    return false;
                }
            }

            // Detect button changes using baseline comparison
            let changes = self.detect_hid_changes(
                handle_key,
                data_slice,
                stable_device_id,
                device_info.device_type,
            );

            // Check switch key first (before paused check)
            if likely(!changes.is_empty()) {
                let switch_button_id = self
                    .state
                    .switch_key_cache
                    .generic_button_id
                    .load(Ordering::Relaxed);

                // Check for switch key toggle
                if unlikely(switch_button_id != 0) {
                    for (button_id, is_pressed) in &changes {
                        if *button_id == switch_button_id && *is_pressed {
                            self.state.handle_switch_key_toggle();
                            return true;
                        }
                    }
                }
            }

            // Fast paused check (only for activated devices)
            if unlikely(self.state.is_paused()) {
                return false;
            }

            // Dispatch events for each button change
            if likely(!changes.is_empty())
                && let Some(pool) = self.state.get_worker_pool()
            {
                for (button_id, is_pressed) in changes {
                    let device = InputDevice::GenericDevice {
                        device_type: device_info.device_type,
                        button_id,
                    };

                    // Only dispatch if mapping exists
                    if likely(self.state.get_input_mapping(&device).is_some()) {
                        let event = if is_pressed {
                            InputEvent::Pressed(device)
                        } else {
                            InputEvent::Released(device)
                        };
                        pool.dispatch(event);
                    }
                }
                true
            } else {
                false
            }
        }
    }

    /// Captures HID button input at bit level.
    /// Finds the first changed bit in the busiest frame and returns its button_id.
    #[inline(always)]
    fn handle_capture_mode(
        &self,
        handle_key: isize,
        device_info: CachedDeviceInfo,
        current_data: &[u8],
    ) -> bool {
        // Check if device baseline is established
        let has_baseline = self
            .device_states
            .read_sync(&handle_key, |_, state| state.baseline_ready)
            .unwrap_or(false);

        if unlikely(!has_baseline) {
            // Device not activated - handle activation regardless of capture mode
            if likely(self.state.is_device_activating(handle_key)) {
                // Send HID data to activation dialog
                let pooled_data =
                    unsafe { HID_DATA_POOL.with(|pool| (*pool.get()).copy_to_vec(current_data)) };
                self.state.send_hid_activation_data(handle_key, pooled_data);
                return false;
            } else {
                // Request activation for first time
                self.request_device_activation(handle_key, &device_info);
                return false;
            }
        }

        let stable_device_id = Self::generate_stable_device_id(&device_info);
        Self::update_device_display_info(stable_device_id, &device_info);

        // Compare current data with baseline
        let is_baseline = if let Some(baseline) = self
            .device_states
            .read_sync(&handle_key, |_, state| state.baseline_data.clone())
        {
            DeviceCaptureState::is_equal_fast(current_data, &baseline)
        } else {
            false
        };

        if unlikely(is_baseline) {
            // All buttons released - finalize capture if we have frames
            let has_frames = self
                .capture_states
                .read_sync(&handle_key, |_, state| state.frame_count > 0)
                .unwrap_or(false);

            if likely(has_frames) {
                // Get best frame based on capture mode
                let capture_mode = self.state.get_rawinput_capture_mode();

                if let Some((best_data, baseline_data)) = self
                    .device_states
                    .read_sync(&handle_key, |_, state| state.baseline_data.clone())
                    .and_then(|baseline| {
                        self.capture_states
                            .read_sync(&handle_key, |_, state| {
                                state
                                    .get_best_frame(&baseline, capture_mode)
                                    .map(|slice| slice.to_vec())
                            })
                            .flatten()
                            .map(|best| (best, baseline))
                    })
                {
                    // Hash all changed bit positions to uniquely identify this input pattern
                    let button_id = Self::hash_changed_bit_pattern(
                        &best_data,
                        &baseline_data,
                        stable_device_id,
                    );

                    let device = InputDevice::GenericDevice {
                        device_type: device_info.device_type,
                        button_id,
                    };

                    let _ = self.state.get_raw_input_capture_sender().send(device);
                    self.capture_states.remove_sync(&handle_key);

                    return true;
                }
            }
            return false;
        }

        // Not baseline - add frame with timestamp
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let updated = self
            .capture_states
            .update_sync(&handle_key, |_, capture_state| {
                capture_state.add_frame(current_data, timestamp_ms);
            })
            .is_some();

        if unlikely(!updated) {
            // Create new state with first frame
            let _ = self.capture_states.insert_sync(
                handle_key,
                DeviceCaptureState::with_frame(current_data, timestamp_ms),
            );
        }

        false
    }

    /// Hashes all changed bit positions to create a unique button_id for this input pattern.
    /// Returns button_id in format: (device_id << 32) | hash(changed_bit_positions)
    ///
    /// Uses FNV-1a to hash the positions of all changed bits. This ensures:
    /// - Different input patterns get unique IDs (e.g., joystick UP vs RIGHT vs UP+RIGHT)
    /// - Same input pattern always gets the same ID (deterministic)
    /// - Extremely fast with minimal collisions
    #[inline(always)]
    fn hash_changed_bit_pattern(data: &[u8], baseline: &[u8], stable_device_id: u64) -> u64 {
        let min_len = data.len().min(baseline.len());
        let mut hash = fnv32::OFFSET_BASIS;

        // Hash each changed bit position
        for byte_idx in SKIP_BYTES..min_len {
            let data_byte = data[byte_idx];
            let baseline_byte = baseline[byte_idx];
            let mut diff = data_byte ^ baseline_byte;

            if diff != 0 {
                // Process each changed bit in this byte
                while diff != 0 {
                    let bit_idx = diff.trailing_zeros();

                    // Hash the position (byte_idx, bit_idx) using FNV-1a
                    hash = fnv1a_hash_u32(hash, byte_idx as u32);
                    hash = fnv1a_hash_u32(hash, bit_idx);

                    // Clear the lowest set bit (BLSR instruction)
                    diff &= diff - 1;
                }
            }
        }

        (stable_device_id << 32) | (hash as u64)
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

    /// Generate stable device ID for consistent identification.
    ///
    /// Uses a hybrid strategy:
    /// - If device has a valid serial number (not Windows instance ID), use VID+PID+Serial
    /// - Otherwise, use only VID+PID to support device reconnection
    #[inline(always)]
    fn generate_stable_device_id(device_info: &CachedDeviceInfo) -> u64 {
        if let Some(ref serial) = device_info.serial_number {
            // Check if this is a real serial number or Windows instance ID
            // Windows instance IDs contain '&' (e.g., "6&2c5b8c5d&0&0000")
            // Real serial numbers are typically longer and don't contain '&'
            let is_real_serial = !serial.contains('&') && serial.len() > 4;

            if is_real_serial {
                return Self::hash_vid_pid_serial(
                    device_info.vendor_id,
                    device_info.product_id,
                    serial,
                );
            }
        }

        // No serial or unreliable serial - use only VID+PID
        Self::hash_vid_pid(device_info.vendor_id, device_info.product_id)
    }

    /// Computes FNV-1a hash for device identification using VID, PID, and serial number.
    #[inline(always)]
    fn hash_vid_pid_serial(vendor_id: u16, product_id: u16, serial: &str) -> u64 {
        let mut hash = fnv64::OFFSET_BASIS;
        hash = fnv1a_hash_u64(hash, vendor_id as u64);
        hash = fnv1a_hash_u64(hash, product_id as u64);
        hash = fnv1a_hash_bytes(hash, serial.as_bytes());
        hash
    }

    /// Computes FNV-1a hash for device identification using VID and PID only.
    #[inline(always)]
    fn hash_vid_pid(vendor_id: u16, product_id: u16) -> u64 {
        let mut hash = fnv64::OFFSET_BASIS;
        hash = fnv1a_hash_u64(hash, vendor_id as u64);
        hash = fnv1a_hash_u64(hash, product_id as u64);
        hash
    }

    /// Parses device_id string (format: "VID:PID" or "VID:PID:Serial") into components.
    #[inline]
    fn parse_device_id(device_id: &str) -> Option<(u16, u16, Option<String>)> {
        let parts: Vec<&str> = device_id.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let vid = u16::from_str_radix(parts[0], 16).ok()?;
        let pid = u16::from_str_radix(parts[1], 16).ok()?;
        let serial = if parts.len() >= 3 {
            Some(parts[2].to_string())
        } else {
            None
        };

        Some((vid, pid, serial))
    }

    /// Request device activation from GUI.
    #[inline(never)]
    #[cold]
    fn request_device_activation(&self, handle_key: isize, device_info: &CachedDeviceInfo) {
        let device_name = format!(
            "{} ({:04X}:{:04X})",
            match device_info.usage {
                HID_USAGE_GAMEPAD => "Gamepad",
                HID_USAGE_JOYSTICK => "Joystick",
                HID_USAGE_MULTI_AXIS => "Controller",
                _ => "HID Device",
            },
            device_info.vendor_id,
            device_info.product_id
        );

        let request = crate::state::HidActivationRequest {
            device_handle: handle_key,
            device_name,
            vid: device_info.vendor_id,
            pid: device_info.product_id,
            usage_page: device_info.usage_page,
            usage: device_info.usage,
        };

        self.state.request_hid_activation(request);
    }

    /// Detects HID button state changes at bit level using optimized bit operations.
    /// Returns list of (button_id, is_pressed) for detected changes.
    ///
    /// Strategy:
    /// 1. Collect all currently active bits (relative to baseline)
    /// 2. Compare with previously active bits to find press/release events
    /// 3. For each event, generate both combo and individual button_ids
    ///    - Combo: for multi-button combos or unique patterns
    ///    - Individual: for simultaneous independent keys
    #[inline]
    fn detect_hid_changes(
        &self,
        handle_key: isize,
        current_data: &[u8],
        stable_device_id: u64,
        _device_type: crate::state::DeviceType,
    ) -> SmallVec<[(u64, bool); 8]> {
        let mut changes = SmallVec::new();

        if let Some(result_changes) = self.device_states.update_sync(&handle_key, |_, state| {
            let mut temp_changes: SmallVec<[(u64, bool); 8]> = SmallVec::new();

            if unlikely(current_data.len() <= SKIP_BYTES) {
                return temp_changes;
            }

            let min_len = current_data
                .len()
                .min(state.last_data.len())
                .min(state.baseline_data.len());

            // Collect all currently active bit positions (relative to baseline)
            let mut curr_active: SmallVec<[(usize, u32); 8]> = SmallVec::new();
            let mut prev_active: SmallVec<[(usize, u32); 8]> = SmallVec::new();

            #[allow(clippy::needless_range_loop)]
            for byte_idx in SKIP_BYTES..min_len {
                let curr_byte = current_data[byte_idx];
                let prev_byte = state.last_data[byte_idx];
                let baseline_byte = state.baseline_data[byte_idx];

                // Collect currently active bits
                let mut curr_diff = curr_byte ^ baseline_byte;
                while curr_diff != 0 {
                    let bit_idx = curr_diff.trailing_zeros();
                    curr_active.push((byte_idx, bit_idx));
                    curr_diff &= curr_diff - 1;
                }

                // Collect previously active bits
                let mut prev_diff = prev_byte ^ baseline_byte;
                while prev_diff != 0 {
                    let bit_idx = prev_diff.trailing_zeros();
                    prev_active.push((byte_idx, bit_idx));
                    prev_diff &= prev_diff - 1;
                }
            }

            // Detect changes in active button combination
            if curr_active != prev_active {
                // If previous combination existed and is different, release it
                if !prev_active.is_empty()
                    && let Some(last_id) = state.last_button_id
                {
                    temp_changes.push((last_id, false));
                }

                // If new combination exists, press it
                if !curr_active.is_empty() {
                    Self::generate_button_events(
                        &curr_active,
                        stable_device_id,
                        true,
                        &mut temp_changes,
                    );

                    // Save the new button_id for next release
                    if let Some(&(new_button_id, _)) = temp_changes.last() {
                        state.last_button_id = Some(new_button_id);
                    }
                } else {
                    // All released
                    state.last_button_id = None;
                }
            }

            // Update state
            state.last_data.copy_from_slice(current_data);
            state.last_update = Instant::now();

            temp_changes
        }) {
            changes = result_changes;
        }

        changes
    }

    /// Generates button events for a set of bit positions.
    /// Only creates a single combo button_id from all positions together.
    #[inline(always)]
    fn generate_button_events(
        positions: &[(usize, u32)],
        stable_device_id: u64,
        is_pressed: bool,
        events: &mut SmallVec<[(u64, bool); 8]>,
    ) {
        // Generate single combo button_id (hash all positions together)
        let combo_id = Self::hash_bit_positions(positions, stable_device_id);
        events.push((combo_id, is_pressed));
    }

    /// Hashes a set of bit positions using FNV-1a for consistent button_id generation.
    /// Extremely fast with minimal collisions.
    #[inline(always)]
    fn hash_bit_positions(positions: &[(usize, u32)], stable_device_id: u64) -> u64 {
        let mut hash = fnv32::OFFSET_BASIS;
        for &(byte_idx, bit_idx) in positions {
            hash = fnv1a_hash_u32(hash, byte_idx as u32);
            hash = fnv1a_hash_u32(hash, bit_idx);
        }

        (stable_device_id << 32) | (hash as u64)
    }

    /// Updates the global device display information cache.
    #[inline(always)]
    fn update_device_display_info(stable_device_id: u64, device_info: &CachedDeviceInfo) {
        let display_info = DeviceDisplayInfo {
            vendor_id: device_info.vendor_id,
            product_id: device_info.product_id,
            serial_number: device_info.serial_number.clone(),
        };

        let cache = DEVICE_DISPLAY_INFO.get_or_init(scc::HashMap::new);
        // Only use lower 32 bits to match button_id format (device_id is stored in high 32 bits of button_id)
        let _ = cache.upsert_sync(stable_device_id & 0xFFFFFFFF, display_info);
    }
}

/// Activates a HID device with the given baseline data.
///
/// Updates runtime device state and baseline cache for reconnection support.
/// The caller is responsible for persisting baseline data to configuration file.
#[inline]
pub fn activate_hid_device(device_handle: isize, baseline_data: Vec<u8>) {
    if let Some(handler) = RAW_INPUT_HANDLER.get() {
        let _ = handler.device_states.insert_sync(
            device_handle,
            DeviceHidState::with_baseline(baseline_data.clone()),
        );

        if let Some(device_info) = handler
            .device_cache
            .read_sync(&device_handle, |_, v| v.clone())
        {
            let stable_device_id = RawInputHandler::generate_stable_device_id(&device_info);
            let _ = handler
                .config_baselines
                .insert_sync(stable_device_id, baseline_data);
        }
    }
}

/// Resets all HID device states to baseline and clears capture states.
pub fn reset_hid_device_states() {
    if let Some(handler) = RAW_INPUT_HANDLER.get() {
        handler.reset_device_states_to_baseline();
        handler.capture_states.retain_sync(|_, _| false);
    }
}

/// Gets device info for a device handle.
pub fn get_device_info_for_handle(device_handle: isize) -> Option<(u16, u16, Option<String>)> {
    if let Some(handler) = RAW_INPUT_HANDLER.get()
        && let Some(device_info) = handler
            .device_cache
            .read_sync(&device_handle, |_, v| v.clone())
    {
        return Some((
            device_info.vendor_id,
            device_info.product_id,
            device_info.serial_number,
        ));
    }
    None
}

/// Retrieves cached display information for a device.
pub fn get_device_display_info(stable_device_id: u64) -> Option<DeviceDisplayInfo> {
    let cache = DEVICE_DISPLAY_INFO.get()?;
    cache
        .get_sync(&stable_device_id)
        .map(|entry| entry.get().clone())
}

/// Registers device display information.
///
/// Used for devices not detected through Raw Input (e.g., XInput).
pub fn register_device_display_info(stable_device_id: u64, info: DeviceDisplayInfo) {
    if let Some(cache) = DEVICE_DISPLAY_INFO.get() {
        let _ = cache.upsert_sync(stable_device_id, info);
    }
}

/// Clears the device display information cache.
pub fn clear_device_display_info_cache() {
    if let Some(cache) = DEVICE_DISPLAY_INFO.get() {
        cache.clear_sync();
    }
}

/// Clears baseline data for a device identified by VID:PID.
///
/// Removes activation data from runtime state and configuration cache.
#[inline]
pub fn clear_device_baseline(vid: u16, pid: u16) {
    if let Some(handler) = RAW_INPUT_HANDLER.get() {
        let stable_id = RawInputHandler::hash_vid_pid(vid, pid);
        handler.config_baselines.remove_sync(&stable_id);

        let mut handles_to_remove = smallvec::SmallVec::<[isize; 4]>::new();

        handler.device_cache.retain_sync(|&handle, info| {
            if info.vendor_id == vid && info.product_id == pid {
                handles_to_remove.push(handle);
            }
            true
        });

        for handle in handles_to_remove {
            handler.device_states.remove_sync(&handle);
            handler.capture_states.remove_sync(&handle);
        }
    }

    // Release device ownership to allow re-claiming
    crate::input_manager::release_device_ownership((vid, pid));
}

/// Enumerates all HID devices currently connected.
pub fn enumerate_hid_devices() -> Vec<crate::gui::device_manager_dialog::HidDeviceInfo> {
    use windows::Win32::UI::Input::*;

    let mut devices = Vec::new();

    unsafe {
        let mut device_count = 0u32;
        if GetRawInputDeviceList(
            None,
            &mut device_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        ) == u32::MAX
        {
            return devices;
        }

        if device_count == 0 {
            return devices;
        }

        let mut device_list = vec![RAWINPUTDEVICELIST::default(); device_count as usize];
        let result = GetRawInputDeviceList(
            Some(device_list.as_mut_ptr()),
            &mut device_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        );

        if result == u32::MAX {
            return devices;
        }

        for device in device_list.iter().take(device_count as usize) {
            if device.dwType != RIM_TYPEHID {
                continue;
            }

            let handle = device.hDevice;

            // Get device info
            let mut size = 0u32;
            if GetRawInputDeviceInfoW(Some(handle), RIDI_DEVICEINFO, None, &mut size) != 0 {
                continue;
            }

            let mut buffer = vec![0u8; size as usize];
            if GetRawInputDeviceInfoW(
                Some(handle),
                RIDI_DEVICEINFO,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut size,
            ) == u32::MAX
            {
                continue;
            }

            let device_info = &*(buffer.as_ptr() as *const RID_DEVICE_INFO);
            if device_info.dwType != RIM_TYPEHID {
                continue;
            }

            let hid = device_info.Anonymous.hid;

            // Get device name
            let mut name_size = 0u32;
            if GetRawInputDeviceInfoW(Some(handle), RIDI_DEVICENAME, None, &mut name_size) != 0 {
                continue;
            }

            let mut name_buffer = vec![0u16; name_size as usize];
            if GetRawInputDeviceInfoW(
                Some(handle),
                RIDI_DEVICENAME,
                Some(name_buffer.as_mut_ptr() as *mut _),
                &mut name_size,
            ) == u32::MAX
            {
                continue;
            }

            let device_path = String::from_utf16_lossy(&name_buffer);
            let device_name = extract_device_name(&device_path);

            // Parse VID/PID from device path
            let (vid, pid) =
                if let Some((v, p)) = RawInputHandler::parse_vid_pid_from_path(&device_path) {
                    (v, p)
                } else {
                    continue;
                };

            devices.push(crate::gui::device_manager_dialog::HidDeviceInfo {
                vid,
                pid,
                device_name,
                usage_page: hid.usUsagePage,
                usage: hid.usUsage,
            });
        }
    }

    devices
}

impl RawInputHandler {
    /// Parses VID and PID from a device path.
    /// Device path format: \\?\HID#VID_045E&PID_028E#...
    fn parse_vid_pid_from_path(path: &str) -> Option<(u16, u16)> {
        let upper = path.to_uppercase();

        let vid_pos = upper.find("VID_")?;
        let pid_pos = upper.find("PID_")?;

        let vid_str = &upper[vid_pos + 4..].split(&['&', '#'][..]).next()?;
        let pid_str = &upper[pid_pos + 4..].split(&['&', '#'][..]).next()?;

        let vid = u16::from_str_radix(vid_str, 16).ok()?;
        let pid = u16::from_str_radix(pid_str, 16).ok()?;

        Some((vid, pid))
    }
}

/// Extracts a human-readable device name from the device path.
fn extract_device_name(path: &str) -> String {
    if let Some(name_part) = path.split('#').nth(1) {
        name_part.replace('_', " ").to_string()
    } else {
        "Unknown HID Device".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_device_capture_state_initialization() {
        let state = DeviceCaptureState::new();
        assert_eq!(state.frame_count, 0);
    }

    #[test]
    fn test_device_capture_state_add_frame() {
        let mut state = DeviceCaptureState::new();
        let data = vec![0, 0, 0, 0, 0, 0x01, 0x02, 0x03];

        state.add_frame(&data, 1000);
        assert_eq!(state.frame_count, 1);
        assert!(state.get_most_sustained_frame(1000).is_some());
    }

    #[test]
    fn test_device_capture_state_sustained_duration() {
        let mut state = DeviceCaptureState::new();
        let mut time = 1000u64;

        // Pattern A: 100ms duration (3 frames, 50ms apart)
        let pattern_a = vec![0, 0, 0, 0, 0, 0x01, 0x00];
        state.add_frame(&pattern_a, time);
        time += 50;
        state.add_frame(&pattern_a, time);
        time += 50;
        state.add_frame(&pattern_a, time);
        time += 50;

        // Pattern B: 200ms duration (3 frames, 100ms apart) - longest
        let pattern_b = vec![0, 0, 0, 0, 0, 0x03, 0x00];
        state.add_frame(&pattern_b, time);
        time += 100;
        state.add_frame(&pattern_b, time);
        time += 100;
        state.add_frame(&pattern_b, time);
        time += 100;

        // Pattern C: 30ms duration (2 frames, 30ms apart)
        let pattern_c = vec![0, 0, 0, 0, 0, 0x02, 0x00];
        state.add_frame(&pattern_c, time);
        time += 30;
        state.add_frame(&pattern_c, time);

        // Total unique patterns: pattern_a, pattern_b, pattern_c = 3 patterns
        assert_eq!(state.frame_count, 3);

        let sustained = state.get_most_sustained_frame(time).unwrap();
        assert_eq!(&sustained[5], &0x03); // Pattern B has longest duration
    }

    #[test]
    fn test_device_capture_state_capacity_limit() {
        let mut state = DeviceCaptureState::new();
        let data = vec![0, 0, 0, 0, 0, 0x01];

        // Add 32 frames
        for i in 0..32u8 {
            let mut pattern = data.clone();
            pattern[5] = i;
            state.add_frame(&pattern, i as u64 * 10);
        }
        assert_eq!(state.frame_count, 32);

        // Try to add 33rd frame - should be ignored
        let mut new_pattern = data.clone();
        new_pattern[5] = 0xFF;
        state.add_frame(&new_pattern, 1000);
        assert_eq!(state.frame_count, 32);
    }

    #[test]
    fn test_device_capture_state_joystick_scenario() {
        let mut state = DeviceCaptureState::new();
        let mut time = 1000u64;

        // Simulate joystick: Right (0x0C) for 50ms, then Right-Up (0x08) for 150ms

        // Right for 50ms (2 frames)
        let right = vec![0, 0, 0, 0, 0, 0x0C, 0x00];
        state.add_frame(&right, time);
        time += 25;
        state.add_frame(&right, time);
        time += 25;

        // Right-Up for 150ms (3 frames) - should be selected
        let right_up = vec![0, 0, 0, 0, 0, 0x08, 0x00];
        state.add_frame(&right_up, time);
        time += 50;
        state.add_frame(&right_up, time);
        time += 50;
        state.add_frame(&right_up, time);
        time += 50;

        // Total unique patterns: right and right_up = 2 patterns
        assert_eq!(state.frame_count, 2);

        let sustained = state.get_most_sustained_frame(time).unwrap();
        assert_eq!(&sustained[5], &0x08); // Right-Up is selected
    }

    #[test]
    fn test_device_hid_state_with_baseline() {
        let baseline = vec![0x00, 0xFF, 0x7F, 0xFF, 0x7F, 0x00, 0x80, 0x00];
        let state = DeviceHidState::with_baseline(baseline.clone());

        assert!(state.baseline_ready);
        assert_eq!(state.baseline_data, baseline);
        assert_eq!(state.last_data, baseline);
    }

    #[test]
    fn test_parse_device_id_with_serial() {
        let device_id = "045E:0B05:ABC123";
        let result = RawInputHandler::parse_device_id(device_id);

        assert!(result.is_some());
        let (vid, pid, serial) = result.unwrap();
        assert_eq!(vid, 0x045E);
        assert_eq!(pid, 0x0B05);
        assert_eq!(serial, Some("ABC123".to_string()));
    }

    #[test]
    fn test_parse_device_id_without_serial() {
        let device_id = "045E:0B05";
        let result = RawInputHandler::parse_device_id(device_id);

        assert!(result.is_some());
        let (vid, pid, serial) = result.unwrap();
        assert_eq!(vid, 0x045E);
        assert_eq!(pid, 0x0B05);
        assert!(serial.is_none());
    }

    #[test]
    fn test_parse_device_id_invalid() {
        assert!(RawInputHandler::parse_device_id("invalid").is_none());
        assert!(RawInputHandler::parse_device_id("").is_none());
        assert!(RawInputHandler::parse_device_id("ZZZZ:0B05").is_none());
    }
}
