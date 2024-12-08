#![deny(clippy::unwrap_used)]
#![warn(clippy::all, clippy::perf, clippy::missing_const_for_fn)]
#![deny(missing_docs)]
//!Crate for shared types between slicer and external applications like GUI and Mods

/// Error types
pub mod error;

/// Load in model files
pub mod loader;

/// Settings types
pub mod settings;

/// Common shared types
pub mod types;

/// Messages for IPC
pub mod messages;

/// Warning Types
pub mod warning;

/// Utilities Functions
pub mod utils;

/// Handles input
pub mod input;

/// the standard imports for the shared crate
pub mod prelude;

///Re-Export Geo to make sure versions are consistent
pub use geo;
