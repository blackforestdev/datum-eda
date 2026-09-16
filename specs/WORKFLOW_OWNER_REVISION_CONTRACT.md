# Owner-revision infrastructure contract

Status: OR-C01 preparation under the active planning claim; not readiness,
enrollment, implementation permission or product acceptance.
Frontier: WORKFLOW-OWNER-REVISION; issue: dat-workflow-owner-revision-71j.
The installed repair plan remains the unchanged owner-pinned boundary.

<!-- OWNER-REVISION-CONTRACT -->
## Bounded delivery contract prepared under OR-C01

Machine contract: `specs/workflow_delivery/owner-revision.contract.json`.
This is an infrastructure contract, not execution enrollment or installed code.
The real validator surfaces are staged CLI, candidate CLI and authenticated owner
hook, all reaching `check_workflow_delivery.main`; the selector reaches
`workflow_delivery_selector.selector_failures`. Observe actual calls and early
refusals; do not manufacture dispatch events for a handler that did not run.

The eight scenario groups below are mandatory. Every listed variant must retain
its own command, exit status, stdout/stderr, source and tool identities, relevant
before/after protected state and disposition. A grouped pass or historical test
count cannot replace a missing case. Cases run in owned isolated fixtures, never
as hostile changes to the live repository. Synthetic owner messages are fixture
inputs, not real authority. Independent replay uses distinct sessions and fresh
observations against the same reviewed source/fixture identities.

- OR-S01: no-grant legacy behavior; unchanged lane after grant installation;
  exact correction target; synchronized planning claim; correction completion
  returning to pending owner review.
- OR-S02: missing/malformed/duplicate/unpromoted grant; mismatched identity,
  source decision or ancestor; changed instruction, numbered decision, record,
  governing classification, outcome, requirements or historical evidence.
- OR-S03: source writes without scope; correction relabeled execution;
  premature or completed owner review; product landing/acceptance; rollback to
  the old boundary; rewriting committed correction evidence; new work outside
  the exact promoted target. Each must refuse without granting authority.
- OR-S04: missing Frontier identity; missing classification; unpromoted policy;
  valid governance-only admission; dirty/stale base; unrelated item, source,
  environment, enrollment or acceptance changes; invalid/missing exact owner
  response; unchanged existing gates and tasks; all future steps pending.
- OR-S05: missing, expired, mismatched or out-of-scope lease; unsynchronized
  tracker claim; legitimate claim/heartbeat renewal; verified claim before
  implementation. Hooks and selectors enforce their actual inputs; arbitrary
  off-repository edits remain agent obligations and are never claimed prevented.
- OR-S06: retained exact candidate object/bundle and raw evidence; substantive
  Problem/Change/Proof/Roadmap commit and issue references; stale original
  source identity retained rather than relabeled; recoverable handoff. Commit
  prose and scratch-space discipline remain agent obligations unless a tested
  existing entry point enforces them. No universal filesystem enforcement claim.
- OR-S07: report-only output is not enforcement; plain-text usable diagnostics;
  staged/worktree/history views; interrupted inspection, shared-lock refusal,
  missing workspace pins and fresh retry. Refusal preserves source/index/refs/
  trust except explicitly declared diagnostic/support preparation outputs.
- OR-S08: real owner-hook and selector integration; wrong runtime or support
  hash; exact environment selection; distinct independent replay; preserve
  legacy enrollment phases, product ownership and final owner review.

Actual admission/promotion mutation remains separately reviewed: exact request,
owner response and writer coordination; shared lock and inherited child lock;
prepublication pinned-workspace validation; fast-forward only; trust updates with
hooks last; before/after support and effective-trust verification; installed hook
and selector checks; durable partial-state diagnostics, never automatic rollback.
The verified admission packet is evidence for its exact unchanged bootstrap only,
not proof that a newly implemented general admission path works.

<!-- OWNER-REVISION-FOUNDATIONS -->
## Foundation and dimension boundaries

CAD units/precision, numeric entry, design selection identity, grid/snap and
library/connectivity do not apply: this infrastructure authors no design object.
Undo/cancel means refusal/interruption preserves the protected repository state;
persistence/recovery means exact durable candidate/evidence and fresh-invocation
recovery, not native project Save. Settings scope means exact repository trust
and environment, not Global Preferences. Normal, invalid, cancel, scope,
save/reopen (process restart), accessibility (plain text) and failure/recovery
cases are required; CAD precision and design undo/redo are not applicable.

The input inventory deliberately captures all `scripts/` source and fixture
helpers, governing docs, the machine contract and exact recorded headless tool
identities. It is a read boundary, never a write grant. The implementation write
proposal is limited to owner-revision validation, its coverage/category integration,
a general governance-admission validator and activation dispatch, and focused
regressions. Exact paths, owner-selected environment and enrollment must be
presented together before OR-C02 starts. No code is authorized by this prose.

The prepared checkpoint mapping is ready=OR-C01, verify=OR-C02, review=OR-C03,
with null product activation/acceptance slots. OR-C04 remains exact owner-controlled
workflow promotion. Completing readiness requires its actual contract/input/route
checks and an admitted execution transition; this document does not mark it done.

## Fixture control inputs and remaining readiness work

Read the rustfmt exemption manifest and each of its four exact referenced Rust
files, even in documentation-only hook fixtures: exemption loading inspects them.
Retain the original environment-selection map and both legacy environment Blobs,
workspace policy and alignment wiring as explicit input files. These read inputs
never authorize Rust, environment or policy changes in the live repository.

Each case must record its actual isolated policy, Frontier, tracker, index, Git
refs and local trust before/after. Fixture construction supplies those explicit
control inputs from a frozen recipe and records generated commits, timestamps,
lease values and scope. They are case parameters, not fabricated observations;
never copy mutable live state and silently call it the original fixture. The
source-tree snapshot and separate authority closure bind the recipe, tools and
governing bytes; retained fixture snapshots bind the instantiated control inputs.

Before completing OR-C01, freeze a machine-checkable case/surface inventory and
concrete fixture recipes for every mandatory variant above, reconcile the numbered
mechanism proposal, and independently inspect exact enrollment/source/environment
promotion. Structure/reference checks on this draft do not satisfy those remaining
requirements. OR-C01 remains in progress and the execution proposal is unpromoted.
