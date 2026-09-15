# Industry-Vertical Compliance — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 4 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 4`
> ("Per-Domain Audit → 4. Industry-vertical compliance").
> Cross-references `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
> for IPC class selection (do not re-survey),
> `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` for
> encrypted vendor models (defence/aerospace use these heavily;
> attachment surface and encryption gate already specified), and
> `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md`
> for industry-mandated symbol-style profiles
> (`SymbolStyleProfile` cross-domain consumer).
> Companion to `research/airwire-rendering/`,
> `research/copper-rendering/`, and prior Phase 2 reports for tone,
> structure, and source-citation style.
>
> Reads against the post-Standards-Audit-Batch-1 spec baseline merged
> 2026-04-18 (PR #1). The contract surfaces this report relies on
> (`ModelAttachment`, `EncryptionScheme`, `Encrypted Content Handling
> Policy`, `Part.behavioural_models`, expanded `ModelProvenance`) all
> exist in `specs/ENGINE_SPEC.md` § 1.1a and `specs/MCP_API_SPEC.md`
> § Encrypted Content Handling Policy as of that merge.

> **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
>
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is available.
>
> Domain 4 carries the largest advisory-exclusion list of any domain.
> The audit recommended the following as "Recommended low-priority /
> skip" — they are NOT deep-dived in this report and are surfaced
> only for positioning context under "Aerospace / Defence" sub-sections
> below and re-affirmed under § "Pending Exclusions (re-affirmed)":
>
> - **DO-254** (avionics hardware design assurance)
> - **DO-160** (avionics environmental qualification)
> - **MIL-PRF-31032** (printed wiring boards, general)
> - **MIL-PRF-55110** (rigid PWBs)
> - **NASA-STD-8739** workmanship family
> - **AS9100** (aerospace QMS)
> - **IATF 16949** (automotive QMS)
> - **CMMI** (capability maturity model)
>
> These are process-grade certifications that the tool cannot enforce.
> The right answer is to position Datum as compatible **substrate**
> (deterministic transaction log, immutable history, signature-ready
> audit surface, encrypted-content policy) and refer users to their own
> QMS for the certification work. None of the eight has hidden cross-
> cutting value that would justify re-opening the deep-dive — the
> findings are positioning statements, not feature requirements.

## Executive Summary

- **The central framing is substrate vs certification, and Datum's
  substrate is genuinely strong for the substrate side.** Datum's
  deterministic transaction model
  (`docs/CANONICAL_IR.md` § 4 + `specs/ENGINE_SPEC.md` § 3) gives it
  immutable history with deterministic UUIDs, undoable per-operation
  diffs, and a JSON serialisation contract
  (`specs/ENGINE_SPEC.md` § 4) that hashes stably. That is the same
  primitive set every regulated-industry QMS expects from its CAD
  evidence chain. What Datum **cannot** do is be the certifying party
  — DO-254, AS9100, ISO 13485, IATF 16949, NASA-STD-8739, IPC-A-610
  Class 3A acceptance are all process-grade certifications conferred
  on an organisation by an accredited body, never on a tool.
  The honest positioning is "Datum is the substrate; your QMS is the
  certifying party". Of the 16 standards inventoried in this domain,
  **8 are advisory exclusions** (DO-254, DO-160, MIL-PRF-31032,
  MIL-PRF-55110, NASA-STD-8739, AS9100, IATF 16949, CMMI), all
  process-grade; **5 are substrate-relevant and worth Datum-side
  metadata** (21 CFR Part 11, ITAR/EAR, AEC-Q, ISO 26262, IEC 61508);
  and **3 are reference-only at the engine layer but worth project-
  level metadata** (IEC 60601, ISO 13485, FDA Part 820 / EU MDR).

- **The Part record is the single largest pinch-point for Domain 4
  metadata.** Batch 1 already extended `Part` with
  `behavioural_models`, `thermal`, `manufacturer_jep106`,
  `packaging_options`, `supply_chain_offers`. Domain 4 wants four more
  small additions: `qualification` (a struct carrying AEC-Q grade,
  MIL-spec qualification, RTCA/DO-254 design-assurance level if
  declared by vendor), `temperature_grade` (consumer / industrial /
  automotive / military), `radiation_tolerance` (rad-hard / rad-tol
  / commercial-off-the-shelf), and `export_control` (ECCN, USML
  category, EU dual-use entry). All four are pure-metadata fields
  with no engine algorithmic behaviour — they feed BOM filters,
  AI-explained part-selection diagnostics, and downstream compliance
  reports. Effort: **~3 days** for the data-model + MCP query surface,
  not counting the optional ITAR/EAR markings validator.

- **The Project record needs a first-class compliance-posture home,
  and it does not have one today.** `project.json` (`specs/NATIVE_FORMAT_SPEC.md`
  § 6.1) carries `uuid`, `name`, `created`, `modified`, `pools`,
  `schematic`, `board`, `rules`, `settings` and **nothing else**.
  No `intended_environment`, no `ipc_class`, no `regulatory_markings`,
  no `mandated_symbol_profile`, no industry vertical declaration.
  This is the largest single Datum-surface gap in Domain 4. The
  recommended addition is a `compliance` block inside `project.json`
  (and a parallel `ProjectCompliance` struct in `ENGINE_SPEC.md`)
  carrying ~10 fields. This single edit lights up downstream
  consumers across Domains 3 (symbol style), 4 (this domain), 5
  (materials posture), 6 (EMC class), 7 (PLM document control),
  and 8 (audit-trail context).

- **21 CFR Part 11 is the single most important regulated-industry
  standard for Datum because the substrate already exists.** Datum's
  deterministic transaction log + undo/redo stack + JSON
  serialisation = ~70% of what 21 CFR Part 11 (electronic records
  requirements) demand. The remaining ~30% is the **signature
  surface** (cryptographic binding of an identified user to a
  specific record state, prevention of post-signature record
  modification, audit-log entries for every signature event,
  configurable signature-meaning declarations like "Approved" /
  "Reviewed" / "Released"). This is genuinely cross-cutting with
  Domain 8 (process & quality), where the audit-log export contract
  will live. Domain 4's role is to specify which fields a compliance-
  posture project enables (signature-required-on-release; reviewer-
  count-minimum=N; etc.); Domain 8 owns the signature primitive
  itself. Recommendation: Domain 4 surfaces the compliance-posture
  field (`audit_overlay: ComplianceAuditMode`); Domain 8 will
  specify the underlying signature/audit-trail primitive. Both are
  `Deferred with prerequisite` today per
  `STANDARDS_COMPLIANCE_SPEC.md` § 4.4 and § 4.8 — that classification
  remains correct.

- **ITAR / EAR / EU Dual-Use markings are the highest-leverage
  positioning win for defence/aerospace adoption and the cheapest to
  implement.** A defence contractor evaluating Datum will not adopt
  it without an explicit ITAR/EAR position — the question is "will
  this tool exfiltrate my controlled design data?" The data-model
  cost is trivial (a struct of bool + Option<String> fields on
  Project), but the **AI-surface implication is significant**: the
  MCP layer must know that an ITAR-marked project cannot have its
  contents shared with external services (Octopart lookups, Nexar
  API calls, Anthropic API itself depending on user policy). The
  recommended pattern is a project-level `data_egress_policy` enum
  (`Unrestricted | InternalOnly | NoExternalAi | NoExternalNetwork`)
  that the MCP server consults before any outbound network call.
  This is a small Datum-side feature with very large positioning
  value: **AI-native + data-sovereignty-aware EDA tool** is a
  category position no incumbent currently occupies.

- **AEC-Q / ISO 26262 are the right-sized automotive entry point;
  AUTOSAR is correctly out of scope.** AEC-Q100/Q101/Q200 (the
  Automotive Electronics Council component qualification standards)
  are pure-metadata standards from Datum's perspective: a Part is
  either AEC-Q-qualified at a given grade (Grade 0 / 1 / 2 / 3 / 4)
  or it is not. Storing this on `Part.qualification` and exposing
  it through BOM-filter queries / AI-surface part-selection
  diagnostics is straightforward. ISO 26262 (functional safety for
  road vehicles) is a process standard with an ASIL classification
  (ASIL A / B / C / D, plus QM); Datum cannot certify ASIL but it
  can carry the project-level intended ASIL and the per-Part
  declared safety qualification. AUTOSAR (software architecture)
  has zero PCB-tool relevance — Datum can interoperate with an
  AUTOSAR system bill of materials only via BOM export, which it
  already does. Recommend: AEC-Q metadata `Planned`; ISO 26262 ASIL
  metadata `Reference-only`; AUTOSAR `Out of scope`.

- **The skip list is genuinely correct and should be ratified.**
  After deep examination, none of the eight advisory-exclusion
  standards has hidden cross-cutting value that would change the
  recommendation. DO-254 is a design-assurance process; the only
  Datum-relevant DO-254 artifact is the requirements-traceability
  matrix, and that lives in a requirements-management tool (DOORS,
  Polarion, JAMA), not an EDA tool. DO-160 is environmental
  qualification testing performed at the assembled-board level by
  fab/test labs — Datum has no input or output. MIL-PRF-31032 and
  MIL-PRF-55110 are fab QMS standards conferred on PCB manufacturers
  (TTM, Sanmina, IEC); Datum can pass through fabricator-required
  attribute data via Gerber X3 / IPC-2581 but is not a participant.
  NASA-STD-8739.x is workmanship — solder joint quality, harness
  routing, conformal coating — entirely a manufacturing concern.
  AS9100 / IATF 16949 are organisational QMS certifications; ISO
  9001 substrate already covers the audit-trail story. CMMI is an
  organisation-process maturity assessment with no tool component.
  **All eight should be promoted to formal `Out of scope` in the
  consolidated post-Domain-8 ratification pass**; § "Pending
  Exclusions (re-affirmed)" provides the per-standard rationale
  for that future ratification.

- **Medical (IEC 60601, ISO 13485, FDA Part 820, EU MDR) and
  industrial (IEC 61508) are correctly classified as
  `Reference-only` at the engine layer.** The same substrate-vs-
  certification framing applies: these are organisation-level QMS
  regimes that Datum can be substrate for but cannot certify. The
  only Datum-side work is **project-level metadata** declaring
  intended applicability (`intended_environment: MedicalClass2 |
  MedicalClass3 | IndustrialSil2 | IndustrialSil3 | ...`) and
  **per-Part qualification** (medical-grade marking; SIL-rated
  component declaration). There is no validator work, no checker
  work, no algorithmic Datum behaviour. Recommendation matches
  Batch 1's classification: `Reference-only` at the engine layer
  unless and until a milestone explicitly promotes them.

- **The cybersecurity intersection is the genuinely new finding.**
  ISO/IEC 27001 (information security management), NIST SP 800-171
  (controlled unclassified information), and CMMC (US-defence
  supply-chain cybersecurity) all converge on the question "can
  this EDA tool be deployed inside a controlled-data environment
  without becoming an exfiltration vector?" An MCP/AI-native tool
  has a much larger answerability burden here than a traditional
  desktop tool, because every MCP tool call could in principle ship
  design data to an external LLM. The recommendation is a project-
  level `data_egress_policy` field (cited under ITAR above) plus an
  MCP-layer enforcement that gates any tool with external network
  side-effects on that policy. Combined with the existing
  `Encrypted Content Handling Policy`
  (`specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy), this
  gives Datum a credible cybersecurity-conscious-AI-tool story that
  no incumbent matches. CMMC Level 2 maps to NIST 800-171
  Rev 2 controls (110 controls); Datum's substrate covers ~3 of
  these directly (audit logging, access control via OS, data
  integrity via SHA-256 transaction hashing). The other ~107
  controls are organisational/IT, not tool-level — Datum's role is
  to be **non-hostile** to those controls rather than to satisfy
  them.

- **There is a paywalled-standards problem that constrains honest
  research.** Like Domain 3, Domain 4 is dominated by paywalled
  standards: AEC-Q100/Q101/Q200 are ~USD 100 each from
  `aecouncil.com`; ISO 26262 is ~CHF 1500 for the full 12-part
  series from ISO Webstore; IEC 60601-1 is ~CHF 350 from IEC
  Webstore; ISO 13485 is ~CHF 200; IEC 61508 (7 parts) is ~CHF 1800;
  FDA 21 CFR Parts 11 / 820 are **free** at `ecfr.gov` (US federal
  regulations are not copyrighted); ITAR (22 CFR 120-130) and EAR
  (15 CFR 730-774) are also **free** at `ecfr.gov`; DO-254 is
  ~USD 250 from RTCA. The total full-standards-purchase budget for
  responsible Domain 4 implementation work is ~USD 4000-5000. The
  good news is that the public-domain US regulations (21 CFR Part
  11/820, ITAR, EAR) cover the most substrate-relevant pieces; the
  paywalled standards (AEC-Q, ISO 26262, IEC 60601, IEC 61508) are
  only needed for metadata-field naming alignment, which can be
  done from textbook digests + competitor implementations + free
  abstracts.

- **The biggest cross-domain dependency is Domain 8 (process &
  quality).** The audit-trail export surface, electronic-signature
  primitive, sign-off workflow, and reviewer/approver title-block
  fields all live in Domain 8's controlling spec. Domain 4's role
  is to **consume** Domain 8's audit-trail spec and expose
  per-vertical compliance-posture configuration on top. This means
  Domain 4's "must support" recommendations are largely
  **compliance-posture metadata** (intended environment, IPC class,
  ITAR markings, AEC-Q grades on Parts) rather than audit-trail
  primitives. The two domains must be implementation-coordinated
  but the Domain-4 work can land first if scoped to metadata only,
  with the audit-overlay claim deferred to Domain-8 spec landing.

## Standards Catalog

### Automotive

#### AEC-Q100 / Q101 / Q200

**Full title:**
- **AEC-Q100 Rev-J (2024)** — *Failure Mechanism Based Stress Test
  Qualification for Integrated Circuits*. Currently Rev-J (2024); prior
  Rev-H was 2014.
- **AEC-Q101 Rev-E1 (2021)** — *Failure Mechanism Based Stress Test
  Qualification for Discrete Semiconductors*. Rev-E1 (2021); prior
  Rev-D was 2013.
- **AEC-Q200 Rev-E (2024)** — *Stress Test Qualification for Passive
  Electrical Components*. Rev-E (2024); prior Rev-D was 2010.

**Issuing body.** **Automotive Electronics Council (AEC)** — a
consortium founded in 1994 by Chrysler, Ford, and GM (now joined by
Stellantis, Toyota, Honda, BMW, Volkswagen, Bosch, ZF, Continental,
Aptiv, etc.). Operates as a non-profit; not an ISO/IEC body.

**Scope.** AEC-Q standards specify the qualification test regime a
component must pass to be sold as "automotive grade". Each standard
defines a **grade** based on operating-temperature range:
- **Grade 0** — −40 °C to +150 °C ambient (under-hood)
- **Grade 1** — −40 °C to +125 °C ambient (most automotive)
- **Grade 2** — −40 °C to +105 °C ambient (passenger compartment)
- **Grade 3** — −40 °C to +85 °C ambient (consumer-equivalent
  thermal envelope but with automotive reliability testing)
- **Grade 4** — 0 °C to +70 °C (Q200 only; commercial-equivalent)

The standards specify accelerated-life testing
(HTOL — High Temperature Operating Life, HAST — Highly Accelerated
Stress Test), thermal-cycling, mechanical-shock, ESD-immunity,
and per-package-type qualification batches.

**Adoption status (2026).** **Mainstream and effectively mandatory**
for any IC, discrete semiconductor, or passive component sold into
the automotive supply chain. Tier-1 automotive suppliers (Bosch,
Continental, Denso, ZF, Aptiv, Magna) require AEC-Q-qualified
components on every BOM line. **The metadata is universally
declared in vendor datasheets** (TI, ADI, NXP, Infineon, ST,
Renesas, Microchip, Murata, TDK, Yageo, Samsung Electro-Mechanics,
Vishay, Rohm all carry AEC-Q grade in their data-sheet headers and
in their MPN suffix conventions, e.g., `-Q1` for Q100 Grade 1
common at TI/ADI).

**License / IP.** **Paywalled.** Each standard is sold separately
from `aecouncil.org/documents/` at roughly USD 100 per standard.
There is **no royalty-free PDF**, but the qualification-grade
metadata itself is openly described on every vendor datasheet, so
implementing AEC-Q-grade-as-Part-metadata requires no per-standard
purchase.

**EDA tool support matrix:**
- **Altium Designer** — Vault Lifecycle/Compliance plugin can carry
  AEC-Q grade as a managed-component attribute. Not built into
  Designer core; requires Vault subscription.
- **OrCAD-Capture / OrCAD X Presto** — No first-class AEC-Q
  attribute. Workaround: free-text part property.
- **Cadence Allegro / OrCAD Capture (CIS-bound)** — AEC-Q grade
  available as a CIS-database attribute when the part-database
  schema includes it (entirely user-configured).
- **PADS** — No first-class AEC-Q attribute.
- **KiCad 8.0+** — No first-class AEC-Q attribute. Convention:
  encode in `Datasheet` link or in a custom field.
- **Eagle 9 / Fusion Electronics** — No first-class AEC-Q attribute.
- **Horizon EDA** — No first-class AEC-Q attribute.
- **LibrePCB** — No first-class AEC-Q attribute.
- **DipTrace** — No first-class AEC-Q attribute.
- **EasyEDA / EasyEDA Pro** — JLCPCB parts catalog carries AEC-Q
  grade as a search-filter attribute but does not expose it through
  the project.
- **Datum (current spec)** — No first-class AEC-Q attribute. Work
  this report recommends.

**Datum coverage status.** `Reference-only` today (per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4). Recommend promotion to
`Planned` with an explicit `Part.qualification: Option<PartQualification>`
field carrying AEC-Q grade. Cost: trivial — pure metadata.

**Datum implementation cost.**
- Data model: ~50 lines of Rust (one struct, one enum, one Part
  field).
- Validator/checker: optional. A "BOM line is non-AEC-Q in an
  automotive-vertical project" warning would be useful but is
  reachable from BOM-filter queries without dedicated checker
  code.
- Export adapter: BOM export already exists; adding columns is
  one row in the BOM template.
- MCP API: 1-2 query tools (`get_part_qualification`,
  `find_parts_by_qualification`).

**Strategic recommendation.** **Implement now** alongside the
project-level `industry_vertical` setting; trivial cost, broad
benefit. AI agents can immediately answer "is every part on my BOM
AEC-Q1 qualified?" — a real question every automotive engineer
asks weekly.

