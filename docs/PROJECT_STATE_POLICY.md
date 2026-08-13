# Project-State Operational Policy

Status: active

This document implements
`docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md`. Decision 025 is
controlling. Decision 022 independently controls source size, decomposition, and
burndown; project-state work cannot weaken or bypass it.

## Authority Map

| Question | Authority |
|---|---|
| What landed and passed? | Git, tests, and proof gates |
| What is scheduled and next? | `specs/active_frontier.json` |
| What is the readable roadmap? | Generated Active Frontier in `specs/PROGRESS.md` |
| What is available, claimed, blocked, or closed? | `br` / `.beads/issues.jsonl` |
| What mechanism is permitted? | Doctrine, numbered decisions, and governed specs |

Never substitute one layer for another. In particular, `br ready`, issue
priority, recent commits, and an assignee are not roadmap authority.

## Standard Commands

Use the deterministic selector for every “what is next?” query:

```bash
python3 scripts/project_status.py next
python3 scripts/project_status.py next --json
python3 scripts/project_status.py check
python3 scripts/project_status.py check-render
```

Use the project-state checker and standard drift battery before landing state
changes. Text and JSON output MUST select the same stable Frontier item.
They also report its current tracker status, assignee, and live-claim state.
Agents MUST present those joined facts as one answer and MUST NOT supplement
them with stale conversational ownership or earlier tracker output.
`render` updates only the region between `<!-- ACTIVE FRONTIER:START -->` and
`<!-- ACTIVE FRONTIER:END -->`; `check-render` rejects projection drift.

## Work-Item Workflow

1. **Capture:** create an issue in `br`. It remains intake-only unless promoted.
2. **Govern:** complete required research, specification, owner choice, and
   numbered decision without claiming implementation authorization.
3. **Promote:** add a stable item to `specs/active_frontier.json`, including its
   issue, dependency, authorization, state, governing documents, and unblocks.
4. **Generate:** refresh the `specs/PROGRESS.md` Active Frontier from the
   manifest; never hand-edit generated roadmap prose.
5. **Claim:** atomically add the structured Frontier lease, set the issue
   `in_progress`/assignee, and export beads before editing scoped files.
6. **Execute:** stay inside claim scope, decision 022, and the authorized
   acceptance boundary. Cite the issue in commits.
7. **Verify and close:** record proof evidence, close `br` with the landing
   commit, update Frontier state, regenerate its projection, and export beads in
   the same change.

## Claim Contract

The canonical claim is the closed-shape `claim` object on an `in_progress`
Frontier record:

```json
{
  "agent": "codex",
  "harness": "codex-cli",
  "session": "...",
  "claimed_at": "RFC3339 timestamp",
  "heartbeat_at": "RFC3339 timestamp",
  "expires_at": "RFC3339 timestamp",
  "scope": ["path or subsystem"],
  "worktree": "worktree identity",
  "head": "git object id"
}
```

`claim_ttl_hours` at the manifest root is a positive integer and initially eight.
The tracker assignee must equal the claim agent. `heartbeat_at` cannot be in the
future, `expires_at` must be later than the current time, and the heartbeat-to-
expiry interval cannot exceed the configured TTL. A handoff identifies remaining
scope and whether uncommitted work exists. Expiry never silently releases or
reallocates work: reconcile it before another claim. Runtime coordination tools
MAY provide stronger live evidence, but repository state must remain
interpretable without them.

## State and Authorization Rules

- tracker-only intake never appears as a Frontier state and is never selected.
- `planned`: scheduled planning work not yet specified.
- `specified`: specified work not yet execution-ready; with `planning`, the
  remaining specification task may itself be selected.
- `ready`: scheduled, execution-authorized or authorization-free, and unblocked.
- `in_progress`: ready work with a valid unexpired claim.
- `blocked`: at least one explicit hard prerequisite remains.
- `deferred`: deliberately outside the current executable frontier.
- `landed`: committed work, with a required `landing_commit`; verification and
  tracker closure are represented by evidence and tracker state.

Authorization is independent:

- `planning`: research/design may proceed and may be the next task;
  implementation may not.
- `execution`: implementation may proceed when otherwise ready.
- `owner_decision`: an explicit owner decision is required.
- `none`: the work needs no separate execution grant.

## Same-Change Checklist

When roadmap truth changes, update only the affected surfaces, but update all of
them together:

- `specs/active_frontier.json`;
- generated Active Frontier projection;
- `.beads/issues.jsonl` after `br sync --flush-only`;
- governing spec/decision and governance/parity classification, if affected;
- proof or conformance evidence; and
- issue-linked commit message.

Run `git status` and stage only files owned by the current claim. Do not sweep up
another session's work.

## Contradiction Response

If `next` or the checker finds a contradiction, do not choose a task by judgment
alone. Reconcile the higher authority and its projections in a bounded governance
change. The checker may identify stale claims, missing links, invalid transitions,
or dependency conflicts, but must never auto-authorize work, reopen a deferred
decision, reorder the roadmap, or reclaim an issue.

## Source-Health Checklist

Before changing project-state tooling, run the decision 022 source-health gate.
Keep parsers, validation rules, command presentation, and tests in cohesive
modules below their applicable budgets. If a legacy oversized file is touched,
perform real ownership extraction and ratchet its ceiling downward in the same
change. A generated file, manifest, claim, or tracker entry is never an exception
to source-health governance.
