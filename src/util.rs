//! Common utility functions.
//!
//! Provides branch prediction hints and hash functions used across modules.

/// Marker function for cold code paths.
///
/// Used with branch prediction hints to inform the compiler about infrequently executed paths.
#[inline(always)]
#[cold]
pub fn cold() {}

/// Branch prediction hint for conditions expected to be false.
///
/// Helps the compiler optimize for the more common case where the condition is false.
///
/// # Example
/// ```
/// use sorahk::util::unlikely;
///
/// fn process_data(value: i32) -> Result<i32, &'static str> {
///     if unlikely(value < 0) {
///         return Err("negative value");
///     }
///     Ok(value * 2)
/// }
///
/// assert_eq!(process_data(5).unwrap(), 10);
/// assert!(process_data(-1).is_err());
/// ```
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

/// Branch prediction hint for conditions expected to be true.
///
/// Helps the compiler optimize for the more common case where the condition is true.
///
/// # Example
/// ```
/// use sorahk::util::likely;
///
/// fn validate_input(value: i32) -> Option<i32> {
///     if likely(value >= 0 && value <= 100) {
///         return Some(value);
///     }
///     None
/// }
///
/// assert_eq!(validate_input(50), Some(50));
/// assert_eq!(validate_input(150), None);
/// ```
#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

/// Numpad key remapping helpers.
///
/// Windows reports numpad digits as nav-cluster virtual keys such as
/// VK_INSERT or VK_END when NumLock is off. The capture dialog only sees
/// VKs via `GetAsyncKeyState`, so it needs both directions to keep a
/// numpad finalize key working regardless of NumLock state.
pub mod numpad {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_CLEAR, VK_DECIMAL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LEFT,
        VK_NEXT, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5,
        VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_PRIOR, VK_RIGHT, VK_UP,
    };

    #[inline(always)]
    const fn vk(key: VIRTUAL_KEY) -> u32 {
        key.0 as u32
    }

    /// Remaps nav-cluster VKs back to VK_NUMPAD* for callers that only see
    /// NumLock-translated virtual keys. Call this only when NumLock is off;
    /// the caller is responsible for checking `GetKeyState(VK_NUMLOCK) & 1`.
    #[inline(always)]
    pub fn from_nav_vk(key: u32) -> u32 {
        match key {
            k if k == vk(VK_INSERT) => vk(VK_NUMPAD0),
            k if k == vk(VK_END) => vk(VK_NUMPAD1),
            k if k == vk(VK_DOWN) => vk(VK_NUMPAD2),
            k if k == vk(VK_NEXT) => vk(VK_NUMPAD3),
            k if k == vk(VK_LEFT) => vk(VK_NUMPAD4),
            k if k == vk(VK_CLEAR) => vk(VK_NUMPAD5),
            k if k == vk(VK_RIGHT) => vk(VK_NUMPAD6),
            k if k == vk(VK_HOME) => vk(VK_NUMPAD7),
            k if k == vk(VK_UP) => vk(VK_NUMPAD8),
            k if k == vk(VK_PRIOR) => vk(VK_NUMPAD9),
            k if k == vk(VK_DELETE) => vk(VK_DECIMAL),
            other => other,
        }
    }

    /// Inverse of [`from_nav_vk`]. Returns the nav-cluster VK that Windows
    /// reports for a numpad VK when NumLock is off, or `None` for non-numpad
    /// inputs. Needed by callers that query a specific numpad key state via
    /// `GetAsyncKeyState`.
    #[inline(always)]
    pub fn to_nav_vk(key: u32) -> Option<u32> {
        match key {
            k if k == vk(VK_NUMPAD0) => Some(vk(VK_INSERT)),
            k if k == vk(VK_NUMPAD1) => Some(vk(VK_END)),
            k if k == vk(VK_NUMPAD2) => Some(vk(VK_DOWN)),
            k if k == vk(VK_NUMPAD3) => Some(vk(VK_NEXT)),
            k if k == vk(VK_NUMPAD4) => Some(vk(VK_LEFT)),
            k if k == vk(VK_NUMPAD5) => Some(vk(VK_CLEAR)),
            k if k == vk(VK_NUMPAD6) => Some(vk(VK_RIGHT)),
            k if k == vk(VK_NUMPAD7) => Some(vk(VK_HOME)),
            k if k == vk(VK_NUMPAD8) => Some(vk(VK_UP)),
            k if k == vk(VK_NUMPAD9) => Some(vk(VK_PRIOR)),
            k if k == vk(VK_DECIMAL) => Some(vk(VK_DELETE)),
            _ => None,
        }
    }
}

