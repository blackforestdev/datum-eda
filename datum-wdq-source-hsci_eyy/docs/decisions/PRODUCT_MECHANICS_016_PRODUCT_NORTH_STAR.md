# Product Mechanics Decision 016: Product North Star

> **Status**: Ratified.
> **Date**: 2026-06-29.
> **Scope**: Product identity, reference bar, and implementation priority.
> **Supersedes as priority signal**: milestone-era import/viewer framing and any
> board-first execution language not explicitly marked as compatibility work.
> **Depends on**:
> `PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY`,
> `PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM`,
> `PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE`,
> `PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE`,
> `PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR`.

## Decision

Datum is a native professional EDA system. Its product center is:

1. governed native libraries and component identity;
2. schematic capture as the normal authored electrical source of truth;
3. explicit schematic-to-PCB implementation through stable identities,
   constraints, checks, and reviewable ECO/proposal flows;
4. manufacturing-ready PCB/CAM output from the resolved native design model;
5. optional first-class AI assistance over the same deterministic primitives
   available to manual GUI, CLI, and API users.

KiCad, Eagle, Altium, Cadence, Siemens, Zuken, vendor libraries, and generated
files are interoperability inputs or comparison references. They are not Datum's
native architecture and must not define the roadmap center.

## North Star

Datum should be judged against professional commercial EDA expectations, not
against hobby/open-source import viewers. The bar is the class of capability a
serious user expects from tools such as Altium Designer, Cadence/OrCAD/Allegro,
Siemens Xpedition/PADS, and Zuken:

- coherent component/library lifecycle and reuse;
- schematic-first electrical intent with hierarchy, variants, and ERC;
- constraint-driven PCB implementation and DRC;
- traceable schematic-to-PCB identity, ECO, and back-annotation;
- manufacturing outputs with auditable process and standards basis;
- project governance, provenance, review, and reproducibility.

Open-source projects remain useful as cautionary evidence and migration targets.
Horizon EDA is important prior art because it recognized that weak library
management corrupts the entire EDA workflow. KiCad is important because its
loose symbol/footprint/design-level binding exposes failure modes Datum must not
copy. Neither defines the target quality bar.

## Implementation Priority

Until the foundation is real, work proceeds in this order:

1. **Native library substrate**: component, symbol, package/footprint,
   padstack, pin/pad map, part metadata, lifecycle, provenance, standards basis,
   project-local overrides, and revisioned bindings.
2. **Schematic capture substrate**: sheets, symbols, wires, labels, ports,
   hierarchy, buses, junctions, no-connects, net derivation, annotations, ERC,
   and manual editing.
3. **Schematic-to-PCB contract**: stable `ComponentInstance`, `NetId`, rules,
   constraints, variants, ECO/proposal flows, and relationship-state handling.
4. **PCB layout/editor**: placement, routing, zones, constraints, DRC, board
   geometry, and physical implementation of schematic intent.
5. **Manufacturing/CAM**: outputs, validation, live production views,
   panelization, standards reports, and revisioned export artifacts.
6. **Import/export interop**: conversion, migration, source provenance, repair
   proposals, and compatibility output.

GUI, CLI, MCP, and AI work may continue only when it advances or protects this
sequence. A GUI shell or imported-board review surface is infrastructure, not
proof that Datum has an EDA editor.

## Product Rules

- A manual user must be able to complete core library, schematic, PCB, checking,
  and manufacturing workflows without AI.
- AI agents may assist, propose, automate, and review, but they must not own
  private edit paths or become required for core workflows.
- Schematic/electrical intent is normally authored before PCB implementation.
  Board-first and reverse-engineering workflows are supported through explicit
  relationship/provenance states, not by weakening schematic authority.
- Libraries are native governed product data. Imported symbols, footprints, and
  vendor data are conversion inputs with provenance and unknown-basis markers.
- Import fidelity cannot be the active maturity metric while native library and
  schematic authoring remain incomplete.
- Compatibility work must say what native primitive, migration path, standards
  audit, or proof fixture it enables.

## Governance

Future specs, roadmap updates, and agent plans must preserve this vocabulary:

- **Product identity**: professional native EDA system.
- **Foundation**: governed library plus schematic authority.
- **Normal flow**: library -> schematic -> PCB -> manufacturing.
- **Interop role**: import/export are compatibility and migration paths.
- **AI role**: optional first-class collaborator over deterministic primitives.

Any document that frames Datum primarily as a KiCad importer, imported-board
viewer, board-first editor, or AI-only design agent must either be historical
evidence or explicitly mark the wording as non-goal/compatibility scope.

## Completion Gate For This Decision

This decision is satisfied when:

1. the roadmap and progress tracker name library/schematic foundation as the
   next product-driving work;
2. library and schematic contracts are treated as implementation drivers, not
   abandoned research artifacts;
3. a drift gate prevents unqualified product-identity regressions in the primary
   docs/specs;
4. the next production code goal begins from native library/schematic substrate
   rather than imported-board fidelity or board-editor UI alone.

