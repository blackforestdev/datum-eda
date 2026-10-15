# Standards Audit Batch 1 Integration Guidance

> **Status**: Active guidance for the first batch of spec edits derived
> from the Phase 2 standards-audit deep-dives on Domain 1 (Data exchange
> & interop) and Domain 2 (Component modelling).
>
> **Research basis:**
> - [STANDARDS_AUDIT.md](../research/standards-audit/STANDARDS_AUDIT.md) — Phase 1 landscape inventory
> - [DATA_EXCHANGE_INTEROP_RESEARCH.md](../research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md) — Domain 1 deep-dive
> - [COMPONENT_MODELING_RESEARCH.md](../research/component-modeling/COMPONENT_MODELING_RESEARCH.md) — Domain 2 deep-dive
>
> **Sibling guidance:**
> - [STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md](STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md) — meta-rules for research → spec integration
> - [IPC_FOOTPRINT_SYSTEM.md](IPC_FOOTPRINT_SYSTEM.md) — IPC-derived guidance

## Purpose

Consolidate the 16 must-land spec edits surfaced by the Domain 1 and
Domain 2 deep-dives into one bridging document. This guidance fixes:

- the **must-land vs deferred** categorisation (16 of 19 in this batch)
- the **apply order** that minimises file re-reads and keeps schema
  bedrock first
- the **`ModelProvenance` overlap resolution** between Domain 1 and
  Domain 2 (one-time consolidation)
- the **STANDARDS_COMPLIANCE_SPEC.md disposition updates** required by
  that spec's § 9 Cross-Spec Update Rule

This batch lands on `feature/standards-audit-batch-1` and ships as a
single PR.

## Scope

### In scope (16 must-land edits)

The 16 edits define contract surfaces, schema ownership, persistence,
import behaviour, MCP intent, and library scope that the remaining
Phase 2 deep-dives (Domains 3–8) will read against. Without these
contracts in place, downstream domains either re-propose the same
substrate or make weaker recommendations against a stale surface.

