# Lisa Loop Setup Guide -- First Implementer Feedback

**Project:** moron (Rust motion graphics engine)
**Date:** 2026-02-09
**Setup method:** Manual, following `docs/knowledge/lisa-loop-setup-guide.md`
**Lisa version:** Pre-release (no `lisa init`, no clonable repo yet)

We are the first team to set up the lisa-loop workflow on a new project. moron is a Rust workspace (Cargo.toml at root, multiple crates) targeting motion graphics rendering. We created the full directory structure, CLAUDE.md, stories, tickets, and dependency graph by hand following the setup guide.

This document captures what worked, what didn't, and what `lisa init` should do differently.

---

## What Went Well

**Directory structure convention is simple and clear.** `docs/active/tickets`, `docs/active/stories`, `docs/active/work` -- three directories, obvious names, flat hierarchy. We set it up in under a minute and never second-guessed the layout. This is the right level of convention: opinionated enough to be consistent, simple enough to remember.

**The RDSPI workflow definition is well-explained and easy to transplant.** We copied it into CLAUDE.md verbatim. The phase descriptions are specific enough that agents produce consistent artifacts without additional prompting. The "descriptive, not prescriptive" framing for Research is particularly effective -- it prevents agents from jumping to solutions before understanding the codebase.

**Ticket YAML frontmatter is straightforward.** The field set is small and every field pulls its weight. We wrote six tickets across two stories and never needed a field that wasn't there, nor felt burdened by a field we didn't need.

**Dependency modeling is the most valuable section.** The rule "if two tickets modify the same files, one must depend on the other" is the single most important sentence in the guide. It directly enables correct parallelism. The example dependency chain in section 4 made it immediately clear how to structure our own tickets. We modeled our five S-002 tickets (T-002-01 through T-002-05) with the facade ticket blocking the integration ticket, and the three middle crate tickets running in parallel -- all based on this section.

**Phase artifacts serve triple duty.** Review checkpoints, crash insurance, knowledge capture. This is a genuinely strong design. The fact that each artifact is a standalone ~200-line document means you can review a ticket's progress by reading one file, restart a crashed session by pointing a new agent at the ticket plus the latest artifact, and understand past decisions months later by reading design.md. We haven't needed crash recovery yet, but the design gives confidence.

---

## Pain Points and Suggestions

### 1. No `lisa init` yet

Manual setup is tedious but manageable for one project. The guide compensates well with clear instructions and a copy-paste CLAUDE.md template. But the biggest win from `lisa init` would be auto-detecting the project type. Our repo has a `Cargo.toml` at the root -- that's enough to pre-fill `cargo check`, `cargo build`, `cargo test`, `cargo clippy` into CLAUDE.md. The same applies for `package.json` (npm), `go.mod` (go build/test), `pyproject.toml` (pytest), etc. This is low-hanging fruit that removes the most error-prone part of manual setup: getting the build commands right.

### 2. CLAUDE.md is doing double duty

Our CLAUDE.md contains both project-specific context (architecture, build commands, source layout) and the full RDSPI workflow definition. The workflow definition is identical across every project that uses lisa -- it's not project-specific at all.

This creates two problems:
- **Setup friction.** You have to copy ~80 lines of workflow definition into every project. It's boilerplate.
- **Drift.** If the RDSPI workflow evolves (new phase rules, updated artifact guidance), every project's CLAUDE.md is now out of date. There's no mechanism to update them.

**Suggestion:** Ship the RDSPI workflow as part of lisa itself, injected into agent context automatically (via the system prompt, a prepended file, or a `--workflow` flag on the claude invocation). CLAUDE.md should only need the project-specific section: description, build commands, source layout, directory conventions. This cuts CLAUDE.md in half and eliminates workflow drift entirely.

### 3. Ticket ID scheme is slightly awkward

The `T-{story}-{sequence}` convention (T-002-03) couples ticket identity to story membership. This creates friction in two scenarios:

