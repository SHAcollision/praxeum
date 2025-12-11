/// Learner answers for different exercise types.
///
/// Frontends construct these from UI input and feed them to the engine.
#[derive(Debug, Clone)]
pub enum Answer {
    /// For classification exercises.
    ///
    /// Each entry is the index of the chosen category for that item.
    /// Length must match the number of items in the exercise.
    Classification(Vec<usize>),

    /// Indices of selected options (0-based).
    MultipleChoice(Vec<usize>),

    /// Index of chosen scenario choice (0-based).
    Scenario(usize),
}
