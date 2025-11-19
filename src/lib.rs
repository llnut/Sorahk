//! Core modules for the Sorahk auto key press application.
//!
//! This library exposes internal modules for testing purposes.
//! It is not intended for external use as a library.

pub mod config;
pub mod i18n;

// Re-export types for test modules
pub use config::{AppConfig, KeyMapping};
pub use i18n::{CachedTranslations, Language};
