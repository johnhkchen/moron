# T-003-02 Plan: Implement Core Techniques

## Steps

### Step 1: Add TechniqueOutput and ease() in technique.rs
- Define TechniqueOutput struct
- Add apply() to Technique trait
- Implement ease() function with all 7 curves
- Update WithEase::apply()

### Step 2: Implement apply() for each technique
- FadeIn, FadeUp in reveals.rs
- Slide, Scale in motion.rs
- Stagger (+ apply_item) in staging.rs
- CountUp in data.rs

### Step 3: Update lib.rs re-exports
- Add TechniqueOutput and ease to pub use

### Step 4: Write unit tests (8+)
- fade_in_at_start_and_end
- fade_up_opacity_and_translation
- slide_translation
- scale_interpolation
- count_up_value
- stagger_delegates_to_inner
- easing_linear_identity
- easing_curves_boundaries (0→0, 1→1 for all)
- with_ease_remaps_progress

### Step 5: Full workspace verification
- cargo check, cargo test, cargo clippy

### Step 6: Commit and update ticket
