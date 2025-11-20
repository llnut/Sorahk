//! GUI type definitions.

/// Key capture mode for keyboard input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCaptureMode {
    None,
    ToggleKey,
    MappingTrigger(usize),
    MappingTarget(usize),
    NewMappingTrigger,
    NewMappingTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_capture_mode_pattern_matching() {
        // Test extracting index from MappingTrigger variant
        let mode = KeyCaptureMode::MappingTrigger(7);

        match mode {
            KeyCaptureMode::MappingTrigger(idx) => {
                assert_eq!(idx, 7);
            }
            _ => panic!("Pattern matching failed"),
        }

        // Test extracting index from MappingTarget variant
        let mode = KeyCaptureMode::MappingTarget(42);

        match mode {
            KeyCaptureMode::MappingTarget(idx) => {
                assert_eq!(idx, 42);
            }
            _ => panic!("Pattern matching failed"),
        }
    }

    #[test]
    fn test_key_capture_mode_discriminant() {
        // Test that variants with different indices are distinguished
        let trigger_0 = KeyCaptureMode::MappingTrigger(0);
        let trigger_1 = KeyCaptureMode::MappingTrigger(1);
        let target_0 = KeyCaptureMode::MappingTarget(0);

        assert_ne!(
            trigger_0, trigger_1,
            "Different indices should not be equal"
        );
        assert_ne!(
            trigger_0, target_0,
            "Different variants should not be equal"
        );
    }
}
