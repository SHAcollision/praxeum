use crate::error::PraxeumError;
use crate::model::exercise::Exercise;
use crate::validator::validate_exercises_with_source;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported data formats for exercise collections.
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Json,
    Toml,
    /// Infer from file extension or content.
    Auto,
}

/// Helper for loading exercises from files or raw strings.
pub struct ExerciseLoader;

#[derive(Debug, Deserialize)]
struct TomlRoot {
    exercise: Vec<Exercise>,
}

impl ExerciseLoader {
    /// Load and validate exercises from a file path.
    ///
    /// JSON format: top-level array of exercises or `{ "exercises": [...] }`.
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
        let resolved_format = resolve_format(Some(path), &contents, format)?;
        let mut exercises = parse_exercises(&contents, resolved_format)?;
        run_validation(&mut exercises, path.to_str())?;
        Ok(exercises)
    }

    /// Load exercises from an in-memory string, validating them before return.
    pub fn load_from_str(
        contents: &str,
        format: DataFormat,
    ) -> Result<Vec<Exercise>, PraxeumError> {
        let resolved_format = resolve_format(None, contents, format)?;
        let mut exercises = parse_exercises(contents, resolved_format)?;
        run_validation(&mut exercises, None)?;
        Ok(exercises)
    }

    /// Validate exercises from a file path without returning them.
    pub fn validate_path(path: impl AsRef<Path>, format: DataFormat) -> Result<(), PraxeumError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let resolved_format = resolve_format(Some(path), &contents, format)?;
        let mut exercises = parse_exercises(&contents, resolved_format)?;
        run_validation(&mut exercises, path.to_str())
    }

    /// Validate exercises from an in-memory string.
    pub fn validate_str(contents: &str, format: DataFormat) -> Result<(), PraxeumError> {
        let resolved_format = resolve_format(None, contents, format)?;
        let mut exercises = parse_exercises(contents, resolved_format)?;
        run_validation(&mut exercises, None)
    }
}

fn run_validation(exercises: &mut [Exercise], source: Option<&str>) -> Result<(), PraxeumError> {
    validate_exercises_with_source(exercises, source).map_err(PraxeumError::Validation)
}

fn resolve_format(
    path: Option<&Path>,
    contents: &str,
    requested: DataFormat,
) -> Result<DataFormat, PraxeumError> {
    match requested {
        DataFormat::Json | DataFormat::Toml => Ok(requested),
        DataFormat::Auto => {
            if let Some(path) = path {
                if let Ok(fmt) = infer_format_from_path(path) {
                    return Ok(fmt);
                }
            }
            infer_format_from_str(contents)
        }
    }
}

fn parse_exercises(contents: &str, format: DataFormat) -> Result<Vec<Exercise>, PraxeumError> {
    match format {
        DataFormat::Json => parse_json(contents),
        DataFormat::Toml => parse_toml(contents),
        DataFormat::Auto => unreachable!(),
    }
}

fn parse_json(contents: &str) -> Result<Vec<Exercise>, PraxeumError> {
    if contents.trim_start().starts_with('[') {
        serde_json::from_str::<Vec<Exercise>>(contents).map_err(PraxeumError::from)
    } else {
        #[derive(Deserialize)]
        struct JsonRoot {
            exercises: Vec<Exercise>,
        }
        let root: JsonRoot = serde_json::from_str(contents)?;
        Ok(root.exercises)
    }
}

fn parse_toml(contents: &str) -> Result<Vec<Exercise>, PraxeumError> {
    let root: TomlRoot = toml::from_str(contents)?;
    Ok(root.exercise)
}

fn infer_format_from_path(path: &Path) -> Result<DataFormat, PraxeumError> {
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

fn infer_format_from_str(contents: &str) -> Result<DataFormat, PraxeumError> {
    let trimmed = contents.trim_start();
    if trimmed.starts_with('{') {
        return Ok(DataFormat::Json);
    }

    if trimmed.starts_with('[') {
        if trimmed.starts_with("[[exercise]") {
            return Ok(DataFormat::Toml);
        }
        // Heuristic: if we see colons near the top, assume JSON; otherwise TOML array-of-tables.
        let looks_like_json = trimmed.contains(":") && trimmed.contains("\"");
        if looks_like_json {
            return Ok(DataFormat::Json);
        }
        return Ok(DataFormat::Toml);
    }

    Ok(DataFormat::Toml)
}

/// Utility to resolve a default exercises path (for CLI demos).
pub fn default_examples_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("exercises_basic.toml")
}
