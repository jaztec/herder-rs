//! Play-state implementation.
//!
//! The play state owns the active run: world setup, actors, sheep AI, score,
//! pause, finish handling, and off-screen indicators.

mod common;
mod dog;
mod finish;
mod herd;
mod indicators;
mod plugin;
mod score;
mod sheep;
mod shepherd;

pub use plugin::*;
