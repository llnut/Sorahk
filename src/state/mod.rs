//! Application state management.

pub mod handlers;
pub mod parsing;
pub mod simulation;
#[cfg(test)]
mod tests;
pub mod types;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use scc::{AtomicShared, Guard, Shared, Tag};
use smallvec::SmallVec;

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PWSTR;

use std::str::FromStr;

use crate::config::AppConfig;
use crate::i18n::Language;
use crate::util::likely;

pub use types::*;

static GLOBAL_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

#[derive(Debug, Clone)]
pub(crate) struct ProcessInfo {
    pub name: Option<String>,
    pub timestamp: Instant,
}

pub struct AppState {
    language: AtomicU8,
    show_tray_icon: AtomicBool,
    show_notifications: AtomicBool,
    pub switch_key_cache: SwitchKeyCache,
    pub should_exit: Arc<AtomicBool>,
    is_paused: AtomicBool,
    show_window_requested: AtomicBool,
    show_about_requested: AtomicBool,
    input_timeout: AtomicU64,
    worker_count: AtomicU64,
    configured_worker_count: usize,
    pub(crate) input_mappings: scc::HashMap<InputDevice, InputMappingInfo>,
    pub(crate) worker_pool: OnceLock<Arc<dyn EventDispatcher>>,
    notification_sender: OnceLock<Sender<NotificationEvent>>,
    process_whitelist: AtomicShared<Vec<String>>,
    pub(crate) cached_process_info: AtomicShared<ProcessInfo>,
    pub(crate) pressed_keys: scc::HashSet<u32>,
    active_combo_triggers: scc::HashMap<InputDevice, SmallVec<[u32; 8]>>,
    cached_turbo_keyboard: [AtomicBool; 256],
    cached_turbo_other: scc::HashMap<InputDevice, bool>,
    pub(crate) cached_combo_index: scc::HashMap<u32, Vec<InputDevice>>,
    cached_xinput_combos: scc::HashMap<DeviceType, Vec<Vec<u32>>>,
    raw_input_capture_sender: Sender<InputDevice>,
    raw_input_capture_receiver: Receiver<InputDevice>,
    is_capturing_raw_input: AtomicBool,
    rawinput_capture_mode: AtomicShared<CaptureMode>,
    xinput_capture_mode: AtomicShared<crate::config::XInputCaptureMode>,
    hid_activation_sender: Sender<HidActivationRequest>,
    hid_activation_receiver: Receiver<HidActivationRequest>,
    hid_activation_data_sender: Sender<(isize, Vec<u8>)>,
    hid_activation_data_receiver: Receiver<(isize, Vec<u8>)>,
    activating_device_handle: std::sync::atomic::AtomicIsize,
    xinput_cache_invalid: AtomicBool,
    pub(crate) sequence_matcher: crate::sequence_matcher::SequenceMatcher,
    pub(crate) last_sequence_device: AtomicShared<InputDevice>,
    pub(crate) last_sequence_inputs: AtomicShared<Vec<InputDevice>>,
    pub(crate) last_mouse_x: std::sync::atomic::AtomicI32,
    pub(crate) last_mouse_y: std::sync::atomic::AtomicI32,
    pub(crate) last_mouse_direction: std::sync::atomic::AtomicU8,
}

impl AppState {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let switch_key_cache = SwitchKeyCache::new();
        Self::update_switch_key_cache(&switch_key_cache, &config.switch_key)?;

        let input_mappings_map = Self::create_input_mappings(&config)?;

        let input_mappings = scc::HashMap::new();
        for (k, v) in input_mappings_map {
            let _ = input_mappings.insert_sync(k, v);
        }

        let cached_turbo_keyboard: [AtomicBool; 256] =
            std::array::from_fn(|_| AtomicBool::new(true));
        let cached_turbo_other = scc::HashMap::new();
        let cached_combo_index: scc::HashMap<u32, Vec<InputDevice>> = scc::HashMap::new();
        let cached_xinput_combos: scc::HashMap<DeviceType, Vec<Vec<u32>>> = scc::HashMap::new();

