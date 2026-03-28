# M3 Detailed Progress Notes

This file holds detailed M3 implementation notes referenced by
`specs/PROGRESS.md`.

## MoveComponent

Engine/daemon/MCP/CLI slice implemented for board components by UUID with
persisted KiCad footprint `(at ...)` rewrites.

## RotateComponent

Engine/daemon/MCP/CLI slice implemented for board components by UUID with
persisted KiCad footprint rotation rewrites.

## DeleteComponent

Engine/daemon/MCP/CLI slice implemented for board components by UUID with
persisted KiCad `footprint` removal.

## SetValue

Engine/daemon/MCP/CLI slice implemented for board components by UUID with
persisted KiCad `Value` property rewrites.

## SetReference

Engine/daemon/MCP/CLI slice implemented for board components by UUID with
persisted KiCad `Reference` property rewrites.

## AssignPart

Engine/daemon/MCP/CLI slice implemented for sidecar-backed part assignment
with logical pin-net preservation for known part remaps.

## SetPackage

Engine/daemon/MCP/CLI slice implemented for package-backed footprint rewrite,
package-assignment sidecar persistence, and compatible logical remap behavior.

## SetPackageWithPart

Engine/daemon/MCP/CLI slice implemented for explicit compatible part+package
mutation with footprint rewrite and logical pin-net preservation.

## ReplaceComponents

Batched explicit replacement is implemented as one transaction/undo step with
CLI flag collation parity.

## ApplyComponentReplacementPlan

Plan-driven replacement apply from replacement-plan query output is
implemented.

## ApplyComponentReplacementPolicy

Deterministic best-candidate replacement policy apply is implemented.

## ApplyScopedComponentReplacementPolicy

Scoped deterministic best-candidate replacement policy apply is implemented.

## ApplyScopedComponentReplacementPlan

Previously previewed scoped plan apply without policy re-resolution is
implemented.

## ScopedReplacementPlanManifest

Versioned scoped manifest export/inspect/validate/upgrade/apply and legacy
upgrade-on-load behavior are implemented.

## Package-change introspection

Component-scoped package compatibility query and resolution reporting are
implemented.

## Part-change introspection

Component-scoped compatible part candidate query reporting is implemented.

## Replacement-plan introspection

Unified per-component/scoped replacement planning and scoped override/exclusion
editing are implemented.

## SetNetClass

Engine/daemon/MCP/CLI slice implemented with sidecar-backed net-class
persistence.

## SetDesignRule

Default all-scope clearance-rule mutation and verification query path are
implemented.

## DeleteTrack

Engine/daemon/MCP/CLI slice implemented for board track deletion by UUID.

## DeleteVia

Engine/daemon/MCP/CLI slice implemented for board via deletion by UUID.

## Undo/redo (100% undoable)

Current M3 write slice is covered by dedicated undo/redo round-trip evidence
hooks.

## Operation determinism

Current M3 write slice is covered by base and replacement determinism evidence
hooks.

## KiCad write-back

Current M3 write slice plus sidecar writes are covered by write-back fidelity
evidence hooks.

## Round-trip fidelity

Current M3 board and sidecar save->reimport->save artifact stability is covered
by dedicated fidelity hooks.

## MCP write tools

Current write-surface MCP tools for M3 slice are wired and behaviorally
verified.

## Derived data recomputation

Current write operations have immediate recomputation coverage with parity
evidence across engine/daemon/MCP/CLI query surfaces.

## CLI modify command

Current `modify` command write slice and follow-up verification query flows are
implemented.
