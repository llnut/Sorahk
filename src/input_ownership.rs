//! Device ownership management for input priority system.
//!
//! Manages device ownership across XInput and Raw Input to prevent
//! duplicate event processing while enforcing API priority.

use crate::config::DeviceApiPreference;
use std::sync::Arc;

/// Input source type for device ownership tracking.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSource {
    /// XInput device (Xbox controllers).
    XInput(u32),
    /// Raw Input device.
    RawInput(isize),
}

impl InputSource {
    /// Returns priority level (lower value = higher priority).
    #[inline(always)]
    pub fn priority(&self) -> u8 {
        match self {
            InputSource::XInput(_) => 0,
            InputSource::RawInput(_) => 1,
        }
    }
}

/// Device ownership manager.
pub struct DeviceOwnership {
    /// Maps (VID, PID) to owning input source.
    owners: Arc<scc::HashMap<(u16, u16), InputSource>>,
    /// User-configured API preferences for devices.
    preferences: Arc<scc::HashMap<(u16, u16), DeviceApiPreference>>,
}

impl DeviceOwnership {
    /// Creates a new device ownership manager.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            owners: Arc::new(scc::HashMap::new()),
            preferences: Arc::new(scc::HashMap::new()),
        }
    }

    /// Sets API preference for a device.
    #[inline(always)]
    pub fn set_preference(&self, vid_pid: (u16, u16), preference: DeviceApiPreference) {
        let _ = self.preferences.insert_sync(vid_pid, preference);
    }

    /// Checks if source matches device preference.
    #[inline(always)]
    fn matches_preference(&self, vid_pid: (u16, u16), source: &InputSource) -> bool {
        // Fast path: no preference cached means allow any
        let pref = match self.preferences.read_sync(&vid_pid, |_, v| *v) {
            Some(p) => p,
            None => return true,
        };

        // Use bitwise operations for fast matching
        // Auto = 0b11 (both bits set), XInput = 0b01, RawInput = 0b10
        let source_bit = match source {
            InputSource::XInput(_) => 0b01u8,
            InputSource::RawInput(_) => 0b10u8,
        };

        let pref_mask = match pref {
            DeviceApiPreference::Auto => 0b11u8,
            DeviceApiPreference::XInput => 0b01u8,
            DeviceApiPreference::RawInput => 0b10u8,
        };

        (source_bit & pref_mask) != 0
    }

    /// Claims device ownership. Returns true if claimed, false if owned by higher priority source or doesn't match preference.
    #[inline(always)]
    pub fn claim_device(&self, vid_pid: (u16, u16), source: InputSource) -> bool {
        // Check if source matches user preference
        if crate::util::unlikely(!self.matches_preference(vid_pid, &source)) {
            return false;
        }

        // Check existing ownership
        if let Some(existing) = self.owners.read_sync(&vid_pid, |_, v| v.clone())
            && crate::util::unlikely(existing.priority() <= source.priority())
        {
            return false;
        }

        let _ = self.owners.insert_sync(vid_pid, source);
        true
    }

    /// Releases ownership of a device.
    #[inline(always)]
    pub fn release_device(&self, vid_pid: (u16, u16)) {
        self.owners.remove_sync(&vid_pid);
    }

    /// Checks if device is claimed by higher priority source.
    #[inline(always)]
    pub fn is_claimed_by_higher_priority(&self, vid_pid: (u16, u16), source: &InputSource) -> bool {
        match self.owners.read_sync(&vid_pid, |_, v| v.priority()) {
            Some(owner_priority) => owner_priority < source.priority(),
            None => false,
        }
    }

    /// Gets the current owner of a device.
    #[inline(always)]
    pub fn get_owner(&self, vid_pid: (u16, u16)) -> Option<InputSource> {
        self.owners.read_sync(&vid_pid, |_, v| v.clone())
    }

    /// Clears all ownership records.
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.owners.clear_sync();
    }
}

impl Default for DeviceOwnership {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DeviceOwnership {
    fn clone(&self) -> Self {
        Self {
            owners: Arc::clone(&self.owners),
            preferences: Arc::clone(&self.preferences),
        }
    }
}
