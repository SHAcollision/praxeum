use crate::error::PraxeumError;
use crate::model::exercise::Exercise;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported data formats for exercise collections.
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Json,
    Toml,
    /// Infer from file extension.
    Auto,
}

/// Helper for loading exercises from files.
pub struct ExerciseLoader;

#[derive(Debug, Deserialize)]
struct TomlRoot {
    exercise: Vec<Exercise>,
}

impl ExerciseLoader {
    /// Load exercises from a file path.
    ///
    /// JSON format: top-level array of exercises.
    ///
    /// TOML format:
    /// ```toml
    /// [[exercise]]
    /// kind = "classification"
    /// id = "intro_action_event"
    /// # ...
    /// ```
    pub fn load_from_path(
        path: impl AsRef<Path>,
        format: DataFormat,
    ) -> Result<Vec<Exercise>, PraxeumError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let fmt = match format {
            DataFormat::Auto => infer_format(path)?,
            other => other,
        };

        let exercises = match fmt {
            DataFormat::Json => {
                // JSON: either a bare array or a root { "exercises": [...] }
                if contents.trim_start().starts_with('[') {
                    serde_json::from_str::<Vec<Exercise>>(&contents)?
                } else {
                    #[derive(Deserialize)]
                    struct JsonRoot {
                        exercises: Vec<Exercise>,
                    }
                    let root: JsonRoot = serde_json::from_str(&contents)?;
                    root.exercises
                }
            }
            DataFormat::Toml => {
                let root: TomlRoot = toml::from_str(&contents)?;
                root.exercise
            }
            DataFormat::Auto => unreachable!(),
        };

        Ok(exercises)
    }
}

fn infer_format(path: &Path) -> Result<DataFormat, PraxeumError> {
    match path
        .extension()
        .and_then(|os| os.to_str())
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "json" => Ok(DataFormat::Json),
        "toml" | "tml" => Ok(DataFormat::Toml),
        _ => Err(PraxeumError::UnsupportedFormat),
    }
}

/// Utility to resolve a default exercises path (for CLI demos).
pub fn default_examples_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("exercises_basic.toml")
}