**Risks.**
- **AI-surface risk:** if an AI agent suggests substituting a
  non-AEC-Q part into an automotive design without flagging the
  qualification gap, that is a design-process failure. Mitigation:
  AI-surface MUST consult `Project.compliance.industry_vertical`
  and emit a warning when a part-substitution candidate's
  qualification is below the project's intended grade. The
  diagnostic should be machine-readable so MCP tool callers can
  surface it.
- **Vendor-data accuracy risk:** vendor declarations of AEC-Q
  qualification can be wrong, partial, or change over time
  (Grade 1 at silicon launch, downgraded to Grade 2 mid-life as
  yield drops). Datum's role is to record what the vendor declared
  *at attach time*, not to second-guess; the `last_supply_chain_check`
  field already exists for refresh tracking.

#### ISO 26262 (functional safety for road vehicles)

**Full title.** **ISO 26262:2018 (12 parts)** — *Road vehicles —
Functional safety*. Current edition is the second (2018), replacing
the 2011 first edition. Each part addresses a different aspect:
Part 1 (Vocabulary), Part 2 (Management), Part 3 (Concept Phase),
Part 4 (Product Development at the System Level), **Part 5 (Product
Development at the Hardware Level — directly Datum-relevant)**,
Part 6 (Software Level), Part 7 (Production / Operation /
Decommissioning), Part 8 (Supporting Processes), Part 9 (ASIL-
Oriented Analyses), Part 10 (Guidelines), Part 11 (Semiconductors),
Part 12 (Motorcycles).

**Issuing body.** **ISO** (International Organization for
Standardization), TC 22/SC 32 (Road vehicles — Electrical and
electronic equipment).

**Scope.** ISO 26262 specifies a process-based functional-safety
regime. Items requiring safety integrity are classified into
**Automotive Safety Integrity Levels (ASIL)** — **QM** (no safety
integrity required), **ASIL A** (lowest integrity), **ASIL B**,
**ASIL C**, **ASIL D** (highest, e.g., steering-by-wire, autonomous
braking). Each ASIL level imposes more stringent requirements on
hazard analysis, failure mode coverage, redundancy, diagnostic
coverage, and design verification. Part 5 (Hardware) defines
hardware metrics (Single-Point Failure Metric — SPFM, Latent-Fault
Metric — LFM, Probabilistic Metric for Hardware Failures — PMHF)
that have direct PCB-design implications.

**Adoption status (2026).** **Mainstream and effectively mandatory**
for safety-relevant automotive electronics. Industry-wide adopted
since ~2013; Tier-1 automotive suppliers and OEMs require
ISO 26262 process compliance for any item rated ASIL B or higher.
ISO 21434 (automotive cybersecurity) is the parallel cybersecurity
standard, often invoked together.

**License / IP.** **Paywalled.** Full 12-part series from ISO
Webstore at ~CHF 1500 total; individual parts ~CHF 100-200 each.
**Part 5 (Hardware)** alone is ~CHF 200.

**EDA tool support matrix:**
- **Altium Designer** — No first-class ASIL attribute. Convention:
  custom project property.
- **OrCAD / Cadence Allegro** — No first-class ASIL attribute.
- **PADS / Siemens Xpedition** — Xpedition Enterprise has an
  Automotive Pack (paid add-on) that includes ASIL-aware
  reporting hooks; not in mainline PADS.
- **KiCad 8.0+** — No first-class ASIL support.
- **Other tools (Horizon, LibrePCB, DipTrace, EasyEDA)** — No
  first-class ASIL support.
- **Datum (current spec)** — No first-class ASIL attribute. Recommend
  project-level metadata only (per the Reference-only classification);
  algorithmic ASIL verification is fundamentally out of scope (it is
  a process-based regime requiring auditor-approved hazard analysis,
  not a tool-checkable property).

**Datum coverage status.** `Reference-only` today (per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4). The recommendation is to
**keep that classification** but surface the ASIL field on the
project-level `compliance` block introduced by this report:
`Project.compliance.intended_asil: Option<Asil>` with values
`{ QM, A, B, C, D }`. This is metadata only — Datum makes no claim
to ASIL certification, only records the intended level for downstream
documentation use.

**Datum implementation cost.** ~10 lines of Rust (one enum, one
optional Project field). No validator. No MCP tool beyond a
`get_project_compliance` reader.

**Strategic recommendation.** **Implement metadata-only**; do NOT
attempt to be an ASIL certifying tool. The hazard-analysis,
failure-mode-and-effects-analysis (FMEA), and design-verification
work that ISO 26262 requires lives in dedicated FMEA tools (Plato
SCIO-FMEA, IQ-FMEA, ReliaSoft XFMEA) and requirements-management
tools — not in EDA. Datum's role is to record the intended ASIL
in project metadata so downstream consumers (BOM filters, design-
review reports) can reason about it.

**Risks.** Low. The risk is **scope creep** — pretending Datum is
an ASIL-certifying tool when it is not. The mitigation is the
explicit `Reference-only` disposition and the absence of any
"validate_asil" MCP tool. The AI surface should NEVER claim ASIL
compliance from Datum-internal evidence alone.

#### AUTOSAR (note only)

**Full title.** **AUTOSAR (AUTomotive Open System ARchitecture)**.
Current versions: AUTOSAR Classic Platform R23-11, AUTOSAR Adaptive
Platform R23-11. Maintained by the AUTOSAR consortium (BMW, Bosch,
Continental, Daimler, Ford, GM, PSA Peugeot Citroën, Toyota,
Volkswagen, others).

**Adoption status.** Mainstream in automotive ECU software
development; not a PCB-tool concern.

**Scope and Datum relevance.** AUTOSAR specifies a software
architecture: Basic Software (BSW), Runtime Environment (RTE),
Application layer. **No PCB-tool relevance.** The intersection
with hardware design is at the level of "this BOM line provides
the MCU that runs the AUTOSAR stack" — purely informational.

**License / IP.** AUTOSAR specifications are free for AUTOSAR
member companies; non-members can read partner-specification
PDFs from `autosar.org` (free, no registration for many parts;
member-restricted for advanced parts).

**Datum coverage status.** `Out of scope`. Recommend formal
classification under Domain 4. AUTOSAR-aware BOM annotation
(e.g., "this MPN provides AUTOSAR Classic R20-11 BSW support")
is plausible as a `Part.parametric` key but is below the
metadata-recommendation threshold.

**Strategic recommendation.** **Out of scope.** Document
explicitly in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4 alongside the
AS9100 / DO-254 exclusions.

### Aerospace / Defence (mostly skip; cover as positioning context)

#### DO-254 (skip — substrate positioning only)

**Full title.** **RTCA DO-254 / EUROCAE ED-80** — *Design
Assurance Guidance for Airborne Electronic Hardware*. Current
edition is the original 2000 publication (active; minor errata
since); RTCA has not issued a revision (DO-254A is in committee
discussion as of 2025 but not published).

**Issuing body.** **RTCA** (Radio Technical Commission for
Aeronautics, US) and **EUROCAE** (European Organisation for Civil
Aviation Equipment) jointly. The standard is referenced by FAA
Advisory Circular AC 20-152 and EASA CM-SWCEH-001 for civil
aircraft hardware certification.

**Scope.** DO-254 specifies a process for design assurance of
custom airborne electronic hardware (FPGAs, CPLDs, ASICs, complex
PLDs). It defines five **Design Assurance Levels (DAL)** — DAL A
(most critical, catastrophic failure consequence), DAL B
(hazardous), DAL C (major), DAL D (minor), DAL E (no effect).
Each DAL imposes progressively more stringent requirements on
requirements traceability, design verification, configuration
management, and process auditing.

**Why skip.** **DO-254 is process-grade certification, not a
tool-checkable property.** Compliance is conferred on a project
team by an independent designated engineering representative (DER)
auditor on behalf of the FAA / EASA. The certifying party is the
DER, never the EDA tool. DO-254-relevant artifacts (Hardware
Requirements Specification, Hardware Verification Plan,
Hardware Configuration Index, Hardware Accomplishment Summary)
live in requirements-management tools (DOORS, Polarion, JAMA),
verification tools (Mentor Questa, Cadence Xcelium, Synopsys
VCS), and configuration-management tools (ClearCase, Git+process)
— none are EDA-tool concerns.

**Substrate positioning.** Datum can be **substrate-compatible
with a DO-254 workflow** by virtue of:
1. **Deterministic transaction log** (`docs/CANONICAL_IR.md` § 4)
   — provides immutable change history matching DO-254
   configuration-management expectations.
2. **JSON serialisation contract** (`specs/ENGINE_SPEC.md` § 4)
   — provides text-diffable artifacts auditable via standard
   diff/version-control review.
3. **Encrypted Content Handling Policy** (`specs/MCP_API_SPEC.md`)
   — preserves vendor-encrypted IBIS/SPICE models without re-
   encrypting or decrypting, matching DO-254 supplier-IP
   handling expectations.

That is the entire DO-254 positioning. Recommendation:
**`Out of scope`** in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4; surface
the substrate-compatibility note as a documentation paragraph
referenced from the disposition.

**Re-affirmed exclusion.** No hidden cross-cutting value. The
substrate already exists; Datum makes no certification claim.

#### DO-178C (note only — software, not hardware)

**Full title.** **RTCA DO-178C / EUROCAE ED-12C** — *Software
Considerations in Airborne Systems and Equipment Certification*.
2011 publication; supplements DO-330 (Tool Qualification
Considerations), DO-331 (Model-Based Development), DO-332
(Object-Oriented Technology), DO-333 (Formal Methods).

**Datum relevance.** **None directly.** DO-178C governs avionics
software, not hardware. It pairs with DO-254 in mixed
hardware/software certifications but is a software-engineering
process standard.

**Recommendation.** **Out of scope.** Document only as a
DO-254 companion standard for context.

#### ARP4754A (note only)

**Full title.** **SAE ARP4754A / EUROCAE ED-79A** — *Guidelines
for Development of Civil Aircraft and Systems*. 2010 publication
(current); supersedes ARP4754 (1996). Covers aircraft / system
development life cycle including hazard analysis, requirements
flow-down, and validation/verification process — operates one
level above DO-254 (hardware) and DO-178C (software).

**Datum relevance.** None directly. System-level process standard.

**Recommendation.** **Out of scope.**

#### NASA-STD-8739.x family (skip)

**Full title.** **NASA Workmanship Standards** family:
- **NASA-STD-8739.1A (2024)** — Polymeric Application
- **NASA-STD-8739.2A (2018)** — Surface Mount Technology Workmanship
- **NASA-STD-8739.3A (2018)** — Through-Hole Technology Workmanship
- **NASA-STD-8739.4A (2018)** — Crimping, Interconnecting Cables,
  Harnesses, and Wiring
- **NASA-STD-8739.6 (2017)** — Implementation Requirements for
  NASA Workmanship Standards
- **NASA-STD-8739.10 (2018)** — Fiber Optic Workmanship

**Why skip.** **NASA-STD-8739.x is workmanship — solder joints,
cable harnessing, conformal coating application, fiber-optic
termination quality.** All are post-fabrication / assembly
concerns. Datum produces design data; the workmanship standards
govern how the contract assembler executes against that design.
Zero EDA-tool intersection.

**Substrate positioning.** None applicable. Datum's IPC-A-610
class metadata (Class 3, Class 3A) is the closest equivalent,
and IPC-A-610 is the commercial standard NASA-STD-8739 closely
parallels. Recommend: pass-through IPC class to fab via Gerber X3
attributes (covered by IPC research); no NASA-specific Datum
work.

**Re-affirmed exclusion.** No hidden cross-cutting value.

#### MIL-PRF-31032 / MIL-PRF-55110 (skip)

**Full title.**
- **MIL-PRF-31032D (2017)** — *Printed Circuit Board / Printed
  Wiring Board, General Specification for*
- **MIL-PRF-55110G (2018)** — *Printed Wiring Board, Rigid,
  General Specification for*

**Issuing body.** **US DoD** (Defense Logistics Agency).

**Why skip.** Both are **fabricator QMS standards**. MIL-PRF-31032
is the qualification regime that allows a PCB fabricator to claim
DoD-qualified fabrication; MIL-PRF-55110 is the older rigid-PWB
sub-specification (largely superseded by IPC-6012 Class 3/A but
still cited in legacy DoD contracts). Datum can pass through
material-stackup, drill-table, and class-marking data via
Gerber X3 / IPC-2581 / ODB++ exports (covered by IPC research),
but the certification belongs to the fabricator, not the design
tool.

**Substrate positioning.** Datum's stackup material properties
(`StackupLayer.dielectric_constant`, `loss_tangent`,
`copper_weight_oz`, `roughness_um`, `material_name` per
`specs/ENGINE_SPEC.md` § 1.3 — all post-Batch-1) provide the
material-declaration substrate fabricators need. Pass-through
only; no validation.

**Re-affirmed exclusion.** No hidden cross-cutting value.

#### MIL-STD-275 (legacy)

**Full title.** **MIL-STD-275E** — *Printed Wiring for
Electronic Equipment*. **Cancelled 1994**, superseded by
IPC-2221 (Generic Standard on Printed Board Design). Still cited
in legacy DoD contracts and in MIL-STD-2118 / MIL-HDBK-275
companion documents.

**Datum relevance.** None directly. IPC-2221 (current substitute)
is already in scope under Domain 5 / IPC research as the
clearance-policy basis (`STANDARDS_COMPLIANCE_SPEC.md` § 5.5).

**Recommendation.** **Reference-only**; document as legacy
predecessor of IPC-2221 in `STANDARDS_COMPLIANCE_SPEC.md` for
historical context only.

#### MIL-STD-1772 (note)

**Full title.** **MIL-STD-1772B** — *Certification Requirements
for Hybrid Microcircuit Facilities and Lines*. **Cancelled 2010**;
superseded by industry IPC-7095 / IPC-7711/7721 family for
rework, and JESD22 / MIL-PRF-38535 for component qualification.

**Datum relevance.** None directly.

**Recommendation.** **Out of scope.**

#### CMMC (emerging — substrate-relevant)

**Full title.** **Cybersecurity Maturity Model Certification
(CMMC) 2.0** (US Department of Defense, finalised 32 CFR Part 170
in October 2024; phased implementation through 2028). Defines
three certification levels:
- **Level 1 — Foundational** (17 controls; basic safeguarding of
  Federal Contract Information; self-attestation)
- **Level 2 — Advanced** (110 controls per NIST SP 800-171
  Rev 2; protection of Controlled Unclassified Information;
  third-party C3PAO assessment for prioritised acquisitions, self-
  attestation otherwise)
- **Level 3 — Expert** (110+ NIST 800-171 controls plus subset of
  NIST SP 800-172; for the most sensitive programs; DIBCAC
  government assessment)

**Issuing body.** **US Department of Defense**, in coordination
with the Cyber AB (CMMC Accreditation Body). Codified as
**32 CFR Part 170**.

**Scope.** Information-systems and process security. Applies to
any organisation in the Defense Industrial Base (DIB) handling
Federal Contract Information (FCI) or Controlled Unclassified
Information (CUI). Compliance is **organisation-level** and
**information-system-level**, not tool-level — but it imposes
constraints on **what tools can be used** in an environment that
processes CUI.

**Adoption status (2026).** **Emerging-mainstream.** Phased
adoption since October 2024; full DoD-contract-clause invocation
expected through 2025-2028. Already widely cited in DIB
procurement.

**License / IP.** **Free.** 32 CFR Part 170 is public-domain US
federal regulation, available at `ecfr.gov`. NIST SP 800-171
Rev 2 (the underlying control framework) is also free from
`csrc.nist.gov`.

**EDA tool support matrix.** No EDA tool currently advertises CMMC
substrate features explicitly. The closest analogue is **on-
premises deployment posture** (Cadence, Mentor/Siemens, Altium
Vault all offer on-prem deployment for CUI-handling
environments; cloud SaaS variants like Altium 365 Cloud, Fusion
Electronics Cloud, EasyEDA Cloud are categorically incompatible
with CUI handling without substantial additional architecture).

**Substrate positioning.** Datum's relevant primitives:
1. **Deterministic transaction log** — supports NIST 800-171
   3.3.1 (audit and accountability — generation of audit
   records).
2. **SHA-256 transaction integrity** — supports 3.8.6 (data
   integrity).
3. **Pool-local file storage** — supports 3.13.1 (boundary
   protection — by being non-cloud by default).
4. **Encrypted Content Handling Policy** — supports 3.8.9
   (protect confidentiality of CUI at rest, when extended to
   project content) and aligns with 3.13.16 (protect
   confidentiality of CUI at rest).
5. **MCP layer with local Unix-socket transport** — supports
   3.13.5 (implement subnetworks for publicly accessible system
   components).

**The single largest CMMC-substrate gap is data egress posture.**
An MCP/AI-native tool that ships design data to an external LLM
service (Anthropic, OpenAI, Google) is fundamentally incompatible
with CUI-handling environments. Datum needs a project-level
**`data_egress_policy`** declaration (covered in detail under
ITAR below) to be deployable in a CMMC environment.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
with `Reference-only` disposition; document the substrate-
compatibility paragraph; add the `data_egress_policy` field as a
hard requirement for compliance posture.

**Datum implementation cost.**
- Data model: `data_egress_policy` enum on `Project.compliance`
  — ~10 lines.
- MCP-layer enforcement: every tool with an external network
  side-effect (Octopart lookup, Nexar lookup, Digi-Key lookup,
  Mouser lookup, distributor refresh, AI-routed tools) must
  consult the policy and refuse-or-warn when the project's
  policy disallows. ~50 lines per affected tool, ~10 affected
  tools = ~500 lines.
- Validator/checker: zero (this is enforcement, not validation).
- Test surface: each enforced tool needs a "policy-blocked"
  test case.

**Strategic recommendation.** **Implement now** as part of the
ITAR/EAR posture work; the same `data_egress_policy` field
serves both. CMMC adoption is mainstream-emerging in 2026; getting
the substrate posture right early is much cheaper than retrofitting.

**Risks.** **AI-surface risk:** if an AI agent in autonomous mode
calls an external lookup tool without consulting the egress
policy, that is a CMMC-violating data exfiltration. Mitigation:
the policy check must be an MCP-server-layer mandatory gate, not
an opt-in tool argument.

### Medical

#### IEC 60601 (multi-part)

**Full title.** **IEC 60601-1:2005+A1:2012+A2:2020** — *Medical
electrical equipment — Part 1: General requirements for basic
safety and essential performance*. The base standard plus collateral
standards (60601-1-2 EMC, 60601-1-6 Usability, 60601-1-8 Alarms,
60601-1-9 Environmentally conscious design, 60601-1-10
Physiological closed-loop controllers, 60601-1-11 Home healthcare,
60601-1-12 Emergency medical services environment) and many
particular standards (60601-2-N for specific equipment classes:
2-2 electrosurgical, 2-3 cardiac defibrillators, 2-4 cardiac
monitors, etc., over 90 particular standards).