        for mapping in config.mappings.iter() {
            if let Some(device) = parsing::input_name_to_device(&mapping.trigger_key) {
                match &device {
                    InputDevice::Keyboard(vk) if *vk < 256 => {
                        cached_turbo_keyboard[*vk as usize]
                            .store(mapping.turbo_enabled, Ordering::Relaxed);
                    }
                    InputDevice::KeyCombo(keys) => {
                        if let Some(&last_key) = keys.last() {
                            let mut combos = cached_combo_index
                                .get_sync(&last_key)
                                .map(|v| v.get().clone())
                                .unwrap_or_default();
                            combos.push(device.clone());
                            let _ = cached_combo_index.upsert_sync(last_key, combos);
                        }
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                    InputDevice::XInputCombo {
                        device_type,
                        button_ids,
                    } => {
                        let mut combos = cached_xinput_combos
                            .get_sync(device_type)
                            .map(|v| v.get().clone())
                            .unwrap_or_default();
                        combos.push(button_ids.clone());
                        let _ = cached_xinput_combos.upsert_sync(*device_type, combos);
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                    _ => {
                        let _ =
                            cached_turbo_other.insert_sync(device.clone(), mapping.turbo_enabled);
                    }
                }
            }

            if mapping.is_sequence_trigger()
                && let Some(seq_str) = &mapping.trigger_sequence {
                    let parts: Vec<&str> = seq_str.split(',').collect();

                    for part in parts {
                        let part_trimmed = part.trim();
                        if let Some(device) = parsing::input_name_to_device(part_trimmed) {
                            match device {
                                InputDevice::XInputCombo {
                                    device_type,
                                    button_ids,
                                } => {
                                    let mut combos = cached_xinput_combos
                                        .get_sync(&device_type)
                                        .map(|v| v.get().clone())
                                        .unwrap_or_default();

                                    if !combos.iter().any(|c| c == &button_ids) {
                                        combos.push(button_ids);
                                        let _ =
                                            cached_xinput_combos.upsert_sync(device_type, combos);
                                    }
                                }
                                InputDevice::KeyCombo(ref keys) => {
                                    if let Some(&last_key) = keys.last() {
                                        let mut combos = cached_combo_index
                                            .get_sync(&last_key)
                                            .map(|v| v.clone())
                                            .unwrap_or_default();

                                        if !combos.iter().any(|c| c == &device) {
                                            combos.push(device.clone());
                                            let _ =
                                                cached_combo_index.upsert_sync(last_key, combos);
                                        }
                                    }
                                }
                            _ => {}
                            }
                        }
                    }
                }
        }

        let (raw_input_capture_sender, raw_input_capture_receiver) = crossbeam_channel::unbounded();
        let (hid_activation_sender, hid_activation_receiver) = crossbeam_channel::unbounded();
        let (hid_activation_data_sender, hid_activation_data_receiver) =
            crossbeam_channel::unbounded();

        let sequence_matcher = crate::sequence_matcher::SequenceMatcher::new();
        for mapping in config.mappings.iter() {
            if mapping.is_sequence_trigger()
                && let Some(seq_str) = &mapping.trigger_sequence
                    && let Ok(sequence) = crate::sequence_matcher::parse_sequence_string(
                        seq_str,
                        Some(mapping.sequence_window_ms),
                    ) {
                        sequence_matcher.register_sequence(sequence);
                    }
        }

        Ok(Self {
            language: AtomicU8::new(config.language.to_u8()),
            show_tray_icon: AtomicBool::new(config.show_tray_icon),
            show_notifications: AtomicBool::new(config.show_notifications),
            switch_key_cache,
            should_exit: Arc::new(AtomicBool::new(false)),
            is_paused: AtomicBool::new(false),
            show_window_requested: AtomicBool::new(false),
            show_about_requested: AtomicBool::new(false),
            input_timeout: AtomicU64::new(config.input_timeout),
            worker_count: AtomicU64::new(0),
            process_whitelist: AtomicShared::from(Shared::new(config.process_whitelist.clone())),
            configured_worker_count: config.worker_count,
            input_mappings,
            worker_pool: OnceLock::new(),
            notification_sender: OnceLock::new(),
            cached_process_info: AtomicShared::from(Shared::new(ProcessInfo {
                name: None,
                timestamp: Instant::now(),
            })),
            pressed_keys: scc::HashSet::new(),
            active_combo_triggers: scc::HashMap::new(),
            cached_turbo_keyboard,
            cached_turbo_other,
            cached_combo_index,
            cached_xinput_combos,
            raw_input_capture_sender,
            raw_input_capture_receiver,
            is_capturing_raw_input: AtomicBool::new(false),
            rawinput_capture_mode: AtomicShared::from(Shared::new(
                CaptureMode::from_str(&config.rawinput_capture_mode).unwrap(),
            )),
            xinput_capture_mode: AtomicShared::from(Shared::new(
                crate::config::XInputCaptureMode::from_str(&config.xinput_capture_mode)?,
            )),
            hid_activation_sender,
            hid_activation_receiver,
            hid_activation_data_sender,
            hid_activation_data_receiver,
            activating_device_handle: std::sync::atomic::AtomicIsize::new(-1),
            xinput_cache_invalid: AtomicBool::new(false),
            sequence_matcher,
            last_sequence_device: AtomicShared::default(),
            last_sequence_inputs: AtomicShared::default(),
            last_mouse_x: std::sync::atomic::AtomicI32::new(0),
            last_mouse_y: std::sync::atomic::AtomicI32::new(0),
            last_mouse_direction: std::sync::atomic::AtomicU8::new(0),
        })
    }

    pub fn reload_config(&self, config: AppConfig) -> anyhow::Result<()> {
        Self::update_switch_key_cache(&self.switch_key_cache, &config.switch_key)?;

        self.language.store(config.language.to_u8(), Ordering::Relaxed);
        self.show_tray_icon.store(config.show_tray_icon, Ordering::Relaxed);
        self.show_notifications.store(config.show_notifications, Ordering::Relaxed);
        self.input_timeout.store(config.input_timeout, Ordering::Relaxed);
        let new_rawinput_mode =
            Shared::new(CaptureMode::from_str(&config.rawinput_capture_mode).unwrap());
        let _ = self
            .rawinput_capture_mode
            .swap((Some(new_rawinput_mode), Tag::None), Ordering::Release);

        let new_xinput_mode = Shared::new(crate::config::XInputCaptureMode::from_str(
            &config.xinput_capture_mode,
        )?);
        let _ = self
            .xinput_capture_mode
            .swap((Some(new_xinput_mode), Tag::None), Ordering::Release);
        let new_input_mappings = Self::create_input_mappings(&config)?;
        self.input_mappings.clear_sync();
        for (k, v) in new_input_mappings {
            let _ = self.input_mappings.insert_sync(k, v);
        }

        self.cached_turbo_other.clear_sync();
        self.cached_combo_index.clear_sync();
        self.cached_xinput_combos.clear_sync();
        for i in 0..256 {
            self.cached_turbo_keyboard[i].store(true, Ordering::Relaxed);
        }

        for mapping in config.mappings.iter() {
            if let Some(device) = parsing::input_name_to_device(&mapping.trigger_key) {
                match &device {
                    InputDevice::Keyboard(vk) if *vk < 256 => {
                        self.cached_turbo_keyboard[*vk as usize]
                            .store(mapping.turbo_enabled, Ordering::Relaxed);
                    }
                    InputDevice::KeyCombo(keys) => {
                        if let Some(&last_key) = keys.last() {
                            let mut combos = self
                                .cached_combo_index
                                .get_sync(&last_key)
                                .map(|v| v.clone())
                                .unwrap_or_default();
                            combos.push(device.clone());

                            let _ = self.cached_combo_index.upsert_sync(last_key, combos);
                        }
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                    InputDevice::XInputCombo {
                        device_type,
                        button_ids,
                    } => {
                        let mut combos = self
                            .cached_xinput_combos
                            .get_sync(device_type)
                            .map(|v| v.get().clone())
                            .unwrap_or_default();
                        combos.push(button_ids.clone());
                        let _ = self.cached_xinput_combos.upsert_sync(*device_type, combos);
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                    _ => {
                        let _ = self
                            .cached_turbo_other
                            .insert_sync(device, mapping.turbo_enabled);
                    }
                }
            }

            if mapping.is_sequence_trigger()
                && let Some(seq_str) = &mapping.trigger_sequence {
                    let parts: Vec<&str> = seq_str.split(',').collect();

                    for part in parts {
                        let part_trimmed = part.trim();
                        if let Some(device) = parsing::input_name_to_device(part_trimmed) {
                            match device {
                                InputDevice::XInputCombo {
                                    device_type,
                                    button_ids,
                                } => {
                                    let mut combos = self
                                        .cached_xinput_combos
                                        .get_sync(&device_type)
                                        .map(|v| v.get().clone())
                                        .unwrap_or_default();

                                    if !combos.iter().any(|c| c == &button_ids) {
                                        combos.push(button_ids);
                                        let _ = self
                                            .cached_xinput_combos
                                            .upsert_sync(device_type, combos);
                                    }
                                }
                                InputDevice::KeyCombo(ref keys) => {
                                    if let Some(&last_key) = keys.last() {
                                        let mut combos = self
                                            .cached_combo_index
                                            .get_sync(&last_key)
                                            .map(|v| v.clone())
                                            .unwrap_or_default();

                                        if !combos.iter().any(|c| c == &device) {
                                            combos.push(device.clone());
                                            let _ = self
                                                .cached_combo_index
                                                .upsert_sync(last_key, combos);
                                        }
                                    }
                                }
                            _ => {}
                            }
                        }
                    }
                }
        }

        let new_whitelist = Shared::new(config.process_whitelist.clone());
        let _ = self
            .process_whitelist
            .swap((Some(new_whitelist), Tag::None), Ordering::Release);
        let new_cache = Shared::new(ProcessInfo {
            name: None,
            timestamp: Instant::now(),
        });
        let _ = self
            .cached_process_info
            .swap((Some(new_cache), Tag::None), Ordering::Release);
        self.pressed_keys.clear_sync();
        self.active_combo_triggers.clear_sync();

        self.sequence_matcher.clear_sequences();
        self.sequence_matcher.clear_history();
        for mapping in config.mappings.iter() {
            if mapping.is_sequence_trigger()
                && let Some(seq_str) = &mapping.trigger_sequence
                    && let Ok(sequence) = crate::sequence_matcher::parse_sequence_string(
                        seq_str,
                        Some(mapping.sequence_window_ms),
                    ) {
                        self.sequence_matcher.register_sequence(sequence);
                    }
        }
        if let Some(pool) = self.worker_pool.get() {
            pool.clear_cache();
        }
        self.xinput_cache_invalid.store(true, Ordering::Release);

        Ok(())
    }

    pub fn set_worker_pool(&self, pool: Arc<dyn EventDispatcher>) {
        let _ = self.worker_pool.set(pool);
    }

    pub fn get_worker_pool(&self) -> Option<&Arc<dyn EventDispatcher>> {
        self.worker_pool.get()
    }

    pub fn get_raw_input_capture_sender(&self) -> &Sender<InputDevice> {
        &self.raw_input_capture_sender
    }

    #[inline(always)]
    pub fn check_and_reset_xinput_cache_invalid(&self) -> bool {
        self.xinput_cache_invalid.swap(false, Ordering::Acquire)
    }

    pub fn get_rawinput_capture_mode(&self) -> CaptureMode {
        let guard = Guard::new();
        self.rawinput_capture_mode
            .load(Ordering::Acquire, &guard)
            .as_ref()
            .map(|mode| *mode)
            .unwrap_or_default()
    }

    pub fn get_xinput_capture_mode(&self) -> crate::config::XInputCaptureMode {
        let guard = Guard::new();
        self.xinput_capture_mode
            .load(Ordering::Acquire, &guard)
            .as_ref()
            .map(|mode| *mode)
            .unwrap_or_default()
    }

    pub fn try_recv_raw_input_capture(&self) -> Option<InputDevice> {
        self.raw_input_capture_receiver.try_recv().ok()
    }

    pub fn set_raw_input_capture_mode(&self, enabled: bool) {
        self.is_capturing_raw_input.store(enabled, Ordering::Relaxed);

        if enabled {
            while self.raw_input_capture_receiver.try_recv().is_ok() {}
            crate::rawinput::clear_device_display_info_cache();
            crate::rawinput::reset_hid_device_states();
        }
    }

    /// Clears activation baseline for device.
    #[inline]
    pub fn clear_device_baseline(&self, vid: u16, pid: u16) {
        crate::rawinput::clear_device_baseline(vid, pid);
    }

    /// Checks if Raw Input capture mode is active.
    pub fn is_raw_input_capture_active(&self) -> bool {
        self.is_capturing_raw_input.load(Ordering::Relaxed)
    }

    /// Sends HID device activation request.
    #[inline]
    pub fn request_hid_activation(&self, request: HidActivationRequest) {
        self.activating_device_handle
            .store(request.device_handle, Ordering::Relaxed);
        let _ = self.hid_activation_sender.send(request);
    }

    /// Polls for HID device activation requests
    pub fn poll_hid_activation_requests(&self) -> SmallVec<[HidActivationRequest; 2]> {
        let mut requests = SmallVec::new();
        while let Ok(req) = self.hid_activation_receiver.try_recv() {
            requests.push(req);
        }
        requests
    }

    /// Sends HID activation data during activation process.
    #[inline]
    pub fn send_hid_activation_data(&self, device_handle: isize, data: Vec<u8>) {
        let _ = self.hid_activation_data_sender.send((device_handle, data));
    }

    /// Tries to receive HID activation data for a specific device.
    pub fn try_recv_hid_activation_data(&self, device_handle: isize) -> Option<Vec<u8>> {
        // Consume all messages until we find one for this device
        // or channel is empty
        while let Ok((handle, data)) = self.hid_activation_data_receiver.try_recv() {
            if handle == device_handle {
                return Some(data);
            }
            // Discard data for other devices (shouldn't happen in normal operation)
        }
        None
    }

    /// Checks if a device is currently being activated.
    #[inline(always)]
    pub fn is_device_activating(&self, device_handle: isize) -> bool {
        self.activating_device_handle.load(Ordering::Relaxed) == device_handle
    }

    /// Clears the activating device handle.
    #[inline]
    pub fn clear_activating_device(&self) {
        self.activating_device_handle.store(-1, Ordering::Relaxed);
    }

    /// Sets the notification event sender.
    pub fn set_notification_sender(&self, sender: Sender<NotificationEvent>) {
        let _ = self.notification_sender.set(sender);
    }

    /// Returns the notification sender if available.
    pub fn get_notification_sender(&self) -> Option<&Sender<NotificationEvent>> {
        self.notification_sender.get()
    }

    /// Signals the application to exit.
    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
    }

    /// Checks if the application should exit (hot path - inlined)
    #[inline(always)]
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    /// Toggles pause state and returns the previous state.
    pub fn toggle_paused(&self) -> bool {
        self.is_paused.fetch_xor(true, Ordering::Relaxed)
    }

    /// Returns the current pause state (hot path - inlined)
    #[inline(always)]
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    /// Sets the pause state.
    pub fn set_paused(&self, paused: bool) {
        self.is_paused.store(paused, Ordering::Relaxed);
    }

    /// Returns whether the tray icon should be shown.
    pub fn show_tray_icon(&self) -> bool {
        self.show_tray_icon.load(Ordering::Relaxed)
    }

    /// Returns whether notifications should be displayed.
    pub fn show_notifications(&self) -> bool {
        self.show_notifications.load(Ordering::Relaxed)
    }

    /// Returns the current UI language.
    #[inline(always)]
    pub fn language(&self) -> Language {
        Language::from_u8(self.language.load(Ordering::Relaxed))
    }

    /// Requests the main window to be shown.
    pub fn request_show_window(&self) {
        self.show_window_requested.store(true, Ordering::Relaxed);
    }

    /// Checks and clears the show window request flag.
    pub fn check_and_clear_show_window_request(&self) -> bool {
        self.show_window_requested.swap(false, Ordering::Relaxed)
    }

    /// Requests the about dialog to be shown.
    pub fn request_show_about(&self) {
        self.show_about_requested.store(true, Ordering::Relaxed);
    }

    /// Checks and clears the show about request flag.
    pub fn check_and_clear_show_about_request(&self) -> bool {
        self.show_about_requested.swap(false, Ordering::Relaxed)
    }

    /// Returns the input timeout in milliseconds.
    pub fn input_timeout(&self) -> u64 {
        self.input_timeout.load(Ordering::Relaxed)
    }

    /// Returns the configured worker thread count.
    pub fn get_configured_worker_count(&self) -> usize {
        self.configured_worker_count
    }

    /// Returns the actual number of active worker threads.
    pub fn get_actual_worker_count(&self) -> usize {
        self.worker_count.load(Ordering::Relaxed) as usize
    }

    pub fn set_actual_worker_count(&self, count: usize) {
        self.worker_count.store(count as u64, Ordering::Relaxed);
    }

    /// Fast mapping lookup using lock-free read
    #[inline(always)]
    pub fn get_input_mapping(&self, device: &InputDevice) -> Option<InputMappingInfo> {
        self.input_mappings.read_sync(device, |_, v| v.clone())
    }

    /// Gets all XInputCombo button_ids for a specific device type
    /// Used for subset matching in runtime (cached for performance)
    #[inline(always)]
    pub fn get_xinput_combos_for_device(&self, device_type: &DeviceType) -> Vec<Vec<u32>> {
        self.cached_xinput_combos
            .get_sync(device_type)
            .map(|v| v.get().clone())
            .unwrap_or_default()
    }

    /// Check turbo_enabled state from cache (hot path)
    #[inline(always)]
    pub(crate) fn is_turbo_enabled(&self, device: &InputDevice) -> bool {
        match device {
            InputDevice::Keyboard(vk) if *vk < 256 => {
                self.cached_turbo_keyboard[*vk as usize].load(Ordering::Relaxed)
            }
            _ => self
                .cached_turbo_other
                .read_sync(device, |_, v| *v)
                .unwrap_or(true),
        }
    }

    #[inline]
    pub fn handle_switch_key_toggle(&self) {
        let was_paused = self.toggle_paused();
        self.active_combo_triggers.clear_sync();

        if let Some(sender) = self.notification_sender.get() {
            let msg = if was_paused {
                "Sorahk activating".to_string()
            } else {
                "Sorahk paused".to_string()
            };
            let _ = sender.send(NotificationEvent::Info(msg));
        }
    }

    #[inline]
    pub(crate) fn record_and_match_sequence(
        &self,
        device: InputDevice,
        timestamp: Instant,
    ) -> Option<(InputDevice, Vec<InputDevice>)> {
        self.sequence_matcher.record_input(device, timestamp);
        self.sequence_matcher
            .try_match_with_sequence()
            .map(|(device, arc)| (device, arc.to_vec()))
    }
    #[inline(always)]
    pub(crate) fn is_in_active_combo(&self, vk_code: u32) -> bool {
        self.active_combo_triggers.any_sync(|combo_device, _| {
            matches!(combo_device, InputDevice::KeyCombo(keys) if keys.contains(&vk_code))
        }).is_some()
    }

    #[inline(always)]
    pub(crate) fn is_main_key_in_active_combo_no_turbo(&self, vk_code: u32) -> bool {
        self.active_combo_triggers.any_sync(|combo_device, _| {
            matches!(combo_device, InputDevice::KeyCombo(keys)
                if keys.last() == Some(&vk_code) && !self.is_turbo_enabled(combo_device))
        }).is_some()
    }

    /// Add a combo to active triggers
    pub(crate) fn add_active_combo(&self, combo: InputDevice, modifiers: SmallVec<[u32; 8]>) {
        let _ = self.active_combo_triggers.insert_sync(combo, modifiers);
    }

    /// Check if a specific combo is active
    #[inline(always)]
    pub(crate) fn is_combo_active(&self, combo: &InputDevice) -> bool {
        self.active_combo_triggers.contains_sync(combo)
    }

    #[inline]
    pub(crate) fn cleanup_released_combos(&self) -> SmallVec<[InputDevice; 4]> {
        let mut pressed: SmallVec<[u32; 16]> = SmallVec::new();
        self.pressed_keys.iter_sync(|&k| {
            pressed.push(k);
            true
        });

        let mut removed = SmallVec::new();
        self.active_combo_triggers.retain_sync(|combo_device, _| {
            if let InputDevice::KeyCombo(keys) = combo_device
                && !keys.iter().all(|&k| pressed.contains(&k)) {
                    removed.push(combo_device.clone());
                    return false;
                }
            true
        });

        removed
    }

    #[inline]
    pub(crate) fn find_device_for_release(&self, vk_code: u32) -> Option<InputDevice> {
        // Check active combos first
        let combo_result = self.active_combo_triggers.any_sync(|combo_device, _| {
            if let InputDevice::KeyCombo(keys) = combo_device {
                keys.contains(&vk_code)
            } else {
                false
            }
        });

        if let Some(entry) = combo_result {
            return Some(entry.key().clone());
        }

        // Fallback to keyboard device if it has a mapping
        let device = InputDevice::Keyboard(vk_code);
        self.get_input_mapping(&device).map(|_| device)
    }

    /// Get the process name of the foreground window
    fn get_foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
            if process_id == 0 {
                return None;
            }

            let process_handle =
                match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                    Ok(handle) => handle,
                    Err(_) => return None,
                };

            let mut buffer = [0u16; MAX_PATH as usize];
            let mut size = buffer.len() as u32;

            match QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            ) {
                Ok(_) => {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    // Extract filename from full path
                    path.split('\\').next_back().map(|s| s.to_lowercase())
                }
                Err(_) => None,
            }
        }
    }

    /// Check if current foreground process is in whitelist (empty whitelist = all allowed)
    /// Hot path: called on every input event, inlined for minimal overhead
    #[inline(always)]
    pub(crate) fn is_process_whitelisted(&self) -> bool {
        let guard = Guard::new();
        let whitelist_ptr = self.process_whitelist.load(Ordering::Acquire, &guard);
        let empty_vec = vec![];
        let whitelist = whitelist_ptr.as_ref().unwrap_or(&empty_vec);

        if whitelist.is_empty() {
            return true;
        }

        const CACHE_DURATION_MS: u64 = 50;
        let now = Instant::now();

        let process_name = {
            let cache_ptr = self.cached_process_info.load(Ordering::Acquire, &guard);
            let cache = cache_ptr.as_ref();

            if let Some(info) = cache {
                if likely(
                    now.duration_since(info.timestamp) < Duration::from_millis(CACHE_DURATION_MS),
                ) {
                    // Cache hit: return cached name
                    info.name.clone()
                } else {
                    // Cache miss: need refresh
                    let new_name = Self::get_foreground_process_name();
                    let new_cache = Shared::new(ProcessInfo {
                        name: new_name.clone(),
                        timestamp: now,
                    });
                    let _ = self
                        .cached_process_info
                        .swap((Some(new_cache), Tag::None), Ordering::Release);
                    new_name
                }
            } else {
                // No cache exists
                let new_name = Self::get_foreground_process_name();
                let new_cache = Shared::new(ProcessInfo {
                    name: new_name.clone(),
                    timestamp: now,
                });
                let _ = self
                    .cached_process_info
                    .swap((Some(new_cache), Tag::None), Ordering::Release);
                new_name
            }
        };

        // Check if process is in whitelist
        if let Some(name) = process_name {
            whitelist.iter().any(|p| p.to_lowercase() == name)
        } else {
            // If we can't get process name, allow by default
            true
        }
    }

    pub fn create_input_mappings(
        config: &AppConfig,
    ) -> anyhow::Result<HashMap<InputDevice, InputMappingInfo>> {
        let mut input_mappings = HashMap::new();

        for mapping in &config.mappings {
            // For sequence triggers, use the LAST key in the sequence as the trigger device
            // This aligns with the sequence matcher which returns the last key for triggering
            let trigger_device = if mapping.is_sequence_trigger() {
                // Extract last key from trigger_sequence
                if let Some(seq_str) = &mapping.trigger_sequence {
                    let parts: Vec<&str> = seq_str.split(',').collect();
                    if let Some(last_key) = parts.last() {
                        let last_key_trimmed = last_key.trim();
                        
                        parsing::input_name_to_device(last_key_trimmed).ok_or_else(|| {
                                anyhow::anyhow!("Invalid sequence last key: {}", last_key_trimmed)
                            })?
                    } else {
                        parsing::input_name_to_device(&mapping.trigger_key).ok_or_else(|| {
                            anyhow::anyhow!("Invalid trigger input: {}", mapping.trigger_key)
                        })?
                    }
                } else {
                    parsing::input_name_to_device(&mapping.trigger_key).ok_or_else(|| {
                        anyhow::anyhow!("Invalid trigger input: {}", mapping.trigger_key)
                    })?
                }
            } else {
                // For non-sequence triggers, use trigger_key as usual
                parsing::input_name_to_device(&mapping.trigger_key).ok_or_else(|| {
                    anyhow::anyhow!("Invalid trigger input: {}", mapping.trigger_key)
                })?
            };

            let target_keys = mapping.get_target_keys();
            if target_keys.is_empty() {
                continue; // Skip mappings without target keys
            }

            let interval = mapping.interval.unwrap_or(config.interval).max(5);
            let event_duration = mapping
                .event_duration
                .unwrap_or(config.event_duration)
                .max(2);
            let move_speed = mapping.move_speed.max(1);

            // Parse target keys into output actions
            let mut actions: SmallVec<[OutputAction; 4]> = SmallVec::new();
            for target_key in target_keys {
                if let Some(action) = parsing::input_name_to_output(target_key) {
                    // Update MouseMove and MouseScroll actions with configured speed
                    let action = match action {
                        OutputAction::MouseMove(direction, _) => {
                            OutputAction::MouseMove(direction, move_speed)
                        }
                        OutputAction::MouseScroll(direction, _) => {
                            OutputAction::MouseScroll(direction, move_speed)
                        }
                        other => other,
                    };
                    actions.push(action);
                } else {
                    return Err(anyhow::anyhow!("Invalid target input: {}", target_key));
                }
            }

            if actions.is_empty() {
                continue; // Skip if no valid actions
            }

            // Create the final target action based on target_mode
            let target_action = if actions.len() == 1 {
                actions.into_iter().next().unwrap()
            } else if mapping.target_mode == 2 {
                // Sequence mode: output keys sequentially with interval between each
                OutputAction::SequentialActions(Arc::new(actions), interval)
            } else {
                // Single/Multi mode: output keys simultaneously
                OutputAction::MultipleActions(Arc::new(actions))
            };

            // Create input mapping
            // For sequence triggers, this maps the LAST key to the action,
            // allowing users to hold the last key for continuous repeat
            input_mappings.insert(
                trigger_device.clone(),
                InputMappingInfo {
                    target_action: target_action.clone(),
                    interval,
                    event_duration,
                    turbo_enabled: mapping.turbo_enabled,
                    is_sequence: mapping.is_sequence_trigger(),
                },
            );

            // For sequence triggers, register intermediate keys to enable detection
            // These mappings allow input subsystems to recognize and record sequence inputs
            if mapping.is_sequence_trigger()
                && let Some(seq_str) = &mapping.trigger_sequence {
                    let parts: Vec<&str> = seq_str.split(',').collect();
                    for (idx, part) in parts.iter().enumerate() {
                        if idx == parts.len() - 1 {
                            continue; // Skip last key, already registered above
                        }

                        let part_trimmed = part.trim();
                        if let Some(device) = parsing::input_name_to_device(part_trimmed) {
                            // Register intermediate key with same mapping info
                            // This ensures rawinput/xinput will process it
                            input_mappings.entry(device).or_insert_with(|| InputMappingInfo {
                                        target_action: target_action.clone(),
                                        interval,
                                        event_duration,
                                        turbo_enabled: mapping.turbo_enabled,
                                        is_sequence: true, // Mark as sequence-only
                                    });
                        }
                    }
                }
        }

        Ok(input_mappings)
    }

    /// Parse and cache switch key configuration
    #[inline]
    fn update_switch_key_cache(cache: &SwitchKeyCache, key_name: &str) -> anyhow::Result<()> {
        cache.clear();

        let device = parsing::input_name_to_device(key_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid switch key: {}", key_name))?;

        match &device {
            InputDevice::Keyboard(vk) => {
                cache.keyboard_vk.store(*vk, Ordering::Relaxed);
            }
            InputDevice::XInputCombo {
                device_type,
                button_ids,
            } => {
                let mask = Self::inputs_to_bitset(button_ids);
                cache.xinput_button_mask.store(mask, Ordering::Relaxed);

                let hash = Self::hash_device_type(device_type);
                cache.xinput_device_hash.store(hash, Ordering::Relaxed);
            }
            InputDevice::GenericDevice { button_id, .. } => {
                cache.generic_button_id.store(*button_id, Ordering::Relaxed);
            }
            InputDevice::KeyCombo(_) | InputDevice::Mouse(_) | InputDevice::MouseMove(_) => {}
        }

        let shared_device = Shared::new(device);
        let _ = cache
            .full_device
            .swap((Some(shared_device), Tag::None), Ordering::Release);
        Ok(())
    }

    /// Convert button IDs to bitmask
    #[inline(always)]
    fn inputs_to_bitset(inputs: &[u32]) -> u32 {
        inputs
            .iter()
            .fold(0u32, |acc, &id| if id < 32 { acc | (1 << id) } else { acc })
    }

    /// Compute device type hash for comparison
    #[inline(always)]
    pub fn hash_device_type(device_type: &DeviceType) -> u32 {
        match device_type {
            DeviceType::Gamepad(vid) => (*vid as u32) ^ 0x01000000,
            DeviceType::Joystick(vid) => (*vid as u32) ^ 0x02000000,
            DeviceType::HidDevice { usage_page, usage } => {
                (*usage_page as u32) ^ ((*usage as u32) << 16)
            }
        }
    }

    pub(crate) fn parse_input_name(name: &str) -> Option<InputDevice> {
        parsing::input_name_to_device(name)
    }
}

pub fn set_global_state(state: Arc<AppState>) -> Result<(), Arc<AppState>> {
    GLOBAL_STATE.set(state)
}

pub fn get_global_state() -> Option<&'static Arc<AppState>> {
    GLOBAL_STATE.get()
}
