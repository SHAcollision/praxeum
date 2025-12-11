//! Core engine and CLI for Praxeum exercises.
//!
//! This crate stays UI-agnostic so it can back future CLI, desktop, mobile, and WASM frontends.

pub mod engine;
pub mod error;
pub mod loader;
pub mod model;
pub mod validator;

pub use crate::engine::ExerciseEngine;
pub use crate::error::PraxeumError;
pub use crate::loader::{DataFormat, ExerciseLoader};
pub use crate::model::{
    answer::Answer,
    evaluation::Evaluation,
    exercise::Exercise,
    session::{ProgressSnapshot, SessionMetrics},
};
pub use crate::validator::{validate_exercises, ValidationErrorReport, ValidationIssue};
