//! Core engine and CLI for Praxeum exercises.
//!
//! This crate stays UI-agnostic so it can back future CLI, desktop, mobile, and WASM frontends.

pub mod engine;
pub mod error;
pub mod loader;
pub mod model;

pub use crate::engine::ExerciseEngine;
pub use crate::loader::{DataFormat, ExerciseLoader};
pub use crate::model::{answer::Answer, evaluation::Evaluation, exercise::Exercise};