- **Reorganizing stories.** If you move a ticket from S-002 to S-003, its ID (T-002-03) now lies about its parent. You either rename it (breaking all `depends_on`/`blocks` references) or live with the inconsistency.
- **Adding tickets out of order.** If S-002 already has T-002-01 through T-002-05 and you need to insert a new ticket between 02 and 03, you get T-002-06 at the end of the sequence -- the numbering no longer reflects the logical order.

Neither of these was a blocker for us, but the coupling is unnecessary. **Suggestion:** Use sequential global IDs (T-001, T-002, ...) with the `story` field in frontmatter handling grouping. The DAG is what matters for scheduling; the naming convention is just for humans. Decoupling the two makes both more flexible.

### 4. `blocks` is redundant information

The guide says to keep both `depends_on` and `blocks` consistent: if T-002-01 depends on nothing and blocks T-002-05, then T-002-05 should list `depends_on: [T-002-01]`. We maintained both sides for all six of our tickets.

This is busywork and a source of bugs. The guide itself says (section 8): "Lisa computes from `depends_on`." If the DAG can be fully determined from `depends_on` alone, then `blocks` is derived data. Requiring humans to maintain both sides of every edge is asking for inconsistency.

**Suggestion:** Compute `blocks` automatically from `depends_on`. Do not require it in ticket frontmatter. If it's useful for human readability, `lisa` can display it in the dashboard or inject it as a computed field -- but it should not be a source-of-truth that humans maintain.

### 5. Initial phase inconsistency

The guide contradicts itself on what phase new tickets should start at:

- **Section 3** (CLAUDE.md template, ticket format example): shows `phase: research` in the example frontmatter.
- **Section 4** (Writing Tickets): says `phase: ready` and explains that `ready` means "eligible for pickup."
- **Section 8** (Notes for `lisa init`): says "Tickets with wrong initial phase" is a known pain point and that `phase: research` looks like the ticket is already in-progress.

Section 4 and 8 are correct; section 3's example is wrong. We caught this because we read the whole guide before starting, but someone skimming for the YAML format would copy section 3's example and get confused when their tickets aren't picked up.

**Fix:** Change the example in section 3 from `phase: research` to `phase: ready`.

### 6. Stories should reference their tickets

Our story S-002 describes "Core Type System" and lists scope bullets, but there's no field listing which tickets belong to it. To see S-002's tickets, you have to scan every ticket file for `story: S-002`. With six tickets across two stories this is fine. With fifty tickets across ten stories it won't be.

**Suggestion:** Add an optional `tickets` field to story frontmatter:

```yaml
---
id: S-002
title: core-type-system
type: story
status: open
priority: high
tickets: [T-002-01, T-002-02, T-002-03, T-002-04, T-002-05]
---
```

This doesn't need to be authoritative for DAG computation (the `story` field on tickets handles that). It's a human convenience for scanning story scope at a glance. `lisa init` and `lisa add-ticket` could maintain it automatically.

### 7. Ticket IDs must be filesystem-safe (not documented)

The guide shows `docs/active/work/T-001-01/` as the artifact directory. This means the ticket ID is used as a directory name. The guide never explicitly states that ticket IDs must be valid directory names.

Our IDs (T-002-01, etc.) are fine -- alphanumeric plus hyphens. But nothing stops someone from writing `id: T-002/03` or `id: T-002 03` and getting a broken work directory.

**Fix:** Add a note to section 4: "Ticket IDs must be valid directory names. Use only alphanumeric characters and hyphens."

### 8. No guidance on archiving

The guide defines `docs/active/` but says nothing about what happens when work is done. Our repo already has `docs/archive/sprints/`, `docs/archive/stories/`, `docs/archive/tickets/` -- but we created these on our own. The guide doesn't mention:

- When to move tickets from active to archive
- Whether to archive per-sprint, per-story, or per-ticket
- What happens to work artifacts when a ticket is archived
- Whether `lisa` tracks archived tickets at all (e.g., for historical dependency resolution)

