use crate::error::PraxeumError;
use crate::model::{
    answer::Answer,
    evaluation::Evaluation,
    exercise::{ClassificationItem, Exercise, ScenarioChoice},
};

/// In-memory engine holding a set of exercises.
///
/// This is the main API surface for consumers.
#[derive(Debug)]
pub struct ExerciseEngine {
    exercises: Vec<Exercise>,
    index: usize,
}

impl ExerciseEngine {
    /// Create an engine from a list of exercises.
    pub fn new(exercises: Vec<Exercise>) -> Self {
        ExerciseEngine {
            exercises,
            index: 0,
        }
    }

    /// Number of loaded exercises.
    pub fn len(&self) -> usize {
        self.exercises.len()
    }

    /// Whether there are no exercises.
    pub fn is_empty(&self) -> bool {
        self.exercises.is_empty()
    }

    /// Current cursor position (0-based).
    pub fn position(&self) -> usize {
        self.index
    }

    /// Return how many exercises remain from the current cursor.
    pub fn remaining(&self) -> usize {
        self.exercises.len().saturating_sub(self.index)
    }

    /// Reset the internal cursor to the first exercise.
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Get the current exercise, if any.
    pub fn current(&self) -> Option<&Exercise> {
        self.exercises.get(self.index)
    }

    /// Get an exercise by id.
    pub fn by_id(&self, id: &str) -> Option<&Exercise> {
        self.exercises.iter().find(|ex| ex.id() == id)
    }

    /// Get an exercise at a specific index.
    pub fn at(&self, index: usize) -> Option<&Exercise> {
        self.exercises.get(index)
    }

    /// Advance to the next exercise and return it.
    pub fn advance(&mut self) -> Option<&Exercise> {
        if self.index + 1 < self.exercises.len() {
            self.index += 1;
            self.exercises.get(self.index)
        } else {
            self.index = self.exercises.len();
            None
        }
    }

    /// Evaluate an answer against the exercise at the current cursor.
    pub fn answer_current(&mut self, answer: &Answer) -> Result<Evaluation, PraxeumError> {
        let exercise = self.current().ok_or(PraxeumError::NoExercises)?;
        let eval = self.evaluate(exercise, answer)?;
        self.advance();
        Ok(eval)
    }

    /// Evaluate an answer for the given exercise.
    ///
    /// This does not depend on internal cursor state.
    pub fn evaluate(
        &self,
        exercise: &Exercise,
        answer: &Answer,
    ) -> Result<Evaluation, PraxeumError> {
        match (exercise, answer) {
            (
                Exercise::Classification {
                    categories, items, ..
                },
                Answer::Classification(indices),
            ) => self.eval_classification(categories, items, indices),
            (
                Exercise::MultipleChoice {
                    correct_indices, ..
                },
                Answer::MultipleChoice(indices),
            ) => self.eval_multiple_choice(correct_indices, indices),
            (Exercise::Scenario { choices, .. }, Answer::Scenario(index)) => {
                self.eval_scenario(choices, *index)
            }
            (Exercise::Classification { .. }, _) => Err(PraxeumError::InvalidAnswer(
                "Expected classification answer".into(),
            )),
            (Exercise::MultipleChoice { .. }, _) => Err(PraxeumError::InvalidAnswer(
                "Expected multiple-choice answer".into(),
            )),
            (Exercise::Scenario { .. }, _) => Err(PraxeumError::InvalidAnswer(
                "Expected scenario answer".into(),
            )),
        }
    }

    fn eval_classification(
        &self,
        categories: &[String],
        items: &[ClassificationItem],
        indices: &[usize],
    ) -> Result<Evaluation, PraxeumError> {
        if indices.len() != items.len() {
            return Err(PraxeumError::InvalidAnswer(
                "Classification answer length does not match item count".into(),
            ));
        }

        let mut correct_count = 0usize;
        let mut feedback_lines = Vec::new();

        for (i, (item, &cat_idx)) in items.iter().zip(indices.iter()).enumerate() {
            let cat_name = categories.get(cat_idx).ok_or_else(|| {
                PraxeumError::InvalidAnswer(format!("Invalid category index at position {i}"))
            })?;
            let is_correct = cat_name == &item.correct_category;

            if is_correct {
                correct_count += 1;
                feedback_lines.push(format!("✓ \"{}\" → {}", item.text, cat_name));
            } else {
                feedback_lines.push(format!(
                    "✗ \"{}\" → {} (correct: {})",
                    item.text, cat_name, item.correct_category
                ));
            }
        }

        let score = correct_count as f32 / items.len() as f32;
        Ok(Evaluation {
            correct: correct_count == items.len(),
            score,
            feedback: feedback_lines.join("\n"),
        })
    }

    fn eval_multiple_choice(
        &self,
        correct_indices: &[usize],
        selected_indices: &[usize],
    ) -> Result<Evaluation, PraxeumError> {
        use std::collections::BTreeSet;

        let correct_set: BTreeSet<_> = correct_indices.iter().copied().collect();
        let selected_set: BTreeSet<_> = selected_indices.iter().copied().collect();

        let intersection: BTreeSet<_> = correct_set.intersection(&selected_set).copied().collect();
        let union: BTreeSet<_> = correct_set.union(&selected_set).copied().collect();

        let score = if union.is_empty() {
            0.0
        } else {
            intersection.len() as f32 / union.len() as f32
        };

        let correct = selected_set == correct_set;

        let feedback = format!(
            "Selected: {:?}\nCorrect:  {:?}",
            selected_indices, correct_indices
        );

        Ok(Evaluation {
            correct,
            score,
            feedback,
        })
    }

    fn eval_scenario(
        &self,
        choices: &[ScenarioChoice],
        selected_index: usize,
    ) -> Result<Evaluation, PraxeumError> {
        let choice = choices
            .get(selected_index)
            .ok_or_else(|| PraxeumError::InvalidAnswer("Invalid scenario choice index".into()))?;

        let correct = choice.is_correct;
        let score = if correct { 1.0 } else { 0.0 };

        let feedback = choice.feedback.clone();

        Ok(Evaluation {
            correct,
            score,
            feedback,
        })
    }

    /// Return an iterator over all exercises.
    pub fn iter(&self) -> impl Iterator<Item = &Exercise> {
        self.exercises.iter()
    }
}
