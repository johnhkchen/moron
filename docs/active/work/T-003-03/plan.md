# T-003-03 Plan: Implement Pacing Primitives

## Steps

### Step 1: Add Timeline to M and implement methods
- Add constants, timeline field, initialize in new()
- Implement beat(), breath(), wait(), play(), narrate()
- Add timeline() getter

### Step 2: Update and add tests
- beat_adds_silence, breath_adds_silence, wait_adds_custom_silence
- play_records_animation, narrate_records_narration
- timeline_tracks_cumulative_duration

### Step 3: Full workspace verification and commit
