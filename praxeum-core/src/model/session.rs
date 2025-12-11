use serde::{Deserialize, Serialize};

/// Aggregated session-level metrics capturing pacing and accuracy over time.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SessionMetrics {
    /// Total number of answers recorded in the session.
    pub answered: usize,
    /// Number of correct answers.
    pub correct: usize,
    /// Sum of per-question durations in milliseconds.
    pub total_duration_ms: u128,
    /// Fastest answer duration in milliseconds.
    pub fastest_ms: Option<u128>,
    /// Slowest answer duration in milliseconds.
    pub slowest_ms: Option<u128>,
    /// Current streak of consecutive correct answers.
    pub current_streak: usize,
    /// Longest streak achieved in the session.
    pub longest_streak: usize,
}

impl SessionMetrics {
    /// Record the outcome of a single answered exercise.
    pub fn record_attempt(&mut self, duration_ms: u128, was_correct: bool) {
        self.answered += 1;
        self.total_duration_ms = self.total_duration_ms.saturating_add(duration_ms);

        match self.fastest_ms {
            Some(existing) => self.fastest_ms = Some(existing.min(duration_ms)),
            None => self.fastest_ms = Some(duration_ms),
        }

        match self.slowest_ms {
            Some(existing) => self.slowest_ms = Some(existing.max(duration_ms)),
            None => self.slowest_ms = Some(duration_ms),
        }

        if was_correct {
            self.correct += 1;
            self.current_streak += 1;
            self.longest_streak = self.longest_streak.max(self.current_streak);
        } else {
            self.current_streak = 0;
        }
    }

    /// Calculate accuracy as a value between 0.0 and 1.0.
    pub fn accuracy(&self) -> f32 {
        if self.answered == 0 {
            0.0
        } else {
            self.correct as f32 / self.answered as f32
        }
    }

    /// Average pacing in milliseconds per answered exercise.
    pub fn average_duration_ms(&self) -> Option<f32> {
        if self.answered == 0 {
            None
        } else {
            Some(self.total_duration_ms as f32 / self.answered as f32)
        }
    }
}

/// Serializable snapshot that can be persisted between sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProgressSnapshot {
    /// Exercises completed (by ID) in order of completion.
    pub completed_exercises: Vec<String>,
    /// Session metrics accumulated so far.
    pub metrics: SessionMetrics,
}

impl ProgressSnapshot {
    /// Mark an exercise as completed and update metrics.
    pub fn push_completed(&mut self, exercise_id: impl Into<String>, metrics: SessionMetrics) {
        let id = exercise_id.into();
        if !self.completed_exercises.contains(&id) {
            self.completed_exercises.push(id);
        }
        self.metrics = metrics;
    }

    /// Whether the given exercise was already completed.
    pub fn is_completed(&self, exercise_id: &str) -> bool {
        self.completed_exercises
            .iter()
            .any(|existing| existing == exercise_id)
    }
}
