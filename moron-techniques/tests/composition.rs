//! Composition tests for technique chaining and easing.

use moron_techniques::*;

#[test]
fn stagger_fade_up_with_easing() {
    let fade_up = FadeUp::default(); // duration 0.6, distance 30.0
    let eased = fade_up.with_ease(Ease::OutBack);
    let stagger = Stagger::new(eased).with_count(3).with_delay(0.1);

    assert_eq!(stagger.name(), "Stagger");

    // Total duration: 0.6 + 0.1 * 2 = 0.8
    assert!((stagger.duration() - 0.8).abs() < f64::EPSILON);

    // At progress 0, inner technique hasn't started — output is at start state
    let start = stagger.apply(0.0);
    assert!(start.opacity < 0.01, "opacity at start should be near 0, got {}", start.opacity);
    assert!(start.translate_y > 29.0, "translate_y at start should be near 30, got {}", start.translate_y);

    // At progress 1, first item should be complete — OutBack settles to ~1.0
    let end = stagger.apply(1.0);
    assert!((end.opacity - 1.0).abs() < 0.05, "opacity at end should be near 1.0, got {}", end.opacity);
    assert!(end.translate_y.abs() < 1.5, "translate_y at end should be near 0, got {}", end.translate_y);
}

#[test]
fn eased_slide_midpoint() {
    let slide = Slide::default(); // offset_x = 100.0
    let eased = slide.with_ease(Ease::EaseIn);

    // EaseIn at 0.5 = 0.25 (t^2), so translate_x should be 100 * (1 - 0.25) = 75
    let mid = eased.apply(0.5);
    assert!((mid.translate_x - 75.0).abs() < f64::EPSILON);

    // At start, full offset
    let start = eased.apply(0.0);
    assert!((start.translate_x - 100.0).abs() < f64::EPSILON);

    // At end, no offset
    let end = eased.apply(1.0);
    assert!((end.translate_x - 0.0).abs() < f64::EPSILON);
}

#[test]
fn technique_output_identity() {
    let identity = TechniqueOutput::default();
    assert!((identity.opacity - 1.0).abs() < f64::EPSILON);
    assert!((identity.translate_x - 0.0).abs() < f64::EPSILON);
    assert!((identity.translate_y - 0.0).abs() < f64::EPSILON);
    assert!((identity.scale - 1.0).abs() < f64::EPSILON);
    assert!((identity.rotation - 0.0).abs() < f64::EPSILON);
}
