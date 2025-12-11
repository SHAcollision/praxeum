use praxeum_core::loader::{default_examples_path, DataFormat, ExerciseLoader};
use praxeum_core::ExerciseEngine;

#[test]
fn loads_examples_and_counts_exercises() {
    let path = default_examples_path();
    let exercises =
        ExerciseLoader::load_from_path(path, DataFormat::Auto).expect("example file should load");
    assert!(!exercises.is_empty(), "example set should not be empty");

    let engine = ExerciseEngine::new(exercises);
    assert_eq!(engine.len(), 3);
}
