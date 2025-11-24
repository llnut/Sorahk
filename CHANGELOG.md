0.3.0
=====
Feature enhancements:

* **Multi-language Support** - UI available in English, 简体中文, 繁體中文, 日本語
* **Language Selector** - Language selection in settings dialog with real-time preview
* **Key Combination Support** - Support for combo key triggers and targets (e.g., LALT+A, RCTRL+RSHIFT+S)
* **Enhanced Key Capture** - Improved key capture with comprehensive keyboard support
  - Supports all standard keys including function keys (F1-F24), numpad keys, lock keys, system keys, and OEM punctuation
  - Left/right modifier distinction (LCTRL/RCTRL, LALT/RALT, LSHIFT/RSHIFT)
  - Initial state filtering to prevent false positives from pre-existing key states
  - Combo key capture with proper formatting (e.g., LCTRL+RSHIFT+A)
* **Mouse Button Support** - Auto-fire for mouse buttons (Left, Right, Middle, X1, X2)
* **Input Validation** - Prevents duplicate trigger keys and duplicate process names
* **Turbo Mode Toggle** - Per-mapping control for auto-repeat behavior
  - Toggle button in settings dialog for each mapping
  - True: rapid-fire with configurable interval
  - False: single press with Windows native key repeat support

Implementation details:
- **Combo key handling** - Multiple combos can operate concurrently with independent state management
- **Modifier suppression** - Modifiers suppressed when outputting single keys from combo triggers (e.g., LALT+1 outputs "1")
- **Repeat event handling** - Allows repeat events for non-turbo mappings to preserve native Windows behavior

Performance optimizations:
- **Lock-free concurrency** - Replaced synchronization primitives with `scc` lock-free containers for mapping info and switch key access
- **Multi-layer caching** - Process whitelist (50ms expiration), turbo state (dual-layer), and mapping info caches in worker threads
- **Combo key reverse index** - O(1) main key lookup with HashMap-based reverse index
- **Early exit** - Skip processing when paused or not in whitelisted process

Configuration:
- Support "+" separator in key names for combos (e.g., "LALT+A")
- `turbo_enabled` field for per-mapping turbo control (defaults to true)
- Timing parameter minimums: `input_timeout` 2ms, `event_duration` 2ms

UI Improvements:
- Updated UI translations
- Turbo toggle button with visual state indication (⚡ for ON, ○ for OFF)
- Localized hover tooltips for turbo toggle button in all supported languages
- Increased settings window width to 720px for improved layout

Testing:
- Comprehensive unit and integration test suite for all modules
- Tests for combo key parsing, modifier scancodes, and mapping creation
- TESTING.md with testing guide and examples

Documentation:
- Updated README.md and Config.toml with turbo mode, combo keys, and mouse support examples

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
