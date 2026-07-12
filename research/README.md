# `/research/` — Datum EDA Research Index

Governing index for all research artifacts that inform Datum EDA design
decisions. Add a row to the relevant table whenever a new artifact lands;
do not let folders accumulate without an index entry.

## Purpose

`/research/` holds the source-of-truth research that feeds Datum's
design and specification work. Two kinds of artifact live here:

- **Topic research** — long-form survey reports (industry surveys,
  algorithm studies, standards landscapes). Each topic gets its own
  kebab-case folder containing one primary `*_RESEARCH.md` report.
- **Source-analysis sandboxes** — symlinks into out-of-tree
  competitor/reference codebases under `~/sandbox/` that the research
  draws on directly.

Research artifacts feed **derived guidance documents** which live in
`/docs/` (most often `/docs/gui/` for `M7` GUI work) and ultimately
**specification edits** under `/specs/`. See
`/docs/RESEARCH_TRACEABILITY.md` for the traceability convention.

## Conventions

- **Folder naming**: kebab-case, scoped by topic
  (`airwire-rendering/`, not `m7-airwire/` and not `airwireRendering/`).
- **Primary report file**: `<TOPIC>_RESEARCH.md` in SCREAMING_SNAKE_CASE
  (e.g. `IPC_COMPLIANCE_RESEARCH.md`). One primary report per folder.
  Supporting artifacts (extracts, tables, raw notes) sit alongside.
- **Derived guidance docs** belong in `/docs/`, not in `/research/`.
  Example pairing:
  `research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md` →
  `docs/gui/M7_AIRWIRE_RENDERING_GUIDANCE.md`.
- **Spec integration**: research → guidance doc → spec edit (PR).
  Do not edit `/specs/` directly from a research finding without a
  guidance-doc intermediary.
- **No attribution** in any research artifact, per
  `/CLAUDE.md § Attribution Policy`.

## Standards & Compliance Audit (8 domains)

The 2026-04-17 audit identified eight standards/compliance domains a
professional EDA tool is expected to touch. Each gets its own folder
under `/research/`. Phase 1 of the audit is a single inventory pass
(`STANDARDS_AUDIT.md`, scope-mapped against Datum's current spec); per
domain follow-on research lands as `<DOMAIN>_RESEARCH.md` once the
inventory has been triaged.

| # | Domain                                | Folder                              | Status   | Primary report                              |
|---|---------------------------------------|-------------------------------------|----------|---------------------------------------------|
| — | Audit landscape inventory             | `standards-audit/`                  | triaged  | `STANDARDS_AUDIT.md`                        |
| 1 | Data exchange & interop               | `data-exchange-interop/`            | triaged  | `DATA_EXCHANGE_INTEROP_RESEARCH.md`         |
| 2 | Component modelling                   | `component-modeling/`               | triaged  | `COMPONENT_MODELING_RESEARCH.md`            |
| 3 | Schematic & drawing conventions       | `schematic-drawing-conventions/`    | triaged | `SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md` |
| 4 | Industry-vertical compliance          | `industry-vertical-compliance/`     | triaged | `INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md`  |
| 5 | Materials & environmental             | `materials-environmental/`          | delivered | `MATERIALS_ENVIRONMENTAL_RESEARCH.md`       |
| 6 | EMC & signal integrity                | `emc-signal-integrity/`             | delivered | `EMC_SIGNAL_INTEGRITY_RESEARCH.md`          |
| 7 | PLM & lifecycle integration           | `plm-lifecycle-integration/`        | delivered | `PLM_LIFECYCLE_INTEGRATION_RESEARCH.md`     |
| 8 | Process & quality                     | `process-quality/`                  | delivered | `PROCESS_QUALITY_RESEARCH.md`               |

**Status legend**: `pending` (not started) → `in-progress` (agent
running) → `delivered` (report landed) → `triaged` (gaps lifted into
spec edits) → `integrated` (spec PRs merged).

The `standards-audit/` folder is missing from disk until the first
audit pass runs; create it on demand at that point so the empty
directory does not sit in git unloved.

## Existing Topic Research

| Topic                          | Folder                          | Primary report                                | Status     | Derived guidance                                                  |
|--------------------------------|---------------------------------|-----------------------------------------------|------------|-------------------------------------------------------------------|
| Airwire / ratsnest rendering   | `airwire-rendering/`            | `AIRWIRE_RENDERING_RESEARCH.md`               | delivered  | `docs/gui/M7_AIRWIRE_RENDERING_GUIDANCE.md`                       |
| Copper-layer rendering         | `copper-rendering/`             | `COPPER_RENDERING_RESEARCH.md`                | delivered  | `docs/gui/M7_COPPER_RENDERING_GUIDANCE.md`                        |
| IPC standards compliance       | `ipc-compliance/`               | `IPC_COMPLIANCE_RESEARCH.md`                  | triaged    | `docs/IPC_FOOTPRINT_SYSTEM.md`              |
| PCB text rendering             | `pcb-text-rendering/`           | `PCB_TEXT_RENDERING_RESEARCH.md`              | delivered  | (pending — driver: M7-IMP-014)                                    |
| Parametric DXF-native footprint editor | `parametric-footprint-editor/` | `PARAMETRIC_FOOTPRINT_EDITOR_RESEARCH.md`     | delivered  | (pending)                                                         |
| GUI visual regression          | `gui-visual-regression/`        | `GUI_VISUAL_REGRESSION_RESEARCH.md`           | delivered  | (pending)                                                         |
| GUI compound selection/editing | `gui-compound-selection/`       | `GUI_COMPOUND_SELECTION_RESEARCH.md`          | integrated | `docs/gui/DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md`           |

## Future Research (Backlog)

Topics parked here are deliberate next-after-current-work items. Add a
row when an idea surfaces that warrants research but is not yet
sequenced; promote to `Existing Topic Research` when the deep-dive
agent dispatches.

| Topic                                  | Folder (planned)                  | Trigger                                                                         |
|----------------------------------------|-----------------------------------|---------------------------------------------------------------------------------|
| _(no items currently parked)_          |                                   |                                                                                 |

## Current Standards Guidance

The current guidance bridge from standards research into controlling spec work
is:

- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `docs/STANDARDS_DOMAINS_3_4_INTEGRATION_GUIDANCE.md`

**Cross-cuts to flag during the audit**: the IPC compliance research
already touches the materials-environmental domain
(IPC-1752A material-declaration content) and the data-exchange domain
(IPC-2581 / IPC-D-356). Phase 1 audit must reconcile this overlap so
follow-on per-domain research does not duplicate IPC coverage.

## Source-Analysis Sandboxes

Symlinks into out-of-tree competitor/reference codebases studied as
part of various research efforts. Maintained at the sandbox location;
read-only from this repo's perspective.

| Symlink                | Target                                  | Used by                                                   |
|------------------------|-----------------------------------------|-----------------------------------------------------------|
| `eagle-analysis`       | `~/sandbox/eagle-analysis`              | Eagle 9.6.2 architecture study (`docs/EAGLE_BLUEPRINT.md`)|
| `horizon-source`       | `~/sandbox/horizon`                     | Horizon EDA source — the closest open-source design point; cited in airwire, copper, and IPC research |

## Adding a New Research Artifact

1. Create the folder under `/research/` (kebab-case).
2. Write the primary report as `<TOPIC>_RESEARCH.md` inside it.
3. Add a row to the appropriate table in this index.
4. When a derived guidance doc lands, fill in the `Derived guidance`
   column.
5. When spec edits land, update `Status` and reference the spec PR
   in `/docs/RESEARCH_TRACEABILITY.md`.
