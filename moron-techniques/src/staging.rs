//! Staging techniques: GridLayout, StackReveal, SplitScreen, Timeline, etc.

use crate::technique::{Technique, TechniqueOutput};

/// Applies a technique to a sequence of elements with a staggered delay.
#[derive(Debug, Clone)]
pub struct Stagger<T: Technique> {
    pub inner: T,
    pub delay: f64,
    pub count: usize,
}

impl<T: Technique> Stagger<T> {
    /// Create a new Stagger wrapping the given technique.
    ///
    /// Uses default delay of 0.1s and count of 1.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            delay: 0.1,
            count: 1,
        }
    }

    /// Set the delay between each staggered element.
    pub fn with_delay(mut self, delay: f64) -> Self {
        self.delay = delay;
        self
    }

    /// Set the number of elements to stagger across.
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Compute the visual output for a specific item in the stagger sequence.
    ///
    /// `index` is 0-based. `progress` is the overall stagger progress (0.0 to 1.0).
    /// Each item starts at a delayed offset and runs through the inner technique.
    pub fn apply_item(&self, index: usize, progress: f64) -> TechniqueOutput {
        let total_dur = self.duration();
        if total_dur <= 0.0 {
            return self.inner.apply(1.0);
        }

        let item_start = if self.count > 1 {
            self.delay * index as f64 / total_dur
        } else {
            0.0
        };
        let item_dur = self.inner.duration() / total_dur;

        if progress <= item_start {
            self.inner.apply(0.0)
        } else if progress >= item_start + item_dur {
            self.inner.apply(1.0)
        } else {
            let local = (progress - item_start) / item_dur;
            self.inner.apply(local)
        }
    }
}

impl<T: Technique> Technique for Stagger<T> {
    fn name(&self) -> &str {
        "Stagger"
    }

    fn duration(&self) -> f64 {
        // Total duration = inner duration + delay * (count - 1)
        let extra = if self.count > 1 {
            self.delay * (self.count - 1) as f64
        } else {
            0.0
        };
        self.inner.duration() + extra
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        // Default: apply to the first item (index 0)
        self.apply_item(0, progress)
    }
}
