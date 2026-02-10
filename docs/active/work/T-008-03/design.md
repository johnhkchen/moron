# T-008-03 Design: Remove CLI Placeholder Commands

## Approach

Simple deletion. Remove the three unimplemented enum variants and their match arms. Update the module doc comment.

## Alternatives Considered

1. **Keep commands, return proper errors** — Rejected. The ticket explicitly says to remove them. Stub commands create false impressions of functionality.
2. **Feature-gate behind a `dev` flag** — Rejected. Over-engineering for dead code. Re-adding later is trivial.

## Decision

Delete the three variants (`Preview`, `Init`, `Gallery`), their match arms, and update the module doc comment. This is the simplest correct approach.
