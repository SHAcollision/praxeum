use crate::model::exercise::{ClassificationItem, Exercise, ScenarioChoice};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

/// A single validation issue discovered in exercise data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Identifier of the exercise, if known.
    pub exercise_id: Option<String>,
    /// Field or aspect that failed validation.
    pub field: String,
    /// Human-friendly description of the issue.
    pub message: String,
}

impl ValidationIssue {
    pub fn new(
        exercise_id: Option<&str>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        ValidationIssue {
            exercise_id: exercise_id.map(ToOwned::to_owned),
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.exercise_id {
            write!(f, "[{id}] {}: {}", self.field, self.message)
        } else {
            write!(f, "{}: {}", self.field, self.message)
        }
    }
}

/// Collection of validation issues for a given source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrorReport {
    pub source: Option<String>,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationErrorReport {
    pub fn new(source: Option<String>, issues: Vec<ValidationIssue>) -> Self {
        ValidationErrorReport { source, issues }
    }
}

impl fmt::Display for ValidationErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(src) = &self.source {
            writeln!(f, "Validation failed for {src}:")?;
        } else {
            writeln!(f, "Validation failed:")?;
        }

        for issue in &self.issues {
            writeln!(f, "- {issue}")?;
        }

        Ok(())
    }
}

impl Error for ValidationErrorReport {}

/// Validate a list of exercises, returning structured issues.
pub fn validate_exercises(exercises: &[Exercise]) -> Result<(), ValidationErrorReport> {
    validate_exercises_with_source(exercises, None)
}

/// Validate a list of exercises with a given source label (e.g., filename).
pub fn validate_exercises_with_source(
    exercises: &[Exercise],
    source: Option<&str>,
) -> Result<(), ValidationErrorReport> {
    let mut issues = Vec::new();
    let mut seen_ids = HashSet::new();

    for exercise in exercises {
        let id = exercise.id().to_owned();
        if !seen_ids.insert(id.clone()) {
            issues.push(ValidationIssue::new(
                Some(&id),
                "id",
                "Duplicate exercise identifier",
            ));
        }
        validate_exercise(exercise, &mut issues);
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrorReport::new(
            source.map(ToOwned::to_owned),
            issues,
        ))
    }
}

fn validate_exercise(exercise: &Exercise, issues: &mut Vec<ValidationIssue>) {
    match exercise {
        Exercise::Classification {
            id,
            title,
            prompt,
            categories,
            items,
        } => {
            validate_non_empty(id, None, "id", issues);
            validate_non_empty(title, Some(id), "title", issues);
            validate_non_empty(prompt, Some(id), "prompt", issues);
            validate_string_collection("categories", categories, Some(id), issues);
            if categories.len() < 2 {
                issues.push(ValidationIssue::new(
                    Some(id),
                    "categories",
                    "Provide at least two categories to classify items",
                ));
            }
            if items.is_empty() {
                issues.push(ValidationIssue::new(
                    Some(id),
                    "items",
                    "Include at least one item to classify",
                ));
            }
            validate_classification_items(categories, items, id, issues);
        }
        Exercise::MultipleChoice {
            id,
            title,
            prompt,
            options,
            correct_indices,
            multi_select,
        } => {
            validate_non_empty(id, None, "id", issues);
            validate_non_empty(title, Some(id), "title", issues);
            validate_non_empty(prompt, Some(id), "prompt", issues);
            validate_string_collection("options", options, Some(id), issues);

            if correct_indices.is_empty() {
                issues.push(ValidationIssue::new(
                    Some(id),
                    "correct_indices",
                    "Provide at least one correct index",
                ));
            }

            for &idx in correct_indices {
                if idx >= options.len() {
                    issues.push(ValidationIssue::new(
                        Some(id),
                        "correct_indices",
                        format!("Index {idx} is out of range for {} options", options.len()),
                    ));
                }
            }

            if !*multi_select && correct_indices.len() > 1 {
                issues.push(ValidationIssue::new(
                    Some(id),
                    "correct_indices",
                    "Single-select exercises must have exactly one correct index",
                ));
            }
        }
        Exercise::Scenario {
            id,
            title,
            description,
            prompt,
            choices,
        } => {
            validate_non_empty(id, None, "id", issues);
            validate_non_empty(title, Some(id), "title", issues);
            validate_non_empty(description, Some(id), "description", issues);
            validate_non_empty(prompt, Some(id), "prompt", issues);

            if choices.is_empty() {
                issues.push(ValidationIssue::new(
                    Some(id),
                    "choices",
                    "Provide at least one scenario choice",
                ));
            }
            validate_choices(choices, id, issues);
        }
    }
}

fn validate_non_empty(
    value: &str,
    exercise_id: Option<&str>,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if value.trim().is_empty() {
        issues.push(ValidationIssue::new(
            exercise_id,
            field,
            "Value must not be empty",
        ));
    }
}

fn validate_string_collection(
    field: &str,
    values: &[String],
    exercise_id: Option<&str>,
    issues: &mut Vec<ValidationIssue>,
) {
    if values.is_empty() {
        issues.push(ValidationIssue::new(
            exercise_id,
            field,
            "Provide at least one entry",
        ));
        return;
    }

    let mut seen = HashSet::new();
    for (idx, value) in values.iter().enumerate() {
        if value.trim().is_empty() {
            issues.push(ValidationIssue::new(
                exercise_id,
                field,
                format!("Entry {idx} must not be empty"),
            ));
        }
        if !seen.insert(value) {
            issues.push(ValidationIssue::new(
                exercise_id,
                field,
                format!("Duplicate entry: {value}"),
            ));
        }
    }
}

fn validate_classification_items(
    categories: &[String],
    items: &[ClassificationItem],
    id: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for (idx, item) in items.iter().enumerate() {
        if item.text.trim().is_empty() {
            issues.push(ValidationIssue::new(
                Some(id),
                "items",
                format!("Item {idx} text must not be empty"),
            ));
        }
        if !categories.contains(&item.correct_category) {
            issues.push(ValidationIssue::new(
                Some(id),
                "items",
                format!(
                    "Item {idx} correct_category '{}' is not listed in categories",
                    item.correct_category
                ),
            ));
        }
    }
}

fn validate_choices(choices: &[ScenarioChoice], id: &str, issues: &mut Vec<ValidationIssue>) {
    let mut correct_count = 0usize;

    for (idx, choice) in choices.iter().enumerate() {
        if choice.label.trim().is_empty() {
            issues.push(ValidationIssue::new(
                Some(id),
                "choices",
                format!("Choice {idx} label must not be empty"),
            ));
        }
        if choice.feedback.trim().is_empty() {
            issues.push(ValidationIssue::new(
                Some(id),
                "choices",
                format!("Choice {idx} feedback must not be empty"),
            ));
        }
        if choice.is_correct {
            correct_count += 1;
        }
    }

    if correct_count == 0 {
        issues.push(ValidationIssue::new(
            Some(id),
            "choices",
            "Mark at least one choice as correct",
        ));
    }
}
