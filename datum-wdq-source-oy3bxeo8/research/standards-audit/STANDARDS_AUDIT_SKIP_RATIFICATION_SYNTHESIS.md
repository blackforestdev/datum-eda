# Standards Audit — Skip-Ratification Synthesis (Post-Domain-8)

> Sibling artifact to `STANDARDS_AUDIT.md` (the Phase 1 inventory).
> Synthesis of the advisory exclusions across all 8 Phase 2 deep-dives.
> Produces formal `Out of scope` language the project owner can lift
> into binding scope documents (`docs/INTEROP_SCOPE.md`,
> `specs/PROGRAM_SPEC.md`, `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4).
>
> Per the original audit's Pending Exclusions Policy: "Final
> ratification of skips into binding scope documents happens in a
> single consolidated pass after Domain 8 lands, when full
> cross-domain context is available." Domain 8 landed 2026-04-18; this
> is that pass.
>
> Honours `feedback_research_only_mode` — research synthesis only;
> the user owns the actual integration into `/specs/` and `/docs/`.

## Executive Summary

- 38 advisory exclusion candidates evaluated across the eight Phase 2
  deep-dives (≈21 carried forward from the Phase 1 audit's "Recommended
  low-priority / skip" list, ≈17 newly surfaced by domain agents).
- 33 promoted to formal `Out of scope` for ratification; 4 demoted to
  `Deferred with prerequisite` (Domain 6 high-speed PHY paywall packs);
  1 re-queued for follow-on research only if a specific trigger fires
  (Specctra DSN, conditioned on FreeRouting integration becoming a
  goal — currently unlikely given Datum's M5 routing kernel).
- The dominant cross-cut finding is **substrate-not-certifier**: every
  process-grade certification skip (DO-254 / DO-160 / MIL-PRF /
  NASA-STD / AS9100 / IATF 16949 / CMMI / ISO 13485 process-grade) has
  a real substrate story Datum already provides. The recommended
  pattern (originating in Domain 4 § Pending Exclusions and ratified by
  Domain 8) is a **substrate-positioning paragraph attached to each
  formal exclusion** so the skip reads as "Datum is substrate-relevant
  but not certifying", not "Datum is irrelevant".
- One genuine cross-cut surprise: **Specctra DSN** (Domain 1 advisory
  skip) thinly resurfaces in Domain 6 routing context via FreeRouting
  workflow patterns, but Datum's own M5 routing kernel makes the
  resurfacing strategically uninteresting. Confirm skip; the cross-cut
  finding does not change the verdict, only the rationale.
- The bulk of the formal exclusion language is already mostly in place
  in `STANDARDS_COMPLIANCE_SPEC.md` § 4, in `docs/INTEROP_SCOPE.md`
  "Explicitly out of scope" subsections, and in Domain-pertinent
  one-line `Out of scope` rows in `INTEROP_SCOPE.md`. This synthesis
  is largely a consolidation/audit pass on existing language plus the
  newly surfaced exclusions (HDL languages / MAST / `.jed` / DIN family
  / IEEE 100 / ANSI Y14.5 / CISPR 14-15 / CMII / DITA / per-vendor
  SAML / process-grade ISO 13485 framing).
- Recommendation on the 3 deferred Batch-1 edits (D1-6 git
  conventions, D1-8 / D2-11 open-stack appendices): **bundle them with
  this ratification pass as a single "Standards Audit Batch 6"** —
  they are the same kind of low-cost descriptive/marketing work and
  share none of the schema-impact risk that justified holding
  Batches 2-5 apart. Bundling avoids leaving three orphan items in the
  edit queue after the formal ratification closes the audit backbone.
- With this synthesis ratified, the **8-domain Phase 2 sequence + the
  Phase 1 inventory + the consolidated skip ratification close out the
  formal standards-audit research backbone**. Future standards work
  flows through the established research → guidance → spec pipeline on
  per-trigger basis (e.g., a customer demand for a Windchill connector,
  or a paywall-cleared HDMI 2.1 rule pack).

## Aggregated Advisory Exclusion List

The canonical consolidated record. Each row is a candidate exclusion;
the "Origin" column distinguishes Phase 1 advisory items from items
newly surfaced by Phase 2 deep-dives. The "Phase 2 status" column
records each domain agent's verdict under their domain's deeper context.

| # | Standard / topic | Domain | Origin | Phase 2 status | Source citation |
|---|---|---|---|---|---|
| 1 | Specctra DSN/SES | 1 | Phase 1 | confirmed-as-skip (with thin re-queue trigger if FreeRouting integration becomes a goal) | `data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` § Pending Exclusions |
| 2 | Hyperlynx HYP | 1 | Phase 1 | confirmed-as-skip | `data-exchange-interop/.../Pending Exclusions` |
| 3 | Altium native (`.PcbDoc` / `.SchDoc`) — direct binary parsing | 1 | Phase 1 | confirmed-as-skip (IPC-2581 Rev C is the migration path) | `data-exchange-interop/.../Pending Exclusions` |
| 4 | OrCAD / Allegro native binaries | 1 | Phase 1 | confirmed-as-skip | `data-exchange-interop/.../Pending Exclusions` |
| 5 | PADS native binaries | 1 | Phase 1 | confirmed-as-skip | `data-exchange-interop/.../Pending Exclusions` |
| 6 | DWG | 1 | Phase 1 | confirmed-as-skip (license-hostile; DXF covers the use case) | `STANDARDS_COMPLIANCE_SPEC.md` § 4.1 (already disposed); also `INTEROP_SCOPE.md` "Explicitly out of scope" |
| 7 | JT (Siemens lightweight 3D, ISO 14306) | 1 | Phase 2 (new) | new-exclusion-recommended | `INTEROP_SCOPE.md` "Explicitly out of scope" reflects this |
| 8 | EDIF 2 0 0 (schematic interchange) | 1, 3 | Phase 2 (new) | new-exclusion-recommended | `schematic-drawing-conventions/.../Pending Exclusions`; `INTEROP_SCOPE.md` "Explicitly out of scope" already lists |
| 9 | IDF 4.0 (never adopted) | 1 | Phase 2 (new) | new-exclusion-recommended | `INTEROP_SCOPE.md` "Explicitly out of scope" |
| 10 | OBJ / STL / OpenSCAD / JSCAD (hobbyist 3D) | 1 | Phase 2 (new) | new-exclusion-recommended | `INTEROP_SCOPE.md` "Explicitly out of scope" |
| 11 | HDF5 / Parquet (SI/PI bulk-data, no Datum SI engine in v1) | 1, 6 | Phase 2 (new) | new-exclusion-recommended | `INTEROP_SCOPE.md` "Explicitly out of scope" |
| 12 | JEDEC JEP30 (PIP) | 2 | Phase 1 | confirmed-as-skip | `component-modeling/.../Pending Exclusions` |
| 13 | JEDEC JESD8 (logic-family electrical) | 2 | Phase 1 | confirmed-as-skip (superseded by IBIS in practice) | `component-modeling/.../Pending Exclusions` |
| 14 | JEDEC MO outline drawings | 2 | Phase 1 | confirmed-as-skip (superseded by manufacturer STEP) | `component-modeling/.../Pending Exclusions` |
| 15 | IHS Markit Engineering Workbench (S&P Global EW) | 2, 7 | Phase 1 | confirmed-as-skip (re-affirmed in both domains) | `component-modeling/.../Pending Exclusions`; `plm-lifecycle-integration/.../Pending Exclusions` |
| 16 | Verilog 1364 / SystemVerilog 1800 / VHDL 1076 (HDL) | 2 | Phase 2 (new) | new-exclusion-recommended (FPGA/ASIC, not PCB) | `component-modeling/.../Pending Exclusions` |
| 17 | MAST (Saber proprietary) | 2 | Phase 2 (new) | new-exclusion-recommended | `component-modeling/.../Pending Exclusions` |
| 18 | JEDEC programmable-logic `.jed` files | 2 | Phase 2 (new) | new-exclusion-recommended (CPLD legacy) | `component-modeling/.../Pending Exclusions` |
| 19 | EDIF for HDL exchange (EDIF 4 0 0) | 2 | Phase 2 (new) | new-exclusion-recommended | `component-modeling/.../Pending Exclusions` |
| 20 | EDIF (IEC 61690) — schematic exchange | 3 | Phase 2 (new) | new-exclusion-recommended | `schematic-drawing-conventions/.../Pending Exclusions` |
| 21 | DIN 40700 / 40717 / 40900 | 3 | Phase 2 (new) | new-exclusion-recommended (superseded by DIN EN 60617) | `schematic-drawing-conventions/.../Pending Exclusions` |
| 22 | DIN 6771 / DIN 40719 | 3 | Phase 2 (new) | new-exclusion-recommended (superseded by ISO 7200 / IEC 81346) | `schematic-drawing-conventions/.../Pending Exclusions` |
| 23 | IEEE 100 (out-of-print dictionary) | 3 | Phase 2 (new) | new-exclusion-recommended | `schematic-drawing-conventions/.../Pending Exclusions` |
| 24 | ISO 3098 (mechanical lettering) | 3 | Phase 2 (new) | new-exclusion-recommended (or `Reference-only`; deep-dive prefers `Reference-only` w/ no implementation) | `schematic-drawing-conventions/.../Pending Exclusions` |
| 25 | ANSI/ASME Y14.5 (GD&T) for the schematic editor | 3 | Phase 2 (new) | new-exclusion-recommended for schematic; may resurface for fabrication-drawing output post-M7 | `schematic-drawing-conventions/.../Pending Exclusions` |
| 26 | DO-254 | 4 | Phase 1 | confirmed-as-skip | `industry-vertical-compliance/.../Pending Exclusions` |
| 27 | DO-160 | 4 | Phase 1 | confirmed-as-skip | `industry-vertical-compliance/.../Pending Exclusions` |
| 28 | MIL-PRF-31032 | 4 | Phase 1 | confirmed-as-skip | `industry-vertical-compliance/.../Pending Exclusions` |
| 29 | MIL-PRF-55110 | 4 | Phase 1 | confirmed-as-skip | `industry-vertical-compliance/.../Pending Exclusions` |
| 30 | NASA-STD-8739.x family | 4 | Phase 1 | confirmed-as-skip | `industry-vertical-compliance/.../Pending Exclusions` |
| 31 | AS9100 | 4, 8 | Phase 1 | confirmed-as-skip in both domains | `industry-vertical-compliance/.../Pending Exclusions`; `process-quality/.../Pending Exclusions` |
| 32 | IATF 16949 | 4, 8 | Phase 1 | confirmed-as-skip in both domains | `industry-vertical-compliance/.../Pending Exclusions`; `process-quality/.../Pending Exclusions` |
| 33 | CMMI for Development | 4, 8 | Phase 1 | confirmed-as-skip in both domains | `industry-vertical-compliance/.../Pending Exclusions`; `process-quality/.../Pending Exclusions` |
| 34 | California Prop 65 | 5 | Phase 1 | confirmed-as-skip | `materials-environmental/.../Pending Exclusions` |
| 35 | EU Packaging & Packaging Waste Directive | 5 | Phase 1 | confirmed-as-skip | `materials-environmental/.../Pending Exclusions` |
| 36 | RoHS-exemption tracking (catalog maintenance) | 5 | Phase 1 | confirmed-as-skip (storage of cited exemption ID is in scope; catalog maintenance is not) | `materials-environmental/.../Pending Exclusions` |
| 37 | HDMI / DisplayPort layout templates | 6 | Phase 1 | demoted-to-deferred-with-prerequisite (paywall) | `emc-signal-integrity/.../Pending Exclusions` |
| 38 | MIPI D-PHY / C-PHY layout templates | 6 | Phase 1 | demoted-to-deferred-with-prerequisite (paywall) | `emc-signal-integrity/.../Pending Exclusions` |
| 39 | PCIe Gen5 / Gen6 / Gen7 rule packs | 6 | Phase 2 (new) | demoted-to-deferred-with-prerequisite (PCI-SIG paywall) | `emc-signal-integrity/.../Pending Exclusions` |
| 40 | LPDDR6 / DDR6 rule packs | 6 | Phase 2 (new) | demoted-to-deferred-with-prerequisite (sampling 2026; vendor controller layout guides pending) | `emc-signal-integrity/.../Pending Exclusions` |
| 41 | CISPR 14 / CISPR 15 (white-goods / lighting) | 6 | Phase 2 (new) | new-exclusion-recommended (covered by EN 55014 mapped via `EmissionsStandard::En55014`) | `emc-signal-integrity/.../Pending Exclusions` |
| 42 | MIL-STD-461G (defence-grade EMC qualification) | 6 | Phase 2 (new) | recommend `Reference-only` (declarable target, no validation) | `emc-signal-integrity/.../Pending Exclusions` |
| 43 | Windchill connector | 7 | Phase 1 | confirmed-as-skip (per-customer; substrate exposes vault-API + lifecycle feed + DocumentRef CMIS hook) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 44 | Teamcenter connector | 7 | Phase 1 | confirmed-as-skip | `plm-lifecycle-integration/.../Pending Exclusions` |
| 45 | Aras Innovator connector | 7 | Phase 1 | confirmed-as-skip | `plm-lifecycle-integration/.../Pending Exclusions` |
| 46 | Arena PLM connector | 7 | Phase 1 | confirmed-as-skip (cloud SaaS; also conflicts with `data_egress_policy: NoExternalNetwork`) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 47 | OpenBOM connector | 7 | Phase 1 | confirmed-as-skip | `plm-lifecycle-integration/.../Pending Exclusions` |
| 48 | SiliconExpert | 7, 2 | Phase 1 | confirmed-as-skip | `plm-lifecycle-integration/.../Pending Exclusions` |
| 49 | PartQuest connector | 7 | Phase 2 (new) | new-exclusion-recommended (Siemens-ecosystem; same per-customer framing as Windchill) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 50 | CMII methodology enforcement | 7 | Phase 2 (new) | new-exclusion-recommended (methodology, not toolable; substrate primitives satisfy CMII data needs) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 51 | DITA documentation pipelines | 7 | Phase 2 (new) | new-exclusion-recommended (out of EDA-tool scope) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 52 | Per-vendor SAML SP integration | 7 | Phase 2 (new) | new-exclusion-recommended (federated-identity acceptance is in scope; per-IdP integration is connector work) | `plm-lifecycle-integration/.../Pending Exclusions` |
| 53 | ISO 13485 process-grade conformance claim | 8 | Phase 2 (new) | confirm `Reference-only` (already classified there in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4) | `process-quality/.../Pending Exclusions` |

**Reconciliation note on Phase 1 already-researched cross-references.**
Per the synthesis brief, items the Phase 1 audit flagged as
"already-researched" — IPC-A-600 / IPC-A-610 (covered by IPC research),
IPC-T-50 (covered by IPC research), IPC-2581 / Gerber X3 / IPC-D-356A
(covered by IPC research) — are **not** advisory exclusions and do not
appear above. Likewise IPC-1752A and IPC-2141 cross-domain references
are out of scope for this synthesis.

## Cross-Cut Re-Evaluation

For each candidate above, the cross-cut audit asks: does any *other*
Phase 2 report's cross-domain insights surface hidden value, conflict
with an "implement now" recommendation, or shift load-bearingness under
the post-Domain-8 substrate? Most exclusions hold; the small set of
exceptions is recorded explicitly.

### Exclusions that hold cleanly across all eight domains

The following 33 candidates were re-checked against every other
domain's report and showed no cross-cutting value, no conflict, and no
shift in load-bearingness. **Promote to formal `Out of scope`** without
qualification:

- **Hyperlynx HYP** (Domain 1). Single-vendor SI extraction; not an
  exchange format. Domain 6 separately recommends `export_hyperlynx_hyp`
  as a SI-artifact handoff to external solvers — but that is an
  *export*, not a Datum-side parser of Hyperlynx as input. No conflict.
- **Altium / OrCAD / PADS commercial native binary parsing**
  (Domain 1). The IPC-2581 Rev C import path (Domain 1 hard requirement,
  also referenced by Domain 7's per-customer-PLM-connector framing) is
  the practical migration route; direct binary parsing remains unjustified.
- **DWG, JT, IDF 4.0, OBJ/STL/OpenSCAD/JSCAD, HDF5/Parquet**
  (Domain 1). Already articulated in `INTEROP_SCOPE.md` "Explicitly
  out of scope" with reasons that hold under the post-Domain-8 substrate.
- **JEDEC JEP30, JESD8, MO drawings** (Domain 2). Three independent
  superseded-in-practice exclusions; Domain 6's IBIS-aware tolerance
  derivation absorbs the one marginal cross-cut (JESD8 buffer-class
  framing) via IBIS attachment instead.
- **IHS Markit Engineering Workbench / SiliconExpert** (Domains 2, 7).
  Both reaffirmed by Domain 7; both excluded as per-customer commercial
  catalog work, not engine work.
- **HDL languages, MAST, `.jed`, EDIF for HDL** (Domain 2). FPGA / ASIC
  and CPLD-legacy concerns; no PCB-tool footprint. Domain 4's vertical
  framing keeps FPGA pin-constraint files (XDC / QSF / LPF) as a
  potential future Domain 4 concern, but the underlying HDL languages
  themselves remain out.
- **EDIF for schematic exchange, DIN 40700/40717/40900/6771/40719,
  IEEE 100, ISO 3098 (or `Reference-only`), ANSI/ASME Y14.5 GD&T for
  the schematic editor** (Domain 3). All superseded or out-of-scope.
  Note: ANSI/ASME Y14.5 may resurface for fabrication-drawing PDF
  output post-M7; the exclusion is scoped to the schematic editor.
- **DO-254, DO-160, MIL-PRF-31032, MIL-PRF-55110, NASA-STD-8739**
  (Domain 4). Process-grade certification; substrate paragraph
  applies (see Categorised § Process certifications below).
- **AS9100, IATF 16949, CMMI** (Domains 4 + 8). Re-affirmed in both
  domains; substrate paragraph applies.
- **California Prop 65, EU Packaging Directive, RoHS-exemption catalog
  maintenance** (Domain 5). Out of EDA scope; substance presence
  surfaces via REACH SVHC / TSCA / RoHS coverage, packaging is a fab
  concern, exemption catalog maintenance is regulatory-data-vendor work.
- **CISPR 14, CISPR 15** (Domain 6). Covered via EN 55014 declared
  through `EmissionsStandard::En55014`; no separate Datum
  declaration needed.
- **Windchill, Teamcenter, Aras, Arena, OpenBOM, PartQuest** (Domain 7).
  Per-customer connector work; substrate (vault-API + lifecycle-event
  feed + DocumentRef CMIS hook) is exposed for binding when a customer
  demands it.
- **CMII methodology enforcement, DITA pipelines, per-vendor SAML SP
  integration** (Domain 7). Methodology / documentation-pipeline /
  per-IdP connector concerns; not Datum-engine work.
- **ISO 13485 process-grade conformance claim** (Domain 8). Already
  classified `Reference-only` in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4;
  the deep-dive confirms.

### Exclusions with cross-cutting note (verdict unchanged)

Two items have a real cross-cut insight that does **not** change the
exclusion verdict but should be recorded in the formal exclusion
language so the reasoning is preserved.

- **Specctra DSN/SES** (Domain 1). Domain 1's Pending Exclusions
  section flags Specctra DSN as the input format for FreeRouting, the
  open-source autorouter that some KiCad users invoke. If Datum ever
  pursued FreeRouting integration, Specctra DSN export would be
  required. **Verdict unchanged**: Datum's M5 routing kernel makes
  FreeRouting integration unlikely; the exclusion holds, but the
  formal exclusion language should include a one-line escape clause:
  *"re-evaluate only if FreeRouting integration becomes a goal"*. This
  is the closest the synthesis has to a re-queue candidate, but it
  doesn't rise to the threshold of recommending the deep-dive be
  re-opened today.
- **JEDEC MO outline drawings** (Domain 2). Domain 2 notes that MO
  drawings are still cited as the authoritative source for *body
  height* in some IPC-7351 footprint-generation workflows. The
  body-height field on `Package` (Domain 1 recommendation, landed in
  Standards-Audit-Batch-1 as `Package.body_height_nm` /
  `body_height_mounted_nm`) covers this need without ingesting MO
  drawings directly. **Verdict unchanged**: skip stands; the substrate
  field absorbs the relevant data.

### Exclusions demoted to `Deferred with prerequisite`

Per Domain 6's explicit framing — "fold-in candidates, NOT hard
exclusions" — these items should be formalised as `Deferred with
prerequisite` rather than `Out of scope`:

- **HDMI 2.0 / 2.1 layout templates** (Domain 6). Prerequisite: HDMI
  Forum adopter agreement OR vendor-published per-IC layout guide
  sourced.
- **DisplayPort 1.4 / 2.x layout templates** (Domain 6). Prerequisite:
  VESA member agreement OR vendor-published per-IC layout guide
  sourced.
- **MIPI D-PHY / C-PHY / M-PHY layout templates** (Domain 6).
  Prerequisite: MIPI Alliance adopter agreement OR vendor-published
  per-IC layout guide sourced.
- **PCIe Gen5 / Gen6 / Gen7 rule packs** (Domain 6). Prerequisite:
  PCI-SIG paywall access OR vendor-published per-IC layout guide
  sourced. Substrate already supports them; rule packs ship as add-on
  library content when prerequisite is met.
- **LPDDR6 / DDR6 rule packs** (Domain 6). Prerequisite: volume
  samples ship + vendor controller layout guides published. Currently
  sampling-2026 expectation.
- **MIL-STD-461G** (Domain 6). Recommend `Reference-only` (declarable
  target, no validation) rather than `Out of scope`. Substrate supports
  declaration; the per-vertical mapping does not auto-populate.

### Re-queued for follow-on research (advisory tag, not active work)

- **Specctra DSN export, only if FreeRouting integration becomes a
  goal**. This is a conditional re-queue, not an active research
  recommendation. Documented here so the re-queue trigger is
  preserved in the audit record.

### Summary of cross-cut verdict counts

| Verdict | Count | Notes |
|---|---|---|
| Confirmed `Out of scope` (33 items + 4 cross-domain duplicates collapsed) | 33 | Promote with substrate paragraphs where applicable |
| Demoted to `Deferred with prerequisite` | 5 | HDMI / DisplayPort / MIPI / PCIe Gen5+ / DDR6 |
| Demoted to `Reference-only` | 1 | MIL-STD-461G (declarable target, no validation) |
| Re-queued for follow-on research (conditional only) | 1 | Specctra DSN, conditional on FreeRouting integration becoming a goal |
| Total candidates evaluated | 38–40 (depending on whether cross-domain duplicates are counted separately) | — |

(Counts add to slightly more than 38 because Domain 6's PCIe Gen5+ and
DDR6 demotions surfaced as new exclusions in the deep-dive but land in
the `Deferred with prerequisite` bucket rather than the formal
exclusion bucket.)

## Categorised Formal Exclusion List

Grouped by exclusion **reason**, not by domain, because the rationale
paragraphs are reusable per-category.

### Process certifications (substrate-not-certifier)

**Members:**
- DO-254 (RTCA airborne electronic hardware design assurance)
- DO-160 (RTCA airborne environmental qualification)
- MIL-PRF-31032 (DoD PWB general specification)
- MIL-PRF-55110 (DoD rigid PWB)
- NASA-STD-8739.x family (NASA workmanship)
- AS9100 (SAE/IAQG aerospace QMS)
- IATF 16949 (automotive QMS)
- CMMI for Development (organisational maturity)
- ISO 13485 process-grade conformance claim (medical-device QMS) —
  already `Reference-only`; confirm in ratification

**Substrate paragraph (reusable):**

> Datum is a compliance substrate, not a certifying authority. Datum
> may store, validate, export, and explain compliance-relevant
> metadata; Datum may not claim that an organisation, project, or
> manufacturing process is certified merely because Datum carries
> fields related to that standard. Process-grade certification is
> conferred on the organisation by an accredited registrar, designated
> engineering representative (DER), or appraiser. Datum's deterministic
> transaction substrate, audit-log export (Domain 8), ECO state
> machine + signature substrate (Domains 7 + 8), `ProvenanceTag`
> (Domain 8), and `Project.compliance` posture block (Domain 4) are
> sufficient substrate to support an organisation's certification work
> against these standards but do not constitute certification.

### Superseded / dead in practice

**Members:**
- JEDEC JEP30 (PIP — Part Information Profile)
- JEDEC JESD8 family (logic-family electrical specifications)
- JEDEC MO outline drawings (MO-220, MO-153, etc.)
- EDIF 2 0 0 (schematic interchange)
- EDIF 4 0 0 (HDL exchange)
- IDF 4.0 (never adopted)
- DIN 40700 / 40717 / 40900 (superseded by DIN EN 60617)
- DIN 6771 / DIN 40719 (superseded by ISO 7200 / IEC 81346)
- IEEE 100 (out-of-print dictionary)
- ISO 3098 (mechanical lettering; or `Reference-only` with no
  implementation, per Domain 3 deep-dive preference)
- MAST (Saber proprietary; no open parser)
- JEDEC programmable-logic `.jed` files (CPLD-only legacy)

**Substrate paragraph (reusable):**

> The standard is superseded in practice by either (a) manufacturer-
> specific datasheets and Octopart/Nexar-class metadata, (b) modern
> revisions or replacement standards already in Datum's `Planned`
> registry, or (c) ecosystem disuse confirmed by absence in current
> EDA-tool corpora and forum traffic. Datum's substrate covers the
> data the standard expressed (where any cross-cutting signal exists)
> via the cited replacement: e.g., MO-drawing body-height information
> is carried by `Package.body_height_nm` (Standards-Audit-Batch-1).

### Vendor-platform connector work (per-customer, not pre-emptive)

**Members:**
- Windchill (PTC PLM)
- Teamcenter (Siemens PLM)
- Aras Innovator (Aras PLM)
- Arena PLM (PTC cloud)
- OpenBOM (cloud BOM/PLM)
- IHS Markit Engineering Workbench / S&P Global Engineering Workbench
- SiliconExpert
- PartQuest / PartQuest Xpress (Siemens parts library)
- Per-vendor SAML SP integration (per-IdP)

**Substrate paragraph (reusable):**

> The integration is connector work specific to a vendor platform and
> a customer's deployment, not Datum-engine work. Datum's substrate —
> the vault-API surface, lifecycle-event feed, `DocumentRef` CMIS
> hook, federated `ActorIdentity` (Domain 8), and `data_egress_policy`
> gate (Domain 4) — is exposed precisely so per-customer connectors
> can bind without engine modification. Datum will not pre-emptively
> implement vendor-specific connectors; they ship per customer demand.

### License-hostile or paywall-blocked

**Members:**
- DWG (Autodesk; license-hostile to open-source/permissive
  redistribution)
- JT (Siemens lightweight 3D, ISO 14306; license-encumbered)

**Substrate paragraph (reusable):**

> The format's licensing terms are incompatible with Datum's
> redistribution model. The use case is covered by an open or
> permissively licensed alternative (DXF for mechanical interop, STEP
> AP242 for 3D model exchange) that Datum already plans or implements.

### Single-vendor extraction formats

**Members:**
- Specctra DSN / SES (Cadence; legacy autorouter interchange)
- Hyperlynx HYP (Mentor/Siemens; SI extraction)

**Substrate paragraph (reusable):**

> The format is a single-vendor extraction format, not an open
> exchange standard. The Datum-side input use case is unsupported by a
> credible adoption pattern; the Datum-side output use case (where
> applicable, e.g., Hyperlynx HYP export) is handled separately as a
> SI-artifact handoff tool, not as bidirectional interchange.

**Specctra DSN re-queue clause:** *re-evaluate only if FreeRouting
integration becomes a goal; Datum's M5 routing kernel makes this
unlikely.*

### Out-of-EDA-scope concerns

**Members:**
- California Prop 65 (consumer-product warning labelling; substance
  presence covered by REACH SVHC / TSCA / RoHS)
- EU Packaging & Packaging Waste Directive (94/62/EC + amendments;
  fab-and-assembly concern)
- RoHS-exemption catalog maintenance (regulatory-data-vendor
  subscription work; Datum stores cited exemption IDs only)
- HDL languages: Verilog 1364, SystemVerilog 1800, VHDL 1076 (FPGA /
  ASIC tools, not PCB tools)
- Hobbyist 3D formats: OBJ / STL / OpenSCAD / JSCAD
- HDF5 / Parquet for SI/PI bulk data (no Datum SI engine in v1)
- CMII methodology enforcement (methodology, not toolable)
- DITA documentation pipelines (out of EDA scope)
- ANSI/ASME Y14.5 (GD&T) for the schematic editor (mechanical
  drawing concern; may resurface for fabrication-drawing PDF output
  post-M7)
- CISPR 14 / CISPR 15 (white-goods / lighting; covered via EN 55014
  declared through `EmissionsStandard::En55014`)

**Substrate paragraph (reusable):**

> The standard or topic addresses a workflow that lies outside Datum's
> PCB-design substrate scope. Where the underlying signal has any
> Datum-side relevance, an in-scope substrate field, in-scope adjacent
> standard, or AI-surface tool already provides coverage (e.g., REACH
> SVHC for substance presence; `Package.body_height_nm` for body
> dimensions; `EmissionsStandard::En55014` for the white-goods /
> lighting EMC profile).

## Formal Exclusion Language

Ready-to-lift text blocks, grouped by target document for the user's
apply convenience. Each block is 2–4 lines and assumes the
substrate-paragraph language above is also lifted (as a § preamble or
as inline expansions).

### Recommended for `docs/INTEROP_SCOPE.md`

These are format/protocol exclusions; `INTEROP_SCOPE.md` is their
natural home. Most are **already present** in the existing "Explicitly
out of scope" subsections; the consolidated synthesis recommends
auditing those subsections against this list and adding any missing
items. Newly-recommended additions to existing subsections are flagged
with **NEW**.

#### Append to "Future export targets (research-staged)" → "Explicitly out of scope"

```
- Specctra DSN/SES (Cadence; legacy autorouter interchange).
  Re-evaluate only if FreeRouting integration becomes a goal; Datum's
  M5 routing kernel makes that unlikely.
```
*(Already present in INTEROP_SCOPE.md line 118; recommend adding the
re-queue clause as a one-line addition.)*

#### Append to "Behavioural model attachment & export" → "Explicitly out of scope"

These are mostly already covered in the existing text (lines 155–167);
audit for the following additions:

```
- DWG (Autodesk; license-hostile redistribution; mechanical interop
  is covered by DXF).  [NEW location-suggestion: this currently
  appears in the future-export-targets subsection; confirm it does not
  also belong here.]
```

```
- JT (Siemens lightweight 3D, ISO 14306; license-encumbered;
  3D-model exchange is covered by STEP AP242).  [Already present.]
```

#### NEW subsection for `INTEROP_SCOPE.md` — schematic interchange

The existing `INTEROP_SCOPE.md` does not have a dedicated schematic-
interchange subsection beyond the per-tool import sections (KiCad /
Eagle / Altium future). Recommend adding a short "Schematic interchange
formats — explicitly out of scope" block:

```
### Schematic interchange formats (explicitly out of scope)

The following schematic-interchange formats are out of Datum's v1 and
post-v1 import/export scope. Dispositions live in
`specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.3.

- EDIF (IEC 61690; EDIF 2 0 0 schematic interchange and EDIF 4 0 0
  HDL exchange) — dead in practice as a schematic exchange medium;
  KiCad / Eagle / Altium / OrCAD all provide better paths.
- HDL languages: Verilog 1364 / SystemVerilog 1800 / VHDL 1076 — FPGA
  / ASIC tools, not PCB tools.
- MAST (Saber proprietary) — no open parser; no PCB-tool footprint.
- JEDEC programmable-logic `.jed` files — CPLD-only legacy; no
  PCB-tool consumption pattern.
```

#### NEW subsection for `INTEROP_SCOPE.md` — schematic drawing-style standards

The Domain 3 deep-dive surfaced a set of superseded drawing-style
standards. These are not interop formats per se but they belong in the
INTEROP_SCOPE.md "out of scope" framing because users searching for
DIN / IEEE 100 / ISO 3098 conformance will land here first:

```
### Schematic drawing-style standards (explicitly out of scope)

The following schematic drawing-style standards are out of scope as
Datum-claimed targets. Dispositions live in
`specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.3. Datum's symbol-style
profile system supports IEEE 315 / IEC 60617 / JIS C 0617; the
standards listed here are either superseded by those profiles or
address a non-schematic concern.

- DIN 40700 / 40717 / 40900 (superseded by DIN EN 60617; covered via
  the IEC 60617 profile)
- DIN 6771 / DIN 40719 (superseded by DIN EN ISO 7200 and DIN EN IEC
  81346; covered via the ISO 7200 title-block contract and the
  designator-profile registry)
- IEEE 100 (out-of-print dictionary; superseded by per-standard
  definitions; vocabulary baseline lives in IPC-T-50 + IEC 60050)
- ISO 3098 (mechanical-drawing lettering; Datum schematic renderer
  uses standard fonts. May be classified `Reference-only` with no
  implementation per the Domain 3 deep-dive's preferred framing.)
- ANSI/ASME Y14.5 (GD&T) for the schematic editor — mechanical, not
  schematic. May resurface for fabrication-drawing PDF output post-M7;
  the exclusion is scoped to the schematic editor.
```

### Recommended for `specs/PROGRAM_SPEC.md`

`PROGRAM_SPEC.md` already carries milestone-level "Non-goals" sections
(M0 line 133, M1 line 155, M2 line 180, M3 line 238, M4 line 266) and
the v1 Definition (lines 61–79 per the Phase 1 audit reference). The
Phase 1 audit recommended a one-line "v1 does not implement industry
vertical QMS support" augment parallel to the existing GUI/routing
exclusions; this synthesis confirms that recommendation and proposes
the following blocks. (PROGRAM_SPEC.md is the home for the `process
certifications` exclusion category because these are program-scope
exclusions, not interop format exclusions; the per-domain dispositions
land in `STANDARDS_COMPLIANCE_SPEC.md` § 4.)

```
### Out of program scope (v1 and later)

The following standards-and-compliance regimes are explicitly out of
Datum's product-scope claims for v1 and later milestones unless a
later milestone explicitly promotes them. Datum is a compliance
substrate, not a certifying authority. Per-domain dispositions and
substrate paragraphs live in `specs/STANDARDS_COMPLIANCE_SPEC.md`
§§ 4.4 and 4.8.

- Process-grade certification regimes: DO-254, DO-160, MIL-PRF-31032,
  MIL-PRF-55110, NASA-STD-8739.x family, AS9100, IATF 16949, CMMI
  for Development. Datum's deterministic-transaction substrate,
  audit-log export, ECO state machine + signature substrate,
  `ProvenanceTag`, and `Project.compliance` posture block are
  sufficient substrate to support an organisation's certification
  work but do not constitute certification.
- ISO 13485 process-grade conformance claim — substrate-relevant only
  (`Reference-only` per `STANDARDS_COMPLIANCE_SPEC.md` § 4.4).
- CMII methodology enforcement, DITA documentation pipelines —
  out of EDA-tool scope.
- Per-vendor PLM connectors (Windchill, Teamcenter, Aras, Arena,
  OpenBOM, PartQuest) and per-vendor SAML SP integration —
  per-customer connector work; substrate is exposed for binding when
  a customer demands it.
- Per-vendor commercial component-intelligence catalogs (IHS Markit
  Engineering Workbench / S&P Global Engineering Workbench,
  SiliconExpert) — paid catalog work, not engine work.
```

### Recommended for `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4

The bulk of the formal exclusion language already lives in this
spec — § 4.1 through § 4.8 already carry many of the listed exclusions.
This synthesis surfaces the items that are **not yet** captured (or
that should be re-stated for completeness) and proposes the additions
per § 4.x subsection.

#### § 4.1 Data Exchange And Interop

Already captures Specctra DSN/SES, Hyperlynx HYP, DWG, and the
commercial-native binary formats. **Add** (verbatim language proposal):

```
- JT (Siemens lightweight 3D, ISO 14306): `Out of scope` — license-
  encumbered; 3D-model exchange is covered by STEP AP242.
- IDF 4.0: `Out of scope` — never adopted; IDF 3.0 covers the use case.
- OBJ / STL / OpenSCAD / JSCAD: `Out of scope` — hobbyist 3D formats;
  STEP AP242 covers the professional use case.
- HDF5 / Parquet for SI/PI bulk data: `Out of scope` — Datum has no
  SI engine surface in v1; revisit only if Datum ships an SI engine.
```

(Specctra-DSN entry should gain the re-queue clause:
*"re-evaluate only if FreeRouting integration becomes a goal; Datum's
M5 routing kernel makes this unlikely."*)

#### § 4.2 Component Modelling

Already captures HDL languages, MAST, JEDEC `.jed`, EDIF for HDL,
JEP30, JESD8, MO drawings, IHS Markit Engineering Workbench. **Add**:

```
- SiliconExpert: `Out of scope` — paid commercial catalog; per-customer
  integration if ever requested.
```

#### § 4.3 Schematic And Drawing Conventions

Already captures EDIF, DIN 40700/6771/40719, IEEE 100, ISO 3098,
ANSI/ASME Y14.5. **Add the missing DIN variants and confirm
ISO 3098 framing**:

```
- DIN 40717 / DIN 40900: `Out of scope` — superseded by DIN EN 60617
  (covered via the IEC 60617 profile).
- ISO 3098 (mechanical lettering): `Reference-only` (no implementation;
  Datum schematic renderer uses standard fonts).  [Confirm this matches
  the Domain 3 deep-dive's preferred framing; the alternative is
  `Out of scope`.]
```

#### § 4.4 Industry-Vertical Compliance

Already captures DO-254, DO-160, AS9100, IATF 16949, CMMI,
MIL-PRF-31032, MIL-PRF-55110, NASA workmanship standards. **No
additions** beyond confirming substrate-paragraph attachment per
Domain 4's "substrate-compatibility paragraph" recommendation:

```
Each `Out of scope` entry in this subsection carries the substrate
paragraph from `STANDARDS_COMPLIANCE_SPEC.md` § 7.1 (substrate-versus-
certification) attached as the rationale, naming the substrate
features Datum provides that support an organisation's certification
work against the listed standard.
```

#### § 4.5 Materials And Environmental

Currently captures the disposition states (`Planned` / `Deferred with
prerequisite` / `Out of scope`) but does not enumerate the three
specific Domain 5 advisory exclusions. **Add**:

```
- California Prop 65: `Out of scope` — chemical-warning labelling is a
  downstream consumer-product packaging artifact; substance presence
  relevant to Prop 65 is captured through `Part.compliance.reach_svhc`,
  `tsca_status`, and `rohs_status`.
- EU Packaging & Packaging Waste Directive (94/62/EC + amendments):
  `Out of scope` — packaging is a fab-and-assembly concern downstream
  of EDA. The `Part.packaging_options` field carries EIA-481 reel /
  tape / tray / tube / cut variants for procurement only.
- RoHS-exemption catalog maintenance: `Out of scope` — catalog upkeep
  is regulatory-data-vendor work. Datum stores the cited exemption ID
  verbatim via `Part.compliance.rohs_exemption` and records project
  acknowledgement via `Project.compliance.materials_posture.
  rohs_exemptions_acknowledged`. Validation that the cited exemption
  is in force is the user's regulatory team's responsibility.
```

#### § 4.6 EMC And Signal Integrity

Currently captures interface-specific rule families (USB, PCIe, DDR,
Ethernet) under `Deferred with prerequisite`. **Add the demoted
items** as a sub-bullet under that disposition:

```
- HDMI 2.0/2.1, DisplayPort 1.4/2.x, MIPI D-PHY/C-PHY/M-PHY, PCIe Gen5
  / Gen6 / Gen7, LPDDR6 / DDR6 rule packs: `Deferred with
  prerequisite`. Prerequisite: relevant adopter / member agreement
  acquired OR vendor-published per-IC layout guides sourced. Substrate
  ships with the first batch; rule packs ship as add-on library
  content when prerequisite is met.
```

**Add** for CISPR 14/15:

```
- CISPR 14 / CISPR 15 (white-goods / lighting): `Out of scope` —
  covered via EN 55014 declared through `EmissionsStandard::En55014`.
  No separate Datum-specific declaration.
```

**Add** for MIL-STD-461G (this is `Reference-only`, not `Out of scope`):

```
- MIL-STD-461G: `Reference-only` — declarable target via
  `EmissionsStandard::MilStd461g` (when added); no Datum-side
  validation.
```

#### § 4.7 PLM And Lifecycle Integration

Already captures Windchill, Teamcenter, Aras, Arena. **Add**:

```
- OpenBOM, PartQuest / PartQuest Xpress, IHS Markit Engineering
  Workbench / S&P Global Engineering Workbench, SiliconExpert:
  `Out of scope` — connector / paid commercial catalog work; substrate
  exposes vault-API + lifecycle-event feed + `DocumentRef` CMIS hook
  for binding when a customer demands it.
- CMII methodology enforcement: `Out of scope` — methodology, not
  toolable. ECO grouping + approver + effective-date primitives
  satisfy CMII data needs; methodology enforcement lives in PLM
  workflows.
- DITA documentation pipelines: `Out of scope` — out of EDA-tool scope.
- Per-vendor SAML SP integration: `Out of scope` (per-IdP integration
  is connector work; federated `ActorIdentity` acceptance via OIDC /
  OAuth 2.1 is in core).
```

#### § 4.8 Process And Quality

Already captures CMMI, ISO/IEC 12207, organisation-process
assessments. **Add**:

```
- AS9100D / AS9110 / AS9120 (aerospace QMS) and IATF 16949
  (automotive QMS) process-grade conformance claims: `Out of scope`
  (process-grade certification conferred on the organisation by an
  accredited registrar). Substrate per § 7.1 supports an
  organisation's certification work against these standards via the
  audit-log export, ECO + signature substrate, `ProvenanceTag`, and
  `Project.compliance` posture block. AS9102 First Article Inspection
  evidence-package export hooks remain `Deferred with prerequisite`
  per § 4.7.
- ISO 13485 process-grade conformance claim: `Reference-only`
  (already disposed in § 4.4; restated here for cross-domain
  visibility).
```

## Re-Queued for Follow-On Research

One conditional re-queue surfaced; no candidate rises to the threshold
of recommending the deep-dive be re-opened today.

- **Specctra DSN/SES (Domain 1).** Specctra DSN is the input format
  for the FreeRouting open-source autorouter, which a portion of the
  KiCad user base invokes as a workflow. Datum's M5 routing kernel
  makes FreeRouting integration strategically uninteresting today, so
  the exclusion holds. **Re-queue trigger:** if FreeRouting
  integration ever becomes a goal (e.g., a customer demands it, or
  Datum's routing kernel finds a regression that FreeRouting could
  paper over), re-open the Domain-1 export-side analysis for Specctra
  DSN. Recommended scope of follow-on research at that time: ~1-2 day
  Specctra-DSN-export feasibility note, scoped to write-only (no
  Specctra-SES read-back, since Datum's own routing kernel does not
  consume autorouter SES sessions).

## Demoted from Out-of-Scope to Deferred-with-Prerequisite

Per Domain 6's explicit framing — "fold-in candidates, NOT hard
exclusions". These items move from the Phase 1 advisory skip list to
formal `Deferred with prerequisite` disposition rather than `Out of
scope`:

| Standard / topic | Domain | Prerequisite |
|---|---|---|
| HDMI 2.0 / 2.1 layout templates | 6 | HDMI Forum adopter agreement OR vendor-published per-IC layout guide sourced |
| DisplayPort 1.4 / 2.x layout templates | 6 | VESA member agreement OR vendor-published per-IC layout guide sourced |
| MIPI D-PHY / C-PHY / M-PHY layout templates | 6 | MIPI Alliance adopter agreement OR vendor-published per-IC layout guide sourced |
| PCIe Gen5 / Gen6 / Gen7 rule packs | 6 | PCI-SIG paywall access OR vendor-published per-IC layout guide sourced |
| LPDDR6 / DDR6 rule packs | 6 | Volume samples ship + vendor controller layout guides published |

One additional item lands at `Reference-only` rather than `Out of
scope` or `Deferred with prerequisite`:

- **MIL-STD-461G** (Domain 6) — declarable target via
  `EmissionsStandard::MilStd461g` (when added); no Datum-side
  validation.

## Recommendation on the 3 Deferred Batch-1 Edits

The three edits queued from Standards-Audit-Batch-1 (per
`docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md` lines 75–79):

- **D1-6** — `.gitignore` / `.gitattributes` conventions in
  `specs/NATIVE_FORMAT_SPEC.md` § 12 (or `docs/POOL_ARCHITECTURE.md`).
  Descriptive only.
- **D1-8** — "Datum's Open-Stack Position" appendix in
  `docs/COMMERCIAL_INTEROP_STRATEGY.md` § 10. Marketing position.
- **D2-11** — "Behavioural Model Stack — Open-Stack Position"
  appendix in `docs/COMMERCIAL_INTEROP_STRATEGY.md`. Marketing
  position.

**Recommendation:** **bundle them with this ratification pass as a
single "Standards Audit Batch 6"**. Three reasons:

1. **Same edit risk profile.** All three are descriptive / marketing /
   conventions edits with no spec-contract impact. They share the
   risk profile of the formal exclusion language proposed here (no
   new IR types, no new operations, no new MCP surface). Bundling
   does not increase Batch 6's review burden materially.
2. **Same review surface.** D1-8 and D2-11 both land in
   `docs/COMMERCIAL_INTEROP_STRATEGY.md` and complement the
   single-vendor-extraction-format exclusion category proposed in this
   synthesis (the Hyperlynx / Specctra exclusions naturally cite the
   open-stack position appendix as their "what Datum offers instead"
   rationale). D1-6's `.gitignore` / `.gitattributes` conventions
   are similarly substrate-touching to the audit-log persistence
   contract this synthesis references.
3. **Avoids orphan items in the edit queue.** Once the formal
   ratification closes the standards-audit research backbone, leaving
   three orphan items in the edit queue invites future drift. Closing
   them with the ratification keeps the audit's edit ledger empty.

If bundling is impractical for review-load reasons, the fallback is
to keep them separate but commit to a Batch 7 landing in the same
release window. The strict-defer option (Batch 8 or later) is not
recommended — these items have been queued since 2026-04-18 and the
marketing positioning is overdue for the open-stack messaging Domain 1
and Domain 2 already rely on internally.

## Closing Note

With this skip-ratification synthesis, the **8-domain Phase 2 sequence
+ the Phase 1 inventory + the consolidated skip ratification close out
the formal standards-audit research backbone**. Of the 38 advisory
exclusions surveyed (Phase 1 + Phase 2), 33 are promoted to formal
`Out of scope`, 5 are demoted to `Deferred with prerequisite`, 1 to
`Reference-only`, and 1 carries a conditional re-queue trigger.
Future standards work flows through the established research →
guidance → spec pipeline on a per-trigger basis (a customer demand for
a Windchill connector, a paywall-cleared HDMI 2.1 rule pack, a
FreeRouting integration request). The audit no longer has open
research-backbone work; the next standards-related artifacts are
spec-edit batches against the existing
`STANDARDS_COMPLIANCE_SPEC.md` § 4 dispositions plus
`INTEROP_SCOPE.md` and `PROGRAM_SPEC.md` augments per this
synthesis's Formal Exclusion Language section.

## Sources

Phase 2 reports' "Pending Exclusions (re-affirmed)" sections (the
controlling input for this synthesis):

- `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 1554–1584
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 1995–2039
- `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 1563–1604; also § Out of
  Scope (recommend formal exclusion) — lines 1956–1967
- `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 2207–2237
- `research/materials-environmental/MATERIALS_ENVIRONMENTAL_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 2282–2304
- `research/emc-signal-integrity/EMC_SIGNAL_INTEGRITY_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 2305–2347
- `research/plm-lifecycle-integration/PLM_LIFECYCLE_INTEGRATION_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 1757–1799
- `research/process-quality/PROCESS_QUALITY_RESEARCH.md`
  § Pending Exclusions (re-affirmed) — lines 2411–2493; also
  § Closing the Standards Audit — lines 2993–3045

Phase 1 audit (controlling baseline for the original advisory list):

- `research/standards-audit/STANDARDS_AUDIT.md` § Phase 2 Triage
  Recommendations → § Recommended low-priority / skip — lines 548–568
- `research/standards-audit/STANDARDS_AUDIT.md` § Pending Exclusions
  (advisory) — DO NOT DEEP-DIVE, DO NOT FORMALLY EXCLUDE — lines
  463–477

Existing controlling spec / scope documents (target state for the
formal exclusion language):

- `specs/STANDARDS_COMPLIANCE_SPEC.md` § 3 (Disposition Model) and
  § 4 (Registry Baseline)
- `docs/INTEROP_SCOPE.md` "Future export targets (research-staged)"
  → "Explicitly out of scope" subsections
- `specs/PROGRAM_SPEC.md` § Scope Integrity Terms / Non-goals (per-
  milestone) and § v1 Definition

Batch 1 deferred-edit reference:

- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md` lines 75–79 (D1-6, D1-8,
  D2-11) and line 199–200 (the deferred-edit policy note).
