0.3.0
=====
Feature enhancements:

* **Multi-language Support** - UI available in English, 简体中文, 繁體中文, 日本語
* **Language Selector** - Language selection in settings dialog with real-time preview
* **Key Combination Support** - Support for combo key triggers and targets (e.g., ALT+A, CTRL+SHIFT+S)
* **GUI Combo Key Capture** - Combo key capture in Settings dialog
  - Left/right modifier detection using Windows API (LCTRL/RCTRL, LALT/RALT, LSHIFT/RSHIFT)
  - Capture completes on first key release
  - Supports standalone modifiers (e.g., LSHIFT) and complex combos (e.g., LSHIFT+1)
  - Tracks pressed keys during capture for combo detection
  - Supports letters, numbers, function keys, and special keys
  - Formats combos with specific modifiers (e.g., LCTRL+RSHIFT+A)
* **Mouse Button Support** - Auto-fire for mouse buttons (Left, Right, Middle, X1, X2)
* **Input Validation** - Prevents duplicate trigger keys and duplicate process names

Implementation details:
- **Multiple simultaneous combos** - Multiple combos with shared modifiers can operate concurrently
  - Example: ALT+1, ALT+2, ALT+3 all active at the same time
  - Each combo operates independently
  - Modifier suppression: shared modifiers released once per activation
- **Modifier key handling** - Proper modifier state management for combo keys
  - Modifiers suppressed when outputting single keys from combo triggers
  - Keyboard repeat events blocked for active combos
  - Example: Alt+1 outputs "1" without Alt modifier
- **Raw VK code matching** - Direct VK code comparison without normalization
  - Example: LSHIFT+1 responds only to left Shift
  - Supports both specific (LSHIFT) and generic (SHIFT) modifier names
- **State management** - Proper state cleanup on pause/resume and config reload

Performance optimizations:
- **Lock-free data structures** - Replaced synchronization primitives with `scc` lock-free containers
- **Process whitelist cache** - Cache with 50ms expiration to reduce Windows API calls
- **Mapping info cache** - Worker threads cache mapping info to reduce lock reads
- **Early exit optimization** - Skip processing when paused or not in whitelisted process
- **Lock-free switch key access** - Convert switch_key from RwLock to AtomicU32

Configuration:
- Support "+" separator in key names for combos (e.g., "ALT+A")
- Backward compatible with existing single-key configurations
- Configuration examples updated with combo key and mouse button mappings

UI Improvements:
- Updated UI translations

Testing:
- Comprehensive test suite covering unit and integration tests
- Unit tests for config, state, i18n, keyboard, mouse, tray, and signal modules
- Tests for combo key parsing, modifier key scancodes, and mapping creation
- Arc reference counting validation tests
- TESTING.md with testing guide and example patterns
- Added lib target to Cargo.toml for module testing

Documentation:
- README.md updated with combo key examples and mouse support
- Config.toml updated with combo key and mouse button mapping examples

0.2.0
=====
Feature enhancements:

* Add GUI with anime-style design
* Add interactive settings dialog with real-time configuration editing
* Add configurable light/dark theme support with persistent storage
* Add multi-threaded worker pool with load-balanced event dispatching
* Add process whitelist for application-specific turbo-fire control
* Add Windows Toast notification system with fallback support
* Add About dialog with project information
* Add embedded application icon

UI Improvements:
- Replace tray icon display to use custom icon

0.1.1
=====
Feature enhancements:

* Tray icon support
