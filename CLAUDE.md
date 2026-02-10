# moron — Motion graphics, Obviously in Rust, Offline Natively

## Project Overview
Rust-based motion graphics engine that produces professional explainer videos from LLM-authored scene files.

## Architecture
- **Director (Rust):** moron-core — timeline, frame sequencing, Chromium bridge, FFmpeg pipeline
- **Cinematographer (React):** packages/ui — visual composition, typography, templates
- **Voice (TTS):** moron-voice — Kokoro/Piper offline TTS

## Build Commands
- `cargo check` — Verify Rust code compiles
- `cargo build` — Build the workspace
- `cargo test` — Run tests
- `cargo clippy` — Lint

## Repository Structure
- moron-core/ — Core engine (scene graph, timeline, rendering pipeline)
- moron-techniques/ — Animation techniques library
- moron-voice/ — TTS abstraction and backends
- moron-themes/ — Theme system
- moron-macros/ — Proc macro sugar (future)
- moron-cli/ — CLI binary (`moron build`, `moron preview`)
- packages/ui/ — @moron/ui React components
- packages/themes/ — @moron/themes CSS + Tailwind themes
- examples/ — Scene files
- gallery/ — Technique documentation
- docs/ — Specification and roadmap

## Key Design Principles
- Air-gapped operation (no network required)
- Solo maintainable (< 15K lines)
- LLM-first authoring (scene files optimized for LLM generation)
- Convention over configuration

### Directory Conventions

```
docs/active/tickets/    # Ticket files (markdown with YAML frontmatter)
docs/active/stories/    # Story files (same frontmatter pattern)
docs/active/work/       # Work artifacts, one subdirectory per ticket ID
docs/archive/           # Completed sprints, stories, and tickets
docs/knowledge/         # Reference material and guides
```

---

## RDSPI Workflow

Every ticket passes through five phases in order. No phases are skipped regardless of ticket size.

### Research

Map the codebase. Produce `research.md` (~200 lines).

Descriptive, not prescriptive. What exists, where, how it connects. Identify the files, modules, patterns, and boundaries relevant to the ticket. Surface assumptions and constraints. Do not propose solutions.

Artifact: `docs/active/work/{ticket-id}/research.md`

### Design

Explore options, evaluate tradeoffs, decide with rationale. Produce `design.md` (~200 lines).

Enumerate viable approaches. Assess each against the codebase reality from Research. Choose one and explain why. Document what was rejected and why. The decision must be grounded in the research, not assumptions.

Artifact: `docs/active/work/{ticket-id}/design.md`

### Structure

Define file-level changes, architecture, and component boundaries. Produce `structure.md` (~200 lines).

Specify which files are created, modified, or deleted. Define module boundaries, public interfaces, and internal organization. Establish the ordering of changes where it matters. This is the blueprint -- not code, but the shape of the code.

Artifact: `docs/active/work/{ticket-id}/structure.md`

### Plan

Sequence the implementation steps. Produce `plan.md` (~200 lines).

Break the work into ordered steps that can be executed and verified independently where possible. Define the testing strategy: what gets unit tests, what needs integration tests, what the verification criteria are. Each step should be small enough to commit atomically.

Artifact: `docs/active/work/{ticket-id}/plan.md`

### Implement

Execute the plan. Track progress in `progress.md`. Commit incrementally.

Follow the plan step by step. After each meaningful unit of work, commit. Update `progress.md` with what has been completed, what remains, and any deviations from the plan. If the plan needs adjustment, document the deviation and rationale before proceeding.

Artifact: `docs/active/work/{ticket-id}/progress.md`

---

## Phase Rules

1. **All five phases always run.** Research, Design, Structure, Plan, Implement. Each phase is cheap (~200 lines, a few minutes). Skipping phases based on ticket size is how context degrades.

2. **~200 lines per artifact.** This is not a hard limit but a forcing function for structured thinking. Enough to be thorough, short enough to review quickly.

3. **Phase transitions.** When a phase completes, update the ticket's `phase` field in its YAML frontmatter. Lisa watches for these changes to update scheduling.

4. **Review points.** Research and Design are high-leverage review points. Reviewing ~200 lines of research or design catches problems before they become thousands of lines of wrong code.

5. **Artifacts are insurance.** If a session crashes or hits limits, the latest artifact plus the ticket is enough to seed a new session at the correct phase.
