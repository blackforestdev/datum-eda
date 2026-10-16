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
or prose outside the generated Frontier. Ordinary human answers MUST reproduce
its stdout byte-for-byte, including the canonical completion substep, without
supplementation or inferred alternatives.

The command MUST be freshly invoked in the current repository during every user
turn that asks for next-task or completion-plan state. Memory, cached output,
prior conversation or tool calls, direct manifest inspection, tracker output,
and commit history MUST NOT substitute for that invocation. If execution or
validation fails, the agent MUST report the failure and MUST NOT infer an
answer.

A selected `owner_decision` completion step is a hard autonomy boundary. The
parent item MUST expose `owner_decision` authorization, remain unclaimed, and
produce an explicit owner-input alert and complete ordered decision packet in
human and JSON selector output. Every request has a stable ID, exact question,
recommended response, and governed source marker; the packet supplies one exact
response format. Agents MUST notify the owner, present the packet, and stop;
they MUST NOT claim, edit, choose a disposition, or advance to another step.
Explicit owner input is recorded as durable review evidence before the same
governance transaction completes the step, selects its successor, and changes
authorization to match that successor.

The deterministic answer to “how do we complete the current task?” is produced
by `python3 scripts/project_status.py details`; a stable Frontier key or issue ID
may select a named scheduled task. With no target it selects the same canonical
item as `next` and returns its validated ordered completion contract; agents MUST NOT
replace it with a checklist reconstructed from prose.

## PM059 decision-routing amendment

The owner-ratified PM059 delegation is an explicit exception to the human-input
routing above. An owner-appointed manager may disposition only delegated routine
review/continuation/objective-acceptance decisions within existing ratified scope,
using durable manager provenance and the same synchronized state transaction.
Implementers still stop at the boundary; product changes and nondelegated decisions
remain owner-reserved. No selector/schema, claim, response-format or proof bypass
is authorized. See `PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md` and the
project-state policy for appointment, evidence and limits.

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

`specs/active_frontier.json` MUST use schema version 7 for current state (versions 5/6 remain readable for
compatible historical data) and contain every
scheduled roadmap item with a stable unique key, contiguous order, lifecycle
state, authorization state, governing-document links, exact beads issue ID,
hard dependencies, unblocks, summary, and explicit canonical-next/parallel
flags. A ready beads issue absent from this file is intake only.

The manifest MUST explicitly select exactly one deterministic next task or
record a deliberate idle disposition with reason and governed evidence. Idle
may retain ready work without authorizing it; closure never requires a successor. Parallel work MAY be represented, but concurrency MUST NOT make the
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

An item is executable only when its lifecycle, authorization, dependencies,
explicit selection, absence of a hold and claim state jointly permit execution. “Specified,” “ready,” “NEXT,” and
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

### PS-009: Canonical completion contract

The selected non-landed item MUST carry one closed-shape ordered completion
contract with an outcome, stable unique step IDs, work kinds, forward-valid
dependencies, exactly one explicit canonical next substep while work remains,
governed evidence markers, post-completion effects, and an explicit successor
policy. Planning/owner-decision authorization MUST reject
execution-kind steps. Every step ID MUST occur exactly once in the bead acceptance criteria;
every namespaced requirement marker in its governing documents MUST be covered.
`details` human and JSON output MUST preserve the same order and content.

Work-start guidance is derived from validated claim state: open/unassigned work
requires the synchronized Frontier-plus-beads claim transaction only when
selected, unheld and otherwise authorized; a live claim
requires other agents to stand down or coordinate an explicit handoff. Closing
or unblocking work MUST NOT silently authorize or select its successor.

Each completion contract also carries closed-shape execution and presentation
policies. Exactly one step may be `in_progress`; if present, it MUST be the
canonical next substep. The canonical substep MUST be pending or in progress
with every dependency complete, and becomes `null` only when every step is
complete. Dependency independence MUST NOT be represented as authorization for
parallel work. The ordinary human
answer is `details` stdout byte-for-byte: manifest order and numbering are
preserved, and regrouping, supplementation, and inferred concurrency are
forbidden. JSON MUST expose the same policies.

## Owner-authorized production-process correction — 2026-10-16

