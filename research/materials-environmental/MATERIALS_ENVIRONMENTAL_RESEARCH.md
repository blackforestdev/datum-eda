# Materials & Environmental Compliance — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 5 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 5`
> ("Per-Domain Audit → 5. Materials & environmental").
> Cross-references `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
> § IPC-1752 (lines 744-764) for IPC-1752A material-declaration content
> (do not re-research),
> `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
> § Supply Chain for the Octopart/Nexar/Digi-Key/Mouser supply-chain
> lookup contract that Domain 5 consumes for substance-data refresh,
> and `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md`
> for the `Project.compliance` block, the `IndustryVertical` enum, and
> the shared `IntendedEnvironment` enum that Domains 4 and 5 jointly
> own.
>
> Reads against the post-Standards-Audit-Batch-1 spec baseline merged
> 2026-04-18 (PR #1). The contract surfaces this report builds on
> (`Part` extension shape in `ENGINE_SPEC.md` § 1.2; `Stackup` material
> properties in § 1.3; `ModelAttachment`/`ModelProvenance` in § 1.1a;
> `MCP_API_SPEC.md` § Supply Chain tools and § Encrypted Content
> Handling Policy; `STANDARDS_COMPLIANCE_SPEC.md` § 4.5 disposition
> baseline) all exist as of that merge.

> **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
>
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is available.
>
> Domain 5 carries a moderate advisory-exclusion list. The audit
> recommended the following as "Recommended low-priority / skip" —
> they are NOT deep-dived in this report and are surfaced only in
> § "Pending Exclusions (re-affirmed)" with cross-cutting-value
> notes:
>
> - **California Prop 65** — chemical-warning labelling is a
>   downstream compliance artifact, not an EDA-tool surface
> - **EU Packaging & Packaging Waste Directive (94/62/EC + amendments)**
>   — packaging is fab/assembly concern, not EDA
> - **RoHS-exemption tracking** — exemption-list maintenance is
>   regulatory-domain work; Datum can store an "exemption applied"
>   free-form ID but never the catalog itself
>
> These are positioning statements, not feature requirements. None of
> the three has hidden cross-cutting value that would justify
> re-opening the deep-dive.

## Executive Summary

- **The central framing for Domain 5 is "Datum consumes substance-list
  data, never maintains it".** RoHS, REACH SVHC, China RoHS, J-MOSS,
  TSCA section 6(h), the SCIP database, and IEC 62474's DSL list all
  have one thing in common: **the lists update too frequently for any
  EDA tool to be the catalog of record**. The SVHC candidate list is
  refreshed by ECHA twice a year (typically January and June);
  TSCA section 6(h) restrictions added five new PBT chemicals
  in 2021 and continue to expand; RoHS exemption catalog Annex III
  and Annex IV are revised on rolling schedules with delegated EU
  Commission acts. **Datum's role is to ingest substance-presence
  declarations from upstream sources** (IPC-1752A submissions from
  vendors, Octopart/Nexar parametric lookups, IEC 62474 DSL exports,
  ChemSHERPA imports for Japanese supply chains) and to **carry the
  results in stable per-Part metadata**. Datum never validates the
  authority of a cited exemption; that is the regulatory-affairs
  function of the user's compliance team.

- **The Part record is the natural pinch-point for substance-presence
  metadata, in the same shape as Domain 4's `qualification` block.**
  Batch 1 already extended `Part` with `behavioural_models`, `thermal`,
  `manufacturer_jep106`, `packaging_options`, `supply_chain_offers`.
  Domain 4 proposed a `qualification: Option<PartQualification>` block.
  Domain 5 proposes a parallel `compliance: Option<PartCompliance>`
  block carrying `rohs_status`, `rohs_exemption: Option<String>`
  (free-form, NOT validated), `reach_svhc: Vec<SubstanceCandidate>`,
  `halogen_free: Option<HalogenFreeStatus>`, `tsca_status`,
  `china_rohs_status`, `j_moss_status`, `conflict_minerals_status`,
  `cmrt_attestation: Option<DocumentRef>`, `last_compliance_check:
  Option<DateTime<Utc>>`, plus a `compliance_evidence:
  Vec<ComplianceEvidence>` collection of documentary attestations.
  Effort: **~3 days** for the data model + serialisation + atomic
  MCP get/set surface; **+2 days** for the Octopart/Nexar refresh
  integration that populates the fields.

- **The Project record needs a `materials_posture` sub-block parallel
  to Domain 4's `compliance` block — and, in fact, nested inside it.**
  Domain 4's recommended `Project.compliance: ProjectCompliance`
  carries the cross-cutting compliance posture (industry vertical,
  IPC class, intended environment, ITAR markings). Domain 5
  recommends adding a `materials_posture: MaterialsPosture` field
  inside `ProjectCompliance` with sub-fields:
  `target_rohs_revision: Option<RohsRevision>`,
  `requires_halogen_free: bool`, `requires_reach_svhc_disclosure: bool`,
  `requires_china_rohs: bool`, `requires_jmoss: bool`,
  `requires_conflict_minerals_attestation: bool`, `requires_iec_62474:
  bool`, `svhc_list_pinned_date: Option<NaiveDate>`,
  `rohs_exemptions_acknowledged: Vec<String>`. The single
  `requires_*` boolean fan-out drives validation, BOM-export
  enrichment, and substance-disclosure lint diagnostics. Effort:
  **~1 day** added on top of the Domain 4 ProjectCompliance work.

- **IPC-1752A is the canonical XML carrier for materials declarations
  and Datum should be a first-class consumer (import) and producer
  (export). It is NOT itself a substance-list registry.** IPC-1752A
  Rev B (July 2020) defines three declaration classes (Class A
  yes/no compliance assertion, Class B declarable-substance-groups
  presence, Class C full composition breakdown to ~100 ppm). It
  carries RoHS, REACH, China RoHS, J-MOSS, halogen-free, and SCIP
  substance presence data in a single schema, declared per-component
  by the manufacturer. **Datum's "consume IPC-1752A imports, emit
  IPC-1752A exports" is the highest-leverage Domain 5 implementation
  per engineering hour.** Importing a vendor's IPC-1752A submission
  populates `Part.compliance.*` directly; exporting the project's
  per-Part compliance data as a project-wide IPC-1752A document
  produces the artifact downstream tools (Assent, Source Intelligence,
  iPoint Compliance, Sphera Material Compliance) expect. Per-Part
  IPC-1752A import is **~1 week** of XML-parser work; project-wide
  export is **~3 days** more. The IPC-1752 schema is **freely
  downloadable** from `ipc.org` (with free registration), so this
  is a tractable undertaking.

- **Substance-list-version pinning is the single most overlooked
  regulated-industry compliance hygiene practice and Datum can lead
  on it.** When a user marks a project "RoHS-compliant", **which
  RoHS revision?** RoHS 2 (the 2011/65/EU baseline 6-substance list),
  the 2015/863 amendment 10-substance list (adding the four
  phthalates DEHP / BBP / DBP / DIBP), or RoHS 3 (the 2019 medical-
  device / monitoring-instrument scope expansion)? When the user
  says "REACH SVHC disclosed", **as of which SVHC list date?** The
  list is refreshed twice yearly by ECHA. A project authored against
  the SVHC list as of January 2026 is materially different from the
  same project re-claimed as compliant in July 2026 if any newly-
  added substance is now present in a BOM line. **Datum's
  recommendation: every compliance posture declaration carries
  pinning fields** (`target_rohs_revision: Rohs2_2011 | Rohs2_2015
  | Rohs3_2019`, `svhc_list_pinned_date: NaiveDate`,
  `iec_62474_dsl_version: Option<String>`). Pinning is recorded as
  authored data; the pin update is itself a transaction event (so
  the audit trail captures "the user moved the SVHC pin from
  2025-01 to 2025-07; here are the diagnostics that fired against
  the new list"). This is a Datum-differentiator: no other EDA tool
  surveyed treats substance-list-version pinning as first-class.

- **Conflict-minerals reporting (CMRT 3TG and EMRT cobalt+mica) is
  the BOM-export work where Domain 5 actively produces value, not
  just stores metadata.** Dodd-Frank §1502 (US, 2010) mandates 3TG
  (tin / tungsten / tantalum / gold) sourcing disclosure for SEC
  filers; EU Regulation 2017/821 mandates the same for EU importers
  of 3TG above thresholds since January 2021. The de-facto industry
  interchange is the Responsible Minerals Initiative (RMI) **Conflict
  Minerals Reporting Template (CMRT)** — a versioned Excel template
  (CMRT 6.32 is current 2026), with a JSON-export companion. The
  **Extended Minerals Reporting Template (EMRT)** adds cobalt and
  mica beyond 3TG; widely used for EV-battery supply chains.
  **Datum should emit CMRT and EMRT exports** from per-Part
  compliance data plus the project's BOM. The CMRT structure is
  documented (RMI publishes the template format publicly), the
  required per-Part inputs are CAS-number presence flags, and the
  output is one Excel/JSON file per project. Effort: **~1 week**
  for the CMRT exporter (after the underlying `Part.compliance`
  field lands) and **~3 days** for the EMRT extension.

- **Halogen-free is a single-flag declaration with two-source
  authority; Datum should carry it as a structured per-Part field
  and a project-level posture flag.** JEDEC JS709C (the current
  2024 revision; supersedes JS709B 2017) defines halogen-free as
  **<900 ppm Br, <900 ppm Cl, <1500 ppm Br + Cl combined**. IEC
  61249-2-21 is the IEC's parallel laminate-substrate qualification
  standard with the same thresholds. The two are functionally
  equivalent for declaration purposes; vendors typically cite both.
  **Datum's `Part.compliance.halogen_free: Option<HalogenFreeStatus>`
  carries `{ Compliant, NotCompliant, Unknown, ComplianceClaimed
  WithoutEvidence }` plus a free-text `halogen_free_basis` field for
  the cited standard ("JS709C" / "IEC 61249-2-21" / vendor-specific).
  The project-level `requires_halogen_free: bool` flag drives a
  validation pass that warns when any BOM line lacks a positive
  declaration.** This is one of the cheapest lint diagnostics with
  the highest practical value — a real workflow improvement for any
  customer with a green-electronics product specification.

- **End-of-life directives (WEEE, EU Battery Regulation 2023/1542,
  ELV, ESPR / Digital Product Passport) are mostly NOT engine-side
  work but the Digital Product Passport is the genuinely emerging
  topic the audit underweighted.** WEEE registration and recycling
  fees are organisational regulatory work; Datum can store a "WEEE
  registered" flag at the project level but does not produce WEEE
  filings. EU Battery Regulation 2023/1542 (effective February 2024,
  phased through 2027) imposes carbon-footprint declaration, recycled-
  content mandates, and the upcoming Battery Passport on industrial
  and EV batteries — **but only when the board includes a battery**;
  for the ~99% of PCBs without an integrated cell, this is irrelevant
  metadata. ELV (End-of-Life Vehicles 2000/53/EC) is the historical
  predecessor of automotive RoHS and shares its substance list; the
  ELV-specific exemptions are tracked through the Domain 4 vertical
  selection. **The genuinely emerging topic is ESPR's Digital
  Product Passport (DPP)** — adopted in EU Regulation 2024/1781
  (effective July 2024), the DPP creates a per-product digital
  identifier carrying material composition, repairability scores,
  recyclability metrics, and supplier-chain provenance, with phased
  implementation through 2030. The DPP is **not yet a hard EDA-tool
  requirement** but is on track to become one for any product
  marketed in the EU. **Recommend: Datum monitors DPP development;
  no implementation work this milestone, but the `materials_posture`
  block should reserve `dpp_required: bool` and `dpp_identifier:
  Option<String>` fields for forward compatibility.**

- **The Octopart / Nexar / Digi-Key / Mouser external-lookup contract
  from Domain 2 is the right substance-data refresh source, and the
  Domain 5 work specifies what fields land where.** Octopart's
  parametrics include `RoHS Status`, `Lead Free`, `REACH SVHC`,
  `Halogen Free`, `Conflict Minerals Status`, and `Lifecycle Status`
  for most parts in the catalog. Nexar (Octopart's GraphQL API)
  exposes them as part of the `specs` field. Digi-Key returns
  RoHS status and lead-free designation in its standard product
  response; Mouser similar. **Datum's `refresh_supply_chain` MCP
  tool (Domain 2 / Batch 1) should grow a contract addendum**: when
  it refreshes a Part's `supply_chain_offers`, it also refreshes the
  `Part.compliance.*` fields in a single atomic operation. The
  refresh emits an audit-trail event capturing the source URL, the
  fields populated, and the timestamp — feeding Domain 8's audit
  trail. This is **architectural coordination work, not new code**:
  the `refresh_supply_chain` tool gains additional output fields;
  the underlying HTTP call is the same. **Effort: ~2 days** to
  extend the refresh contract.

- **Compliance-posture lint at validate time is the highest-leverage
  AI-surface integration and the natural Domain 5 deliverable for
  the M7+ window.** When a user runs `validate_project_compliance`
  (Domain 4 MCP tool), the materials sub-validator should emit
  diagnostics like:
  - "Project requires halogen-free (`requires_halogen_free: true`)
    but Part U3 (TPS54622 from TI) has `halogen_free: None`."
  - "Project requires REACH SVHC disclosure but Part C42's
    `last_compliance_check` is 2025-09-15, which is before the
    pinned SVHC list date 2026-01-15 — refresh required."
  - "Project's `target_rohs_revision: Rohs3_2019` but Part R7's
    `rohs_status: Compliant` cites `rohs_basis: 'RoHS 2 (2011/65)'`
    — vendor declaration predates RoHS 3 scope."
  - "Project `industry_vertical: Medical` and `ipc_class: Class3`
    enables RoHS Annex IV exemptions — Part U2's
    `rohs_exemption: '7(c)-I'` cited; Datum does not validate the
    cited exemption ID; verify with your regulatory team."
  These diagnostics are **structured (machine-readable) so MCP tool
  callers can surface them**; the AI surface can explain the gap
  in natural language ("you have a halogen-free requirement but
  three BOM lines lack the declaration; here are the parts and the
  vendor pages to check"). **Effort: ~3-4 days** for the lint
  rules + integration with the Domain 4 validate-project-compliance
  surface.

- **There is no paywalled-standards crisis for Domain 5; the public-
  domain regulatory text covers nearly everything.** EU regulations
  (RoHS 2011/65/EU, REACH EC 1907/2006, WEEE 2012/19/EU, Battery
  Regulation 2023/1542, ELV 2000/53/EC, ESPR 2024/1781, EU Conflict
  Minerals 2017/821) are all **free** at `eur-lex.europa.eu`. US
  regulations (TSCA, Dodd-Frank §1502 SEC implementing rules) are
  **free** at `ecfr.gov` and `sec.gov`. China RoHS (SJ/T 11364-2014,
  GB/T 26572-2011) is **free** but Mandarin-only on official
  Chinese-government sites; English summaries are available from
  several commercial compliance consultancies. JEDEC JS709C is
  **free with JEDEC registration**. IEC standards (IEC 62474, IEC
  61249-2-21) are **paywalled** from IEC Webstore at ~CHF 200-300
  each — but the schema content of IEC 62474 is **freely accessible**
  via the IEC 62474 Database (`std.iec.ch/iec62474`), so the
  implementation-relevant metadata is available. IPC-1752A
  technical reports (IPC-1752A-WAM1) are **free with IPC.org
  registration**. ChemSHERPA's specification is **free** at
  `chemsherpa.net` (English documentation available). IMDS
  (International Material Data System) is **commercial-platform-
  only** — operated by HP / DXC for the automotive industry; the
  per-substance schema is documented but the platform itself
  requires per-company subscription. Total Domain 5 standards
  acquisition cost for honest research: **~CHF 600-800** (IEC
  documents only, optional given the free database). **No
  paywalled-standards constraint blocks the Domain 5 work.**

- **The biggest cross-domain dependency is Domain 7 (PLM &
  lifecycle integration).** Octopart/Nexar/Digi-Key/Mouser supply-
  chain lookups feed both Domain 2 (component modelling) and
  Domain 5 (substance disclosure) and Domain 7 (supply-chain /
  lifecycle / availability). The three domains share one underlying
  HTTP call; they differ in which response fields they care about.
  **Coordinate the API contract** so a single
  `refresh_supply_chain(part_uuid)` call populates Domain 2's
  `behavioural_models`/`thermal` (when available; rare in
  distributor APIs), Domain 5's `compliance.rohs_status` /
  `reach_svhc` / `halogen_free`, and Domain 7's
  `supply_chain_offers` / `lifecycle` / `last_supply_chain_check`.
  This is the consolidated "what fields a single refresh
  populates" decision the spec needs to make explicit; today the
  fields are scattered across Batch-1 commits and three Phase-2
  reports. The Domain 7 deep-dive should ratify the consolidated
  field map.

## Standards Catalog

### Substance Restriction Directives

#### EU RoHS 2 (Directive 2011/65/EU + Delegated Directive 2015/863)

**Full title.** **Directive 2011/65/EU** — *Restriction of the use of
certain Hazardous Substances in electrical and electronic equipment
(RoHS recast)*. Replaced the original RoHS Directive 2002/95/EC.
Amended by **Commission Delegated Directive (EU) 2015/863** which
added four phthalates (DEHP, BBP, DBP, DIBP) to Annex II, taking
the restricted-substance count from 6 to 10.

**Restricted substances (Annex II).**
- **Lead (Pb)** — 0.1% (1000 ppm) maximum concentration by weight
  in homogeneous materials
- **Mercury (Hg)** — 0.1%
- **Cadmium (Cd)** — 0.01% (100 ppm)
- **Hexavalent chromium (Cr VI)** — 0.1%
- **Polybrominated biphenyls (PBB)** — 0.1%
- **Polybrominated diphenyl ethers (PBDE)** — 0.1%
- **Bis(2-ethylhexyl) phthalate (DEHP)** — 0.1% (added 2015/863,
  effective July 2019)
- **Butyl benzyl phthalate (BBP)** — 0.1% (added 2015/863)
- **Dibutyl phthalate (DBP)** — 0.1% (added 2015/863)
- **Diisobutyl phthalate (DIBP)** — 0.1% (added 2015/863)

**Issuing body.** **EU** (European Parliament and Council;
Commission for delegated acts). DG Environment is the responsible
Directorate-General. Member States transpose the directive into
national law (e.g., UK Statutory Instrument 2012/3032 for the UK;
the German Elektro- und Elektronikgeräte-Stoff-Verordnung
ElektroStoffV for Germany).

**Scope and category structure.** RoHS 2 covers EEE in 11
categories (Annex I): large household appliances; small household
appliances; IT and telecommunications; consumer equipment; lighting;
electrical and electronic tools; toys; **medical devices** (Cat 8);
**monitoring and control instruments** (Cat 9); automatic
dispensers; other EEE not covered (catch-all Cat 11). The Cat 8
and Cat 9 inclusions came in stages from 2014 (RoHS 2) through
2019 (industrial monitoring instruments and in-vitro diagnostics).
**Annex III** lists time-limited exemptions; **Annex IV** lists
medical-device-specific and monitoring-and-control-specific
exemptions.

**Adoption status (2026).** **Mainstream-mandatory** for any EEE
sold in the EU. Effective dates have all passed; the 2024 RoHS
review consultation considered tightening (substance additions
TBPA, MCCPs, additional phthalates) but no immediate amendment is
in force as of early 2026; ECHA's RoHS pack publication is
expected late 2026 with effective dates 2027-2028.

**License / IP.** **Free.** EU directives are public-domain at
`eur-lex.europa.eu`. Substance-list updates are published as
delegated acts in the EU Official Journal.

**EDA tool support.** Discussed in the EDA Tool Support Matrix
section below (covers Altium, OrCAD, PADS, Cadence, KiCad, Eagle/
Fusion, Horizon, LibrePCB, DipTrace, EasyEDA, Datum-current,
Datum-post-Domain-5 for all per-Part compliance fields).

**Datum coverage status.** `Planned` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5 ("RoHS, REACH, IPC-1752A,
IEC 62474 posture and metadata fields: `Planned`"). **Confirm**;
this report defines the contract surface (`Part.compliance.rohs_status`,
`Project.compliance.materials_posture.target_rohs_revision`).

**Datum implementation cost.**
- Data model: `RohsStatus` enum (`Compliant | NonCompliant |
  CompliantWithExemption | Unknown | ComplianceClaimedWithoutEvidence`)
  + `RohsRevision` enum (`Rohs2_2011 | Rohs2_2015 | Rohs3_2019`) +
  `Part.compliance.rohs_status` field + `Part.compliance.rohs_basis:
  Option<String>` (vendor-cited revision string) +
  `Part.compliance.rohs_exemption: Option<String>` (free-form Annex III
  / Annex IV reference like "7(c)-I" or "9(b)-III").
- Validator: a per-Part RoHS-status checker against the project's
  pinned `target_rohs_revision`; emits structured diagnostics.
- Export adapter: BOM export adds `rohs_status` / `rohs_exemption`
  columns; IPC-1752A export consumes the field.
- MCP API: `set_part_rohs_status`, `get_part_rohs_status`,
  `find_parts_by_rohs_status`, `validate_project_rohs_compliance`.

**Strategic recommendation.** **Implement now** as part of the
`PartCompliance` block (HR-1 below). The RoHS field is the
single most-asked-for materials-disclosure attribute; not having
it is a credibility gap.

**Risks.**
- **Vendor-data accuracy risk.** Vendor RoHS declarations can
  cite older RoHS revisions (RoHS 1 / RoHS 2 pre-2015 amendment)
  while claiming current compliance. Mitigation: the
  `rohs_basis` field captures the cited revision so reviewers
  can detect mismatch with the project's target revision.
- **Exemption-catalog rot risk.** If Datum maintained an
  exemption catalog, the catalog would need bi-annual updates as
  Annex III / IV exemption expirations pass. **Datum does NOT
  maintain the exemption catalog**; the `rohs_exemption` field
  is free-form text. Validation that the exemption is still in
  force is the user's regulatory team's responsibility.
- **Homogeneous-material reading risk.** RoHS thresholds apply
  per *homogeneous material*, not per component. A solder joint
  containing 1% Pb on the entire weight of a 0603 resistor is
  RoHS-compliant if the solder itself is RoHS-exempt under
  exemption 7(c)-I, even though the gross-component-weight
  calculation might exceed 1000 ppm Pb. Datum cannot perform
  this calculation; vendor declarations are authoritative.

#### RoHS 3 — Directive (EU) 2017/2102 + 2024 review

**Full title.** **Directive (EU) 2017/2102** — amends RoHS 2 to
remove the 22 July 2019 expiration of the medical-devices
exemption and to extend RoHS scope to **in-vitro diagnostic
medical devices** (effective 22 July 2016) and to certain
industrial monitoring and control instruments (effective 22 July
2017 / 2019). Often colloquially called "RoHS 3" though the
directive itself is an amendment to RoHS 2.

**Scope expansion.**
- **Cat 8** (medical devices) — RoHS-applicable since 2014;
  in-vitro diagnostics added 2016.
- **Cat 9** (monitoring and control instruments) —
  RoHS-applicable since 2014; **industrial monitoring and control
  instruments** added 2019.
- **Cat 11** (catch-all "other EEE") — RoHS-applicable since 22
  July 2019.

**Substance list.** Identical to RoHS 2 + 2015/863 (10
substances). RoHS 3 changes scope, not substances.

**Adoption status (2026).** **Mainstream-mandatory** for
medical-device manufacturers (especially in-vitro diagnostics —
glucose meters, blood-gas analysers, imaging readouts) and for
industrial monitoring/control instruments (factory automation,
process control, energy management). The 2024 RoHS review (see
above) is the next-expected revision.

**License / IP.** **Free** at `eur-lex.europa.eu`.

**Datum coverage status.** Same as RoHS 2; the `RohsRevision::
Rohs3_2019` enum variant disambiguates the project's target.
The substance-list Datum carries is identical (10 substances).

**Strategic recommendation.** Implementation is unified with
RoHS 2 / 2015. The only Domain-5 specific addition is the
revision-pin enum.

#### EU REACH (Regulation EC 1907/2006)

**Full title.** **Regulation (EC) No 1907/2006** — *Registration,
Evaluation, Authorisation and Restriction of Chemicals (REACH)*.
In force June 2007. Administered by the European Chemicals Agency
(ECHA, Helsinki). Continuously updated by Commission regulations
amending Annexes XIV (authorisation list) and XVII (restriction
list) plus the SVHC Candidate List.

**Issuing body.** **EU** (European Commission, EU Council, EU
Parliament). Administered by **ECHA** (European Chemicals Agency).

**Scope.** REACH is the EU's general chemical regulation; it
governs the manufacture, import, and use of chemical substances
in the EU. Three operational substance lists are immediately
EDA-relevant:
- **SVHC Candidate List** (Substances of Very High Concern)
  — substances under consideration for authorisation. Updated
  ~bi-annually by ECHA. As of January 2026 contains **240
  substances** including many phthalates, brominated flame
  retardants, lead compounds, and per- and polyfluoroalkyl
  substances (PFAS).
- **Annex XIV** (Authorisation List) — substances requiring EU
  authorisation for use after a sunset date. Subset of SVHC list
  promoted to actual restriction. ~70 substances.
- **Annex XVII** (Restriction List) — substances with use
  restrictions (concentration limits, banned applications).

**Article 33 obligation.** Suppliers of articles containing SVHCs
above 0.1% w/w must inform downstream recipients. This is the
operative SVHC-disclosure obligation that flows from REACH into
EDA tools — vendors declare SVHC presence to their customers,
who must in turn declare it to *their* customers, all the way
down to the final article producer.

**SCIP database.** (Detailed below — separate sub-section.)

**Adoption status (2026).** **Mainstream-mandatory.** REACH is
the EU's general chemical regulation; every EEE imported into or
sold in the EU is subject to it. Brexit forked **UK REACH**
(2021) which mirrors EU REACH structure with separate UK SVHC
list maintenance.

**License / IP.** **Free** at `eur-lex.europa.eu` and
`echa.europa.eu`. The SVHC list itself is freely downloadable
from `echa.europa.eu/candidate-list-table` in CSV / Excel /
XML formats. **No EDA tool needs to maintain this list**; ECHA
publishes machine-readable updates.

**EDA tool support.** Same matrix as RoHS — discussed below.

**Datum coverage status.** `Planned` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5. **Confirm**; contract is
`Part.compliance.reach_svhc: Vec<SubstanceCandidate>`.

**Datum implementation cost.**
- Data model: `SubstanceCandidate` struct
  (`{ cas_number: String, ec_number: Option<String>, name: String,
    concentration_pct: Option<f32>, list_revision_date:
    NaiveDate, evidence_source: Option<String> }`) +
  `Part.compliance.reach_svhc: Vec<SubstanceCandidate>` field.
- Validator: per-Part SVHC-disclosure checker against the
  project's pinned `svhc_list_pinned_date`.
- Export adapter: BOM export adds an SVHC column; IPC-1752A
  Class B/C export consumes the per-Part list.
- MCP API: `set_part_reach_svhc`, `get_part_reach_svhc`,
  `find_parts_with_svhc`, `refresh_svhc_from_octopart` (delegated
  to `refresh_supply_chain`), `pin_svhc_list_date`.

**Strategic recommendation.** **Implement now** as part of the
`PartCompliance` block. SVHC reporting is the second most-asked-
for materials-disclosure attribute after RoHS.

**Risks.**
- **List-rot risk.** ECHA refreshes the SVHC list ~bi-annually
  (typically January and June). Without a project-level
  `svhc_list_pinned_date`, "compliant" loses meaning over time.
  Mitigation: pinning is mandatory at the project level; refresh
  is an authored transaction.
- **0.1% w/w threshold reading risk.** The threshold applies per
  *article*. For complex assemblies, the per-component disclosure
  must roll up correctly; Datum carries the per-Part declarations
  but does not perform the assembly-level rollup (that is BOM-
  export consumer work, with appropriate weight data sourced
  from packaging-options / vendor mass).
- **PFAS uncertainty.** The EU is considering a near-blanket
  PFAS restriction (the Universal PFAS Restriction proposal,
  submitted to ECHA in 2023; expected outcome 2026-2027). If
  adopted, the SVHC list will see a major expansion. Datum's
  consume-don't-maintain framing handles this without code
  change; the user pins to the new list date when ready.

#### EU REACH SCIP Database (ECHA, in force January 2021)

**Full title.** **SCIP** — *Substances of Concern In articles, as
such or in complex objects (Products)*. Established under Article
9(1)(i) of the Waste Framework Directive (Directive 2008/98/EC as
amended by Directive (EU) 2018/851). Submission obligation in
force since **5 January 2021**.

**Issuing body.** **ECHA** (European Chemicals Agency).

**Scope.** Suppliers of articles containing SVHCs above 0.1% w/w
must submit information to the SCIP database, which is
publicly accessible. Submission identifies the article, the
SVHC, the concentration range, and the article's category in the
EU classification (per the European Single Procurement Document
ESPD article codes). Submission is via the **IUCLID 6** desktop
software (ECHA-distributed, free) or via the IUCLID Cloud Service
(free for SMEs, paid for larger volumes).

**SCIP versus IPC-1752A.** SCIP is an EU-mandated submission
target; IPC-1752A is a supplier-to-customer XML interchange
format. The two complement each other: IPC-1752A is the
*transport*; SCIP is the *destination* for articles sold into
the EU. A vendor's IPC-1752A submission to its customer
typically carries the data the customer's SCIP submission will
consume.

**Adoption status (2026).** **Mainstream-mandatory** for EU-
imported / EU-sold articles containing SVHCs above the threshold.
Penalties for non-submission vary by Member State; typical fines
EUR 50,000 - 500,000 per violation.

**License / IP.** **Free.** ECHA infrastructure is free; the
IUCLID submission software is free; the SCIP database is publicly
queryable at `echa.europa.eu/scip-database`.

**EDA tool support.** No EDA tool surveyed performs SCIP
submission directly; this is regulatory-affairs-tool work
(commonly Assent Compliance, Source Intelligence, iPoint
Compliance, Sphera Material Compliance, or in-house IUCLID
workflows). EDA tools' role is to **emit the per-Part substance
data that feeds the SCIP submission**, typically as IPC-1752A
exports.

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5 (under "WEEE, conflict
minerals, JS709C, ELV, China RoHS, SCIP" — the "stable
compliance-declaration schema on `Part`" prerequisite is the
Domain 5 work this report defines). **Once the `PartCompliance`
block lands, SCIP support promotes to `Reference-only` with
substrate paragraph** (Datum produces the IPC-1752A export that
feeds SCIP; Datum does not submit to SCIP itself).

**Datum implementation cost.** Zero engine-side code beyond what
RoHS / REACH already requires. The per-Part `reach_svhc` field
plus IPC-1752A export is the SCIP-feeding substrate.

**Strategic recommendation.** **Reference-only**; document the
SCIP-feeding role of IPC-1752A export. **Out of scope:** direct
IUCLID submission, SCIP database query.

#### China RoHS (GB/T 26572-2011 + SJ/T 11364-2014)

**Full title.**
- **GB/T 26572-2011** — *Requirements of concentration limits for
  certain restricted substances in electrical and electronic
  products*. National standard (GB/T = recommended national
  standard).
- **SJ/T 11364-2014** — *Marking for the restricted use of
  hazardous substances in electronic and electrical products*
  (industry standard; "China RoHS marking" definition).
- **Order No. 32** (China MIIT, January 2016) — *Management
  Methods for the Restriction of the Use of Hazardous Substances
  in Electrical and Electronic Products*. The administrative
  measure that gives the standards regulatory teeth.

**Issuing body.** **MIIT** (Ministry of Industry and Information
Technology of the People's Republic of China). The standards
themselves are published by SAC (Standardization Administration
of China).

**Scope.** Restricts the same six substances as the original RoHS
1 (lead, mercury, cadmium, hexavalent chromium, PBB, PBDE) in
electrical and electronic products. **Does not** include the four
phthalates added by EU RoHS 2015/863 — China RoHS substance list
is the older, narrower 6-substance list. **Does** require
distinctive markings on the product including the
**Environment Protection Use Period (EPUP)** symbol (a number
inside a circle indicating years before the substance becomes a
hazard) and the substance-presence table (a 5x6 table indicating
which restricted substances are present in which sub-assemblies).

**EPUP marking.** "10" inside an orange circle is the most common
EPUP value (10 years); some products use "20", "30", "40", "50";
green circle indicates no restricted substances above limits.

**Adoption status (2026).** **Mainstream-mandatory** for products
sold or imported into China. China RoHS-2 (the 2016 update) added
the catalogue management approach: products in MIIT's Conformity
Assessment Catalogue have additional certification requirements;
products outside the catalogue follow self-declaration.

**License / IP.** **Free** but Mandarin-only on official Chinese
sources (`miit.gov.cn`, `sac.gov.cn`). English summaries are
available from commercial compliance consultancies (SGS, TUV,
Intertek) and from Chinese-government bilingual portals.

**EDA tool support.** No EDA tool surveyed first-class-supports
China RoHS markings. Workaround: BOM export annotation;
title-block notation. Datum opportunity: structured per-Part
field + project-level `requires_china_rohs: bool` flag.

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5. **Promote to `Planned`**
once the `PartCompliance` block lands; the `china_rohs_status:
Option<ChinaRohsStatus>` field is a near-zero-cost addition.

**Datum implementation cost.**
- Data model: `ChinaRohsStatus` enum (`Compliant | NonCompliant |
  CompliantWithEpup10y | CompliantWithEpup20y |
  CompliantWithEpup30y | Unknown`) + free-form
  `china_rohs_substance_table: Option<ChinaRohsSubstanceTable>`
  carrying the 5x6 presence matrix per substance per sub-assembly.
- Validator: per-Part check against project's
  `requires_china_rohs` flag.
- Export adapter: BOM export adds China RoHS column; IPC-1752A
  Class B/C carries the substance table.
- MCP API: `set_part_china_rohs`, `get_part_china_rohs`.

**Strategic recommendation.** **Implement now** as part of the
`PartCompliance` block. Trivial incremental cost given the
broader RoHS work; meaningful for any China-market product.

**Risks.** Substance-list divergence between EU RoHS (10
substances) and China RoHS (6 substances) means a part can be
EU-RoHS-non-compliant and China-RoHS-compliant simultaneously
(if the divergence is the four phthalates). Datum's separate
`rohs_status` and `china_rohs_status` fields capture this; the
validator must not conflate them.

#### Korea RoHS / K-REACH

**Full title.**
- **Korea Act on Resource Circulation of Electrical and
  Electronic Equipment and Vehicles** (Act No. 8405, 2007;
  amended several times) — Korea's RoHS-equivalent legislation.
- **K-REACH** — **Act on Registration and Evaluation of
  Chemical Substances** (Act No. 11789, 2013) — Korea's REACH-
  equivalent.

**Issuing body.** **Ministry of Environment of the Republic of
Korea**.

**Scope.** Korea RoHS substantially mirrors EU RoHS substance
restrictions (six-substance baseline; phthalate amendments
synchronised with EU updates); K-REACH mirrors EU REACH for
chemical substance reporting in articles. K-REACH has its own
**KOSHA list** (substance reporting list) parallel to ECHA SVHC.

**Adoption status (2026).** **Mainstream-mandatory** for
products sold in South Korea. Korean OEMs (Samsung, LG, Hyundai-
Kia, SK Hynix) treat K-REACH compliance as a supplier
qualification gate.

**License / IP.** **Free** at `me.go.kr` (Ministry of
Environment) and `nics.me.go.kr` for K-REACH-specific resources;
Korean-only on official sources, English translations available
from commercial consultancies.

**EDA tool support.** No EDA tool surveyed first-class-supports
Korea RoHS / K-REACH. Same workaround as China RoHS.

**Datum coverage status.** No current classification.
**Recommendation:** treat as `Reference-only` initially; if a
Korean-market customer surfaces, promote to `Planned` for the
specific KOSHA-list substance fields. The substance content is
largely covered by the EU REACH SVHC field already proposed;
the Korean-specific delta is small.

**Strategic recommendation.** **Reference-only** initially; do
not implement Korea-specific fields speculatively. The
`Part.compliance.reach_svhc` field carries the substantive
data; a `korea_rohs_status: Option<KoreaRohsStatus>` enum can
be added on demand.

#### Japan J-MOSS / JIS C 0950

**Full title.** **JIS C 0950** — *Marking for presence of the
specific chemical substances for electrical and electronic
equipment*. Current edition **JIS C 0950:2021** (revised from
2008 to align with updated METI guidelines). The "J-MOSS" name
(Japan Marking of presence of the Specific Substances) is the
common-usage designation.

**Issuing body.** **JISC** (Japanese Industrial Standards
Committee). Administered under **METI** (Ministry of Economy,
Trade and Industry).

**Scope.** Japan's RoHS-equivalent marking requirement. Covers
the same six substances as the original RoHS 1 (lead, mercury,
cadmium, hexavalent chromium, PBB, PBDE). Requires a "J-MOSS
mark" on products containing any restricted substance above the
threshold; products below threshold may carry the green "J-MOSS
Green Mark". The marking includes a substance-presence table
analogous to China RoHS.

**Adoption status (2026).** **Regional-mandatory** for products
sold in Japan. Japanese OEMs (Sony, Panasonic, Toyota, Honda,
Nintendo, Murata, TDK) treat J-MOSS compliance as a supplier
qualification gate. Compliance is typically declared via
ChemSHERPA submissions in the Japanese supply chain.

**License / IP.** **Paywalled** through JSA (Japanese Standards
Association) at ~JPY 5000-10,000 per standard. Practical
implementation is documented in METI's free guidelines and
in ChemSHERPA's free schema.

**EDA tool support.** Same as China RoHS — no first-class
support; workaround is BOM annotation.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Planned` disposition; the `j_moss_status: Option<JMossStatus>`
field on `PartCompliance` is a near-zero-cost addition.

**Datum implementation cost.**
- Data model: `JMossStatus` enum (`GreenMark | OrangeMark |
  Unknown`) + per-substance presence flags inherited from the
  RoHS substance list.
- Validator: project-level `requires_jmoss: bool` triggers a
  per-Part check.
- Export adapter: BOM export adds J-MOSS column; ChemSHERPA
  export consumes the field (ChemSHERPA carries J-MOSS data
  natively).
- MCP API: `set_part_jmoss`, `get_part_jmoss`.

**Strategic recommendation.** **Implement now** as part of the
`PartCompliance` block. Trivial incremental cost; meaningful
for any Japan-market product.

**Risks.** J-MOSS substance list lags EU RoHS phthalate
additions; same risk profile as China RoHS.

#### TSCA (US Toxic Substances Control Act)

**Full title.** **Toxic Substances Control Act**, 15 USC
Chapter 53, originally enacted 1976; substantially amended by
the **Frank R. Lautenberg Chemical Safety for the 21st Century
Act** (Public Law 114-182, 2016). Implementing regulations at
**40 CFR Parts 700-799**.

**Issuing body.** **US EPA** (Environmental Protection Agency),
Office of Pollution Prevention and Toxics.

**Scope.** US chemical-management regime covering manufacture,
import, processing, and disposal of chemical substances. TSCA
section 6(h) targets **persistent, bioaccumulative, and toxic
(PBT) chemicals**; the EPA published five PBT restrictions in
January 2021:
- **Decabromodiphenyl ether (DecaBDE)** — flame retardant
- **Phenol, isopropylated phosphate (3:1) (PIP-3:1)** — plasticiser
- **2,4,6-tris(tert-butyl)phenol (2,4,6-TTBP)** — antioxidant
- **Hexachlorobutadiene (HCBD)** — solvent
- **Pentachlorothiophenol (PCTP)** — rubber additive

**PIP-3:1 specifically** has been a major industry compliance
event for electronics — PIP-3:1 is widely used as a plasticiser
in PVC, in flexible PCBs, in cable insulation, and in some
solder pastes. The original 2021 deadline was extended several
times under continuing pressure from electronics industry
associations; the current effective compliance date is October
2024 for most articles.

**Adoption status (2026).** **Mainstream-mandatory** for any
article imported into or sold in the US containing the listed
PBT substances. The EPA continues to add substances under TSCA
section 6 (chrysotile asbestos, methylene chloride, and others);
industry expects more additions through 2026-2028.

**License / IP.** **Free** at `ecfr.gov/current/title-40/chapter-I/
subchapter-R`. EPA guidance documents are free at `epa.gov`.

**EDA tool support.** No EDA tool surveyed first-class-supports
TSCA section 6(h). Same workaround pattern as REACH SVHC.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Planned` disposition; the `tsca_status: Option<TscaStatus>`
field on `PartCompliance`. Substance presence is captured
through the `reach_svhc` field (the substance list is
overlapping; PIP-3:1 / DecaBDE / HCBD are also REACH SVHCs).

**Datum implementation cost.**
- Data model: `TscaStatus` enum (`Compliant |
  ContainsRestrictedSubstance | Unknown`) +
  `tsca_pbt_substances: Vec<TscaPbtSubstance>` carrying the
  per-substance presence flag.
- Validator: project-level `requires_tsca_section_6h: bool`
  triggers per-Part check.
- Export adapter: BOM export adds TSCA column.
- MCP API: `set_part_tsca`, `get_part_tsca`.

**Strategic recommendation.** **Implement now** as a small
extension to the SVHC field; the substance list is small (5
substances as of early 2026) and the data model overlap with
SVHC means the marginal cost is negligible.

**Risks.** Substance-list expansion. EPA continues to add
substances; the `Part.compliance.tsca_pbt_substances` field
must accommodate growth without schema migration. Mitigation:
the field is a `Vec<TscaPbtSubstance>` rather than a
substance-by-substance fixed enum.

#### California Prop 65 (skipped — see exclusion section)

Per the audit's advisory exclusion list (re-affirmed in this
report's § "Pending Exclusions"), Prop 65 is **out of EDA tool
scope**. The chemical-warning labelling requirement is a
downstream compliance artifact attached to consumer-product
packaging, not a board-design surface. Vendor-provided Prop 65
warnings are typically free-form text strings not tractable as
structured data. **Do not implement Prop 65-specific Datum
fields.** The substance presence relevant to Prop 65 (Prop 65
list overlaps heavily with REACH SVHC, TSCA section 6(h), and
the RoHS list) is captured through the existing fields.

### Halogen-Free Standards

#### JEDEC JS709C (current 2024) / IEC 61249-2-21

**Full title.**
- **JEDEC JS709C (March 2024)** — *Definition of "Halogen-Free"
  for Electronic Components and Assemblies*. Current revision;
  supersedes JS709B (March 2017) and JS709A (April 2010).
- **IEC 61249-2-21:2003** — *Materials for printed boards and
  other interconnecting structures — Part 2-21: Reinforced base
  materials clad and unclad — Non-halogenated epoxide woven
  E-glass reinforced laminated sheets of defined flammability,
  copper-clad*. Substrate-qualification standard with the same
  thresholds as JS709C.

**Issuing body.** **JEDEC** (JEDEC Solid State Technology
Association) for JS709C; **IEC** (TC 91 — Electronics assembly
technology) for IEC 61249-2-21.

**Scope.** Defines "halogen-free" for electronic-industry use.
Both standards specify the same numerical thresholds:
- **<900 ppm bromine (Br)** in any component or homogeneous
  material
- **<900 ppm chlorine (Cl)** in any component or homogeneous
  material
- **<1500 ppm total halogen** (Br + Cl combined)

These thresholds are the consensus industry definition. JS709C
specifically applies to electronic components and assemblies;
IEC 61249-2-21 specifically applies to laminate substrates
(FR-4-class base material). Vendors typically cite both standards
when declaring halogen-free.

**Adoption status (2026).** **Mainstream**, especially for
consumer-electronics products marketed as "green electronics"
or for industrial products with corporate environmental
certification (ISO 14001-driven sustainability programs).
Major OEMs (Apple, Samsung, HP, Dell, Lenovo, Sony, Panasonic)
specify halogen-free as a supplier-qualification requirement.

**License / IP.**
- **JEDEC JS709C** — **free** with JEDEC registration at
  `jedec.org` (free account; download access).
- **IEC 61249-2-21:2003** — **paywalled** at IEC Webstore (~CHF
  280); also available through national-standards-body
  redistribution (ANSI, BSI, DIN, JISC).

**EDA tool support.** No EDA tool surveyed first-class-supports
halogen-free as a structured per-Part field. Workaround: vendor
parametric in `Part.parametric` map; free-text custom property;
BOM annotation.

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5 (under "WEEE, conflict
minerals, JS709C, ELV, China RoHS, SCIP" — same prerequisite as
SCIP). **Promote to `Planned`** once the `PartCompliance` block
lands.

**Datum implementation cost.**
- Data model: `HalogenFreeStatus` enum (`Compliant | NotCompliant
  | Unknown | ComplianceClaimedWithoutEvidence`) +
  `halogen_free: Option<HalogenFreeStatus>` field +
  `halogen_free_basis: Option<String>` (e.g., "JS709C", "IEC
  61249-2-21", "vendor-internal").
- Validator: project-level `requires_halogen_free: bool` triggers
  per-Part check; emits structured diagnostics.
- Export adapter: BOM export adds halogen-free column;
  IPC-1752A Class B/C carries the field; ChemSHERPA carries it
  natively.
- MCP API: `set_part_halogen_free`, `get_part_halogen_free`,
  `find_parts_without_halogen_free_declaration`.

**Strategic recommendation.** **Implement now** as part of the
`PartCompliance` block. Halogen-free is the most-asked-for
"green electronics" metadata after RoHS / SVHC; not having it
is a credibility gap for any consumer-electronics customer.

**Risks.** **Bromine measurement reading risk.** The 900 ppm
bromine threshold can be met by vendor-substitution of brominated
flame retardants with phosphorus-based or nitrogen-based
alternatives. The replacement substances may have their own
toxicity / SVHC implications (the "regrettable substitution"
problem). Datum carries the declaration but does not assess
substitution alternatives. Mitigation: the `reach_svhc` field
captures any regrettable-substitution concerns separately.

#### IEC 61249 family (substrate qualification)

**Full title.** **IEC 61249 (multi-part)** — *Materials for
printed boards and other interconnecting structures*. The series
covers laminate substrates by composition and process:
- **IEC 61249-2-X** — Reinforced base materials clad and unclad
  (the core PCB-substrate parts; -21 is the halogen-free epoxy
  variant detailed above)
- **IEC 61249-3-X** — Unreinforced base materials clad and unclad
- **IEC 61249-4-X** — Sectional specifications for prepregs
- **IEC 61249-5-X** — Sectional specifications for finished
  copper-clad laminates

**Adoption status (2026).** **Mainstream** as a substrate-
qualification framework. Typically referenced in PCB-fabricator
material-stackup data sheets ("FR-4 per IEC 61249-2-7", "high-
Tg per IEC 61249-2-7 with FT-2 designation").

**License / IP.** **Paywalled** at IEC Webstore (~CHF 200-300
per part).

**Datum relevance.** **Substrate-qualification metadata.** The
existing `Stackup.material_name` field
(`specs/ENGINE_SPEC.md` § 1.3, post-Batch-1) is the natural home
for IEC 61249 designations. No new schema needed; existing
free-text field is sufficient. **Datum's role**: pass through
the IEC 61249 designation in the stackup material declaration
without validating it.

**Datum coverage status.** `Reference-only`; the
`Stackup.material_name` field carries the IEC 61249 designation
verbatim. **No new disposition needed.**

**Strategic recommendation.** **Reference-only**. No new field
work; the existing stackup material-name string carries the
IEC 61249 reference.

**Cross-domain note.** Halogen-free laminates have different
Dk/Df than standard FR-4 — typically 4.0-4.4 vs 4.4-4.8 for Dk
at 1 GHz, and 0.010-0.015 vs 0.018-0.025 for Df. The Dk/Df
fields in `StackupLayer` (post-Batch-1, `dielectric_constant`
and `loss_tangent`) capture this for impedance calculation
(Domain 6). The two domains share the stackup data structure;
no Domain 5 work needed beyond the Stackup.material_name
free-text reference.

### Material-Declaration Formats

#### IPC-1752A (cross-ref IPC research)

**Already-researched in detail:**
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-1752
(lines 744-764).

**Brief recap (per IPC research):**
- **IPC-1752 Rev B (July 2020)** — current; SCIP / REACH-aligned.
  Three declaration classes: Class A (yes/no), Class B (declarable
  substance groups), Class C (full composition to ~100 ppm).
- **XML format** for vendor-to-customer materials declarations.
- **Carries**: RoHS substance presence, REACH SVHC presence,
  China RoHS substance table, J-MOSS substance table, halogen-
  free status, conflict-minerals status, recycled-content
  declarations, SCIP database identifiers.
- **License / IP**: schema **free** with IPC registration at
  `ipc.org`.

**Domain 5-specific framing: Datum is producer + consumer.**

The IPC research treated IPC-1752A as out-of-scope for M7. This
report **revises that framing**: IPC-1752A is the natural
single XML carrier for nearly every Domain 5 substance-disclosure
field, and Datum's BOM-shaped artifacts can both consume IPC-1752A
imports (populating `Part.compliance.*` from a vendor-supplied
declaration) and emit IPC-1752A exports (producing the project-
wide declaration that downstream regulatory-affairs tools
consume).

**Datum implementation cost.**
- **IPC-1752A import** (per-Part):
  - Parser for IPC-1752A Rev B XML (~1 week of work, schema is
    documented and bounded).
  - Mapper from IPC-1752A fields to `Part.compliance.*` fields.
  - MCP tool `import_ipc_1752a_for_part(part_uuid, xml_path)`.
- **IPC-1752A export** (project-wide):
  - Serialiser for IPC-1752A Rev B XML (~3 days of work after
    importer lands; schema reuse).
  - Tool `export_ipc_1752a_for_project(project_uuid, output_path,
    declaration_class: A | B | C)`.

**Strategic recommendation.** **Implement now** as the canonical
substance-disclosure interchange. IPC-1752A as a single XML
covers ~80% of the materials-disclosure use cases.

**Risks.** **Schema versioning.** IPC-1752 Rev C is in committee
discussion as of 2026; expected publication 2026-2027. Datum
should structure the parser / serialiser to accommodate version
drift; the AI surface should warn when consuming an out-of-spec
or unrecognised version.

#### IPC-1754 (automotive-specific)

**Full title.** **IPC-1754** — *Materials and Substances
Declaration for Aerospace and Defense and Other Industries
Requiring Sustained Production*. Note: despite the "automotive
extension" framing in the audit (which is a common industry
shorthand), IPC-1754 is actually targeted at **aerospace,
defence, and other long-lifecycle industries** — its IPC-1752A
relationship is "extension for industries that need decades-long
data persistence and additional substance categories beyond the
EU RoHS / REACH scope".

**Status.** Published 2017; current revision IPC-1754. Used by
some aerospace primes (Lockheed Martin, Boeing Defense, Raytheon)
and select automotive OEMs that prefer the IPC-stack over the
IMDS-stack.

**Adoption status (2026).** **Niche.** IMDS dominates
automotive supply chains; IPC-1754 has limited adoption. For
aerospace, no single-format consensus exists — each prime
maintains its own preferred substance-data interchange.

**License / IP.** **Paywalled** at IPC.org (~USD 100); IPC-1754
is not free even with registration.

**EDA tool support.** No EDA tool surveyed first-class-supports
IPC-1754. Implementation is via IPC-1752A pass-through with
extended substance fields.

**Datum coverage status.** No current classification.
**Recommendation:** **Reference-only.** Do not invest in IPC-
1754-specific parsing; the substance content is largely
covered by IPC-1752A + the extended PartCompliance fields.
If an aerospace customer requires IPC-1754, the Datum-emitted
IPC-1752A export plus the per-Part qualification metadata
(Domain 4's `qualification` block) is the substrate for an
external IPC-1754 conversion.

**Strategic recommendation.** **Reference-only / Out of scope
for v1.** Re-evaluate if a defence or aerospace customer
specifically demands IPC-1754.

#### IPC-1755 (additional profiles)

**Full title.** **IPC-1755** — *Conflict Minerals Data
Exchange*. Published 2018; IPC's parallel to the RMI CMRT
template.

**Adoption status (2026).** **Niche.** RMI's CMRT (Excel-based)
dominates conflict-minerals reporting; IPC-1755 (XML-based) has
limited adoption despite being the more structured, more
machine-friendly format.

**License / IP.** **Paywalled** at IPC.org (~USD 100).

**Datum coverage status.** **Reference-only.** Datum's
recommended conflict-minerals export is **CMRT** (the de-facto
RMI standard) rather than IPC-1755. If an IPC-stack-aligned
customer demands IPC-1755, the data substrate (per-Part
`conflict_minerals_status` + project-level
`requires_conflict_minerals_attestation`) is sufficient for an
external converter.

**Strategic recommendation.** **Out of scope for v1.** CMRT is
the operational format.

#### IEC 62474 (DSL — Declarable Substance List)

**Full title.** **IEC 62474:2018** (with multiple amendments
through 2024) — *Material declaration for products of and for
the electrotechnical industry*. The standard plus its **DSL
(Declarable Substance List)** maintained at the IEC 62474
Database (`std.iec.ch/iec62474`).

**Issuing body.** **IEC** TC 111 (Environmental standardization
for electrical and electronic products and systems).

**Scope.** XML-based material-declaration format for the
electrotechnical industry. Defines:
- **DSL** — the substance list (~500 substances as of early
  2026) with declaration thresholds and reference standards.
  Updated quarterly by the IEC TC 111 working group.
- **Reportable application** — the specific use of a substance
  that triggers the declaration (e.g., flame retardant in
  cable insulation).
- **Material classification** — the IEC 62474 Category
  taxonomy for material composition reporting.

IEC 62474 is widely used in **automotive supply chains** alongside
IMDS — IMDS for the OEM-internal database; IEC 62474 as the XML
data exchange format (especially Tier-2 to Tier-1 to OEM data
flow).

**Adoption status (2026).** **Mainstream-niche.** Heavy adoption
in automotive (Bosch, Continental, Denso, Aptiv) and industrial
electronics (Siemens, ABB, Schneider Electric, Rockwell
Automation). Less common in pure consumer-electronics supply
chains where IPC-1752A dominates.

**License / IP.**
- **Standard text**: **paywalled** at IEC Webstore (~CHF 200).
- **DSL itself**: **free** at `std.iec.ch/iec62474`. The
  database is publicly queryable and downloadable as XML.
- **Schema**: published with the standard; XSD files are
  redistributed by some commercial implementations.

**EDA tool support.** No EDA tool surveyed first-class-supports
IEC 62474. Pattern: vendor IEC 62474 declarations are imported
into PLM (Teamcenter, Windchill, Aras) or commercial compliance
platforms (Assent, Source Intelligence) and propagated as
filterable BOM metadata.

**Datum coverage status.** `Planned` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5. **Confirm.**

**Datum implementation cost.**
- **IEC 62474 DSL XML import** (per-Part):
  - Parser for IEC 62474 XML declarations (~5 days, schema
    bounded).
  - Mapper from IEC 62474 fields to `Part.compliance.*`
    (substantial field overlap with IPC-1752A; IEC 62474 carries
    additional substance-application metadata that goes into
    `Part.compliance.iec_62474_applications:
    Vec<Iec62474Application>`).
  - MCP tool `import_iec_62474_for_part(part_uuid, xml_path)`.
- **IEC 62474 DSL XML export** (project-wide):
  - Serialiser for IEC 62474 XML (~3 days after importer
    lands).
  - MCP tool `export_iec_62474_for_project(project_uuid,
    output_path)`.
- **DSL version pinning**: `Project.compliance.materials_posture.
  iec_62474_dsl_version: Option<String>` (e.g., "2024-Q4").

**Strategic recommendation.** **Implement post-M7** as the
second-tier substance-disclosure interchange after IPC-1752A.
Automotive customers in particular expect IEC 62474 alongside
IPC-1752A.

**Risks.** **DSL version drift.** IEC TC 111 updates the DSL
quarterly. Same pinning approach as REACH SVHC; the DSL version
date is captured in `Project.compliance.materials_posture.
iec_62474_dsl_version`.

#### IMDS (International Material Data System)

**Full title.** **International Material Data System (IMDS)**.
Operated by **HP / DXC Technology** under contract from the
automotive industry. Founded 2000 by Mercedes-Benz, Audi, BMW,
Ford, Porsche, Volvo, and Volkswagen; now used by every major
global automotive OEM.

**Scope.** Commercial platform for automotive-supply-chain
material composition reporting. Tier-N suppliers submit material
data sheets (MDS) into the IMDS database; OEMs roll up the data
to vehicle-level material declarations for ELV (End-of-Life
Vehicles) compliance and recycled-content tracking.

**Format.** IMDS data sheets follow a proprietary MDS schema;
import/export to the IMDS database is via the IMDS web platform
or via the **IMDS-AI** (Application Interface) for system
integrators. The schema overlaps substantially with IEC 62474
but is not identical.

**Adoption status (2026).** **Mainstream-mandatory** in global
automotive supply chains. Approximately 1000+ OEM and Tier-1
companies are IMDS members; Tier-2 / Tier-3 access is via OEM
sponsorship.

**License / IP.** **Commercial-platform-only.** Per-company
subscription required. Schema is documented but the platform
itself is paid. The ELV directive recognises IMDS as the
de-facto compliance evidence for vehicle-level substance
reporting.

**EDA tool support.** No EDA tool surveyed first-class-supports
IMDS. Pattern: PLM tools (Teamcenter, Windchill, Aras) export
BOM data to commercial compliance platforms (Assent, Source
Intelligence, iPoint) which submit to IMDS on behalf of the
manufacturer.

**Datum coverage status.** No current classification.
**Recommendation:** **Out of scope** for direct IMDS submission;
**Reference-only** with substrate paragraph (Datum's IPC-1752A
export plus per-Part compliance metadata feeds the commercial-
compliance-platform IMDS submission workflow).

**Strategic recommendation.** **Out of scope for direct
integration.** IMDS submission is regulatory-affairs-tool work
inherently coupled to a paid commercial platform. The Datum
substrate (IPC-1752A export + per-Part `conflict_minerals_status`
+ `reach_svhc` + `china_rohs_status`) is sufficient as input
for IMDS-submission-tool consumption.

#### ChemSHERPA (Japan)

**Full title.** **ChemSHERPA — Chemical Substance Information
Sharing Scheme to Promote Communication on Substances in
Articles**. Maintained by the **Joint Article Management
Promotion-consortium (JAMP)** in Japan.

**Scope.** Japan's industry-standard chemical-management data-
sharing scheme. Replaces the older **JAMP-MSDSplus** and the
**Article Information Sheet (AIS)** as of October 2018. Carries
RoHS, REACH SVHC, J-MOSS, China RoHS, and additional
Japan-specific substance-disclosure data in a single Excel-or-XML
format.

**Two flavours.**
- **chemSHERPA-CI** (Chemical Information) — the chemical-
  substance disclosure carrier.
- **chemSHERPA-AI** (Article Information) — the article-level
  composition declaration.

**Adoption status (2026).** **Mainstream-mandatory** in Japanese
electronics supply chains. Sony, Panasonic, Toyota, Honda,
Nintendo, Murata, TDK, and most Japanese Tier-1 / Tier-2
suppliers require chemSHERPA submissions from upstream suppliers.

**License / IP.** **Free.** Specification, schema, and free-tier
authoring tool are at `chemsherpa.net`. English documentation
available.

**EDA tool support.** No EDA tool surveyed first-class-supports
chemSHERPA. Same workaround pattern as IEC 62474.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Reference-only` disposition initially; promote to
`Planned` if a Japanese-market customer drives the requirement.
The substance content is largely covered by IPC-1752A + the
existing `PartCompliance` fields.

**Strategic recommendation.** **Reference-only** initially; do
not implement chemSHERPA-specific parsing speculatively. The
`Part.compliance.*` substrate plus IPC-1752A export is
sufficient for an external chemSHERPA conversion. Promote to
`Planned` if a Japanese-market customer demands native support.

### End-of-Life Directives

#### EU WEEE (Directive 2012/19/EU)

**Full title.** **Directive 2012/19/EU** — *Waste Electrical
and Electronic Equipment (WEEE) (recast)*. Replaced original
2002/96/EC.

**Scope.** Producer-responsibility regime for end-of-life
electrical and electronic equipment in the EU. Producers must:
- Register in the national WEEE register of each Member State
  where they sell EEE.
- Pay recycling fees (typically per-tonne of EEE placed on
  market).
- Provide information on disposal / recycling to consumers.
- Mark products with the "wheelie bin with a cross" pictogram
  (per EN 50419:2022).

**Adoption status (2026).** **Mainstream-mandatory** for EEE
sold in the EU. Some Member States (Germany, France, Netherlands,
Sweden) have additional national requirements layered on top.

**License / IP.** **Free** at `eur-lex.europa.eu`.

**EDA tool support.** No EDA tool surveyed first-class-supports
WEEE registration metadata. Pattern: organisational responsibility,
not tool responsibility.

**Datum coverage status.** **Reference-only.** Producer
registration is regulatory-affairs work; Datum's only relevance
is allowing the project to record a "WEEE registered" flag for
reference / title-block annotation.

**Strategic recommendation.** **Reference-only**; if needed,
add a single optional field
`Project.compliance.materials_posture.weee_registered: bool`
(or per-Member-State registration numbers as
`weee_registrations: HashMap<String, String>` keyed on country
code). No validator. No export.

#### EU Battery Directive (2006/66/EC) and Battery Regulation (2023/1542)

**Full title.**
- **Directive 2006/66/EC** — *Batteries and accumulators and
  waste batteries and accumulators*. The historical baseline.
- **Regulation (EU) 2023/1542** — *Concerning batteries and
  waste batteries* (in force February 2024; phased
  implementation through 2027). Replaces the directive with a
  much more comprehensive framework.

**Scope.** EU's battery-specific regulatory regime. Covers
substance restrictions (mercury, cadmium, lead in batteries),
collection and recycling targets, carbon-footprint declarations
(industrial and EV batteries), recycled-content mandates
(progressive 2031-2036), the upcoming **Battery Passport** (a
per-battery digital record analogous to ESPR's Digital Product
Passport), and supply-chain due diligence for battery raw
materials (lithium, cobalt, nickel, natural graphite).

**Adoption status (2026).** **Mainstream-mandatory** for any
product containing a battery sold in the EU. The 2023/1542
Regulation phases requirements through 2024-2027:
- **Carbon-footprint declarations** (industrial + EV batteries):
  August 2024 - February 2025.
- **Recycled-content declarations**: August 2025.
- **Battery Passport** (industrial + EV + LMT batteries): 18
  February 2027.

**Datum relevance.** **Mostly out of scope.** Datum projects
that include batteries are a small minority; for those that do,
the battery-specific substance restrictions are already captured
through the RoHS / REACH SVHC fields. The Battery Passport is
emerging EU regulation; Datum should monitor.

**License / IP.** **Free** at `eur-lex.europa.eu`.

**Datum coverage status.** **Reference-only.** No current
classification.

**Strategic recommendation.** **Reference-only.** Reserve
`Project.compliance.materials_posture.contains_battery: bool`
and `battery_passport_identifier: Option<String>` fields for
forward compatibility; do not implement Battery Regulation-
specific exporters without a customer driver.

#### EU ELV (Directive 2000/53/EC)

**Full title.** **Directive 2000/53/EC** — *End-of-Life
Vehicles*. In force September 2000; amended several times.

**Scope.** Vehicle-specific RoHS-equivalent. Restricts the same
heavy metals as RoHS (lead, mercury, cadmium, hexavalent
chromium) plus additional substances; with vehicle-specific
exemptions in Annex II (e.g., lead in solder for specific
applications). Predated EU RoHS by 3 years and influenced the
RoHS substance list.

**ELV-specific substances.** ELV substance scope is RoHS-
equivalent for the four heavy metals (Pb, Hg, Cd, Cr VI); ELV
does not include the brominated flame retardants (PBB / PBDE) or
the four phthalates that EU RoHS adds.

**Adoption status (2026).** **Mainstream-mandatory** for
vehicles and vehicle components in the EU. The EU is
considering a 2026 ELV revision (the "Vehicle End-of-Life
Regulation" proposal) that would update the substance list and
add recycled-content mandates.

**Datum relevance.** **Domain 4 vertical-driven.** When
`Project.compliance.industry_vertical: Automotive`, ELV
applies. The substance content is captured through the existing
RoHS / SVHC fields. ELV-specific exemptions are recorded in the
free-form `Part.compliance.rohs_exemption` field with appropriate
ELV citation (e.g., "ELV Annex II 2.c.i").

**License / IP.** **Free** at `eur-lex.europa.eu`.

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5. **Promote to `Reference-
only`** once `PartCompliance` lands; no ELV-specific code work
beyond the vertical-driven exemption handling.

**Strategic recommendation.** **Reference-only**; the substance
data is captured through RoHS fields, the exemption catalog is
not maintained by Datum (per the audit's advisory exclusion on
RoHS-exemption tracking).

#### EU Eco-design Directive 2009/125/EC and ESPR (Regulation 2024/1781) — Digital Product Passport

**Full title.**
- **Directive 2009/125/EC** — *Eco-design Requirements for
  Energy-related Products (ErP)*. The historical baseline,
  primarily focused on energy-efficiency requirements.
- **Regulation (EU) 2024/1781** — *Ecodesign for Sustainable
  Products Regulation (ESPR)*. Adopted June 2024; effective July
  2024; phased implementation through 2030. Replaces and
  substantially expands the Eco-design Directive.

**Scope of ESPR.** Comprehensive sustainability framework for
products marketed in the EU:
- **Mandatory product requirements** on durability, repairability,
  recyclability, recycled content, energy efficiency, water
  efficiency, environmental footprint, and substance restrictions.
- **Digital Product Passport (DPP)** — a per-product digital
  identifier carrying material composition, repairability scores,
  recyclability metrics, and supplier-chain provenance.
- **Destruction of unsold consumer goods** prohibition.
- **Public procurement** requirements favouring ESPR-compliant
  products.

Phased implementation by product category through 2030:
- 2026: Initial product categories (steel, aluminium, textiles,
  furniture, tyres).
- 2027: Battery passport (under Regulation 2023/1542;
  coordinated with ESPR DPP framework).
- 2028-2030: Electronics expected (specific delegated acts
  TBD).

**Adoption status (2026).** **Emerging-mainstream.** Initial
product-category delegated acts published 2024-2025; electronics-
specific delegated acts expected 2027-2028.

**License / IP.** **Free** at `eur-lex.europa.eu`.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Reference-only` disposition; reserve forward-compatibility
fields in `Project.compliance.materials_posture`:
`dpp_required: bool` and `dpp_identifier: Option<String>`.

**Strategic recommendation.** **Reference-only / monitor.** The
electronics-specific DPP delegated act is not yet published.
Once published (expected 2027-2028), Datum should plan a
focused DPP-export feature. The substrate (per-Part
compliance metadata, BOM with material declarations,
deterministic transaction log for provenance) is in place.

**Cross-domain note.** The DPP overlaps with Domain 7 (PLM)
because it encodes supplier-chain provenance and lifecycle data;
the eventual DPP export is jointly Domain 5 + Domain 7
territory. Flag for the Domain 7 deep-dive.

### Conflict-Minerals Reporting

#### Dodd-Frank §1502 (US, 2010)

**Full title.** **Dodd-Frank Wall Street Reform and Consumer
Protection Act, Section 1502** (15 USC §78m note). SEC
implementing rule **17 CFR Parts 240 and 249b** (originally
2012; rule amended after the 2014 D.C. Circuit court decision
that struck the "conflict-free" labelling requirement).

**Issuing body.** **US SEC** (Securities and Exchange
Commission).

**Scope.** SEC filers (US-listed companies) must conduct
"reasonable country-of-origin inquiry" for **3TG** —
**tin (Sn), tungsten (W), tantalum (Ta), gold (Au)** —
sourced from the Democratic Republic of the Congo (DRC) or
neighbouring conflict region. Findings are reported on **SEC
Form SD** annually.

**Adoption status (2026).** **Mainstream-mandatory** for SEC
filers. Non-SEC-filer suppliers are pulled into compliance via
customer demands — every Tier-1 / Tier-2 supplier in the US
electronics supply chain produces conflict-minerals
declarations to support their customers' Form SD filings.

**License / IP.** **Free** at `sec.gov` (rule text at
`17 CFR 240` / `17 CFR 249b`).

**Reporting interchange format.** **CMRT** (Conflict Minerals
Reporting Template), maintained by the Responsible Minerals
Initiative (RMI; formerly EICC-GeSI Conflict-Free Sourcing
Initiative). De-facto industry standard. Excel-based with JSON
companion. Current version **CMRT 6.32** (published 2024;
updated annually).

**Datum coverage status.** `Deferred with prerequisite` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.5. **Promote to `Planned`**
once `PartCompliance` lands.

**Datum implementation cost.**
- Data model: `ConflictMineralsStatus` enum (`DRCConflictFree |
  ContainsConflictMinerals | UnableToDetermine | Unknown`) +
  `Part.compliance.conflict_minerals_status:
  Option<ConflictMineralsStatus>` field +
  `Part.compliance.cmrt_attestation: Option<DocumentRef>`
  (reference to vendor-supplied attestation document).
- Validator: project-level
  `requires_conflict_minerals_attestation: bool` triggers per-
  Part check.
- Export adapter: **CMRT exporter** — generates the CMRT 6.x
  Excel file from the per-Part data plus the project's BOM.
  Effort: ~1 week (template structure is well-documented at
  `responsiblemineralsinitiative.org`).
- MCP API: `set_part_conflict_minerals`, `get_part_conflict_
  minerals`, `export_cmrt_for_project`, `validate_cmrt_completeness`.

**Strategic recommendation.** **Implement now** as the canonical
conflict-minerals interchange. CMRT export is a real
differentiator — most EDA tools require an external compliance
platform to produce CMRT.

**Risks.** **CMRT template version drift.** RMI updates the
template annually; Datum's exporter must accommodate version
bumps. Mitigation: the exporter takes the target template
version as a parameter; ships with the latest version as
default.

#### EU Conflict Minerals Regulation (Regulation EU 2017/821)

**Full title.** **Regulation (EU) 2017/821** — *Laying down
supply chain due diligence obligations for Union importers of
tin, tantalum and tungsten, their ores, and gold originating
from conflict-affected and high-risk areas*. In force June 2017;
binding obligations effective **1 January 2021**.

**Issuing body.** **EU** (European Parliament and Council).

**Scope.** Applies to **EU importers** of 3TG (and 3TG-
containing intermediate products) above per-substance volume
thresholds. Importers must conduct OECD-aligned due diligence
on supply chain provenance. The regulation aligns with Dodd-
Frank §1502 framework but with EU-specific implementation.

**Adoption status (2026).** **Mainstream-mandatory** for EU
importers above thresholds. Reporting interchange is again
**CMRT** (and increasingly **EMRT** for cobalt-inclusive
supply chains).

**License / IP.** **Free** at `eur-lex.europa.eu`.

**Datum coverage status.** Same as Dodd-Frank §1502; the
substrate (per-Part `conflict_minerals_status` + project-level
`requires_conflict_minerals_attestation`) serves both regulations
identically. **No EU-specific Datum work needed.**

**Strategic recommendation.** **Implement alongside Dodd-Frank
§1502.** Single CMRT exporter serves both regulations.

#### CMRT (Conflict Minerals Reporting Template)

**Full title.** **Conflict Minerals Reporting Template (CMRT)**.
Maintained by the **Responsible Minerals Initiative (RMI)**
under the **Responsible Business Alliance (RBA)**.

**Current version (early 2026).** **CMRT 6.32** (October 2024
release). Previous: CMRT 6.22 (June 2023), CMRT 6.1 (2022).

**Format.** Excel workbook with structured tabs:
- Declaration tab (company-level statement)
- Smelter list tab (3TG smelters in the supply chain)
- Standard Smelter Names tab (RMI-controlled smelter ID list)
- Comments tab (free-text)
- Definitions tab (vocabulary)

**JSON companion format** is published alongside Excel; the
JSON is increasingly used for tool-to-tool exchange.

**License / IP.** **Free** at `responsiblemineralsinitiative.
org/reporting-templates/cmrt/`.

**Adoption status (2026).** **Mainstream-mandatory** in
electronics supply chains for both Dodd-Frank §1502 and EU
2017/821 compliance.

**Datum implementation cost.** ~1 week for the CMRT 6.x
Excel exporter (xlsxwriter or equivalent Rust crate; template
structure is bounded). ~3 days for the JSON companion exporter.

**Strategic recommendation.** **Implement now** as the canonical
conflict-minerals export. The CMRT template version is a
parameter; default to latest.

#### EMRT (Extended Minerals Reporting Template)

**Full title.** **Extended Minerals Reporting Template (EMRT)**.
Maintained by the **Responsible Minerals Initiative (RMI)** as
the cobalt-and-mica extension to CMRT.

**Current version (early 2026).** **EMRT 1.4** (2024).

**Scope.** Adds **cobalt (Co)** and **mica** beyond the 3TG of
CMRT. Driven by EV-battery supply-chain transparency demands
(cobalt is critical to lithium-ion batteries; mica is critical
to insulation in some battery and motor applications).
Increasingly demanded by EV OEMs (Tesla, BMW, VW, GM, Ford,
Stellantis) for battery and motor BOMs.

**Format.** Same structure as CMRT (Excel workbook + JSON
companion); EMRT is a parallel template, not a CMRT extension.
Suppliers may need to submit both CMRT (for 3TG) and EMRT (for
cobalt + mica) for the same product.

**License / IP.** **Free** at `responsiblemineralsinitiative.
org/reporting-templates/emrt/`.

**Adoption status (2026).** **Mainstream-emerging.** Standard
for EV-battery and EV-motor supply chains; less common in
non-battery electronics. Expected to expand as EU and US
regulations evolve.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Planned` disposition (paired with CMRT).

**Datum implementation cost.** ~3 days additional after CMRT
exporter lands; the structure is parallel.

**Strategic recommendation.** **Implement post-M7** alongside
CMRT. Trivial incremental cost; meaningful for EV-vertical
customers.

### Material Risk / Supply-Chain

#### Counterfeit-prevention (IDEA-1010, AS6081)

**Full title.**
- **IDEA-STD-1010-B (2023)** — *Acceptability of Electronic
  Components Distributed in the Open Market*. Maintained by
  IDEA (Independent Distributors of Electronics Association),
  partnered with RBA.
- **SAE AS6081 (2012; revised 2024)** — *Fraudulent / Counterfeit
  Electronic Parts: Avoidance, Detection, Mitigation, and
  Disposition — Distributors*. Aerospace-quality counterfeit-
  avoidance standard.

**Issuing body.** **IDEA** for IDEA-STD-1010; **SAE**
International for AS6081.

**Scope.** Counterfeit-component avoidance and detection
standards for distributors. Define inspection regimes (visual,
X-ray, decap, electrical) and chain-of-custody requirements.

**Datum relevance.** **Note only.** Datum is the design tool,
not a procurement tool. Datum can carry **provenance attestation
metadata** (which distributor a part was purchased from, lot
numbers, date codes, attestation document references) but cannot
enforce counterfeit-avoidance upstream.

**Datum coverage status.** **Out of scope** for direct
counterfeit-detection; **Reference-only** for provenance
metadata. Domain 7 (supply-chain integration) will own the
provenance fields if they get added.

**Strategic recommendation.** **Out of scope for v1.** The
substrate (`Part.manufacturer_jep106`, `Part.supply_chain_offers`,
proposed `Part.compliance.cmrt_attestation`) covers what Datum
needs. Active counterfeit-database integration (ECIA, ERAI, IHS
Markit) is post-M8 customer-driven work.

#### Tin-whisker risk (JEDEC JESD201, IPC-J-STD-006, IPC-A-610 Class 3 / 3A interaction)

**Full title.**
- **JEDEC JESD201A (2017)** — *Environmental Acceptance
  Requirements for Tin Whisker Susceptibility of Tin and Tin
  Alloy Surface Finishes*. Test methodology for tin-whisker
  susceptibility.
- **JEDEC JESD22A121 (2024)** — *Test Method for Measuring Whisker
  Growth on Tin and Tin Alloy Surface Finishes*. Specific test
  procedure.
- **IPC J-STD-006D (2024)** — *Requirements for Electronic Grade
  Solder Alloys and Fluxed and Non-Fluxed Solid Solders for
  Electronic Soldering Applications*. Solder-alloy specification
  including lead-free formulations.
- **IPC-A-610 H (2025)** — *Acceptability of Electronic
  Assemblies*. Class 3 and Class 3A inspection criteria are
  particularly strict on solder-joint integrity, including
  tin-whisker considerations.

**Adoption status (2026).** **Mainstream** in any product where
tin-whisker-induced shorts would be catastrophic (aerospace,
defence, medical Class 3, automotive ASIL B+). RoHS-driven
lead-free solder transition (2006) made tin-whiskers a
significant new failure mode for Class 3 / 3A applications.

**Datum relevance.** **Material-source-risk metadata.** A part
with pure-tin (Sn-100) finish is at higher tin-whisker risk than
a part with lead-bearing (SnPb) finish or a tin-bismuth (SnBi)
finish. For Class 3 / 3A applications, the finish is design-
critical metadata.

**Recommended Datum field.** `Part.compliance.lead_finish:
Option<LeadFinish>` enum (`PureTinSn100 | TinLeadSnPb | TinSilverSnAg
| TinSilverCopperSAC | TinBismuthSnBi | NickelPalladiumGoldNiPdAu |
GoldOverNickelGoldNi | Other(String) | Unknown`). When
`Project.compliance.ipc_class: Class3 | Class3A`, a validator
emits a "pure-tin lead finish on Part X may exhibit tin-whisker
risk in Class 3 application" diagnostic.

**Datum coverage status.** No current classification.
**Recommendation:** add to `STANDARDS_COMPLIANCE_SPEC.md` § 4.5
with `Planned` disposition (paired with the broader
`PartCompliance` block).

**Datum implementation cost.** ~1 day for the field +
diagnostic.

**Strategic recommendation.** **Implement now** as a low-cost,
high-value addition. Lead-finish risk is well-known to Class 3
designers; first-class support is a credibility signal.

**Risks.** **Vendor-data accuracy risk.** Vendor-declared lead
finish can be wrong (especially for distributor-secondary-market
parts). Mitigation: the field is authored data; the `last_compliance_check`
timestamp tracks freshness.

#### Rare-earth source restrictions (cross-ref Domain 4)

**Full title.** Various, including:
- **DFARS 252.225-7052** (US Defense Federal Acquisition
  Regulation Supplement) — *Restriction on the Acquisition of
  Certain Magnets and Tungsten*. Restricts certain Chinese-
  sourced rare-earth magnets and tungsten in DoD acquisitions.
- **DFARS 252.225-7016** — *Restriction on Acquisition of
  Ball and Roller Bearings*. Related supply-chain restriction.

**Adoption status (2026).** **Mainstream-mandatory** for US
defence contracts. Increasing scope through 2025-2026 as US
strategic-materials independence policy expands.

**Datum relevance.** **Cross-domain — Domain 4 owns the
project-level vertical declaration; Domain 5 owns the per-Part
material-source metadata.** When
`Project.compliance.industry_vertical: Defence`, parts containing
restricted-source materials should warn the user. The
substrate is `Part.compliance.material_origin: Vec<MaterialOrigin>`
where each `MaterialOrigin` carries country-of-origin and material
type.

**Datum coverage status.** No current classification.
**Recommendation:** **Out of scope for v1**; the country-of-
origin metadata is rarely available from distributor APIs and
the validation workflow is regulatory-affairs-tool work. Reserve
forward-compatibility fields if a defence customer surfaces.

**Strategic recommendation.** **Out of scope for v1.** The
substrate (Domain 4's `industry_vertical: Defence` flag plus
Octopart's `manufacturer_country` field where available) is
sufficient for an external compliance-tool integration.

### Note Only

#### Energy Star

**Full title.** **ENERGY STAR®** — joint US EPA / Department of
Energy energy-efficiency program for consumer products.

**Datum relevance.** **None directly.** ENERGY STAR is a
finished-product certification; Datum produces design data.

**Recommendation.** **Out of scope.**

#### EPEAT (Electronic Product Environmental Assessment Tool)

**Full title.** **EPEAT registry**, operated by the **Global
Electronics Council (GEC)**.

**Datum relevance.** **None directly.** EPEAT is a finished-
product registration; Datum produces design data. EPEAT criteria
overlap heavily with RoHS / REACH / WEEE / halogen-free / energy-
efficiency criteria already addressed elsewhere.

**Recommendation.** **Out of scope.**

#### GHS / SDS (Globally Harmonized System for Safety Data Sheets)

**Full title.** **GHS** — UN Globally Harmonized System of
Classification and Labelling of Chemicals (revised periodically;
GHS Rev 10 published 2023). **SDS** — Safety Data Sheet, the
GHS-mandated 16-section chemical handling document.

**Datum relevance.** **None directly.** GHS / SDS govern
chemical handling for occupational safety; not a board-design
surface.

**Recommendation.** **Out of scope.**

## Cross-Cutting Patterns

### Substance-list-as-data (Datum consumes, doesn't maintain)

**The single most important framing for Domain 5.** Datum cannot
maintain the regulatory substance lists themselves: the SVHC list
refreshes ~bi-annually; TSCA section 6(h) gains substances on EPA's
schedule; RoHS Annex II additions follow EU delegated-acts; IEC
62474 DSL refreshes quarterly; CMRT smelter list updates monthly.
Maintaining catalogs at this frequency is regulatory-data-vendor
work (Assent, Source Intelligence, iPoint, Sphera), not EDA-tool
work.

**Datum's pattern: consume substance-list data from upstream sources.**

**Three consumption paths:**

1. **IPC-1752A imports** — vendor-supplied per-Part declarations,
   imported into `Part.compliance.*` via a one-time XML parse.
   Vendors update IPC-1752A submissions when their supply chains
   change; Datum re-imports to refresh.

2. **Octopart / Nexar / Digi-Key / Mouser parametric lookups** —
   the `refresh_supply_chain` MCP tool (Domain 2 / Batch 1)
   populates `Part.compliance.*` fields as a side effect of
   distributor-catalog refresh. RoHS status, REACH SVHC, halogen-
   free, and lifecycle status are universally exposed by these
   APIs.

3. **Vendor compliance-cert PDFs** — manufacturer declarations of
   conformity, signed environmental compliance statements. These
   are typically **out of Datum's automated parsing scope** (PDF
   layouts vary widely); Datum stores the document reference in
   `Part.compliance.compliance_evidence: Vec<ComplianceEvidence>`
   and treats the PDF as evidence, not as parseable data. The AI
   surface can offer to OCR / extract from a PDF when explicitly
   requested, but this is opt-in rather than automatic.

**Datum's catalog responsibility: zero.** The user pins the
substance-list version (`svhc_list_pinned_date`,
`iec_62474_dsl_version`, `target_rohs_revision`); Datum carries
the pin and emits diagnostics when refresh is recommended.

### Per-Part compliance metadata (PartCompliance block)

The `Part` struct extension proposed by this report:

```rust
pub struct Part {
    // ... existing fields (Batch 1 + Domain 4 additions) ...
    pub compliance: Option<PartCompliance>,
}

pub struct PartCompliance {
    // EU RoHS — substance-restriction posture
    pub rohs_status: Option<RohsStatus>,
    pub rohs_basis: Option<String>,           // "RoHS 2 (2011/65 + 2015/863)"
    pub rohs_exemption: Option<String>,       // free-form, e.g., "Annex IV § 7(c)-I"

    // EU REACH — SVHC presence
    pub reach_svhc: Vec<SubstanceCandidate>,
    pub reach_svhc_list_date: Option<NaiveDate>,  // SVHC list date the declaration is against

    // China RoHS
    pub china_rohs_status: Option<ChinaRohsStatus>,
    pub china_rohs_substance_table: Option<ChinaRohsSubstanceTable>,

    // Japan J-MOSS
    pub j_moss_status: Option<JMossStatus>,

    // US TSCA section 6(h) PBT
    pub tsca_status: Option<TscaStatus>,
    pub tsca_pbt_substances: Vec<TscaPbtSubstance>,

    // Halogen-free
    pub halogen_free: Option<HalogenFreeStatus>,
    pub halogen_free_basis: Option<String>,   // "JS709C" / "IEC 61249-2-21"

    // Conflict minerals (3TG + cobalt/mica)
    pub conflict_minerals_status: Option<ConflictMineralsStatus>,
    pub cmrt_attestation: Option<DocumentRef>,

    // Lead finish (tin-whisker risk indicator)
    pub lead_finish: Option<LeadFinish>,

    // IEC 62474 substance-application metadata (auto-populated from
    // IEC 62474 DSL imports; opaque to Datum-internal logic).
    pub iec_62474_applications: Vec<Iec62474Application>,

    // Documentary evidence — PDFs, signed declarations, vendor compliance certs.
    pub compliance_evidence: Vec<ComplianceEvidence>,

    // Refresh tracking
    pub last_compliance_check: Option<DateTime<Utc>>,
}

pub struct SubstanceCandidate {
    pub cas_number: String,                   // CAS Registry Number
    pub ec_number: Option<String>,            // EC / EINECS number
    pub name: String,                         // canonical substance name
    pub concentration_pct: Option<f32>,       // weight percent (>0.1% triggers Article 33)
    pub list_revision_date: NaiveDate,        // SVHC list date this entry was added
    pub evidence_source: Option<String>,      // vendor declaration, IPC-1752A doc, Octopart
}

pub enum RohsStatus {
    Compliant,
    NonCompliant,
    CompliantWithExemption,
    Unknown,
    ComplianceClaimedWithoutEvidence,
}

pub enum ChinaRohsStatus {
    Compliant,
    NonCompliant,
    CompliantWithEpup10y,
    CompliantWithEpup20y,
    CompliantWithEpup30y,
    CompliantWithEpup40y,
    CompliantWithEpup50y,
    Unknown,
}

pub struct ChinaRohsSubstanceTable {
    pub sub_assemblies: Vec<String>,
    pub presence: HashMap<(String, String), SubstancePresence>,  // (substance, sub_assembly)
}

pub enum SubstancePresence {
    BelowLimit,                               // "○" in the standard table
    AboveLimit,                               // "×"
    NotApplicable,                            // empty
}

pub enum JMossStatus {
    GreenMark,                                // no restricted substances above limit
    OrangeMark,                               // contains substances above limit
    Unknown,
}

pub enum TscaStatus {
    Compliant,
    ContainsRestrictedSubstance,
    Unknown,
}

pub struct TscaPbtSubstance {
    pub cas_number: String,
    pub name: String,                         // PIP-3:1, DecaBDE, etc.
    pub concentration_pct: Option<f32>,
}

pub enum HalogenFreeStatus {
    Compliant,                                // <900 ppm Br, <900 ppm Cl, <1500 ppm total
    NotCompliant,
    Unknown,
    ComplianceClaimedWithoutEvidence,
}

pub enum ConflictMineralsStatus {
    DRCConflictFree,                          // no 3TG sourced from DRC conflict region
    ContainsConflictMinerals,
    UnableToDetermine,                        // due-diligence inconclusive
    Unknown,                                  // not investigated
}

pub enum LeadFinish {
    PureTinSn100,                             // tin-whisker risk for Class 3
    TinLeadSnPb,                              // RoHS-non-compliant baseline; lowest whisker risk
    TinSilverSnAg,
    TinSilverCopperSAC,                       // most common lead-free
    TinBismuthSnBi,
    NickelPalladiumGoldNiPdAu,
    GoldOverNickelGoldNi,
    Other(String),
    Unknown,
}

pub struct Iec62474Application {
    pub substance_id: String,                 // IEC 62474 DSL substance identifier
    pub application: String,                  // "flame retardant in cable insulation"
    pub material_classification: Option<String>,
    pub concentration_pct: Option<f32>,
}

pub struct ComplianceEvidence {
    pub kind: ComplianceEvidenceKind,
    pub document_ref: DocumentRef,            // pool-resolved or external URL
    pub issued_at: Option<NaiveDate>,
    pub valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

pub enum ComplianceEvidenceKind {
    IpcD1752Declaration,
    Iec62474Declaration,
    ChemSherpa,
    VendorRohsCertificate,
    VendorReachLetter,
    VendorHalogenFreeCertificate,
    Cmrt,
    Emrt,
    Other(String),
}

pub struct DocumentRef {
    pub uri: String,                          // pool path or external URL
    pub sha256: Option<String>,               // integrity hash if local
}
```

**Total Rust line count for the data model: ~350 lines plus
documentation.** Pure metadata; no algorithmic engine work
beyond serialisation and the lint validator.

### Project-level material posture

The `MaterialsPosture` sub-block proposed by this report sits
inside Domain 4's `ProjectCompliance`:

```rust
pub struct ProjectCompliance {
    // ... Domain 4 fields ...
    pub materials_posture: MaterialsPosture,
}

pub struct MaterialsPosture {
    // RoHS pinning
    pub target_rohs_revision: Option<RohsRevision>,
    // SVHC list pinning — when "REACH SVHC disclosed", as of which list date?
    pub svhc_list_pinned_date: Option<NaiveDate>,
    // IEC 62474 DSL pinning
    pub iec_62474_dsl_version: Option<String>,

    // Posture flags driving validation
    pub requires_halogen_free: bool,
    pub requires_reach_svhc_disclosure: bool,
    pub requires_china_rohs: bool,
    pub requires_jmoss: bool,
    pub requires_tsca_section_6h: bool,
    pub requires_iec_62474: bool,
    pub requires_conflict_minerals_attestation: bool,
    pub requires_emrt_cobalt: bool,           // EV / battery extension

    // Acknowledged exemptions — free-form list of cited exemption IDs that the
    // user has acknowledged as applicable to this project. Datum stores the IDs
    // verbatim and does NOT validate them against any catalog. Validation is
    // the user's regulatory team's responsibility.
    pub rohs_exemptions_acknowledged: Vec<String>,

    // Refresh staleness threshold — emit warnings if any Part.compliance
    // last_compliance_check is older than this many days.
    pub compliance_refresh_threshold_days: Option<u32>,

    // WEEE registration metadata (organisational, not validated)
    pub weee_registered: bool,
    pub weee_registrations: HashMap<String, String>,  // country code → registration number

    // Battery (EU 2023/1542) forward-compatibility
    pub contains_battery: bool,
    pub battery_passport_identifier: Option<String>,

    // ESPR Digital Product Passport forward-compatibility
    pub dpp_required: bool,
    pub dpp_identifier: Option<String>,
}

pub enum RohsRevision {
    Rohs2_2011,         // EU 2011/65/EU baseline 6 substances
    Rohs2_2015,         // + 2015/863 4 phthalates = 10 substances
    Rohs3_2019,         // (EU) 2017/2102 scope expansion; same 10 substances
}
```

### Vertical-driven exemption applicability (cross-ref Domain 4)

RoHS and REACH exemptions are vertical-specific:
- **RoHS Annex IV** — medical-devices and monitoring-and-control
  exemptions. When `Project.compliance.industry_vertical: Medical`
  (per Domain 4), Annex IV exemptions become applicable.
- **RoHS Annex III** — general industrial/automotive exemptions
  with sunset dates. When `industry_vertical: Automotive`,
  certain Annex III exemptions (especially solder applications)
  apply.
- **ELV Annex II** — vehicle-specific heavy-metal exemptions.
  When `industry_vertical: Automotive`, ELV exemption applicability
  is the primary substrate for lead-in-solder declarations.
- **REACH SVHC** — disclosure threshold (0.1% w/w in articles)
  applies universally regardless of vertical, but **enforcement
  intensity** varies (medical and aerospace customers expect
  fuller disclosures than consumer-electronics customers).

**Datum's pattern.** Datum does NOT enforce that a cited exemption
is appropriate for the project's vertical. The free-form
`Part.compliance.rohs_exemption: Option<String>` field stores
whatever the vendor declared. The
`Project.compliance.materials_posture.rohs_exemptions_acknowledged:
Vec<String>` field captures which exemption IDs the user has
explicitly acknowledged as applicable. The lint diagnostic emits:
"Part U7 cites RoHS exemption '7(c)-I' (lead in solder for medical
devices); project's `industry_vertical: Medical` and the cited
exemption is in Annex IV — verify with your regulatory team that
the exemption is currently in force." Datum surfaces the cross-
domain context (vertical + exemption); the human decides.

### Compliance-report export contract

Datum's recommended BOM-shaped compliance artifact exports:

| Format | Producer | Consumer | Datum effort | Schema source |
|---|---|---|---|---|
| **IPC-1752A Class A/B/C XML** | Datum-emitted | Vendor-supplier-customer chain; SCIP submission feed | ~3 days exporter (after Part.compliance lands) | `ipc.org` (free with registration) |
| **CMRT 6.x Excel + JSON** | Datum-emitted | Form SD filing (US); EU 2017/821 due-diligence record | ~1 week exporter | `responsiblemineralsinitiative.org` (free) |
| **EMRT 1.x Excel + JSON** | Datum-emitted | Cobalt/mica supply-chain disclosure (EV) | ~3 days exporter (after CMRT) | `responsiblemineralsinitiative.org` (free) |
| **IEC 62474 DSL XML** | Datum-emitted | Automotive Tier-N supply chain | ~5 days exporter | `iec.ch` (paywalled standard; free DSL database) |
| **BOM with compliance columns** | Datum-emitted | Internal review; PLM ingestion | ~1 day BOM-export extension | Datum-internal |
| **chemSHERPA-CI Excel/XML** | (reference-only initially) | Japanese supply chain | ~5 days if implemented | `chemsherpa.net` (free) |

**Common pattern.** All exports take the project UUID + the per-
Part compliance metadata as input and produce the format-specific
artifact. The exporter is **stateless** beyond the project state;
the deterministic transaction model means re-export from the same
project state produces byte-identical output. This is a Datum
differentiator for audit-trail purposes.

### Substance-list-version pinning

Already detailed in the executive summary. The pinning fields:
- `Project.compliance.materials_posture.target_rohs_revision`
- `Project.compliance.materials_posture.svhc_list_pinned_date`
- `Project.compliance.materials_posture.iec_62474_dsl_version`

Each pin is captured as authored data; pin updates are
authored transaction operations (`SetTargetRohsRevision`,
`PinSvhcListDate`, `SetIec62474DslVersion`), each carrying the
prior value in OpDiff for undo. The transaction log captures the
pinning history; Domain 8's audit-trail export will surface it
as an audit event.

### Compliance posture lint

Lint diagnostics emit through Domain 4's
`validate_project_compliance` MCP tool, with a Domain-5 sub-
validator. Examples (machine-readable shape):

```json
{
  "diagnostic_id": "MAT-001",
  "severity": "Warning",
  "message": "Project requires halogen-free; Part U3 lacks declaration.",
  "scope": { "part_uuid": "...", "part_mpn": "TPS54622" },
  "context": {
    "project_requires_halogen_free": true,
    "part_halogen_free_status": null
  },
  "remediation": "Refresh Part via refresh_supply_chain or import vendor IPC-1752A declaration.",
  "vertical_context": null
}

{
  "diagnostic_id": "MAT-002",
  "severity": "Warning",
  "message": "REACH SVHC declaration on Part C42 is older than the pinned SVHC list date.",
  "scope": { "part_uuid": "...", "part_mpn": "GRM188R71H104KA93D" },
  "context": {
    "svhc_pinned_date": "2026-01-15",
    "part_last_compliance_check": "2025-09-15"
  },
  "remediation": "Run refresh_supply_chain or re-import vendor declaration."
}

{
  "diagnostic_id": "MAT-003",
  "severity": "Info",
  "message": "RoHS exemption '7(c)-I' cited for Part U7; vertical is Medical so Annex IV is applicable. Datum does not validate exemption status.",
  "scope": { "part_uuid": "...", "part_mpn": "AD8051" },
  "vertical_context": "Medical"
}

{
  "diagnostic_id": "MAT-004",
  "severity": "Warning",
  "message": "Lead finish 'PureTinSn100' on Part R12 may exhibit tin-whisker growth in Class 3 application.",
  "scope": { "part_uuid": "...", "part_mpn": "RC0603FR-0710KL" },
  "context": {
    "ipc_class": "Class3A",
    "part_lead_finish": "PureTinSn100"
  }
}
```

The diagnostic shape is **structured (JSON)** so MCP tool callers
can surface them; the AI surface can render natural-language
summaries from the structured fields.

### Octopart / Nexar / Digi-Key / Mouser substance-data source

The Domain 2 / Batch 1 `refresh_supply_chain(part_uuid)` MCP tool
populates `Part.supply_chain_offers` from Octopart/Nexar/Digi-Key/
Mouser. **Domain 5's contract addendum**: the same refresh call
also populates `Part.compliance.*` fields when the source API
exposes them, in a single atomic operation. Field mapping (per
common API):

| Source field (Octopart / Nexar) | Datum target |
|---|---|
| `RoHS Status` (`Compliant` / `Non-Compliant`) | `Part.compliance.rohs_status` |
| `Lead Free` (`Yes` / `No`) | `Part.compliance.lead_finish` heuristic |
| `REACH SVHC` (substance list) | `Part.compliance.reach_svhc` (list cleared and re-populated) |
| `Halogen Free` (`Yes` / `No`) | `Part.compliance.halogen_free` |
| `Conflict Minerals Status` | `Part.compliance.conflict_minerals_status` |
| `Last Update` (timestamp) | `Part.compliance.last_compliance_check` |
| `Lifecycle Status` | `Part.lifecycle` (existing) |

The audit-trail event captured by the refresh operation includes:
- the source URL queried (Octopart / Nexar / Digi-Key / Mouser)
- the timestamp of the refresh
- the fields populated and their prior/new values (per OpDiff)
- the SVHC list date as reported by the source (if available)

This is **architectural coordination**, not new code: the
`refresh_supply_chain` tool's response shape grows additional
output fields; the Python MCP server's behaviour expands; the
Rust engine's compliance-block deserialiser handles the new
fields. Domain 7 (PLM) will own the consolidated refresh
contract definition.

## EDA Tool Support Matrix

| Standard / Feature | Altium | OrCAD-Capture | Cadence Allegro | PADS / Xpedition | KiCad 8 | Eagle / Fusion | Horizon | LibrePCB | DipTrace | EasyEDA Pro | Datum (current) | Datum (post-Domain-5) |
|----|----|----|----|----|----|----|----|----|----|----|----|----|
| **RoHS status as Part attribute** | Vault attribute | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Catalog filter | Not supported | `Part.compliance.rohs_status` `Planned` |
| **REACH SVHC presence** | Vault attribute (Assent integration) | Custom property | CIS-database attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Catalog filter | Not supported | `Part.compliance.reach_svhc` `Planned` |
| **Halogen-free status** | Vault attribute | Custom property | CIS attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Catalog filter | Not supported | `Part.compliance.halogen_free` `Planned` |
| **China RoHS substance table** | Vault attribute (Source Intelligence integration) | Custom property | CIS attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.compliance.china_rohs_status` `Planned` |
| **J-MOSS status** | Vault attribute | Custom property | CIS attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.compliance.j_moss_status` `Planned` |
| **TSCA section 6(h)** | Vault (via Assent) | Not supported | CIS attribute | Custom property | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Part.compliance.tsca_status` `Planned` |
| **Conflict-minerals status** | Vault attribute (Assent / Source Intelligence) | Not supported | CIS attribute | Custom property | Custom field | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Part.compliance.conflict_minerals_status` `Planned` |
| **IPC-1752A import** | Yes (Vault BOM import) | No | CIS-bridge import | No | No | No | No | No | No | No | No | `import_ipc_1752a_for_part` `Planned` |
| **IPC-1752A export** | Yes (Vault BOM export, paid) | No | Yes (CIS export) | Yes (Xpedition Compliance Manager, paid) | No | No | No | No | No | No | No | `export_ipc_1752a_for_project` `Planned` |
| **CMRT export** | Yes (Vault, via Assent integration) | No | Yes (paid PLM integration) | Yes (Xpedition, paid) | No (manual) | No | No | No | No | No | No | `export_cmrt_for_project` `Planned` |
| **EMRT export (cobalt/mica)** | Vault (Assent) | No | Paid PLM integration | Xpedition Compliance Manager | No | No | No | No | No | No | No | `export_emrt_for_project` `Planned (post-M7)` |
| **IEC 62474 DSL import/export** | Vault | No | Paid PLM integration | Xpedition Compliance Manager | No | No | No | No | No | No | No | `import/export_iec_62474` `Planned (post-M7)` |
| **chemSHERPA import/export** | Vault (Japan-region add-on) | No | Paid PLM integration | Xpedition Compliance Manager | No | No | No | No | No | No | No | `Reference-only` |
| **Project-level material posture** | Vault project type | Project property | Project property | Project property | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `Project.compliance.materials_posture` `Planned` |
| **Substance-list-version pinning** | Not first-class | Not supported | Not first-class | Not first-class | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | Not supported | `materials_posture.svhc_list_pinned_date` etc. `Planned` (Datum differentiator) |
| **Compliance posture lint** | Vault (configurable) | No | Constraint Manager (limited) | Xpedition (paid) | No | No | No | No | No | No | No | `validate_project_compliance` Domain-5 sub-validator `Planned` |
| **Lead-finish (tin-whisker risk)** | Vault attribute | Custom property | CIS attribute | Custom property | Custom field | Custom property | Custom property | Custom property | Custom property | Not supported | Not supported | `Part.compliance.lead_finish` `Planned` |

**Key:** "Vault" = Altium Vault subscription (paid; many Altium
features are gated behind Vault); "CIS" = Cadence Component
Information System (paid); "Custom property" = free-text user-
defined field; "Catalog filter" = the field is searchable in the
tool's parts catalog but not exposed in the project data model.

**Reading the matrix.** Same pattern as Domain 4:
- **Vault-class commercial tools** (Altium Vault + Assent, Cadence
  PLM-integrated CIS, Siemens Xpedition Compliance Manager) provide
  first-class compliance metadata, but only at significant additional
  subscription cost. Assent and Source Intelligence (commercial
  compliance-platform companies) are the de-facto data providers
  bolted onto the EDA tools.
- **Mid-tier tools** (PADS, OrCAD Capture standalone) rely on
  per-property workarounds.
- **Open-source tools** (KiCad, Eagle, Horizon, LibrePCB, DipTrace,
  EasyEDA) have **no first-class compliance support** in any
  shipped product.

**Datum's opportunity is to be first-class without the Vault
subscription requirement** — every materials-compliance metadata
field a paying Altium-Vault-plus-Assent customer takes for granted,
Datum exposes natively in the open-source engine, with the added
Datum differentiator of substance-list-version pinning and the
deterministic CMRT export. This is the single most credible
positioning win for Datum in the green-electronics / regulated-
industry market segment.

## Pending Exclusions (re-affirmed)

The audit's three Domain-5 advisory exclusions are confirmed.
**None has hidden cross-cutting value worth re-opening for deep-
dive.**

| Standard | Confirmed exclusion rationale | Recommended formal disposition (post-Domain-8) |
|---|---|---|
| **California Prop 65** | Chemical-warning labelling is a downstream consumer-product packaging artifact, not a board-design surface. The substance list overlaps heavily with REACH SVHC, TSCA section 6(h), and the RoHS list — substance presence relevant to Prop 65 is captured through the existing `Part.compliance.reach_svhc`, `tsca_status`, and `rohs_status` fields. Vendor-supplied Prop 65 warnings are typically free-form text not tractable as structured data. EDA-tool work is zero. | `Out of scope` (with substrate paragraph noting overlap with SVHC / TSCA / RoHS coverage) |
| **EU Packaging & Packaging Waste Directive (94/62/EC + amendments)** | Packaging is the fab-and-assembly concern (PCB shipped on which reel / in which tube / wrapped in which ESD bag), entirely downstream of EDA. The `Part.packaging_options: Vec<PackagingOption>` field (Batch 1) carries EIA-481 reel / tape / tray / tube / cut variants for procurement purposes, but this is supply-chain metadata, not packaging-directive compliance. The directive's substance restrictions on packaging (heavy metals in cardboard, recyclability of plastic films) are entirely outside Datum's scope. | `Out of scope` |
| **RoHS-exemption tracking** | Maintaining the RoHS Annex III / Annex IV exemption catalog with current sunset dates is regulatory-data-vendor work, not EDA-tool work. The catalog is updated by EU Commission delegated acts on rolling schedules; keeping it current requires regulatory-affairs subscription (Assent, Source Intelligence). Datum's `Part.compliance.rohs_exemption: Option<String>` field stores the cited exemption ID verbatim and explicitly does NOT validate it against any catalog; the `Project.compliance.materials_posture.rohs_exemptions_acknowledged: Vec<String>` field records user acknowledgement. **The validation that the cited exemption is in force is the user's regulatory team's responsibility.** | `Out of scope` (with substrate paragraph noting that exemption-ID storage IS in scope; only the catalog-maintenance is excluded) |

**No new Domain-5 exclusions discovered during deep-dive that
warrant flagging back to the audit.** The three above are correctly
identified as advisory exclusions and should be promoted to formal
`Out of scope` in the consolidated post-Domain-8 ratification pass.

**Note for the consolidated ratification pass:** Each of the three
exclusions has a clear "what Datum does instead" paragraph that
should be attached to the formal `Out of scope` disposition. Datum's
substance-list framing (consume, don't maintain) and the IPC-1752A-
based interchange path mean the practical capability surface is
preserved even with these formal exclusions.

## User Pain Points & Wishlist Items

Distilled from EEVblog, EDN, AspenCore Power Electronics News,
PCB West / IPC APEX EXPO conference talks, the IPC EDGE community,
the Reddit r/PrintedCircuitBoard and r/PCB threads, the
Compliance & Risks blog, Assent Compliance webinar Q&A, and the
European Electronics Industry Association (EUREL) sustainability
working group discussions:

1. **"Why doesn't my EDA tool know my BOM has REACH SVHC parts?"**
   The most common single complaint in the regulated-electronics
   community is that the EDA tool is unaware of substance presence
   in the BOM. Engineers manually check every line against vendor
   IPC-1752A declarations or against Octopart's RoHS field. **Datum
   opportunity:** `Part.compliance.reach_svhc` populated by
   `refresh_supply_chain` + AI-surface diagnostic when project
   requires SVHC disclosure. This is a real workflow improvement.

2. **"My halogen-free declaration uses parts that vendor-substituted
   to a non-halogen-free formulation last quarter."** Vendor
   substitution drift. Particularly painful for consumer-electronics
   OEMs (Apple, Samsung, HP) that have halogen-free product lines.
   **Datum opportunity:** the
   `compliance_refresh_threshold_days` field + lint diagnostic
   when `last_compliance_check` is older than threshold + halogen-
   free is a project requirement.

3. **"I need to produce a CMRT for our customer and our PLM tool
   doesn't support it."** Mid-market companies (50-500-person
   electronics firms) often lack a Vault / Source Intelligence
   subscription and end up producing CMRTs by hand in Excel.
   **Datum opportunity:** `export_cmrt_for_project` is a
   first-class differentiator. CMRT 6.x exporter with auto-
   populated 3TG smelter list from `Part.compliance.cmrt_attestation`
   evidence.

4. **"REACH SVHC list updated in January but my project's
   compliance status is from June 2025."** Pin rot. **Datum
   opportunity:** `svhc_list_pinned_date` field + lint diagnostic
   that fires when the pin is more than 6 months old (one ECHA
   update cycle).

5. **"My contract assembler asked for IPC-1752A declarations on
   every BOM line and I had to chase 47 vendors for PDFs that I
   then re-typed into a spreadsheet."** PDF workflow pain. **Datum
   opportunity:** the `compliance_evidence: Vec<ComplianceEvidence>`
   field stores the PDF reference; `import_ipc_1752a_for_part`
   parses the structured XML when vendors provide it; the AI
   surface can offer to extract declared values from a PDF when
   explicitly requested. Result: vendor PDFs become tracked
   evidence rather than re-typed data.

6. **"Our automotive customer requires IEC 62474 DSL XML and our
   EDA tool can only export BOM CSV."** IEC 62474-specific pain.
   **Datum opportunity:** `export_iec_62474_for_project` is a
   second-tier differentiator for automotive-vertical customers.

7. **"I switched from a SnPb part to a SAC part for RoHS but my
   Class 3 designer warned me about tin-whisker risk."** Cross-
   field-tradeoff pain. **Datum opportunity:** the `lead_finish`
   field + IPC-class-aware lint diagnostic. AI surface explains
   the trade-off ("the SnPb-to-SAC substitution improves RoHS
   posture but introduces tin-whisker risk in Class 3
   applications; consider a SnAg or SnBi alternative").

8. **"How do I know if my project complies with the new TSCA
   PIP-3:1 restriction?"** Substance-restriction discovery pain.
   **Datum opportunity:** the
   `Project.compliance.materials_posture.requires_tsca_section_6h`
   flag + diagnostic enumerating any BOM line containing a TSCA-
   restricted substance.

9. **"Our Japanese customer rejected our BOM because we didn't
   declare J-MOSS green-mark eligibility."** Regional-compliance
   pain for non-Japanese-market suppliers shipping into Japan.
   **Datum opportunity:** the `j_moss_status` field + lint
   diagnostic.

10. **"The vendor's IPC-1752A says 'RoHS compliant' but cites
    RoHS 1, and we're a medical-device company under RoHS 3."**
    Revision-mismatch pain. **Datum opportunity:** the
    `target_rohs_revision` pin + lint diagnostic when a vendor
    declaration's `rohs_basis` is older than the project's pin.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

These land as part of the Domain 5 spec edits in the next batch
(Standards Audit Batch 4, following Domain 4's Batch 3).

#### HR-1. `Part.compliance: Option<PartCompliance>` extension

**Standard / driver.** Cross-cutting — supports all of: EU RoHS
(via `rohs_status` + `rohs_exemption`), REACH SVHC (via `reach_svhc`),
China RoHS (via `china_rohs_status` + `china_rohs_substance_table`),
J-MOSS (via `j_moss_status`), TSCA section 6(h) (via `tsca_status`
+ `tsca_pbt_substances`), halogen-free (via `halogen_free` +
`halogen_free_basis`), conflict minerals (via
`conflict_minerals_status` + `cmrt_attestation`), tin-whisker risk
(via `lead_finish`), IEC 62474 (via `iec_62474_applications`),
documentary evidence (via `compliance_evidence`).

**Canonical IR changes.** Add `PartCompliance` struct,
`SubstanceCandidate`, `RohsStatus`, `ChinaRohsStatus`,
`ChinaRohsSubstanceTable`, `SubstancePresence`, `JMossStatus`,
`TscaStatus`, `TscaPbtSubstance`, `HalogenFreeStatus`,
`ConflictMineralsStatus`, `LeadFinish`, `Iec62474Application`,
`ComplianceEvidence`, `ComplianceEvidenceKind`, `DocumentRef`
types to `ENGINE_SPEC.md` § 1.1a (Shared Enums). Add
`compliance: Option<PartCompliance>` field to `Part` (§ 1.2).

**Pool / library changes.** Pool index `parts` SQL table gains
columns: `rohs_status`, `halogen_free`, `conflict_minerals_status`,
`china_rohs_status`, `j_moss_status`, `tsca_status`,
`lead_finish`, `last_compliance_check` (all nullable). Query
interface allows filtering by compliance posture.

**Transaction model changes.** New operations:
- `SetPartCompliance(part_uuid, compliance)` — atomic
  replacement; carries prior compliance in OpDiff.
- `SetPartRohsStatus(part_uuid, status)` — focused.
- `SetPartReachSvhc(part_uuid, substances)` — focused.
- `SetPartHalogenFree(part_uuid, status)` — focused.
- `SetPartConflictMinerals(part_uuid, status)` — focused.
- `SetPartLeadFinish(part_uuid, finish)` — focused.
- `AttachComplianceEvidence(part_uuid, evidence)` — focused.

**MCP API additions.**
- `get_part_compliance(part_uuid)` — returns the full block.
- `set_part_compliance(part_uuid, compliance)` — atomic.
- Focused setters: `set_part_rohs_status`,
  `set_part_reach_svhc`, `set_part_halogen_free`,
  `set_part_china_rohs`, `set_part_j_moss`, `set_part_tsca`,
  `set_part_conflict_minerals`, `set_part_lead_finish`.
- `find_parts_by_compliance(filter)` — pool query.
- `attach_compliance_evidence(part_uuid, document_ref, kind)`.
- `validate_part_compliance(part_uuid, project_uuid)` — runs
  the per-Part compliance check against the project's posture.

**Minimum viable.** Struct + field + atomic get/set + filter
query + serialisation.

**Full implementation.** All focused operations + lint
diagnostics + Octopart/Nexar refresh integration (HR-3).

**Partner / library dependencies.** None for the data model.

**Effort estimate.** **3-4 days** for the data-model + MCP +
serialisation. ~350 lines of Rust.

#### HR-2. `Project.compliance.materials_posture: MaterialsPosture` sub-block

**Standard / driver.** Cross-cutting — supports project-wide
posture for: RoHS revision pinning, REACH SVHC list pinning,
IEC 62474 DSL pinning, halogen-free requirement, China RoHS
requirement, J-MOSS requirement, TSCA requirement, IEC 62474
requirement, conflict-minerals attestation requirement, EMRT
cobalt requirement, RoHS exemption acknowledgement, refresh
staleness threshold, WEEE registration, battery (forward-
compatibility), DPP (forward-compatibility).

**Canonical IR changes.** Add `MaterialsPosture` struct +
`RohsRevision` enum to `ENGINE_SPEC.md` § 1.1a. Add
`materials_posture: MaterialsPosture` field to
`ProjectCompliance` (per Domain 4's Batch 3 introduction).

**Native-format changes.** `project.json` schema's
`compliance.materials_posture` block carries the serialised
struct.

**Transaction model changes.** New operations:
- `SetTargetRohsRevision(project_uuid, revision)`.
- `PinSvhcListDate(project_uuid, date)`.
- `SetIec62474DslVersion(project_uuid, version)`.
- `SetMaterialsRequirement(project_uuid, requirement, value)`
  (covers all `requires_*` flags through one shape).
- `AcknowledgeRohsExemption(project_uuid, exemption_id)`.
- `SetComplianceRefreshThreshold(project_uuid, days)`.

**MCP API additions.**
- `get_materials_posture(project_uuid)`.
- `set_materials_posture(project_uuid, posture)`.
- `pin_svhc_list_date(project_uuid, date)`.
- `set_target_rohs_revision(project_uuid, revision)`.
- Focused setters per `requires_*` flag.
- `acknowledge_rohs_exemption(project_uuid, exemption_id)`.

**Minimum viable.** Struct + serialisation + atomic get/set.

**Full implementation.** All focused operations + lint
diagnostics consumption.

**Partner / library dependencies.** None.

**Effort estimate.** **1-2 days** added on top of Domain 4's
ProjectCompliance work.

#### HR-3. Octopart/Nexar refresh contract addendum (Domain 2 / 5 / 7 coordination)

**Standard / driver.** Coordinates with Domain 2's
`refresh_supply_chain` MCP tool (Batch 1) and Domain 7's PLM
integration. The `refresh_supply_chain` tool's response shape
grows to include `Part.compliance.*` field updates when the
source API exposes them.

**Architecture.** No new MCP tool; the existing
`refresh_supply_chain(part_uuid)` tool's contract grows. The
Python MCP server's HTTP-call handler maps source-API fields to
`Part.compliance.*` targets per the field-map table in
§ "Octopart / Nexar / Digi-Key / Mouser substance-data source"
above. The transaction-log entry captures the source URL,
timestamp, and the `OpDiff` for the compliance-block update.

**Domain handoff.**
- **Domain 2** (component modelling) — owns the
  `refresh_supply_chain` tool's existing contract for
  `behavioural_models` and `thermal` field updates.
- **Domain 5** (this domain) — owns the contract addendum for
  `compliance.*` field updates.
- **Domain 7** (PLM) — owns the consolidated field-map
  ratification across the three domains.

**MCP API changes.** No new tool; `refresh_supply_chain` output
shape extended:

```
Method: refresh_supply_chain
Input:  { "part_uuid": uuid, "source": "Octopart" | "DigiKey" | "Mouser" | "Nexar" | "Auto" }
Output: { "offers_count": int,
          "last_check": string,
          "compliance_fields_updated": [string],   // e.g., ["rohs_status", "reach_svhc"]
          "svhc_list_date_at_source": string | null,
          "warnings": [string] }
```

**Minimum viable.** Octopart / Nexar refresh populates
`rohs_status`, `halogen_free`, `reach_svhc`, `lifecycle`.

**Full implementation.** All four sources (Octopart, Nexar,
Digi-Key, Mouser); all field mappings; SVHC list date
preservation; audit-trail of every refresh.

**Partner / library dependencies.** Octopart / Nexar API key
(end-user supplies); Digi-Key API key (free); Mouser API key
(free).

**Effort estimate.** **2-3 days** for the contract extension
(Python MCP server + Rust deserialiser); **+1 day per source**
for full source-API coverage.

#### HR-4. IPC-1752A import + export

**Standard / driver.** IPC-1752A Rev B (canonical materials-
declaration interchange).

**Architecture.** Two MCP tools:
- `import_ipc_1752a_for_part(part_uuid, xml_path)` — parses
  vendor-supplied IPC-1752A XML and populates
  `Part.compliance.*` fields. Updates `last_compliance_check`
  to the import time.
- `export_ipc_1752a_for_project(project_uuid, output_path,
  declaration_class: "A" | "B" | "C")` — emits a project-wide
  IPC-1752A declaration document carrying the per-Part
  compliance data plus the project's `materials_posture`.
  Class A produces yes/no compliance assertions; Class B
  produces declarable-substance-group presence; Class C
  produces full composition (when per-Part data supports it).

**Implementation note.** IPC-1752A schema is documented and
bounded (XSD available). Parser implementation: ~1 week of
XML handling (Rust `quick-xml` or `serde-xml-rs`). Serialiser:
~3 days additional after parser lands. The schema reuse
between import and export reduces total effort.

**MCP API additions.**
- `import_ipc_1752a_for_part`.
- `export_ipc_1752a_for_project`.
- `validate_ipc_1752a_completeness(project_uuid,
  declaration_class)` — pre-export sanity check.

**Minimum viable.** Parser for Class A/B; exporter for Class B.

**Full implementation.** All three classes (A, B, C); validator;
schema-version-aware (IPC-1752 Rev B current; Rev C in
committee).

**Partner / library dependencies.** IPC-1752A schema (free with
IPC.org registration). Rust XML library (`quick-xml`).

**Effort estimate.** **1.5 weeks** (5 days parser + 3 days
exporter + 2 days validator + tests).

#### HR-5. CMRT export

**Standard / driver.** Dodd-Frank §1502, EU 2017/821. CMRT 6.x
template.

**Architecture.** One MCP tool:
- `export_cmrt_for_project(project_uuid, output_path,
  cmrt_version: "6.32" | "latest")` — emits the CMRT 6.x
  Excel workbook + JSON companion. Auto-populates the
  Declaration tab from `Project.compliance` metadata, the
  Smelter List tab from per-Part `cmrt_attestation` evidence,
  and the Standard Smelter Names tab from RMI's published
  smelter ID list (shipped with Datum as a versioned static
  resource).

**Implementation note.** CMRT template structure is documented
at `responsiblemineralsinitiative.org`; current format is
Excel 2010+ XML (.xlsx). Rust crate `xlsxwriter` or `rust_xlsxwriter`
generates the format. Total exporter: ~1 week.

**MCP API additions.**
- `export_cmrt_for_project`.
- `validate_cmrt_completeness(project_uuid)` — pre-export
  sanity check.
- `update_smelter_id_list(version)` — periodic update of the
  RMI smelter ID list shipped with Datum (this IS catalog
  maintenance — the only Datum-side catalog because the RMI
  smelter list is bounded and updated monthly with stable
  identifiers, not the bi-annual SVHC churn).

**Minimum viable.** Excel exporter for CMRT 6.x latest;
auto-populated Declaration tab; Smelter List tab from
attestation evidence.

**Full implementation.** Excel + JSON exporters; CMRT
version selectable; smelter ID list bundled and updateable;
EMRT exporter (HR-6) follows.

**Partner / library dependencies.** RMI CMRT template (free).
Rust Excel-writing crate.

**Effort estimate.** **1-1.5 weeks** for CMRT exporter (after
HR-1 lands).

### Should Support (post-M7)

#### SS-1. EMRT export (cobalt + mica)

**Standard / driver.** Extended Minerals Reporting Template;
EV-battery-supply-chain disclosure.

**Effort estimate.** **3-5 days** after CMRT exporter lands;
parallel structure.

#### SS-2. IEC 62474 DSL import + export

**Standard / driver.** IEC 62474; automotive Tier-N supply
chain.

**Effort estimate.** **1.5 weeks** (parser + serialiser +
DSL-version pinning).

#### SS-3. Compliance posture lint sub-validator

**Standard / driver.** Cross-cutting; emits diagnostics for
all the materials-posture fields.

**Effort estimate.** **3-4 days** for the lint rules + AI-
surface integration with structured-diagnostic shape.

#### SS-4. AI-surface compliance-aware part-substitution warnings

**Standard / driver.** Cross-cutting with Domain 4's similar
recommendation; consult both `Part.qualification` (Domain 4)
and `Part.compliance` (Domain 5) when suggesting substitutions.

**Behaviour.** When the AI agent suggests substituting a Part
into a project, it consults `Project.compliance.materials_posture`
and the substitution candidate's `Part.compliance`, and emits a
warning when:
- candidate `rohs_status: NonCompliant` for a project requiring
  RoHS compliance
- candidate has REACH SVHCs not present in original
- candidate `halogen_free: NotCompliant` for a halogen-free-
  required project
- candidate `lead_finish: PureTinSn100` for an IPC Class 3 / 3A
  project

**Effort estimate.** **2 days** for the warning rules (depends
on HR-1 landing + SS-3).

### On-Demand Only

#### OD-1. chemSHERPA import + export

If a Japanese-market customer surfaces, the chemSHERPA-CI/AI
import + export is **~1 week** of work. The substance content
overlaps substantially with IPC-1752A; the chemSHERPA-specific
delta is small.

#### OD-2. Per-vertical compliance-report exporters

If a customer surfaces with a specific format need (e.g., a
medical-device customer requiring FDA-aligned environmental
declaration formatting), build the per-vertical exporter on
demand. **Do not pre-build per-vertical exporters
speculatively.** The substrate (`Part.compliance.*` +
IPC-1752A export + CMRT export) is sufficient as raw input.

#### OD-3. Vendor compliance-cert PDF parsing / OCR

PDF parsing of vendor environmental compliance certificates
(RoHS letters, REACH letters, halogen-free certificates) is
**out of scope as automatic** — PDF layouts vary widely and
incorrect parsing risks compliance-data corruption. The AI
surface can offer to extract values from a PDF when
**explicitly requested by the user**, with the user reviewing
the extraction. This is opt-in customer-driven work, not
automatic.

#### OD-4. IMDS submission

IMDS submission is regulatory-affairs work coupled to a paid
commercial platform. **Out of scope for direct integration.**
Datum's IPC-1752A export + per-Part compliance metadata is
the substrate for a third-party IMDS-submission-tool
consumption.

#### OD-5. ESPR Digital Product Passport (DPP) export

The electronics-specific DPP delegated act is not yet published
(expected 2027-2028). **Out of scope until the delegated act
lands.** Forward-compatibility fields are reserved in
`materials_posture`.

### Out of Scope (recommend formal exclusion)

The full advisory-exclusion list re-affirmed under § "Pending
Exclusions (re-affirmed)". Plus:

- **California Prop 65** — re-affirmed advisory exclusion;
  promote to formal `Out of scope`.
- **EU Packaging Directive** — re-affirmed; promote to formal
  `Out of scope`.
- **RoHS-exemption catalog maintenance** — re-affirmed; promote
  to formal `Out of scope` (with the substrate paragraph
  noting that exemption-ID storage IS in scope).
- **IPC-1754** — niche; the data substrate via IPC-1752A is
  sufficient.
- **IPC-1755** — superseded in practice by RMI CMRT.
- **WEEE producer registration** — organisational, not tool;
  reserved field for reference only.
- **Battery Regulation 2023/1542 specific exporters** — only
  ~1% of PCBs include batteries; substance content captured
  through RoHS / SVHC; no Datum-specific exporter needed
  unless a customer drives.
- **Energy Star / EPEAT** — finished-product certifications;
  out of scope.
- **GHS / SDS** — chemical-handling, not EDA.

### Datum Differentiators

Where Datum's deterministic substrate + AI-native surfaces can
do better than incumbents:

1. **AI-explained compliance gaps.** When a lint diagnostic
   fires ("project requires halogen-free; Part U3 lacks
   declaration"), the AI surface explains the gap in natural
   language, suggests refresh actions ("run
   `refresh_supply_chain` for Part U3 to populate from
   Octopart"), and links to vendor data sources. No incumbent
   EDA tool has an AI-surface that explains compliance posture
   decisions; even Vault-tier tools surface the gap as a static
   error.

2. **MCP-queryable substance posture.** Every aspect of the
   compliance posture is queryable via MCP. An auditor's AI
   agent can ask "does this project declare a target RoHS
   revision, an SVHC pin date, a halogen-free requirement, and
   are all BOM lines compliant against the pinned posture?"
   and get a structured answer in milliseconds. Incumbent
   tools require SQL queries against the vault database or
   manual report generation.

3. **Deterministic CMRT export.** Datum's transaction model +
   JSON serialisation determinism mean a CMRT export from the
   same project state produces a byte-identical Excel file
   every time. This is much stronger than Vault-tier tools
   whose CMRT exports include timestamps and session IDs that
   change between runs. Audit reviewers can verify CMRT
   integrity by re-export-and-diff rather than trusting a
   vendor's audit-log claim.

4. **Substance-list-version pinning as authored data.** No
   other EDA tool surveyed treats substance-list-version
   pinning as first-class. Datum's
   `svhc_list_pinned_date`, `target_rohs_revision`, and
   `iec_62474_dsl_version` fields are persistent project state
   with transaction-log audit-trail. A reviewer can ask "when
   was this project last verified against the SVHC list, and
   which list version?" and get an authoritative answer.

5. **Refresh-staleness lint as first-class diagnostic.** The
   `compliance_refresh_threshold_days` field + lint diagnostic
   fires automatically when a Part's `last_compliance_check`
   is older than threshold. This is the "freshness" check that
   regulated-industry compliance teams currently perform
   manually with calendar reminders. Datum makes it a build-
   time diagnostic.

6. **Vertical-driven default suggestion.** When a user declares
   `Project.compliance.industry_vertical: Medical` (Domain 4)
   and `materials_posture` is empty, the AI surface can
   immediately suggest sensible defaults: `target_rohs_revision:
   Rohs3_2019`, `requires_halogen_free: true`,
   `requires_reach_svhc_disclosure: true`. This reduces
   materials-posture configuration from a 15-field manual form
   to a one-field declaration with reviewable defaults.

7. **AI-surface explained substance-list refresh.** When the
   user pins a new SVHC list date, the AI surface can explain
   what changed since the prior pin ("the SVHC list was
   updated on 2026-01-15 with 5 new substances; here are the
   ones that may apply to your BOM") and surface the diff for
   review. No incumbent EDA tool offers this kind of regulatory
   change explanation.

8. **Open-source, pool-native, no Vault subscription required.**
   Every materials-compliance metadata field a paying Altium-
   Vault-plus-Assent customer takes for granted, Datum exposes
   natively in the open-source engine. The CMRT export
   in particular is a meaningful cost-reduction for mid-market
   companies that currently produce CMRTs by hand or via
   Source Intelligence subscriptions.

### Recommended Spec Edits

Concrete file:line edits for the user to review. Pattern
follows Standards Audit Batch 1, Batch 2 (Domain 3 & Domain 2),
and Batch 3 (Domain 4 deep-dive).

Claude is in research-only mode per the project's
`feedback_research_only_mode` rule; these recommendations are
NOT to be applied by the agent. The user will review,
prioritise, and apply via the standard spec-edit process.

| # | Source | Target file | Substance |
|---|--------|-------------|-----------|
| **Pass 0 — `STANDARDS_COMPLIANCE_SPEC.md` disposition refresh** ||||
| D5-0a | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.5 | Domain 5 dispositions refreshed: RoHS 2 / RoHS 3 / REACH SVHC / IPC-1752A confirmed `Planned` (contract surface defined; `Part.compliance` extension in ENGINE_SPEC.md § 1.2; `Project.compliance.materials_posture` in § 1.x); China RoHS / J-MOSS / TSCA section 6(h) / halogen-free / conflict-minerals / lead-finish promoted from `Deferred with prerequisite` to `Planned` (the prerequisite — stable `Part.compliance` schema — is delivered by D5-1 below); IEC 62474 DSL confirmed `Planned` (post-M7 import/export); SCIP confirmed `Reference-only` (Datum produces IPC-1752A that feeds SCIP); WEEE / Battery Regulation / ELV / ESPR DPP confirmed `Reference-only` with substrate paragraph; chemSHERPA / IPC-1754 / IPC-1755 added with `Reference-only` / `Out of scope` dispositions; Korea RoHS / K-REACH added with `Reference-only`; advisory exclusions (California Prop 65, EU Packaging Directive, RoHS-exemption-catalog maintenance) confirmed for promotion to formal `Out of scope` in consolidated post-Domain-8 ratification pass with substrate-paragraph attached |
| D5-0b | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 7 (Project-Level Compliance Metadata) | Section expanded to enumerate the materials-posture fields (target_rohs_revision, svhc_list_pinned_date, iec_62474_dsl_version, requires_halogen_free, requires_reach_svhc_disclosure, requires_china_rohs, requires_jmoss, requires_tsca_section_6h, requires_iec_62474, requires_conflict_minerals_attestation, requires_emrt_cobalt, rohs_exemptions_acknowledged, compliance_refresh_threshold_days, weee_registered, weee_registrations, contains_battery, battery_passport_identifier, dpp_required, dpp_identifier); cross-references to ENGINE_SPEC.md § 1.1a additions; nests inside Domain 4's `ProjectCompliance` block |
| D5-0c | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` new § 7.2 | New subsection "Substance-List-as-Data Framing" — formal statement that Datum consumes substance-list data (IPC-1752A imports, Octopart/Nexar refresh, vendor PDFs as evidence) but does NOT maintain regulatory catalogs (SVHC list, RoHS exemptions, IEC 62474 DSL); the `compliance_refresh_threshold_days` field defines the freshness expectation; pinning fields (target_rohs_revision, svhc_list_pinned_date, iec_62474_dsl_version) capture the version-of-record at declaration time |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** ||||
| D5-1 | this report | `specs/ENGINE_SPEC.md` § 1.1a (Shared Enums) | New shared types: `RohsStatus`, `ChinaRohsStatus`, `ChinaRohsSubstanceTable`, `SubstancePresence`, `JMossStatus`, `TscaStatus`, `TscaPbtSubstance`, `HalogenFreeStatus`, `ConflictMineralsStatus`, `LeadFinish`, `Iec62474Application`, `ComplianceEvidence`, `ComplianceEvidenceKind`, `DocumentRef`, `RohsRevision`, `SubstanceCandidate` |
| D5-2 | this report | `specs/ENGINE_SPEC.md` § 1.2 (Pool Types) | Extend `Part` with `compliance: Option<PartCompliance>`; new struct `PartCompliance` carrying the 14 fields enumerated in § "Per-Part compliance metadata" above |
| D5-3 | this report | `specs/ENGINE_SPEC.md` § 1.x (Project Type — added by Domain 4's Batch 3) | Extend `ProjectCompliance` (Domain 4) with `materials_posture: MaterialsPosture` field; new struct `MaterialsPosture` carrying the 19 fields enumerated above. Coordinated with Domain 4's `ProjectCompliance` block — must land after or with Domain 4's Batch 3. |
| D5-4 | this report | `specs/ENGINE_SPEC.md` § 3 (Operations) | New operations: `SetPartCompliance`, `SetPartRohsStatus`, `SetPartReachSvhc`, `SetPartHalogenFree`, `SetPartChinaRohs`, `SetPartJMoss`, `SetPartTsca`, `SetPartConflictMinerals`, `SetPartLeadFinish`, `AttachComplianceEvidence`, `SetTargetRohsRevision`, `PinSvhcListDate`, `SetIec62474DslVersion`, `SetMaterialsRequirement`, `AcknowledgeRohsExemption`, `SetComplianceRefreshThreshold` — each with `inverse()` for undo |
| **Pass 2 — pool & native persistence** ||||
| D5-5 | this report | `docs/POOL_ARCHITECTURE.md` § 2 | `parts` SQL index table gains columns: `rohs_status`, `halogen_free`, `conflict_minerals_status`, `china_rohs_status`, `j_moss_status`, `tsca_status`, `lead_finish`, `last_compliance_check` (all nullable); query API extended with `find_parts_by_compliance` |
| D5-6 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.x (parts persistence) | Per-Part native persistence gains `"compliance": { ... }` block carrying serialised `PartCompliance`; optional |
| D5-7 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.1 (project.json) | `project.json` `compliance` block (Domain 4) gains `"materials_posture": { ... }` sub-block; required-present with default values for new projects; existing projects deserialise with defaults (all `requires_*` flags `false`; pinning fields `None`) |
| **Pass 3 — `specs/MCP_API_SPEC.md` (Materials Compliance Tools section)** ||||
| D5-8 | this report | `specs/MCP_API_SPEC.md` new "Materials Compliance Tools" section | Section header + per-tool stubs for `get_part_compliance`, `set_part_compliance`, `set_part_rohs_status`, `set_part_reach_svhc`, `set_part_halogen_free`, `set_part_china_rohs`, `set_part_j_moss`, `set_part_tsca`, `set_part_conflict_minerals`, `set_part_lead_finish`, `attach_compliance_evidence`, `find_parts_by_compliance`, `validate_part_compliance`, `get_materials_posture`, `set_materials_posture`, `pin_svhc_list_date`, `set_target_rohs_revision`, `acknowledge_rohs_exemption`, `import_ipc_1752a_for_part`, `export_ipc_1752a_for_project`, `validate_ipc_1752a_completeness`, `export_cmrt_for_project`, `validate_cmrt_completeness`, `update_smelter_id_list`, `import_iec_62474_for_part` (post-M7), `export_iec_62474_for_project` (post-M7), `export_emrt_for_project` (post-M7). ~25 new tool stubs. |
| D5-9 | this report | `specs/MCP_API_SPEC.md` § Supply Chain (Component Modelling Tools section) | Existing `refresh_supply_chain` tool's response shape extended to include `compliance_fields_updated: [string]` and `svhc_list_date_at_source: string | null`; documents the contract addendum that the same refresh call populates `Part.compliance.*` when source API exposes the data; cross-references to D5-8 |
| D5-10 | this report | `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy | Cross-reference added: vendor compliance-cert PDFs and IPC-1752A declarations imported as `compliance_evidence` are subject to the encrypted-content gate when distributed under encrypted vendor schemes (rare but non-zero — some defence vendors distribute encrypted RoHS letters) |
| **Pass 4 — `specs/IMPORT_SPEC.md`** ||||
| D5-11 | this report | `specs/IMPORT_SPEC.md` new § (IPC-1752A import) | Specifies IPC-1752A Rev B XML import as a first-class import format; per-Part-keyed (vendor declarations are typically per-MPN); declaration class A/B/C handling; cited substance-list pin date; mapping table from IPC-1752A fields to `Part.compliance.*` |
| D5-12 | this report | `specs/IMPORT_SPEC.md` § 3 (KiCad) | KiCad has no compliance fields; imported KiCad projects deserialise with empty `Part.compliance` and empty `Project.compliance.materials_posture`. User authority preserved — imported projects do not gain compliance metadata silently |
| **Pass 5 — architecture & guidance docs** ||||
| D5-13 | this report | `docs/STANDARDS_AUDIT_BATCH_4_GUIDANCE.md` (NEW) | Batch-4 bridging doc following the Batch-1 / Batch-2 / Batch-3 pattern (must-land vs deferred, apply order, dependence on Domain 4's Batch 3 `ProjectCompliance` landing first, Pass 0 disposition refresh, Cross-Spec Update Rule compliance, advisory-exclusion ratification deferral) |
| D5-14 | this report | `docs/INTEROP_SCOPE.md` | Add "Materials & Environmental Compliance (research-staged)" section: `Part.compliance` substrate; IPC-1752A import/export; CMRT export; EMRT export (post-M7); IEC 62474 DSL (post-M7); chemSHERPA (on-demand); substance-list-as-data framing |
| D5-15 | this report | `docs/LIBRARY_ARCHITECTURE.md` after the Domain-4 "Compliance metadata in Part records" subsection | New "Materials compliance metadata in Part records" subsection; cross-references D5-2 and D5-5; documents the `Part.compliance` field's intended use; notes the consume-don't-maintain framing |
| **Pass 6 — exporters (delivery sequence)** ||||
| D5-16 | this report | `docs/EXPORT_ROADMAP.md` (NEW) | Documents the recommended delivery sequence: HR-1/HR-2 (Part.compliance + materials_posture) first; HR-3 (refresh contract addendum) second; HR-4 (IPC-1752A import + export) third; HR-5 (CMRT export) fourth; SS-2 (IEC 62474) post-M7; SS-1 (EMRT) post-M7. Documents the dependency that HR-3 needs Domain 7's PLM deep-dive to ratify the consolidated supply-chain refresh field map |

**Total recommended spec edits:** **18** (3 disposition
refreshes, 4 schema bedrock, 3 pool/native persistence, 3 MCP,
2 import, 4 architecture/guidance/exporter-roadmap docs).

**Comparable to Batch 3's count (16 edits)** — Domain 5 is
slightly heavier because of the four export-format MCP tools
(IPC-1752A import + export, CMRT export, plus stubs for
post-M7 IEC 62474 / EMRT). The user can split into multiple
PRs if the batch is judged too large for a single review pass;
suggested split is **"Pass 0 + Pass 1 + Pass 2"** as Batch
4.0 (the schema and disposition work) and **"Pass 3 + Pass 4
+ Pass 5 + Pass 6"** as Batch 4.1 (the MCP, import, guidance,
and roadmap docs).

**Dependency note.** D5-3 (extending `ProjectCompliance`) requires
Domain 4's Batch 3 to have landed first, since Domain 4 introduces
the `ProjectCompliance` struct itself. If Batch 3 has not landed
when Batch 4 is sequenced, the Domain-5 work either waits on
Batch 3 or carries the `ProjectCompliance` introduction itself
(in which case Batch 3 will need to coordinate when it lands).

## Cross-Domain Insights to Thread Forward

### To Domain 6 (EMC & signal integrity)

- **Halogen-free laminates have different Dk/Df than standard
  FR-4.** Halogen-free laminates typically have Dk 4.0-4.4 vs
  4.4-4.8 for standard FR-4 at 1 GHz; Df typically 0.010-0.015
  vs 0.018-0.025. The `Stackup.dielectric_constant` and
  `loss_tangent` fields (Batch 1 in `ENGINE_SPEC.md` § 1.3)
  capture this for impedance calculation. Domain 6 should be
  aware that a project flagged
  `Project.compliance.materials_posture.requires_halogen_free:
  true` likely uses a different laminate family; Domain 6's
  controlled-impedance work should source Dk/Df from
  Stackup-material data rather than from a default-FR-4 table.

- **Stackup material name carries IEC 61249 reference.** The
  `Stackup.material_name` free-text field is the natural
  carrier for IEC 61249 substrate designations ("FR-4 per IEC
  61249-2-7"); Domain 6 should consume this for material-
  specific impedance modelling.

- **Lead-free SAC solder has different acoustic properties
  than SnPb solder.** For acoustic-class designs (some medical
  imaging boards), the lead-free transition affects ultrasonic
  signal propagation. This is post-M8 work but flagging for
  awareness.

### To Domain 7 (PLM & lifecycle integration)

- **Octopart / Nexar / Digi-Key / Mouser external-lookup
  contract is jointly owned by Domains 2, 5, and 7.** A single
  `refresh_supply_chain(part_uuid)` call should populate
  Domain 2's `behavioural_models`/`thermal` (when available),
  Domain 5's `compliance.*`, and Domain 7's
  `supply_chain_offers`/`lifecycle`/`last_supply_chain_check`.
  **Domain 7's deep-dive should ratify the consolidated field
  map** — Datum needs one source-of-truth table for "what
  fields a single refresh populates from each source API". The
  three Phase-2 reports each touched part of this; the
  consolidation belongs in Domain 7.

- **Vendor compliance-cert PDFs are evidence; PLM is the
  storage system.** The
  `Part.compliance.compliance_evidence: Vec<ComplianceEvidence>`
  field carries `DocumentRef` pointers to vendor PDFs. The
  PDFs themselves typically live in a PLM document repository
  (Windchill, Teamcenter, Aras, Arena) or in a corporate
  document-management system (SharePoint, Box, Google Drive).
  Domain 7's PLM integration should resolve `DocumentRef.uri`
  values that point into the PLM system.

- **Substance-list-version pinning is an audit event.** When
  the user pins a new SVHC list date (`pin_svhc_list_date`),
  the transaction is captured in the audit log. Domain 7's
  PLM integration should propagate the pin event to PLM-side
  audit systems if configured.

- **Compliance posture is a project-classification attribute.**
  PLM systems classify projects by compliance posture for
  approval-workflow routing (medical projects route through
  RA-team review; defence projects route through ITAR-marking
  review; consumer projects route through standard QA). Domain
  7's PLM integration should consume
  `Project.compliance.materials_posture` for routing.

- **Conflict-minerals attestation evidence is a supply-chain
  artifact.** The `Part.compliance.cmrt_attestation:
  Option<DocumentRef>` field's lifecycle (when supplied, when
  refreshed, when expired) is supply-chain-tool work that
  Domain 7 should manage.

### To Domain 8 (process & quality)

- **Compliance posture lint results emit through the audit
  trail.** Each lint diagnostic (MAT-001 through MAT-NNN)
  generated during `validate_project_compliance` is captured as
  an audit-trail event in Domain 8's exported audit log. The
  audit log records the diagnostic ID, severity, scope (Part
  UUID + MPN), context fields, and remediation note. Reviewers
  / auditors can re-run lint and diff against historical
  results.

- **Substance-list-version pinning is a transaction event.**
  Each pin update (`pin_svhc_list_date`,
  `set_target_rohs_revision`, `set_iec_62474_dsl_version`)
  carries the prior pin in OpDiff for undo; Domain 8's audit-
  trail export will surface the pinning history as a per-
  project regulatory-evidence chain.

- **Signed materials declarations require electronic-signature
  substrate.** When a regulated-industry project (medical /
  aerospace / defence) requires that a CMRT / IPC-1752A
  declaration be cryptographically signed by an authorised
  reviewer, the signature primitive lives in Domain 8. Domain
  5's exporters (HR-4 IPC-1752A, HR-5 CMRT) must be designed
  to consume the Domain-8 signature surface (when it lands)
  and embed the signature into the exported artifact (PDF
  signatures, signed XML, detached signature files).

- **The `compliance_evidence` field is itself audit-trail
  data.** When a `ComplianceEvidence` is attached to a Part
  (`attach_compliance_evidence`), the operation is captured in
  the transaction log; the exported audit log will surface
  evidence-attachment history as a per-Part compliance-data
  chain.

- **CMRT exporter output is reproducible audit evidence.**
  Datum's deterministic transaction model means CMRT exports
  from byte-identical project state produce byte-identical
  Excel files. This is Domain-8 audit-trail-relevant: an
  auditor can verify CMRT integrity by re-export-and-diff.

### Cross-domain coordination summary

| Concern | Owner | Cross-cuts |
|---|---|---|
| `Part.compliance` block | Domain 5 | Domain 2 (`refresh_supply_chain` populates), Domain 7 (PLM stores evidence), Domain 8 (transactions audit-logged) |
| `Project.compliance.materials_posture` | Domain 5 | nested inside Domain 4's `ProjectCompliance` |
| `IntendedEnvironment` enum | Domain 4 (introduced); Domain 5 (consumes for derate considerations) | Domain 6 (EMC class), Domain 7 (PLM filters) |
| `industry_vertical` enum | Domain 4 | Domain 5 (drives RoHS exemption applicability), Domain 6 (drives EMC class), Domain 7 (drives PLM routing) |
| Substance-list-version pinning | Domain 5 | Domain 8 (audit-trail event) |
| Octopart/Nexar refresh field map | **Domain 7 (consolidates)** | Domain 2 + Domain 5 + Domain 7 (each owns their fields) |
| CMRT / IPC-1752A / IEC 62474 exporters | Domain 5 | Domain 1 (export-adapter framework), Domain 8 (signature surface when needed) |
| Compliance posture lint diagnostics | Domain 5 | Domain 4 (`validate_project_compliance` MCP tool surface), Domain 8 (audit-trail) |
| Encrypted vendor PDF handling | Domain 2 (Encrypted Content Handling Policy) | Domain 5 (compliance evidence subject to gate when distributed encrypted) |

## Sources

### Primary regulatory and standards references

- [Directive 2011/65/EU (RoHS 2)](https://eur-lex.europa.eu/eli/dir/2011/65/oj) — *Restriction of the use of certain Hazardous Substances in electrical and electronic equipment*. EU Official Journal; **free**.
- [Commission Delegated Directive (EU) 2015/863](https://eur-lex.europa.eu/eli/dir_del/2015/863/oj) — Adds DEHP, BBP, DBP, DIBP to RoHS Annex II. **Free**.
- [Directive (EU) 2017/2102 (RoHS 3)](https://eur-lex.europa.eu/eli/dir/2017/2102/oj) — Removes 22 July 2019 medical-devices exemption expiration; in-vitro diagnostics scope. **Free**.
- [Regulation (EC) No 1907/2006 (REACH)](https://eur-lex.europa.eu/eli/reg/2006/1907/oj) — *Registration, Evaluation, Authorisation and Restriction of Chemicals*. **Free**.
- [ECHA SVHC Candidate List](https://echa.europa.eu/candidate-list-table) — Substances of Very High Concern; ~bi-annual refresh; CSV/XLSX/XML downloads. **Free**.
- [ECHA SCIP Database](https://echa.europa.eu/scip-database) — Substances of Concern In articles; submission obligation since 5 January 2021. **Free**.
- [Directive 2008/98/EC (Waste Framework Directive)](https://eur-lex.europa.eu/eli/dir/2008/98/oj) — Establishes SCIP under Article 9(1)(i). **Free**.
- [Directive 2012/19/EU (WEEE recast)](https://eur-lex.europa.eu/eli/dir/2012/19/oj) — *Waste Electrical and Electronic Equipment*. **Free**.
- [Regulation (EU) 2023/1542 (Battery Regulation)](https://eur-lex.europa.eu/eli/reg/2023/1542/oj) — *Concerning batteries and waste batteries*. **Free**.
- [Directive 2000/53/EC (ELV)](https://eur-lex.europa.eu/eli/dir/2000/53/oj) — *End-of-Life Vehicles*. **Free**.
- [Directive 2009/125/EC (Eco-design)](https://eur-lex.europa.eu/eli/dir/2009/125/oj) — *Eco-design Requirements for Energy-related Products*. **Free**.
- [Regulation (EU) 2024/1781 (ESPR)](https://eur-lex.europa.eu/eli/reg/2024/1781/oj) — *Ecodesign for Sustainable Products Regulation*; introduces Digital Product Passport. **Free**.
- [Regulation (EU) 2017/821 (EU Conflict Minerals)](https://eur-lex.europa.eu/eli/reg/2017/821/oj) — Supply chain due diligence for 3TG. **Free**.
- [40 CFR Part 751 (TSCA section 6)](https://www.ecfr.gov/current/title-40/chapter-I/subchapter-R/part-751) — *Significant New Use Rules for Persistent, Bioaccumulative, and Toxic Chemicals*. US EPA; **free**.
- [Dodd-Frank §1502 SEC Implementing Rule](https://www.sec.gov/divisions/corpfin/cfconflictminerals.htm) — US conflict-minerals reporting. **Free**.
- [GB/T 26572-2011 (China RoHS limit values)](http://openstd.samr.gov.cn/) — Concentration limits for restricted substances. Mandarin; **free**.
- [SJ/T 11364-2014 (China RoHS marking)](http://www.miit.gov.cn/) — Marking requirements. Mandarin; **free**.
- [JIS C 0950:2021 (J-MOSS)](https://webdesk.jsa.or.jp/) — Marking for restricted-substance presence. **Paywalled** through JSA (~JPY 5000-10,000).
- [Korean Act on Resource Circulation](https://www.law.go.kr/) — Korea RoHS-equivalent. Korean; **free**.
- [Korean K-REACH Act](https://www.law.go.kr/) — Korea REACH-equivalent. Korean; **free**.

### IPC standards and templates

- [IPC-1752A Rev B (2020)](https://www.ipc.org/TOC/IPC-1752A.pdf) — *Materials Declaration Management*. IPC; **free with registration**.
- [IPC-1754](https://www.ipc.org/) — *Materials and Substances Declaration for Aerospace and Defense*. IPC; **paywalled** (~USD 100).
- [IPC-1755](https://www.ipc.org/) — *Conflict Minerals Data Exchange*. IPC; **paywalled** (~USD 100).
- [JEDEC JS709C (March 2024)](https://www.jedec.org/standards-documents/docs/jesd-709c) — *Definition of "Halogen-Free"*. JEDEC; **free with registration**.
- [JEDEC JESD201A (2017)](https://www.jedec.org/standards-documents/docs/jesd201a) — *Environmental Acceptance Requirements for Tin Whisker Susceptibility*. JEDEC; **free with registration**.
- [JEDEC JESD22A121 (2024)](https://www.jedec.org/standards-documents/docs/jesd22-a121) — *Test Method for Measuring Whisker Growth*. JEDEC; **free with registration**.
- [IPC J-STD-006D (2024)](https://www.ipc.org/) — *Requirements for Electronic Grade Solder Alloys*. IPC; **paywalled**.
- [IPC-A-610 H (2025)](https://www.ipc.org/) — *Acceptability of Electronic Assemblies*. IPC; **paywalled**.
- [IPC-T-50 Rev N (2021)](https://www.ipc.org/) — *Terms and Definitions*. IPC; **free with registration**.

### IEC standards and database

- [IEC 62474 Database](https://std.iec.ch/iec62474) — Declarable Substance List (DSL); reference applications; material categories. **Free**.
- [IEC 62474:2018 standard text](https://webstore.iec.ch/publication/27406) — *Material declaration for products of and for the electrotechnical industry*. **Paywalled** at IEC Webstore (~CHF 200).
- [IEC 61249-2-21:2003](https://webstore.iec.ch/publication/4783) — *Reinforced base materials clad and unclad — Non-halogenated epoxide woven E-glass reinforced laminated sheets*. **Paywalled** (~CHF 280).
- [IEC 61249 series](https://webstore.iec.ch/searchform&q=IEC%2061249) — Multi-part substrate qualification series. **Paywalled** (per part).

### Industry interchange formats

- [RMI Conflict Minerals Reporting Template (CMRT 6.32)](https://www.responsiblemineralsinitiative.org/reporting-templates/cmrt/) — Excel template + JSON companion + Standard Smelter Names list. RBA / RMI; **free**.
- [RMI Extended Minerals Reporting Template (EMRT 1.4)](https://www.responsiblemineralsinitiative.org/reporting-templates/emrt/) — Cobalt + mica extension. RBA / RMI; **free**.
- [chemSHERPA Specification](https://chemsherpa.net/english) — JAMP; chemSHERPA-CI and chemSHERPA-AI schemas; free authoring tool. JAMP; **free**.
- [IMDS (International Material Data System)](https://www.mdsystem.com/imdsnt/startpage/index.jsp) — HP/DXC operated; commercial-platform-only. **Paid platform**.

### Compliance-platform vendor documentation

- [Assent Compliance](https://www.assentcompliance.com/) — Commercial supply-chain compliance platform; product compliance, ESG, due diligence. **Paid platform**; vendor documentation free.
- [Source Intelligence](https://www.sourceintelligence.com/) — Commercial compliance platform; conflict minerals, REACH SVHC, RoHS, ESG. **Paid platform**.
- [iPoint Compliance](https://www.ipoint-systems.com/) — Material compliance and circular economy. **Paid platform**.
- [Sphera Material Compliance](https://sphera.com/material-compliance/) — Commercial compliance platform. **Paid platform**.

### EDA tool documentation

- [Altium Vault Compliance Management](https://www.altium.com/altium-365/lifecycle-management) — Altium Vault sign-off workflow; relevant for compliance metadata. **Free** documentation.
- [Cadence Allegro / OrCAD CIS Compliance](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis.html) — Cadence PLM-integration documentation; compliance attribute support. **Free** marketing pages.
- [Siemens Xpedition Compliance Manager](https://eda.sw.siemens.com/en-US/pcb/xpedition/) — Xpedition compliance-data add-on. **Free** marketing pages.
- [KiCad Custom Fields Documentation](https://docs.kicad.org/master/en/eeschema/eeschema.html) — KiCad custom-property workaround for compliance metadata. **Free**.
- [Octopart / Nexar API](https://nexar.com/api) — GraphQL API; RoHS, REACH, halogen-free, lifecycle parametrics. **Free tier** (1000 queries/day).
- [Digi-Key API](https://developer.digikey.com/) — REST/JSON API; RoHS, lead-free metadata. **Free with registration**.
- [Mouser API](https://www.mouser.com/api-hub/) — REST/JSON API. **Free with registration**.

### Forum / industry discussion

- [r/PrintedCircuitBoard — REACH SVHC, RoHS discussions](https://www.reddit.com/r/PrintedCircuitBoard/) — Community discussion on materials compliance workflow. **Free**.
- [EEVblog forum — RoHS / REACH / halogen-free threads](https://www.eevblog.com/forum/) — Community discussion on substance-restriction practice. **Free**.
- [EDN - Compliance & Standards](https://www.edn.com/category/compliance-standards/) — Industry coverage of materials-compliance trends. **Free**.
- [IPC EDGE community](https://www.ipcedge.org/) — IPC training material and community discussion including IPC-1752A. **Free** registration.
- [Compliance & Risks blog](https://www.complianceandrisks.com/blog/) — Tracks SVHC list updates and global regulatory trends. **Free**.
- [Chemical Watch (Enhesa)](https://chemicalwatch.com/) — Industry news on chemical regulation; SVHC list updates, TSCA actions. **Paid subscription** (free tier limited).

### Cross-references (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 5 — Phase 1 inventory of Domain 5.
- `research/standards-audit/STANDARDS_AUDIT.md` § 5 advisory exclusions list — re-affirmed in this report.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-1752 (lines 744-764) — IPC-1752A material-declaration cross-reference.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-T-50 — vocabulary baseline cross-reference.
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Supply Chain — Octopart/Nexar/Digi-Key/Mouser supply-chain lookup contract.
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Encrypted Models — encrypted-content gate cross-reference for vendor compliance-cert PDFs.
- `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md` — Domain 4 cross-reference for `ProjectCompliance` block, `IndustryVertical` enum, and shared `IntendedEnvironment` enum.
- `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md` § Cross-Domain Insights → To Domain 5 — reciprocal cross-reference establishing the vertical-driven exemption applicability and the shared `IntendedEnvironment` enum.
- `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md` — Domain 3 cross-reference for project-level metadata patterns.
- `docs/CANONICAL_IR.md` — canonical IR (transaction model for new operations).
- `docs/POOL_ARCHITECTURE.md` — pool architecture (parts SQL index extension for compliance fields).
- `docs/LIBRARY_ARCHITECTURE.md` — library architecture (Part compliance metadata as library extension).
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md` — Batch-1 integration pattern (model for Batch-4 guidance doc).
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md` — meta-rules for research → spec integration.
- `specs/STANDARDS_COMPLIANCE_SPEC.md` — controlling standards spec (Pass 0 disposition refresh target).
- `specs/ENGINE_SPEC.md` — canonical types (Pass 1 schema bedrock target).
- `specs/NATIVE_FORMAT_SPEC.md` — on-disk persistence (Pass 2 update target).
- `specs/MCP_API_SPEC.md` — MCP API (Pass 3 update target; Materials Compliance Tools section + Supply Chain refresh contract addendum).
- `specs/IMPORT_SPEC.md` — import semantics (Pass 4 update target; IPC-1752A import).
- `CLAUDE.md` — project framing (no spec edit recommended for Domain 5; Domain 4's Batch-3 substrate-vs-certification principle covers the broader framing).
