# Research Traceability

> **Status**: Non-normative governance artifact.
> This document links research conclusions to roadmap sequencing and current
> implementation status. Formal contracts remain in `specs/`.
> Scope terminology follows `specs/PROGRAM_SPEC.md` §Scope Integrity Terms.

## Purpose

Prevent strategy drift by making one thing explicit:
- what research concluded
- where each conclusion appears in the roadmap
- whether it is implemented, in progress, or intentionally deferred
- why deferment exists (when applicable)

## Source Corpus

Primary research source for this mapping:
- `/home/bfadmin/sandbox/eagle-analysis/ARCHITECTURE.md`
- `docs/gui/REFERENCE_STUDY.md`
- `docs/gui/FRONTEND_DECISION_CRITERIA.md`
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/M7_DECISION_PROPOSALS.md`

Primary roadmap/spec anchors:
- `PLAN.md`
- `specs/PROGRAM_SPEC.md`

## Traceability Matrix

| Research conclusion | Roadmap/spec anchor | Current status | Deferment rationale |
|---|---|---|---|
| Build an AI-native, Linux-first EDA system (not a re-skin of legacy tools). | `PLAN.md` Mission Layers (`Product identity`), `specs/PROGRAM_SPEC.md` `Product identity` | Active | N/A |
| Engine-first architecture with machine-native interfaces. | `PLAN.md` Mission Layers, `specs/PROGRAM_SPEC.md` Scope Integrity Terms | Active | N/A |
| Strong command interface as a first-class surface. | `M2` CLI + MCP surfaces, later AI intent layers (`M6`) | Active (CLI/MCP), expanded later (AI intent) | Natural-language strategy layer depends on mature core semantics. |
| API-first: operations should be programmatically available. | `M2` read/check catalog, `M3` write operations, `M4+` authoring ops | In progress | Full write/authoring surface follows operation-model maturity. |
| Proper part model and structured component/pool relationships. | `M0` pool foundation + `IMPORT_SPEC` mapping rules | Active, expanding | Broader library ecosystems staged after core fidelity gates. |
| Integrated checking (ERC/DRC) as core behavior, not bolt-on. | `M2` exit criteria and check catalogs | Implemented for current slice | Rule depth and cross-domain sophistication continue after `M2`. |
| Structured canonical data model for deterministic automation. | `M0` canonical IR + deterministic serialization; `M4` native format | Active (in-memory canonical IR), native persistence deferred | Native project format is intentionally sequenced after stable read/check/write slices. |
| Broad interoperability (`Import everything`: open + commercial ecosystems). | `M0/M1` open-format interop, `R1` commercial interop research, later implementation milestones | Staged | Commercial paths are high-cost and sequenced behind proven core execution. |
| Advanced routing kernel (push-shove sophistication, geometry-heavy layout). | `M5` deterministic layout kernel | Deferred | Requires mature operations, constraints, and recomputation infrastructure first. |
| AI-assisted placement/routing strategy layer. | `M6` layout strategy + AI layer | Deferred | Depends on stable baseline layout kernel from `M5`. |
| GUI as a consumer, not architecture driver. | `M7` GUI + review interface; engine-first guardrails in implementation docs | Planned and consistent | GUI sequencing protects core determinism and API-first design. |
| Frontend architecture should be chosen against product-level criteria, not convenience-tooling defaults. | `docs/gui/FRONTEND_DECISION_CRITERIA.md`; later frontend foundation and `M7` planning docs | Active research guidance | Prevents a narrow first viewer from locking in the wrong long-term frontend shape. |
| Professional GUI direction should be grounded in Altium plus OrCAD-family and PADS/Xpedition workflow lessons where relevant, use Blender as a custom-application architecture/interaction reference rather than an EDA UX template, and treat KiCad as contrast and ecosystem context rather than the product ceiling. | `docs/gui/REFERENCE_STUDY.md`, `docs/gui/FOUNDATION.md`, `docs/gui/WORKSPACE_MODEL.md`, `docs/gui/INTERACTION_MODEL.md`, `docs/gui/CANVAS_REVIEW_MODEL.md`, `docs/gui/VISUAL_LANGUAGE.md`, `docs/gui/TECHNICAL_PRINCIPLES.md`, `docs/gui/M7_DECISION_PROPOSALS.md`, `specs/M7_FRONTEND_SPEC.md` | Active frontend foundation and planning guidance | The frontend direction is now concretized in repo docs and the opening `M7` spec, reducing the risk of generic or ad hoc GUI decisions. |
| Frontend must reserve first-class space for graphical, terminal, and AI lanes without creating parallel design truth. | `docs/gui/FOUNDATION.md`, `docs/gui/WORKSPACE_MODEL.md`, `docs/gui/INTERACTION_MODEL.md`, `docs/gui/TECHNICAL_PRINCIPLES.md` | Active foundation guidance | The shell should support command and AI workflows natively while preserving engine authority and deterministic action boundaries. |
| The opening `M7` slice should make explicit human-review UI decisions before coding, grounded in professional tool patterns rather than generic app defaults. | `docs/gui/M7_DECISION_PROPOSALS.md`; `specs/M7_FRONTEND_SPEC.md` | Active planning guidance | Prevents the architecture spike from drifting into ad hoc layout, panel, and visual-state decisions. |
| Python/scriptability in the ecosystem. | Current Python MCP host (`mcp-server/`), deeper scripting pathways later | Partially active | Rich scripting API surface expands with editor/layout maturity. |

## Sequencing Integrity Notes

- The roadmap intentionally uses **interop-first execution strategy** in early
  milestones (`M0-M2`) to validate core semantics on real designs.
- This is a sequencing choice, not a product-identity claim.
- Product identity remains AI-native platform architecture, with interop as an
  adoption and validation path.

## Change Control

When adding or changing milestones, update this document if:
- a research conclusion is newly implemented
- a deferment rationale changes
- a conclusion is explicitly descoped or replaced

If a roadmap/spec statement appears to conflict with this matrix:
- specs remain authoritative for implementation contracts
- raise a scope-integrity correction in the relevant spec/plan doc
