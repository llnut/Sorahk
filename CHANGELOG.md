0.3.0
=====
Feature enhancements:

* Mouse button auto-fire support (Left, Right, Middle, X1, X2)
* Multi-language support (English, 简体中文, 繁體中文, 日本語)
* Language selector in settings dialog with real-time preview
* Duplicate trigger key validation to prevent conflicts
* Duplicate process validation in whitelist

Performance optimizations:
- Convert switch_key from RwLock to AtomicU32 for lock-free access
- Add process whitelist cache (50ms expiration) to reduce Windows API calls
- Add mapping info cache in worker threads to avoid repeated lock reads

UI Improvements:
- Add mouse button capture in settings dialog
- Update UI translations for better clarity

Testing:
- Add comprehensive test suite covering unit and integration tests
- Add unit tests for config, state, i18n, keyboard, mouse, tray, and signal modules
- Add TESTING.md with testing guide and example patterns
- Add lib target to Cargo.toml to enable module testing

Documentation:
- Update README.md with mouse support and testing sections
- Update Config.toml with mouse button mapping examples

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
