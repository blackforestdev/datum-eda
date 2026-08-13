# Product Mechanics 025: Project-State Authority

Status: ratified doctrine

## Decision

Datum records project state through three deliberately separate authorities:

1. Git, tests, and proof gates establish what has landed and been verified.
2. `specs/active_frontier.json` is the canonical, machine-readable roadmap and
   determines the next authorized task.
3. beads (`br`, canonically exported to `.beads/issues.jsonl`) owns operational
   intake, dependencies, assignment, and closure. The Frontier carries the
   validated lease snapshot for scheduled `in_progress` work.

The Active Frontier section of `specs/PROGRESS.md` is a generated human
projection of `specs/active_frontier.json`, not an independently edited roadmap.
Specifications and numbered decisions authorize behavior and mechanism; they do
not establish implementation status merely by existing.

The deterministic answer to “what is the next task?” is produced by:

```bash
python3 scripts/project_status.py next
```

`--json` changes presentation, not selection. Agents MUST use this command
instead of inferring priority from recent commits, `br ready`, issue priority,
or prose outside the generated Frontier.

## Why This Is Required

Datum previously exposed several individually useful but mutually drifting
views: prose ordering, implementation ledgers, tracker readiness, old agent
assignments, and commit momentum. Different agents could truthfully inspect
those views and still recommend different work. A roadmap must therefore be
structured, mechanically checked, and distinct from both implementation
evidence and backlog state.

## Normative Rules

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, and **MAY** are
normative.

### PS-001: One roadmap authority

`specs/active_frontier.json` MUST use schema version 1 and contain every
scheduled roadmap item with a stable unique key, contiguous order, lifecycle
state, authorization state, governing-document links, exact beads issue ID,
hard dependencies, unblocks, summary, and explicit canonical-next/parallel
flags. A ready beads issue absent from this file is intake only.

The manifest MUST identify exactly one deterministic next task when executable
work exists. Parallel work MAY be represented, but concurrency MUST NOT make the
answer to `project_status.py next` ambiguous.

### PS-002: Generated human projection

The Active Frontier in `specs/PROGRESS.md` MUST be generated from the manifest
and MUST NOT contain hand-maintained roadmap state. Other documents MAY link to
the Frontier but MUST NOT publish rival current/next ordering.
The generated region is bounded by the literal markers
`<!-- ACTIVE FRONTIER:START -->` and `<!-- ACTIVE FRONTIER:END -->`.

### PS-003: Separate lifecycle and authorization

Every scheduled item MUST separately state:

- lifecycle: `planned`, `specified`, `ready`, `in_progress`, `blocked`,
  `deferred`, or `landed`; and
- authorization: `planning`, `execution`, `owner_decision`, or `none`.

An item is executable only when its lifecycle, authorization, dependencies, and
claim state jointly permit execution. “Specified,” “ready,” “NEXT,” and
“in_progress” are not synonyms. `specified` + `planning` MAY be selected when
the next task is specification work; it never authorizes feature execution.

### PS-004: Tracker role and dependency honesty

beads owns operational issue state. Hard `blocks` edges are execution blockers;
`related` and `parent-child` edges MUST NOT be reported as blockers or a strict
chain. Roadmap scheduling MUST NOT contradict tracker blockers. When a broad
issue contains independently executable portions, it MUST be split rather than
described as both blocked and buildable.

### PS-005: Expiring claims

An assignee or `in_progress` status alone is not a live claim. Every
`in_progress` Frontier record MUST carry a closed-shape structured claim with
agent, harness, session, claimed-at, heartbeat-at, expiry, nonempty scope,
worktree, and claimed HEAD. The tracker assignee MUST equal the claim agent.
Claiming is one synchronization transaction across the manifest and beads
status/assignee/export. Claims expire after the
manifest's positive `claim_ttl_hours` interval unless a renewal extends them;
the initial policy value is eight hours.

Expiry diagnoses stale state; it MUST NOT silently reassign or mutate an issue.
Release, handoff, or reclaim is explicit and auditable. Project-state checks
MUST reject missing, malformed, overlapping, or expired active claims.

### PS-006: Same-change synchronization

A change that alters roadmap truth MUST atomically update all affected
authorities: the Frontier manifest and generated projection, beads export,
governing specification/decision and governance/parity manifests when
applicable, and acceptance evidence. Closure MUST cite the landing commit or
other durable verification evidence. Remaining work becomes a new bounded issue.

### PS-007: Mechanical enforcement

The standard drift battery MUST fail on malformed or ambiguous Frontier state,
duplicate IDs/order, unknown issue or document references, scheduled work with
no issue, tracker/roadmap status or blocker conflicts, stale claims, generated
projection drift, and unauthorized selection of a next task. Checks diagnose;
they MUST NOT invent roadmap or owner decisions.

### PS-008: Source-health law remains controlling

This project-state system is subject to decision 022. Its scripts and tests MUST
remain within normal source limits; touched legacy debt MUST burn down; generated
or data-driven implementation MUST NOT evade physical or logical-module checks.
No roadmap, tracker, claim, or emergency status can waive source-health gates.

## Consequences

A new agent can obtain one reproducible task recommendation, then inspect its
governance, dependencies, claim, and proof evidence without conversational
history. Tracker intake remains cheap, while promotion into scheduled work is an
explicit governance act. Project-state changes carry synchronization work, but
the cost replaces repeated and riskier reconstruction of stale intent.

## Acceptance Criteria

This decision is implemented when tests prove that:

- identical clean checkouts return the same `next` result in text and JSON;
- generated Frontier prose exactly matches the manifest;
- stale or overlapping claims and tracker/roadmap contradictions fail checks;
- intake-only issues cannot become the canonical next task;
- blocked, deferred, and landed items are not selected as executable work, and
  planning authorization cannot select feature execution; and
- all project-state tooling passes decision 022 source-health enforcement.