| # | Source | Target file | Substance |
|---|---|---|---|
| **Pass 0 — STANDARDS_COMPLIANCE_SPEC.md disposition refresh** ||||
| 0a | this batch | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.1 | Domain 1 dispositions refreshed: STEP/IDF/IDX/EDMD/DXF prerequisites delivered, Gerber X3 / IPC-2581C / IPC-D-356 / ODB++ contracts framed |
| 0b | this batch | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.2 | Domain 2 dispositions refreshed: IBIS/Touchstone/SPICE attachment surface defined, encrypted-content policy noted |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** ||||
| D2-1 | Domain 2 | `specs/ENGINE_SPEC.md` § 1.1a | `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelAttachment`, `ModelProvenance`, `ModelFormatMetadata` |
| D1-2 | Domain 1 | `specs/ENGINE_SPEC.md` § 1.1a + § 1.2 | `ModelFormat` enum, typed `Transform3D`, expanded `ModelRef`, `Package.body_height_nm` / `body_height_mounted_nm` |
| D1-3 | Domain 1 | `specs/ENGINE_SPEC.md` § 1.3 | `StackupLayer` material fields (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`) |
| D1-4 | Domain 1 | `specs/ENGINE_SPEC.md` § 1.3 | `Net.controlled_impedance: Option<ImpedanceSpec>`, `ImpedanceSpec` |
| D2-2 | Domain 2 | `specs/ENGINE_SPEC.md` § 1.2 | `Part` extended: `manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, `last_supply_chain_check`, plus `ThermalSpec`, `PackagingKind`, `PackagingOption`, `SupplyOffer` |
| D2-4 | Domain 2 | `specs/ENGINE_SPEC.md` § 3 | `AttachModel` and `DetachModel` operations with `inverse()` reversibility |
| **Pass 2 — pool & native persistence** ||||
| D2-3 | Domain 2 | `docs/POOL_ARCHITECTURE.md` § 2 | `pool/models/{ibis,spice,touchstone,ami,thermal}/` directory; `models` and `part_model_attachments` SQL index tables |
| D2-9 | Domain 2 | `specs/NATIVE_FORMAT_SPEC.md` § 4 + new § 6.x | `pool/models/` added to project layout; new § 6.x "Pool Model Files" schema |
| **Pass 3 — `specs/MCP_API_SPEC.md` (stubs and headings only)** ||||
| D1-5 | Domain 1 | `specs/MCP_API_SPEC.md` new "M7+ Export Tools" | Section header + per-tool stubs for `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline` |
| D2-5 | Domain 2 | `specs/MCP_API_SPEC.md` new "Component Modelling Tools (M7+)" | Section header + per-tool stubs for `attach_ibis`, `attach_touchstone`, `attach_spice`, `validate_*`, `extract_*`, `export_spice_netlist`, `lookup_part_*`, `refresh_supply_chain`, `find_alternate_parts`, `query_packaging_options`, `normalize_manufacturer`, `infer_diffpair_from_pinnames` |
| D2-6 | Domain 2 | `specs/MCP_API_SPEC.md` new top-level | "Encrypted Content Handling Policy" section heading + minimum contract framing |
| **Pass 4 — `specs/IMPORT_SPEC.md`** ||||
| D1-7 | Domain 1 | `specs/IMPORT_SPEC.md` new § 5 | "IPC-2581 Import (Future — Post-M7)" section: rationale + feature matrix |
| D2-8 | Domain 2 | `specs/IMPORT_SPEC.md` § 3 + § 4 | KiCad and Eagle import matrices: SPICE / IBIS / Touchstone rows promoted from "Deferred" to "Best-effort (M7+)" with `Part.behavioural_models` mapping note |
| **Pass 5 — architecture & scope docs** ||||
| D2-7 | Domain 2 | `docs/LIBRARY_ARCHITECTURE.md` after § "Canonical Datum Library Model" | New "Behavioural model attachment" subsection |
| D1-1 | Domain 1 | `docs/INTEROP_SCOPE.md` § Future (M5+) | Re-organise into Hard / Should / On-demand / Out-of-scope buckets per Domain 1 research |
| D2-10 | Domain 2 | `docs/INTEROP_SCOPE.md` (new section) | "Behavioural model attachment & export" Hard / Should / On-demand / Out-of-scope buckets |

### Out of scope (3 deferred edits — held for a later batch)

Pure marketing/conventions edits with no contract-surface impact. They
will be batched after Domain 8 with the rest of the marketing/scope-doc
consolidation.

| # | Source | Target file | Why deferred |
|---|---|---|---|
| D1-6 | Domain 1 | `specs/NATIVE_FORMAT_SPEC.md` § 12 (or `docs/POOL_ARCHITECTURE.md`) | `.gitignore` / `.gitattributes` conventions. Descriptive; doesn't change spec contracts. |
| D1-8 | Domain 1 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` § 10 | "Datum's Open-Stack Position" appendix. Marketing position. |
| D2-11 | Domain 2 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` | "Behavioural Model Stack — Open-Stack Position" appendix. Marketing position. |

## ModelProvenance Overlap Resolution (One-Time)

Both Domain 1 (D1-2 `ModelRef` extension) and Domain 2 (D2-1
`ModelAttachment`) carry per-attachment provenance metadata. To prevent
two parallel provenance shapes, the batch consolidates a single
`ModelProvenance` struct that both attachment kinds reference.

**Resolution (applied in Pass 1):**

```rust
pub struct ModelProvenance {
    pub source: String,                       // URL or local path of origin
    pub vendor: Option<String>,               // canonical vendor (JEP106-normalised)
    pub fetched_at: Option<DateTime<Utc>>,
    pub sha256: String,                       // identity-stable hash
}
```

`ModelProvenance` lives in § 1.1a Shared Enums and Reference Types,
adjacent to `ModelRef` and `ModelAttachment`. Both `ModelRef` (3D
geometry: STEP/WRL/IGES/OBJ/glTF) and `ModelAttachment` (behavioural:
IBIS/SPICE/Touchstone) gain an optional `provenance: Option<ModelProvenance>`
field. The same shape covers both attachment kinds; no parallel
provenance schemas land.

This is the only structural overlap between the two reports' edits.

## Apply Order

The order below is the user-approved sequence (with one refinement to
land architecture before native-format codification):

1. **Pass 0** — `specs/STANDARDS_COMPLIANCE_SPEC.md` disposition updates
   (per § 9 Cross-Spec Update Rule)
2. **Pass 1** — `specs/ENGINE_SPEC.md` schema bedrock:
   D2-1 → D1-2 → D1-3 → D1-4 → D2-2 → D2-4 (single careful pass over
   the file; `ModelProvenance` placed adjacent to D1-2 / D2-1 to
   resolve the overlap)
3. **Pass 2** — pool & native persistence:
   D2-3 (architecture concept first) → D2-9 (native-format
   codification of the same concept)
4. **Pass 3** — `specs/MCP_API_SPEC.md`:
   D1-5 → D2-5 → D2-6
   *Stubs, policy headings, and minimum contract framing only — no
   full-implementation surface*
5. **Pass 4** — `specs/IMPORT_SPEC.md`:
   D1-7 → D2-8
6. **Pass 5** — architecture & scope docs:
   D2-7 (also stages the previously-untracked
   `docs/LIBRARY_ARCHITECTURE.md`) → D1-1 → D2-10

Ordering rationale:

- **Schema bedrock first** so subsequent passes can reference real
  types when adding storage, MCP, and import contracts.
- **Architecture before native-format** (D2-3 before D2-9) per the
  user's refinement: the pool/models concept lands in the architecture
  rationale before being codified as a native-format directory
  contract.
- **MCP after pool** because some MCP tools reference pool-model UUIDs.
- **Import after MCP** because import matrices reference the
  attachment ops registered in MCP.
- **Architecture & scope last** because they consolidate references to
  all earlier passes.

## Disposition State Vocabulary (Decision A1)

This batch uses the existing 5-state vocabulary from
`STANDARDS_COMPLIANCE_SPEC.md` § 3 unchanged:

- `Implemented` / `Planned` / `Reference-only` / `Deferred with prerequisite` / `Out of scope`

Post-batch state transitions are recorded as **textual notes** under
the existing state, not as a new intermediate state. Concretely the
common Pass-0 phrasing is "**`Planned` (contract surface defined,
implementation pending)**" — this captures the post-batch state
without expanding the vocabulary.

