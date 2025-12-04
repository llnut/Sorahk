0.4.0
=====
Feature enhancements:

- Add Raw Input API integration for HID device support
  - Support for gamepads, joysticks, and other HID controllers
  - Combo key support: capture multiple buttons pressed simultaneously
  - Automatic device detection and button mapping via GUI capture
  - Multi-device support with vendor ID and serial number identification
  - Device-specific button mapping format (e.g., GAMEPAD_045E_0B05_ABC123_B2.0)
  - Turbo-fire support for HID devices with press/release detection
- Add HID device activation system
  - Interactive activation dialog for establishing device baseline
  - Baseline data persistence across application restarts
  - Press/release event detection based on HID data state changes
  - Always-on-top modal activation window
- Add mouse movement functionality
  - Eight-directional movement (up, down, left, right, diagonals)
  - Configurable speed (1-100 pixels) and interval
  - Target keys: MOUSE_UP, MOUSE_DOWN, MOUSE_LEFT, MOUSE_RIGHT, MOUSE_UP_LEFT, MOUSE_UP_RIGHT, MOUSE_DOWN_LEFT, MOUSE_DOWN_RIGHT
  - Turbo mode support for continuous movement
  - Dedicated worker thread for movement processing
  - Vector-based direction merging for multi-key input
- Add mouse scroll functionality
  - Bidirectional scroll (up and down)
  - Direct wheel delta control (standard Windows units)
  - Target keys: SCROLL_UP, SCROLL_DOWN
  - Turbo mode support for continuous scrolling
  - Non-turbo mode follows Windows repeat events
  - Configurable scroll speed and interval
- Add performance optimizations for input processing pipeline
  - Thread-local buffer pool for Raw Input data
  - Three-tier device information cache (thread-local, global, Windows API)
  - FNV-1a hash algorithm for HID data processing
  - Optional AVX2 SIMD acceleration for data comparison (compile-time feature)
  - Branch prediction hints for hot path optimization
  - Inline optimization for frequently called functions

UI Improvements:

- Add mouse direction selection dialog
- Add mouse scroll direction selection dialog
- Add target type selector buttons in settings (keyboard/mouse, movement, scroll)
- Add HID device activation dialog with theme support
- Add HID device button capture in settings dialog
- Add interval configuration for mouse movement and scroll

Configuration:

- Add `hid_baselines` field for device activation data persistence
- Add `move_speed` field to KeyMapping for movement and scroll speed control
  - Mouse movement: 1-100 pixels per interval
  - Mouse scroll: direct wheel delta value (120 = standard notch)

0.3.0
=====
Feature enhancements:

- Add multi-language support (English, 简体中文, 繁體中文, 日本語)
- Add language selector in settings dialog with real-time preview
- Add key combination support for triggers and targets (e.g., LALT+A, RCTRL+RSHIFT+S)
- Add mouse button support (Left, Right, Middle, X1, X2)
- Add per-mapping turbo mode toggle with Windows native repeat support
- Add lock-free concurrency with scc containers for improved performance
- Add multi-layer caching for process whitelist, turbo state, and mapping info
- Add combo key reverse index for O(1) lookup optimization
- Add enhanced key capture with comprehensive keyboard support
  - Support for F1-F24, numpad, lock keys, system keys, and OEM punctuation
  - Left/right modifier distinction (LCTRL/RCTRL, LALT/RALT, LSHIFT/RSHIFT)
  - Initial state filtering to prevent false positives
- Add input validation for duplicate trigger keys and process names
- Add comprehensive test suite with TESTING.md guide
- Add `turbo_enabled` configuration field (defaults to true)
- Update timing parameter minimums (input_timeout: 2ms, event_duration: 2ms)

UI Improvements:

- Add turbo toggle button with visual state indication (⚡/○)
- Add localized hover tooltips for turbo toggle in all languages
- Add turbo status display in main window mappings table
- Increase settings window width to 720px for better layout

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
