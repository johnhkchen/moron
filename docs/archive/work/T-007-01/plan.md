# T-007-01 Plan: Template System Architecture

## Step 1: Add template field to FrameState

**File**: `packages/ui/src/types.ts`
**Change**: Add `template?: string` to `FrameState` interface
**Verify**: `npm run typecheck` passes (optional field is backward-compatible)

## Step 2: Create template registry

**File**: `packages/ui/src/templates/registry.ts` (new)
**Change**: Implement TemplateComponent type, registry Map, registerTemplate/getTemplate/listTemplates functions, auto-register MoronFrame as "default"
**Verify**: TypeScript compiles cleanly

## Step 3: Update templates index

**File**: `packages/ui/src/templates/index.ts`
**Change**: Replace stub with re-exports from `./registry`
**Verify**: TypeScript compiles cleanly

## Step 4: Update package exports

**File**: `packages/ui/src/index.ts`
**Change**: Add re-exports for template registry functions and type
**Verify**: `npm run typecheck` passes

## Step 5: Create host page

**File**: `packages/ui/src/host.tsx` (new)
**Change**: React entry point with App component, window.__moron_setFrame global, createRoot mounting
**Verify**: TypeScript compiles cleanly

## Step 6: Add esbuild dependency

**File**: `packages/ui/package.json`
**Change**: Add esbuild to devDependencies, add build script
**Verify**: `npm install` succeeds

## Step 7: Create build script

**File**: `packages/ui/build.mjs` (new)
**Change**: Node.js script using esbuild API to bundle host.tsx + default.css into dist/index.html
**Verify**: `npm run build` produces dist/index.html

## Step 8: Verify everything

**Checks**:
- `npm run typecheck` passes
- `npm run build` produces dist/index.html
- dist/index.html contains inlined JS and CSS
- dist/index.html defines window.__moron_setFrame

## Testing Strategy

### TypeScript type checking (npm run typecheck)
- Verifies all types align: TemplateComponent matches MoronFrameProps shape
- Verifies FrameState.template is optional (backward-compatible)
- Verifies host.tsx types correctly

### Build verification (npm run build)
- Produces dist/index.html
- Output is self-contained (no external imports)
- HTML structure is valid (has root div, script, style)

### Manual inspection
- dist/index.html can be opened in a browser
- Console shows no errors on load
- window.__moron_setFrame is defined as a function

### No unit tests in this ticket
The template registry is trivial (Map wrapper). The host page is glue code. The build script is verified by running it. Unit tests would be lower value than the integration verification above. Template-specific tests belong in T-007-02+ when actual templates exist.

## Ordering Rationale

Steps 1-4 build the type system and registry bottom-up. Step 5 depends on the registry existing. Steps 6-7 depend on the host page existing. Step 8 is final verification. Each step can be verified independently.