The owner expressly directs correction of the five audited process defects under
PRODUCTION-PROCESS-CORRECTION, with the bounded appointment and exact receipt in
[existing task evidence](../reviews/specification-history/specification-reorganization.md#owner-authorized-production-process-corrections--ppc01).
This amends standing rules below; historical receipts retain their original
scope and effects. No product grant, qualification or successor is supplied.

### PS-010: Enforced holds and truthful work start

A `hold` has exactly `reason`, governed `source_ref` (repository-relative path
and unique literal marker), `resume_requires`, and `completion_checkpoint`
(the exact internal `canonical_next_step_id` and `step_statuses` map). It preserves underlying
lifecycle/grant and internal completion selection. Held work MUST NOT carry an
agent claim, active completion step or landed lifecycle, or advance beyond its
held completion checkpoint. Both human/JSON next
and details expose hold provenance and resumption boundary with STOP guidance.
Clearing a hold requires the actual authorized resumption disposition; a checker
cannot authenticate human intent. A named query never selects or authorizes work.
Blocked, deferred, landed, unselected and actively owned tasks MUST receive
state-specific guidance rather than a generic invitation to claim.

### PS-011: Explicit idle independent of closure

Schema 7 root `selection` is either `{"mode":"selected"}` with exactly one
canonical task, or `{"mode":"idle","reason":...,"source_ref":...}` with none.
Idle requires a durable governed disposition and no active/unreconciled claims.
Absent/malformed selection, multiple selections, or simultaneous idle/selection
MUST fail. `next` and untargeted `details` return explicit idle in text/JSON;
named details still expose the unchanged task. Ready tasks and tracker intake
are never auto-picked. Completion can select idle without granting a successor.

### PS-012: Work purpose and scoped decision authority

Every unfinished record carries `work_purpose` with exactly `kind` and a concrete
`rationale`. Kinds are `product_delivery`, `qualification`, `maintenance` and
`planning`. Classification is reviewable authority metadata, independent of
tracker type, priority or commit labels. Selected/active product delivery needs
the PM059 contract; unselected planning creates no immediate implementation
prerequisite. Historical landed records need no fabricated retrospective purpose
or delivery evidence. Purpose changes require honest same-change review.

Root `management_appointments` maps stable IDs to `session`, exact `task_keys`,
allowed `decision_kinds` and governed `source_ref`. Immediately after that unique
literal marker, a JSON witness must exactly match `owner_appointed:true`, session,
task keys and decision kinds. This is an independently reviewed transcription of
actual owner appointment; structure alone cannot establish authenticity.

An owner-decision step defaults to owner-reserved. Only an explicit
`decision_authority` (`kind`, `source_ref`) with a separately resolving JSON
witness `{task_key,step_id,decision_kind}` can classify it as delegated. Item
`decision_routing` maps step IDs to requested `appointment` and `decision_kind`.
A route delegates only when its kind matches that independently witnessed step
classification and the verified appointment includes both task and kind. Missing,
unsupported, out-of-scope and owner-reserved routes remain owner boundaries.
Malformed appointment/classification provenance fails validation. Human/JSON
output identifies responsible decision-maker and appointment scope/provenance,
retaining the exact ordered packet and stop-before-advance rule. One actual
sequence-scoped appointment may cover its named decisions without repetitive
owner relays; no broader production appointment is inferred.

### PS-013: Actual claim identities

A claim's worktree is an absolute path or a path relative to the queried repository
root, normalized through filesystem symlinks and parent segments. Bare scope
strings are paths relative to that normalized worktree; absolute scopes, escapes
and ambiguous colon forms fail. Existing aliases resolve to actual paths; future
paths remain valid. Identical/ancestor/descendant path reservations collide,
including equivalent worktree aliases; mere textual prefixes do not. Disjoint
actual worktrees/paths do not collide merely because relative spelling matches.
`subsystem:<name>` is an explicit case-sensitive global exact-name reservation
(name contains alphanumeric, dot, underscore or hyphen, starting alphanumeric).
It neither claims path containment nor replaces declaring paths actually edited.
Expiry and explicit ownership-transfer protections remain unchanged.

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
  planning authorization cannot select feature execution;
- all project-state tooling passes decision 022 source-health enforcement;
- human and JSON task details expose the same complete evidence-linked plan;
- incomplete plans expose exactly one dependency-ready canonical next substep;
- missing acceptance IDs and uncovered governed requirements fail checks; and
- completion never implies automatic successor authorization or selection.
