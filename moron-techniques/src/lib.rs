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
pub use technique::{Ease, Technique, TechniqueExt, WithEase};
pub use reveals::{FadeIn, FadeUp};
pub use motion::{Scale, Slide};
pub use staging::Stagger;
pub use data::CountUp;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn technique_composition_type_checks() {
        // Verify that Stagger(FadeUp.with_ease(Ease::OutBack)) compiles and works.
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
        // 1.0 + 0.2 * 4 = 1.8
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
}
