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

This section reports canonical roadmap milestones and their completion state.
Routine assignments and results are reported from their single task record under
PM059; they do not inherit the milestone selector's completion/claim machinery.
Stop instructions below apply to the affected milestone scope, not every task
covered by another recorded manager grant.

Use the deterministic selector for every “what is next?” query:

```bash
python3 scripts/project_status.py next
python3 scripts/project_status.py next --full
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

For an ordinary “what is next?” query, use compact default output and answer in
a few plain-language sentences: selected task, canonical next step, work-start
status and any blocker or required decision. Do not infer, substitute or append
another dependency-ready substep. Retrieve `--full` or `--json` only when the
work needs the complete contract, evidence or decision packet. Raw output in chat
requires an explicit user request. This owner-directed correction (2026-10-31)
protects both agent context and human attention; historical verbatim rules are
superseded. A shortened action is a briefing, not sufficient execution instructions.

Every such answer requires a fresh invocation in the current repository during
the current user turn. Prior conversation, memory, cached output, an earlier
tool call, direct manifest reading, tracker readiness, and commit history MUST
NOT substitute for the selector. A selector execution or validation failure is
a blocking project-state failure: report it and do not manufacture an answer.
The same fresh-invocation rule applies to `details` for current or named task
completion queries.

When the selected completion step has kind `owner_decision`, the Frontier item
MUST carry `owner_decision` authorization and MUST NOT be `in_progress` or carry
an agent claim. Human and JSON `next`/`details` output MUST expose the responsible
owner or manager boundary plus a closed-shape ordered decision packet: stable request ID,
question, recommended response, exactly resolving governed source marker, and
one explicit response format in full/JSON output. Compact output names the boundary
and responsible actor. Implementers alert that actor and stop without claiming,
editing, choosing a disposition or advancing. Before requesting a disposition,
read the full packet and explain the decision concisely, preserving request IDs,
option meanings and exact response tokens. Covered
manager decisions do not require a renewed owner message. A classification_required
boundary requires classification before final disposition, not acceptance. After explicit
input, one governance transaction records durable review evidence, completes
the owner step, selects the next dependency-ready step, and changes item
authorization to that step's kind.

For every “how do we finish/complete this task?” query, agents MUST run
`details` (no target for current; stable key/issue ID for a named task) and
summarize its relevant ordered steps, work-start instruction and blockers without
adding a reconstructed rival plan. Read the complete contract before execution;
requirements, evidence and post-completion policy remain authoritative. The
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

For an ordinary request for steps, give a concise summary in canonical order;
expand relevant detail when asked. Omitted detail remains available through
`details --full` or `--json`, and must not be treated as waived. Dependency
independence describes ordering validity only; it does not authorize parallel
agents or concurrent step execution.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C01 -->
Every completion contract fixes `max_in_progress_steps` to one and states that
dependency independence does not authorize parallelism.

Every incomplete completion contract selects exactly one dependency-ready
canonical next substep, and human/JSON details expose the same selection.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C02 -->
Every completion contract requires `human_summary`. Its order/numbering and
no-regrouping/supplementation flags protect the canonical contract, not the prose
of a summary. Summaries must preserve meaning and never infer concurrency.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C03 -->
Compact task details expose selection, work-start restrictions and decision
routing. Full text and JSON retain the complete contract and closed-shape policies.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C04 -->
Claude and AGENTS-aware tools use compact output and concise plain-language
answers by default. Raw reports in chat require an explicit user request.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C05 -->
Hermetic tests verify compact output, retained blockers, full-report availability,
manifest order and JSON parity, rejecting unknown policy values.

<!-- REQ:STATE-DETAIL-PRESENTATION:DP-C06 -->
The correction closes only after project-state, governance, and source-health
proof, followed by an explicit Frontier handoff back to S5.

<!-- REQ:STATE-TASK-DETAILS:TD-C01 -->
Completion-plan authority extends decision 025 without merging roadmap,
tracker, or specification roles: the Frontier orders structured remaining work,
beads mirrors its acceptance IDs, and governed documents own the requirements.

<!-- REQ:STATE-TASK-DETAILS:TD-C02 -->
The `details` command must validate the same state and select the same canonical
item as `next`, with equivalent ordered content in full human and JSON output.

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

## Delegated decision routing and delivery rhythm

PM059 is the owner-ratified amendment to PM025's decision routing. The roadmap
explicitly selects one canonical step or durable idle with the validated claim and
presentation rules. An implementer never advances an owner boundary on its own.

An owner-appointed management session resolves the decisions in its verified
scope. A standing project appointment (`task_keys:["*"]`) is reused for declared
kinds across current and future tasks; explicit owner_only reservations remain
owner decisions. Unclassified decisions require manager classification before
disposition when its grant includes that capability; otherwise they stay with
the owner. Legacy bounded appointments retain their original scope. Record appointment
and scope in existing handoff/task evidence. Its disposition identifies manager
session, selected task/step, exact candidate/evidence and PM059 authority; record
it as delegated input, not a fabricated human reply. Then update step evidence,
authorization and affected roadmap/tracker projections in the same transaction.
Use existing response formats and state kinds; do not invent a parallel selector.
No manager is appointed automatically by this policy.

Covered product and visual choices route to the standing project manager;
nondelegated decisions and explicit owner_only reservations route to the owner.
PM029 adoption, external purchases and destructive Git operations retain PM059's
specific approval boundaries. Required literal human actions remain evidence
requirements, not events an agent may assert occurred. Manager authority does not supply missing evidence, invent a numerical
budget or authorize successors lacking their own grant. Preserve independent
review, source health, canonical mutation and the Claude prototype lane.

The delivery unit is complete designer behavior. Within an authorized unit, build,
test, fix, document and deliver through the required independent review. Continue
routine corrections under the retained grant; do not stop after a tracker update,
lease renewal or proof packet. Reuse existing research/fixtures/proof, and attach
supporting records to implementation where practical. Unavailable proof blocks
its acceptance claim only. Escalate genuine product choices together with concrete
recommendations; the owner is not the routine message courier.

PM059's `feature_delivery` field on the existing Frontier item records the
ordinary user workflow, required safety behavior, separate qualification work,
concrete prerequisite reasons and development availability. The selector requires
it for selected or active product-delivery purpose, independently of Beads
type, and validates every supplied contract. Maintenance, pure qualification and
planning are distinct reviewable purposes. Unselected product work needs its
contract before selection/start; migration does not schedule new prerequisite work. Hard issue
dependencies and selected non-implementation steps need explicit blocker reasons.
Development availability requires source-pinned ordinary-application evidence for
every workflow and safety requirement; fixture-only records cannot satisfy it.
These checks validate evidence structure, not observation truth, independent
approval or an agent's time allocation. They never change grants or selection.
CLAUDE.md owns the Rust product-effort requirement; commit labels are not an
effort metric. Continue inside the recorded grant, and
keep broader qualification separate from bounded development availability.
The manager selects product execution by default. Planning/qualification detours
identify a blocked user workflow, unresolved requirement and finite exit in the
existing rationale/dependencies; they are not a queue of unrelated documentation
tasks to consume review waiting time. Explicit owner-requested maintenance or
research keeps its scope. One maintained task record preserves actual independent
findings and disposition; routine substeps do not require separate receipt sets.

## Work-Item Workflow

### Routine implementation and fixes

Use PM059's owner-directed one-task amendment. Record the bounded direct-owner or
verified-PM grant/governing scope, implementer, nonconflicting owned paths and intended
behavior in one Beads task. Claim/export that task before editing, implement, run
focused checks, obtain one actual independent review and land with concise results.
The recorded owner or verified-PM grant supplies authority; a label, priority or claim does
not. `roadmap:intake` records ordinary tasks outside milestone scheduling.
No new Frontier item/lease, completion graph, availability packet or qualification
campaign is required for the correction. Existing milestone schema and selector
checks remain unchanged. Explicit execution holds, revocations, scope conflicts,
hard safety dependencies and retained owner decisions remain binding.

Final milestone acceptance, unavailable proof or a denied action blocks only its
identified scope. The manager continues other authorized work while the owner is
absent. Update milestone projections only when their truth actually changes;
do not restructure an acceptance plan just to land an in-scope fix.

### Milestone admission and execution

The following workflow applies to scheduled roadmap milestones and genuine new
scope/authority boundaries, not to each task inside a product grant.

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
7. **Verify and close:** retain actual proof and independent findings in the
   existing task record; synchronize Frontier, its projection and beads with the
   coherent delivery. Use existing governed document/review markers for same-change
   completion where valid; cite an existing landing commit when available. Do not
   create a second commit solely to reference the first commit hash. A working
   synchronized claim need not have its own commit. Separate genuine authority or
   ownership transitions when necessary; batch routine supporting updates.

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

## Enforced hold, idle and decision-routing operations

The owner-authorized 2026-10-16 PM025 PS-010–013 amendment is controlling over
earlier generic claim/single-next wording. Current schema 7 adds the following
to the existing Frontier; no parallel selector or proof ledger is introduced.

- `work_purpose` (`kind`, concrete `rationale`) is required for every unfinished Frontier
  task. Product-delivery contracts apply before selection/start regardless of
  tracker type. Existing landed history needs no retrospective invented evidence.
- `hold` (`reason`, governed `source_ref`, `resume_requires`, `completion_checkpoint`) retains the grant
  and freezes the internal selected step/status map, prohibiting a claim, active
  step, completion advancement or landed state.
  Record actual resumption authority before removing it. Held next/details stop.
- Root `selection` explicitly says selected or idle. Idle requires a nonempty
  reason and resolving governed disposition reference, zero canonical tasks and
  no unreconciled claims. `next` and untargeted `details` return `next:null` or
  `details:null` plus idle disposition/STOP in JSON, and explicit idle in text.
  Named details still work. Missing data is never idle and ready intake is not
  auto-selected. Closure never requires successor authorization.
- Root `management_appointments` and independently witnessed step
  `decision_authority` use the exact governed JSON witnesses defined by PS-012.
  Item `decision_routing` is only a request; it cannot override step reservation.
  Both next/details expose `decision_input_required`, `owner_input_required`,
  `decision_maker`, appointment scope/provenance and the original packet. Exact
  scoped or standing project delegation routes to the named manager. Explicit
  owner_only and absent/unverified authority route to the owner. A declared
  classification capability routes unknown decisions to the manager for
  classification only; it grants no execution or acceptance. No route authorizes implementer advancement or self-review.

For claim scope, use actual worktree-relative file/directory paths; paths are
normalized with symlinks/parent segments and checked for containment collisions.
A worktree may be absolute or relative to the queried repository. Escaping scopes
are refused. Explicit `subsystem:<name>` reservations are global exact names and
must accompany actual edited paths; they are not a path hierarchy or alias for
missing file ownership. Expiry never releases or transfers ownership silently.

Named details derive stop guidance from hold, completed/blocked/deferred state,
selection and active ownership as well as authorization. A ready task with
`authorization:none` still needs the synchronized claim when otherwise selected
and executable. Planned ungranted placeholders remain stopped. Text/JSON expose
summary, hold, purpose, delivery contract and applicable responsible decision-maker;
querying a task does not select it. Historical successor waits in dated UED/NSD
receipts remain evidence of the prior checker, not a requirement to keep accepted
work open or to resume paused S5A.

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
modules below their applicable budgets. Apply PM065's bounded no-growth defect
rule where applicable; decomposition must perform real ownership extraction and
ratchet its ceiling downward in the same
change. A generated file, manifest, claim, or tracker entry is never an exception
to source-health governance.
