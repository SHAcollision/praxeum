use serde::{Deserialize, Serialize};

/// A single exercise in the Praxeum system.
///
/// Exercises are serialized in JSON or TOML and consumed by frontends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Exercise {
    /// Classification exercise.
    ///
    /// Example: "Action vs Event" with multiple statements to classify.
    Classification {
        /// Unique identifier for this exercise.
        id: String,
        /// Short title for UI.
        title: String,
        /// Main prompt or instruction.
        prompt: String,
        /// List of category labels (e.g. ["action", "event"]).
        categories: Vec<String>,
        /// Items to classify.
        items: Vec<ClassificationItem>,
    },

    /// Multiple-choice exercise.
    ///
    /// Can be single-select or multi-select.
    MultipleChoice {
        id: String,
        title: String,
        prompt: String,
        /// Display options.
        options: Vec<String>,
        /// Indices of correct options (0-based).
        correct_indices: Vec<usize>,
        /// If true, allow multiple selections; otherwise exactly one.
        multi_select: bool,
    },

    /// Scenario-based exercise.
    ///
    /// Example: a praxeologic situation with several possible moves.
    Scenario {
        id: String,
        title: String,
        /// Background description of the situation.
        description: String,
        /// Question posed to the learner.
        prompt: String,
        /// Available choices, each with feedback.
        choices: Vec<ScenarioChoice>,
    },
}

/// Item for classification exercises.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationItem {
    /// Text of the statement.
    pub text: String,
    /// Name of the correct category.
    pub correct_category: String,
}

/// Choice for scenario exercises.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioChoice {
    /// Label shown to the learner (e.g. "Choose project X").
    pub label: String,
    /// Whether this is the praxeologically consistent choice.
    pub is_correct: bool,
    /// Feedback displayed after selection.
    pub feedback: String,
}

impl Exercise {
    /// Return a stable identifier for this exercise.
    pub fn id(&self) -> &str {
        match self {
            Exercise::Classification { id, .. }
            | Exercise::MultipleChoice { id, .. }
            | Exercise::Scenario { id, .. } => id,
        }
    }

    /// Return the title.
    pub fn title(&self) -> &str {
        match self {
            Exercise::Classification { title, .. }
            | Exercise::MultipleChoice { title, .. }
            | Exercise::Scenario { title, .. } => title,
        }
    }
}