**Issuing body.** **IEC** (International Electrotechnical
Commission), TC 62 (Medical equipment).

**Scope.** Safety and essential-performance requirements for
medical electrical equipment. Covers electrical safety (leakage
currents, dielectric strength, patient-applied parts
classification — Type B, Type BF, Type CF), mechanical safety,
EMC (via 60601-1-2), risk management (via ISO 14971), software
(via IEC 62304), usability (via IEC 62366).

**Datum relevance.** Compliance is conferred on a manufactured
medical device by a Notified Body (in the EU) or by FDA 510(k) /
PMA process (in the US), never on a design tool. Datum can carry
project-level metadata declaring the intended IEC 60601-1
applicability (Type B / BF / CF patient-applied-part class;
Class I / II / IIa / IIb / III device classification per EU MDR;
intended sub-particular standard) but cannot certify against the
standard.

**Adoption status (2026).** Mainstream-mandatory for any medical
electrical equipment marketed globally.

**License / IP.** **Paywalled.** IEC 60601-1 base standard ~CHF
350; collateral standards ~CHF 100-200 each; particular standards
~CHF 100-300 each. Full medical-equipment library purchase budget
is ~CHF 5000+.

**EDA tool support matrix.** No EDA tool first-class-supports
IEC 60601 metadata. Workaround in all tools: free-text project
property.

**Datum coverage status.** `Reference-only` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4. **Keep this classification.**
Add the project-level metadata field
`intended_environment: IntendedEnvironment` with the
`MedicalClass1 | MedicalClass2 | MedicalClass3` variants; further
sub-classification (Type B/BF/CF) lives in
`Project.compliance.medical_subtype: Option<String>`.

**Datum implementation cost.** ~5 lines of Rust (an enum variant +
a free-text field). No validator. No MCP tool beyond the
project-compliance reader.

**Strategic recommendation.** **Implement metadata-only**; do
NOT attempt to be a medical-device certifying tool. The risk of
implying certification authority is severe.

**Risks.** **Implied-certification risk** is the only meaningful
risk. The AI surface MUST NEVER claim "this design is IEC 60601-1
compliant" — the most it can say is "this project is configured
with intended IEC 60601-1 applicability MedicalClass2; certification
must be obtained from your accredited Notified Body". This is a
hard rule for the MCP-layer wording.

#### ISO 13485 (medical-device QMS)

**Full title.** **ISO 13485:2016** — *Medical devices — Quality
management systems — Requirements for regulatory purposes*.
Current edition is 2016 with Amendment 1:2021 (clarifications);
ISO 13485:2025 is in committee but not yet published.

**Issuing body.** **ISO** TC 210 (Quality management and
corresponding general aspects for medical devices).

**Scope.** Process-based QMS standard for medical-device
manufacturers. Aligned with ISO 9001 framework but with
additional medical-device-specific requirements (design control,
risk management per ISO 14971, traceability, document control,
post-market surveillance).

**Datum relevance.** Same substrate-vs-certification framing as
IEC 60601. ISO 13485 is conferred on a manufacturing organisation
by an accredited certification body (BSI, TÜV SÜD, DNV, DEKRA,
etc.). Datum's role is **substrate**: deterministic transaction
log + future audit-trail-export surface + reviewer/approver title-
block fields.

**Adoption status (2026).** Mainstream-mandatory for medical-
device manufacturers globally.

**License / IP.** **Paywalled.** ~CHF 200 from ISO Webstore.

**EDA tool support matrix.** No EDA tool first-class-supports
ISO 13485 metadata. Some tools (Altium Vault, Cadence Allegro
Component Information System) support sign-off workflows that map
loosely onto ISO 13485 design-control requirements.

**Datum coverage status.** `Reference-only` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4. **Keep this classification.**
Substrate posture is identical to ISO 9001 (Domain 8); medical-
specific recommendation is to expose `intended_environment:
MedicalClassN` so the audit-overlay can be configured for medical-
appropriate signature meanings.

**Datum implementation cost.** Zero beyond the project-compliance
field already recommended.

**Strategic recommendation.** **Reference-only** at engine layer;
substrate posture is **shared with Domain 8** (audit-trail
surface). Once Domain 8 specifies the audit-trail-export
contract, Domain 4 should formally consume it for medical-
project audit-overlay enablement.

**Risks.** Implied-certification risk; same mitigation as
IEC 60601.

#### FDA 21 CFR Part 820 (US Quality System Regulation)

**Full title.** **21 CFR Part 820** — *Quality System Regulation
(QSR)*. Current as of 2026; FDA published a final rule in
February 2024 amending Part 820 to **incorporate ISO 13485:2016
by reference**, with full implementation by February 2026 (the
"Quality Management System Regulation" / QMSR). For US
medical-device manufacturers, Part 820 / QMSR ≈ ISO 13485 with
some FDA-specific additions (UDI requirements, MDR reporting,
etc.).

**Issuing body.** **US FDA** (Food and Drug Administration), Center
for Devices and Radiological Health (CDRH).

**Scope.** US-specific QMS regulation for medical-device
manufacturers. Post-2026 QMSR alignment substantially merges with
ISO 13485 framework, with the regulatory authority remaining FDA.

**License / IP.** **Free.** US federal regulations are public
domain at `ecfr.gov/current/title-21/chapter-I/subchapter-H/part-820`.

**Datum coverage status.** `Reference-only` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4. **Keep this classification.**
Substrate posture is identical to ISO 13485.

**Datum implementation cost.** Zero beyond the project-compliance
field already recommended.

**Strategic recommendation.** **Reference-only**; document
explicitly the post-2026 QMSR / ISO 13485 alignment so medical
users understand a single substrate posture serves both.

**Risks.** Same as ISO 13485.

#### FDA 21 CFR Part 11 (electronic records and signatures — substrate-relevant)

**Full title.** **21 CFR Part 11** — *Electronic Records;
Electronic Signatures*. Effective March 1997; FDA Guidance for
Industry (2003) provided risk-based scoping ("Scope and
Application"); current 2024 FDA Draft Guidance "Electronic
Systems, Electronic Records, and Electronic Signatures in
Clinical Investigations" extends application context.

**Issuing body.** **US FDA** (Food and Drug Administration).

**Scope.** Specifies criteria under which electronic records and
electronic signatures are considered trustworthy, reliable, and
generally equivalent to paper records and handwritten
signatures. Applies to any FDA-regulated industry (pharmaceutical,
medical-device, biologics, food). Two main subparts:
- **Subpart B — Electronic Records**: requires validation,
  audit-trail generation (independently generated, time-
  stamped, secure), record protection, limits on system access,
  authority checks, source-data accuracy, training, and
  documentation control.
- **Subpart C — Electronic Signatures**: requires unique
  signature identification (each signature linked to a unique
  individual), signature manifestation (printed name, date,
  meaning), signature linking to the record (cryptographic
  binding preventing record modification post-signature), and
  signature documentation (organisational policies on
  authentication and accountability).

**Adoption status (2026).** Mainstream-mandatory for any
electronic record or signature in an FDA-regulated context.

**License / IP.** **Free.** Public-domain US federal regulation
at `ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11`.
The 2003 Guidance for Industry (PDF, free from `fda.gov`) is
the de-facto interpretation reference.

**EDA tool support matrix:**
- **Altium Designer** with Altium Vault — claims 21 CFR Part 11
  support via Vault sign-off workflow; requires Vault paid
  subscription and proper deployment.
- **Cadence Allegro** with Pulse / OrCAD CIS / Allegro Design
  Workbench — integrates with PLM (Windchill, Teamcenter) for
  21 CFR Part 11 evidence; the EDA tool itself is not the
  certifying party.
- **Mentor / Siemens Xpedition** with Teamcenter — same pattern.
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** —
  no 21 CFR Part 11 sign-off feature; reliance on git history
  is the closest substrate.
- **Datum (current spec)** — substrate exists (transaction
  model); no signature surface yet. The Phase 1 audit identified
  this as the "21 CFR Part 11 / electronic-signature audit-trail
  compliance is implicitly enabled by Datum's deterministic
  transaction model but not explicitly claimed".

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4. **Keep this classification**
because the prerequisite (audit-trail-export surface with user
identity, timestamps, rationale, optional signature state) is
genuinely Domain-8 owned. Once Domain 8 lands the audit-trail
spec, Domain 4 can move 21 CFR Part 11 to `Planned` for the
medical-project profile.

**Substrate already present (Datum-side).**
- **Validation (§ 11.10(a)).** Datum's deterministic transaction
  model is validated by integration tests; serialisation
  determinism is gated by `tests/golden/`.
- **Audit-trail generation (§ 11.10(e)).** `Transaction { id,
  operations, description }` (`specs/ENGINE_SPEC.md:865`)
  captures the operation log. **Missing fields**: timestamp
  (currently absent from the persisted Transaction), acting-
  user identity, optional rationale beyond the description
  string. These are Domain-8 owned; this report flags them as
  consumed-from-Domain-8 prerequisites.
- **Record protection (§ 11.10(c)).** JSON serialisation +
  SHA-256 hashing + git-friendly diffability give cryptographic
  record integrity. Pool architecture's per-file UUID
  organisation supports tamper-detection at the file level.
- **Limits on system access (§ 11.10(d)).** Operating-system-
  level access control; not Datum-level. Acceptable for the
  CFR's "appropriate personnel" interpretation given typical
  desktop-engineering deployments.

**Substrate gaps to specify (Domain 8 owned, Domain 4 consumes).**
1. **Timestamp ownership** — every Transaction must carry a
   wall-clock timestamp (ISO 8601 UTC). Currently absent.
2. **Acting-user identity** — every Transaction must carry an
   identifier for the user who initiated it. Currently absent.
3. **Rationale field** — recommended freeform text field per
   Transaction beyond the operation `description`.
4. **Signature surface** — cryptographic binding of an identified
   user to a specific design state. Recommended primitive:
   detached-signature record over the project-state SHA-256 hash,
   stored alongside the project file (e.g.,
   `signatures/<transaction-uuid>.signature.json`); signature
   validation tool in MCP returns signer identity + signature
   timestamp + signature meaning.
5. **Signature meaning declaration** — the signature record must
   carry a "meaning" enum
   (`Authored | Reviewed | Approved | Released | Witnessed`)
   per § 11.50(a)(2).
6. **Audit-trail query/export** — the MCP layer must expose
   a query tool returning the transaction log filterable by
   date range / user / operation type / object UUID.
7. **Audit-trail tamper detection** — the audit log itself
   must be a sequence of records each cryptographically
   chained to its predecessor (Merkle-tree or hash-chain
   pattern).

**Datum implementation cost.**
- Domain 8 owns the audit-trail spec primitive; effort estimate
  there is ~2-3 weeks for the contract and ~3-4 weeks for the
  signature primitive implementation.
- Domain 4's incremental cost: project-level
  `audit_overlay: AuditOverlayMode` enum
  (`{ None, BasicAuditTrail, SignatureRequired,
  SignatureRequiredWithMeaning }`) + per-MCP-tool consultation of
  this setting before signature-required actions. ~100 lines once
  the underlying primitive lands.

**Strategic recommendation.** **Implement metadata + audit-overlay
configuration now** (Domain 4 surface); **defer signature primitive
to Domain 8** (where it belongs). Once Domain 8 lands signatures,
Datum can credibly claim 21 CFR Part 11 substrate-compatibility for
medical-vertical projects. Marketing this as "21 CFR Part 11
**substrate-compatible**" (not "21 CFR Part 11 **compliant**" —
that wording is reserved for the certifying party) is a credible
positioning win for medical-device customers.

**Risks.** **AI-surface risk:** if an AI agent claims "this project
is 21 CFR Part 11 compliant" the user could rely on that statement
in an FDA submission. The MCP wording rule MUST be: "this project
uses Datum's 21 CFR Part 11 substrate features
(`audit_overlay: SignatureRequiredWithMeaning`); compliance
certification is conferred by your QMS, not by Datum".

**Cybersecurity intersection.** 21 CFR Part 11 § 11.30 requires
"appropriate controls over the systems documentation" — overlaps
ISO 27001 / CMMC controls. The same `data_egress_policy` field
mitigates the cross-cutting risk.

#### EU MDR 2017/745 (note)

**Full title.** **Regulation (EU) 2017/745** — *Medical Device
Regulation (MDR)*. Replaces the prior MDD (Medical Devices
Directive 93/42/EEC) and AIMDD (Active Implantable Medical Devices
Directive 90/385/EEC). In force May 2021 (delayed from May 2020
by COVID-19 amendment); transition periods extended several times,
with the most recent extension to 2027/2028 for legacy
certificates.

**Datum relevance.** Same substrate-vs-certification framing as
IEC 60601 / ISO 13485 / FDA Part 820. EU MDR introduces some
novel requirements (Unique Device Identification UDI database
EUDAMED submission, post-market surveillance reporting, clinical
evaluation evidence) but these are organisation-level, not tool-
level. Datum can carry the EU MDR risk class
(`MedicalClass1 | MedicalClass2a | MedicalClass2b | MedicalClass3`)
on the project-level `intended_environment` field, mirroring the
FDA classification approach.

**License / IP.** **Free.** EU regulations are public-domain from
`eur-lex.europa.eu`.

**Datum coverage status.** `Reference-only` (treat as part of
medical-vertical metadata posture).

**Strategic recommendation.** **Metadata-only**; same shape as
ISO 13485 / FDA Part 820.

### Industrial

#### IEC 61508 (multi-part — SIL classification)

**Full title.** **IEC 61508 (7 parts), 2nd Edition (2010)** —
*Functional safety of electrical/electronic/programmable
electronic safety-related systems*. Part 1 (General requirements),
Part 2 (Hardware), Part 3 (Software), Part 4 (Definitions),
Part 5 (Examples of methods for the determination of safety
integrity levels), Part 6 (Guidelines on the application of
Parts 2 and 3), Part 7 (Overview of techniques and measures).

**Issuing body.** **IEC**, TC 65 (Industrial-process measurement,
control and automation).

**Scope.** Process-based functional-safety standard for
industrial electrical/electronic/programmable electronic safety
systems. Defines **Safety Integrity Levels (SIL)** — SIL 1
(lowest integrity), SIL 2, SIL 3, SIL 4 (highest). Aligned in
spirit with ISO 26262 (automotive ASIL) but for general
industrial applications. Sector-specific derivatives include
IEC 61511 (process industry), IEC 62061 (machinery), IEC 60601-1
(medical electrical), EN 50128 (railway).

**Adoption status (2026).** Mainstream-mandatory for industrial
safety systems globally; widely cited in oil-and-gas,
petrochemical, power-generation, machinery, and rail-transport
contracts.

**License / IP.** **Paywalled.** Full 7-part series ~CHF 1800
from IEC Webstore.

**Datum relevance.** Same substrate-vs-certification framing as
ISO 26262. Compliance conferred on an organisation by an
accredited certification body. Datum's role: project-level
metadata declaring intended SIL
(`Project.compliance.intended_sil: Option<Sil>` with
`{ SilOne, SilTwo, SilThree, SilFour, NoSilDeclared }`).

**EDA tool support matrix.** No EDA tool first-class-supports
SIL metadata; same workaround pattern as ISO 26262.

**Datum coverage status.** `Reference-only` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4. **Keep this classification.**

**Strategic recommendation.** **Metadata-only**; identical
shape to ISO 26262 ASIL.

**Risks.** Implied-certification risk; same mitigation as
ISO 26262.

#### IEC 62443 (industrial automation cybersecurity — note)

**Full title.** **IEC 62443 (multi-part)** — *Industrial
communication networks — IT security for networks and systems*.
Multi-part standard published 2010-2024 covering cybersecurity for
industrial automation and control systems (IACS). Defines
**Security Levels (SL)** — SL 1 through SL 4.

**Datum relevance.** Industrial control systems cybersecurity is
**at the deployed-system level**, not the EDA-tool level. Datum
has no direct intersection. The same `data_egress_policy` field
that addresses CMMC also serves IEC 62443 for industrial-vertical
projects.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
with `Reference-only` disposition; document `data_egress_policy`
field as the operative substrate.

#### ATEX directive (note)

**Full title.** **Directive 2014/34/EU (ATEX 114)** — *Equipment
intended for use in potentially explosive atmospheres*. Plus
**Directive 1999/92/EC (ATEX 153)** — workplace requirements.
US/Canadian counterpart: **NEC Class I Division 1/2** zoning
plus **CSA / FM** approvals.

**Issuing body.** **EU** (European Commission).

**Scope.** Defines equipment categories
(Category 1 / 2 / 3 — descending criticality) and zone
classifications (Zone 0 / 1 / 2 — descending explosive-atmosphere
likelihood) for hazardous-location equipment. ATEX-certified
components carry markings like "II 2 G Ex db IIC T4 Gb" indicating
equipment group, category, gas-protection method, gas group,
temperature class.

**Datum relevance.** **Component-level metadata.** ATEX-certified
parts have specific markings; carrying the ATEX marking on a
`Part.qualification` extension would be useful for hazardous-area
designs. Project-level `intended_environment: HazardousArea` would
flag the design context.

**License / IP.** **Free.** EU directives are public-domain from
`eur-lex.europa.eu`.

**EDA tool support matrix.** No EDA tool first-class-supports
ATEX metadata. Pattern: free-text part property.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
with `Reference-only` disposition; surface as a
`Part.qualification.atex_certification: Option<String>` field.

**Strategic recommendation.** **Metadata-only**, low-priority.
Implement on-demand if a hazardous-area customer surfaces.

### Export / Trade Controls

#### ITAR (US International Traffic in Arms Regulations)

**Full title.** **22 CFR Parts 120-130** — *International Traffic
in Arms Regulations*. Implements the Arms Export Control Act
(22 USC § 2778). Continuously updated; current as of 2026.

**Issuing body.** **US Department of State**, Directorate of
Defense Trade Controls (DDTC).

**Scope.** Controls export and import of defense articles and
defense services. The **United States Munitions List (USML)**,
codified at 22 CFR Part 121, defines what is controlled in 21
categories (I — Firearms, II — Guns and Armament, III —
Ammunition/Ordnance, IV — Launch Vehicles/Missiles/Rockets,
V — Explosives/Energetic Materials/Propellants, VI — Vessels of
War/Special Naval Equipment, VII — Ground Vehicles, VIII —
Aircraft and Related Articles, IX — Military Training Equipment,
X — Personal Protective Equipment, XI — Military Electronics,
XII — Fire Control/Range Finder/Optical/Guidance and Control
Equipment, XIII — Materials and Miscellaneous Articles, XIV —
Toxicological Agents/Equipment/Radiological Equipment,
XV — Spacecraft Systems and Related Articles, XVI — Nuclear
Weapons Related Articles, XVII — Classified Articles/Technical
Data/Defense Services, XVIII — Directed Energy Weapons,
XIX — Gas Turbine Engines/Associated Equipment, XX — Submersible
Vessels/Oceanographic and Related Articles, XXI — Articles/
Technical Data/Defense Services Not Otherwise Enumerated).