/// FNV-1a 32-bit hash constants.
pub mod fnv32 {
    /// Offset basis for FNV-1a 32-bit hash.
    pub const OFFSET_BASIS: u32 = 0x811c9dc5;
    /// Prime multiplier for FNV-1a 32-bit hash.
    pub const PRIME: u32 = 0x01000193;
}

/// FNV-1a 64-bit hash constants.
pub mod fnv64 {
    /// Offset basis for FNV-1a 64-bit hash.
    pub const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    /// Prime multiplier for FNV-1a 64-bit hash.
    pub const PRIME: u64 = 0x100000001b3;
}

/// Computes FNV-1a 32-bit hash.
///
/// Non-cryptographic hash function suitable for hash tables and checksums.
///
/// # Arguments
/// * `hash` - Current hash state
/// * `value` - Value to incorporate into hash
///
/// # Returns
/// Updated hash state
#[inline(always)]
pub fn fnv1a_hash_u32(mut hash: u32, value: u32) -> u32 {
    hash ^= value;
    hash.wrapping_mul(fnv32::PRIME)
}

/// Computes FNV-1a 64-bit hash.
///
/// Non-cryptographic hash function suitable for hash tables and checksums.
///
/// # Arguments
/// * `hash` - Current hash state
/// * `value` - Value to incorporate into hash
///
/// # Returns
/// Updated hash state
#[inline(always)]
pub fn fnv1a_hash_u64(mut hash: u64, value: u64) -> u64 {
    hash ^= value;
    hash.wrapping_mul(fnv64::PRIME)
}

/// Computes FNV-1a 64-bit hash for a byte sequence.
///
/// Processes each byte in the input using the FNV-1a algorithm.
///
/// # Arguments
/// * `hash` - Initial hash state
/// * `bytes` - Input bytes to hash
///
/// # Returns
/// Final hash state
#[inline(always)]
pub fn fnv1a_hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        hash = fnv1a_hash_u64(hash, byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely_unlikely() {
        assert!(likely(true));
        assert!(!likely(false));
        assert!(unlikely(true));
        assert!(!unlikely(false));
    }

    #[test]
    fn test_fnv1a_hash_u32() {
        let hash = fnv32::OFFSET_BASIS;
        let result = fnv1a_hash_u32(hash, 42);
        assert_ne!(result, hash);

        // Verify determinism
        let hash1 = fnv1a_hash_u32(fnv32::OFFSET_BASIS, 42);
        let hash2 = fnv1a_hash_u32(fnv32::OFFSET_BASIS, 42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv1a_hash_u64() {
        let hash = fnv64::OFFSET_BASIS;
        let result = fnv1a_hash_u64(hash, 42);
        assert_ne!(result, hash);

        // Verify determinism
        let hash1 = fnv1a_hash_u64(fnv64::OFFSET_BASIS, 42);
        let hash2 = fnv1a_hash_u64(fnv64::OFFSET_BASIS, 42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv1a_hash_bytes() {
        let hash = fnv64::OFFSET_BASIS;
        let data = b"test data";
        let result = fnv1a_hash_bytes(hash, data);
        assert_ne!(result, hash);

        // Verify determinism
        let hash1 = fnv1a_hash_bytes(fnv64::OFFSET_BASIS, data);
        let hash2 = fnv1a_hash_bytes(fnv64::OFFSET_BASIS, data);
        assert_eq!(hash1, hash2);

        // Verify different inputs produce different outputs
        let hash3 = fnv1a_hash_bytes(fnv64::OFFSET_BASIS, b"other data");
        assert_ne!(hash1, hash3);
    }
}
