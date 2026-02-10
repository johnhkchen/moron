//! Staging techniques: `GridLayout`, `StackReveal`, `SplitScreen`, Timeline, etc.

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
    #[must_use]
    pub fn with_delay(mut self, delay: f64) -> Self {
        self.delay = delay;
        self
    }

    /// Set the number of elements to stagger across.
    #[must_use]
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Compute the visual output for a specific item in the stagger sequence.
    ///
    /// `index` is 0-based. `progress` is the overall stagger progress (0.0 to 1.0).
    /// Each item starts at a delayed offset and runs through the inner technique.
    pub fn apply_item(&self, index: usize, progress: f64) -> TechniqueOutput {
        self.apply_item_for_count(index, self.count, progress)
    }

    /// Like [`apply_item`](Self::apply_item) but uses an externally-provided `count`
    /// instead of `self.count`. This allows the rendering pipeline to pass the actual
    /// number of items in a Steps element, keeping Stagger in sync automatically.
    pub fn apply_item_for_count(&self, index: usize, count: usize, progress: f64) -> TechniqueOutput {
        let total_dur = self.duration_for_count(count);
        if total_dur <= 0.0 {
            return self.inner.apply(1.0);
        }

        let item_start = if count > 1 {
            #[allow(clippy::cast_precision_loss)]
            { self.delay * index as f64 / total_dur }
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

    /// Compute the total stagger duration for a given item count.
    fn duration_for_count(&self, count: usize) -> f64 {
        let extra = if count > 1 {
            #[allow(clippy::cast_precision_loss)]
            { self.delay * (count - 1) as f64 }
        } else {
            0.0
        };
        self.inner.duration() + extra
    }
}

impl<T: Technique> Technique for Stagger<T> {
    fn name(&self) -> &'static str {
        "Stagger"
    }

    fn duration(&self) -> f64 {
        // Total duration = inner duration + delay * (count - 1)
        let extra = if self.count > 1 {
            #[allow(clippy::cast_precision_loss)]
            { self.delay * (self.count - 1) as f64 }
        } else {
            0.0
        };
        self.inner.duration() + extra
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        // Default: apply to the first item (index 0)
        self.apply_item(0, progress)
    }

    fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
        (0..count)
            .map(|i| self.apply_item_for_count(i, count, progress))
            .collect()
    }
}