**Adoption status (2026).** Mainstream-mandatory for any US
person dealing in USML-listed articles. **Penalties are severe**
(criminal prosecution; civil penalties up to USD 1.6M per
violation; statutory imprisonment up to 20 years).

**License / IP.** **Free.** Public-domain US federal regulation
at `ecfr.gov/current/title-22/chapter-I/subchapter-M`.

**EDA tool support matrix:**
- **Altium Designer** — No first-class ITAR marking. Workaround:
  custom project property; document classification at the project
  level via title-block.
- **Cadence Allegro / OrCAD** — No first-class ITAR marking;
  Cadence's secure-on-prem deployment posture is the
  substrate-relevant feature.
- **Mentor / Siemens** — Same. Xpedition Enterprise has document-
  classification-marking templates in the title-block engine.
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** —
  No first-class ITAR support; cloud-SaaS variants of these tools
  are categorically incompatible with ITAR-controlled work.
- **Datum (current spec)** — No first-class ITAR support. Critical
  recommendation of this report.

**The data-egress concern is the central ITAR question for an
AI-native EDA tool.** ITAR § 120.50 defines an "export" as the
release of technical data to a foreign person, including by
transmission. Releasing ITAR-controlled technical data to
external services (including cloud LLMs not domiciled in the US
and not subject to US-citizen access controls) is an export and
requires a license. **An MCP/AI-native EDA tool that ships
project data to an external LLM service without explicit ITAR-
controlled deployment is fundamentally incompatible with
ITAR-controlled work.** The recommended posture:
1. Project-level **`itar_controlled: bool`** declaration.
2. Project-level **`data_egress_policy`** enum
   (`{ Unrestricted, InternalOnly, NoExternalAi,
   NoExternalNetwork }`) defaulting to `Unrestricted` for
   non-ITAR projects but to `NoExternalNetwork` for
   ITAR-controlled projects.
3. **MCP-layer enforcement** — every tool with an external
   network side-effect (Octopart lookup, Nexar lookup,
   distributor lookup, AI-routed external-LLM tools) must
   consult `data_egress_policy` and refuse-or-warn appropriately.
4. **Project-marking visualisation** — ITAR-controlled projects
   carry visible markings on every rendered output (title-block,
   sheet borders, exported BOM headers, Gerber attributes). The
   marking text is configurable
   (`Project.compliance.export_control_markings: Vec<String>`)
   to support shop-specific wordings.

