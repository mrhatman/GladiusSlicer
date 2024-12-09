#![deny(clippy::unwrap_used)]
#![warn(clippy::all, clippy::perf, clippy::missing_const_for_fn)]

pub mod bounds_checking;
pub mod calculation;
pub mod command_pass;
pub mod converter;
pub mod optimizer;
pub mod plotter;
pub mod slice_pass;
pub mod slicing;
pub mod tower;

/// The primary pipeline and functions
pub mod pipeline;
pub mod prelude;
