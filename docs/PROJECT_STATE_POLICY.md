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
python3 scripts/project_status.py details [<Frontier-key|issue-id>]
python3 scripts/project_status.py details --json
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

For an ordinary “what is next?” query, stdout from `next` is the entire answer
and MUST be returned byte-for-byte. It reports both the selected Frontier task
and that task's `canonical_next_step_id`; agents must not infer, substitute, or
append another dependency-ready substep.

For every “how do we finish/complete this task?” query, agents MUST run
`details` (no target for current; stable key/issue ID for a named task) and
reproduce its ordered steps, work-start instruction, requirements, evidence,
and post-completion policy without adding a reconstructed rival plan. The
canonical item carries a closed-shape completion contract whose stable step IDs
must appear exactly once in the bead acceptance criteria and whose evidence
markers must resolve exactly once in its governed documents. Each step declares
`planning`, `owner_decision`, `governance`, or `execution`; planning-authorized
tasks cannot smuggle implementation into their completion plan.

The contract also names exactly one `canonical_next_step_id` whenever
incomplete work remains. It must identify a pending or in-progress step whose
dependencies are complete; an active step and the canonical step must be
identical. A `null` value is valid only after every step is complete. Manifest
order remains presentation order, but order or dependency readiness alone never
selects among multiple candidates.

For an ordinary request for the steps, stdout from `details` is the entire
answer and MUST be returned byte-for-byte: no preface, summary, regrouping,
renumbering, relabeling, or postscript. If the user explicitly requests analysis
in addition to the steps, reproduce the stdout block unchanged before a clearly
separate analysis. Dependency independence describes ordering validity only; it
does not authorize parallel agents or concurrent step execution.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C01 -->
Every completion contract fixes `max_in_progress_steps` to one and states that
dependency independence does not authorize parallelism.

Every incomplete completion contract selects exactly one dependency-ready
canonical next substep, and human/JSON details expose the same selection.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C02 -->
Every completion contract requires `stdout_verbatim`, preserves manifest step
order and numbering, and forbids regrouping, supplementation, and inferred
concurrency.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C03 -->
Human task details render the canonical presentation and execution policy before
the task; JSON exposes the same closed-shape policy objects.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C04 -->
Claude and AGENTS-aware tools must return task-detail stdout byte-for-byte for
ordinary completion-step queries, with no surrounding narrative.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C05 -->
Hermetic tests lock exact policy wording, manifest order, numbering, and JSON
parity and reject any weakened or unknown policy value.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C06 -->
The correction closes only after project-state, governance, and source-health
proof, followed by an explicit Frontier handoff back to S5.

<!-- REQ:STATE-TASK-DETAILS:TD-C01 -->
Completion-plan authority extends decision 025 without merging roadmap,
tracker, or specification roles: the Frontier orders structured remaining work,
beads mirrors its acceptance IDs, and governed documents own the requirements.

<!-- REQ:STATE-TASK-DETAILS:TD-C02 -->
The `details` command must validate the same state and select the same canonical
item as `next`, with equivalent ordered content in human and JSON output.

<!-- REQ:STATE-TASK-DETAILS:TD-C03 -->
The first production plan must reconcile all S5 controlling requirements,
tracker acceptance criteria, governing-document links, and transition effects.

<!-- REQ:STATE-TASK-DETAILS:TD-C04 -->
Hermetic regressions must reject missing plans, malformed steps, bad dependency
order, missing acceptance IDs, and missing or uncovered evidence markers.

<!-- REQ:STATE-TASK-DETAILS:TD-C05 -->
Landing a completion-plan tooling task must close its bead, preserve durable
evidence, and explicitly restore the selected successor; selection is never an
automatic consequence of unblocking.

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
