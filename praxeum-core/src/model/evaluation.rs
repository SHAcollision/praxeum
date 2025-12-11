/// Result of evaluating an answer.
#[derive(Debug, Clone)]
pub struct Evaluation {
    /// Whether the answer is fully correct.
    pub correct: bool,
    /// Score in [0.0, 1.0].
    pub score: f32,
    /// Text feedback.
    pub feedback: String,
}
