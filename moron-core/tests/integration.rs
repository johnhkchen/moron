//! Integration tests for M facade -> timeline -> technique pipeline.

use moron_core::prelude::*;
use moron_core::{BEAT_DURATION, BREATH_DURATION};

#[test]
fn simple_scene_produces_valid_timeline() {
    struct ExplainerScene;

    impl Scene for ExplainerScene {
        fn build(m: &mut M) {
            m.title("Revenue Growth Q4");
            m.narrate("Let's look at the numbers");
            m.beat();
            m.play(moron_techniques::FadeIn::default());
            m.show("$1.2M revenue");
            m.breath();
            m.metric("Growth", "+24%", Direction::Up);
            m.narrate("That's a strong quarter");
        }
    }

    let mut m = M::new();
    ExplainerScene::build(&mut m);

    let tl = m.timeline();
    // narrate, beat, play, breath, narrate = 5 segments
    // (title, show, metric don't add timeline segments yet)
    assert_eq!(tl.segments().len(), 5);
    assert!(tl.total_duration() > 0.0);

    // Verify segment types in order
    assert!(matches!(tl.segments()[0], Segment::Narration { .. }));
    assert!(matches!(tl.segments()[1], Segment::Silence { .. }));
    assert!(matches!(tl.segments()[2], Segment::Animation { .. }));
    assert!(matches!(tl.segments()[3], Segment::Silence { .. }));
    assert!(matches!(tl.segments()[4], Segment::Narration { .. }));
}

#[test]
fn pacing_inserts_correct_durations() {
    let mut m = M::new();
    m.beat();
    m.breath();
    m.wait(2.0);

    let tl = m.timeline();
    assert_eq!(tl.segments().len(), 3);

    assert!((tl.segments()[0].duration() - BEAT_DURATION).abs() < f64::EPSILON);
    assert!((tl.segments()[1].duration() - BREATH_DURATION).abs() < f64::EPSILON);
    assert!((tl.segments()[2].duration() - 2.0).abs() < f64::EPSILON);

    let expected = BEAT_DURATION + BREATH_DURATION + 2.0;
    assert!((tl.total_duration() - expected).abs() < f64::EPSILON);
}

#[test]
fn frame_mapping_at_30fps() {
    let tl = TimelineBuilder::new()
        .fps(30)
        .narration("Hello world", 1.0)
        .silence(1.0)
        .animation("FadeIn", 1.0)
        .build();

    // 3.0s at 30fps = 90 frames
    assert_eq!(tl.total_frames(), 90);
    assert_eq!(tl.frame_at(0.0), 0);
    assert_eq!(tl.frame_at(1.0), 30);
    assert_eq!(tl.frame_at(1.5), 45);
    assert_eq!(tl.frame_at(2.99), 89);
    assert_eq!(tl.frame_at(3.0), 89); // clamped to last frame
    assert_eq!(tl.frame_at(100.0), 89);
}

#[test]
fn frame_mapping_at_60fps() {
    let tl = TimelineBuilder::new()
        .fps(60)
        .animation("FadeUp", 2.0)
        .silence(0.5)
        .build();

    // 2.5s at 60fps = 150 frames
    assert_eq!(tl.total_frames(), 150);
    assert_eq!(tl.frame_at(0.0), 0);
    assert_eq!(tl.frame_at(1.0), 60);
    assert_eq!(tl.frame_at(2.0), 120);
    assert_eq!(tl.frame_at(2.5), 149); // clamped to last frame
}

#[test]
fn multiple_scenes_independent_timelines() {
    struct SceneA;
    impl Scene for SceneA {
        fn build(m: &mut M) {
            m.narrate("Scene A narration");
            m.beat();
        }
    }

    struct SceneB;
    impl Scene for SceneB {
        fn build(m: &mut M) {
            m.play(moron_techniques::FadeIn { duration: 1.0 });
            m.wait(0.5);
            m.play(moron_techniques::Slide::default());
        }
    }

    let mut m_a = M::new();
    SceneA::build(&mut m_a);

    let mut m_b = M::new();
    SceneB::build(&mut m_b);

    // Different number of segments
    assert_eq!(m_a.timeline().segments().len(), 2);
    assert_eq!(m_b.timeline().segments().len(), 3);

    // Different total durations
    assert!((m_a.timeline().total_duration() - m_b.timeline().total_duration()).abs() > 0.01);

    // Independent: modifying one doesn't affect the other
    assert!(matches!(m_a.timeline().segments()[0], Segment::Narration { .. }));
    assert!(matches!(m_b.timeline().segments()[0], Segment::Animation { .. }));
}
