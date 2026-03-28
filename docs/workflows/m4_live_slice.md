# M4 Live Slice Command Ledger

Status: auxiliary workflow detail shard.
Authority: implementation status remains in `specs/PROGRESS.md` and
`specs/progress/m4_details.md`.

This file carries the detailed current M4 command ledger so
`docs/USER_WORKFLOWS.md` can stay scenario-focused and compact.

## Scope

The current M4 live slice includes:
- Native project scaffold, inspect, and query surfaces.
- Native schematic and board authoring mutation families.
- Forward-annotation audit/proposal/review/apply and artifact flows.
- Manufacturing export/inspect/validate/compare slices for BOM/PnP, drill,
  Excellon, and Gerber (outline/copper/soldermask/silkscreen/paste/mechanical
  subsets).

Current package-linked truth boundary:
- Persisted package-linked subset is limited to silkscreen non-text
  primitives, package `courtyard` on fixed mechanical layer `41`, and
  `models_3d`.
- `component_silkscreen_texts` exists in schema but remains intentionally
  unpopulated in this slice.

For formal contract wording and closure status, use:
- `docs/NATIVE_FORMAT.md`
- `specs/progress/m4_details.md`
