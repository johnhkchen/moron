# T-008-03 Research: Remove CLI Placeholder Commands

## Scope

Single file change: `moron-cli/src/main.rs` (250 lines).

## Current State

The `Commands` enum (line 20-60) has four variants:
- `Build` — fully implemented, calls `run_build()`
- `Preview` — prints "not yet implemented" (lines 47-52)
- `Init` — prints "not yet implemented" (lines 53-57)
- `Gallery` — prints "not yet implemented" (lines 58-59)

The `main()` match block (lines 66-87) handles all four variants. The three placeholder arms (lines 77-86) just print messages and exit.

## Dependencies

No other code references `Commands::Preview`, `Commands::Init`, or `Commands::Gallery`. These variants are only constructed by clap's argument parsing and matched in `main()`. No tests reference these subcommands.

## Module-level doc comment

Line 1 says: `//! moron-cli: Binary wrapper for 'moron build', 'moron preview', and related commands.`

This should be updated to reflect the reduced command set.

## Risk

Zero. Removing dead code with no callers or dependents.
