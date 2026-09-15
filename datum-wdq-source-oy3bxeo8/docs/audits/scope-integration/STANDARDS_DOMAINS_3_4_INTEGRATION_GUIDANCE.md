# Standards Domains 3-4 Integration Guidance

> **Status**: R&D-to-spec bridge.
> **Inputs**:
> `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md`
> and
> `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md`.
> **Controlling spec**: `specs/STANDARDS_COMPLIANCE_SPEC.md`.

## Purpose

This document converts delivered Domain 3 and Domain 4 research into a
reviewable implementation order. It is intentionally a bridge document:
research findings stay in `/research/`, controlling product contracts live in
`/specs/`, and this file explains how the findings should enter the spec stack.

The immediate goal is not to implement every feature. The immediate goal is to
stop Datum's standards posture from remaining implicit in delivered research.

## Domain 3 Decision

Domain 3 covers schematic and drawing conventions.

Approved target posture:

- Datum supports symbol-style profiles rather than pretending schematic symbols
  are style-neutral forever.
- Datum must support `IEEE 315`, `IEC 60617`, `JIS C 0617`, and
  `ImportedCustom` / `Mixed` profile assertions.
- Datum must not redistribute KiCad symbol libraries as built-in content.
  KiCad libraries are useful reference material and user-supplied library
  input, but Datum's bundled symbol baseline must be permissively authored or
  re-derived.
- Reference designators remain authored strings, but Datum must provide
  warn-only validation against selectable ASME Y14.44 / IEC 81346-style
  profiles.
- ISO 7200 is implemented first as structured title-block data fields plus a
  sheet-template mechanism. ISO 7200 does not define the visual frame layout.
- KiCad-style bus syntax `NAME[a..b]` is the canonical Datum-authored bus
  syntax. Other common source syntaxes are import-normalized.

## Domain 4 Decision

Domain 4 covers industry-vertical compliance.

Approved target posture:

- Datum is compliance substrate, not the certifying party.
- Project compliance posture must become first-class project metadata.
- Part qualification metadata must become first-class library metadata.
- ITAR / EAR / EU dual-use handling is primarily project metadata plus
  mandatory data-egress enforcement at the MCP boundary.
- AEC-Q is planned as part qualification metadata.
- ISO 26262, IEC 61508, IEC 60601, ISO 13485, FDA Part 820, EU MDR, CMMC,
  ISO 27001, and NIST 800-171 are reference-only or metadata-only at the
  engine layer unless promoted by a later milestone.
- 21 CFR Part 11 remains deferred until Domain 8 audit/signature primitives
  exist.
- Process-grade certifications such as DO-254, DO-160, AS9100, IATF 16949,
  CMMI, MIL-PRF-31032, MIL-PRF-55110, and NASA-STD-8739 remain out of scope
  for Datum as a product claim.

## Cross-Domain Overlap

Domain 3 and Domain 4 intentionally overlap:

- `ProjectCompliance.mandated_symbol_profile` consumes Domain 3
  `SymbolStyleProfile`.
- `ProjectCompliance.mandated_designator_profile` consumes Domain 3
  designator-profile work.
- ISO 7200 title-block fields overlap Domain 4 export-control markings,
  document classification, approver/reviewer fields, and future Domain 8
  sign-off records.
- The data-egress policy must apply to future Domain 2 supply-chain and
  model-lookup MCP tools.

This overlap should be resolved once in the schema bedrock pass, not
rediscovered independently by each later domain.

## Must-Land Before Domain 5 Research Integration

These items define the contract surface later domains will read against:

