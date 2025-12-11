use praxeum_core::loader::{default_examples_path, DataFormat, ExerciseLoader};
use praxeum_core::model::exercise::Exercise;
use praxeum_core::validator::ValidationIssue;
use praxeum_core::PraxeumError;

#[test]
fn validates_bundled_examples() {
    let path = default_examples_path();
    ExerciseLoader::validate_path(path, DataFormat::Auto).expect("examples should validate");
}

#[test]
fn rejects_duplicate_ids() {
    let toml = r#"
[[exercise]]
kind = "classification"
id = "dup"
title = "one"
prompt = "p"
categories = ["a", "b"]

[[exercise.items]]
text = "foo"
correct_category = "a"

[[exercise]]
kind = "multiple_choice"
id = "dup"
title = "two"
prompt = "p"
options = ["o1", "o2"]
correct_indices = [0]
multi_select = false
"#;

    let err = ExerciseLoader::validate_str(toml, DataFormat::Toml).unwrap_err();
    let issues = extract_issues(err);
    assert!(issues.iter().any(|issue| issue.field == "id"));
}

#[test]
fn catches_invalid_scenario_choices() {
    let toml = r#"
[[exercise]]
kind = "scenario"
id = "scenario_missing"
title = "Scenario Test"
description = ""
prompt = "Pick"

[[exercise.choices]]
label = ""
is_correct = false
feedback = ""
"#;

    let err = ExerciseLoader::validate_str(toml, DataFormat::Auto).unwrap_err();
    let issues = extract_issues(err);
    assert!(issues.iter().any(|issue| issue.field == "description"));
    assert!(issues
        .iter()
        .any(|issue| issue.message.contains("Choice 0 label")));
    assert!(issues
        .iter()
        .any(|issue| issue.message.contains("Mark at least one")));
}

#[test]
fn loads_and_validates_json_root_object() {
    let json = r#"
{"exercises": [
  {
    "kind": "classification",
    "id": "json_classify",
    "title": "Json Classify",
    "prompt": "Sort",
    "categories": ["a", "b"],
    "items": [
      {"text": "Thing", "correct_category": "a"}
    ]
  }
]}
"#;

    let loaded = ExerciseLoader::load_from_str(json, DataFormat::Auto).expect("json should load");
    assert_eq!(loaded.len(), 1);
    if let Exercise::Classification { id, .. } = &loaded[0] {
        assert_eq!(id, "json_classify");
    } else {
        panic!("expected classification exercise");
    }
}

fn extract_issues(err: PraxeumError) -> Vec<ValidationIssue> {
    match err {
        PraxeumError::Validation(report) => report.issues,
        other => panic!("unexpected error type: {other:?}"),
    }
}
