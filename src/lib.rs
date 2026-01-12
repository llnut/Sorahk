//! Core modules for the Sorahk auto key press application.
//!
//! This library exposes internal modules for testing purposes.
//! It is not intended for external use as a library.

pub mod config;
pub mod gui;
pub mod i18n;
pub mod input_manager;
pub mod input_ownership;
pub mod rawinput;
pub mod sequence_matcher;
pub mod state;
pub mod util;
pub mod xinput;

// Re-export types for test modules
pub use config::{AppConfig, KeyMapping};
pub use i18n::{CachedTranslations, Language};
