use praxeum_core::{ProgressSnapshot, SessionMetrics};

#[test]
fn metrics_track_accuracy_and_pacing() {
    let mut metrics = SessionMetrics::default();
    metrics.record_attempt(150, true);
    metrics.record_attempt(320, false);
    metrics.record_attempt(90, true);

    assert_eq!(metrics.answered, 3);
    assert_eq!(metrics.correct, 2);
    assert_eq!(metrics.current_streak, 1);
    assert_eq!(metrics.longest_streak, 1);
    assert_eq!(metrics.fastest_ms, Some(90));
    assert_eq!(metrics.slowest_ms, Some(320));
    assert!((metrics.accuracy() - 0.6666).abs() < 0.01);
    assert!(metrics.average_duration_ms().unwrap() > 180.0);
}

#[test]
fn progress_snapshot_serializes_and_tracks_completions() {
    let mut metrics = SessionMetrics::default();
    metrics.record_attempt(200, true);

    let mut snapshot = ProgressSnapshot::default();
    snapshot.push_completed("ex_1", metrics.clone());
    snapshot.push_completed("ex_1", metrics.clone());

    assert!(snapshot.is_completed("ex_1"));
    assert_eq!(snapshot.completed_exercises.len(), 1);

    let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    let roundtrip: ProgressSnapshot =
        serde_json::from_str(&json).expect("snapshot should deserialize");
    assert_eq!(roundtrip, snapshot);
}
