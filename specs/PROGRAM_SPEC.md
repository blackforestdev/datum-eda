# Program Specification

Status: governed ownership and standards contract; original milestone/v1
planning is historical. This entry identifies document ownership, not delivered
capability or execution permission.

> **RECONCILIATION (2026-07-06).** This file is the *legacy milestone master*
> (M0–M4, R1) from **before** the product-mechanics course correction. It is
> **not** the current roadmap authority and is **not** in the CLAUDE.md authority
> ordering (CLAUDE.md → product-mechanics + `docs/decisions/` → `docs/contracts/`
> → `specs/PROGRESS.md`); for scope and sequencing it is subordinate to all four.
>
> - **HISTORICAL (original plan, not current scope):** the "v1 Definition"
>   (including *"v1 does not: create designs, edit designs, route, export
>   manufacturing files, or provide a GUI"*) and the "Milestone Exit Criteria"
>   (M0–M4). Native authoring, export, and the GUI are now **active** work — see
>   `docs/decisions/` and the Active Frontier at the top of `specs/PROGRESS.md`.
>   Do not cite these sections as current product limits.
> - **STILL LIVE AND GOVERNED:** the *subordinate-spec ownership map* below (which
>   formal spec owns which contract) and the *Standards And Compliance Governance*
>   terms + the *Standards Integration Gate*. These remain in force.
>
> For "what is current" and "what is next," read the Active Frontier in
> `specs/PROGRESS.md` and the decision records — never infer current scope from
> the milestone/v1 sections here.

## Ownership and Precedence

This is the existing entry point for formal domain specification ownership.
Product doctrine and domain tool contracts remain controlling; each document
below owns its detailed schemas and interface requirements within that authority.
All nine are classified `governed` in the existing governance manifest. That
classification is a requirements status, not a delivery or qualification claim.

| Subject | Detailed specification owner |
|---|---|
| Engine types, invariants, operations and API | [ENGINE_SPEC.md](ENGINE_SPEC.md) |
| Native persistence, schema and migration | [NATIVE_FORMAT_SPEC.md](NATIVE_FORMAT_SPEC.md) |
| Import identity, mapping and lossiness | [IMPORT_SPEC.md](IMPORT_SPEC.md) |
| Standards dispositions, waivers and deviations | [STANDARDS_COMPLIANCE_SPEC.md](STANDARDS_COMPLIANCE_SPEC.md) |
| Schematic connectivity and hierarchy | [SCHEMATIC_CONNECTIVITY_SPEC.md](SCHEMATIC_CONNECTIVITY_SPEC.md) |
| Electrical rule semantics | [ERC_SPEC.md](ERC_SPEC.md) |
| Shared checking and report contracts | [CHECKING_ARCHITECTURE_SPEC.md](CHECKING_ARCHITECTURE_SPEC.md) |
| Authored schematic objects and operations | [SCHEMATIC_EDITOR_SPEC.md](SCHEMATIC_EDITOR_SPEC.md) |
| MCP transport, payloads and errors | [MCP_API_SPEC.md](MCP_API_SPEC.md) |

For product behavior, start with [product mechanics](../docs/DATUM_PRODUCT_MECHANICS.md),
[ratified decisions](../docs/decisions/) and [domain tool contracts](../docs/contracts/).
For integration clauses, use [INTEGRATED_PROGRAM_SPEC.md](INTEGRATED_PROGRAM_SPEC.md);
its full-spec promotion is unfinished. For completion requirements, use
[FOUNDATION_CONSUMER_COMPLETION_PLAN.md](FOUNDATION_CONSUMER_COMPLETION_PLAN.md)
and the selected task's governing documents. Scheduling/grants come from the
[structured Frontier](active_frontier.json), with the generated human view in
[PROGRESS.md](PROGRESS.md). None of these links selects or authorizes work.

Precedence rules:
- CLAUDE.md controls operational instructions; product mechanics and ratified
  decisions control product doctrine, followed by domain tool contracts.
- Formal domain and integration specifications implement that inherited
  authority. Directory placement does not make a spec override doctrine or a
  domain tool contract. On conflict, reconcile the lower document.
- docs/CANONICAL_IR.md retains canonical IR ownership until formally promoted
  or subsumed. Research/design rationale is distinct from controlling doctrine.

Status tracking rule:
- `specs/PROGRESS.md` is the sole source of truth for implementation status.
- This file owns the subject map and permanent standards requirements; original
  milestone contracts and non-goals are retained as linked historical records.
- `specs/PROGRESS.md` should remain a compact milestone/checkpoint/frontier
  tracker; slice-by-slice ledgers and detailed adjacent-contract history belong
  in milestone shard docs such as `specs/progress/*.md`.

## Current PM061 CLI inventory reconciliation

The implemented `datum-eda project process <root>` command groups manual profile,
basis and physical-face association authoring with effective-process inspection.
The existing reviewed component-replacement proposal carries process-update intent.
The library-authoring contract and PM061 govern semantics and qualification;
this inventory note changes no historical milestone or successor authority.

## Scope Integrity Terms

The following terms are normative for scope framing across this repository:

- **Product identity**: Datum EDA as an AI-native EDA platform.
  This is cross-milestone and does not shrink to current feature coverage.
- **Implementation slice**: the currently shipped capability subset at a given
  milestone boundary (for example `M2` import/query/check, current `M3` write
  subset).
- **Execution strategy**: sequencing choices used to deliver the implementation
  slice with low risk (for example KiCad-first interoperability).
- **Non-goals**: explicit exclusions for a milestone, used to control delivery
  scope. Non-goals for one milestone are not permanent product exclusions.

Interpretation rule:
- Do not infer product identity from implementation-slice limits.
- Do not infer permanent product limits from milestone non-goals.

## Standards And Compliance Governance

Datum's research corpus includes standards findings that must not remain
implicit. A researched standard is considered integrated only when a
controlling spec assigns it one of the following dispositions:

- implemented
- planned
- reference-only
- deferred with explicit prerequisite
- out of scope

Silence is not an acceptable disposition.

The controlling source for this policy is
`specs/STANDARDS_COMPLIANCE_SPEC.md`. Research and guidance inputs feeding that
spec currently include:

- `research/standards-audit/STANDARDS_AUDIT.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`

Minimum program-level requirement:
- milestone planning and milestone closure must not introduce or preserve a
  standards blind spot already identified in repository research without an
  explicit disposition recorded in the controlling standards-compliance spec.

---

## Standards Integration Gate (Cross-Milestone)

This is a permanent governance gate, not a standalone delivery milestone.

Required for any milestone that adds new library, schematic-authoring,
manufacturing-export, compliance-metadata, or standards-facing capability:

| Criterion | Threshold |
|-----------|-----------|
| Research disposition completeness | Every relevant researched standard family referenced by the change is classified in `specs/STANDARDS_COMPLIANCE_SPEC.md` as implemented, planned, reference-only, deferred with prerequisite, or out of scope |
| Guidance intermediary | New standards research is lifted through a guidance doc in `docs/` before or with the controlling spec edit |
| Cross-spec parity | If the change affects engine data, project persistence, import/export, checking, or MCP/CLI exposure, the relevant domain specs are updated in the same change |
| No silent compliance claims | User-visible wording and milestone text must not imply support for a standard that is only vocabulary-level or partially scaffolded |
| Deferment boundedness | Any deferred standard-facing capability must record the prerequisite that gates it |

---

## Historical program context

The links below preserve original anchors; current standards requirements are
above and current product/scheduling authority is identified at the entry.

## v1 Definition

Historical record: [v1 Definition](../docs/reviews/specification-history/program-milestones.md#v1-definition).

## Milestone Exit Criteria

Historical record: [Milestone Exit Criteria](../docs/reviews/specification-history/program-milestones.md#milestone-exit-criteria).

### M0: Canonical IR + Foundation

Historical record: [M0: Canonical IR + Foundation](../docs/reviews/specification-history/program-milestones.md#m0-canonical-ir--foundation).

### M1: Design Ingestion + Query Engine

Historical record: [M1: Design Ingestion + Query Engine](../docs/reviews/specification-history/program-milestones.md#m1-design-ingestion--query-engine).

### M2: ERC + DRC + Reporting + MCP/CLI (v1)

Historical record: [M2: ERC + DRC + Reporting + MCP/CLI (v1)](../docs/reviews/specification-history/program-milestones.md#m2-erc--drc--reporting--mcpcli-v1).

### R1: Commercial Interop Research Track

Historical record: [R1: Commercial Interop Research Track](../docs/reviews/specification-history/program-milestones.md#r1-commercial-interop-research-track).

### M3: Write Operations on Imported Designs

Historical record: [M3: Write Operations on Imported Designs](../docs/reviews/specification-history/program-milestones.md#m3-write-operations-on-imported-designs).

### M4: Native Project Creation + Export

Historical record: [M4: Native Project Creation + Export](../docs/reviews/specification-history/program-milestones.md#m4-native-project-creation--export).

## User Stories

Historical record: [User Stories](../docs/reviews/specification-history/program-milestones.md#user-stories).

### M2 (v1) User Stories

Historical record: [M2 (v1) User Stories](../docs/reviews/specification-history/program-milestones.md#m2-v1-user-stories).

### M3 User Stories

Historical record: [M3 User Stories](../docs/reviews/specification-history/program-milestones.md#m3-user-stories).

### M4 User Stories

Historical record: [M4 User Stories](../docs/reviews/specification-history/program-milestones.md#m4-user-stories).
