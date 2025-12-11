use crate::model::evaluation::Evaluation;
use crate::model::exercise::Exercise;
use crate::model::session::{ProgressSnapshot, SessionMetrics};

/// Events emitted during a learner session.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Started {
        total_exercises: usize,
    },
    Answered {
        outcome: ScoreOutcome,
        metrics: SessionMetrics,
    },
    Completed {
        metrics: SessionMetrics,
        snapshot: ProgressSnapshot,
    },
}

/// Outcome of scoring a single exercise.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreOutcome {
    pub exercise_id: String,
    pub score: f32,
    pub points_awarded: u32,
    pub duration_ms: u128,
    pub was_correct: bool,
    pub streak: usize,
}

/// Tracks session metrics and points while emitting high-level events.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionTracker {
    metrics: SessionMetrics,
    snapshot: ProgressSnapshot,
    points: u32,
    streak: usize,
    total_score: f32,
    events: Vec<SessionEvent>,
}

impl SessionTracker {
    pub fn new(total_exercises: usize) -> Self {
        let mut tracker = SessionTracker {
            metrics: SessionMetrics::default(),
            snapshot: ProgressSnapshot::default(),
            points: 0,
            streak: 0,
            total_score: 0.0,
            events: Vec::new(),
        };
        tracker
            .events
            .push(SessionEvent::Started { total_exercises });
        tracker
    }

    /// Register an answered exercise and emit the `Answered` event.
    pub fn record_answer(
        &mut self,
        exercise: &Exercise,
        evaluation: &Evaluation,
        duration_ms: u128,
    ) -> SessionEvent {
        self.metrics.record_attempt(duration_ms, evaluation.correct);
        self.streak = if evaluation.correct {
            self.streak.saturating_add(1)
        } else {
            0
        };
        self.total_score += evaluation.score;

        let points_awarded = compute_points(
            evaluation.score,
            evaluation.correct,
            self.streak,
            duration_ms,
        );
        self.points = self.points.saturating_add(points_awarded);

        let outcome = ScoreOutcome {
            exercise_id: exercise.id().to_owned(),
            score: evaluation.score,
            points_awarded,
            duration_ms,
            was_correct: evaluation.correct,
            streak: self.streak,
        };

        self.snapshot
            .push_completed(outcome.exercise_id.clone(), self.metrics.clone());

        let event = SessionEvent::Answered {
            outcome,
            metrics: self.metrics.clone(),
        };
        self.events.push(event.clone());
        event
    }

    /// Emit a final completion event, returning accumulated metrics and snapshot.
    pub fn complete(&mut self) -> SessionEvent {
        let event = SessionEvent::Completed {
            metrics: self.metrics.clone(),
            snapshot: self.snapshot.clone(),
        };
        self.events.push(event.clone());
        event
    }

    pub fn metrics(&self) -> &SessionMetrics {
        &self.metrics
    }

    pub fn snapshot(&self) -> &ProgressSnapshot {
        &self.snapshot
    }

    pub fn points(&self) -> u32 {
        self.points
    }

    pub fn events(&self) -> &[SessionEvent] {
        &self.events
    }

    pub fn average_score(&self) -> f32 {
        if self.metrics.answered == 0 {
            0.0
        } else {
            self.total_score / self.metrics.answered as f32
        }
    }
}

fn compute_points(score: f32, was_correct: bool, streak: usize, duration_ms: u128) -> u32 {
    let base = (score * 120.0).round() as u32 + if was_correct { 40 } else { 15 };
    let speed_bonus = if duration_ms <= 20_000 {
        15
    } else if duration_ms <= 40_000 {
        8
    } else {
        0
    };
    let streak_bonus = streak.saturating_sub(1) as u32 * 5;

    base + speed_bonus + streak_bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::evaluation::Evaluation;

    #[test]
    fn awards_points_and_updates_metrics() {
        let mut tracker = SessionTracker::new(2);
        let exercise = Exercise::MultipleChoice {
            id: "mc1".into(),
            title: "One".into(),
            prompt: "Pick".into(),
            options: vec!["A".into(), "B".into()],
            correct_indices: vec![0],
            multi_select: false,
        };

        let eval = Evaluation {
            correct: true,
            score: 1.0,
            feedback: "nice".into(),
        };

        let event = tracker.record_answer(&exercise, &eval, 10_000);

        let SessionEvent::Answered { outcome, metrics } = event else {
            panic!("expected answered event");
        };

        assert_eq!(outcome.exercise_id, "mc1");
        assert!(metrics.accuracy() > 0.9);
        assert_eq!(tracker.points(), outcome.points_awarded);
        assert_eq!(tracker.events().len(), 2); // started + answered
    }

    #[test]
    fn complete_emits_snapshot() {
        let mut tracker = SessionTracker::new(1);
        let event = tracker.complete();
        let SessionEvent::Completed { snapshot, .. } = event else {
            panic!("expected completion event");
        };
        assert!(snapshot.completed_exercises.is_empty());
    }
}