| ID | Source | Target | Requirement |
|---|---|---|---|
| D3-0 | Domain 3 | `specs/STANDARDS_COMPLIANCE_SPEC.md` | Refresh Domain 3 dispositions and make symbol profile, designator profile, title-block, and vocabulary obligations concrete. |
| D4-0 | Domain 4 | `specs/STANDARDS_COMPLIANCE_SPEC.md` | Refresh Domain 4 dispositions and make project-compliance / data-egress / part-qualification obligations concrete. |
| D3-1 | Domain 3 | `specs/ENGINE_SPEC.md` | Add shared enums for symbol style, logic form, designator profile, document status/type, and sheet size. |
| D4-1 | Domain 4 | `specs/ENGINE_SPEC.md` | Add shared enums for industry verticals, IPC class, intended environment, data-egress policy, audit overlay, safety integrity, and part qualification. |
| D3-2 | Domain 3 | `specs/ENGINE_SPEC.md` | Extend symbol model with style-profile assertion and provenance. |
| D4-2 | Domain 4 | `specs/ENGINE_SPEC.md` | Add canonical `ProjectCompliance` surface and wire it to the project model. |
| D4-3 | Domain 4 | `specs/ENGINE_SPEC.md` | Extend `Part` with `PartQualification`. |
| D3-4 | Domain 3 | `specs/ENGINE_SPEC.md` and `specs/SCHEMATIC_EDITOR_SPEC.md` | Extend `SheetFrame` with ISO 7200-aligned fields. |
| D3-5 | Domain 3 | `specs/ENGINE_SPEC.md` and `specs/SCHEMATIC_EDITOR_SPEC.md` | Add sheet size and optional sheet-template reference. |
| D3-16 | Domain 3 | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` | Declare canonical Datum bus syntax and import-normalization behavior. |
| D4-9 | Domain 4 | `specs/MCP_API_SPEC.md` | Add the Data Egress Policy gate, because later network-facing tools must not design around a missing policy. |

## Can Batch Later

These are important but do not need to block Domain 5 research integration:

| ID | Source | Target | Reason |
|---|---|---|---|
| D3-7/D3-10 | Domain 3 | Pool/native sheet-template persistence | Needed before native template authoring, but not before materials/environmental research integration. |
| D3-11 | Domain 3 | Schematic standards MCP tools | Useful API surface, but can follow schema bedrock. |
| D3-17/D3-18 | Domain 3 | Import style-profile inference | Important for import fidelity; can follow the style-profile schema. |
| D3-20/D3-22 | Domain 3 | Library/interop prose docs | Architecture documentation; not schema-blocking. |
| D4-5/D4-7 | Domain 4 | Pool/native part-qualification persistence | Needed for implementation; can follow schema acceptance. |
| D4-8 | Domain 4 | Compliance MCP tools | Useful query surface; can follow `ProjectCompliance` and `PartQualification`. |
| D4-11/D4-12 | Domain 4 | Compliance metadata import | Imported projects should default safely, but this can follow the project schema. |
| D4-14/D4-15 | Domain 4 | Interop/library prose docs | Documentation consolidation; not schema-blocking. |

## Recommended Apply Order

1. `Pass 0`: update `specs/STANDARDS_COMPLIANCE_SPEC.md` for Domain 3 and
   Domain 4 disposition detail.
2. `Pass 1`: land shared `ENGINE_SPEC.md` schema bedrock for Domain 3 and
   Domain 4 together, resolving the `SymbolStyleProfile` /
   `ProjectCompliance` overlap once.
3. `Pass 2`: update schematic-specific specs for sheet frame, sheet size,
   sheet template reference, operations, and bus syntax.
4. `Pass 3`: update `NATIVE_FORMAT_SPEC.md` and `POOL_ARCHITECTURE.md` for
   persistence once the schema bedrock is accepted.
5. `Pass 4`: add MCP stubs and policy sections, with the Data Egress Policy
   gate taking priority over general query tools.
6. `Pass 5`: update import, library, and interop docs.

## Non-Goals

- Do not implement certification claims.
- Do not copy KiCad symbol libraries into Datum.
- Do not make symbol-style profile a project-only hard rule.
- Do not block materials/environmental research on sheet-template rendering.
- Do not implement 21 CFR Part 11 signatures before Domain 8 audit primitives
  are specified.
- Do not silently apply compliance metadata to imported projects.

## Completion Criteria

The Domain 3 and Domain 4 research is considered spec-integrated when:

- `STANDARDS_COMPLIANCE_SPEC.md` has concrete Domain 3 and Domain 4 target
  fields and dispositions.
- `ENGINE_SPEC.md` owns the schema surfaces required by both domains.
- Schematic specs own the sheet/template/bus policy surfaces.
- `MCP_API_SPEC.md` owns the data-egress gate before any network-facing
  compliance or supply-chain tools are expanded.
- `research/README.md` links the guidance and tracks each report's integration
  status.
