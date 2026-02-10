# T-008-03 Structure: Remove CLI Placeholder Commands

## Files Modified

### `moron-cli/src/main.rs`

1. **Line 1**: Update module doc comment to remove mention of `moron preview`
2. **Lines 47-59**: Delete `Preview`, `Init`, `Gallery` variants from `Commands` enum
3. **Lines 77-86**: Delete the three placeholder match arms

No files created or deleted. No interface changes — this is purely subtractive.
