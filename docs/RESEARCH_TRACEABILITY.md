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

## Enforced evidence routes

`specs/evidence_traceability_manifest.json` is the machine-checked reverse
index for this human synthesis. It covers every tracked `research/**/*.md` and
`docs/gui/prototypes/*.html` artifact exactly once, names the governed specs and
decisions that consume each evidence set, and records a digest over both sides.
`scripts/check_evidence_traceability.py`, wired into the standing drift gates,
fails when an artifact is orphaned, a consumer is ungoverned or missing, or
either evidence or specification changes without an explicit route review.

This prevents implementation state, agent memory, or a newer isolated draft
from silently displacing already-ratified decisions and visual sources of
truth. The prose matrix below remains useful synthesis; the manifest supplies
coverage, reverse routing, and freshness enforcement.

## Source Corpus

Primary research source for this mapping:
- `/home/bfadmin/sandbox/eagle-analysis/ARCHITECTURE.md`
- `research/standards-audit/STANDARDS_AUDIT.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
- `docs/gui/REFERENCE_STUDY.md`
- `docs/gui/FRONTEND_DECISION_CRITERIA.md`
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/M7_DECISION_PROPOSALS.md`
- `research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md`
- `research/application-status-bar/APPLICATION_STATUS_BAR_RESEARCH.md`
- `research/selection-visual-language/SELECTION_VISUAL_LANGUAGE_RESEARCH.md`
- `research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md`
- `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md`
- `research/documentation-system/REV_C01_INTERNAL_AUTHORITY_AUDIT.md`
- `research/documentation-system/PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md`
- `research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`
- `research/documentation-system/REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md`
- `research/documentation-system/REV_C03_REVISION_IDENTITY_DECISION_PACKET.md`
- `research/documentation-system/REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md`
- `research/documentation-system/REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md`
- `research/documentation-system/REV_C06_UX_VISUAL_STUDY_BRIEF.md`

