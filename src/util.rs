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
/// ```ignore
/// if unlikely(error_condition) {
///     handle_error();
/// }
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
/// ```ignore
/// if likely(success_condition) {
///     continue_processing();
/// }
/// ```
#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
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
