# T-009-04 Design: what-is-moron-showcase

## Core Decision: Rewrite Scene with Correct Ordering

The existing scene structure has the right content but wrong ordering. The fix is
straightforward: reorder calls so animations play before narrations. No new API
surfaces needed.

### Pattern for each slide

```
clear()                          // clean slate
element()                       // create the element
play(animation)                 // animate it in
narrate("...")                  // narrate while element is visible
[optional: more elements + animations]
breath() / beat()               // pacing pause
```

This ensures elements are visible (animation completed) during narration.

## Approach: Targeted Rewrite, Not Wholesale Replacement

**Option A: Minimal reorder** — Keep all existing content/narration text, just fix
ordering and add section header animations.

**Option B: Full rewrite** — New narration text, more slides, dramatic restructure.

**Chosen: Option A.** The existing text is well-written and the 6-slide structure
maps cleanly to the ticket's outline. A full rewrite risks scope creep. The goal
is visual polish, not content authoring.

## Slide-by-Slide Design

### Slide 1: Cold Open
```
title("What is moron?") → play(FadeIn 0.8s) → beat() → narrate() → breath()
```
Title fades in, holds briefly, then narration plays with title visible.

### Slide 2: The Problem
```
section("The Problem") → play(Slide from left + EaseOut) →
narrate() → show("Complex tools...") → play(FadeUp) → breath()
```
Header slides in, narration plays, then pain point text fades up from below.

### Slide 3: The Solution
```
section("A Better Way") → play(Slide from left + EaseOut) →
narrate() → steps([...]) → play(Stagger(FadeUp+OutBack) count=3) → breath()
```
Header slides in, narration, then steps stagger in one by one.

### Slide 4: Key Features
```
section("Built Different") → play(Slide from left + EaseOut) →
narrate() → steps([...]) → play(Stagger(FadeUp+OutBack) count=3) → breath()
```
Same pattern as slide 3 for visual consistency.

### Slide 5: The Metric
```
section("Lean and Mean") → play(Slide from left + EaseOut) →
narrate() → metric("Lines of Code", "< 15K", Down) →
play(CountUp) → beat() → play(Scale pop with OutBack) → beat()
```
Wait — `play()` targets the LAST element, and after CountUp the last element is
still the metric. But we can only call `play()` once per element effectively
(each play adds a new segment). Actually we CAN play multiple techniques
sequentially on the same element: first CountUp, then Scale. Both target the
metric because it's still the last element. But the Scale would overwrite the
CountUp's final state... Actually no, each animation writes to the element
independently based on its own progress. The second animation would run AFTER the
first completes (sequential timeline). At the time the Scale runs, the CountUp
would already be at progress=1.0 (opacity=1.0).

Issue: Scale's `apply()` only sets `scale`, leaving opacity at default 1.0. But
CountUp's `apply()` only sets `opacity`. After CountUp completes (progress=1.0,
opacity=1.0), the Scale animation starts — but Scale's apply returns opacity=1.0
(default), and apply_animations writes ALL fields. So Scale would correctly show
opacity=1.0 and animate scale.

Wait, actually `apply_animations()` writes all 5 fields from `TechniqueOutput`
for non-Steps elements. So a later animation completely replaces an earlier one's
output. If CountUp is at progress=1.0 → opacity=1.0, then Scale starts and
writes scale=0→1 plus opacity=1.0 (default). That works fine: CountUp's final
state is "fully visible", then Scale pops from 0→1 while keeping opacity at 1.

But actually there's a subtle issue: BOTH animations apply every frame. The
last-applied animation's output wins because they write to the same fields. At
time during Scale animation, CountUp is also at progress=1.0 and writes
opacity=1.0 first, then Scale writes opacity=1.0. Order doesn't matter since
both write 1.0. But during CountUp animation (before Scale starts), Scale is at
progress=0.0 which writes scale=0.0. That would make the element invisible!

Actually let me re-read: the Scale default is `from: 0.0, to: 1.0`. At
progress=0.0 (before start), scale = 0.0. That's bad — the metric would be
invisible during CountUp.

**Decision: Skip the Scale pop.** Two sequential animations on the same element
have conflicting states. CountUp alone is sufficient. This avoids the last-write-
wins problem.

Alternative: Use Scale `from: 1.0, to: 1.2` for a subtle pop after CountUp, but
still has the write-conflict issue during CountUp where Scale at progress=0
would set scale=1.0 (which is fine) — wait: `from: 1.0, to: 1.2` at progress=0
gives scale=1.0. That's actually fine! The metric would show at scale=1.0 during
CountUp, then scale up to 1.2 during the pop.

Revised: Use `Scale { from: 1.0, to: 1.1, duration: 0.3 }` with OutBack for a
subtle overshoot pop. At progress=0.0, scale=1.0 (no visual impact during
CountUp).

### Slide 6: Closing
```
title("moron") → play(FadeIn 0.8s) →
show("Offline. Fast. Professional.") → play(FadeIn 0.6s) →
narrate() → beat()
```
Title fades in, tagline fades in, then narration over both.

## Technique Inventory (meets ≥4 requirement)

1. FadeIn — slides 1, 6
2. FadeUp — slides 2, within Stagger on 3, 4
3. Slide — slides 2, 3, 4, 5 (section headers)
4. Stagger(FadeUp+OutBack) — slides 3, 4
5. CountUp — slide 5
6. Scale — slide 5 (optional pop)

6 techniques total. Exceeds the ≥4 requirement.

## Duration Control

Current narration totals ~32.8s. With the reordering, animations now run BEFORE
narrations (adding time), not overlapping with them. Additional animation time:
- Slide 1: FadeIn 0.8s + beat 0.3s
- Slides 2-5: Slide ~0.5s each = 2.0s
- Slides 3-4: Stagger ~0.7s each = 1.4s
- Slide 5: CountUp 1.0s + Scale 0.3s = 1.3s
- Slide 6: FadeIn 0.8s + FadeIn 0.6s = 1.4s

Additional ~7-8s of animation time. Total would be ~40-48s.

To stay within 20-40s, trim narration text. Target: shorter narrations averaging
~2 seconds each. Reduce verbose explanations to punchy statements.

## Test Updates

Existing tests verify structural properties. The segment count assertion
(`>= 10`) may need updating. The closing slide test checks for 2 visible
elements — this should remain valid since slide 6 still has title + show.
