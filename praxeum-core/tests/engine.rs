use praxeum_core::model::exercise::{ClassificationItem, Exercise, ScenarioChoice};
use praxeum_core::{Answer, ExerciseEngine};

#[test]
fn answers_current_and_advances() {
    let exercises = vec![Exercise::MultipleChoice {
        id: "mc1".into(),
        title: "First".into(),
        prompt: "Pick one".into(),
        options: vec!["A".into(), "B".into()],
        correct_indices: vec![1],
        multi_select: false,
    }];

    let mut engine = ExerciseEngine::new(exercises);
    let eval = engine
        .answer_current(&Answer::MultipleChoice(vec![1]))
        .expect("evaluate");
    assert!(eval.correct);
    assert_eq!(engine.position(), 1);
    assert!(engine.current().is_none());
}

#[test]
fn evaluation_handles_all_kinds() {
    let exercises = vec![
        Exercise::Classification {
            id: "class1".into(),
            title: "Class".into(),
            prompt: "Sort".into(),
            categories: vec!["action".into(), "event".into()],
            items: vec![ClassificationItem {
                text: "Digging".into(),
                correct_category: "action".into(),
            }],
        },
        Exercise::Scenario {
            id: "scenario".into(),
            title: "Scene".into(),
            description: "Background".into(),
            prompt: "Choose".into(),
            choices: vec![
                ScenarioChoice {
                    label: "First".into(),
                    is_correct: false,
                    feedback: "No".into(),
                },
                ScenarioChoice {
                    label: "Second".into(),
                    is_correct: true,
                    feedback: "Yes".into(),
                },
            ],
        },
    ];
    let engine = ExerciseEngine::new(exercises);

    let class_eval = engine
        .evaluate(
            engine.by_id("class1").unwrap(),
            &Answer::Classification(vec![0]),
        )
        .expect("classification eval");
    assert!(class_eval.correct);

    let scenario_eval = engine
        .evaluate(engine.by_id("scenario").unwrap(), &Answer::Scenario(1))
        .expect("scenario eval");
    assert!(scenario_eval.correct);
}
