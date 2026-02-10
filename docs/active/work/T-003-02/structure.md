# T-003-02 Structure: Implement Core Techniques

## Files Modified

### moron-techniques/src/technique.rs
- Add `TechniqueOutput` struct with derives (Debug, Clone, Copy, PartialEq, Default)
- Add `apply(&self, progress: f64) -> TechniqueOutput` to Technique trait
- Add `pub fn ease(curve: Ease, t: f64) -> f64` function
- Update `WithEase` impl to include apply()

### moron-techniques/src/reveals.rs
- FadeIn: implement apply()
- FadeUp: implement apply()

### moron-techniques/src/motion.rs
- Slide: implement apply()
- Scale: implement apply()

### moron-techniques/src/staging.rs
- Stagger: implement apply() (delegates to inner for first item)
- Add `apply_item(index, progress)` method

### moron-techniques/src/data.rs
- CountUp: implement apply()

### moron-techniques/src/lib.rs
- Add `TechniqueOutput` to re-exports
- Add `ease` function to re-exports

## Files NOT Modified
- moron-core/* — no changes needed
- emphasis.rs, transitions.rs, camera.rs — stubs, no techniques to update
