# Testing Guide

This document describes the test suite for Sorahk.

## Quick Start

```bash
# Run all tests
cargo test

# Run specific module
cargo test config::tests
cargo test state::tests

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
├── state.rs       # State and key mapping logic
├── i18n.rs        # Internationalization
├── keyboard.rs    # Worker pool and event handling
├── tray.rs        # System tray utilities
└── signal.rs      # Signal handling
```

### Integration Tests

Located in `tests/` directory:

```
tests/
├── integration_tests.rs    # Cross-module tests
└── example_test_guide.rs   # Testing patterns reference
```

## Test Coverage

| Module | Primary Focus |
|--------|---------------|
| **config.rs** | Configuration loading, saving, validation, error handling, TOML serialization |
| **state.rs** | Key conversion (VK/scancode), mappings validation, thread safety, atomic operations |
| **i18n.rs** | Multi-language translations, formatting functions, translation completeness |
| **keyboard.rs** | Worker pool creation, event distribution, multi-threading, channel communication |
| **tray.rs** | XML escaping for notifications, utility functions, constants |
| **signal.rs** | Console control event constants, type wrappers |
| **Integration** | Cross-module interactions, configuration persistence, concurrent operations |

Run `cargo test -- --list` to see all available test functions.

## What is Tested

- Configuration management and validation
- Key name to VK code conversion
- VK code to scancode mapping
- Multi-language translation system
- Worker pool and event distribution
- Thread safety and atomic operations
- Error handling and edge cases

## What is Not Tested

Due to Windows API requirements, the following are not covered by automated tests:

- Windows API calls (SetWindowsHookExA, Shell_NotifyIconW, SendInput, etc.)
- Actual keyboard hook installation
- System tray icon display
- Toast notification display
- Physical key press simulation

**Note:** Internal logic of these modules is tested without requiring Windows API interaction.

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
