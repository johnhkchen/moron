//! Core Technique trait and shared types for composable animation techniques.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

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

/// Apply an easing curve to a linear progress value `t` in `[0.0, 1.0]`.
#[must_use]
pub fn ease(curve: Ease, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match curve {
        Ease::Linear => t,
        Ease::EaseIn => t * t,
        Ease::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Ease::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        Ease::OutBack => {
            let s = 1.70158;
            let t1 = t - 1.0;
            1.0 + t1 * t1 * ((s + 1.0) * t1 + s)
        }
        Ease::OutBounce => ease_out_bounce(t),
        Ease::Spring => {
            // Damped spring: overshoots then settles
            1.0 - (-6.0 * t).exp() * (2.0 * PI * t).cos()
        }
    }
}

fn ease_out_bounce(t: f64) -> f64 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984_375
    }
}

// ---------------------------------------------------------------------------
// TechniqueOutput
// ---------------------------------------------------------------------------

/// The visual state produced by a technique at a given progress.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechniqueOutput {
    /// Opacity (0.0 = transparent, 1.0 = fully opaque).
    pub opacity: f64,
    /// Horizontal translation in pixels.
    pub translate_x: f64,
    /// Vertical translation in pixels.
    pub translate_y: f64,
    /// Scale factor (1.0 = normal size).
    pub scale: f64,
    /// Rotation in degrees.
    pub rotation: f64,
}

impl Default for TechniqueOutput {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Technique trait
// ---------------------------------------------------------------------------

/// The core trait for all animation techniques.
///
/// Every technique has a name, a duration, and an `apply` method that
/// computes the visual state at a given progress value (0.0 to 1.0).
pub trait Technique {
    /// Human-readable name of this technique.
    fn name(&self) -> &str;

    /// Duration of the technique in seconds.
    fn duration(&self) -> f64;

    /// Compute the visual output at the given progress (0.0 = start, 1.0 = end).
    fn apply(&self, progress: f64) -> TechniqueOutput;

    /// Compute per-item visual output for a multi-item element (e.g., Steps).
    ///
    /// `count` is the number of items. Default: all items get the same output
    /// from `apply(progress)`. Stagger overrides this to produce staggered per-item state.
    fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
        (0..count).map(|_| self.apply(progress)).collect()
    }
}

/// Extension trait providing combinators on any [`Technique`].
pub trait TechniqueExt: Technique + Sized {
    /// Wrap this technique with an easing curve.
    fn with_ease(self, ease_curve: Ease) -> WithEase<Self> {
        WithEase {
            inner: self,
            ease: ease_curve,
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

    fn apply(&self, progress: f64) -> TechniqueOutput {
        let eased = ease(self.ease, progress);
        self.inner.apply(eased)
    }

    fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
        let eased = ease(self.ease, progress);
        self.inner.apply_items(count, eased)
    }
}