Vocabulary stability outweighs precision here; the textual note
conveys the same information without forcing the rest of the registry
to be re-classified.

## Cross-Spec Update Rule Compliance

Per `specs/STANDARDS_COMPLIANCE_SPEC.md` § 9, any disposition
promotion must update **all** affected specs in the same change. This
batch crosses disposition thresholds in Domain 1 and Domain 2; Pass 0
provides the required SCS update. The other specs touched by this
batch (`ENGINE_SPEC`, `NATIVE_FORMAT_SPEC`, `MCP_API_SPEC`,
`IMPORT_SPEC`, `POOL_ARCHITECTURE`, `LIBRARY_ARCHITECTURE`,
`INTEROP_SCOPE`) are the same affected specs, so § 9 is satisfied
within this single batch.

Specs **not** touched in this batch (deliberately):

- `specs/PROGRAM_SPEC.md` — milestone gate language stays in your own
  milestone-tracking flow, not this contract batch
- `specs/PROGRESS.md` — same
- `specs/CHECKING_ARCHITECTURE_SPEC.md` and `specs/ERC_SPEC.md` —
  not materially affected by additive schema; the new types do not
  introduce new rule semantics

This narrowing is the user-approved Decision C interpretation.

## Acceptance Criteria

This batch is complete when:

1. All 16 must-land edits are applied to the listed target files on
   `feature/standards-audit-batch-1`.
2. `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.1 and § 4.2 reflect the
   post-batch dispositions using the existing 5-state vocabulary plus
   textual notes.
3. `ModelProvenance` is defined once in `specs/ENGINE_SPEC.md` § 1.1a
   and referenced from both `ModelRef` and `ModelAttachment`.
4. `docs/LIBRARY_ARCHITECTURE.md` exists in git (this PR is its first
   appearance) with the D2-7 behavioural-model-attachment subsection
   in place.
5. The 3 deferred edits (D1-6, D1-8, D2-11) are absent from this PR
   and queued for a later marketing/conventions batch.
6. The PR diff touches only the files enumerated above plus this
   guidance doc itself; no incidental dirty-worktree files leak in.
7. Phase-2 deep-dive briefs for Domains 3–8 can quote post-batch
   contract types (e.g. `ModelAttachment`, `ImpedanceSpec`,
   `Part.behavioural_models`) when proposing further changes.

## Source Pointers

| Edit | Source report section |
|---|---|
| D1-1 | `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` § Recommended Spec Edits → Edit 1 |
| D1-2 | …Edit 2 |
| D1-3 | …Edit 3 |
| D1-4 | …Edit 4 |
| D1-5 | …Edit 5 |
| D1-7 | …Edit 7 |
| D2-1 | `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Recommended Spec Edits → Edit 1 |
| D2-2 | …Edit 2 |
| D2-3 | …Edit 3 |
| D2-4 | …Edit 4 |
| D2-5 | …Edit 5 |
| D2-6 | …Edit 6 |
| D2-7 | …Edit 7 |
| D2-8 | …Edit 8 |
| D2-9 | …Edit 9 |
| D2-10 | …Edit 10 |

## Follow-Up

After this batch lands:

1. The `/research/README.md` index should advance Domain 1 and
   Domain 2 status from `delivered` to `triaged`.
2. Domain 3 (Schematic & drawing conventions) deep-dive can dispatch
   against the post-batch spec baseline.
3. The 3 deferred edits remain queued for a later batch alongside
   any marketing/positioning consolidation that emerges from
   Domains 3–8.
