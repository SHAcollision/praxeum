use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PraxeumError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Unsupported data format")]
    UnsupportedFormat,

    #[error("Invalid answer: {0}")]
    InvalidAnswer(String),

    #[error("No exercises loaded")]
    NoExercises,
}
