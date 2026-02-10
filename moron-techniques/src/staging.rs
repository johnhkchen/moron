//! Staging techniques: GridLayout, StackReveal, SplitScreen, Timeline, etc.

use crate::technique::Technique;

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
}
