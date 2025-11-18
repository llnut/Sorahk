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