Primary roadmap/spec anchors:
- `PLAN.md`
- `specs/PROGRAM_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`

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
| Datum's embedded terminal must be a fully fledged daily-driver terminal—not a console lane or partial emulator—and must run arbitrary shells, TUIs, Codex, Claude Code, Cursor-compatible CLI agents, and local agents through normal PTY semantics plus native client discovery of governed Datum CLI/MCP, pinned context, scoped authority, and portable workflows. Its terminal semantics and Linux PTY/session transport are clean-room Datum-owned Rust: Zig supplied no runtime behavior, Ghostty was never linked, and no external terminal implementation or fallback is permitted. | `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`; `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`; `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`; `docs/decisions/PRODUCT_MECHANICS_030_DATUM_TERMINAL_CORE_ARCHITECTURE.md`; `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`; `docs/gui/DATUM_TERMINAL_CORE_RESEARCH.md`; `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`; `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`; Active Frontier T0–T4e | Ratified product, interoperability, dependency-authority, and first-party-core contracts; DTC-P00..P02 planning is complete and execution begins with the owned PTY package DTC-P03 | Standards-first PTY, parser/state, Unicode, history/reflow, selection/search, protocols, scratch codecs/graphics, renderer/input/accessibility, launcher/discovery, MCP, context/authority, workflow parity, and production acceptance are explicit DTC-P03..P30 and T4 tracked slices. |
| The opening `M7` slice should make explicit human-review UI decisions before coding, grounded in professional tool patterns rather than generic app defaults. | `docs/gui/M7_DECISION_PROPOSALS.md`; `specs/M7_FRONTEND_SPEC.md` | Active planning guidance | Prevents the architecture spike from drifting into ad hoc layout, panel, and visual-state decisions. |
| Python/scriptability in the ecosystem. | Current Python MCP host (`mcp-server/`), deeper scripting pathways later | Partially active | Rich scripting API surface expands with editor/layout maturity. |
| Research findings about standards and compliance must not remain implicit; each researched standard family needs a controlling disposition. | `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`; `specs/STANDARDS_COMPLIANCE_SPEC.md`; `specs/PROGRAM_SPEC.md` `Standards And Compliance Governance` | Active governance contract | Prevents future spec drift where research exists but no controlling spec states whether Datum implements, defers, references, or excludes the standard. |
| Datum needs first-class IPC footprint-basis ownership rather than loose "IPC-aware" claims. | `docs/IPC_FOOTPRINT_SYSTEM.md`; `specs/STANDARDS_COMPLIANCE_SPEC.md` `Footprint And Library Contracts` | Planned in controlling spec | The library and footprint subsystem must preserve source assumptions, deviation tracking, and import-audit behavior before stronger IPC claims are credible. |
| Native schematic authoring needs an explicit standards policy for symbol style, designators, and title-block metadata. | `research/standards-audit/STANDARDS_AUDIT.md` Domain 3; `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`; `specs/STANDARDS_COMPLIANCE_SPEC.md` `Schematic And Capture Contracts` | Planned in controlling spec | This closes the blind spot where imported symbols inherit style but native symbols have no stated compliance posture. |
| Multi-selection must become a typed compound Inspector subject with explicit Common/`Mixed`/Unavailable fields, per-type scopes, exact affected counts, and atomic refusal; persistent groups, universal locks, and specialized electrical/manufacturing edits require honest substrate/domain boundaries. | `research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md` → `docs/gui/DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md` → `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2 | Research integrated into the in-progress S5 specification; implementation not authorized | S5A selection/inspection precedes S5B group/lock/batch authority; topology-, rule-, library-, variant-, manufacturing-, and hierarchy-sensitive edits remain dedicated later tools until their typed contracts land. |
| Consequential and fast-changing editor feedback must stay near the engaged pane/object; a global bottom strip is defensible only for calm global state and cannot be the sole acknowledgement of selection, snap, refusal, invalid placement, or check failure. | `research/application-status-bar/APPLICATION_STATUS_BAR_RESEARCH.md` → `docs/gui/DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md` → `docs/gui/DATUM_GUI_DESIGN_SPEC.md` / `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` / `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §7 | Research delivered; Application Status Bar retention and contents reopened for owner review | Existing prototypes/runtime mix global, focused-pane, and pointer-local state. Owner must choose remove versus minimal calm retained bar before field ownership or S8 readout implementation. |
| S5 selection must preserve Datum's `#CE5A92` whole-object soft-glow identity while becoming a typed immediate overlay for every honestly supported PCB/schematic class; retained-world recoloring, incomplete state precedence, and unbounded compound/net rendering cannot remain implicit. | `research/selection-visual-language/SELECTION_VISUAL_LANGUAGE_RESEARCH.md` → `docs/gui/DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md` → `docs/gui/DATUM_RENDERING_BOOK.md` / `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2 | Visual forks locked; exhaustive matrix, functional reconciliation, evidence, owner final review, and selection-identity ratification remain under `dat-s5-selection-visual-contract-zid` | Rendering Book owns the locked visual rules; the stable S5-C01..C13 plan owns remaining closure work. The separate Application Status Bar decision remains deferred. |
| Datum's scalable Project, continuous Design Space, universal Publish Space, and documentation object model must be derived from recorded owner intent plus internal and external research rather than inherited page-based EDA assumptions. | `research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md` → `docs/gui/prototypes/workspace-panes.html` + `docs/gui/prototypes/publish-space-study.html` → `specs/PUBLISH_SPACE_SPEC.md` → `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` | DOC-C01..C05 are complete and owner-approved; DOC-C06 integrates the governed Publish contract and selects Product Revision Engine specification next; no Publish implementation is authorized | The controlling contract specifies stable Publish objects, typed operations, projection/write fences, ConfigurationRef integration, conformance obligations, the narrow schematic proof, and the approved visual dispositions. Distributed collaboration and verified redaction retain separate mandatory re-entry tracks. |
| Engineering revision must govern complete product configurations and controlled documents without equating Git commits, technical model revisions, or title-block counters with approved releases. | `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md` + `REV_C01_INTERNAL_AUTHORITY_AUDIT.md` + `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md` + `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md` + `REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md` + `REV_C03_REVISION_IDENTITY_DECISION_PACKET.md` + `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md` + `REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md` + `REV_C06_UX_VISUAL_STUDY_BRIEF.md` + `docs/gui/prototypes/revision-identity-study.html` + `docs/gui/prototypes/revision-authority-study.html` → `PRODUCT-REVISION-SPEC` / `dat-product-revision-engine-k9f` → future governed decision/specification | REV-C01 factual baseline, REV-C03 Q1–Q6, and all six revision-identity decisions are owner-approved; REV-C02 supplies standards evidence and the split V1–V10 prototypes are reviewed; REV-C03 is complete; REV-C04/REV-C05 supply candidate offline, adapter, impact, freshness, regeneration, comparison, and reproducibility contracts; REV-C06 now has a bounded Claude-owned visual-study brief; no implementation is authorized | Datum uses one profile-driven Revision Engine. Lightweight post-release divergence automatically opens identity-free Draft successor Change work; explicit preparation later enters V2-P. Sequential Alphanumeric is the factory preference; ISO 19650 keeps revision/status separate in V3d title-block cells. V8-A makes revision-bearing CIs the sole EngineeringRevision owners; baselines own composition and one Release may issue several affected CIs' independent revisions. V9-A separates mutable Publish Sets from CI-designated ControlledDocuments and immutable DocumentIssues. V10-C stores typed standing facts. REV-C04 requires complete local authority and keeps Git optional/non-authoritative. REV-C05 distinguishes changed, affected, stale, orphaned, standing, and non-reproducible facts and requires byte-identical release reproduction. REV-C06 requires twelve shell/workflow/accessibility frames before six one-at-a-time owner questions may be presented. |

## Sequencing Integrity Notes

- The roadmap intentionally uses **interop-first execution strategy** in early
  milestones (`M0-M2`) to validate core semantics on real designs.
- This is a sequencing choice, not a product-identity claim.
- Product identity remains AI-native platform architecture, with interop as an
  adoption and validation path.
- Standards and compliance work follows the same rule: explicit deferment is
  acceptable, implicit absence is not.

## Change Control

When adding or changing milestones, update this document if:
- a research conclusion is newly implemented
- a deferment rationale changes
- a conclusion is explicitly descoped or replaced

If a roadmap/spec statement appears to conflict with this matrix:
- specs remain authoritative for implementation contracts
- raise a scope-integrity correction in the relevant spec/plan doc
