# Testing Guide

This document describes the test suite for Sorahk.

## Quick Start

```bash
# Run all tests
cargo test

# Run specific module
cargo test config::tests
cargo test state::tests

# Run combo key related tests
cargo test combo

# Run key conversion tests
cargo test key_name_to_vk
cargo test vk_to_scancode

# Run with output
cargo test -- --nocapture

# Run in release mode
cargo test --release
```

Windows batch file:
```batch
run_tests.bat
```

## Test Structure

### Unit Tests

Located within each module using `#[cfg(test)]`:

```
src/
├── config.rs      # Configuration management
├── state.rs       # State and input mapping logic
├── i18n.rs        # Internationalization
├── keyboard.rs    # Worker pool and event handling
├── mouse.rs       # Mouse input handling
├── rawinput.rs    # HID device input processing
├── xinput.rs      # XInput controller handling
├── tray.rs        # System tray utilities
├── signal.rs      # Signal handling
└── gui/
    ├── utils.rs   # GUI utility functions
    └── types.rs   # GUI type definitions
```

### Integration Tests

Located in `tests/` directory:

```
tests/
├── integration_tests.rs    # Cross-module tests
├── state_tests.rs          # State management tests
└── example_test_guide.rs   # Testing patterns reference
```

## Test Coverage

| Module | Primary Focus |
|--------|---------------|
| **config.rs** | Configuration loading, saving, validation, error handling, TOML serialization, multiple target keys management (add, remove, clear, set operations), sequence trigger/target fields, target mode handling |
| **state.rs** | Key conversion (VK/scancode for all key types: standard, numpad, system, lock, OEM, mouse), input device mappings (keyboard, mouse, HID devices), combo key parsing (including numpad and OEM keys), device type parsing (gamepad, joystick), mouse scroll direction parsing, output action handling (keyboard, mouse buttons, mouse movement, mouse scroll, sequential actions), state management, thread safety, atomic operations, lock-free concurrent data structures, batch INPUT event processing, extended scancode bitmap detection |
| **sequence_matcher.rs** | Ring buffer implementation, input sequence recording, pattern matching algorithm, time window validation, input deduplication, device matching logic, transition tolerance, diagonal bidirectional matching, cache-aligned structures, atomic operations, lock-free sequence registration |
| **i18n.rs** | Multi-language translations, formatting functions, translation completeness |
| **keyboard.rs** | Worker pool creation, worker distribution stability, mapping cache retrieval, sequential action handling, turbo mode processing |
| **mouse.rs** | Mouse button handling, message parsing, event processing, mouse movement turbo |
| **rawinput.rs** | FNV-1a hash algorithm, device ID generation, VID/PID parsing, button counting, HID baseline management, combo key capture logic, sequence input recording |
| **xinput.rs** | VID/PID hash generation, button state detection, analog stick direction mapping, trigger state detection, input combination hashing, deadzone filtering, combo mask building, bitset matching, layered index matching, AVX2 SIMD batch matching (compile-time), extended scancode detection, sequence input recording, diagonal combo matching |
| **gui/utils.rs** | Key string conversion, icon loading |
| **gui/types.rs** | KeyCaptureMode enum |
| **gui/settings_dialog.rs** | Sequence capture mode, continuous key input, target mode selection, sequence window editing |
| **tray.rs** | XML escaping for notifications, utility functions, constants |
| **signal.rs** | Console control event constants, type wrappers |
| **Integration** | Cross-module interactions, configuration persistence, concurrent operations, sequence trigger/target workflow |

Run `cargo test -- --list` to see all available test functions.

## What is Tested

- Configuration management and validation
- Multiple target keys functionality (add, remove, clear, display)
- Sequence trigger and target fields persistence
- Target mode serialization and deserialization
- Key name to VK code conversion
- VK code to scancode mapping
- Combo key parsing and validation
- Mouse button name parsing and event handling
- Mouse movement target validation and direction parsing
- Multiple simultaneous actions and sequential actions
- Sequence matcher ring buffer operations
- Input sequence recording and pattern matching
- Time window validation for sequence completion
- Input deduplication logic
- Device matching and transition tolerance
- Diagonal bidirectional matching for XInput
- HID device input parsing and device type identification
- Device ID generation and vendor/product ID handling
- FNV-1a hash algorithm correctness
- Generic device button ID format parsing
- HID device baseline establishment and persistence
- HID combo key capture logic (frame selection, button counting)
- Device ID parsing and validation (VID/PID/Serial format)
- XInput combo mask building and bitset matching
- GUI utility functions and type definitions
- Multi-language translation system
- Worker pool and event distribution
- Device-based load balancing and hashing
- Thread safety and atomic operations
- Lock-free sequence registration and access
- Cache-aligned data structure layout
- Error handling and edge cases

## What is Not Tested

Due to Windows API requirements, the following are not covered by automated tests:

- Windows API calls (SetWindowsHookExA, Shell_NotifyIconW, SendInput, XInputGetState, XInputGetCapabilities, etc.)
- Actual keyboard and mouse hook installation
- System tray icon display
- Toast notification display
- Physical key press and mouse click simulation
- GUI rendering and user interactions
- Combo key event handling with real keyboard input
- Sequence capture with real hardware input
- Real-time sequence matching with physical devices
- HID device activation dialog interactions
- Raw Input API device enumeration and data reception
- XInput API device polling and state retrieval
- Physical Xbox controller connection and disconnection
- Sequential action dispatch timing accuracy
- Turbo mode behavior with actual input events

**Note:** Internal logic is tested without requiring Windows API interaction. Tests verify key conversion, combo parsing, sequence matching algorithms, data structure operations, button state detection, and hash generation, but avoid triggering actual input or API calls to prevent system interference.

## Writing New Tests

### Example Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange: Set up test data
        let config = create_test_config();

        // Act: Perform operation
        let result = process_config(config);

        // Assert: Verify result
        assert!(result.is_ok());
    }
}
```

### Conventions

- **Naming**: Use descriptive names that clearly indicate what is being tested (e.g., `test_config_save_and_load`, `test_key_name_to_vk_letters`)
- **Isolation**: Use unique temporary files with timestamps, always clean up resources
- **Focus**: Test one specific behavior per function
- **Helpers**: Extract common setup code into helper functions

### Example Test Helpers

```rust
fn get_test_file_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("test_{}_{}.toml", name, timestamp()));
    path
}

fn cleanup_test_file(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}
```

## Platform Requirements

Tests require a Windows environment due to platform-specific APIs used in the project.

Potential issues:
- File access errors may occur if antivirus software interferes with temporary files
- Dependencies must be built before running tests (`cargo build`)
- Initial test runs may take longer due to compilation

## Before Committing

Run tests and lints to verify changes:

```bash
cargo test && cargo clippy
```

## Additional Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Testing Patterns Reference](tests/example_test_guide.rs) - Code examples and test patterns