**Suggestion:** Add a section on the ticket lifecycle beyond `done`. At minimum: "When a story is complete, move its tickets and story file to `docs/archive/`. `lisa init` should create `docs/archive/{sprints,stories,tickets}/` alongside the active directories."

### 9. No guidance on mid-flight ticket modifications

What happens if acceptance criteria need to change while an agent is working on a ticket? Can you edit a ticket that's in `phase: implement`? Does the agent notice? Should you stop the agent first?

We haven't hit this yet, but it's a foreseeable scenario. If an agent is in the Implement phase and you change the acceptance criteria, the agent is working against stale requirements. Its Research through Plan artifacts are all based on the old criteria.

**Suggestion:** Add guidance. Our recommendation: if a ticket is in Research or Design, you can edit freely -- the agent re-reads the ticket at each phase boundary. If a ticket is in Structure or later, stop the agent, update the ticket, reset the phase to Research, and let a new session start fresh. Document this policy in the guide.

### 10. `max_threads` default of 4 is too aggressive

The guide's config table shows `max_threads` defaulting to 4. The prose below it says "start with 2." These should agree, and 2 is the right default.

Four concurrent agents on one branch works when your dependency graph is correct. When it's not -- and on a first setup, it won't be -- four agents stepping on each other is four times the mess. Starting with 2 gives you room to verify the DAG is correct before scaling up.

**Fix:** Change the default in the config table from 4 to 2. Keep the note about going higher once you trust your dependency graph.

---

## Suggestions for `lisa init`

Based on our manual setup experience, here's what `lisa init` should do:

1. **Detect project type.** Look for `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `Makefile`, etc. Pre-fill build, test, and lint commands in CLAUDE.md. For our project: `Cargo.toml` at root means Rust workspace, so fill in `cargo check`, `cargo build`, `cargo test`, `cargo clippy`.

2. **Separate workflow from project config.** Don't put the RDSPI workflow in CLAUDE.md. Inject it into agent context from lisa itself. CLAUDE.md should only contain the project-specific section.

3. **Interactive first story/ticket creation.** Prompt: "Describe what you want to build." Generate an initial story with 3-5 tickets and dependency edges. This bootstraps the DAG so `lisa` has something to work with immediately.

4. **Validate the DAG on creation.** Circular dependency detection, orphan ticket detection (tickets referencing nonexistent dependencies), and a warning for tickets that modify the same crate or module without a dependency edge.

5. **Auto-populate `blocks` from `depends_on`.** Or better: drop `blocks` from required frontmatter entirely. Compute it.

6. **Create archive directories.** `docs/archive/{sprints,stories,tickets}/` alongside `docs/active/`. Signal that archiving is part of the expected workflow.

7. **Generate a `.lisa.toml` config file.** Move configuration out of the zellij plugin config block and into a versionable, shareable file at the project root. The zellij layout can reference it, but the source of truth should be in the repo.

8. **Dry-run mode.** `lisa init --dry-run` shows what would be created without writing files. Useful for understanding the convention before committing to it. Especially valuable for teams evaluating whether to adopt lisa.

9. **Enforce filesystem-safe IDs.** Validate that generated ticket IDs contain only alphanumeric characters and hyphens. Reject or sanitize anything else.

10. **Set `max_threads` default to 2.** Let users scale up deliberately after verifying their DAG.

---

## Summary

The lisa-loop setup guide is solid for a v0 document. The directory conventions, RDSPI workflow, and dependency modeling sections are strong and required minimal interpretation. The main issues are inconsistencies between sections (phase values, max_threads default), missing lifecycle guidance (archiving, mid-flight edits), and redundant manual work (maintaining `blocks`). The CLAUDE.md double-duty problem is the most impactful design issue -- solving it would reduce setup friction and eliminate workflow drift across projects.

For `lisa init`, the highest-value features are: project type detection, DAG validation, and separating the workflow definition from project config. Everything else is nice-to-have.
