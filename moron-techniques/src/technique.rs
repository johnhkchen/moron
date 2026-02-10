//! Core Technique trait and shared types for composable animation techniques.

/// Common easing curves for animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ease {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    OutBack,
    OutBounce,
    Spring,
}

/// The core trait for all animation techniques.
///
/// Every technique has a name and a duration, and can be composed
/// with easing curves via [`TechniqueExt::with_ease`].
pub trait Technique {
    /// Human-readable name of this technique.
    fn name(&self) -> &str;

    /// Duration of the technique in seconds.
    fn duration(&self) -> f64;
}

/// Extension trait providing combinators on any [`Technique`].
pub trait TechniqueExt: Technique + Sized {
    /// Wrap this technique with an easing curve.
    fn with_ease(self, ease: Ease) -> WithEase<Self> {
        WithEase {
            inner: self,
            ease,
        }
    }
}

// Blanket implementation: every Technique automatically gets TechniqueExt.
impl<T: Technique> TechniqueExt for T {}

/// Combinator that wraps a technique with an easing curve.
#[derive(Debug, Clone)]
pub struct WithEase<T: Technique> {
    pub inner: T,
    pub ease: Ease,
}

impl<T: Technique> Technique for WithEase<T> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn duration(&self) -> f64 {
        self.inner.duration()
    }
}
