# Product Mechanics 025: Project-State Authority

Status: ratified doctrine

## Decision

Datum records project state through three deliberately separate authorities:

1. Git, tests, and proof gates establish what has landed and been verified.
2. `specs/active_frontier.json` is the canonical, machine-readable roadmap and
   determines the next scheduled milestone.
3. beads (`br`, canonically exported to `.beads/issues.jsonl`) owns operational
   intake, dependencies, assignment, and closure. The Frontier carries the
   validated lease snapshot for milestone `in_progress` work. PM059's one-task
   amendment governs separately authorized routine implementation and fixes;
   these do not require duplicate Frontier admission or leases.

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
or prose outside the generated Frontier. Per owner direction on 2026-10-31,
ordinary human answers summarize the selected task, canonical completion substep
and blockers concisely. Compact command output is the default; retrieve `--full`
or `--json` only when needed. Paste raw output only on explicit user request.
Summary wording grants no alternative selection or execution authority.

The command MUST be freshly invoked in the current repository during every user
turn that asks for next-task or completion-plan state. Memory, cached output,
prior conversation or tool calls, direct manifest inspection, tracker output,
and commit history MUST NOT substitute for that invocation. If execution or
validation fails, the agent MUST report the failure and MUST NOT infer an
answer.

A selected `owner_decision` completion step is a decision boundary. PM059 and
PS-012 route covered decisions to the verified manager; uncovered or explicitly
retained decisions require owner input. For implementers it is an autonomy boundary. The
parent item MUST expose `owner_decision` authorization, remain unclaimed, and
produce an explicit responsible-actor alert and complete ordered decision packet in
full human and JSON selector output. Every request has a stable ID, exact question,
recommended response, and governed source marker; the packet supplies one exact
response format. Agents MUST notify the responsible owner or manager and stop.
Before soliciting a disposition, read the full packet; present a concise explanation
preserving request IDs, option meanings and exact response tokens;
they MUST NOT claim, edit, choose a disposition, or advance that milestone step.
This boundary does not stop a separate routine task authorized under PM059;
explicit execution holds and scope restrictions still apply.
The authorized owner or manager disposition is recorded as durable review evidence before the same
governance transaction completes the step, selects its successor, and changes
authorization to match that successor.

The deterministic answer to “how do we complete the current task?” is produced
by `python3 scripts/project_status.py details`; a stable Frontier key or issue ID
may select a named scheduled task. With no target it selects the same canonical
item as `next` and returns its validated ordered completion contract; agents MUST NOT
replace it with a checklist reconstructed from prose.

## PM059 decision-routing amendment

The owner-ratified PM059 delegation is an explicit exception to the human-input
routing above. An owner-appointed manager may disposition its covered decisions using durable
provenance and the same synchronized state transaction. A verified standing
project appointment covers declared kinds across current and future tasks;
unclassified decisions require classification first, and explicit owner_only
decisions stay with the owner. Implementers still stop for actual disposition;
no routine owner relay is required when the standing appointment already covers it.
The 2026-10-31 PM059 amendment prospectively separates routine task authorization
and ownership from milestone claims; it supplies no proof or owner-only approval.
See `PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md` and the
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
flags. A ready beads issue absent from this file has no independent milestone
priority. Routine execution requires PM059's recorded manager grant; intake,
readiness and assignment alone never grant it.

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
milestone contains independently executable portions, separate actual blocked
scope from authorized execution. Do not promote each ordinary correction into
another roadmap item; PM059 records its bounded authority and ownership once.

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

This contract describes scheduled milestones, not each routine implementation
task or regression fix. PM059's one-task workflow needs no duplicate step graph.

The selected non-landed item MUST carry one closed-shape ordered completion
contract with an outcome, stable unique step IDs, work kinds, forward-valid
dependencies, exactly one explicit canonical next substep while work remains,
governed evidence markers, post-completion effects, and an explicit successor
policy. Planning/owner-decision authorization MUST reject
execution-kind steps. Every step ID MUST occur exactly once in the bead acceptance criteria;
every namespaced requirement marker in its governing documents MUST be covered.
`details --full` and JSON output MUST preserve the same order and content.

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
parallel work. Presentation mode is `human_summary`: ordinary replies summarize
relevant steps in canonical order and distinguish current work from later work.
Order/numbering and no-regrouping/supplementation flags protect the canonical
contract, not the wording of a human summary. Summaries must not invent steps,
evidence, authorization or concurrency. JSON exposes the same policies; `--full`
retains the complete ordered report. This supersedes earlier verbatim-reply rules
without rewriting historical completion evidence.

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
held completion checkpoint. Both full human/JSON next
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
PM059's delivery-priority correction requires planning/qualification detours to
identify the blocked workflow and finite exit in existing rationale/dependencies.
Ordinary evidence and synchronized state updates accompany the coherent delivery;
existing document/review markers avoid hash-only follow-up commits. A synchronized
working claim requires no separate registration commit. Retain actual ownership,
independent review and mandatory validation.

Root `management_appointments` maps stable IDs to `session`, exact `task_keys`,
allowed `decision_kinds` and governed `source_ref`. Immediately after that unique
literal marker, a JSON witness must exactly match `owner_appointed:true`, session,
task keys and decision kinds. This is an independently reviewed transcription of
actual owner appointment; structure alone cannot establish authenticity.

A verified standing project appointment uses the sole task key `"*"`; mixed
wildcard/list scopes and multiple standing project managers fail validation.
Its actual owner witness may authorize `owner_reserved` decisions and the
separate `decision_classification` duty in addition to the four routine kinds.
Task-scoped appointments cannot acquire these capabilities by adding a route.

A step's `decision_authority` (`kind`, `source_ref`) resolves a separate JSON
witness `{task_key,step_id,decision_kind}`. Explicit `owner_only` never delegates.
Explicit `owner_reserved` can route to a verified standing project manager only
when its actual grant includes that kind. The existing routine classifications
and task routes retain their scope checks. With no explicit route, a unique
covering project appointment supplies the manager route; this reuses the standing
grant without a fresh owner message for each step or future task.

Missing classification never authorizes a final decision. A verified project
manager with `decision_classification` receives `classification_required`, with
`decision_input_required:true`, `owner_input_required:false`, and execution still
stopped. The manager must read the governing source and durably classify the
step before disposition. Without that capability, owner routing remains. An
explicit route cannot turn absence into a verified classification. Malformed
provenance fails validation; invalid explicit routes never silently fall back.
Full human/JSON expose responsible actor, provenance and the unchanged packet. An
appointment proves no test, assessment or independent review; holds and explicit
owner-retained approvals remain effective. Revocation restores owner routing.

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
- full human and JSON task details expose the same complete evidence-linked plan;
- incomplete plans expose exactly one dependency-ready canonical next substep;
- missing acceptance IDs and uncovered governed requirements fail checks; and
- completion never implies automatic successor authorization or selection.
