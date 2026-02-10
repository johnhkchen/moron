//! "What is moron?" — A showcase scene introducing the engine.

use moron_techniques::{FadeIn, FadeUp, Slide, Stagger, CountUp, TechniqueExt, Ease};

use crate::facade::{Direction, M, Scene};

/// A multi-slide showcase scene that introduces the moron engine.
pub struct WhatIsMoronScene;

impl Scene for WhatIsMoronScene {
    fn build(m: &mut M) {
        // Slide-in animation for section headers.
        let section_slide = || {
            Slide { duration: 0.5, offset_x: -200.0, offset_y: 0.0 }
                .with_ease(Ease::EaseOut)
        };

        // -- Slide 1: Cold Open --------------------------------------------
        m.title("What is moron?");
        m.play(FadeIn { duration: 0.8 });
        m.beat();
        m.narrate("What if making explainer videos was as simple as writing Rust?");
        m.breath();

        // -- Slide 2: The Problem ------------------------------------------
        m.clear();
        m.section("The Problem");
        m.play(section_slide());
        m.narrate(
            "Complex tools. Expensive licenses. Hours of manual work.",
        );
        m.show("Complex tools. Expensive licenses. Manual labor.");
        m.play(FadeUp::default());
        m.breath();

        // -- Slide 3: The Solution -----------------------------------------
        m.clear();
        m.section("A Better Way");
        m.play(section_slide());
        m.narrate("Write a scene. Run one command. Get a video.");
        m.steps(&[
            "Write a scene in Rust",
            "Run moron build",
            "Get a finished MP4",
        ]);
        m.play(Stagger::new(FadeUp::default().with_ease(Ease::OutBack)).with_count(3));
        m.breath();

        // -- Slide 4: Key Features -----------------------------------------
        m.clear();
        m.section("Built Different");
        m.play(section_slide());
        m.narrate("No internet. No cloud. Everything runs on your machine.");
        m.steps(&[
            "Fully offline",
            "Open source",
            "LLM-friendly API",
        ]);
        m.play(Stagger::new(FadeUp::default().with_ease(Ease::OutBack)).with_count(3));
        m.breath();

        // -- Slide 5: The Metric -------------------------------------------
        m.clear();
        m.section("Lean and Mean");
        m.play(section_slide());
        m.narrate("All of this in under fifteen thousand lines of code.");
        m.metric("Lines of Code", "< 15K", Direction::Down);
        m.play(CountUp::default());
        m.beat();

        // -- Slide 6: Closing ---------------------------------------------
        m.clear();
        m.title("moron");
        m.play(FadeIn { duration: 0.8 });
        m.show("Offline. Fast. Professional.");
        m.play(FadeIn { duration: 0.6 });
        m.narrate("Motion graphics. Obviously in Rust. Offline natively.");
        m.beat();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_moron_builds_without_panic() {
        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);
    }

    #[test]
    fn what_is_moron_has_nonzero_duration() {
        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);
        assert!(m.timeline().total_duration() > 0.0);
    }

    #[test]
    fn what_is_moron_has_multiple_segments() {
        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);
        assert!(m.timeline().segments().len() >= 10);
    }

    #[test]
    fn what_is_moron_clears_between_slides() {
        use crate::frame::compute_frame_state;

        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);

        // At the very end, only the closing slide elements should be visible.
        let total = m.timeline().total_duration();
        let fs = compute_frame_state(&m, total - 0.01);

        let visible: Vec<_> = fs.elements.iter().filter(|e| e.visible).collect();
        // Closing slide has: title("moron") + show("Offline. Fast. Professional.")
        assert_eq!(visible.len(), 2, "Only closing slide elements should be visible at the end");
        assert_eq!(visible[0].content, "moron");
    }

    #[test]
    fn what_is_moron_title_fades_in() {
        use crate::frame::compute_frame_state;

        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);

        // Title is element 0, FadeIn(0.8s) is the first animation segment.
        // At midpoint of FadeIn (t=0.4), opacity should be ~0.5.
        let fs = compute_frame_state(&m, 0.4);
        let title = &fs.elements[0];
        assert!(title.visible, "title should be visible during fade");
        assert!(title.opacity > 0.1 && title.opacity < 0.9,
            "title opacity at midpoint should be partial, got: {}", title.opacity);

        // After FadeIn completes (t=0.8+), opacity should be 1.0.
        let fs = compute_frame_state(&m, 1.0);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn what_is_moron_duration_in_range() {
        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);
        let dur = m.timeline().total_duration();
        assert!(dur >= 15.0, "scene too short: {dur:.1}s");
        assert!(dur <= 50.0, "scene too long: {dur:.1}s");
    }

    #[test]
    fn what_is_moron_uses_at_least_four_techniques() {
        use crate::timeline::Segment;

        let mut m = M::new();
        WhatIsMoronScene::build(&mut m);

        let technique_names: std::collections::HashSet<&str> = m
            .timeline()
            .segments()
            .iter()
            .filter_map(|seg| match seg {
                Segment::Animation { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(technique_names.len() >= 4,
            "should use at least 4 techniques, found: {:?}", technique_names);
    }
}
