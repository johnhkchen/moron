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
