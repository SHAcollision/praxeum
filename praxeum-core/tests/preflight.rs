use praxeum_core::loader::{DataFormat, ExerciseLoader};

#[test]
fn preflight_collects_issues_without_panicking() {
    let contents = r#"
[[exercise]]
kind = "multiple_choice"
id = "dup"
title = "First"
prompt = "Pick"
options = ["A", "B"]
correct_indices = [0]
multi_select = false

[[exercise]]
kind = "multiple_choice"
id = "dup"
title = "Second"
prompt = "Pick"
options = ["A", "B"]
correct_indices = [1]
multi_select = false
"#;

    let report = ExerciseLoader::preflight_str(contents, DataFormat::Auto, Some("memory"))
        .expect("preflight runs");
    assert_eq!(report.exercise_count, 2);
    assert!(report.issues.iter().any(|i| i.field == "id"));
    assert!(!report.is_success());
}
