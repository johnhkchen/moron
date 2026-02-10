# T-008-03 Plan: Remove CLI Placeholder Commands

## Steps

1. **Update module doc comment** (line 1) — remove "moron preview" reference
2. **Remove enum variants** — delete `Preview`, `Init`, `Gallery` from `Commands` enum (lines 47-59)
3. **Remove match arms** — delete the three placeholder arms in `main()` (lines 77-86)
4. **Verify** — `cargo check`, `cargo test`, `cargo clippy`

## Testing

- `cargo test` — existing tests continue to pass
- `cargo clippy` — no new warnings
- Manual: `cargo run -- --help` shows only `build` subcommand
