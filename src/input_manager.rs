//! Input manager coordinating XInput and Raw Input.
//!
//! Manages device ownership with priority: XInput for Xbox controllers,
//! Raw Input for other HID devices.

use crate::config::HidDeviceBaseline;
use crate::input_ownership::DeviceOwnership;
use crate::rawinput::RawInputHandler;
use crate::state::AppState;
use crate::xinput::XInputHandler;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

/// Global device ownership manager.
static DEVICE_OWNERSHIP: OnceLock<DeviceOwnership> = OnceLock::new();

/// Input manager for all input sources.
pub struct InputManager {
    ownership: DeviceOwnership,
}

impl InputManager {
    /// Creates a new input manager and initializes all input handlers.
    pub fn new(
        state: Arc<AppState>,
        hid_baselines: Vec<HidDeviceBaseline>,
        api_preferences: std::collections::HashMap<String, crate::config::DeviceApiPreference>,
    ) -> anyhow::Result<Self> {
        let ownership = DeviceOwnership::new();

        // Load API preferences
        for (key, pref) in api_preferences {
            if let Some((vid, pid)) = key.split_once(':')
                && let (Ok(vid_u16), Ok(pid_u16)) =
                    (u16::from_str_radix(vid, 16), u16::from_str_radix(pid, 16))
            {
                ownership.set_preference((vid_u16, pid_u16), pref);
            }
        }

        // Initialize global ownership reference
        let _ = DEVICE_OWNERSHIP.set(ownership.clone());

        // Start XInput handler
        let xinput_ownership = ownership.clone();
        let xinput_state = state.clone();
        let _ = thread::Builder::new()
            .name("xinput_thread".to_string())
            .spawn(move || {
                let mut handler = XInputHandler::new(xinput_state, xinput_ownership);
                handler.initialize();

                const POLL_INTERVAL: Duration = Duration::from_millis(1);
                loop {
                    handler.poll();
                    thread::sleep(POLL_INTERVAL);
                }
            })
            .map_err(|e| anyhow::anyhow!("Failed to start XInput thread: {}", e))?;

        // Start Raw Input handler
        let _rawinput_thread =
            RawInputHandler::start_thread(state, hid_baselines, ownership.clone());

        Ok(Self { ownership })
    }

    /// Gets reference to device ownership manager.
    #[allow(dead_code)]
    pub fn ownership(&self) -> &DeviceOwnership {
        &self.ownership
    }
}

/// Sets API preference for a device.
#[inline]
pub fn set_device_api_preference(
    vid_pid: (u16, u16),
    preference: crate::config::DeviceApiPreference,
) {
    if let Some(ownership) = DEVICE_OWNERSHIP.get() {
        ownership.set_preference(vid_pid, preference);
    }
}

/// Releases ownership of a device, allowing it to be claimed by a different API.
#[inline]
pub fn release_device_ownership(vid_pid: (u16, u16)) {
    if let Some(ownership) = DEVICE_OWNERSHIP.get() {
        ownership.release_device(vid_pid);
    }
}
