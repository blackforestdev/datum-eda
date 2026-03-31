# M7 Opening Charter

This file records the recommended opening boundary for `M7`. Progress-state
updates belong in `specs/PROGRESS.md`; this shard exists to pin the opening
scope and guardrails before implementation begins.

## Recommended Opening

Open `M7` as the first visual review milestone, not as a general-purpose GUI
or a placement/routing semantic expansion milestone.

Recommended first `M7` slice:
- one read-only review surface for the completed routing-kernel substrate
- consume the existing M5 route proposal, explain, artifact, and selector
  outputs without changing their semantics
- reuse the frozen M6 report/compare/delta evidence vocabulary when strategy
  context is shown
- support review of one selected route proposal or one saved route-proposal
  artifact, with explicit accept/reject handoff remaining in existing machine
  surfaces

The opening `M7` slice should explicitly avoid:
- new routing candidate families or selector semantics
- new M6 objectives, profiles, or intent interpretation
- placement editing, placement autorouting, or push-and-shove behavior
- broad schematic editing, canvas architecture decisions, or widget-framework
  sprawl
- using GUI needs to back-drive engine semantics before a review surface is
  proven

## Entry Conditions

Before coding the first `M7` slice:
1. `M5` is treated as closed for routing-kernel scope in `specs/PROGRESS.md`.
2. `M6` is treated as frozen pending evidence from the current baseline gate.
3. Daemon/runtime panic risk is reduced enough that GUI consumers are not
   exposed to obvious unwrap/expect crashes on ordinary query/review paths.
4. MCP registration/request plumbing is aligned enough that adding a bounded
   visual-review tool does not require fragile catalog/dispatch/runtime
   hand-sync.

## Success Criteria For The First Slice

The first `M7` slice should satisfy all of:
- visual review of one existing route proposal or proposal artifact
- no change to CLI/MCP routing semantics or deterministic selector behavior
- machine-native surfaces remain the authority for apply/export/inspection
- focused proof that the GUI/review layer is a consumer of the engine, not a
  semantic fork of it
