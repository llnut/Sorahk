// GUI type definitions

/// Key capture mode enumeration for keyboard input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCaptureMode {
    None,
    ToggleKey,
    MappingTrigger(usize), // Index of mapping being edited
    MappingTarget(usize),
    NewMappingTrigger,
    NewMappingTarget,
}
