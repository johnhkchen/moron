//! Built-in demo scene for pipeline validation.
//!
//! `DemoScene` produces a short timeline (~5 seconds) using only existing
//! facade methods and techniques. It is used by `moron build` when no
//! scene file is specified, and by end-to-end tests (T-005-04).

use moron_techniques::{FadeIn, FadeUp};

use crate::facade::{M, Scene};

/// A minimal demo scene that exercises the full rendering pipeline.
///
/// Produces a title card, narration, section header, and body text
/// with FadeIn and FadeUp animations. Total duration is approximately
/// 5 seconds at 30 FPS.
pub struct DemoScene;

impl Scene for DemoScene {
    fn build(m: &mut M) {
        m.title("moron Demo");
        m.narrate("This is a demo of the moron rendering pipeline.");
        m.play(FadeIn::default());
        m.beat();

        m.section("Pipeline");
        m.narrate("Scene to timeline to frames to video.");
        m.play(FadeUp::default());
        m.breath();

        m.show("Built with Rust.");
        m.play(FadeIn { duration: 0.5 });
        m.beat();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_scene_builds_without_panic() {
        let mut m = M::new();
        DemoScene::build(&mut m);
    }

    #[test]
    fn demo_scene_has_nonzero_duration() {
        let mut m = M::new();
        DemoScene::build(&mut m);
        assert!(m.timeline().total_duration() > 0.0);
    }

    #[test]
    fn demo_scene_produces_frames() {
        let mut m = M::new();
        DemoScene::build(&mut m);
        assert!(m.timeline().total_frames() > 0);
    }

    #[test]
    fn demo_scene_has_multiple_segments() {
        let mut m = M::new();
        DemoScene::build(&mut m);
        // The scene adds narrations, animations, beats, breaths
        assert!(m.timeline().segments().len() >= 5);
    }
}