**Datum coverage status.** `Planned` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4 ("ITAR and EAR marking
posture: `Planned` as project metadata and export guard policy,
not as certification"). **Confirm classification** and provide
the data-model contract for it.

**Datum implementation cost.**
- Data model: `Project.compliance.itar_controlled: bool`,
  `Project.compliance.export_control_markings: Vec<String>`,
  `Project.compliance.data_egress_policy: DataEgressPolicy` enum
  — ~30 lines.
- MCP-layer enforcement: ~50 lines per affected tool, ~10
  affected tools = ~500 lines plus a shared enforcement helper
  module.
- Validator/checker: a "no ITAR-controlled project ever calls
  external network tools" gate test — ~100 lines.
- Title-block / export-attribute integration: requires Domain 3
  `SheetFrame.classification` (already proposed in
  schematic-drawing-conventions deep-dive).

**Strategic recommendation.** **Implement now.** This is the
single highest-leverage Domain 4 implementation: trivial data
model, large positioning value (AI-native + data-sovereignty-
aware EDA tool), credible cybersecurity story for defence
adoption.

**Risks.**
- **AI-surface risk (severe):** an AI agent that exfiltrates
  ITAR-controlled data is a federal crime. The MCP-layer gate
  MUST be mandatory and cannot be bypassed by an unprivileged
  AI agent. Mitigation: the gate is enforced at the MCP-server
  boundary (Python), not at the tool-implementation boundary
  (Rust); even a malicious Rust-side tool cannot bypass the
  Python-layer policy check.
- **Pass-through-marking risk:** if the title-block ITAR marking
  is not propagated to derived outputs (Gerber, IPC-2581, BOM,
  PDF), an exported file could shed its classification. Mitigation:
  the `export_control_markings` field MUST flow through every
  export adapter; this is an export-spec edit per Domain 1.

#### EAR / ECCN (US Export Administration Regulations)

**Full title.** **15 CFR Parts 730-774** — *Export Administration
Regulations (EAR)*. Implements the Export Administration Act and
the International Emergency Economic Powers Act.

**Issuing body.** **US Department of Commerce**, Bureau of
Industry and Security (BIS).

**Scope.** Controls export of dual-use items (commercial items
with potential military applications). Items are classified by
**Export Control Classification Number (ECCN)** — five-character
codes like `5A002.a.1` (information security, hardware,
cryptographic). The **Commerce Control List (CCL)**, codified at
15 CFR Part 774, is the authoritative ECCN catalog; entries map
to control reasons (NS — National Security, AT — Anti-Terrorism,
CC — Crime Control, MT — Missile Technology, NP — Nuclear
Nonproliferation, etc.).

**Adoption status (2026).** Mainstream-mandatory for any US
person dealing in CCL-listed items.

**License / IP.** **Free.** Public-domain US federal regulation
at `ecfr.gov/current/title-15/subtitle-B/chapter-VII/subchapter-C`.
ECCN classifications themselves are encoded in the CCL and
freely searchable via BIS web tools.

**EDA tool support matrix.** Same as ITAR — no first-class EAR
ECCN support in any major EDA tool; workaround is custom-property.

**Substrate posture.** The same project-level posture serving ITAR
serves EAR with one addition: per-Part **ECCN** declaration.
ECCN is a component-level attribute (an integrated circuit may
carry an ECCN of `3A991` or `5A002.a.1` per the CCL; this is
declared on the part datasheet by the manufacturer or in the
distributor's catalog). Recommended: extend `Part.qualification`
with `eccn: Option<String>` so BOM-export can include the
per-line ECCN. Project-level `Project.compliance.intended_eccn:
Option<String>` declares the project's intended export-control
posture.

**Datum coverage status.** `Planned` (paired with ITAR per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4).

**Datum implementation cost.** Pure metadata; trivial. Combined
with ITAR work in a single Datum-side implementation.

**Strategic recommendation.** **Implement now** alongside ITAR.
Single combined "export-control posture" feature.

**Risks.** Same as ITAR.

#### EU Dual-Use Regulation (EC 2021/821)

**Full title.** **Regulation (EU) 2021/821** — *Setting up a Union
regime for the control of exports, brokering, technical
assistance, transit and transfer of dual-use items*. Replaces the
prior Regulation (EC) No 428/2009. In force September 2021.

**Issuing body.** **EU** (European Commission), DG Trade.

**Scope.** EU-wide controls on dual-use exports. Annex I (the
EU dual-use list) is the EU equivalent of the US CCL, structured
similarly with classification codes (e.g., `5A002` for
information-security hardware — categories largely aligned with
the multilateral Wassenaar Arrangement so EU and US codes overlap
heavily).

**Adoption status (2026).** Mainstream-mandatory for EU
exporters of dual-use items.

**License / IP.** **Free.** EU regulations from
`eur-lex.europa.eu`.

**Datum coverage status.** No current classification (Phase 1
audit grouped with ITAR/EAR under "regulatory-export controls"
generally). **Recommendation:** add to
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4 with `Planned` disposition;
the same `Project.compliance` block carries an EU dual-use entry
(`Project.compliance.eu_dual_use_entry: Option<String>`). For
many parts the same code applies under both EU and US regimes,
so a single ECCN-style field can serve both with appropriate
labelling.

**Strategic recommendation.** **Implement alongside ITAR/EAR**;
trivial incremental cost.

#### Wassenaar Arrangement (multilateral export control)

**Full title.** **Wassenaar Arrangement on Export Controls for
Conventional Arms and Dual-Use Goods and Technologies**. 42
participating states (2026); not a treaty but a multilateral
political agreement establishing common control lists. The
**Munitions List (ML)** and the **List of Dual-Use Goods and
Technologies (DUGT)** are the operative documents.

**Datum relevance.** **Background context only.** Wassenaar
common lists are the source for ITAR/EAR/EU-Dual-Use national
implementations. No direct Datum data-model relevance —
projects are subject to one or more national implementations,
not to Wassenaar directly.

**Datum coverage status.** `Reference-only`; document as
upstream multilateral framework.

#### USML (United States Munitions List)

**Full title.** **22 CFR Part 121** — *The United States
Munitions List*. Subordinate document under ITAR (Part 120-130).
Defines 21 categories of defense articles and services
controlled under ITAR.

**Datum relevance.** USML category is the natural per-project
classification under ITAR. Recommended: when
`Project.compliance.itar_controlled: true`, an optional
`Project.compliance.usml_category: Option<UsmlCategory>` field
captures the specific USML category (Category XI — Military
Electronics is the most common for PCB design contexts;
Category XV — Spacecraft Systems for satellite electronics).

**Datum coverage status.** `Planned` (encompassed by ITAR
posture). Implement as a child field of the ITAR posture.

### Cybersecurity

#### CMMC

(Covered above under "Aerospace / Defence" because of the strong
DIB context. Cross-reference the analysis there.)

#### ISO/IEC 27001

**Full title.** **ISO/IEC 27001:2022** — *Information security,
cybersecurity and privacy protection — Information security
management systems — Requirements*. Current edition published
October 2022; supersedes 2013 edition. Companion documents:
ISO/IEC 27002:2022 (controls catalog), ISO/IEC 27005:2022 (risk
management).

**Issuing body.** **ISO/IEC** Joint Technical Committee 1
(JTC 1/SC 27 — Information security, cybersecurity and privacy
protection).

**Scope.** Information security management system (ISMS) standard.
Specifies a process-based approach to managing information
security risks. Compliance is conferred on an organisation by an
accredited certification body (BSI, TÜV SÜD, DNV, DEKRA, etc.).

**Adoption status (2026).** Mainstream globally; broadly
recognised as the de-facto international ISMS standard.

**License / IP.** **Paywalled.** ~CHF 130 from ISO Webstore.

**Datum relevance.** **Substrate, not certification.** Same
framing as 21 CFR Part 11 / ISO 13485. Datum's substrate
contributions: encrypted-content handling policy, deterministic
audit log (post-Domain-8), data-egress policy, on-premises
default deployment.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
with `Reference-only` disposition.

**Strategic recommendation.** **Reference-only**; document the
substrate-compatibility paragraph. No code work beyond the
data-egress and encrypted-content policies already specified.

#### NIST SP 800-171

**Full title.** **NIST SP 800-171 Rev 2 (2020, errata 2021)** —
*Protecting Controlled Unclassified Information in Nonfederal
Systems and Organizations*. **Rev 3** is in final-draft as of
2024 (NIST SP 800-171r3) and expected publication late 2025;
substantial restructuring (110 base controls reorganised into
~107).

**Issuing body.** **NIST** (US National Institute of Standards
and Technology).

**Scope.** Catalog of 110 (Rev 2) cybersecurity controls for
non-federal organisations handling Controlled Unclassified
Information (CUI). Underlying control framework for CMMC Level 2.

**Adoption status (2026).** Mainstream-mandatory for any
US-defence-contracted organisation handling CUI; widely adopted
beyond DIB contexts.

**License / IP.** **Free.** NIST publications are US-federal-
government public domain from `csrc.nist.gov`.

**Datum relevance.** Same substrate framing as CMMC. Datum
substrate covers ~3 controls directly (audit logging, integrity,
confidentiality at rest via encryption pass-through). The other
~107 are organisational/IT.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
with `Reference-only` disposition; document via the same
substrate paragraph as CMMC / ISO 27001.

**Strategic recommendation.** **Reference-only**.

### Consumer / IoT (note only)

#### CE marking (note)

**Full title.** **CE marking** — Conformité Européenne. Not a
standalone standard; an indication of conformity with applicable
EU directives (Low Voltage Directive 2014/35/EU, EMC Directive
2014/30/EU, Radio Equipment Directive 2014/53/EU, RoHS
2011/65/EU, REACH EC 1907/2006, etc.).

**Datum relevance.** **Fab/assembly concern, not EDA.** CE
marking is affixed to the manufactured product post-production;
the design tool's role is to enable compliance with the underlying
directives (which Datum does via materials-declaration metadata
under Domain 5 and EMC layout work under Domain 6).

**Datum coverage status.** **Out of scope** for direct CE
marking certification. The underlying directives (RoHS, REACH,
EMC, LVD) are addressed by Domains 5 and 6.

#### Regional approvals — FCC Part 15 / PSE / KC / RCM

**FCC Part 15** — US FCC unlicensed radio devices regulation.
Cross-referenced as Domain 6 (EMC) concern.

**PSE** — Japan METI Product Safety of Electrical Appliances and
Materials marking.

**KC** — Korea Communications Commission certification mark.

**RCM** — Australia/New Zealand Regulatory Compliance Mark.

**Datum relevance.** All are end-product-certification regimes
performed by accredited test labs. Datum's role is to enable
the underlying technical compliance (EMC, materials, safety) via
metadata.

**Datum coverage status.** **Reference-only / Out of scope** at
the EDA-tool layer.

### Component-Qualification Metadata

#### AEC-Q lifecycle indicator as a Part attribute

(Detailed treatment under "Automotive — AEC-Q100/Q101/Q200" above.)

The recommended `Part.qualification` extension carries:
- `aec_q_grade: Option<AecQGrade>` (`{ Grade0, Grade1, Grade2,
  Grade3, Grade4, NotQualified }`)
- `aec_q_standard: Option<AecQStandard>` (`{ Q100, Q101, Q200,
  Q103, Q104 }`)
- `aec_q_revision: Option<String>` (e.g., `"Rev-J"` for Q100,
  `"Rev-E1"` for Q101, `"Rev-E"` for Q200)
- `aec_q_qualification_date: Option<NaiveDate>`
- `aec_q_qualification_evidence: Option<String>` (URL to
  qualification report or vendor declaration)

#### MIL-spec qualification as a Part attribute

Defence-grade parts are qualified against MIL-PRF-38535
(integrated circuits — replaces older MIL-M-38510 designations
like JM38510), MIL-PRF-19500 (semiconductor devices —
JANTX/JANTXV/JANS qualification levels), MIL-PRF-55365 (chip
capacitors), MIL-PRF-49467 (chip resistors), etc.

The recommended `Part.qualification` extension carries:
- `mil_spec_qualification: Option<MilSpecQualification>` enum
  (`{ JanS, JanTxv, JanTx, Jan, Jantx, MilStd883, NotQualified }`
  — variants per the most common quality levels)
- `mil_spec_designator: Option<String>` (e.g., `"5962-9851601QXC"`
  for SMD designations)

#### Radiation hardening (rad-hard / rad-tolerant) as a Part attribute

Space and defence electronics carry radiation-tolerance
classifications:
- **Total Ionising Dose (TID)** — measured in krad(Si); typical
  qualification points 30 krad / 100 krad / 300 krad / 1 Mrad.
- **Single Event Effects (SEE)** — Single Event Latch-up (SEL),
  Single Event Upset (SEU), Single Event Burnout (SEB), Single
  Event Gate Rupture (SEGR). Typical thresholds in MeV·cm²/mg
  (LET — Linear Energy Transfer threshold).
- **Displacement Damage Dose (DDD)** — measured in MeV/g for
  proton/neutron environments.

The recommended `Part.qualification` extension carries:
- `radiation_tolerance: Option<RadiationTolerance>`
  (`{ RadHard, RadTolerant, Cots, Unknown }`)
- `radiation_tid_krad: Option<f32>` (TID rating)
- `radiation_let_threshold_mev_cm2_per_mg: Option<f32>` (SEE LET
  threshold)

These fields are populated only for the small fraction of parts
that have radiation qualification; for general commercial parts
they remain `None`.

#### Operating-temperature grade

The recommended `Part.qualification` extension carries:
- `temperature_grade: Option<TemperatureGrade>` (`{ Commercial,
  Industrial, Automotive, Military }`)
- `temperature_min_c: Option<i16>`
- `temperature_max_c: Option<i16>`

These map onto industry-conventional ranges:
- **Commercial** — 0 °C to +70 °C (Q200 Grade 4 boundary)
- **Industrial** — −40 °C to +85 °C (Q100 Grade 3 boundary)
- **Extended Industrial** — −40 °C to +105 °C
- **Automotive** — −40 °C to +125 °C (Q100 Grade 1 boundary)
- **Automotive Under-Hood** — −40 °C to +150 °C (Q100 Grade 0)
- **Military** — −55 °C to +125 °C
- **Military Extended** — −55 °C to +150 °C

The min/max integers are the authoritative datasheet-published
range; the enum is a coarse classification useful for filtering
and reporting.

### IPC Class Selection (cross-ref IPC research)

**Standards.** **IPC-A-600 J-2024** (Acceptability of Printed
Boards) and **IPC-A-610 H-2025** (Acceptability of Electronic
Assemblies). Class definitions:
- **Class 1** — General Electronic Products (toys, consumer)
- **Class 2** — Dedicated Service Electronic Products
  (commercial industrial; uninterrupted service less critical)
- **Class 3** — High Performance / Harsh Environment Electronic
  Products (continued service or service-on-demand critical;
  medical, aerospace, military)
- **Class 3A** — Automotive (introduced in IPC-A-610 G; effectively
  IPC-A-610 Class 3 with automotive-specific addenda)

**Already-researched in detail:**
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-A-600 /
IPC-A-610 (lines 566-622). Disposition under
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4 ("IPC-A-600 / IPC-A-610 class
metadata hooks: `Planned`") and § 5.5 ("IPC-A-600 / IPC-A-610
class selection: `Reference-only` until checking and output
contracts consume the field").

**Domain 4-specific finding: industries select class predictably.**
- **Consumer / IoT** → Class 1 (acceptable; cost-driven)
- **Industrial** → Class 2 (default; quality-of-life)
- **Automotive** → Class 3 or Class 3A (3A explicitly automotive-
  qualified)
- **Aerospace (civil)** → Class 3 + DO-254 / ARP4754A process
  context
- **Aerospace / Defence (military)** → Class 3 + MIL-PRF-31032 /
  MIL-PRF-55110 fab qualification (Datum: pass-through only)
- **Medical** → Class 3 (life-supporting / life-sustaining
  applications); Class 2 for non-life-critical medical
  (most diagnostic / monitoring equipment)
- **Space** → Class 3 with NASA-STD-8739.x workmanship overlay
  (Datum: pass-through only)

**Recommended Project metadata:** the `Project.compliance.ipc_class:
Option<IpcClass>` field is already proposed by IPC research. Domain
4 confirms the same field with a small additional value: a derived
**default-class-suggestion** based on `industry_vertical` —
e.g., when `industry_vertical: Automotive` the AI surface can
suggest `ipc_class: Class3A` if not already set. This is
suggestion only; not enforcement.

## Cross-Cutting Patterns

### Substrate vs certification — the central framing

The recurring finding across all 16 standards is the same:
**Datum is substrate, not certifier.** This is the
single most important framing for Domain 4 communication,
documentation, and AI-surface wording.

**What "substrate" means concretely for Datum:**

1. **Deterministic transaction log** (`docs/CANONICAL_IR.md` § 4
   + `specs/ENGINE_SPEC.md` § 3) provides immutable change
   history. Every operation is a `Transaction` with a UUID; the
   operation list is captured in `OpDiff` records suitable for
   audit reconstruction.

2. **Cryptographic record integrity** (SHA-256 hashing of JSON
   serialisation per `specs/ENGINE_SPEC.md` § 4) provides
   tamper-evidence at the file level. The integrity chain extends
   from the operation log through the JSON-serialised project
   state.

3. **Diff-friendly persistence** (per-file pool layout per
   `docs/POOL_ARCHITECTURE.md`; deterministic JSON serialisation
   per `specs/ENGINE_SPEC.md` § 4) enables standard git-based
   change-control workflows. Audit reviewers can use familiar
   tools (git log, git diff, git blame) on Datum project files.

4. **Encrypted-content handling policy**
   (`specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy,
   Batch-1 contract) preserves vendor-encrypted IBIS / SPICE /
   Touchstone models without ever decrypting them. This matches
   defence/aerospace IP-handling expectations for vendor-supplied
   models.

5. **On-premises-by-default deployment** (no cloud dependency in
   the engine; MCP server is local Unix-socket transport) provides
   a CMMC / ISO 27001 / ITAR-compatible default posture. Cloud
   variants would be opt-in features, not the default.

6. **Future signature surface (Domain-8 owned)** will provide
   cryptographic binding of identified users to specific design
   states for 21 CFR Part 11 / ISO 9001 / AS9100 / ISO 13485
   evidence chains.

**What substrate *cannot* do:**

- Substrate cannot be the certifying party. AS9100, ISO 13485,
  IATF 16949, CMMC, IEC 60601, IEC 61508, ISO 26262 certifications
  are all conferred on **organisations** by accredited
  **certification bodies**, never on tools. The wording rule for
  AI-surface and documentation is: **"Datum supports the
  substrate primitives required by [standard]; certification is
  conferred by your accredited certification body, not by Datum."**

- Substrate cannot replace requirements management,
  hazard analysis, or process documentation. Tools like DOORS,
  Polarion, JAMA, Plato SCIO-FMEA, IQ-FMEA exist for those.

- Substrate cannot certify workmanship or fabrication
  (NASA-STD-8739, MIL-PRF-31032, IPC-A-610 acceptance). Those are
  conferred on the contract assembler / fabricator.

**Why this framing matters for AI-surface design.** An AI agent
operating on a Datum project can easily slip into language like
"this design is ISO 13485 compliant" — language that, if relied
upon by a user in a regulatory submission, could create severe
liability. The rule is: **the AI surface must NEVER make
compliance certifications on Datum's behalf**. Compliance language
is reserved for the user's QMS. Datum-side claims are limited to
substrate features ("audit log enabled; signature-required-on-
release configured; export-control marking propagated to
output").

### Project metadata for compliance posture

Today, `Project` metadata in `specs/NATIVE_FORMAT_SPEC.md` § 6.1
is minimal: `uuid`, `name`, `created`, `modified`, `pools`,
`schematic`, `board`, `rules`, `settings`. There is no compliance
posture, no industry vertical declaration, no IPC class, no
intended environment, no export-control marking, no audit-overlay
configuration. **This is the largest single Datum-surface gap in
Domain 4.**

The recommended addition is a `compliance` block in `project.json`
and a parallel `Project` struct field in `ENGINE_SPEC.md`:

```rust
pub struct Project {
    // ... existing fields ...
    pub compliance: ProjectCompliance,
}

pub struct ProjectCompliance {
    // Industry vertical declaration. Drives AEC-Q grade defaults,
    // IPC class suggestions, mandated-symbol-style profile
    // suggestions, and audit-overlay configuration.
    pub industry_vertical: Option<IndustryVertical>,

    // IPC-A-600 / IPC-A-610 class. Drives DRC defaults
    // (per IPC research) and BOM filters.
    pub ipc_class: Option<IpcClass>,

    // Intended operating environment. Coarse-grained.
    // Cross-cuts Domain 6 (EMC) and Domain 5 (materials).
    pub intended_environment: Option<IntendedEnvironment>,

    // Coarse compliance-posture flags. Detailed configuration in
    // sub-fields below. Boolean-valued so absence is unambiguous.
    pub itar_controlled: bool,
    pub eu_dual_use_controlled: bool,
    pub regulated_medical: bool,
    pub safety_critical_industrial: bool,

    // Detailed export-control posture. Populated when any of the
    // above _controlled flags is true.
    pub export_control: ExportControlPosture,

    // Functional-safety integrity-level declaration (intended,
    // not certified). Cross-references ISO 26262 ASIL and
    // IEC 61508 SIL.
    pub intended_safety_integrity: SafetyIntegrityDeclaration,

    // Mandated symbol style profile (cross-ref Domain 3).
    // Some industries mandate IEEE 315 (defence US) or IEC 60617
    // (industrial EU); others have no preference.
    pub mandated_symbol_profile: Option<SymbolStyleProfile>,

    // Mandated reference-designator profile (cross-ref Domain 3).
    pub mandated_designator_profile: Option<DesignatorProfile>,

    // Data-egress policy. Mandatory MCP-layer enforcement gate.
    // Default: Unrestricted for non-controlled projects.
    pub data_egress_policy: DataEgressPolicy,

    // Audit-overlay configuration. Consumes the Domain-8 audit-
    // trail surface once available.
    pub audit_overlay: AuditOverlayMode,

    // Compliance reviewer / approver / classification metadata
    // for title-block propagation. Cross-ref Domain 3
    // SheetFrame.classification.
    pub export_control_markings: Vec<String>,
    pub compliance_notes: Vec<String>,
}

pub enum IndustryVertical {
    Consumer,
    Industrial,
    Automotive,
    Aerospace,
    Medical,
    Defence,
    Space,
    HazardousArea,        // ATEX / NEC Class I
    Other(String),
}

pub enum IntendedEnvironment {
    ConsumerIndoor,            // 0..70 C, controlled environment
    ConsumerHandheld,
    Industrial,                // -40..85 C
    IndustrialExtended,        // -40..105 C
    Automotive,                // -40..125 C
    AutomotiveUnderHood,       // -40..150 C
    AerospaceCommercial,
    AerospaceMilitary,
    Space,
    MedicalClass1,
    MedicalClass2,
    MedicalClass2a,
    MedicalClass2b,
    MedicalClass3,
    HazardousArea,
    Custom(String),
}

pub enum IpcClass {
    Class1,
    Class2,
    Class3,
    Class3A,           // Automotive
}

pub struct ExportControlPosture {
    pub usml_category: Option<UsmlCategory>,         // ITAR
    pub eccn: Option<String>,                        // EAR
    pub eu_dual_use_entry: Option<String>,           // EU 2021/821 Annex I
    pub other_jurisdictions: HashMap<String, String>,
}

pub enum UsmlCategory {
    Cat01_Firearms,
    Cat02_Guns,
    Cat03_Ammunition,
    // ... 21 categories per 22 CFR Part 121
    Cat11_MilitaryElectronics,
    Cat12_FireControl,
    Cat15_Spacecraft,
    // ... etc.
    Other(String),
}

pub struct SafetyIntegrityDeclaration {
    pub iso_26262_asil: Option<Asil>,                // automotive
    pub iec_61508_sil: Option<Sil>,                  // industrial
    pub do_254_dal: Option<Dal>,                     // avionics (note-only)
    pub iec_60601_class: Option<MedicalClass>,       // medical
}

pub enum Asil { QM, A, B, C, D }
pub enum Sil { SilOne, SilTwo, SilThree, SilFour }
pub enum Dal { DalA, DalB, DalC, DalD, DalE }

pub enum DataEgressPolicy {
    Unrestricted,                // default; permits all MCP tools
    InternalOnly,                // forbids external network calls
    NoExternalAi,                // permits supply-chain lookups but no AI exfil
    NoExternalNetwork,           // forbids any external network call
}

pub enum AuditOverlayMode {
    None,                            // no audit overlay
    BasicAuditTrail,                 // log every transaction with timestamp+user
    SignatureRequired,               // basic + cryptographic signature on release
    SignatureRequiredWithMeaning,    // signature carries explicit meaning enum
}
```

This is **one new struct field** on `Project` plus **one new sub-
struct** with **eight enum types**. The total Rust line count for
the data model is ~250 lines plus the documentation.

### Per-Part qualification metadata

Today, `Part` (extended in Batch 1) carries: `uuid`, `entity`,
`package`, `pad_map`, `mpn`, `manufacturer`, `manufacturer_jep106`,
`value`, `description`, `datasheet`, `parametric`, `orderable_mpns`,
`packaging_options`, `tags`, `lifecycle`, `base`,
`behavioural_models`, `thermal`, `supply_chain_offers`,
`last_supply_chain_check`. Compliance-related fields: only
`lifecycle` (`Active | NRND | EOL | Obsolete | Unknown`).

The recommended addition is a `qualification` field carrying the
component-qualification metadata:

```rust
pub struct Part {
    // ... existing fields ...
    pub qualification: Option<PartQualification>,
}

pub struct PartQualification {
    // Automotive — AEC-Q
    pub aec_q_grade: Option<AecQGrade>,
    pub aec_q_standard: Option<AecQStandard>,
    pub aec_q_revision: Option<String>,
    pub aec_q_qualification_date: Option<NaiveDate>,
    pub aec_q_qualification_evidence: Option<String>,

    // Defence — MIL-spec
    pub mil_spec_qualification: Option<MilSpecQualification>,
    pub mil_spec_designator: Option<String>,

    // Space — radiation
    pub radiation_tolerance: Option<RadiationTolerance>,
    pub radiation_tid_krad: Option<f32>,
    pub radiation_let_threshold_mev_cm2_per_mg: Option<f32>,

    // Hazardous area — ATEX / IECEx
    pub atex_certification: Option<String>,        // free-text marking
    pub iecex_certification: Option<String>,

    // Operating temperature
    pub temperature_grade: Option<TemperatureGrade>,
    pub temperature_min_c: Option<i16>,
    pub temperature_max_c: Option<i16>,

    // Export-control attributes (per-Part — distinct from
    // Project.compliance.export_control which is project-level)
    pub eccn: Option<String>,
    pub usml_category: Option<UsmlCategory>,
    pub eu_dual_use_entry: Option<String>,

    // Generic free-text qualification notes
    pub other_qualifications: Vec<String>,
}

pub enum AecQGrade { Grade0, Grade1, Grade2, Grade3, Grade4 }
pub enum AecQStandard { Q100, Q101, Q200, Q103, Q104 }
pub enum MilSpecQualification {
    JanS,        // most stringent (space-qualified)
    JanTxv,      // class S derivative
    JanTx,
    Jan,
    MilStd883,
    NotQualified,
}
pub enum RadiationTolerance { RadHard, RadTolerant, Cots, Unknown }
pub enum TemperatureGrade {
    Commercial,
    Industrial,
    ExtendedIndustrial,
    Automotive,
    AutomotiveUnderHood,
    Military,
    MilitaryExtended,
}
```

Total Rust line count for the data model: ~150 lines plus
documentation. Pure metadata; no validator-engine work.

### Audit trail export (cross-ref Domain 8)

**Domain 8 owns this primitive.** Domain 4 specifies what
compliance posture *consumes* from it.

The Domain-4-specific consumption:

1. **Compliance-posture-driven audit-overlay enablement.** When
   `Project.compliance.audit_overlay: BasicAuditTrail`, every
   Transaction must persist with:
   - timestamp (ISO 8601 UTC)
   - acting user identifier (OS user; later: configured user)
   - operation description (already present)
   - optional rationale (new field)
   The MCP layer consults the audit-overlay setting before
   determining what fields are required at transaction-commit time.

2. **Compliance-posture-driven signature requirements.** When
   `Project.compliance.audit_overlay: SignatureRequired` or
   `SignatureRequiredWithMeaning`, certain operations require
   signature before commit (typically: state-transition
   operations like `release`, `approve`, `submit-for-review`).
   The Domain 8 spec will define which operations are signature-
   required by category; Domain 4 specifies the configuration that
   maps a project's vertical to those categories.

3. **Audit-trail export format.** Domain 8 will define a
   queryable / exportable audit-trail format. Domain 4
   contributes the wrapper requirements:
   - For 21 CFR Part 11 contexts: must include signature-event
     records, signature-meaning declarations, signed-record
     hashes.
   - For ISO 9001 / AS9100 contexts: must include reviewer /
     approver identities and state-transition records.
   - For ITAR contexts: must include data-egress events (any
     external network call attempt and its outcome).

4. **Export format suggestion.** **Signed CSV + PDF** is the
   pragmatic choice for regulated-industry audit-trail
   consumption. Most QMS tools accept CSV import for evidence
   capture; PDF (with detached signature) is the formal
   evidence record. JSON is the diff-friendly source-of-truth.
   The Domain 8 spec should produce all three from one
   export operation.

### 21 CFR Part 11 electronic signatures

(Detailed treatment in the standards catalog above. This section
captures the technical-requirement summary as a reference card.)

**Subpart B (Electronic Records) requirements that map to Datum:**

| § | Requirement | Datum substrate |
|---|---|---|
| § 11.10(a) | Validation of systems | Integration tests + golden-state regression |
| § 11.10(b) | Generate accurate / complete records | JSON serialisation determinism |
| § 11.10(c) | Protection of records | SHA-256 hashing + git-friendly diffability |
| § 11.10(d) | Limit system access | OS-level access; on-prem deployment |
| § 11.10(e) | Audit trail | Transaction model + future audit-trail export |
| § 11.10(f) | Operational system checks | Validate-project + ERC/DRC |
| § 11.10(g) | Authority checks | Future signature primitive (Domain 8) |
| § 11.10(h) | Device checks | N/A for design tool |
| § 11.10(i) | Personnel qualifications | Organisational, not tool |
| § 11.10(j) | Hold individuals accountable | Future signature primitive |
| § 11.10(k) | Documentation control | Pool architecture + version control |

**Subpart C (Electronic Signatures) requirements:**

| § | Requirement | Datum implementation |
|---|---|---|
| § 11.50(a)(1) | Signature manifestation: printed name | Signature record carries signer name |
| § 11.50(a)(2) | Signature manifestation: signature meaning | `SignatureMeaning` enum |
| § 11.50(a)(3) | Signature date and time | Signature record carries timestamp |
| § 11.70 | Signature linking to record | Signature over project SHA-256 |
| § 11.100 | Unique signature identification | One signer = one identifier |
| § 11.200(a)(1) | Two distinct identification components | Username + password / cert |
| § 11.300 | Controls for identification codes | Domain 8 spec / OS / SSO |

The **Domain 8** scope owns § 11.10(g), (j), and all of Subpart C.
**Domain 4** owns the project-posture configuration that drives
when these features are required.

### Industry-mandated symbol-style profiles (cross-ref Domain 3)

(From `research/schematic-drawing-conventions/` § Cross-Domain
Insights → To Domain 4.)

**Defence (US)** typically mandates IEEE 315 (and the older
MIL-STD-15 lineage subsumed under IEEE 315 + IEEE 91):
- Resistors as zigzag
- Logic gates as distinctive D-and-bubble
- Power symbols per IEEE convention
- Reference designators per ASME Y14.44 (or older MIL-STD-16)

**Industrial (EU)** typically mandates IEC 60617:
- Resistors as rectangle
- Logic gates as rectangle with `&` / `1` / `≥1` qualifying labels
- Power symbols per IEC convention
- Reference designators per IEC 81346-2

**Industrial (Japan)** typically mandates JIS C 0617 (essentially
IEC 60617 with a few JIS-specific symbols added).

**Consumer / hobbyist / commercial** typically has no mandated
profile — engineer choice.

**Datum coverage.** The Domain 3 deep-dive proposes
`SymbolStyleProfile` enum on `Symbol` records and
`mandated_symbol_profile` on Project metadata. Domain 4 confirms
this — when `Project.compliance.industry_vertical: Defence`, the
`mandated_symbol_profile` defaults to `Ieee315`. When
`industry_vertical: IndustrialEu`, defaults to `Iec60617`.
This is suggestion only; user authority preserved per Datum's
authoring-tools philosophy.

### Encrypted vendor model handling (cross-ref Domain 2)

(From `research/component-modeling/` § Encrypted Models.)

Defence and aerospace vendors heavily distribute encrypted IBIS,
encrypted SPICE, encrypted Touchstone, and encrypted IBIS-AMI
models. The encryption preserves vendor IP while allowing
substrate-level system simulation. Common schemes:
- **IBIS BIRD-176** (AES-128, vendor key)
- **PSpice Encrypt-It** (Cadence-proprietary)
- **HSPICE AvantHash** (Synopsys-proprietary)
- **LTspice obfuscation** (ADI-proprietary)
- **Spectre encryption** (Cadence-proprietary)

**Datum policy (Batch 1).** The MCP layer enforces an Encrypted
Content Handling Policy
(`specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy):
1. Detection at attach time → `ModelAttachment.encrypted: bool` +
   `encryption_scheme`
2. Metadata always allowed
3. Content extraction gated → opaque-handle mode
4. Pass-through preserved → Datum never decrypts, never
   re-encrypts
5. Audit-trail of every gate-check decision

**Domain 4 reaffirmation.** This policy is exactly what defence /
aerospace customers expect. The Batch-1 work is sufficient; no
new Domain 4 contract is needed beyond ensuring the
`Encrypted Content Handling Policy` audit-trail entries are
captured by the future audit-trail export surface (Domain 8).

### Data residency / sovereign cloud

The cybersecurity intersection with sovereign cloud requirements
(US DoD IL-4 / IL-5 / IL-6, UK G-Cloud, EU GAIA-X, Australia
PROTECTED, Japan ISMAP) all converge on the question "where is
the design data physically stored, and who controls the
infrastructure?"

**Datum's posture.** The engine is **headless and on-prem by
default.** No cloud dependency. The MCP server transport is local
Unix socket. Pool storage is local filesystem (or local network
share). The only outbound network calls are explicit per-tool
operations: `lookup_part_octopart`, `lookup_part_digikey`,
`lookup_part_mouser`, `refresh_supply_chain`, and any AI-routed
external-LLM tools that the MCP server exposes.

The `data_egress_policy` field gates these. With
`NoExternalNetwork`, Datum operates in a fully air-gapped /
sovereign-cloud-compatible mode. With `Unrestricted`, full
external-lookup capability is available. Intermediate modes
(`InternalOnly`, `NoExternalAi`) provide nuanced postures for
mixed-environment deployments.

**Sovereign-cloud deployment patterns.**
- **Air-gapped / classified.** `data_egress_policy:
  NoExternalNetwork`. No external lookups. Pool is fully local.
  AI agent (if any) operates against local-deployed model only.
- **Sensitive-but-unclassified / CMMC L2.** `data_egress_policy:
  NoExternalAi`. Distributor lookups allowed (Octopart, etc.) but
  AI tool calls to external LLMs are blocked. This is the
  recommended default for most regulated-industry projects.
- **Open commercial.** `data_egress_policy: Unrestricted`. Full
  external-lookup capability.

**Recommendation.** The `data_egress_policy` field is the single
most important Domain-4 implementation for sovereign-cloud /
air-gap compatibility. Implement now; it is trivially small
(an enum + an enforcement gate) and unlocks a very large
addressable market (defence, aerospace, medical-device, US-state-
level government).

## EDA Tool Support Matrix

| Standard / Feature | Altium | OrCAD-Capture / OrCAD X | Cadence Allegro | PADS / Xpedition | KiCad 8 | Eagle / Fusion | Horizon | LibrePCB | DipTrace | EasyEDA Pro | Datum (current spec) | Datum (post-Domain-4) |
|----|----|----|----|----|----|----|----|----|----|----|----|----|
| **AEC-Q grade as Part attribute** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Catalog filter only | Not supported | `Part.qualification.aec_q_grade` `Planned` |
| **MIL-spec qualification** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.qualification.mil_spec_qualification` `Planned` |
| **Radiation tolerance** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.qualification.radiation_tolerance` `Planned` |
| **Operating-temperature grade** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Catalog filter only | Not supported | `Part.qualification.temperature_grade` `Planned` |
| **ECCN per Part** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.qualification.eccn` `Planned` |
| **IPC class as Project metadata** | Project setting | Project setting | Project setting | Project setting | Project setting | Project setting | Project setting | Project setting | Project setting | Project setting | Not supported | `Project.compliance.ipc_class` `Planned` (per IPC research) |
| **Industry vertical declaration** | Vault project type | Not supported | Project property | Project property | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Project.compliance.industry_vertical` `Planned` |
| **Intended environment** | Vault project type | Not supported | Project property | Project property | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Project.compliance.intended_environment` `Planned` |
| **ITAR / EAR project markings** | Vault classification | Not supported | Vault classification | Xpedition title-block templates | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Project.compliance.itar_controlled` + `export_control_markings` `Planned` |
| **ASIL / SIL declaration** | Custom property | Custom property | Custom property | Xpedition Automotive Pack (paid) | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Project.compliance.intended_safety_integrity` `Planned` |
| **21 CFR Part 11 sign-off** | Vault Workflow (paid) | Not supported | Vault / PLM integration | Teamcenter integration | Not supported (git only) | Not supported | Not supported | Not supported | Not supported | Not supported | Substrate only (no signature) | `Deferred with prerequisite` (Domain 8) |
| **ISO 9001 audit trail export** | Vault | Not supported | Vault / PLM | Teamcenter | Git only | Not supported | Not supported | Not supported | Not supported | Not supported | Substrate only | `Deferred with prerequisite` (Domain 8) |
| **Encrypted vendor model handling** | Pass-through | Pass-through | Pass-through (license-gated) | Pass-through | Pass-through | Limited | Not supported | Not supported | Not supported | Not supported | Pass-through (Batch-1 policy) | `Implemented` (post-Batch-1) |
| **Data-egress policy / sovereign cloud** | On-prem deploy option | On-prem only | On-prem deploy option | On-prem deploy option | Inherently on-prem | Inherently on-prem | Inherently on-prem | Inherently on-prem | Inherently on-prem | Cloud-only (incompatible with controlled work) | Inherently on-prem; no policy gate | `Project.compliance.data_egress_policy` `Planned` |
| **Mandated symbol-style profile** | Library default | Library default | Library default | Library default | Library default | Library default | Library default | Library default | Library default | Library default | Not supported | `Project.compliance.mandated_symbol_profile` `Planned` (cross-ref Domain 3) |
| **AS9102 First Article Inspection** | Vault FAI report (paid) | Not supported | Vault / PLM FAI | Teamcenter FAI | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Variant-substrate only | `Deferred with prerequisite` (audit-trail + variants) |

**Key:** "Vault" = Altium Vault subscription; "CIS" = Cadence Component
Information System (paid); "Custom property" = free-text user-defined
field. "Not supported" = no first-class support in the tool's data model.

**Reading the matrix.** The pattern is consistent: the **Vault-class
commercial tools** (Altium Vault, Cadence Vault, Teamcenter)
provide first-class compliance metadata; the **mid-tier and
open-source tools** rely on free-text custom-property workarounds.
**Datum's opportunity is to be first-class without the Vault
license requirement** — every compliance metadata field a paying
Altium Vault customer takes for granted, Datum exposes natively in
the open-source engine. This is a credible positioning win in the
Vault-priced market segment.

## Pending Exclusions (re-affirmed)

The audit's eight Domain-4 advisory exclusions are confirmed. None
has hidden cross-cutting value worth re-opening for deep-dive.

| Standard | Confirmed exclusion rationale | Recommended formal disposition (post-Domain-8) |
|---|---|---|
| **DO-254** | Process-grade hardware-design-assurance certification conferred on the project team by an FAA / EASA DER. EDA tool is not a participant. Substrate is sufficient — Datum's transaction log + JSON serialisation + encrypted-content policy support the configuration-management and supplier-IP-handling requirements that DO-254 imposes. The certifying party is the DER. | `Out of scope` |
| **DO-160** | Avionics environmental qualification — testing performed at the assembled-board level by accredited test labs (UL, TÜV, NTS). EDA tool has no input or output. | `Out of scope` |
| **MIL-PRF-31032** | Fab-QMS qualification regime conferred on the PCB fabricator (TTM, Sanmina, IEC). Datum can pass through material-stackup and class-marking data via Gerber X3 / IPC-2581 (covered by Domain 1 / IPC research) but is not a participant in fabricator certification. | `Out of scope` |
| **MIL-PRF-55110** | Older rigid-PWB sub-specification of MIL-PRF-31032 lineage; same fab-QMS framing. Largely superseded by IPC-6012 Class 3 / 3A. | `Out of scope` |
| **NASA-STD-8739.x family** | Workmanship — solder joint quality, harness routing, conformal coating application. All post-fabrication / assembly concerns. Zero EDA-tool intersection. IPC-A-610 Class 3 (already in IPC research scope) is the closest commercial-equivalent. | `Out of scope` |
| **AS9100** | Aerospace QMS — process certification conferred on an organisation by an accredited certification body. ISO 9001 substrate (Domain 8) covers the audit-trail story. AS9100-specific extensions (AS9102 FAI report) live in PLM tools, not EDA. | `Out of scope` |
| **IATF 16949** | Automotive QMS — same framing as AS9100. Process certification conferred on an organisation. AEC-Q metadata work (per Datum's Domain-4 plan) covers the relevant component-qualification metadata; IATF 16949 process compliance is organisational. | `Out of scope` |
| **CMMI** | Capability Maturity Model Integration — organisational process maturity assessment conferred by accredited CMMI Institute appraisers. No tool component. ISO 9001 substrate is sufficient for any tool-level compliance claim. | `Out of scope` |

**No new Domain-4 exclusions discovered during deep-dive that
warrant flagging back to the audit.** The eight above are correctly
identified as advisory exclusions and should be promoted to formal
`Out of scope` in the consolidated post-Domain-8 ratification pass.

**Note for the consolidated ratification pass:** Some of these
standards (AS9100, IATF 16949, ISO 13485) carry explicit substrate
expectations that Datum can advertise even while declining
certification. The recommended pattern is a **substrate-
compatibility paragraph** in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4
attached to each `Out of scope` disposition, naming the substrate
features Datum provides that support a customer's organisational
certification work. This avoids the "out of scope" disposition
reading as "Datum is irrelevant to this standard" when the truth
is "Datum is substrate-relevant but not certifying".

## User Pain Points & Wishlist Items

Distilled from EEVblog, AviationStack forums, MedicalDeviceHQ
forums, defense-electronics LinkedIn discussions, the IPC EDGE
community, NASA EEE-INST-002 reviews, and the EDA industry survey
blogs (EDN, AspenCore, Aviation Week, MedTech Dive):

1. **"Why doesn't my EDA tool know my project is automotive?"** —
   The biggest single complaint in the AEC-Q-using community is
   that the EDA tool is unaware of the project's vertical. Engineer
   manually checks every BOM line for AEC-Q grade. **Datum
   opportunity:** `Project.compliance.industry_vertical:
   Automotive` + `Part.qualification.aec_q_grade` + AI-surface
   warning when grade < project minimum. This is a real workflow
   improvement.

2. **"My medical project's BOM has commercial-grade parts and the
   reviewer caught it at the design review."** — Same pattern as
   automotive: lack of project-vertical awareness means
   inappropriate parts slip into BOMs. **Datum opportunity:** the
   same AEC-Q-style check, generalised to per-vertical
   appropriateness rules.

3. **"How do I prove to my Notified Body that my Datum project
   is the same one that was reviewed?"** — Substrate-recognition
   issue. Engineer wants a one-click "produce evidence package"
   that bundles project state, transaction log, signature
   records, reviewer-approver list, ECO history. **Datum
   opportunity:** the Domain-8 audit-trail-export contract should
   produce this evidence package; Domain 4 specifies the
   compliance-vertical-aware wrapper.

4. **"Our defence customer required ITAR markings on every output
   and we missed marking the Gerber files."** — Marking propagation
   failure. Engineer correctly marks the title-block but the export
   adapter doesn't propagate the marking to fab-output files.
   **Datum opportunity:** `Project.compliance.export_control_markings`
   propagated to every export adapter (BOM, Gerber, IPC-2581, PDF,
   ODB++) by contract.

5. **"I can't use [cloud EDA tool] because we're CMMC-controlled
   and our IT department blocked it."** — Cloud-incompatibility
   complaint from defence-adjacent engineering teams. **Datum
   opportunity:** Datum is inherently on-prem; positioning as
   "AI-native + CMMC-compatible" is a real differentiator.

6. **"Our AI assistant looked up our BOM on Octopart but the
   project was ITAR-controlled — is that a violation?"** —
   Real concern emerging in 2025-2026 as AI agents become
   workflow-integrated. The answer depends on whether the BOM
   itself is technical data (yes for ITAR-controlled designs);
   the lookup may have been an inadvertent export. **Datum
   opportunity:** the `data_egress_policy` gate prevents this
   class of inadvertent export.

7. **"Why can't my tool tell me if a part has a known
   counterfeit-supply-chain history?"** — Counterfeit-component
   detection (per IDEA-STD-1010, AS5553A counterfeit-electronic-
   parts standards) is the next-tier compliance concern. **Datum
   note:** out of scope for v1; the substrate (AEC-Q grade,
   distributor offers, JEP106 manufacturer codes) is in place to
   support a future counterfeit-detection feature, but the actual
   counterfeit-database integration is post-M8 work.

8. **"My export-control reviewer wants to see the ECCN on every
   line of the BOM but Octopart doesn't always carry it
   accurately."** — Vendor-data accuracy complaint. ECCN
   classification is the manufacturer's responsibility but is
   often missing from distributor catalogs. **Datum opportunity:**
   `Part.qualification.eccn` is authored data; the AI surface can
   suggest values from manufacturer datasheets but the user
   confirms / overrides per project authority.

9. **"Why doesn't [my tool] support 21 CFR Part 11 signatures? I
   have to print to PDF and sign in DocuSign."** — Substrate-gap
   complaint. Signature workflow is bolted on via external tools.
   **Datum opportunity:** native signature primitive (Domain 8
   owned) eliminates the bolt-on.

10. **"Our auditor asked for the change history of net `VCC_5V`
    over the last six months and we couldn't produce it."** —
    Audit-query-surface gap. Even tools with version control
    (KiCad + git) make per-object history queries hard. **Datum
    opportunity:** the canonical-IR transaction model + future
    audit-trail-export surface should make object-scoped history
    a first-class query (e.g., "show me every Transaction that
    modified net `VCC_5V` between dates X and Y, with signer if
    signed").

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

These land as part of the Domain 4 spec edits in the next batch.

#### HR-1. `Project.compliance: ProjectCompliance` block

**Standard / driver.** Cross-cutting — supports all of: AEC-Q
(via industry_vertical), ISO 26262 (via intended_safety_integrity),
ITAR/EAR (via itar_controlled + export_control + data_egress_policy),
21 CFR Part 11 (via audit_overlay), IEC 60601 (via
intended_environment), IPC-A-600/610 (via ipc_class — already
recommended by IPC research), and Domain 3 (via
mandated_symbol_profile + mandated_designator_profile).

**Canonical IR changes.** Add `ProjectCompliance` struct,
`IndustryVertical`, `IntendedEnvironment`, `IpcClass`,
`ExportControlPosture`, `UsmlCategory`, `SafetyIntegrityDeclaration`,
`Asil`, `Sil`, `Dal`, `MedicalClass`, `DataEgressPolicy`,
`AuditOverlayMode` enums to `ENGINE_SPEC.md` § 1.1a (Shared Enums).
Add `compliance: ProjectCompliance` field to `Project` (new section
in ENGINE_SPEC.md § 1.x — Project type isn't currently spec'd as a
struct; this is its first appearance in the spec).

**Native-format changes.** `project.json` (`NATIVE_FORMAT_SPEC.md`
§ 6.1) gains a `"compliance": { ... }` block carrying the
serialised `ProjectCompliance` struct. The block is required-
present (with default values) for new native projects; existing
projects without the block deserialise with defaults
(industry_vertical: None; data_egress_policy: Unrestricted).

**Transaction model changes.** New operation
`SetProjectCompliance` (authored, undoable). For cases where
individual sub-fields are commonly set independently, also add
focused operations: `SetIndustryVertical`, `SetIpcClass`,
`SetIntendedEnvironment`, `SetExportControlPosture`,
`SetDataEgressPolicy`, `SetAuditOverlayMode`. Each carries the
prior value in its OpDiff for undo.

**MCP API additions.**
- `get_project_compliance(project_uuid)` — returns the full
  compliance block.
- `set_project_compliance(project_uuid, compliance)` — atomic
  replacement.
- `set_industry_vertical(project_uuid, vertical)` — focused.
- `set_ipc_class(project_uuid, class)` — focused (already
  recommended by IPC research).
- `set_intended_environment(project_uuid, env)` — focused.
- `set_export_control(project_uuid, posture)` — focused.
- `set_data_egress_policy(project_uuid, policy)` — focused.
- `set_audit_overlay(project_uuid, mode)` — focused.
- `validate_project_compliance(project_uuid)` — runs the
  internal-consistency checker (e.g., "ITAR-controlled but
  data_egress_policy: Unrestricted is inconsistent" warning).

**Minimum viable.** The struct + serialisation + atomic
get/set MCP surface. No validator. Defaults to
unrestricted / no compliance posture for backward compatibility.

**Full implementation.** All focused operations + validator +
AI-surface integration (suggest defaults from industry_vertical) +
title-block propagation (Domain 3 handoff) + export-adapter
propagation (Domain 1 handoff).

**Partner / library dependencies.** None for the data model.

**Effort estimate.** **3-5 days** for the data-model + MCP +
serialisation work; **+2 days** for the validator and AI-surface
integration polish. Small but high-leverage.

#### HR-2. `Part.qualification: Option<PartQualification>` extension

**Standard / driver.** AEC-Q100/Q101/Q200, MIL-PRF-38535/19500,
ATEX, EAR (per-Part ECCN), space-electronics radiation
qualification, generic operating-temperature grade.

**Canonical IR changes.** Add `PartQualification` struct,
`AecQGrade`, `AecQStandard`, `MilSpecQualification`,
`RadiationTolerance`, `TemperatureGrade` enums to
`ENGINE_SPEC.md` § 1.1a. Add `qualification:
Option<PartQualification>` field to `Part` (§ 1.2).

**Pool / library changes.** Pool index `parts` table gains
columns: `aec_q_grade`, `temperature_grade`,
`mil_spec_qualification`, `radiation_tolerance`, `eccn`. Query
interface allows filtering by qualification.

**Transaction model changes.** New operation `SetPartQualification`
(authored, undoable; carries prior qualification in OpDiff).

**MCP API additions.**
- `get_part_qualification(part_uuid)` — returns qualification
  block.
- `set_part_qualification(part_uuid, qualification)` — atomic
  replacement.
- `find_parts_by_qualification(filter)` — pool query for
  parts matching qualification predicates (e.g., "all
  AEC-Q100 Grade 1 or better, temperature_grade: Automotive").
- `infer_part_qualification(part_uuid)` — heuristic inference
  from MPN suffix conventions (TI's `-Q1` for Q100 Grade 1;
  ADI's `S` for Mil/JANS; etc.). Returns suggestion only;
  user confirms via `set_part_qualification`.

**Minimum viable.** Struct + field + atomic get/set + filter
query.

**Full implementation.** All focused operations + heuristic
inference + AI-surface part-substitution warnings (suggest
substitution candidates that match project compliance vertical).

**Partner / library dependencies.** None for the data model.

**Effort estimate.** **2-3 days** for the data-model + MCP +
serialisation work. ~150 lines of Rust.

#### HR-3. `data_egress_policy` MCP-layer enforcement gate

**Standard / driver.** ITAR (22 CFR Part 120), EAR (15 CFR
Parts 730-774), CMMC Level 2, NIST SP 800-171 Rev 2, EU 2021/821,
ISO/IEC 27001.

**Architecture.** The `data_egress_policy` field on
`Project.compliance` is enforced at the **MCP server boundary**
(Python layer). Every MCP tool with an external network side
effect declares the side effect in its tool definition; the MCP
server's pre-tool-call hook consults the project's
`data_egress_policy` and refuses-or-warns the call accordingly.

**Affected MCP tools (initial list).** All tools in the Component
Modelling section that perform external lookups
(`lookup_part_octopart`, `lookup_part_digikey`,
`lookup_part_mouser`, `refresh_supply_chain`,
`find_alternate_parts`); future tools (`fetch_3d_model_from_url`,
`refresh_compliance_status`, etc.).

**MCP API additions.**
- Existing tools gain a `data_egress_policy_check` pre-hook (no
  user-visible API change).
- New tool: `get_egress_policy_decision(tool_name)` — returns
  whether the project's current policy permits the tool. Useful
  for AI-surface dispatch.
- New tool: `audit_egress_attempts(date_range)` — returns the
  audit-trail of egress attempts and their decisions. Domain 8
  consumer.

**Minimum viable.** Enforcement gate on the four currently-
specified external-lookup tools; refuse-or-warn behaviour
configurable; audit-trail of decisions written.

**Full implementation.** Gate applies to all tools with external
side effects; per-tool override permissions configurable;
deferred-decision mode (queue calls for human approval); detailed
audit-trail with decision rationale.

**Partner / library dependencies.** None.

**Effort estimate.** **3-4 days** for the gate + per-tool
integration on the four current external tools; **+1-2 days** per
new external tool.

#### HR-4. Title-block / export-adapter classification-marking propagation

**Standard / driver.** ITAR (22 CFR § 125.4 — controlled-data
markings), EAR (15 CFR § 750.7 — license requirements),
EU 2021/821 export-control marking conventions, contract-
specified markings ("Confidential — Customer XYZ", "Subject to
ECCN xxxx", "Distribution Statement A/B/C/D").

**Architecture.** The
`Project.compliance.export_control_markings: Vec<String>` field
holds the marking strings to propagate to every output. Each
export adapter (BOM, Gerber, IPC-2581, ODB++, PDF, KiCad
write-back, native save) consumes the field and renders the
markings appropriately.

**Domain handoff.**
- **Domain 3** owns the title-block / sheet-frame
  classification field
  (`SheetFrame.classification`) per the Domain 3 deep-dive's
  recommendation.
- **Domain 1** owns the export-adapter changes per format
  (Gerber X3 attribute, IPC-2581 `<Header>` attribute, ODB++
  classification entry, PDF watermark).
- **Domain 4** owns the project-level posture field that drives
  what gets propagated.

**MCP API additions.**
- Coupled to `set_project_compliance` (HR-1).
- New tool: `validate_export_marking_propagation(project_uuid)`
  — checks that all configured export adapters propagate the
  markings; warns on any adapter that doesn't.

**Minimum viable.** Field present; propagated to BOM and PDF
exports.

**Full implementation.** Propagated to all export adapters;
configurable per-adapter formatting; validator confirms
propagation completeness.

**Partner / library dependencies.** None.

**Effort estimate.** **2 days** for the field + propagation to
the BOM / PDF adapters; **+1 day per additional adapter**.

### Should Support (post-M7)

#### SS-1. AI-surface compliance-aware part-substitution warnings

**Standard / driver.** AEC-Q, ISO 26262, IEC 60601, ITAR, EAR.

**Behaviour.** When the AI agent suggests substituting a Part
into a project, it consults `Project.compliance` and the
substitution candidate's `Part.qualification`, and emits a
warning when:
- candidate AEC-Q grade is below project minimum (for
  industry_vertical: Automotive)
- candidate is not MIL-spec qualified (for industry_vertical:
  Defence)
- candidate's ECCN is more restrictive than the project's
  intended ECCN (potentially expands export-control burden)
- candidate's temperature grade is below project's intended
  environment range
- candidate's radiation tolerance is below project minimum
  (for industry_vertical: Space)

**Effort estimate.** **2-3 days** for the warning rules + AI-
surface integration. Depends on HR-2 landing.

#### SS-2. Compliance-posture-aware DRC defaults

**Standard / driver.** IPC-A-600 / IPC-A-610 Class 1/2/3/3A.

**Behaviour.** When `Project.compliance.ipc_class` is set, the
DRC engine selects appropriate default-rule values per the IPC
class table (IPC-2221 clearance, IPC-2152 current-carrying
capacity, IPC-7351 land-pattern density level). Already
recommended by IPC research; Domain 4 confirms.

**Domain handoff.** IPC research / Domain 5 owns the actual
class-to-rule-value mapping; Domain 4 confirms `ipc_class`
on `Project.compliance`.

**Effort estimate.** **0 days additional** beyond IPC-research-
recommended work; this is integration only.

#### SS-3. Audit-overlay configuration consumption (post Domain-8 audit-trail)

**Standard / driver.** 21 CFR Part 11, ISO 9001, AS9100,
IATF 16949, ISO 13485.

**Behaviour.** When `Project.compliance.audit_overlay:
SignatureRequired` (or stricter), the engine requires signature
binding for state-transition operations (release, approve,
submit-for-review). The signature primitive itself is
Domain-8 owned.

**Domain handoff.** Domain 8 owns the signature primitive;
Domain 4 specifies which audit-overlay modes activate which
behaviours.

**Effort estimate.** **2 days** for the configuration
consumption layer once the Domain-8 signature primitive lands.

### On-Demand Only

#### OD-1. Per-vertical compliance-report exporters

If a customer surfaces with a specific need (e.g., "produce a
21 CFR Part 11 audit-trail evidence package matching FDA's
preferred format"), build the per-vertical export adapter for
that need on demand. **Do not pre-build per-vertical exporters
speculatively.** The substrate (audit-trail JSON export from
Domain 8) is sufficient as raw input for any per-vertical
formatter the customer requires.

#### OD-2. Counterfeit-component detection (per IDEA-STD-1010, AS5553A)

The substrate (AEC-Q grade, distributor offers, JEP106
manufacturer codes) is in place. The actual counterfeit-detection
feature requires integration with a counterfeit-database service
(ECIA, ERAI, IHS Markit) which is paid-only and customer-
specific. Build only when a customer drives the requirement.

#### OD-3. Per-customer ECCN classification database integration

Some defence-vertical customers maintain in-house ECCN
classification databases. Integration is per-customer
configuration, not a Datum-engine feature.

### Out of Scope (recommend formal exclusion)

The full advisory-exclusion list re-affirmed under § "Pending
Exclusions (re-affirmed)". Plus:

- **AS9102 First Article Inspection report formatting** — beyond
  Datum's variant-substrate (which is in scope per
  `STANDARDS_COMPLIANCE_SPEC.md` § 4.7's "AS9102 first-article
  traceability hooks: Deferred with prerequisite"), the FAI
  report formatting itself is PLM-tool work (Net-Inspect,
  iBASEt, Discus). Datum exposes the variant data; the PLM
  tool formats the FAI report.

- **Per-aerospace-program qualification tracking** (e.g., NASA
  EEE-INST-002 Class 2 parts list, NASA Goddard PPL,
  ESA-EPPL European Preferred Parts List) — these are program-
  specific qualified-parts lists maintained by their respective
  agencies. Datum can carry the per-Part qualification metadata
  (HR-2) but does not maintain the actual qualified-parts lists.
  Customer-specific configuration if needed.

- **Per-medical-program FDA submission packaging** (510(k) cover
  letter, CE Marking technical-file binding) — submission
  packaging is regulatory-affairs work, not EDA work.

- **Cybersecurity certification audits** (CMMC C3PAO assessment,
  ISO 27001 surveillance audit, NIST 800-171 self-attestation
  documentation) — auditor work, not tool work. Datum's
  substrate (data-egress policy, audit log, encrypted-content
  policy) is the tool-side input the auditor reviews.

### Datum Differentiators

Where Datum's deterministic substrate + AI-native surfaces can
do better than incumbents:

1. **AI-explained ITAR / EAR marking violations.** When an MCP
   tool call is blocked by `data_egress_policy`, the AI surface
   can explain why ("this project is ITAR-controlled and the
   tool would have shipped design data to an external service")
   rather than just refusing. No incumbent EDA tool has an
   AI-surface to explain compliance posture decisions.

2. **MCP-queryable compliance posture.** Every aspect of the
   compliance posture is queryable via MCP. An auditor's AI
   agent can ask "does this project declare an industry
   vertical, an intended environment, an IPC class, an export-
   control posture, an audit-overlay mode?" and get a structured
   answer in milliseconds. No incumbent EDA tool exposes this
   structured query surface.

3. **Deterministic audit-trail export.** Datum's transaction
   model + JSON serialisation determinism mean every audit-
   trail export produces byte-identical output for byte-identical
   project state. This is much stronger than tools whose audit
   logs include timestamps and session-IDs that change between
   runs. Audit reviewers can verify integrity by re-export-and-
   diff rather than trusting a vendor's audit-log claim.

4. **Encrypted-content gate as a compliance feature.** The
   `Encrypted Content Handling Policy` (Batch 1) is the first
   formal contract in any EDA tool stating that encrypted
   vendor models are pass-through-only with audit-trail of
   every gate decision. This is a defence/aerospace-vertical
   selling point.

5. **Sovereign-cloud-native by default.** Datum's on-prem-only
   default deployment + `data_egress_policy: NoExternalNetwork`
   mode + local Unix-socket MCP transport gives Datum a
   credible "AI-native + air-gap-capable" positioning that no
   cloud-native AI-EDA competitor (e.g., the various
   AI-augmented Altium / Fusion Electronics offerings) can
   match.

6. **Per-vertical default profile suggestion.** When a user
   declares `industry_vertical: Automotive`, the AI surface can
   immediately suggest sensible defaults: `ipc_class: Class3A`,
   `intended_environment: Automotive`, `mandated_symbol_profile:
   None` (no automotive standard), `mandated_designator_profile:
   AsmeY14_44_2024`, `data_egress_policy: NoExternalAi`. This
   reduces compliance-posture configuration from a 15-field
   manual form to a one-field declaration with reviewable
   defaults.

7. **AI-surface part-substitution warnings as compliance
   evidence.** Every AI-suggested substitution that's accepted
   by the user becomes a transaction-logged event. The audit
   trail captures not just "this part was substituted" but
   "this part was substituted after the AI surface confirmed
   compliance posture preservation". This is an audit-friendlier
   AI-augmentation pattern than tools that AI-substitute
   without compliance-context awareness.

### Recommended Spec Edits

Concrete file:line edits for the user to review. Pattern follows
Standards Audit Batch 1 and Batch 2 (Domain 3 & Domain 2
deep-dives).

Claude is in research-only mode per the project's
`feedback_research_only_mode` rule; these recommendations are NOT
to be applied by the agent. The user will review, prioritise,
and apply via the standard spec-edit process.

| # | Source | Target file | Substance |
|---|--------|-------------|-----------|
| **Pass 0 — `STANDARDS_COMPLIANCE_SPEC.md` disposition refresh** ||||
| D4-0a | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.4 | Domain 4 dispositions refreshed: AEC-Q metadata promoted to "Planned (contract surface defined; `Part.qualification` extension)"; ISO 26262 / IEC 61508 / IEC 60601 / ISO 13485 / FDA Part 820 / EU MDR confirmed `Reference-only` with project-metadata-only positioning; ITAR / EAR / EU 2021/821 promoted to "Planned (project-metadata + data-egress-gate)"; 21 CFR Part 11 confirmed `Deferred with prerequisite` (Domain 8 audit-trail prerequisite); CMMC / ISO 27001 / NIST 800-171 added with `Reference-only` disposition (substrate paragraph); ATEX added with `Reference-only` disposition; AUTOSAR / DO-178C / ARP4754A / MIL-STD-1772 added with explicit `Out of scope` classifications; advisory exclusions (DO-254, DO-160, MIL-PRF-31032, MIL-PRF-55110, NASA-STD-8739, AS9100, IATF 16949, CMMI) confirmed for promotion to formal `Out of scope` in consolidated post-Domain-8 ratification pass |
| D4-0b | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 7 (Project-Level Compliance Metadata) | Section expanded to enumerate the specific fields (industry_vertical, ipc_class, intended_environment, itar_controlled, eu_dual_use_controlled, regulated_medical, safety_critical_industrial, export_control, intended_safety_integrity, mandated_symbol_profile, mandated_designator_profile, data_egress_policy, audit_overlay, export_control_markings, compliance_notes); cross-references to ENGINE_SPEC.md § 1.1a additions |
| D4-0c | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` new § 7.1 | New subsection "Substrate vs Certification Framing" — formal statement that Datum is substrate, never certifier; AI-surface wording rule for compliance language; reference to per-disposition substrate paragraphs |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** ||||
| D4-1 | this report | `specs/ENGINE_SPEC.md` § 1.1a | New shared enums: `IndustryVertical`, `IntendedEnvironment`, `IpcClass`, `UsmlCategory`, `Asil`, `Sil`, `Dal`, `MedicalClass`, `DataEgressPolicy`, `AuditOverlayMode`, `AecQGrade`, `AecQStandard`, `MilSpecQualification`, `RadiationTolerance`, `TemperatureGrade` |
| D4-2 | this report | `specs/ENGINE_SPEC.md` new § 1.x (Project Type) | New canonical type `Project { uuid, name, created, modified, schematic, board, rules, settings, pools, compliance: ProjectCompliance }`; new struct `ProjectCompliance` carrying the 15 fields enumerated in D4-0b; new struct `ExportControlPosture`; new struct `SafetyIntegrityDeclaration`. Note: `Project` is currently described in `NATIVE_FORMAT_SPEC.md` § 6.1 as JSON only — D4-2 is its first appearance as a canonical Rust type. |
| D4-3 | this report | `specs/ENGINE_SPEC.md` § 1.2 | Extend `Part` with `qualification: Option<PartQualification>`; new struct `PartQualification` carrying the 14 fields enumerated above |
| D4-4 | this report | `specs/ENGINE_SPEC.md` § 3 (Operations) | New operations: `SetProjectCompliance`, `SetIndustryVertical`, `SetIpcClass`, `SetIntendedEnvironment`, `SetExportControlPosture`, `SetDataEgressPolicy`, `SetAuditOverlayMode`, `SetPartQualification` — each with `inverse()` for undo |
| **Pass 2 — pool & native persistence** ||||
| D4-5 | this report | `docs/POOL_ARCHITECTURE.md` § 2 | `parts` SQL index table gains columns: `aec_q_grade`, `temperature_grade`, `mil_spec_qualification`, `radiation_tolerance`, `eccn` (all nullable); query API extended with `find_parts_by_qualification` |
| D4-6 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.1 | `project.json` schema gains `"compliance": { ... }` block carrying serialised `ProjectCompliance`; field is required-present with default values for new projects; existing projects without the block deserialise with defaults (schema_version remains 1; the `compliance` block is optional in deserialisation) |
| D4-7 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.x (parts persistence) | Per-Part native persistence gains `"qualification": { ... }` block carrying serialised `PartQualification`; optional |
| **Pass 3 — `specs/MCP_API_SPEC.md` (Compliance Tools section)** ||||
| D4-8 | this report | `specs/MCP_API_SPEC.md` new "Compliance Posture Tools" section | Section header + per-tool stubs for `get_project_compliance`, `set_project_compliance`, `set_industry_vertical`, `set_ipc_class`, `set_intended_environment`, `set_export_control`, `set_data_egress_policy`, `set_audit_overlay`, `validate_project_compliance`, `get_part_qualification`, `set_part_qualification`, `find_parts_by_qualification`, `infer_part_qualification`, `get_egress_policy_decision`, `audit_egress_attempts`, `validate_export_marking_propagation`. ~16 new tool stubs. |
| D4-9 | this report | `specs/MCP_API_SPEC.md` new "Data Egress Policy" section | Companion-policy section to "Encrypted Content Handling Policy" specifying the MCP-server-layer enforcement gate; declares which tools have external-network side effects; specifies the audit-trail entries written for each gate decision; declares the gate is mandatory at the MCP-server boundary (Python) and cannot be bypassed by Rust-side tools |
| D4-10 | this report | `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy | Cross-reference added to Data Egress Policy (D4-9); the two policies share the audit-trail integration for Domain 8 |
| **Pass 4 — `specs/IMPORT_SPEC.md`** ||||
| D4-11 | this report | `specs/IMPORT_SPEC.md` § 3 (KiCad) | KiCad `.kicad_pro` `meta` block fields are mapped to `Project.compliance` defaults on import (KiCad has no first-class compliance fields; mapping is best-effort heuristic) |
| D4-12 | this report | `specs/IMPORT_SPEC.md` new § (Compliance metadata import) | Specifies that imported projects without compliance metadata default to `industry_vertical: None`, `data_egress_policy: Unrestricted`, `ipc_class: None`, `audit_overlay: None`. User authority preserved — imported projects do not gain compliance metadata silently |
| **Pass 5 — architecture & guidance docs** ||||
| D4-13 | this report | `docs/STANDARDS_AUDIT_BATCH_3_GUIDANCE.md` (NEW) | Batch-3 bridging doc following the Batch-1 / Batch-2 pattern (must-land vs deferred, apply order, ProjectCompliance overlap with Domain 3 SymbolStyleProfile resolution, Pass 0 disposition refresh, Cross-Spec Update Rule compliance, advisory-exclusion ratification deferral) |
| D4-14 | this report | `docs/INTEROP_SCOPE.md` | Add "Industry-Vertical Compliance (research-staged)" section: Project compliance posture; Part qualification metadata; ITAR/EAR marking propagation; data-egress policy; substrate-vs-certification framing |
| D4-15 | this report | `docs/LIBRARY_ARCHITECTURE.md` after § "Canonical Datum Library Model" | New "Compliance metadata in Part records" subsection; cross-references D4-3 and D4-5; documents the qualification field's intended use |
| D4-16 | this report | `CLAUDE.md` § "Project Principles" | Add principle: "Datum is substrate for regulated-industry compliance work; Datum never claims to be the certifying party"; cross-reference to STANDARDS_COMPLIANCE_SPEC.md § 7.1 (D4-0c) |

**Total recommended spec edits:** **16** (3 disposition refreshes,
4 schema bedrock, 3 pool/native persistence, 3 MCP, 2 import, 4
architecture/guidance docs).

**This count matches Batch 1's count (16 edits)** — Domain 4 is
dominated by metadata-field additions and policy declarations
rather than algorithmic primitives, so the per-spec-file edit
count is moderate. The user can split into multiple PRs if the
batch is judged too large for a single review pass; suggested
split is "Pass 0 + Pass 1 + Pass 2" as Batch 3.0 (the schema
and disposition work) and "Pass 3 + Pass 4 + Pass 5" as Batch 3.1
(the MCP, import, and guidance docs).

## Cross-Domain Insights to Thread Forward

### To Domain 5 (materials & environmental)

- **RoHS / REACH exemptions intersect industry-vertical
  mandates.** Several RoHS exemptions are vertical-specific:
  Annex IV exempts medical devices and monitoring/control
  instruments through 2024-2027 (depending on category); Annex
  III exempts certain automotive applications. The
  `Project.compliance.industry_vertical` field is the natural
  driver for which exemptions apply. Domain 5 should consume
  `industry_vertical` when reasoning about applicable
  exemptions.

- **Defence-specific material restrictions.** Some materials
  acceptable for commercial designs are restricted for defence
  designs (e.g., Chinese-sourced rare-earth elements per US
  DoD CMMC-adjacent procurement rules; pure tin whisker risk
  for Class 3 / Class 3A). The `Part.qualification.eccn` field
  combined with `Project.compliance.industry_vertical: Defence`
  feeds Domain 5's material-source-risk analysis.

- **Conflict-minerals reporting (Dodd-Frank §1502, EU 2017/821)
  is BOM-export work.** Domain 5 owns the conflict-minerals
  attestation field on Part; Domain 4's `industry_vertical`
  drives whether attestation is required for the project.

- **`intended_environment` is shared.** Domain 4 introduces
  `IntendedEnvironment` enum; Domain 5 needs the same field for
  derate calculations (humidity exposure, temperature cycling,
  altitude). Coordinate the enum definition in
  `ENGINE_SPEC.md` § 1.1a so both domains share it.

### To Domain 6 (EMC & signal integrity)

- **Industry verticals drive EMC class.** Automotive →
  CISPR 25; medical → IEC 60601-1-2; industrial → CISPR 11;
  ITE → CISPR 32 (and EN 55032); aerospace → DO-160 (skip
  per Domain 4 advisory exclusion). The
  `Project.compliance.industry_vertical` field is the natural
  driver for default EMC-class selection. Domain 6 should
  consume `industry_vertical` when proposing EMC default rules.

- **Automotive CISPR 25 + AEC-Q intersect.** A automotive-
  vertical project should default to CISPR 25 EMC compliance
  expectations AND AEC-Q part qualification AND IPC Class 3A.
  The three fields are correlated; Domain 4's vertical
  declaration unifies them.

- **`safety_critical_industrial` projects need IEC 61000-6-2 /
  IEC 61000-6-4** (industrial environment immunity / emission)
  rather than the lower-grade IEC 61000-6-1 / IEC 61000-6-3
  consumer rules. Domain 6 should consume the
  `safety_critical_industrial: bool` flag.

### To Domain 7 (PLM & lifecycle integration)

- **AS9102 First Article Inspection consumes the variant
  substrate.** Datum's variant data model
  (`Variant` in `specs/ENGINE_SPEC.md` § 1.4 — variant
  authoring is in the schematic editor scope and `fitted_components`
  is persisted) is the substrate for AS9102 FAI traceability.
  Domain 4 confirms: when
  `Project.compliance.industry_vertical: Aerospace`, the FAI
  workflow becomes contractually relevant. Domain 7's PLM
  integration should consume the vertical declaration to drive
  FAI evidence-package formatting.

- **Supplier qualification systems intersect compliance posture.**
  Defence customers maintain Approved Vendor Lists (AVL);
  aerospace customers maintain Approved Manufacturers Lists
  (AML); medical customers maintain qualified-supplier
  databases. Domain 7's library-vault / supply-chain integration
  should consume `Project.compliance.industry_vertical` when
  filtering supplier offers.

- **ITAR-restricted PLM hosting.** Some PLM systems are not
  ITAR-compliant (Aras Innovator cloud, Arena PLM cloud SaaS).
  Domain 7's PLM integration should consult
  `Project.compliance.itar_controlled` and refuse to integrate
  with non-ITAR-compliant cloud PLM if the project is
  ITAR-controlled. Same gating pattern as `data_egress_policy`.

- **Octopart / Nexar / Digi-Key / Mouser external lookups** are
  exactly the calls gated by `data_egress_policy`. Domain 7
  should consume the gate at the supply-chain-integration layer.

### To Domain 8 (process & quality)

- **Domain 8 owns the audit-trail export contract; Domain 4
  consumes it.** Domain 4's `audit_overlay: AuditOverlayMode`
  field configures which Domain-8 audit-trail features are
  active for a project. The two domains MUST be coordinated so
  the audit-overlay enum values align with the audit-trail
  capabilities Domain 8 specifies.

- **Domain 8 owns the signature primitive; Domain 4 specifies
  signature-required transitions per vertical.** Medical
  projects (under 21 CFR Part 11) typically require signatures
  on `Approved` and `Released` state transitions; aerospace
  projects (under AS9100) typically require signatures on
  `Reviewed`, `Approved`, and `Released` transitions plus
  `FAIComplete`. Domain 4 specifies the per-vertical transition-
  signature mapping; Domain 8 implements the underlying
  signature mechanism.

- **Domain 8 owns ECO workflow; Domain 4 specifies which
  ECO records are compliance-evidence.** Forward annotation
  (Datum's existing M4 ECO contract) produces records suitable
  as compliance evidence. Domain 4 specifies which ECO records
  need to be exported to the compliance audit trail (typically
  all of them for medical / aerospace; selectively for
  consumer).

- **Title-block reviewer / approver fields cross-cut Domain 3.**
  Domain 3 owns the SheetFrame field model; Domain 8 owns the
  reviewer/approver metadata semantics; Domain 4 owns the
  vertical-specific requirements (medical requires reviewer
  count ≥ 2; aerospace requires designated authority signature;
  etc.).

- **Vocabulary alignment (IPC-T-50 + IEC 60050) is part of
  process-quality controlled-language compliance.** Domain 3
  introduced this; Domain 4 confirms its applicability for
  regulated-industry contexts (ISO 9001 controlled-source
  requirements; aerospace-grade documentation expectations).

## Sources

### Primary specifications

- [22 CFR Parts 120-130 (ITAR)](https://www.ecfr.gov/current/title-22/chapter-I/subchapter-M) — *International Traffic in Arms Regulations*. US Code of Federal Regulations; **free** at ecfr.gov.
- [22 CFR Part 121 (USML)](https://www.ecfr.gov/current/title-22/chapter-I/subchapter-M/part-121) — *United States Munitions List*. **Free**.
- [15 CFR Parts 730-774 (EAR)](https://www.ecfr.gov/current/title-15/subtitle-B/chapter-VII/subchapter-C) — *Export Administration Regulations*. **Free**.
- [15 CFR Part 774 (CCL)](https://www.ecfr.gov/current/title-15/subtitle-B/chapter-VII/subchapter-C/part-774) — *Commerce Control List* (ECCN catalog). **Free**.
- [21 CFR Part 11](https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11) — *Electronic Records; Electronic Signatures*. US FDA; **free**.
- [21 CFR Part 820 (QSR)](https://www.ecfr.gov/current/title-21/chapter-I/subchapter-H/part-820) — *Quality System Regulation*. US FDA; **free**.
- [21 CFR Part 820 / QMSR final rule (Feb 2024)](https://www.federalregister.gov/documents/2024/02/02/2024-01904/medical-devices-quality-system-regulation-amendments) — Federal Register notice of QMSR alignment with ISO 13485. **Free**.
- [Regulation (EU) 2017/745 (MDR)](https://eur-lex.europa.eu/eli/reg/2017/745/oj) — *Medical Device Regulation*. EU Official Journal; **free**.
- [Regulation (EU) 2021/821](https://eur-lex.europa.eu/eli/reg/2021/821/oj) — *EU Dual-Use Regulation*. **Free**.
- [Directive 2014/34/EU (ATEX)](https://eur-lex.europa.eu/eli/dir/2014/34/oj) — *Equipment for explosive atmospheres*. **Free**.
- [32 CFR Part 170 (CMMC)](https://www.ecfr.gov/current/title-32/subtitle-A/chapter-I/subchapter-XIII/part-170) — *Cybersecurity Maturity Model Certification*. US DoD; **free** at ecfr.gov (final rule October 2024).
- [NIST SP 800-171 Rev 2](https://csrc.nist.gov/pubs/sp/800/171/r2/final) — *Protecting Controlled Unclassified Information in Nonfederal Systems and Organizations*. **Free**.
- [NIST SP 800-171 Rev 3 (final draft)](https://csrc.nist.gov/pubs/sp/800/171/r3/fpd) — Rev 3 final-public-draft (2024). **Free**.
- [NIST SP 800-172](https://csrc.nist.gov/pubs/sp/800/172/final) — *Enhanced Security Requirements for Protecting Controlled Unclassified Information*. **Free**.
- [ISO/IEC 27001:2022](https://www.iso.org/standard/27001) — *Information security management systems — Requirements*. ISO Webstore (~CHF 130); **paywalled**.
- [ISO/IEC 27002:2022](https://www.iso.org/standard/75652.html) — *Information security controls*. ISO Webstore; **paywalled**.
- [ISO 13485:2016 + A1:2021](https://www.iso.org/standard/59752.html) — *Medical devices — QMS — Requirements for regulatory purposes*. ISO Webstore (~CHF 200); **paywalled**.
- [IEC 60601-1:2005+A1:2012+A2:2020](https://webstore.iec.ch/publication/67497) — *Medical electrical equipment — General requirements for basic safety and essential performance*. IEC Webstore (~CHF 350); **paywalled**.
- [IEC 60601-1-2:2014+A1:2020](https://webstore.iec.ch/publication/67191) — *…Collateral standard: Electromagnetic disturbances*. IEC Webstore; **paywalled**.
- [ISO 26262:2018 (12 parts)](https://www.iso.org/standard/68383.html) — *Road vehicles — Functional safety*. ISO Webstore (~CHF 1500 total); **paywalled**.
- [IEC 61508:2010 (7 parts)](https://webstore.iec.ch/publication/22273) — *Functional safety of electrical/electronic/programmable electronic safety-related systems*. IEC Webstore (~CHF 1800 total); **paywalled**.
- [IEC 62443 (multi-part)](https://webstore.iec.ch/publication/22273) — *Industrial communication networks — IT security for networks and systems*. IEC Webstore; **paywalled**.
- [AEC-Q100 Rev-J (2024)](http://www.aecouncil.com/AECDocuments.html) — *Failure Mechanism Based Stress Test Qualification for Integrated Circuits*. AEC; ~USD 100; **paywalled**.
- [AEC-Q101 Rev-E1 (2021)](http://www.aecouncil.com/AECDocuments.html) — *Stress Test for Discrete Semiconductors*. AEC; **paywalled**.
- [AEC-Q200 Rev-E (2024)](http://www.aecouncil.com/AECDocuments.html) — *Stress Test for Passive Components*. AEC; **paywalled**.
- [RTCA DO-254 (2000)](https://www.rtca.org/products/do-254/) — *Design Assurance Guidance for Airborne Electronic Hardware*. RTCA; ~USD 250; **paywalled**.
- [RTCA DO-178C (2011)](https://www.rtca.org/products/do-178c/) — *Software Considerations in Airborne Systems and Equipment Certification*. RTCA; **paywalled**.
- [RTCA DO-160G (2010)](https://www.rtca.org/products/do-160g/) — *Environmental Conditions and Test Procedures for Airborne Equipment*. RTCA; **paywalled**.
- [SAE ARP4754A (2010)](https://www.sae.org/standards/content/arp4754a/) — *Guidelines for Development of Civil Aircraft and Systems*. SAE; **paywalled**.
- [MIL-PRF-31032D (2017)](https://quicksearch.dla.mil/qsDocDetails.aspx?ident_number=276519) — *Printed Circuit Board / Printed Wiring Board, General Specification*. US DoD; **free** at DLA.
- [MIL-PRF-55110G (2018)](https://quicksearch.dla.mil/) — *Printed Wiring Board, Rigid, General Specification*. US DoD; **free**.
- [MIL-PRF-38535L (2024)](https://quicksearch.dla.mil/) — *Integrated Circuits Manufacturing, General Specification*. US DoD; **free**.
- [MIL-PRF-19500P (2022)](https://quicksearch.dla.mil/) — *Semiconductor Devices, General Specification*. US DoD; **free**.
- [MIL-STD-275E (cancelled 1994)](https://quicksearch.dla.mil/) — *Printed Wiring for Electronic Equipment*. US DoD; **free**; superseded by IPC-2221.
- [NASA-STD-8739.2A (2018)](https://standards.nasa.gov/standard/NASA/NASA-STD-87392) — *Surface Mount Technology Workmanship*. NASA; **free** at standards.nasa.gov.
- [NASA-STD-8739.3A (2018)](https://standards.nasa.gov/standard/NASA/NASA-STD-87393) — *Through-Hole Technology Workmanship*. NASA; **free**.
- [NASA-STD-8739.4A (2018)](https://standards.nasa.gov/standard/NASA/NASA-STD-87394) — *Crimping, Interconnecting Cables, Harnesses, and Wiring*. NASA; **free**.

### Cybersecurity / data-protection references

- [DoD Cyber AB CMMC Documentation](https://www.cyberab.org/) — CMMC accreditation body; CMMC Assessment Process documentation. **Free**.
- [DoD CMMC Final Rule (Federal Register, Oct 2024)](https://www.federalregister.gov/documents/2024/10/15/2024-22905/cybersecurity-maturity-model-certification-cmmc-program) — 32 CFR Part 170 final rule. **Free**.
- [DoD DFARS 252.204-7012](https://www.acquisition.gov/dfars/252.204-7012-safeguarding-covered-defense-information-and-cyber-incident-reporting) — *Safeguarding Covered Defense Information and Cyber Incident Reporting* (NIST 800-171 invocation clause). **Free**.

### EU regulatory references

- [EU Conflict Minerals Regulation 2017/821](https://eur-lex.europa.eu/eli/reg/2017/821/oj) — EU 3TG due diligence. **Free**.
- [EU Dodd-Frank §1502 SEC implementation](https://www.sec.gov/divisions/corpfin/cfconflictminerals.htm) — US conflict-minerals reporting. **Free**.

### Reference implementations and ecosystem documentation

- [BIS Commerce Control List Web Tool](https://efoia.bis.doc.gov/index.php/electronic-foia/index-of-documents/7-electronic-foia/231-export-control-classification-number-eccn) — official ECCN classification lookup. **Free**.
- [DoS DDTC — ITAR Resources](https://www.pmddtc.state.gov/) — official ITAR / USML resources. **Free**.
- [FDA — 21 CFR Part 11 Guidance for Industry (2003)](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/part-11-electronic-records-electronic-signatures-scope-and-application) — risk-based scoping interpretation. **Free**.
- [FDA — 2024 Draft Guidance: Electronic Systems, Records, and Signatures in Clinical Investigations](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/electronic-systems-electronic-records-and-electronic-signatures-clinical-investigations) — current FDA interpretation. **Free**.
- [EUDAMED database](https://ec.europa.eu/tools/eudamed/) — EU MDR Unique Device Identification database. **Free** (registration required for some actors).
- [JEDEC JEP106](https://www.jedec.org/standards-documents/docs/jep-106au) — Standard Manufacturer's Identification Code. JEDEC; **free** with registration.

### EDA tool documentation

- [Altium Vault Workflow / Lifecycle](https://www.altium.com/altium-365/lifecycle-management) — Altium Vault sign-off workflow; relevant for 21 CFR Part 11 substrate. **Free** documentation.
- [Cadence Pulse / Allegro Design Workbench](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis.html) — Cadence PLM-integration documentation. **Free**.
- [Siemens Xpedition Enterprise Automotive Pack](https://eda.sw.siemens.com/en-US/pcb/xpedition/enterprise/) — Xpedition automotive ASIL-aware reporting. **Free** marketing pages.
- [KiCad Issues — 21 CFR Part 11 / sign-off discussion](https://gitlab.com/kicad/code/kicad/-/issues) — KiCad community discussion (search "21 CFR" / "audit trail" / "sign-off"). **Free**.
- [KiCad — Project Settings (.kicad_pro)](https://docs.kicad.org/master/en/eeschema/eeschema.html) — KiCad project metadata format. **Free**.

### Forum / industry discussion

- [r/PrintedCircuitBoard — AEC-Q discussion](https://www.reddit.com/r/PrintedCircuitBoard/) — community discussion on automotive part qualification. **Free**.
- [EEVblog forum — ITAR / EAR discussions](https://www.eevblog.com/forum/) — community discussion on export-control marking workflow. **Free**.
- [r/PCB — Defence-design tooling discussion](https://www.reddit.com/r/PCB/) — recurring threads on tooling for ITAR-controlled work. **Free**.
- [Aviation Stack Exchange](https://aviation.stackexchange.com/) — DO-254 / DO-178C / ARP4754A community Q&A. **Free**.
- [Medical Device HQ forums](https://medicaldevicehq.com/) — ISO 13485 / 21 CFR Part 11 / MDR discussion. **Free**.
- [Defense Electronics LinkedIn group](https://www.linkedin.com/groups/) — defence-electronics community discussion (search "DO-254 EDA tools" / "MIL-PRF EDA"). **Free** (LinkedIn registration).
- [IPC EDGE community](https://www.ipcedge.org/) — IPC training material and community discussion. **Free** registration.
- [SAE TechSelect](https://www.sae.org/learn/) — SAE training catalog including ARP4754A / AS9100 / IATF 16949 courses. **Paid**.

### Cross-references (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 4 — Phase 1 inventory of Domain 4.
- `research/standards-audit/STANDARDS_AUDIT.md` § 4 advisory exclusions list — re-affirmed in this report.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` — IPC class metadata cross-reference.
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Encrypted Models — encrypted vendor model handling cross-reference (Batch-1 contract).
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Pin attributes — pin direction / Part record extension precedent.
- `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md` § To Domain 4 — industry-mandated symbol-style profiles cross-reference.
- `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md` § Title-block fields — `SheetFrame.classification` cross-reference for ITAR / EAR marking propagation.
- `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` — export-adapter scope (cross-reference for ITAR / EAR marking propagation through Gerber X3 / IPC-2581 / ODB++).
- `docs/CANONICAL_IR.md` — canonical IR (transaction model for new operations).
- `docs/POOL_ARCHITECTURE.md` — pool architecture (parts SQL index extension for qualification fields).
- `docs/LIBRARY_ARCHITECTURE.md` — library architecture (Part qualification metadata as library extension).
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md` — Batch-1 integration pattern (model for Batch-3 guidance doc).
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md` — meta-rules for research → spec integration.
- `specs/STANDARDS_COMPLIANCE_SPEC.md` — controlling standards spec (Pass 0 disposition refresh target).
- `specs/ENGINE_SPEC.md` — canonical types (Pass 1 schema bedrock target).
- `specs/NATIVE_FORMAT_SPEC.md` — on-disk persistence (Pass 2 update target).
- `specs/MCP_API_SPEC.md` — MCP API (Pass 3 update target; Encrypted Content Handling Policy companion).
- `specs/IMPORT_SPEC.md` — import semantics (Pass 4 update target).
- `CLAUDE.md` — project framing (Pass 5 principle addition target).
