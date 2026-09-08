# Foundation authority review — working evidence

Status: WDQ-F01 in progress; no foundation contract or product acceptance.
Tracking: `FOUNDATION-WORKFLOW-SPEC` / `dat-manual-foundation-contracts-fsw`.
Review baseline: `82623dde` (2026-09-08 UTC).

## Ownership and purpose

The owner directed this session to finish the development-quality rollout and
explicitly identified GP-CM05V as another session's work. Commit `82623dde`
records this separate foundation planning lease. Preferences selection, scope,
evidence and owner acceptance are unchanged. This record preserves review work;
it is not a new specification, enrollment policy or substitute roadmap.

<!-- EVIDENCE:FOUNDATION-WORKFLOW-SPEC:WDQ-F01-PARTIAL -->
## Reviewed baseline and unresolved authority

PM001, PM002 and PM012 were read completely. None is listed as a source or
consumer in the evidence traceability manifest at this baseline. All three are
classified as doctrine by the governance manifest, but their document headers
retain draft language. This is a clause-status ambiguity, not permission to
ratify every open question or discard explicit requirements.

| Source section | Observed requirement or qualification | Consequence for the foundation review |
| --- | --- | --- |
| PM001, Resolved Mechanism | Journal history, durable undo, identity persistence and object/model revisions are explicitly ratified | Preserve these settled mechanisms; do not ask the owner to choose session-only history again |
| PM001, Direct Commit Versus Proposal; Open Owner Questions | Local visible undoable manual edits have direct-commit criteria, but the document retains detailed policy questions and historical migration ordering | Check later decisions before classifying a question as still open; a historical question list alone is not current authority |
| PM002, Decision; First Proof Slice | Native manual tools must use typed operations and the journaled commit path; proof includes binding, electrical/physical edits, checks, output and reopen | Engine capability alone does not establish native delivery; the bounded foundation contract must state its consuming handoffs without claiming this entire proof slice |
| PM012, QG-DIRECT-EDITING-FEEL | Preview is session state; cancellation leaves no committed mutation; fields validate before commit | Future scenarios need separate preview, invalid-input, cancellation and committed-result observations |
| PM012, QG-CRASH-RECOVERY; QG-DURABLE-UNDO | Failure before and after the journal commit point has different expected recovery; undo survives reopen | Unchanged-source reopen from the pilot cannot discharge edit recovery or durable undo |
| PM012, QG-PERFORMANCE-LATENCY | Preview and transaction latency are distinct; numeric budgets depend on representative fixtures and owner decisions | Do not invent thresholds or reuse Preferences budgets as acceptance of board editing |
| PM012, owner correction dated 2026-08-23 | Floating/PiP/multi-monitor composition requirements were withdrawn | Do not revive those requirements while resolving draft-header ambiguity |

These are source observations and review obligations, not a completed
cross-document reconciliation. PM012's header/classification discrepancy remains
unresolved pending later-authority review. No decision header, mechanism, route
digest, golden or acceptance receipt was changed to resolve it prematurely.

## Process route review

The complete ten-file `workflow-delivery-quality` route was read: its five
research sources, quality plan, PM041, gate contract, adoption packet and pilot
JSON. Existing route digest is
`ec4b5a58ecf43f323e5a05ca25f9ede2de3e972716db5a6d83a35d075c843e0b`.
No member or digest was changed. Important preserved limits:

- The pilot intentionally excludes authored geometry, numeric entry, snapping,
  library edits and durable edit undo. Those N/A dispositions cannot be copied
  into an authoring contract.
- The adoption packet calls for measurements on the pilot and three separately
  enrolled slices. Pilot completion is not evidence of that broader adoption.
- Research uses the existing planning/owner-decision mechanism. It does not need
  a fake infrastructure delivery contract to proceed.
- PM041 enrollment expansion and trust promotion require exact owner review;
  the instruction to finish the rollout is not a fabricated acceptance receipt.

## Remaining review coverage

WDQ-F01 remains in progress. The routed-domain inventory currently includes
`gui-selection` (five files), `prototype-selection` (three files, with a shared
viewport-spec consumer), and `workspace-documentation-and-revision` (79 files).
These counts are inventory, not claims of completed semantic review. Their
complete sources and consumers must be reviewed before adjudicating F1–F8 or
editing their specifications. Later storage, mutation, identity and authoring
authority must also be checked where it resolves the historical questions above.

WDQ-F02 still owes concrete values, boundary cases, transaction timelines and
consumer bindings. WDQ-F03 still owes the reviewed contract and exact proposed
prerequisite handoffs for owner disposition. The broader goal also owes explicit
enrollment and verified adoption beyond the pilot; this foundation task does not
close that goal. No new dependency, runtime implementation, native proof,
independent replay, visual-truth change or product acceptance is claimed here.
