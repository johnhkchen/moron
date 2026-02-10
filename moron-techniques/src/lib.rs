//! moron-techniques: Composable animation techniques for motion graphics.
//!
//! Each technique is a Rust struct implementing the Technique trait.
//! ~30 techniques across categories: reveals, motion, morphing, staging, emphasis, camera, transitions, data.

pub mod technique;
pub mod reveals;
pub mod motion;
pub mod staging;
pub mod emphasis;
pub mod camera;
pub mod transitions;
pub mod data;

// Re-export core types for convenient access.
pub use technique::{ease, Ease, Technique, TechniqueExt, TechniqueOutput, WithEase};
pub use reveals::{FadeIn, FadeUp};
pub use motion::{Scale, Slide};
pub use staging::Stagger;
pub use data::CountUp;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn technique_composition_type_checks() {
        let fade_up = FadeUp::default();
        let eased = fade_up.with_ease(Ease::OutBack);
        let stagger = Stagger::new(eased);

        assert_eq!(stagger.name(), "Stagger");
        assert_eq!(stagger.inner.name(), "FadeUp");
        assert_eq!(stagger.inner.ease, Ease::OutBack);
    }

    #[test]
    fn with_ease_preserves_duration() {
        let slide = Slide::default();
        let original_duration = slide.duration();
        let eased = slide.with_ease(Ease::EaseInOut);
        assert_eq!(eased.duration(), original_duration);
    }

    #[test]
    fn stagger_calculates_total_duration() {
        let fade_in = FadeIn { duration: 1.0 };
        let stagger = Stagger::new(fade_in).with_delay(0.2).with_count(5);
        assert!((stagger.duration() - 1.8).abs() < f64::EPSILON);
    }

    #[test]
    fn all_techniques_have_names() {
        assert_eq!(FadeIn::default().name(), "FadeIn");
        assert_eq!(FadeUp::default().name(), "FadeUp");
        assert_eq!(Slide::default().name(), "Slide");
        assert_eq!(Scale::default().name(), "Scale");
        assert_eq!(CountUp::default().name(), "CountUp");
    }

    // -- apply() tests --

    #[test]
    fn fade_in_at_start_and_end() {
        let fi = FadeIn::default();
        let start = fi.apply(0.0);
        let end = fi.apply(1.0);
        assert!((start.opacity - 0.0).abs() < f64::EPSILON);
        assert!((end.opacity - 1.0).abs() < f64::EPSILON);
        assert!((end.scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fade_up_opacity_and_translation() {
        let fu = FadeUp::default(); // distance = 30.0
        let start = fu.apply(0.0);
        let mid = fu.apply(0.5);
        let end = fu.apply(1.0);

        assert!((start.opacity - 0.0).abs() < f64::EPSILON);
        assert!((start.translate_y - 30.0).abs() < f64::EPSILON);

        assert!((mid.opacity - 0.5).abs() < f64::EPSILON);
        assert!((mid.translate_y - 15.0).abs() < f64::EPSILON);

        assert!((end.opacity - 1.0).abs() < f64::EPSILON);
        assert!((end.translate_y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn slide_translation() {
        let s = Slide::default(); // offset_x = 100.0
        let start = s.apply(0.0);
        let end = s.apply(1.0);

        assert!((start.translate_x - 100.0).abs() < f64::EPSILON);
        assert!((end.translate_x - 0.0).abs() < f64::EPSILON);
        assert!((start.opacity - 1.0).abs() < f64::EPSILON); // opacity stays 1
    }

    #[test]
    fn scale_interpolation() {
        let s = Scale::default(); // from=0.0, to=1.0
        assert!((s.apply(0.0).scale - 0.0).abs() < f64::EPSILON);
        assert!((s.apply(0.5).scale - 0.5).abs() < f64::EPSILON);
        assert!((s.apply(1.0).scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn count_up_value() {
        let cu = CountUp { duration: 1.0, from: 10.0, to: 50.0 };
        assert!((cu.current_value(0.0) - 10.0).abs() < f64::EPSILON);
        assert!((cu.current_value(0.5) - 30.0).abs() < f64::EPSILON);
        assert!((cu.current_value(1.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stagger_delegates_to_inner() {
        let fi = FadeIn { duration: 0.5 };
        let stagger = Stagger::new(fi).with_count(3).with_delay(0.1);
        // At progress 0, first item should be at start
        let start = stagger.apply(0.0);
        assert!((start.opacity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn easing_linear_identity() {
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            assert!((ease(Ease::Linear, t) - t).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn easing_curves_boundaries() {
        // All curves should map 0→~0 and 1→~1
        let curves = [
            Ease::Linear, Ease::EaseIn, Ease::EaseOut, Ease::EaseInOut,
            Ease::OutBack, Ease::OutBounce, Ease::Spring,
        ];
        for curve in curves {
            let at_0 = ease(curve, 0.0);
            let at_1 = ease(curve, 1.0);
            assert!(
                at_0.abs() < 0.01,
                "{curve:?} at 0.0 = {at_0} (expected ~0)"
            );
            assert!(
                (at_1 - 1.0).abs() < 0.01,
                "{curve:?} at 1.0 = {at_1} (expected ~1)"
            );
        }
    }

    #[test]
    fn with_ease_remaps_progress() {
        let fi = FadeIn { duration: 0.5 };
        let eased = fi.with_ease(Ease::EaseIn);

        // EaseIn at 0.5 = 0.25 (t^2), so opacity should be 0.25
        let mid = eased.apply(0.5);
        assert!((mid.opacity - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn outback_overshoots() {
        // OutBack should overshoot 1.0 somewhere in the middle
        let mid = ease(Ease::OutBack, 0.5);
        assert!(mid > 1.0, "OutBack at 0.5 should overshoot: {mid}");
    }
}
