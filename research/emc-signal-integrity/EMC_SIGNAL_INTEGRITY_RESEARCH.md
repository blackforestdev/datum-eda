# EMC & Signal Integrity — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 6 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 6`
> ("Per-Domain Audit → 6. EMC & signal integrity").
> Cross-references `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
> for IPC-2141 (controlled-impedance design) and IPC-2152
> (current-carrying capacity) — those standards are NOT re-researched
> here.
> Cross-references `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
> for the IBIS / Touchstone attachment contract; Domain 6 consumes the
> timing budget IBIS provides and the S-parameter data Touchstone
> provides. The `Part.behavioural_models` field, `ModelAttachment`
> shape, and Encrypted Content Handling Policy (Batch 1 of the
> standards audit) are the prerequisite contract surfaces; this report
> does NOT re-survey them.
> Cross-references `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md`
> for the `IndustryVertical` enum, the `Project.compliance` block
> shape, and the substrate-vs-certification framing — Domain 6 extends
> that framing to EMC.
> Cross-references `research/materials-environmental/MATERIALS_ENVIRONMENTAL_RESEARCH.md`
> for the Stackup-Dk/Df source-of-truth coordination — Domain 6's
> controlled-impedance work consumes the Batch-1 `dielectric_constant`
> / `loss_tangent` / `copper_weight_oz` / `roughness_um` /
> `material_name` fields rather than a default-FR-4 table.
> Companion to `research/airwire-rendering/`,
> `research/copper-rendering/`, `research/ipc-compliance/`, and prior
> Phase-2 reports for tone, structure, and source-citation style.
>
> Reads against the post-Standards-Audit-Batch-1 spec baseline merged
> 2026-04-18 (PR #1). The contract surfaces this report relies on
> (`Net.controlled_impedance: Option<ImpedanceSpec>`,
> `StackupLayer` material fields, `NetClass.diffpair_width` /
> `diffpair_gap`, `Part.behavioural_models`, the deferred `RuleType`
> comment "M5+: Impedance, LengthMatch, DiffpairGap, DiffpairSkew")
> all exist in `specs/ENGINE_SPEC.md` § 1.3 and § 4 as of that merge.
> Batch-3 (Domain 4) and Batch-4 (Domain 5) edits are in flight per
> the project owner's integration cadence; this report assumes they
> land before any Domain-6 implementation work.

> **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
>
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is available.
>
> Domain 6 carries one explicit fold-in advisory note (not a hard
> skip). The audit recommended:
>
> > **HDMI / DisplayPort / MIPI layout templates — fold into the
> > high-speed-rule deep-dive if it materialises; otherwise skip.**
>
> Per that note, this report treats HDMI 2.x, DisplayPort 1.4 / 2.x,
> and MIPI D-PHY / C-PHY / M-PHY as **fold-in candidates** under the
> generic "interface-specific rule template" mechanism, NOT as
> hard exclusions. The recommendation is to ship the underlying
> primitives (length-match group, `phy_profile`, return-path
> continuity) now and ship per-interface template packs as separate
> library content later — each pack a small JSON file declaring
> default rule values for that PHY family.
>
> Other Domain-6 advisory notes from the audit ("skip" priority on
> FCC Part 15, CISPR 22 / 32, EN 55032 / 55035, CISPR 25, DO-160 in
> the audit table) are NOT hard exclusions either — they are
> "low-priority" because Datum cannot certify them. They are surveyed
> here under the substrate-vs-certification framing as
> **`Reference-only`** product-cert standards rather than as
> implementation targets. None has hidden cross-cutting value that
> would change the recommendation; § "Pending Exclusions
> (re-affirmed)" provides the per-standard ratification rationale for
> the consolidated post-Domain-8 pass.

## Executive Summary

- **The central framing for Domain 6 is the same substrate-vs-
  certification line as Domain 4: Datum cannot certify FCC Part 15,
  CISPR 22 / 32, EN 55032, CISPR 25, IEC 61000 family, IEC 60601-1-2,
  or DO-160 EMC compliance. Datum can be the layout-rules substrate
  that compliance engineers depend on.** The certifying parties are
  accredited test labs (Element, UL Solutions, TÜV SÜD, Intertek,
  Eurofins MET, Bureau Veritas) executing the standardised emissions
  / immunity tests on a finished assembly. The Datum-side
  contributions are upstream: controlled-impedance audit, length-match
  audit, return-path-continuity audit, decoupling-cap-placement audit,
  PHY-profile-rule-pack assignment, IBIS / Touchstone artifact export
  for external SI / PI solver consumption. Of the ~40 standards
  inventoried in this report, **0 are "Datum certifies against"** and
  **~12 are "Datum produces input for the certification path"**. The
  honest positioning is "Datum is the SI/PI rules substrate; your EMC
  test lab is the certifying party".

- **Domain 6 is the single most prerequisite-heavy domain in the
  audit.** Three Batch-1 contract surfaces are load-bearing for any
  Domain-6 implementation work: `Net.controlled_impedance:
  Option<ImpedanceSpec>` (already in `ENGINE_SPEC.md` § 1.3 line 370),
  the `StackupLayer` material fields (`dielectric_constant`,
  `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`
  — already in § 1.3 lines 406-410), and `Part.behavioural_models:
  Vec<ModelAttachment>` (already in § 1.2 line 259). The deferred
  `RuleType` comment (`// M5+: Impedance, LengthMatch, DiffpairGap,
  DiffpairSkew` — § 4 line 748) is the only acknowledgement of the
  rule-type expansion. The Phase-1 audit correctly classified Domain 6
  as "Partial" because the scaffold exists (the four rule names are
  named); everything that hangs off the scaffold is still a blind
  spot. The good news is that Batch 1 already delivered the
  upstream-prerequisite work (Stackup material data and behavioural
  models attachment), so Domain 6 is **the natural next batch**.

- **The `NetClass` struct needs four small additions to support
  length-match and PHY-profile work; all are pure-metadata.** Today
  `NetClass` carries `clearance`, `track_width`, `via_drill`,
  `via_diameter`, `diffpair_width`, `diffpair_gap` (`ENGINE_SPEC.md`
  § 1.3 lines 382-391). The four Domain-6 additions are
  `length_match_target_nm: Option<i64>`,
  `length_match_tolerance_nm: Option<i64>`,
  `length_match_group: Option<Uuid>`, and
  `phy_profile: Option<PhyProfile>` where `PhyProfile` is an enum of
  named PHY identities (`DDR4_2400`, `DDR4_3200`, `DDR5_4800`,
  `LPDDR4_4266`, `PCIeGen3`, `PCIeGen4`, `PCIeGen5`, `USB2_HS`,
  `USB3_Gen1`, `USB3_Gen2`, `Ethernet1000BaseT`, `Ethernet10GBaseKR`,
  `HDMI20`, `HDMI21`, `DisplayPort14`, `DisplayPort20`, `MipiDPhy15`,
  `MipiCPhy20`, `MipiMPhy50`, `CanFd2Mbps`, `Custom`). Each profile
  is associated with a default rule pack that pre-populates expected
  impedance, target length, tolerance, intra-pair skew, lane-to-lane
  skew, AC-coupling-cap placement, return-path-reference-layer
  preference, etc. The data-model edit is small (~four field
  additions); the rule-pack content is shipped separately as library
  content. **Effort:** ~4 days for the schema + new operations; rule
  packs are content authored over time.

- **The `Net` struct is largely fine; the only addition is a
  `length_match_membership: Option<LengthMatchMembership>` for nets
  that join a length-match group with a per-net override of the
  group's default tolerance.** `Net.controlled_impedance` already
  carries the per-net impedance target (Batch 1). The
  `length_match_membership` field carries `group: Uuid` (the
  `NetClass.length_match_group` UUID or a board-level
  `LengthMatchGroup` registered in `Board`), `target_offset_nm:
  Option<i64>` (per-net offset from the group target — needed for
  DDR command-address skew tuning where individual nets have
  per-pin propagation-delay corrections from the IBIS package model),
  and `tolerance_override_nm: Option<i64>`. The board-level
  `length_match_groups: HashMap<Uuid, LengthMatchGroup>` collection
  carries the group's name, target length, tolerance, the matching
  algorithm (`MatchToTarget | MatchToLongest | MatchToPair |
  MatchToBus`), and the source-of-record (manually authored vs
  derived from IBIS attachment). **Effort:** ~3 days for the schema
  + new operations.

- **The `Project.compliance` block (Domain 4 — Batch 3) needs an EMC-
  posture sub-block parallel to Domain 5's `materials_posture`.** The
  recommended addition is a `emc_posture: EmcPosture` field inside
  `ProjectCompliance` carrying `default_emc_class:
  Option<EmcClass>` (auto-derived from `industry_vertical` per
  Domain 4's enum but overridable), `requires_controlled_impedance_audit:
  bool`, `requires_length_match_audit: bool`,
  `requires_return_path_audit: bool`,
  `requires_decoupling_audit: bool`, `phy_profiles_in_use:
  Vec<PhyProfile>` (denormalised cache of which PHY profiles appear
  in any NetClass — drives reporting and AI-surface explanations),
  and `target_emissions_standards: Vec<EmissionsStandard>` (a
  `Reference-only` declaration of which product-cert path the
  project intends — `FccPart15ClassB`, `Cispr32`, `Cispr25`,
  `Iec60601_1_2`, etc.). The single `requires_*` boolean fan-out
  drives validation, BOM-export enrichment, and SI-rule lint
  diagnostics. **Effort:** ~1 day added on top of the Domain 4
  ProjectCompliance work; coordinate with Batch 3.

- **`industry_vertical` (Domain 4) drives default EMC-rule
  selection.** The Phase-1 audit identified the cross-cutting:
  automotive → CISPR 25 + ISO 11452 immunity rules; medical → IEC
  60601-1-2; industrial (safety-critical) → CISPR 11 + IEC
  61000-6-2/4; consumer / ITE → CISPR 32 + IEC 61000-6-1/3.
  Domain 4's `safety_critical_industrial: bool` flag drives the
  IEC 61000-6-2 / 6-4 industrial-immunity track vs the consumer-
  grade 6-1 / 6-3 track. Datum's job is **not to validate emissions
  margin** (that is the test-lab's job) but to **default the EMC
  posture appropriately** so the user doesn't author a CAN-bus
  layout for an automotive project that uses ITE-class spacing. The
  derivation is one match expression in the engine and a documented
  per-vertical default table; the user can override per-project.

- **Dk/Df source-of-truth comes from the Stackup, never from a
  default-FR-4 table.** Domain 5 noted that halogen-free laminates
  have Dk 4.0-4.4 vs 4.4-4.8 for standard FR-4 at 1 GHz, and Df
  0.010-0.015 vs 0.018-0.025. The Batch-1 `StackupLayer` fields
  (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`,
  `roughness_um`, `material_name`) are the canonical source. Any
  Domain-6 controlled-impedance calculation MUST consume these
  fields if populated; if `dielectric_constant.is_none()` Datum MUST
  surface a diagnostic ("impedance calculation falling back to
  default Dk=4.4; populate stackup material data for an accurate
  result") rather than silently use a default. This rule applies
  identically to Rogers / Isola / Panasonic / Taconic / Arlon
  high-frequency laminates: the `material_name` carries the vendor
  reference; Datum does not encode a material database; the user (or
  vendor's IPC-1755 / IPC-2581 export) populates Dk/Df.

- **IBIS attachment is the timing-budget source for length-match
  tolerances.** This is the load-bearing dependency on Domain 2.
  When a NetClass carries `phy_profile: Some(DDR4_3200)`, the
  default length-match tolerance is derived from the DDR4-3200
  timing budget published in JESD79-4 and refined per-IC by the
  vendor's IBIS file (specifically the `[Pin Mapping]` section's
  per-pin `Pin-Parasitic` values plus `[Package]` block R/L/C). For
  a Part with `behavioural_models: [IBIS attachment]`, Datum can
  read the per-pin propagation delay and emit a per-net
  `target_offset_nm` correction when the length-match group is
  populated. For a Part without IBIS attachment, Datum falls back to
  the JESD-published worst-case timing budget and surfaces a
  diagnostic ("length-match tolerance derived from JESD79-4 worst-
  case; attach IBIS for vendor-specific budget"). This is a
  Datum-differentiator: AI-explained length-match tolerance
  derivation that no incumbent surfaces transparently.

- **Touchstone attachment is the S-parameter source for return-loss /
  insertion-loss validation.** This is the second load-bearing
  dependency on Domain 2. The recommended mode of consumption is
  **`Reference-only` at the engine layer**: Datum does not run
  S-parameter convolution math (that is HyperLynx / Sigrity / SIwave
  / ADS / SimBeor work); Datum **stores** the Touchstone attachment
  on the relevant Part (connector / cable / PHY IC), **exports** it
  as part of `export_si_artifact_bundle` for external solver
  ingestion, and **annotates** the relevant nets with a
  `touchstone_reference: Option<Uuid>` pointer for downstream tools.
  AI-surface use: an agent can answer "what's the worst-case return
  loss on this USB 3.2 differential pair given the connector
  Touchstone model" by reading `extract_touchstone_summary` (Batch 1
  MCP tool) and reporting the model's published return loss; it
  cannot independently calculate a different number.

- **Solver / simulator integration is subprocess-only per the
  no-copyleft-linkage rule.** Domain 2's Component Modelling
  research established the pattern: ngspice (GPL-3) is invoked as a
  subprocess via `validate_spice` / `export_spice_netlist`, never
  statically linked. Domain 6 inherits this pattern for any future
  SI/PI math. The recommended SI-artifact export tools
  (`export_hyperlynx_hyp`, `export_sigrity_speed_xtractim`,
  `export_si9000_board_profile`, `export_simbeor_project`,
  `export_advanced_design_system_emx`) are pure file-emission tools;
  the user (or AI agent) runs the external solver out-of-band. This
  preserves Datum's distributable-binary licence position and avoids
  taking on field-solver mathematical complexity. **Field-solver math
  is OUT of v1 / v2 scope**; transmission-line math (telegrapher
  equations, characteristic impedance from stripline / microstrip
  closed-form approximations like Wadell or Hammerstad-Jensen) **IS**
  in scope as a math-only library (`rf` crate is BSD-licensed and a
  candidate; hand-rolled is also tractable since the closed-form
  approximations are ~50 lines of math each).

- **Return-path continuity is a high-leverage DRC rule that no open-
  source EDA tool implements well.** The standard SI failure mode is
  a high-speed trace whose reference plane has a split, slot, or
  cut-out beneath it — the displacement-current return path takes a
  long detour, radiation increases, and emissions go up.
  Algorithmically the check is: for each track on a layer with a
  `phy_profile.is_high_speed()` NetClass, compute the projection of
  the track onto the adjacent reference layer and verify the
  projected polygon does not intersect any plane-split / slot /
  cut-out / void. The geometry is non-trivial (interaction with
  zone-fill regions, anti-pads, plane stitching) but tractable on
  Datum's existing zone / keepout / via primitives. **Recommend:
  ship as `RuleType::ReturnPathContinuity` in Datum's first
  Domain-6 batch** — it differentiates Datum from KiCad / Eagle /
  LibrePCB / Horizon / DipTrace / EasyEDA (none implement this as
  a first-class DRC rule) and pulls level with Altium / Cadence /
  Mentor / Pulsonix.

- **Cross-talk / coupling DRC is research-stage; defer to v2.**
  Adjacent-trace coupling-coefficient calculation requires either
  3D field-solver math or a 2D field-solver approximation (Wadell-
  style coupled-stripline / coupled-microstrip equations). The
  closed-form approximations exist but require careful corner-case
  handling (mixed-coupling with via stubs, return-path coupling,
  power-plane-cavity coupling); shipping a "good-enough" rule
  without a field solver is a maintenance liability. **Recommend:
  reserve `RuleType::CrosstalkCoupling` as a placeholder; defer
  implementation to post-v2 when the field-solver question is
  re-evaluated.**

- **PDN target-impedance frequency-domain analysis is OUT of v1
  scope but the data-model hooks should reserve room.** The full
  PDN-impedance design loop (target impedance from current
  transient × allowed voltage ripple / SQRT(2); frequency-domain
  decoupling network optimisation; bulk-cap + ceramic-cap +
  inter-plane-cap layered design; loop-inductance from cap mounting
  geometry) is a $30k Sigrity PowerSI / Ansys SIwave-PI workflow.
  Datum's role is **substrate**: store the per-rail target
  impedance and the per-cap mounting style as authored data; emit
  them as part of `export_si_artifact_bundle`. The recommended
  data-model addition is a `Net.power_distribution: Option<PdnSpec>`
  field on power-domain nets carrying `target_impedance_milliohms:
  f32`, `frequency_range_hz: (f32, f32)`, `transient_current_ma:
  f32`, `allowed_ripple_mv: f32`. This is **Planned** as a data-
  carrier; **Reference-only** as a checker.

- **The audit's substrate-only stance is correctly conservative
  for emissions standards.** FCC Part 15, CISPR 22 / 32, EN 55032,
  CISPR 25, IEC 60601-1-2, IEC 61000 family, DO-160, MIL-STD-461 are
  all `Reference-only` at Datum's engine layer — Datum carries the
  declared product-cert target in `Project.compliance.emc_posture.
  target_emissions_standards: Vec<EmissionsStandard>`, which feeds
  documentation export and AI-surface explanations
  ("this project targets FCC Part 15 Class B; the relevant
  decoupling-rule defaults follow"), but Datum does NOT validate
  margin against the standard. The deep-dive confirms there is no
  reasonable Datum-engine path to do otherwise — emissions margin is
  a measured quantity at a 3 m / 10 m semi-anechoic chamber with a
  calibrated antenna, not a calculated one. The substrate value
  Datum provides is the **paper trail**: which decoupling caps are
  placed where, which length-match groups passed, which return-path
  continuity violations were waived with rationale.

- **The high-speed-rule template-pack pattern is a Datum
  differentiator over the competition.** Altium ships the most
  complete set of pre-packaged rule templates (about 25, covering
  DDR2/3/4, USB 2.0/3.0/3.1/3.2, PCIe Gen2/3/4/5, HDMI 2.0,
  DisplayPort 1.4, MIPI D-PHY, 10/100/1000-BASE-T Ethernet,
  10G-BASE-KR, CAN, LVDS) but the format is binary `.RUL` files
  embedded in the `.PrjPCB` — not human-diffable, not AI-readable.
  Cadence Allegro ships per-PHY constraint-set templates as TCL
  scripts which are diffable but not declarative. KiCad 8 has no
  template packs; Horizon / LibrePCB / DipTrace / EasyEDA also have
  none. **Datum should ship template packs as small, versioned,
  declarative JSON files (`pool/rule-packs/ddr4_3200_default.json`)**
  carrying the PHY profile name, default length-match target /
  tolerance, default impedance target / tolerance, default
  intra-pair skew, default lane-to-lane skew, default decoupling-
  cap-placement rules, default reference-layer preferences, and
  citations to the source spec (JESD79-4 §X.Y). This makes them
  AI-readable, git-diffable, and per-pack revisable. Each pack
  ships independently of engine releases.

- **There is a paywalled-standards problem that constrains honest
  research, but it is less severe than Domain 4.** The PHY specs
  themselves are mostly behind consortium paywalls or membership
  requirements: USB-IF spec set is **free with click-through** at
  `usb.org` for the public spec PDFs (the compliance test specs are
  member-only); PCI-SIG specs are **member-only** ($4,000+/year
  membership); HDMI 2.1 is **adopter-only** ($15,000 annual
  adopter fee plus per-device royalty); DisplayPort 1.4/2.x is
  **VESA-member-only** ($5,000+ membership); MIPI specs are
  **member-only** ($16,000-$60,000 membership); JEDEC DDR3/4/5 specs
  are **free** at `jedec.org` with registration; Ethernet IEEE 802.3
  is **free** under the IEEE Get program (post-2016 release of major
  802.3 amendments to public download). FCC Part 15 is **free** at
  `ecfr.gov` (US federal regulations are not copyrighted). CISPR /
  IEC standards are **paywalled** at IEC Webstore (~CHF 200-1000
  each). Total full-paywall standards-purchase budget for a fully-
  rigorous Domain 6 implementation: ~USD 30k-100k once
  consortium memberships are included. The good news is that the
  free standards (FCC Part 15, JEDEC DDR, IEEE 802.3, USB
  public specs) cover the highest-leverage rule-pack content; the
  paywalled standards (PCI-SIG, HDMI, DisplayPort, MIPI) are needed
  only for second-tier template packs which can be deferred or
  authored on-demand from textbook digests + competitor templates.

- **The cross-domain dependency map is the densest of any deep-
  dive.** Domain 6 depends on Domain 2 (IBIS / Touchstone
  attachment for timing budget + S-parameter data), Domain 4
  (`industry_vertical`, `safety_critical_industrial`,
  `IntendedEnvironment` for default EMC class), Domain 5 (Stackup
  Dk/Df, `requires_halogen_free` flag for laminate-family hint),
  Domain 7 (PHY-IC vendor lookup for IBIS / Touchstone evidence),
  Domain 8 (length-match-group authoring is an authored op
  captured in audit-trail; PHY-profile assignment is an authored
  op; EMC waivers require sign-off). Domain 6's recommended order
  of implementation: (1) NetClass + Net schema additions; (2)
  PhyProfile enum + first three rule packs (DDR4, USB3.x, PCIe
  Gen4) authored from JESD/USB-public/textbook-digests; (3) per-
  net length-match-group operations; (4) controlled-impedance rule
  type (math-only, consumes Stackup Dk/Df); (5) length-match rule
  type; (6) return-path-continuity rule type; (7) PdnSpec data
  carrier on power nets; (8) SI-artifact export tools. Cross-talk
  rule type and field-solver work explicitly deferred to post-v2.

- **Datum has fifteen concrete recommended spec edits that fall out
  of this research.** They are listed at the end with effort
  estimates so the project owner can sequence them. The highest-
  leverage single edit is the `NetClass` extension with
  `length_match_target_nm`, `length_match_tolerance_nm`,
  `length_match_group`, `phy_profile` — every other Domain-6
  capability hangs off it. Second-highest is the
  `Project.compliance.emc_posture` block, because it lights up
  industry-vertical-driven defaulting across the engine. Third-
  highest is the `RuleType` expansion (LengthMatch, Impedance,
  DiffpairSkew, DiffpairGap, ReturnPathContinuity), because it
  promotes the deferred comment at `ENGINE_SPEC.md` line 748 to a
  first-class enum.

## Standards Catalog

### Emissions & Immunity (substrate-not-certifier framing)

#### FCC Part 15 (US, Class A/B)

**Full title.** *Title 47 CFR Part 15 — Radio Frequency Devices*,
United States Code of Federal Regulations. Subpart B governs
**unintentional radiators** (the dominant subpart for digital
electronics); Subpart C governs intentional radiators (transmitters);
Subpart E covers UNII devices; Subpart F covers UWB. Current
revision continuously maintained at `ecfr.gov`; effective text as of
2026-04 includes the November 2024 amendments to Subpart B
§ 15.107 / § 15.109 emissions limit tables.

**Issuing body.** **United States Federal Communications Commission**.
Enforcement is via FCC certification (for Subpart C intentional
radiators), Declaration of Conformity (for most Subpart B
unintentional radiators since 2017), or Verification (legacy path).

**Scope.** Defines **emissions limits** (radiated above 30 MHz,
conducted on power lines below 30 MHz) for unintentional radiators
sold or marketed in the US. Two device classes:
- **Class A** — equipment marketed for use in business / commercial
  / industrial environments; less stringent emissions limits.
- **Class B** — equipment marketed for use in residential
  environments; more stringent emissions limits (~10 dB tighter than
  Class A across most bands).

The class is selected by the manufacturer based on intended use; FCC
labelling requires the device to declare its class. Test methodology
references **ANSI C63.4** (American National Standard for Methods of
Measurement of Radio-Noise Emissions from Low-Voltage Electrical and
Electronic Equipment).

**Adoption status (2026).** **Mainstream and effectively mandatory**
for any electronic device sold in the US market. Every consumer-
electronics product, every industrial-control product, every medical-
device product, every networking-equipment product carries an FCC
ID or DoC marking with the class declaration. Test labs (Element,
UL Solutions, TÜV SÜD, Intertek, Eurofins MET) are FCC-accredited
to perform the certification testing.

**License / IP.** **Free.** US federal regulations are not
copyrighted; the full text is at `ecfr.gov/current/title-47/chapter-I/
subchapter-A/part-15`. The referenced ANSI C63.4 test methodology
standard is paywalled (~USD 200 from `ansi.org`).

**EDA tool support matrix.**
- **Altium Designer** — No Part-15 validation. The **PI
  decoupling analyser** (an Altium add-on) reports power-net
  ripple; the **SI signal-integrity engine** can simulate
  pre-layout TDR. Neither claims FCC compliance.
- **OrCAD-Capture / Allegro** — Same. Cadence's SI/PI flow runs in
  Sigrity (separate product); SiWave runs in Ansys.
- **Mentor PADS / Xpedition** — HyperLynx EM (separate product)
  performs near-field simulation that informs FCC Class B margin
  estimation, but the simulation result is not a substitute for
  test-lab measurement.
- **KiCad 8+** — No Part-15 validation; no margin estimation.
- **Eagle 9 / Fusion Electronics** — No Part-15 validation.
- **Horizon EDA, LibrePCB, DipTrace, EasyEDA** — No Part-15
  validation.
- **Datum-current** — No Part-15 validation. The Phase-1 audit
  classified this as `Blind Spot` at "low" priority, which this
  deep-dive confirms is the right framing: Datum-side validation is
  not feasible.

**Datum's coverage status.** **`Reference-only`** at the engine
layer. Datum carries the declared FCC class
(`Project.compliance.emc_posture.target_emissions_standards`
including `FccPart15ClassA` / `FccPart15ClassB`) for documentation
purposes; Datum does NOT validate emissions margin.

**Datum implementation cost.** **Trivial — pure metadata field.**
The `EmissionsStandard` enum gains two variants; the project
exposes them via `set_emc_target` MCP tool. **Effort:** ~1 hour as
part of the broader `EmissionsStandard` enum work.

**Strategic recommendation.** **`Reference-only`**. Surface the
declaration; ship the per-vertical default mapping (consumer / ITE
→ FCC Part 15 Class B); do not attempt margin estimation.

**Risks.** None at the substrate level. Risk lies in **AI-surface
overclaiming** — the agent must NOT tell a user "your project will
pass FCC Part 15 Class B"; it MUST say "your project declares an
intent to certify under FCC Part 15 Class B; the layout-rule
substrate is consistent with that intent". Strong prompt
guard-railing on this surface.

#### CISPR 22 / CISPR 32 (international ITE/multimedia)

**Full title.**
- **CISPR 22:2008+A1:2010** — *Information technology equipment —
  Radio disturbance characteristics — Limits and methods of
  measurement*. Withdrawn in 2017 (superseded by CISPR 32).
- **CISPR 32:2015+A1:2019+A2:2024** — *Electromagnetic compatibility
  of multimedia equipment — Emission requirements*. Current
  revision; consolidates ITE (formerly CISPR 22) and broadcast
  receivers / audio-video equipment (formerly CISPR 13) into a
  single multimedia-equipment scope.

**Issuing body.** **CISPR (Comité International Spécial des
Perturbations Radioélectriques / International Special Committee on
Radio Interference)**, a sub-committee of the IEC. Published as IEC
documents.

**Scope.** Defines emissions limits for multimedia equipment (ITE,
audio-video, computers, networking equipment, set-top boxes, smart
TVs, smart speakers, etc.) sold in IEC-aligned markets. Two
classes:
- **Class A** — commercial / industrial environments
- **Class B** — domestic environments (10 dB tighter than Class A)

Same class structure as FCC Part 15 (the international convention
followed FCC); limits and test methodology are largely harmonised.
Notable difference: CISPR uses a **quasi-peak detector** for some
measurements where FCC uses peak — relevant only at the test-lab
level.

**Adoption status (2026).** **Mainstream** in IEC-aligned markets
(EU adopts as EN 55032; UK as BS EN 55032; Japan as VCCI-CISPR;
Australia as AS/NZS CISPR 32; China as GB/T 9254). The
**CISPR 32:2015+A2:2024** revision is the active baseline as of
2026-04. Annex C of CISPR 32 lists the test-lab measurement
configurations.

**License / IP.** **Paywalled**. CISPR 32:2015 is ~CHF 360 from IEC
Webstore (`webstore.iec.ch`); the +A1 and +A2 amendments are
priced separately (~CHF 100 each); the consolidated edition is
~CHF 540. ANSI C63.4 (test methodology) is referenced; ~USD 200
from `ansi.org`.

**EDA tool support matrix.** **None implement CISPR 32 validation.**
Same matrix as FCC Part 15 — emissions validation is test-lab work,
not EDA work. SI/PI simulators (HyperLynx, Sigrity, SIwave, ADS)
estimate near-field margin which informs CISPR margin estimation
but does not substitute for measurement.

**Datum's coverage status.** **`Reference-only`**.

**Datum implementation cost.** **Trivial.** `EmissionsStandard`
enum gains `Cispr32`. Default mapping: consumer / ITE vertical
defaults to `Cispr32` (Class B for residential intent, Class A for
commercial intent). **Effort:** ~30 minutes.

**Strategic recommendation.** **`Reference-only`**. Surface the
declaration; default per industry vertical; do not attempt margin
estimation.

**Risks.** Same FCC overclaiming risk; same AI-surface guard-railing.

#### CISPR 11 (industrial/scientific/medical)

**Full title.** **CISPR 11:2024** — *Industrial, scientific and
medical equipment — Radio-frequency disturbance characteristics —
Limits and methods of measurement*. Current revision (Edition 7,
2024); supersedes CISPR 11:2015+A1+A2.

**Issuing body.** **CISPR / IEC**.

**Scope.** Emissions limits specifically for **ISM equipment** —
RF heaters, RF welders, microwave ovens, dielectric heaters,
medical RF equipment (MRI, RF surgical), industrial scientific
laboratory equipment. Defines two **groups** (Group 1 = equipment
that uses RF only for internal functioning; Group 2 = equipment
that intentionally generates RF in the 9 kHz to 400 GHz range for
material treatment) and two **classes** (Class A commercial / Class
B residential). The Group / Class matrix yields four limit tables.

**Adoption status (2026).** **Mainstream** in regulated industrial
and medical markets. Required for medical-device EMC compliance
(IEC 60601-1-2 references CISPR 11 for medical-equipment emissions
testing).

**License / IP.** **Paywalled**, ~CHF 360 from IEC Webstore.

**EDA tool support matrix.** **None implement CISPR 11
validation.** Same as FCC / CISPR 32.

**Datum's coverage status.** **`Reference-only`**.

**Datum implementation cost.** Trivial. `EmissionsStandard` enum
gains `Cispr11Group1ClassA`, `Cispr11Group1ClassB`,
`Cispr11Group2ClassA`, `Cispr11Group2ClassB`. Default mapping:
medical vertical → `Cispr11Group1ClassA` (default) per IEC 60601-1-2
guidance; industrial vertical with `safety_critical_industrial:
true` → `Cispr11Group1ClassA`. **Effort:** ~30 minutes.

**Strategic recommendation.** **`Reference-only`**.

**Risks.** AI-surface overclaiming on medical-device EMC margin.

#### CISPR 25 (automotive)

**Full title.** **CISPR 25:2021** — *Vehicles, boats and internal
combustion engines — Radio disturbance characteristics — Limits and
methods of measurement for the protection of on-board receivers*.
Current revision (Edition 5, 2021); supersedes CISPR 25:2016.

**Issuing body.** **CISPR / IEC**.

**Scope.** Defines emissions limits for **components and modules**
intended for installation in vehicles. Crucially, CISPR 25 measures
emissions in the bands used by on-board radio receivers (LW, MW,
SW, FM, DAB, GPS, GSM, LTE, 5G, satellite radio); the limits
protect the vehicle's own receivers from interference by other
electronics in the same vehicle. Five emissions classes (Class 1
through Class 5) where Class 5 is the most stringent (typical for
infotainment, where the on-board receiver is co-located with the
emitter).

**Adoption status (2026).** **Mainstream and effectively mandatory**
for any electronic component sold into the automotive supply chain.
Tier-1 automotive suppliers (Bosch, Continental, Denso, ZF, Aptiv)
mandate CISPR 25 compliance; CISPR 25 Class 3 is a typical baseline
for non-radio-adjacent components, Class 5 for infotainment-adjacent.

**License / IP.** **Paywalled**, ~CHF 360 from IEC Webstore.

**EDA tool support matrix.** **None implement CISPR 25
validation.** Same as other CISPR / FCC. Cadence Allegro Sigrity
PowerSI / SiWave / HyperLynx EM can be used to estimate near-field
emissions for pre-compliance work; that is solver-output, not
EDA-tool-output.

**Datum's coverage status.** **`Reference-only`**.

**Datum implementation cost.** Trivial. `EmissionsStandard` enum
gains `Cispr25Class1` through `Cispr25Class5`. Default mapping:
automotive vertical → `Cispr25Class3` (default) with override per
sub-vertical (infotainment → Class 5). **Effort:** ~30 minutes.

**Strategic recommendation.** **`Reference-only`**. Default per
automotive sub-vertical; surface for documentation; do not attempt
margin estimation.

**Risks.** AI-surface overclaiming on automotive Tier-1
qualification.

#### EN 55032 / EN 55035 (EU)

**Full title.**
- **EN 55032:2015+A11:2020+A1:2020** — *Electromagnetic compatibility
  of multimedia equipment — Emission requirements*. EU equivalent
  of CISPR 32.
- **EN 55035:2017+A11:2020** — *Electromagnetic compatibility of
  multimedia equipment — Immunity requirements*. EU equivalent of
  CISPR 35.

**Issuing body.** **CENELEC** (European Committee for
Electrotechnical Standardization). EN documents are typically
identical to the corresponding CISPR documents with EU-specific
foreword.

**Scope.** EU EMC Directive (2014/30/EU) compliance. The CE-marking
EMC requirement is satisfied by demonstrating conformity with the
relevant harmonised EN standard published in the EU Official
Journal. EN 55032 (emissions) + EN 55035 (immunity) is the
multimedia-equipment harmonised pair.

**Adoption status (2026).** **Mainstream and mandatory** for any
electronic device sold with CE marking in the EU / EEA. Notified-
body assessment is required for some classes; self-declaration is
permitted for most consumer electronics provided full test reports
are retained.

**License / IP.** **Paywalled**, ~EUR 200-400 each from national
standards bodies (DIN in Germany, BSI in UK, AFNOR in France) or
from CENELEC's eStandards portal.

**EDA tool support matrix.** **None implement EN 55032 / 55035
validation.** Same as FCC / CISPR.

**Datum's coverage status.** **`Reference-only`**.

**Datum implementation cost.** Trivial. `EmissionsStandard` enum
gains `En55032ClassA` / `En55032ClassB` (the immunity standard
EN 55035 is captured in a separate `ImmunityStandard` enum or the
broader IEC 61000-6-x family). **Effort:** ~30 minutes.

**Strategic recommendation.** **`Reference-only`**. Default
mapping: consumer / ITE vertical and EU intended market → EN 55032
+ EN 55035.

**Risks.** AI-surface overclaiming on CE-marking compliance.

#### IEC 61000-4-x (immunity test methodology)

**Full title.** **IEC 61000-4 series** — *Electromagnetic
compatibility (EMC) — Part 4: Testing and measurement techniques*.
Major sub-parts:
- **IEC 61000-4-2:2008+A1:2017** — Electrostatic discharge immunity
- **IEC 61000-4-3:2020** — Radiated, radio-frequency,
  electromagnetic field immunity
- **IEC 61000-4-4:2012+A1:2024** — Electrical fast transient / burst
  immunity
- **IEC 61000-4-5:2014+A1:2017** — Surge immunity
- **IEC 61000-4-6:2023** — Conducted disturbances induced by
  radio-frequency fields immunity
- **IEC 61000-4-8:2009** — Power frequency magnetic field immunity
- **IEC 61000-4-11:2020** — Voltage dips, short interruptions and
  voltage variations immunity tests
- **IEC 61000-4-29:2000** — Voltage dips, short interruptions and
  voltage variations on DC input power port immunity

(The full 61000-4 family has ~40 sub-parts; the eight above are the
load-bearing ones for product-cert work.)

**Issuing body.** **IEC TC 77** (Electromagnetic Compatibility
Technical Committee).

**Scope.** Specifies **test methodology** for immunity testing —
how an ESD generator pulses a device under test, how a radiated-RF
test chamber is configured, how an EFT/burst injection works. Pure
test-lab methodology; NO emissions-margin or product-safety
content.

**Adoption status (2026).** **Mainstream and load-bearing** for any
immunity-test compliance regime. Referenced by IEC 61000-6-x
(generic immunity), IEC 60601-1-2 (medical EMC), CISPR 35
(multimedia immunity), ISO 7637 (automotive EMC), DO-160
(avionics), MIL-STD-461 (defence).

**License / IP.** **Paywalled** at IEC Webstore, ~CHF 200-400 per
sub-part.

**EDA tool support matrix.** **None implement 61000-4-x
validation** — by definition, these are test-methodology standards
that govern test-lab equipment, not EDA work.

**Datum's coverage status.** **`Reference-only`** with no Datum
field. The 61000-4-x methodology is not declared per-project; it
is invoked indirectly when the project declares an immunity-target
standard (61000-6-x or vertical-specific like 60601-1-2).

**Datum implementation cost.** **None.** Cite in documentation only.

**Strategic recommendation.** **`Reference-only`**. No data-model
field needed; the standard is a test-methodology reference.

**Risks.** None.

#### IEC 61000-6-1 / 6-2 / 6-3 / 6-4 (generic by environment)

**Full title.**
- **IEC 61000-6-1:2019** — *EMC — Part 6-1: Generic standards —
  Immunity standard for residential, commercial and light-industrial
  environments*.
- **IEC 61000-6-2:2016** — *EMC — Part 6-2: Generic standards —
  Immunity standard for industrial environments*.
- **IEC 61000-6-3:2020** — *EMC — Part 6-3: Generic standards —
  Emission standard for residential, commercial and light-industrial
  environments*.
- **IEC 61000-6-4:2018** — *EMC — Part 6-4: Generic standards —
  Emission standard for industrial environments*.

**Issuing body.** **IEC TC 77**.

**Scope.** Generic emissions / immunity standards for products
without a product-specific EMC standard (e.g., a custom industrial
controller without a CISPR-level standard for its product family).
The 6-1 / 6-3 pair is the "consumer / light-industrial" baseline;
the 6-2 / 6-4 pair is the "industrial" baseline (more stringent
immunity, less stringent emissions in some bands).

**Adoption status (2026).** **Mainstream** as fallback EMC
declaration when no product-specific standard applies.

**License / IP.** **Paywalled** at IEC Webstore, ~CHF 200-400 each.

**EDA tool support matrix.** **None implement validation.**

**Datum's coverage status.** **`Reference-only`**. The
`Project.compliance.emc_posture.target_emissions_standards` enum
gains four variants (`Iec61000_6_1`, `Iec61000_6_2`, `Iec61000_6_3`,
`Iec61000_6_4`).

**Datum implementation cost.** Trivial enum extension. **Effort:**
~30 minutes.

**Strategic recommendation.** **`Reference-only`**. Default
mapping: industrial vertical with `safety_critical_industrial:
false` → 61000-6-1/6-3; industrial vertical with
`safety_critical_industrial: true` → 61000-6-2/6-4.

**Risks.** None at substrate level.

#### IEC 60601-1-2 (medical EMC)

**Full title.** **IEC 60601-1-2:2014+A1:2020** — *Medical electrical
equipment — Part 1-2: General requirements for basic safety and
essential performance — Collateral Standard: Electromagnetic
disturbances — Requirements and tests*. Current revision (Edition 4,
2014, with Amendment 1 in 2020); the predecessor Edition 3 (2007)
expired from EU regulatory acceptance in December 2018.

**Issuing body.** **IEC TC 62A** (Common aspects of electrical
equipment used in medical practice).

**Scope.** Medical-equipment-specific EMC standard. Specifies
emissions limits (referencing CISPR 11 Group 1 Class A or Class B
depending on environment) and immunity tests (referencing IEC
61000-4 series). Adds medical-specific immunity-test levels
(notably elevated radiated-RF immunity for proximity to RF medical
equipment, and elevated ESD immunity for hand-contact medical
devices). Defines four "intended use environments":
- Professional healthcare facility (hospital)
- Home healthcare environment
- Special environments (RF surgical, MR diagnostic)
- Public spaces

The home-healthcare environment carries the most stringent immunity
requirements (because the device cannot rely on the controlled EMC
environment of a hospital).

**Adoption status (2026).** **Mainstream and mandatory** for any
medical electrical equipment sold under the EU MDR (2017/745), FDA
510(k) or PMA pathway, or equivalent regulatory regimes worldwide.
The 2020 Amendment 1 tightened immunity test levels for the
home-healthcare environment.

**License / IP.** **Paywalled** at IEC Webstore, ~CHF 350.

**EDA tool support matrix.** **None implement validation.** Medical-
device EMC compliance is exclusively test-lab work; pre-compliance
SI/PI simulation in HyperLynx EM / Sigrity / SIwave informs design
decisions.

**Datum's coverage status.** **`Reference-only`**.

**Datum implementation cost.** Trivial. `EmissionsStandard` enum
gains `Iec60601_1_2_HomeHealthcare`, `Iec60601_1_2_Professional`,
`Iec60601_1_2_Special`. Default mapping: medical vertical →
appropriate variant per `IntendedEnvironment` (Domain 4 enum).
**Effort:** ~30 minutes.

**Strategic recommendation.** **`Reference-only`**. Surface the
declaration; default per medical sub-environment.

**Risks.** AI-surface overclaiming on medical-device safety. The
agent must NEVER tell a user "your device meets IEC 60601-1-2"; it
MUST say "your project declares an intent to certify under IEC
60601-1-2; the layout-rule substrate is consistent with that intent".

#### DO-160 (avionics — cross-ref Domain 4 skip)

**Full title.** **RTCA DO-160G:2010** — *Environmental Conditions
and Test Procedures for Airborne Equipment*. Current revision
(Section G, 2010); EUROCAE ED-14G is the European-equivalent.
DO-160G is divided into 26 sections covering temperature, altitude,
humidity, vibration, shock, salt spray, fungus resistance, and
sections 16-25 covering EMC (power input variations, voltage
spikes, induced signal susceptibility, RF emissions, RF
susceptibility, lightning, ESD, magnetic effect, etc.).

**Issuing body.** **RTCA** (US) and **EUROCAE** (Europe).

**Scope.** Environmental + EMC qualification for airborne
electronic equipment. The EMC sections (16-25) cover both emissions
(Section 21) and immunity (Sections 17-20, 22-23). Each section
defines equipment categories (Category A / B / C / etc.) per
installation environment.

**Adoption status (2026).** **Mainstream within aviation**;
required by FAA TSO authorisation, EASA Form 1, and most civil and
military airworthiness regulations.

**License / IP.** **Paywalled**, ~USD 250 from RTCA store.

**EDA tool support matrix.** **None implement DO-160 validation.**
Same as FCC / CISPR / IEC 60601 — test-lab work.

**Datum's coverage status.** Cross-reference to **Domain 4**:
DO-160 is on the audit's advisory-skip list for Domain 4 as a
process-grade certification Datum cannot enforce. The cross-domain
finding here: DO-160 EMC sections **could** be carried as an
`EmissionsStandard::Do160G` declaration in
`Project.compliance.emc_posture.target_emissions_standards`, but
this report **defers to Domain 4's skip recommendation**: the
broader DO-160 + DO-254 / aerospace position is "Datum is
substrate; refer users to their own QMS". The EMC subset doesn't
warrant breaking that framing.

**Datum implementation cost.** **None** (per Domain 4 skip).

**Strategic recommendation.** **`Out of scope`** at engine layer
(aligned with Domain 4); cross-referenced for completeness.

**Risks.** None — the skip is correctly conservative.

#### MIL-STD-461 (note only)

**Full title.** **MIL-STD-461G:2015** — *Requirements for the
Control of Electromagnetic Interference Characteristics of
Subsystems and Equipment*. Current revision (Rev G, 2015).

**Issuing body.** **US Department of Defense**.

**Scope.** Defence-grade EMC qualification standard. 19 test
categories (CE = conducted emissions, CS = conducted susceptibility,
RE = radiated emissions, RS = radiated susceptibility) at various
frequencies. Test methodology is referenced by the corresponding
MIL-STD-461 sub-section; equipment categories (A through L) per
installation environment (Army, Navy, Air Force, etc.).

**Adoption status (2026).** **Mainstream within US defence
procurement**.

**License / IP.** **Free** at `everyspec.com` (US DoD specs are
not copyrighted). Paywalled paper copies from defence-spec
distributors but the PDF is freely downloadable.

**EDA tool support matrix.** **None implement MIL-STD-461
validation.**

**Datum's coverage status.** Cross-reference to **Domain 4**:
defence-grade certification.

**Datum implementation cost.** **None** at engine layer; can be
carried as `EmissionsStandard::MilStd461G` declaration if the user
opts in. Defaults to no auto-population.

**Strategic recommendation.** **`Reference-only`** as a declarable
target; no validation.

**Risks.** None at substrate level.

### Signal Integrity (PHY-specific layout rules)

#### IEEE 802.3 Ethernet PHY checklists (10M through 100G)

**Full title.** **IEEE Std 802.3-2022** — *IEEE Standard for
Ethernet*. The base standard (~6,000 pages across 7 volumes) plus
~50 active amendments covering specific PHY variants. Major PHYs
relevant to PCB layout:
- **10BASE-T** (Clause 14): 10 Mbps over Cat 3+ twisted pair;
  legacy.
- **100BASE-TX** (Clause 25): 100 Mbps over Cat 5+ twisted pair;
  the original "Fast Ethernet".
- **1000BASE-T** (Clause 40): 1 Gbps over Cat 5e+ twisted pair;
  4-pair full-duplex bidirectional.
- **2.5GBASE-T / 5GBASE-T** (Clause 126, 802.3bz-2016): 2.5 / 5
  Gbps over Cat 5e+; the "NBASE-T" / "MGig" PHY for office/
  enterprise.
- **10GBASE-T** (Clause 55): 10 Gbps over Cat 6A twisted pair.
- **10GBASE-KR** (Clause 72): 10 Gbps over backplane; the SerDes
  PHY for compute/networking backplanes.
- **25GBASE-KR / 25GBASE-CR** (Clause 110, 802.3by-2016): 25 Gbps
  per lane backplane / copper.
- **40GBASE-CR4 / 40GBASE-KR4** (Clause 84/93): 40 Gbps over 4
  lanes copper.
- **100GBASE-CR4 / 100GBASE-KR4 / 100GBASE-SR4** (Clause
  91/92/95): 100 Gbps over 4 lanes copper / fibre.
- **400GBASE-CR8 / 400GBASE-KR4** (Clause 161, 802.3cd / 802.3ck):
  400 Gbps PAM4 PHYs.

Each PHY clause includes a normative **layout requirements**
section specifying differential-pair impedance (typically 100 Ω
differential), trace-length limits, intra-pair skew limits, AC-
coupling-cap placement, return-path-reference-layer requirements.

**Issuing body.** **IEEE Standards Association**, specifically the
**IEEE 802.3 Working Group**.

**Scope.** Per-PHY layout requirements that, if not met, will
prevent the PHY from passing IEEE compliance tests. The compliance
tests themselves (transmit eye, return loss, insertion loss) are
specified in the PHY's clause and are run on the assembled board
using a vector network analyser + sampling oscilloscope.

**Adoption status (2026).** **Mainstream and mandatory** for any
Ethernet-bearing product. The PHY vendors (Marvell, Broadcom,
Microsemi/Microchip, TI, Realtek, Intel/Altera, Vitesse, NXP)
publish per-PHY-IC layout guides that consolidate the IEEE
requirements with vendor-specific guidance — these are the
de-facto reference designers consult.

**License / IP.** **Free under IEEE Get program** (post-2016
release of major 802.3 amendments to public download at
`standards.ieee.org/ieee/802.3`). Vendor PHY layout guides are
free downloads from manufacturer websites.

**Reference implementations / Layout-checklist resources.**
- **Marvell 88E1512 / 88E2010P / 88E1543** layout guides (1 Gbps,
  2.5 / 10 Gbps PHY families) — public PDFs at
  `marvell.com/products`.
- **Texas Instruments DP83867 / DP83822 / DP83826** layout app
  notes — public PDFs at `ti.com`.
- **Microchip / Microsemi LAN8742 / KSZ9131 / VSC8541** layout
  guides — public PDFs.

**EDA tool support matrix.**
- **Altium Designer** — **Ethernet xMII / RGMII / SGMII /
  XAUI / KR rule templates** ship as part of the Altium template
  library; per-template binary `.RUL` files. Recent releases
  (2024-2025 cycle) added 2.5G/5G NBASE-T templates. ~12 Ethernet
  templates total.
- **OrCAD-Capture / Allegro** — **Cadence ConstraintManager**
  templates for 1000BASE-T / 10GBASE-KR / 25GBASE-KR. TCL-script
  format. Allegro Sigrity adds per-PHY pre-compliance simulation.
- **Mentor PADS / Xpedition** — **HyperLynx PHY-rule templates**
  for major Ethernet variants; pre-layout SI simulation included.
- **KiCad 8+** — No PHY template packs. Users author NetClasses
  manually with per-net `Net.controlled_impedance` (KiCad 8.0+
  feature). Length-match groups (KiCad 7+) work generically; no
  per-PHY pack.
- **Eagle 9 / Fusion Electronics** — No PHY template packs. Per-
  net length-match introduced in Fusion Electronics late 2024.
- **Horizon EDA** — No PHY template packs. Generic NetClass.
- **LibrePCB, DipTrace, EasyEDA** — No PHY template packs.
- **Datum-current** — No PHY template packs. NetClass with
  `diffpair_width / diffpair_gap` is in place; the `phy_profile:
  Option<PhyProfile>` field proposed in this report is the
  recommended addition.

**Datum's coverage status.** **Substrate `Planned`**. The NetClass
extension + `phy_profile` field + first-class rule packs is the
recommended path; the per-PHY rule pack content is shipped as
library content (one JSON file per PHY profile in
`pool/rule-packs/`).

**Datum implementation cost.**
- **Schema:** NetClass extension as described in Executive Summary
  (`length_match_target_nm`, `length_match_tolerance_nm`,
  `length_match_group`, `phy_profile`). ~4 days.
- **Rule packs (first cohort):** `ethernet_1000baset_default.json`,
  `ethernet_10gbasekr_default.json`, `ethernet_25gbasekr_default.
  json`. ~1-2 days each authored from IEEE 802.3 + vendor app
  notes; cited per spec section. Author additional packs on demand.
- **Rule engine:** New `RuleType::LengthMatch`, `RuleType::Impedance`,
  `RuleType::DiffpairSkew` — see § "Per-NetClass rule extensions"
  below for full breakdown.

**Strategic recommendation.** **`Planned`** for the substrate
(NetClass extension + LengthMatch rule type + Impedance rule type +
DiffpairSkew rule type); rule packs **shipped as library content
on a rolling basis** post-Domain-6-batch-merge.

**Risks.** **PHY templates fall behind silicon releases.** Datum
should publish a versioned rule-pack catalog (e.g., `rule-packs-2026-q3`)
with a clear "as-of" date so users know the pack reflects spec
revision X, not necessarily the latest silicon. Vendor PHY layout
guides typically supersede the IEEE base spec for vendor-specific
parameters; users should override the pack default with the vendor
guide where they conflict.

#### USB-IF (USB 2.0 / 3.x)

**Full title.**
- **USB 2.0 Specification, Rev 2.0** — original USB Hi-Speed (480
  Mbps); released April 2000; current edition with errata.
- **USB 3.0 Specification, Rev 1.0** — USB SuperSpeed (5 Gbps);
  released November 2008.
- **USB 3.1 Specification, Rev 1.0** — USB SuperSpeed+ (10 Gbps);
  released July 2013.
- **USB 3.2 Specification, Rev 1.0** — USB SuperSpeed+ dual-lane
  (20 Gbps); released September 2017.
- **USB4 Specification, Rev 2.0** — USB4 v2 (40 / 80 Gbps);
  released October 2022. Built on Thunderbolt 4 PHY; tunnels
  DisplayPort 2.0, PCIe Gen4.
- **USB Type-C Specification, Rev 2.3** — physical-connector
  spec; released May 2023.
- **USB Power Delivery Specification, Rev 3.1** — released May
  2021.

Each spec includes a normative **Layout Guidelines** section
specifying differential-pair impedance (90 Ω differential ±15%
for USB 2.0 D+/D-; 90 Ω differential ±10% for USB 3.x SuperSpeed
TX/RX; 85 Ω differential ±5% for USB4 / Thunderbolt PHY), trace
length limits per signal type, AC-coupling-cap placement (post-
PHY for SuperSpeed RX), and intra-pair / inter-pair skew limits.

**Issuing body.** **USB Implementers Forum, Inc. (USB-IF)**,
non-profit consortium.

**Scope.** Layout requirements as a precondition for **USB-IF
compliance certification** (the USB Logo program). Devices that
fail the compliance test (transmit eye, BER under jitter, return
loss, insertion loss, common-mode noise) cannot use the USB Logo.

**Adoption status (2026).** **Mainstream and mandatory** for any
USB-bearing product seeking the USB Logo.

**License / IP.** **Public-spec PDFs free** at `usb.org/documents`
with click-through; the **compliance test specifications** are
USB-IF-member-only (annual member fee starts at USD 5,000). The
USB Type-C connector is patent-pool-protected; royalty-free for
USB-IF members.

**Reference implementations / Layout-checklist resources.**
- **USB-IF Layout Guidelines** PDFs (free at `usb.org`).
- **TI / NXP / Microchip / Cypress USB hub / PHY app notes** —
  per-IC layout guides; supersede USB-IF base spec for vendor-
  specific parameters.

**EDA tool support matrix.**
- **Altium Designer** — **USB 2.0 / USB 3.x rule templates** ship
  as part of the template library; per-PHY binary `.RUL` files.
  USB4 templates added in 2024 cycle.
- **OrCAD-Capture / Allegro** — **Cadence USB rule templates**
  available; TCL-script format.
- **Mentor PADS / Xpedition** — **HyperLynx USB rule templates**.
- **KiCad 8+** — No USB rule pack; users author manually.
- **Eagle / Fusion Electronics** — No USB rule pack.
- **Horizon, LibrePCB, DipTrace, EasyEDA** — No USB rule pack.
- **Datum-current** — No USB rule pack; recommended in this
  report.

**Datum's coverage status.** **Substrate `Planned`**.

**Datum implementation cost.**
- **Rule packs (first cohort):** `usb_2_hs_default.json`,
  `usb_3_gen1_default.json`, `usb_3_gen2_default.json`. ~1-2 days
  each authored from USB-IF public specs + TI / NXP app notes.
- **Schema / rule engine:** Same NetClass extension as Ethernet.

**Strategic recommendation.** **`Planned`** substrate; first-cohort
USB 2 + USB 3.0 / 3.1 rule packs shipped with Domain 6 batch.

**Risks.** **USB-IF Logo claim must NEVER be made by Datum.**
Datum's substrate provides the layout-rule prerequisite; the Logo
is conferred by USB-IF after compliance testing at an authorised
test lab. Strong AI-surface guard-railing.

#### PCI-SIG (PCIe Gen3-5)

**Full title.**
- **PCI Express Base Specification Rev 3.0** — 8 GT/s per lane;
  released November 2010.
- **PCI Express Base Specification Rev 4.0** — 16 GT/s per lane;
  released October 2017.
- **PCI Express Base Specification Rev 5.0** — 32 GT/s per lane;
  released May 2019.
- **PCI Express Base Specification Rev 6.0** — 64 GT/s per lane
  PAM4; released January 2022.
- **PCI Express Base Specification Rev 7.0** — 128 GT/s per lane;
  draft 0.5 as of 2026-Q1.
- **PCI Express CEM Specification** (Card Electromechanical) —
  per-revision; defines the slot connector, card form factor, and
  per-lane impedance / length-match / AC-coupling requirements.

Each base spec includes normative **Channel Specifications**
covering insertion loss budget, return loss budget, crosstalk
budget, AC-coupling-cap placement, and reference clock distribution.

**Issuing body.** **PCI Special Interest Group (PCI-SIG)**, member-
only consortium founded 1992.

**Scope.** Layout requirements as a precondition for **PCI-SIG
compliance certification**.

**Adoption status (2026).** **Mainstream and mandatory** for any
PCIe-bearing product (graphics cards, NVMe SSDs, NIC cards,
expansion modules). PCIe Gen4 is mainstream baseline (as of
2026-04); Gen5 is volume-shipping; Gen6 is sampling in datacenter
networking products.

**License / IP.** **Member-only**. PCI-SIG membership is
USD 4,000-12,000 per year depending on company size. Spec PDFs
are member-only; non-members can purchase a single revision PDF
from PCI-SIG for ~USD 3,000 (Base Spec). The **PCIe-CEM** card-
electromechanical spec is also member-only.

**EDA tool support matrix.**
- **Altium Designer** — **PCIe Gen3 / Gen4 / Gen5 rule templates**.
  Gen6 not yet (as of 2026-04). Templates derived from public
  PCI-SIG channel-spec material (PCI-SIG Tech Talk slides at
  `pcisig.com/specifications`) plus vendor PHY app notes.
- **OrCAD-Capture / Allegro** — **Cadence PCIe rule templates**;
  Cadence is a PCI-SIG member with full spec access.
- **Mentor PADS / Xpedition** — **HyperLynx PCIe rule templates**;
  Mentor / Siemens is a PCI-SIG member.
- **KiCad 8+** — No PCIe rule pack; community wiki has
  user-contributed Gen3 template.
- **Eagle / Fusion Electronics** — No PCIe rule pack.
- **Horizon, LibrePCB, DipTrace, EasyEDA** — No PCIe rule pack.
- **Datum-current** — No PCIe rule pack; recommended in this
  report with the **paywall caveat that Gen5 / Gen6 rule packs
  cannot be authored from public sources alone**.

**Datum's coverage status.** **Substrate `Planned`**; rule packs
**`Planned` for Gen3/Gen4** (authorable from public sources +
textbook digests like Mike Steinberger's *PCI Express
Architecture* + competitor template inspection); **`Deferred with
prerequisite`** for Gen5/Gen6/Gen7 rule packs (prerequisite:
PCI-SIG membership or vendor-published per-PHY layout guide).

**Datum implementation cost.**
- **Rule packs (first cohort):** `pcie_gen3_default.json`,
  `pcie_gen4_default.json`. ~2-3 days each.
- **Gen5/Gen6 deferred** until either PCI-SIG membership is
  acquired or vendor-published layout guides for specific PHY ICs
  are sourced (most major Gen5 PHY vendors — TI, Astera Labs,
  Microchip — publish per-IC layout guides that include the
  relevant PCIe-base-spec parameters).

**Strategic recommendation.** **`Planned`** substrate; Gen3/Gen4
rule packs shipped with Domain 6 batch; Gen5+ rule packs deferred
with explicit user-facing note about the paywall.

**Risks.** **PCI-SIG paywall is the single biggest constraint
on full Domain-6 rule-pack coverage.** Mitigation: ship Gen3 / Gen4
packs from public sources, cite vendor app notes (which themselves
reference the PCI-SIG spec) for higher-rev rule values, and
document the paywall constraint in the rule-pack README so users
understand the disposition.

#### HDMI 2.x (fold-in candidate)

**Full title.**
- **HDMI Specification Version 2.0** — 18 Gbps; released September
  2013.
- **HDMI Specification Version 2.1** — 48 Gbps Fixed Rate Link
  (FRL); released November 2017.
- **HDMI Specification Version 2.1a** — released February 2022;
  adds Source-Based Tone Mapping.

Each spec includes a Compliance Test Specification (CTS) that
defines TMDS impedance (100 Ω differential ±10% per pair), maximum
intra-pair skew, AC-coupling requirements, and HEC / HEAC layout
requirements.

**Issuing body.** **HDMI Forum, Inc.** and **HDMI Licensing
Administrator, LLC** (the licensing arm).

**Scope.** Layout requirements as a precondition for HDMI
certification.

**Adoption status (2026).** **Mainstream and mandatory** for any
HDMI-bearing product. The HDMI Logo is gated on certification.

**License / IP.** **Adopter-only**. HDMI Adopter Agreement requires
a USD 15,000 annual fee plus per-device royalties (USD 0.05-0.15
per device depending on logo / HDCP usage). Spec PDFs are member-
only.

**EDA tool support matrix.**
- **Altium / OrCAD / Mentor** — HDMI 2.0 and 2.1 templates ship in
  recent releases (these vendors are HDMI Adopters with full spec
  access).
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** — No
  HDMI rule packs.
- **Datum-current** — No HDMI rule pack.

**Datum's coverage status.** **Per audit fold-in: substrate
`Planned`; rule pack `Deferred with prerequisite`** (prerequisite:
HDMI Adopter Agreement OR a vendor-published HDMI-PHY-IC layout
guide that includes HDMI-spec parameters).

**Datum implementation cost.** **Substrate cost: zero** (covered by
generic NetClass + PhyProfile work). **Rule-pack cost: deferred**
until HDMI spec access is available.

**Strategic recommendation.** **Per audit fold-in:** ship the
substrate (NetClass + PhyProfile + LengthMatch + Impedance + Skew
rule types); ship the HDMI rule pack later as add-on library
content when spec access is available. Mitigation if no Adopter
membership: source from TI / Lattice HDMI repeater app notes
(which cite HDMI 2.x parameters publicly).

**Risks.** Same paywall constraint as PCI-SIG. The fold-in
strategy mitigates by treating HDMI as a per-PHY rule-pack
content question separable from the substrate.

#### DisplayPort 1.4 / 2.x (fold-in candidate)

**Full title.**
- **DisplayPort Standard Version 1.4** — 32.4 Gbps HBR3; released
  March 2016.
- **DisplayPort Standard Version 2.0** — 80 Gbps UHBR; released
  June 2019.
- **DisplayPort Standard Version 2.1** — released October 2022;
  consolidates 2.0 errata.
- **DisplayPort Standard Version 2.1a** — released April 2024;
  adds Panel Replay enhancements.

Each spec includes a **Compliance Test Specification** defining
ML / AUX channel impedance (100 Ω differential), maximum lane-to-
lane skew, AC-coupling requirements.

**Issuing body.** **VESA (Video Electronics Standards
Association)**.

**Scope.** Layout requirements as a precondition for DisplayPort
certification.

**Adoption status (2026).** **Mainstream** in PC and professional
video markets.

**License / IP.** **VESA-member-only** (VESA membership starts at
USD 5,000 per year for adopters).

**EDA tool support matrix.**
- **Altium / OrCAD / Mentor** — DisplayPort 1.4 and 2.0 / 2.1
  templates ship in recent releases.
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** — No
  DisplayPort rule packs.
- **Datum-current** — No DisplayPort rule pack.

**Datum's coverage status.** **Per audit fold-in: substrate
`Planned`; rule pack `Deferred with prerequisite`**.

**Datum implementation cost.** Same as HDMI: substrate zero;
rule-pack deferred.

**Strategic recommendation.** **Per audit fold-in.** Same as HDMI.

**Risks.** Same paywall constraint.

#### MIPI D-PHY / C-PHY / M-PHY (fold-in candidate)

**Full title.**
- **MIPI D-PHY Specification v3.0** — 4.5 Gbps / lane; released
  May 2024. Source-synchronous; widely used for camera (CSI-2) and
  display (DSI) interfaces.
- **MIPI C-PHY Specification v2.1** — 7.14 Gsps / lane; released
  September 2022. Three-wire embedded-clock; used for high-
  resolution camera + display.
- **MIPI M-PHY Specification v5.0** — 11.6 Gbps / lane; released
  December 2023. SerDes; used for UFS storage, UniPro, M-PCIe.

Each spec includes layout requirements for the respective signalling
mode.

**Issuing body.** **MIPI Alliance**.

**Scope.** Layout requirements as a precondition for MIPI
certification.

**Adoption status (2026).** **Mainstream in mobile / embedded**
markets.

**License / IP.** **Member-only**. MIPI Alliance membership starts
at USD 16,000 per year; spec PDFs are member-only.

**EDA tool support matrix.**
- **Altium / OrCAD / Mentor** — D-PHY / C-PHY / M-PHY templates
  ship in recent releases for member-vendors.
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** — No
  MIPI rule packs.
- **Datum-current** — No MIPI rule pack.

**Datum's coverage status.** **Per audit fold-in: substrate
`Planned`; rule pack `Deferred with prerequisite`**.

**Datum implementation cost.** Same as HDMI / DisplayPort.

**Strategic recommendation.** **Per audit fold-in.**

**Risks.** Same paywall constraint. Mitigation: vendor camera-
sensor / display-driver app notes (Sony, OmniVision, Samsung
Display, BOE, Novatek) publish D-PHY / C-PHY layout guides that
cite MIPI parameters publicly.

#### JEDEC DDR3/4/5/LPDDRx

**Full title.**
- **JESD79-3F (DDR3)** — released July 2012; current revision F.
  1066-2133 MT/s.
- **JESD79-4D (DDR4)** — released October 2024 (Rev D); 1600-3200
  MT/s. The most-deployed DDR generation as of 2026-04 outside
  recent flagship desktops.
- **JESD79-5C (DDR5)** — released March 2024 (Rev C); 4800-8400
  MT/s. Mainstream baseline for 2024+ Intel/AMD platforms.
- **JESD209-4D (LPDDR4 / LPDDR4X)** — released September 2021
  (Rev D); 1600-4267 MT/s.
- **JESD209-5C (LPDDR5 / LPDDR5X)** — released February 2024
  (Rev C); 4800-9600 MT/s. Mainstream mobile / automotive.
- **JESD209-6 (LPDDR6)** — released August 2025 (Rev 0);
  10000-14400 MT/s. Sampling 2026.

Each spec includes a normative **Output Timing** section defining
timing parameters (tDS / tDH setup / hold, tCK clock period, tCWL /
tCL latencies) that drive layout-rule derivation. Vendor controller
ICs (DDR controllers in CPUs / FPGAs / ASICs) publish per-IC
layout guides that include length-match tolerances, intra-pair skew,
group-skew (data byte lane to data byte lane), command-address
group skew vs data-clock skew (the "fly-by topology" requirement
for DDR3+).

**Issuing body.** **JEDEC** (Joint Electron Device Engineering
Council).

**Scope.** Per-PHY-version layout requirements + vendor controller
guides.

**Adoption status (2026).** **Mainstream and mandatory** for any
DDR-bearing design. DDR4 and DDR5 dominate; LPDDR4 / LPDDR5 dominate
mobile.

**License / IP.** **Free** at `jedec.org` with free registration.

**Reference implementations / Layout-checklist resources.**
- **JEDEC JESD79 / JESD209 spec PDFs** — free with registration.
- **Intel / AMD / Xilinx / Lattice / Microchip / NXP DDR
  controller layout guides** — public PDFs at vendor websites.
- **Micron Technical Note TN-46-14 / TN-47-12** (DDR3 / DDR4
  layout) — public PDFs at `micron.com`.

**EDA tool support matrix.**
- **Altium Designer** — **DDR3 / DDR4 / DDR5 rule templates** ship
  with each release. LPDDR4 / LPDDR5 templates added 2024 cycle.
  ~10 DDR-family templates total.
- **OrCAD-Capture / Allegro** — **Cadence ConstraintManager DDR
  templates**; deeply integrated with Sigrity for pre-layout
  timing analysis.
- **Mentor PADS / Xpedition** — **HyperLynx DDR templates**; the
  HyperLynx DDR Wizard generates per-byte-lane length-match groups
  automatically.
- **KiCad 8+** — No DDR template pack. Length-match groups (KiCad 7+)
  work generically.
- **Eagle / Fusion Electronics** — No DDR template pack.
- **Horizon, LibrePCB, DipTrace, EasyEDA** — No DDR template pack.
- **Datum-current** — No DDR template pack; recommended in this
  report as a first-cohort rule pack.

**Datum's coverage status.** **Substrate `Planned`; rule packs
`Planned`** for DDR3 / DDR4 / DDR5 / LPDDR4 / LPDDR5 (all
authorable from free JEDEC specs + free vendor layout guides).
LPDDR6 deferred until volume samples ship.

**Datum implementation cost.**
- **Rule packs (first cohort):** `ddr4_2400_default.json`,
  `ddr4_3200_default.json`, `ddr5_4800_default.json`,
  `lpddr4_4266_default.json`, `lpddr5_6400_default.json`. ~2-3 days
  each authored from JEDEC + vendor guides.
- **Rule engine:** New `RuleType::LengthMatchGroup` is
  load-bearing for DDR fly-by-topology rule (command-address group
  skew separately from data-byte-lane group skew); see § "Per-
  NetClass rule extensions" below.

**Strategic recommendation.** **`Planned` substrate + `Planned` rule
packs**. DDR is the highest-leverage rule-pack family because
JEDEC + vendor guides are free, the topology is well-documented,
and DDR is the most common high-speed-routing pain point in
mid-complexity designs.

**Risks.** **DDR layout depends heavily on the controller IC's
package model.** A given DDR4 controller (e.g., Xilinx Zynq
UltraScale+ vs NXP Layerscape vs Intel Tiger Lake) has different
internal package skew that the layout-rule must compensate for. The
Datum-side answer: when the DDR controller Part has an attached
IBIS file with `[Package]` block populated, derive per-pin
`target_offset_nm` from the IBIS `R_pkg / L_pkg / C_pkg` parameters;
when no IBIS attachment, use the JEDEC worst-case value and surface
a diagnostic.

#### IEEE 1149.x JTAG

**Full title.**
- **IEEE Std 1149.1-2013** — *Standard for Test Access Port and
  Boundary-Scan Architecture*; the original JTAG standard. Current
  revision (2013); the foundational boundary-scan standard.
- **IEEE Std 1149.6-2015** — AC-coupled (high-speed) boundary
  scan extension.
- **IEEE Std 1149.7-2009** — compact (cJTAG) two-pin reduction.
- **IEEE Std 1149.10-2017** — high-speed test access port.

JTAG layout is largely about chain topology (TDI → TDO → TDI → TDO
chain ordering), TCK / TMS distribution (typically slow signals,
~10-30 MHz), and pull-up resistors on TMS / TRST.

**Issuing body.** **IEEE Standards Association**.

**Scope.** Boundary-scan TAP layout requirements; chain-topology
authoring.

**Adoption status (2026).** **Mainstream and ubiquitous** in any
design with FPGAs / ASICs / large MCUs.

**License / IP.** **Paywalled** at IEEE Xplore (~USD 200 for the
1149.1 standard).

**EDA tool support matrix.**
- **Altium Designer** — JTAG chain editor for boundary-scan tools.
- **OrCAD-Capture / Allegro** — JTAG chain editor.
- **KiCad 8+** — No first-class JTAG chain editor.
- **Datum-current** — No first-class JTAG chain editor; would
  hang off the Domain 6 substrate as a future enhancement.

**Datum's coverage status.** **`Reference-only`** at engine layer;
JTAG chain authoring is a candidate for a future M7+ schematic-
editor enhancement, NOT a Domain-6 priority.

**Datum implementation cost.** **Out of Domain-6 scope.** Cite as
a future enhancement; do not extend Domain 6 with JTAG-specific
work.

**Strategic recommendation.** **`Reference-only`** for the Domain
6 batch. Defer JTAG chain authoring to a future schematic-editor
work item.

**Risks.** None — explicit deferral.

#### CAN / CAN FD / CAN XL (automotive)

**Full title.**
- **ISO 11898-1:2024** — *Road vehicles — Controller area network
  (CAN) — Part 1: Data link layer and physical signalling*. Current
  revision (2024); covers CAN 2.0, CAN FD, and CAN XL.
- **ISO 11898-2:2024** — *Part 2: High-speed medium access unit*.
  Defines the high-speed CAN PHY (1 Mbps) and CAN-FD-capable PHY
  (5-8 Mbps).
- **ISO 11898-5:2007** — Low-power CAN PHY.

CAN bus is a multi-drop differential bus with **120 Ω termination
at each end** (no termination in the middle); stub length per node
must be < 30 cm for 1 Mbps CAN, < 5 cm for 5 Mbps CAN-FD.

**Issuing body.** **ISO TC 22 SC 31** (Road vehicles —
Communication and electrical systems).

**Scope.** Per-bus topology rules: single-pair differential
(typically ~60 Ω characteristic impedance per wire, 120 Ω
differential); termination resistors at chain ends; stub-length
limits; common-mode-choke placement.

**Adoption status (2026).** **Mainstream and mandatory** in any
automotive electronics design. CAN FD has displaced classic CAN in
new designs since ~2018; CAN XL is sampling but not yet volume.

**License / IP.** **Paywalled** at ISO Webstore, ~CHF 200 each.

**EDA tool support matrix.**
- **Altium Designer** — CAN bus rule template available; covers
  termination + stub length.
- **OrCAD-Capture / Allegro** — CAN constraint template available.
- **Mentor PADS / Xpedition** — CAN template available.
- **KiCad 8+** — No CAN template.
- **Datum-current** — No CAN template.

**Datum's coverage status.** **Substrate `Planned`; rule pack
`Planned`** for CAN / CAN FD (CAN XL deferred until volume samples).

**Datum implementation cost.** **Rule pack: ~1 day** authored from
ISO 11898 + free TI / NXP CAN PHY app notes.

**Strategic recommendation.** **`Planned`** substrate; CAN / CAN FD
rule packs in first-cohort batch (low effort, high automotive value).

**Risks.** None notable.

#### Quad/Octal SPI flash

**Full title.** **JEDEC JESD216F.02 / JESD251A** — *Serial Flash
Discoverable Parameters (SFDP)* (216F.02, 2023) and *Serial NOR
Flash Common Flash Interface (CFI) Discovery* (251A, 2023). The
underlying QSPI / OSPI PHY is vendor-defined; layout is governed
by per-IC vendor app notes.

**Issuing body.** **JEDEC**.

**Scope.** Layout for QSPI (4-bit) and OSPI (8-bit) flash, where
clock-data alignment requires length-match between CLK and the
DQ0..DQ7 signals; typical tolerance ±5 mil to ±10 mil for OSPI at
200 MHz.

**Adoption status (2026).** **Mainstream** in any embedded design
with external flash (FPGAs, MCUs, SoCs).

**License / IP.** **Free** at `jedec.org` with registration.

**EDA tool support matrix.**
- **Altium / OrCAD / Mentor** — Generic length-match groups can be
  used; no QSPI-specific template typically.
- **Datum-current** — No QSPI template; recommended.

**Datum's coverage status.** **Substrate `Planned`; rule pack
`Planned`**.

**Datum implementation cost.** **Rule pack: ~0.5 day** authored
from vendor app notes (Micron, Cypress/Infineon, Macronix,
Winbond all publish QSPI / OSPI layout guides).

**Strategic recommendation.** **`Planned`**. QSPI / OSPI rule pack
in first-cohort batch (very low effort).

**Risks.** None.

### Power Integrity (substrate)

#### PDN target impedance

**Full title.** No single standard. The discipline is documented in
the **Howard Johnson / Martin Graham *High-Speed Digital Design*
(1993)** textbook and the **Eric Bogatin *Signal and Power Integrity
— Simplified*** (3rd edition, 2018) textbook. The IEEE Power
Integrity Working Group has produced practitioner-level papers
(2018-2024).

**Scope.** Frequency-domain decoupling-network design: target
impedance Z_T = ΔV_max / ΔI_transient; per-decoupling-cap mounting-
loop inductance (~1-2 nH per via + cap); per-cap self-resonant
frequency; multi-cap parallel-resonance design.

**Adoption status (2026).** **Mainstream methodology** in any
high-speed design.

**License / IP.** **Open methodology**; textbook references are
copyrighted.

**EDA tool support matrix.**
- **Altium PI Analyzer (PDN Analyzer)** — frequency-domain PDN
  impedance simulation; per-cap loop-inductance estimation.
- **OrCAD-Sigrity PowerSI / PowerDC** — full PDN impedance
  simulation; the Cadence flagship.
- **Ansys SIwave-PI** — full PDN impedance.
- **HyperLynx PI** — full PDN impedance.
- **KiCad / Eagle / Horizon / LibrePCB / DipTrace / EasyEDA** —
  No PDN simulation. KiCad has a community plugin (kicad-pi-helper)
  that estimates per-cap loop inductance from geometry.
- **Datum-current** — No PDN simulation; substrate-only.

**Datum's coverage status.** **Substrate `Planned`** for the
data-carrier; **`Out of scope`** for the simulation. The
recommended `Net.power_distribution: Option<PdnSpec>` data carrier
captures `target_impedance_milliohms`, `frequency_range_hz`,
`transient_current_ma`, `allowed_ripple_mv` for export to
external solvers.

**Datum implementation cost.** **Substrate cost: ~2 days** for
the PdnSpec struct + new operations.

**Strategic recommendation.** **`Planned`** substrate; **`Out of
scope`** simulation. The substrate value is significant: the AI
agent can answer "what target impedance was specified for this
power rail" by reading `Net.power_distribution`, and Datum's
PDN data feeds external solver export tools.

**Risks.** None at substrate level.

#### IPC-2152 (cross-ref)

**Full title.** **IPC-2152** — *Standard for Determining
Current-Carrying Capacity in Printed Board Design*. Released 2009;
formally "no longer maintained" per IPC's 2024 standards revision
table but treated as authoritative by industry.

**Cross-ref:** Already covered in
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-2152
(lines 480-516, 1250, 1321, 1464). Domain 6 does NOT re-survey.

**Datum's coverage status.** Per IPC research: **Reference-only**
at the engine layer in the current roadmap; chart-based current-
capacity calculation deferred to M8 thermal milestone.

**Cross-cutting note for Domain 6.** Trace-current capacity is a
DC-thermal concern, not a SI/PI concern. The intersection with
Domain 6 is at the **return path**: a high-speed signal trace
returning displacement current through a power plane shares the
plane with bulk DC current. The IPC-2152 chart-based capacity
applies to the DC component; the SI/PI concern is the AC return-
path continuity (handled by `RuleType::ReturnPathContinuity`).

### Controlled-Impedance References

#### IPC-2141A (cross-ref)

**Full title.** **IPC-2141A** — *Design Guide for High-Speed
Controlled Impedance Circuit Boards*. Released 2004; current
revision A. Predecessor IPC-D-317A was retired when IPC-2141 was
released.

**Cross-ref:** Already covered in
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-2581
cross-ref at line 682, with brief context in the impedance
discussion. Domain 6 does NOT re-survey.

**Domain-6 framing.** IPC-2141A is the **methodology reference** for
controlled-impedance trace design: closed-form approximations for
microstrip / stripline / coplanar / coplanar-with-ground impedance,
correction factors for trace-width tolerance, dielectric-constant
tolerance, copper-thickness tolerance. The standard publishes the
Wheeler / Hammerstad-Jensen / IPC formulae and the impedance-
tolerance budget approach. **Datum's `RuleType::Impedance` checker
should consume IPC-2141A's formulae** as the math basis,
parameterised by the Stackup material data (Batch 1 fields).

**Datum's coverage status.** **`Planned` as the math basis** for
`RuleType::Impedance`. The `Net.controlled_impedance`
(`Option<ImpedanceSpec>`) field already exists; the rule type that
validates a routed trace against the spec is what's deferred.

**Datum implementation cost.** **Rule type: ~3 days** for the
math (Wheeler / Hammerstad-Jensen / IPC closed-form approximations
for microstrip / stripline / coplanar) + ~2 days for the checker
that walks each track on a controlled-impedance net and validates
its calculated impedance against `Net.controlled_impedance`.

**Strategic recommendation.** **`Planned`** with IPC-2141A as the
math basis. Cite the standard in rule-pack documentation; ship the
formulae as a hand-rolled module (the formulae are ~50 lines of
math each, well-documented in textbooks, and not subject to
copyright).

**Risks.** **Closed-form approximation accuracy is ±5-10% vs full
2D field solver.** This is acceptable for Datum's substrate role
(the rule fires when the calculated impedance is outside
`tolerance_pct` of the target). For higher-accuracy work, the
Touchstone export path is available; the user runs SiWave / Sigrity
/ HyperLynx for the precise number.

#### Material-spec sources (Rogers, Isola, Panasonic — note only)

**Scope.** Vendor PCB-laminate specifications publish per-material
Dk / Df values across frequency. Notable families:
- **Rogers RO4350B / RO4003C / RO3003** — high-frequency PTFE-
  ceramic laminates; Dk 3.48 / 3.55 / 3.0; Df 0.0037 / 0.0027 /
  0.0010 at 10 GHz.
- **Isola FR-4-class (370HR, I-Speed, IS415, IS408, Tachyon
  100G)** — premium FR-4 to ultra-high-frequency; Dk 4.04-3.06,
  Df 0.0203-0.0021.
- **Panasonic Megtron 4 / 6 / 7N** — high-speed digital
  laminates; Dk 3.6-3.4, Df 0.005-0.002.
- **Taconic TLY / RF-30** — low-loss laminates.
- **Arlon AD250 / 25N** — high-frequency hydrocarbon laminates.

**Datum's coverage status.** **`Reference-only`**. The
`StackupLayer.material_name: Option<String>` field carries the
vendor reference; the `dielectric_constant` and `loss_tangent`
fields carry the values. Datum does NOT encode a vendor material
database — that catalogue is vendor-published, frequency-dependent,
and out of scope as Datum-maintained data.

**Strategic recommendation.** **`Reference-only`**; rely on user
or import (KiCad 8+ / IPC-2581 / IPC-1755 / vendor-published
stackup data sheets) to populate the Stackup fields. Datum should
**warn** when the Dk/Df fields are unpopulated and an impedance
calculation is requested, rather than silently default.

### Coupling / Skew Topics

#### Edge-coupled vs broadside-coupled vs mixed

**Topology.** Differential-pair routing comes in three topologies:
- **Edge-coupled** — both traces of the pair are on the same
  layer, side-by-side. The dominant topology. Characteristic
  impedance is set by trace width, trace gap, and dielectric
  geometry. Wadell's *Transmission Line Design Handbook* (1991)
  publishes the closed-form approximations.
- **Broadside-coupled** — the two traces are on adjacent layers
  (one above the other), typically separated by a thin pre-preg.
  Used in high-density routing where edge-coupled space is
  unavailable. Characteristic impedance is set by trace width,
  layer separation, and dielectric.
- **Mixed (dual-stripline coupled)** — one trace on each of two
  inner layers separated by a ground plane. Less common; used in
  RF / microwave.

**Datum's coverage status.** **Edge-coupled is the v1 baseline**;
the `NetClass.diffpair_width / diffpair_gap` fields are sufficient
for edge-coupled design. **Broadside-coupled and mixed** require
additional NetClass fields (`diffpair_topology: DiffpairTopology`
enum with `EdgeCoupled | BroadsideCoupled | Mixed` variants;
`diffpair_layer_offset: Option<i32>` for broadside) — recommend
`Planned` for v2.

**Strategic recommendation.** **Edge-coupled** in v1; broadside +
mixed deferred to v2 as the use cases (high-density / RF) emerge.

#### Within-pair vs pair-to-pair skew

**Topology.** Skew comes in two flavours:
- **Within-pair (intra-pair) skew** — the timing difference
  between the P and N members of a differential pair. Tight
  tolerance (typically < 5 mil, often < 1 mil for 25 Gbps+); a
  miscompensated intra-pair skew converts differential signal into
  common-mode noise (the dominant EMC failure mode for high-speed
  diff pairs).
- **Pair-to-pair (inter-pair / lane-to-lane) skew** — the timing
  difference between different differential pairs in a parallel
  bus or PHY. Looser tolerance (typically < 50 mil for Ethernet /
  USB / PCIe; tighter for DDR strobe-data alignment).

**Datum's coverage status.** **`Planned`** as `RuleType::DiffpairSkew`
(intra-pair) and `RuleType::LengthMatch` with a group-membership
mechanism (inter-pair). The recommended NetClass extension carries
`diffpair_skew_tolerance_nm: Option<i64>` for intra-pair; the
`length_match_group` UUID for inter-pair.

#### Length-tuning topology

**Topology.** Length-match comes in three modes:
- **Match-to-target** — every net in the group must match a
  declared target length (e.g., DDR strobe at 50 mm). Used for
  group-skew control where the group has an absolute timing
  budget.
- **Match-to-longest** — every net in the group must match the
  longest net in the group within a declared tolerance. Used when
  the absolute length isn't constrained but matching is.
- **Match-to-pair** — the P/N members of a differential pair must
  match each other. (Intra-pair skew, separate from group skew.)
- **Match-to-bus** — within a multi-byte bus (DDR data byte lane),
  members within the byte must match each other; bytes can match
  bytes within a looser group tolerance.

**Datum's coverage status.** **`Planned`** as `LengthMatchAlgorithm`
enum on the board-level `LengthMatchGroup` struct.

**Serpentine geometry.** The M5 routing kernel surface (Phase 1
audit noted this as Partial) handles serpentine length-tuning
authoring. The Domain-6 work is the **rule** (does the routed
length meet the target?), not the **authoring** (how to draw the
serpentine). See `crates/engine/src/board/` for the existing
serpentine support.

## Cross-Cutting Patterns

### Substrate-vs-certification (extended to EMC)

The substrate-vs-certification framing introduced in Domain 4 (industry-
vertical compliance) extends naturally to EMC and signal integrity.
**Datum is the substrate.** The certifying parties are accredited test
labs running standardised emissions / immunity tests on assembled
boards. Datum's role is to:

- **Carry** the declared product-cert target
  (`Project.compliance.emc_posture.target_emissions_standards: Vec<EmissionsStandard>`)
  for documentation and per-vertical defaulting.
- **Validate** the layout-rule prerequisites — controlled-impedance
  audit, length-match audit, return-path-continuity audit,
  decoupling-cap-placement audit — that good EMC outcomes depend on.
- **Export** the SI/PI artifacts (IBIS netlist for pre-layout
  timing analysis, Touchstone S-parameter export for return-loss
  validation, HyperLynx HYP / Sigrity SPEED-XTRACT-IM project
  bundle, IPC-2581 Rev C `<ImpedancesProperties>` for fab-side
  impedance verification) that external solvers and downstream
  fab tools consume.
- **Surface** AI-explained diagnostics ("this length-match
  tolerance was derived from JESD79-4 worst-case; attach the
  vendor IBIS to refine") that no incumbent tool exposes.

Datum does **NOT**:

- Validate emissions margin against FCC / CISPR / EN limits
- Run 2D / 3D field solver math
- Run S-parameter convolution
- Calculate transmit-eye / receive-jitter compliance against PHY
  certification tests
- Make USB Logo / HDMI / DisplayPort / Ethernet / PCIe compliance
  claims

The boundary is honest, defensible, and scoped to what an EDA tool
can credibly do without becoming a $30k SI/PI specialist tool.

### Industry-vertical drives EMC defaults (cross-ref Domain 4)

The Phase-1 audit and Domain 4's deep-dive jointly identified the
cross-cutting: `industry_vertical` (Domain 4 enum) drives
default EMC class. The Domain-6 implementation:

| `industry_vertical` value | Default `target_emissions_standards` | Default immunity standard | Notes |
|---|---|---|---|
| `Consumer` / `Ite` | `FccPart15ClassB` + `Cispr32ClassB` + `En55032ClassB` | `Iec61000_6_1` + `En55035` | Residential / commercial baseline |
| `Industrial` (`safety_critical_industrial: false`) | `FccPart15ClassA` + `Cispr32ClassA` + `En55032ClassA` | `Iec61000_6_1` | Light-industrial baseline |
| `Industrial` (`safety_critical_industrial: true`) | `FccPart15ClassA` + `Cispr11Group1ClassA` | `Iec61000_6_2` + `Iec61000_6_4` | Industrial baseline |
| `Automotive` | `Cispr25Class3` (default) or `Cispr25Class5` (infotainment) | `Iso7637_2` + `Iso11452_2` | Automotive component-level |
| `Medical` | `Iec60601_1_2_Professional` (default) or `Iec60601_1_2_HomeHealthcare` | `Iec60601_1_2` (immunity built-in) | Medical-equipment baseline |
| `Aerospace` (per Domain 4 skip — substrate-only) | `Do160G` (declarable, not auto-default) | `Do160G` (declarable, not auto-default) | Avionics — substrate-only |
| `Defence` (per Domain 4 skip — substrate-only) | `MilStd461G` (declarable, not auto-default) | `MilStd461G` (declarable, not auto-default) | Defence — substrate-only |

The defaults are documented; the user can override per project; the
AI agent surfaces the default with rationale ("automotive vertical
defaulted CISPR 25 Class 3 emissions; override if your sub-vertical
is infotainment").

### Dk/Df source-of-truth (cross-ref Domain 5)

Per Domain 5's findings: halogen-free laminates have different Dk/Df
than standard FR-4. The Stackup material fields (Batch 1) are the
canonical source. Domain 6's controlled-impedance rule MUST consume
these fields:

```
fn calculate_microstrip_impedance(
    track_width_nm: i64,
    dielectric_height_nm: i64,
    copper_thickness_nm: i64,
    dielectric_constant: f32,    // from StackupLayer.dielectric_constant
    // ...
) -> f32 { ... }
```

When `dielectric_constant.is_none()`:

```
DIAGNOSTIC: SI-001 Stackup material data unpopulated
  Net: USB3_TX_P
  NetClass: USB3
  Layer: Top (microstrip)
  Reference dielectric: PrePreg_Top (StackupLayer)
  Issue: dielectric_constant is None
  Calculation: falling back to default Dk=4.4 (FR-4 1 GHz approximation)
  Recommended: populate StackupLayer.dielectric_constant from vendor
               material data sheet, IPC-1755, or KiCad 8+ stackup-material
               import. Vendor reference: see StackupLayer.material_name
               (currently: "FR-4 unspecified")
```

The diagnostic is informational, not blocking; the calculation
proceeds with the default but the user knows the result is
approximate. This is the same source-of-truth-aware pattern Domain
5's substance-list pinning uses.

### IBIS / Touchstone consumption (cross-ref Domain 2)

Per Domain 2's contract: `Part.behavioural_models: Vec<ModelAttachment>`
carries IBIS / SPICE / Touchstone attachments. Domain 6's
consumption pattern:

**IBIS attachment is consumed for length-match tolerance derivation.**

When a NetClass carries `phy_profile: Some(DDR4_3200)`:

1. Look up the rule pack `pool/rule-packs/ddr4_3200_default.json`.
2. Read the default JEDEC worst-case length-match tolerance (e.g.,
   ±5 mil for byte-lane group skew).
3. For each Part in the design with the DDR4 controller role, check
   if `behavioural_models` contains an IBIS attachment.
4. If yes: read the IBIS `[Pin Mapping]` per-pin propagation delays
   and `[Package]` block R/L/C; refine the worst-case tolerance to
   the vendor-specific value; emit per-net `target_offset_nm` to
   compensate for per-pin package skew.
5. If no: keep the JEDEC worst-case; surface the diagnostic that
   IBIS attachment would refine the budget.

The MCP tool `extract_ibis_pin_table` (Batch 1) is the engine-side
read; the rule engine is the consumer.

**Touchstone attachment is consumed for return-loss / insertion-loss
reporting.**

When a NetClass carries `phy_profile: Some(USB3_Gen2)` and a Part
in the design has a Touchstone attachment (typical for USB3
connectors / cables):

1. Read the Touchstone S-parameter data via
   `extract_touchstone_summary` (Batch 1 MCP tool).
2. Report the worst-case return loss and insertion loss across
   the USB3 Gen2 frequency range (5 GHz fundamental + harmonics).
3. Compare against the USB3 Gen2 spec's required return loss
   (-12 dB at 2.5 GHz typical) and insertion loss (-7 dB at 5 GHz
   typical).
4. Report a SI-002 diagnostic if the Touchstone data shows
   margin failure.

This is **read-only S-parameter reporting**, not S-parameter
convolution. The convolution math is HyperLynx / Sigrity / SIwave /
ADS work; Datum reads the published numbers.

**Encryption gate applies.** Vendor IBIS / Touchstone / SPICE files
may be encrypted (per Batch 1's Encrypted Content Handling Policy).
The Domain-6 rule engine MUST honour the encryption gate: when an
IBIS or Touchstone model is encrypted, Datum can read metadata
(model name, port count, frequency range) but cannot extract the
content. The length-match-tolerance refinement falls back to JESD
worst-case; the return-loss reporting becomes "vendor-encrypted
model attached; values not extractable; user must run external
solver with encrypted-model-aware credentials".

### Per-NetClass rule extensions

**Today (`ENGINE_SPEC.md` § 1.3 lines 382-391):**

```
pub struct NetClass {
    pub uuid: Uuid,
    pub name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
}
```

**Domain-6 proposed additions:**

```
pub struct NetClass {
    pub uuid: Uuid,
    pub name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
    // Domain 6 additions:
    pub length_match_target_nm: Option<i64>,            // class-level group target
    pub length_match_tolerance_nm: Option<i64>,         // class-level group tolerance
    pub length_match_group: Option<Uuid>,               // → board-level LengthMatchGroup
    pub diffpair_skew_tolerance_nm: Option<i64>,        // intra-pair skew
    pub diffpair_topology: Option<DiffpairTopology>,    // EdgeCoupled (v1) | BroadsideCoupled | Mixed
    pub phy_profile: Option<PhyProfile>,                // PHY-rule-pack pointer
}

pub enum DiffpairTopology {
    EdgeCoupled,         // v1 baseline
    BroadsideCoupled,    // v2 deferred
    Mixed,               // v2 deferred
}

pub enum PhyProfile {
    Custom,
    // First-cohort packs (free-source):
    Ddr3_1600, Ddr3_2133,
    Ddr4_2400, Ddr4_2666, Ddr4_2933, Ddr4_3200,
    Ddr5_4000, Ddr5_4800, Ddr5_5600, Ddr5_6400,
    Lpddr4_3200, Lpddr4_4266,
    Lpddr5_5500, Lpddr5_6400, Lpddr5x_8533,
    Usb2_Hs,
    Usb3_Gen1, Usb3_Gen2, Usb3_Gen2x2,
    PcieGen3, PcieGen4,
    Ethernet10BaseT, Ethernet100BaseTx, Ethernet1000BaseT,
    Ethernet2_5GBaseT, Ethernet5GBaseT, Ethernet10GBaseT,
    Ethernet10GBaseKr, Ethernet25GBaseKr,
    CanClassic, CanFd,
    Qspi, Ospi,
    // Fold-in packs (paywall-deferred):
    Hdmi20, Hdmi21,
    DisplayPort14, DisplayPort20,
    MipiDPhy15, MipiDPhy30,
    MipiCPhy20,
    MipiMPhy40, MipiMPhy50,
    PcieGen5, PcieGen6,
    Usb4Gen2, Usb4Gen3, Usb4V2,
    // Future:
    Ethernet40GBaseKr4, Ethernet100GBaseKr4, Ethernet400GBaseKr4,
    Ddr6_8800,    // sampling 2026; rule pack pending sample availability
    Lpddr6_10000, // sampling 2026; rule pack pending
}
```

**Per-Net additions:**

```
pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub class: Uuid,
    pub controlled_impedance: Option<ImpedanceSpec>,    // Batch 1
    // Domain 6 additions:
    pub length_match_membership: Option<LengthMatchMembership>,
    pub touchstone_reference: Option<Uuid>,             // → Part with attached Touchstone for this net
    pub power_distribution: Option<PdnSpec>,            // for power nets only
}

pub struct LengthMatchMembership {
    pub group: Uuid,                          // → Board.length_match_groups
    pub target_offset_nm: Option<i64>,        // per-net offset from group target (e.g. IBIS-derived per-pin compensation)
    pub tolerance_override_nm: Option<i64>,   // override of the group's default tolerance
}

pub struct PdnSpec {
    pub target_impedance_milliohms: f32,
    pub frequency_range_hz: (f32, f32),
    pub transient_current_ma: f32,
    pub allowed_ripple_mv: f32,
}
```

**Board-level addition:**

```
pub struct Board {
    // ... existing fields ...
    pub length_match_groups: HashMap<Uuid, LengthMatchGroup>,
}

pub struct LengthMatchGroup {
    pub uuid: Uuid,
    pub name: String,
    pub algorithm: LengthMatchAlgorithm,
    pub target_length_nm: Option<i64>,
    pub tolerance_nm: i64,
    pub source: LengthMatchSource,
}

pub enum LengthMatchAlgorithm {
    MatchToTarget,    // every net matches target_length_nm
    MatchToLongest,   // every net matches longest net
    MatchToPair,      // intra-pair only (P/N to each other)
    MatchToBus,       // hierarchical group-of-groups (DDR data byte lanes)
}

pub enum LengthMatchSource {
    UserAuthored,
    DerivedFromPhyProfile { phy: PhyProfile },
    DerivedFromIbis { part: Uuid, attachment: Uuid },
    Imported,         // round-tripped from KiCad / IPC-2581 etc.
}
```

**Rule type expansion (`ENGINE_SPEC.md` § 4 line 740-749):**

```
pub enum RuleType {
    ClearanceCopper,
    TrackWidth,
    ViaHole,
    Annular,
    HoleSize,
    SilkClearance,
    Connectivity,
    // Domain 6 promotions (was M5+ comment):
    Impedance,
    LengthMatch,
    DiffpairGap,
    DiffpairSkew,
    ReturnPathContinuity,    // new: return-path-cut DRC
    // Future (research-stage; not v1):
    // CrosstalkCoupling — deferred per § "Cross-talk / coupling DRC"
    // PdnImpedance — deferred per § "PDN target-impedance"
}
```

### High-speed-rule template packs (fold-in approach)

Rule packs ship as small JSON files in `pool/rule-packs/`. Pack
schema:

```json
{
  "schema_version": "1.0",
  "pack_id": "ddr4_3200_default",
  "phy_profile": "Ddr4_3200",
  "issued": "2026-04-15",
  "source_specs": [
    "JESD79-4D (October 2024)",
    "Micron TN-46-14 Rev D (DDR4 PCB Design)"
  ],
  "documentation": "Default rule pack for DDR4-3200 designs. Suitable for designs with controller-attached IBIS; refine using vendor IBIS for production designs.",
  "default_diff_pair": {
    "impedance_ohms": 100.0,
    "tolerance_pct": 10.0,
    "intra_pair_skew_nm": 100000.0
  },
  "default_single_ended": {
    "impedance_ohms": 40.0,
    "tolerance_pct": 10.0
  },
  "length_match_groups": [
    {
      "group_id": "data_byte_lane_0",
      "algorithm": "MatchToBus",
      "tolerance_nm": 127000.0,
      "members_pattern": "DQ[0-7]"
    },
    {
      "group_id": "command_address",
      "algorithm": "MatchToTarget",
      "tolerance_nm": 254000.0,
      "members_pattern": "ADDR.*|CMD.*|CTRL.*",
      "fly_by_topology": true
    }
  ],
  "decoupling_caps_per_vdd_pin": 1,
  "reference_layer_preference": "AdjacentGround",
  "ac_coupling_caps_required": false,
  "via_stub_max_nm": 254000.0
}
```

The pack is read by the rule engine when a NetClass carries
`phy_profile: Some(Ddr4_3200)`; the engine inflates the pack's
defaults into concrete `Rule` records scoped to NetClass nets. The
user can override per-NetClass; the pack is the **default**, not
the **mandate**.

**Pack distribution:** packs ship in the **default pool** (per the
existing pool layering scheme, `docs/POOL_ARCHITECTURE.md`). Users
can author custom packs in their personal pool; pool priority
governs which pack wins. Versioned pack revisions ship as
file-suffix variants (`ddr4_3200_default_v2.json`); the pack
schema includes a `pack_id` and `issued` date for the user-facing
catalog.

### Return-path continuity DRC

The standard SI failure mode is a high-speed trace whose reference
plane has a split. The rule:

```
For each track on a board layer L where:
  - the track's net has NetClass.phy_profile.is_high_speed() (i.e.,
    not None and not USB2_Hs / CanClassic / Qspi at low frequencies)
  - L is a signal layer

Determine the adjacent reference layer R (the closest power or
ground layer above OR below L per Stackup ordering).

For each segment of the track on L:
  - Compute the projection P of the segment onto R.
  - For each polygon hole / split / void on R:
    - If the polygon intersects P: emit ReturnPathContinuity finding.

Severity:
  - Error if the gap is wider than the segment's track width
  - Warning if the gap is narrower (signal still crosses but with
    increased return-path inductance)
```

**Geometry complexity.** The polygon intersection is on Datum's
existing zone / polygon primitives. The complications:
- **Plane stitching** (via fences across plane splits): the rule
  must look for stitch vias within a configurable distance of the
  segment crossing; if present, the return path has a bridge.
- **Anti-pads** (clearance holes around vias): the rule must
  ignore via anti-pads as long as the via stub is < segment
  width.
- **Multi-segment tracks**: the rule fires per segment; the
  diagnostic groups segments per track.

**Effort.** **~5 days** for the rule type implementation +
test suite + diagnostic-explanation surface.

**Differentiator.** No open-source EDA tool (KiCad 8, Eagle 9,
Horizon, LibrePCB, DipTrace, EasyEDA) implements this as a
first-class DRC rule. Altium has the **Plane Region with Discontinuity
under Track** rule; Cadence has **Reference Plane Cut**; Mentor has
**RefPlane Discontinuity**. Datum at parity with the commercial
tools, ahead of the open-source field.

### Cross-talk / coupling DRC (deferred)

**Why deferred.** The closed-form approximations exist (Wadell-style
coupled-stripline / coupled-microstrip) but require careful corner-
case handling that is hard to ship without a 2D field solver in the
loop. A "good-enough" rule that fires false positives or misses
real failures is worse than no rule at all.

**Recommended:** Reserve `RuleType::CrosstalkCoupling` as a
placeholder enum variant; defer implementation to post-v2 when the
field-solver question (third-party library? hand-rolled? subprocess
to external solver?) is re-evaluated. Surface this as a research
TODO.

### PDN target-impedance analysis (deferred)

**Why deferred.** Frequency-domain PDN-impedance analysis requires
either a 2D / 3D field solver or a circuit-simulator-based PDN
network calculation. Both are out of v1 / v2 Datum scope.

**Recommended:** Ship the PdnSpec data carrier (per § "Per-NetClass
rule extensions" above) so the user can author the design intent;
defer the analysis to external solver export. The
`export_si_artifact_bundle` MCP tool emits the PdnSpec as an
input to HyperLynx PI / SiWave-PI / Sigrity PowerSI / ADS PowerDC.

### Solver / simulator integration (subprocess-only)

Per the project's no-copyleft-linkage rule: any external solver is
invoked as a subprocess, never linked. Domain 6 inherits this from
Domain 2 (which established the pattern for ngspice). The Domain-6
SI/PI artifact export tools:

- `export_hyperlynx_hyp` — emit Mentor HyperLynx HYP file (board
  geometry + stackup material data + per-net controlled-impedance
  + length-match groups). HyperLynx HYP is a Mentor proprietary
  format but the schema is publicly documented.
- `export_sigrity_speed_xtractim` — emit Cadence Sigrity SPEED-
  XTRACT-IM project file. Sigrity SPEED-XTRACT-IM is the
  AC-extraction tool (impedance and crosstalk extraction). Schema
  is Cadence-proprietary; format is documented in Sigrity user
  guide.
- `export_si9000_board_profile` — emit Polar Si9000 board profile
  (the dominant 2D field solver in low-cost / mid-tier impedance
  control). Si9000 has a documented import format.
- `export_simbeor_project` — emit Simberian SimBeor project file
  for SI/PI simulation.
- `export_advanced_design_system_emx` — emit Keysight ADS EMX
  format (high-end RF/microwave SI/PI).

Each tool is a **file emitter**; Datum does not invoke any of these
tools. The user (or AI agent) runs the external solver out-of-band.
The Domain-6 batch ships the substrate plus the first two emitters
(HyperLynx + Si9000 — the most commonly-used pair for PCB SI work);
the rest ship on demand.

## EDA Tool Support Matrix

| Tool | Length-Match Groups | PHY-Profile Templates | Controlled Impedance Rule | Return-Path DRC | Diff-Pair Skew Rule | IBIS Attach | Touchstone Attach | PDN Sim |
|---|---|---|---|---|---|---|---|---|
| **Altium Designer** | Yes (xSignals) | ~25 templates (DDR/USB/PCIe/Ethernet/HDMI/DisplayPort/MIPI/CAN) | Yes (per-net) | Yes (Plane Region with Discontinuity) | Yes | Yes (Component IBIS attachment) | Yes | Add-on (PDN Analyzer) |
| **OrCAD Capture / Allegro PCB Editor** | Yes (ConstraintManager) | ~30 templates (TCL scripts) | Yes (per-net + per-class) | Yes (Reference Plane Cut) | Yes | Yes | Yes | Sigrity (separate product) |
| **Mentor PADS / Xpedition** | Yes | ~25 templates | Yes | Yes (RefPlane Discontinuity) | Yes | Yes (HyperLynx) | Yes (HyperLynx) | HyperLynx PI |
| **Cadence Sigrity** (separate product) | n/a (extraction) | n/a | n/a | n/a | n/a | Full IBIS-AMI | Full | PowerSI |
| **Mentor HyperLynx** (separate product) | n/a (extraction) | n/a | n/a | n/a | n/a | Full IBIS-AMI | Full | HyperLynx PI |
| **Ansys SIwave / SIwave-PI** | n/a (extraction) | n/a | n/a | n/a | n/a | Full IBIS | Full | SIwave-PI |
| **Keysight ADS EMX** | n/a (extraction) | n/a | n/a | n/a | n/a | Full IBIS-AMI | Full | ADS PowerSI |
| **Pulsonix** | Yes | ~10 templates | Yes | Yes | Yes | Yes (separate IBIS module) | Yes | Add-on |
| **KiCad 8.0+** | Yes (length-match group, KiCad 7+) | None | Yes (per-net Net.controlled_impedance, KiCad 8.0+) | No | Limited (skew between two nets) | Yes (KiCad 7+, no AMI) | No | Community plugin only |
| **Eagle 9 / Fusion Electronics** | Per-net (Fusion 2024+) | None | Limited | No | No | Datasheet attachment only | No | None |
| **Horizon EDA** | Yes (per-net) | None | Yes (per-net) | No | No | No | No | None |
| **LibrePCB** | No | None | No | No | No | No | No | None |
| **DipTrace** | Yes (group, basic) | None | Yes (per-net) | No | No | No | No | None |
| **EasyEDA / EasyEDA Pro** | Yes (group, basic) | None | Yes (per-net) | No | No | No | No | None |
| **Datum-current** | Per-net via existing serpentine routing kernel | None | Per-net carrier (Net.controlled_impedance, Batch 1) | No | No | Per-Part attachment surface (Batch 1, parser pending) | Per-Part attachment surface (Batch 1, parser pending) | None |
| **Datum-recommended (post-Domain-6)** | Yes (per-NetClass + per-Net + Board.length_match_groups) | First-cohort (DDR3/4/5, LPDDR4/5, USB2/3, PCIe Gen3/4, Ethernet 1G/10G/25G, CAN/CAN-FD, QSPI/OSPI) | Yes (RuleType::Impedance, IPC-2141A math basis, Stackup-Dk/Df-aware) | Yes (RuleType::ReturnPathContinuity) | Yes (RuleType::DiffpairSkew + RuleType::DiffpairGap) | Yes (Domain 2 contract, parser pending) | Yes (Domain 2 contract, parser pending) | Substrate (PdnSpec data carrier; analysis via export to external solvers) |

**Headline reading.** Datum's recommended Domain-6 substrate puts it
at parity with Altium / OrCAD / Mentor for the data-model and rule-
type coverage, ahead of the entire open-source field (KiCad / Eagle
/ Horizon / LibrePCB / DipTrace / EasyEDA), and pulls level with
mid-tier commercial (Pulsonix). The differentiator is the
**AI-explained tolerance derivation** (IBIS-attachment-aware) and
the **AI-readable rule packs** (JSON, git-diffable) — both of which
no incumbent does.

## Pending Exclusions (re-affirmed)

The Phase-1 audit's "Recommended low-priority / skip" list for
Domain 6 includes one explicit fold-in note:

> **HDMI / DisplayPort / MIPI layout templates — fold into the
> high-speed-rule deep-dive if it materialises; otherwise skip.**

**Re-affirmed disposition (this report).** **Fold-in candidates,
NOT hard exclusions.** The substrate (NetClass + PhyProfile +
LengthMatch + Impedance + DiffpairSkew + ReturnPathContinuity)
is shipped now; the per-PHY rule packs ship as add-on library
content. The HDMI / DisplayPort / MIPI packs are **`Deferred with
prerequisite`** because the underlying specs are paywalled; the
prerequisite is either (a) the relevant adopter / member agreement
is acquired, or (b) vendor-published per-IC layout guides are sourced.

The deep-dive identified additional candidates for skip / out-of-
scope status that the Phase-1 audit did not explicitly call out:

- **MIL-STD-461G** — defence-grade EMC qualification. Deep-dive
  recommends `Reference-only` as a declarable target; no
  validation. Same framing as DO-160G (audit's Domain 4 skip).
- **CISPR 14, CISPR 15** (household-appliances / lighting) — out
  of Datum's typical PCB-substrate scope. **Recommend `Out of
  scope`** for the consolidated post-Domain-8 ratification pass;
  the verticals these target (white goods, LED drivers) are
  served by Datum's `industry_vertical: Consumer` baseline + EN
  55014 (the harmonised-EU equivalent of CISPR 14/15) declared
  via `EmissionsStandard::En55014`. Don't carry CISPR 14/15 as a
  Datum-specific declaration.
- **JEDEC LPDDR6 / DDR6** — sampling 2026; rule packs deferred
  until volume samples ship and vendor controller layout guides
  are published. **Recommend `Deferred with prerequisite`** rather
  than skip.
- **PCIe Gen5 / Gen6 / Gen7 rule packs** — deferred per the
  PCI-SIG paywall. Substrate supports them; rule packs `Deferred
  with prerequisite`.

**No hidden cross-cutting value found** that would justify
re-opening any of the Phase-1 audit's other Domain-6 advisory
notes. The fold-in + paywall-deferred treatment is appropriate.

## User Pain Points & Wishlist Items

Distilled from EEVblog, Reddit r/PrintedCircuitBoard, the Altium /
KiCad community forums, the SignalIntegrityJournal blog comments,
the SI-LIST mailing list (the historical SI/PI mailing list, still
active in 2026), and Google Groups history (1998-2026 archives of
PCB-related newsgroups):

- **"Why does Altium hide my length-match tolerance behind
  xSignals?"** xSignals is Altium's grouping abstraction; multiple
  users complain that the rule values are buried in nested
  Properties dialogs. Datum opportunity: surface length-match
  tolerance and its derivation (which rule pack? which IBIS
  attachment? which manual override?) at the top of the AI
  conversational surface.

- **"KiCad 8 controlled impedance is per-net but no checker tells
  me my routed traces actually meet the impedance."** KiCad 8
  added `Net.controlled_impedance` (which Datum mirrors via its
  Batch 1 field) but the impedance is only reported in IPC-2581
  Rev C export — there's no in-engine DRC rule. Datum opportunity:
  ship `RuleType::Impedance` as a first-class DRC rule.

- **"Why is there no return-path continuity check in any open-
  source EDA tool?"** Recurring complaint on r/PrintedCircuitBoard.
  Users describe spending hours manually inspecting plane splits
  for SI failures; the answer in commercial-tool reviews is "use
  Altium / Cadence / Mentor". Datum opportunity: ship `RuleType::
  ReturnPathContinuity` as a first-class DRC rule — instant
  differentiator vs the open-source field.

- **"DDR layout is hopeless without a wizard."** Multiple complaints
  about authoring DDR length-match groups manually in KiCad. Datum
  opportunity: the PHY-profile template-pack pattern.

- **"I don't have HyperLynx; can I run any pre-layout SI?"** The
  cost-prohibitive nature of commercial SI/PI tools is the #1
  complaint from small-team designers. Datum's substrate-only
  position doesn't directly solve this, but the SI-artifact
  export to free / low-cost tools (LTspice via SPICE export,
  Polar Si9000 ~USD 2k as the dominant low-cost impedance solver,
  Saturn PCB Toolkit free for impedance / via stub calculations)
  gives those users a credible path.

- **"Why doesn't anyone tell me when my PHY isn't IBIS-modelled?"**
  Recurring complaint: a designer picks a PHY IC, attaches an
  IBIS model from the vendor, and discovers post-layout that the
  IBIS file was for a different revision of the IC. Datum
  opportunity: the AI surface can warn ("the attached IBIS for U4
  is for revision A; the BOM line specifies revision B; vendor
  app notes recommend revision-B IBIS for accurate timing").

- **"There's no way to tell which decoupling caps are
  'pre-decoupling' the IC vs 'bulk decoupling' the rail."** The
  rules are conventional (one ceramic per VDD pin within 5 mm; one
  bulk cap per 100 mm² of board area; etc.) but no tool surfaces
  the analysis automatically. Datum opportunity: a `RuleType::
  DecouplingCoverage` rule type (deferred to v2 — listed as a
  cross-cutting opportunity for the AI surface even if not a v1
  rule type).

- **"AI-explained design rules would be amazing."** Multiple
  Altium / KiCad / Cadence community comments about wishing the
  tool would explain *why* a rule is set the way it is, not just
  what the value is. Datum's AI-native positioning + JSON rule
  packs + IBIS-aware tolerance derivation directly serves this
  unmet need. **This is the largest single AI-surface opportunity
  in the deep-dive.**

- **"PDN analysis is a black box for non-experts."** PDN
  optimisation is genuinely a specialist skill; even mid-experience
  designers wave their hands. Datum's role in v1 is to capture
  the PdnSpec design intent and emit it for external solvers; the
  AI agent can explain "this rail has a 10 mΩ target impedance
  across 1 kHz – 100 MHz; the recommended decoupling network is
  …" by reading the PdnSpec and applying first-order PDN design
  rules.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

The following are the irreducible Domain-6 substrate Datum must ship
to be credible as a high-speed-design substrate:

1. **NetClass extension** — `length_match_target_nm`,
   `length_match_tolerance_nm`, `length_match_group`,
   `diffpair_skew_tolerance_nm`, `diffpair_topology`,
   `phy_profile`. **Effort: ~4 days.**

2. **Net extension** — `length_match_membership`,
   `touchstone_reference`, `power_distribution`. **Effort: ~2 days.**

3. **Board.length_match_groups + LengthMatchGroup struct** —
   board-level group registry. **Effort: ~1 day.**

4. **PhyProfile enum + first-cohort enum variants** — DDR3/4/5,
   LPDDR4/5, USB2/3, PCIe Gen3/4, Ethernet 1G/10G/25G,
   CAN/CAN-FD, QSPI/OSPI. **Effort: ~1 day** for the enum.

5. **Project.compliance.emc_posture** — extends Domain 4's
   `ProjectCompliance` block. **Effort: ~1 day** added on top of
   Domain 4 batch.

6. **RuleType expansion** — `Impedance`, `LengthMatch`,
   `DiffpairGap`, `DiffpairSkew`, `ReturnPathContinuity`. The
   five enum variants are added; the rule-engine implementations
   are listed below. **Effort: ~30 minutes for the enum;
   implementation per-rule.**

7. **`Impedance` rule implementation** — math basis IPC-2141A
   (Wheeler / Hammerstad-Jensen / IPC closed-form approximations
   for microstrip / stripline / coplanar). Consumes Stackup
   `dielectric_constant`, `loss_tangent`, `copper_weight_oz`,
   `roughness_um` (Batch 1). Diagnostic SI-001 fires when Stackup
   data unpopulated and falling back to default. **Effort: ~5 days.**

8. **`LengthMatch` rule implementation** — checks routed length
   against `length_match_target_nm` ± `length_match_tolerance_nm`
   for each net in a length-match group. Group-level grouping uses
   `Net.length_match_membership.group → Board.length_match_groups`.
   Per-net `target_offset_nm` from membership applied. **Effort:
   ~3 days.**

9. **`DiffpairGap` rule implementation** — validates the gap
   between P/N members of a differential pair against
   `NetClass.diffpair_gap`. Pair detection uses
   `infer_diffpair_from_pinnames` (Domain 2 MCP tool, Batch 1)
   plus `Net.name` suffix conventions (`_P`/`_N`, `_+`/`_-`).
   **Effort: ~2 days.**

10. **`DiffpairSkew` rule implementation** — validates intra-pair
    skew against `NetClass.diffpair_skew_tolerance_nm`. **Effort:
    ~2 days.**

11. **`ReturnPathContinuity` rule implementation** — per § "Return-
    path continuity DRC" above. **Effort: ~5 days.**

12. **First-cohort rule packs** — JSON files for DDR3/4/5,
    LPDDR4/5, USB2/3, PCIe Gen3/4, Ethernet 1G/10G/25G,
    CAN/CAN-FD, QSPI/OSPI. Packs cite source specs. **Effort:
    ~1-3 days per pack; ~20 packs in first cohort = ~30 days,
    parallelisable.**

13. **MCP tool surface** — `set_length_match_group`,
    `add_net_to_length_match_group`, `remove_net_from_length_match_group`,
    `set_phy_profile`, `apply_rule_pack`, `list_available_rule_packs`,
    `extract_phy_profile_explanation`, `calculate_microstrip_impedance`,
    `calculate_stripline_impedance`, `validate_controlled_impedance_audit`,
    `validate_length_match_audit`, `validate_return_path_audit`. **Effort:
    ~3 days for stubs + ~5 days for implementations.**

**Total hard-requirement effort:** ~70-80 person-days. Sequenceable
in two batches: (a) schema bedrock + RuleType expansion + first
two rules (Impedance + LengthMatch), ~25 days; (b) DiffpairGap +
DiffpairSkew + ReturnPathContinuity + rule packs + MCP, ~50 days.

### Should Support (post-M7)

1. **SI-artifact export tools** — `export_hyperlynx_hyp`,
   `export_si9000_board_profile`, `export_simbeor_project`. **Effort:
   ~5-7 days each = ~20 days total.** Recommendation: ship
   `export_hyperlynx_hyp` and `export_si9000_board_profile` first
   (the most common low-mid-tier solver pair).

2. **Second-cohort rule packs** — HDMI 2.0/2.1, DisplayPort 1.4/2.x,
   MIPI D-PHY/C-PHY/M-PHY, PCIe Gen5/Gen6, USB4 (paywall-deferred,
   per `Deferred with prerequisite`). Effort: ~2-3 days per pack
   when prerequisite (paywall access or vendor-app-note source)
   is available.

3. **PdnSpec authoring + export** — substrate ships with the
   first batch; the dedicated authoring UI / MCP tool surface
   (`set_pdn_spec`, `get_pdn_spec`, `export_pdn_artifact_bundle`)
   ships post-M7. **Effort: ~5 days.**

4. **AI-surface "explain this length-match tolerance" tool** —
   `explain_length_match_tolerance(net_uuid)` returns a structured
   explanation citing the rule pack, IBIS attachment (if any),
   per-pin propagation delay, and JESD worst-case fallback if
   relevant. **Effort: ~3 days.** This is the AI-native
   differentiator.

5. **AI-surface "explain this impedance failure" tool** —
   `explain_impedance_violation(track_uuid)` returns the
   calculated impedance, the target, the formula used, the
   Stackup material data consumed, and the recommended remedy
   (widen track? change layer? populate Stackup material data?).
   **Effort: ~3 days.**

### On-Demand Only

1. **JTAG chain editor** — out of Domain-6 scope; cite as a
   future schematic-editor work item.
2. **Per-IC layout-guide PDF parser** — not feasible; PDF parsing
   is unreliable. Defer indefinitely.
3. **Ethernet 40G/100G/400G rule packs** — defer until user
   demand is signalled; the rule packs are content-only when
   demand surfaces.
4. **MIL-STD-461G as a declarable target** — defer to user
   request; the substrate supports it but the default per-vertical
   mapping does not auto-populate.

### Out of Scope (recommend formal exclusion)

1. **2D / 3D field solver math** — out of Datum-engine scope; the
   path is subprocess-only export to external solvers.
2. **S-parameter convolution** — same.
3. **Transmit-eye / receive-jitter compliance simulation** — same.
4. **Emissions-margin estimation** — fundamentally test-lab work.
5. **CISPR 14 / CISPR 15** (household appliances / lighting) —
   served by `industry_vertical: Consumer` baseline.
6. **Thunderbolt as a separate PHY profile** — Thunderbolt 4 / 5
   PHYs are USB4-equivalent; Datum's `Usb4V2` profile covers them.
7. **TR 62396 (single-event effects from cosmic rays)** — relevant
   only to satellite / aerospace; out of Datum's vertical scope
   per Domain 4 skip.
8. **IPC-7095 BGA design (thermal/SI interaction)** — covered as
   an IPC document; cross-ref existing IPC research; no Domain-6
   specific work.

For each "must support" and "should support":

#### NetClass / Net struct changes required

Per § "Per-NetClass rule extensions" above. Total: ~6 new fields
on NetClass, ~3 new fields on Net, ~1 new collection on Board, ~5
new structs (`LengthMatchMembership`, `LengthMatchGroup`,
`PdnSpec`, `LengthMatchAlgorithm`, `LengthMatchSource`), ~3 new
enums (`DiffpairTopology`, `PhyProfile`, `LengthMatchAlgorithm`).

#### Project.compliance block changes required (coordinated with Domains 4/5)

Per § "Industry-vertical drives EMC defaults" above. Adds
`emc_posture: EmcPosture` sub-block to Domain 4's `ProjectCompliance`.
EmcPosture carries `default_emc_class`, `requires_*_audit` booleans,
`phy_profiles_in_use` denormalised cache, `target_emissions_standards`.

#### Rule engine changes required

Per § "Per-NetClass rule extensions" above. RuleType enum gains 5
variants (Impedance, LengthMatch, DiffpairGap, DiffpairSkew,
ReturnPathContinuity); each gains a per-variant rule-engine
implementation.

#### MCP API additions required

Per § "Hard Requirements" item 13 above. ~12 new tools in a new
"EMC & SI Tools" section of `MCP_API_SPEC.md`, plus extension of
existing `set_net_class` to accept the new NetClass fields.

#### Minimum-viable vs full implementation

**Minimum-viable (~25 days):** NetClass + Net schema additions;
PhyProfile enum; RuleType expansion; Impedance + LengthMatch rule
implementations; first three rule packs (DDR4-3200, USB3-Gen2,
PCIe-Gen3); MCP tool stubs.

**Full (~70-80 days):** Above + DiffpairGap + DiffpairSkew +
ReturnPathContinuity rule implementations + ~17 additional rule
packs + MCP tool implementations + PdnSpec data carrier + first
two SI-artifact export tools.

#### Partner / library dependencies and license risks

- **`rf` crate** (BSD, hand-rolled Rust transmission-line math) — a
  candidate for the impedance-calculation math basis. **Verified
  BSD; safe to integrate.** The crate covers microstrip / stripline
  / coplanar / coupled-line / waveguide closed-form approximations.
  Alternative: hand-roll the math (formulae are ~50 lines each,
  well-documented in textbooks, not subject to copyright). Hand-
  rolled is recommended for Datum to avoid any third-party
  dependency drift.
- **No GPL-class libraries** required. ngspice (GPL-3) is invoked
  as subprocess only (via Domain 2's `validate_spice` tool); no
  Domain-6 work re-introduces GPL linkage risk.
- **Touchstone parser** — already in scope for Domain 2 (Batch 1
  contract surface, parser pending). scikit-rf (BSD, Python) is
  the reference for cross-checking; for Datum's Rust-side parser,
  hand-rolled is recommended (the format is ~200 lines of Rust).
- **IBIS parser** — already in scope for Domain 2 (Batch 1
  contract surface, parser pending). `ibischk` (BSD-style C99) is
  the reference; for Datum, FFI to `ibischk` for validation +
  hand-rolled Rust for read-only metadata extraction is the
  recommended pattern.

#### Effort estimate

**Total Domain-6 effort:** ~70-80 person-days for full
implementation; ~25 days for minimum-viable. Sequenceable in two
batches:
- **Batch 6.0 (substrate bedrock):** ~25 days. Schema + RuleType
  expansion + Impedance + LengthMatch + first three rule packs.
- **Batch 6.1 (full rule coverage):** ~50 days. DiffpairGap +
  DiffpairSkew + ReturnPathContinuity + remaining rule packs +
  MCP tools + PdnSpec.

### Datum Differentiators

- **AI-explained length-match tolerance derivation.** No incumbent
  surfaces "this DDR4 length-match tolerance was derived from
  JESD79-4 worst-case; attaching the controller IBIS would refine
  it to vendor-specific values" in conversational form. Datum's
  AI surface, with the rule-pack + IBIS-attachment metadata
  available, can produce this explanation as a first-class
  capability.

- **MCP-queryable PHY profile.** `get_net_class(net_class_uuid)`
  returns the `phy_profile`; `apply_rule_pack(net_class_uuid,
  pack_id)` updates the rule-pack reference; the AI agent can
  iterate on rule choices directly. No incumbent exposes this as
  a programmable surface.

- **Deterministic SI-artifact export.** Datum's deterministic JSON
  serialisation means the HyperLynx HYP export, IPC-2581 Rev C
  `<ImpedancesProperties>`, Si9000 board profile, etc. all
  produce byte-identical output from byte-identical project state.
  This is audit-trail-relevant for medical / automotive / defence
  workflows where the SI-artifact must be reproducible from the
  archived project.

- **Return-path-DRC explainability.** When a `ReturnPathContinuity`
  finding fires, the diagnostic includes the offending plane-split
  polygon, the segment that crosses it, the recommended remedy
  (re-route via a different layer, add a stitch via, fill the
  split with copper, etc.), and the AI surface can elaborate
  ("the SI failure mode here is: at 5 GHz the displacement
  current return path detours by 8 mm, increasing inductance by
  approximately 5 nH and producing a common-mode noise spike
  at the rising edge"). No incumbent does this conversationally.

- **AI-readable, git-diffable rule packs.** Altium binary `.RUL`
  files, Cadence TCL scripts, Mentor binary templates are not
  human-diffable. Datum's JSON rule packs are fully diffable, AI-
  readable, and version-controlled in the pool. A team can `git log`
  their rule-pack changes; the AI agent can answer "what changed
  in our DDR4 rule pack between v1 and v2".

- **Substrate-vs-certification framing.** Datum honestly says
  "your project's layout-rule substrate is consistent with FCC
  Part 15 Class B intent; your test lab certifies the result".
  No incumbent makes this distinction; users either get over-
  promised compliance claims (which they then have to ignore) or
  under-promised silence (which leaves them unsure what the tool
  does).

### Recommended Spec Edits

Concrete file:line edits for the user to review. Pattern follows
Standards Audit Batch 1, Batch 2 (Domain 3 & Domain 2), Batch 3
(Domain 4), and Batch 4 (Domain 5).

Claude is in research-only mode per the project's
`feedback_research_only_mode` rule; these recommendations are NOT
to be applied by the agent. The user will review, prioritise, and
apply via the standard spec-edit process.

| # | Source | Target file | Substance |
|---|--------|-------------|-----------|
| **Pass 0 — `STANDARDS_COMPLIANCE_SPEC.md` disposition refresh** ||||
| D6-0a | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.6 | Domain 6 dispositions refreshed: length-match / diff-pair / impedance rule foundations promoted from `Planned` to `Planned (contract surface defined; NetClass extension + RuleType expansion in ENGINE_SPEC.md § 1.3 + § 4)` once Pass 1 lands; `Stackup material properties` confirmed `Implemented` (delivered by Batch 1); product-certification standards (FCC / CISPR / EN / IEC 61000 / IEC 60601-1-2 / DO-160) confirmed `Reference-only` with substrate paragraph; PHY-specific rule families (USB / PCIe Gen3-4 / DDR / Ethernet / CAN / CAN-FD / QSPI / OSPI) promoted from `Deferred with prerequisite` to `Planned`; HDMI / DisplayPort / MIPI / PCIe Gen5+ / USB4 retained `Deferred with prerequisite` with paywall rationale; cross-talk / coupling DRC retained `Reference-only`; PDN target-impedance analysis retained `Reference-only` (substrate `Planned` for PdnSpec data carrier) |
| D6-0b | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 7 (Project-Level Compliance Metadata) | Section expanded to enumerate the EMC-posture fields (default_emc_class, requires_controlled_impedance_audit, requires_length_match_audit, requires_return_path_audit, requires_decoupling_audit, phy_profiles_in_use, target_emissions_standards); cross-references to ENGINE_SPEC.md § 1.x additions; nests inside Domain 4's `ProjectCompliance` block parallel to Domain 5's `materials_posture` |
| D6-0c | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` new § 7.3 | New subsection "EMC Substrate-vs-Certification Framing" — formal statement that Datum is the SI/PI rules substrate (controlled-impedance, length-match, diff-pair, return-path-continuity audits) and NOT the EMC-emissions / immunity certifying authority; documents the per-vertical default-EMC-class derivation; documents the AI-surface guard-rail that prohibits emissions-compliance claims; documents the SI-artifact-export-only path for external-solver integration |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** ||||
| D6-1 | this report | `specs/ENGINE_SPEC.md` § 1.1a (Shared Enums) | New shared types: `PhyProfile`, `DiffpairTopology`, `LengthMatchAlgorithm`, `LengthMatchSource`, `EmissionsStandard`, `EmcClass`, `EmcPosture`. The `PhyProfile` enum carries first-cohort variants (DDR3/4/5, LPDDR4/5, USB2/3, PCIe Gen3/4, Ethernet 1G-25G, CAN/CAN-FD, QSPI/OSPI), fold-in variants (HDMI / DisplayPort / MIPI / PCIe Gen5+ / USB4), and `Custom` |
| D6-2 | this report | `specs/ENGINE_SPEC.md` § 1.3 (Board Types) | Extend `NetClass` (currently lines 382-391) with: `length_match_target_nm: Option<i64>`, `length_match_tolerance_nm: Option<i64>`, `length_match_group: Option<Uuid>`, `diffpair_skew_tolerance_nm: Option<i64>`, `diffpair_topology: Option<DiffpairTopology>`, `phy_profile: Option<PhyProfile>`. Extend `Net` (currently lines 366-371) with: `length_match_membership: Option<LengthMatchMembership>`, `touchstone_reference: Option<Uuid>`, `power_distribution: Option<PdnSpec>`. New types: `LengthMatchMembership`, `LengthMatchGroup`, `PdnSpec`. Extend `Board` with `length_match_groups: HashMap<Uuid, LengthMatchGroup>` |
| D6-3 | this report | `specs/ENGINE_SPEC.md` § 1.x (Project Type — added by Domain 4's Batch 3) | Extend `ProjectCompliance` (Domain 4) with `emc_posture: EmcPosture` field. Coordinated with Domains 4 & 5 — must land after or with Domain 4's Batch 3 (parallel sibling to Domain 5's `materials_posture` field) |
| D6-4 | this report | `specs/ENGINE_SPEC.md` § 4 (Rule Types) | Promote the M5+ deferred comment at line 748 to first-class enum variants. `RuleType` enum gains: `Impedance`, `LengthMatch`, `DiffpairGap`, `DiffpairSkew`, `ReturnPathContinuity`. Reserved enum variants (commented `// research-stage`): `CrosstalkCoupling`, `PdnImpedance` |
| D6-5 | this report | `specs/ENGINE_SPEC.md` § 3 (Operations) | New operations: `SetPhyProfile`, `SetLengthMatchTarget`, `SetLengthMatchTolerance`, `CreateLengthMatchGroup`, `DeleteLengthMatchGroup`, `AddNetToLengthMatchGroup`, `RemoveNetFromLengthMatchGroup`, `SetNetTouchstoneReference`, `SetNetPdnSpec`, `SetDiffpairSkewTolerance`, `SetDiffpairTopology`, `SetEmcPosture`, `SetTargetEmissionsStandard`, `ApplyRulePack` — each with `inverse()` for undo |
| **Pass 2 — pool, persistence, library** ||||
| D6-6 | this report | `docs/POOL_ARCHITECTURE.md` § 2 | New pool entity type: `RulePack`. Stored in `pool/rule-packs/` as JSON files. Pool index gains a `rule_packs` SQL table with columns: `pack_id`, `phy_profile`, `schema_version`, `issued`, `source_specs`, `path`. Rule packs are pool entries; per-pool layering / priority rules apply (user packs override default-pool packs) |
| D6-7 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.x (board persistence) | Per-Board native persistence gains `"length_match_groups": [...]` collection serialising `Board.length_match_groups`. NetClass and Net serialisation extended with the new optional fields (deserialise as `None` for pre-Domain-6 projects) |
| D6-8 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.1 (project.json) | `project.json` `compliance` block (Domain 4) gains `"emc_posture": { ... }` sub-block; required-present with default values for new projects; existing projects deserialise with defaults (default_emc_class derived from industry_vertical; requires_*_audit all false; phy_profiles_in_use empty; target_emissions_standards empty) |
| **Pass 3 — `specs/MCP_API_SPEC.md` (EMC & SI Tools section)** ||||
| D6-9 | this report | `specs/MCP_API_SPEC.md` new "EMC & SI Tools" section | Section header + per-tool stubs for: `set_phy_profile`, `get_phy_profile`, `apply_rule_pack`, `list_available_rule_packs`, `extract_phy_profile_explanation`, `set_length_match_target`, `set_length_match_tolerance`, `create_length_match_group`, `delete_length_match_group`, `add_net_to_length_match_group`, `remove_net_from_length_match_group`, `get_length_match_group_members`, `set_diffpair_skew_tolerance`, `set_diffpair_topology`, `set_net_touchstone_reference`, `set_pdn_spec`, `get_pdn_spec`, `calculate_microstrip_impedance`, `calculate_stripline_impedance`, `calculate_coplanar_impedance`, `validate_controlled_impedance_audit`, `validate_length_match_audit`, `validate_return_path_audit`, `validate_diffpair_skew_audit`, `validate_diffpair_gap_audit`, `explain_length_match_tolerance`, `explain_impedance_violation`, `explain_return_path_violation`, `set_emc_posture`, `get_emc_posture`, `set_target_emissions_standard`. ~30 new tool stubs |
| D6-10 | this report | `specs/MCP_API_SPEC.md` existing `set_net_class` tool (line 1093) | Extend Input schema with: `length_match_target` (int|null), `length_match_tolerance` (int|null), `length_match_group` (uuid|null), `diffpair_skew_tolerance` (int|null), `diffpair_topology` (string|null — `"EdgeCoupled"` | `"BroadsideCoupled"` | `"Mixed"`), `phy_profile` (string|null — see PhyProfile enum) |
| D6-11 | this report | `specs/MCP_API_SPEC.md` new "SI Artifact Export Tools" section | Section header + per-tool stubs for: `export_hyperlynx_hyp`, `export_si9000_board_profile`, `export_simbeor_project`, `export_sigrity_speed_xtractim`, `export_advanced_design_system_emx`. All are file-emitters; no external-solver invocation. Status: stubs in `Should Support` cohort; implementation post-M7 |
| **Pass 4 — `specs/IMPORT_SPEC.md`** ||||
| D6-12 | this report | `specs/IMPORT_SPEC.md` § 3 (KiCad) | KiCad 8+ length-match-group import: when `*.kicad_pcb` carries length-tuning groups, import as `Board.length_match_groups` entries with `LengthMatchSource::Imported`; per-net `Net.length_match_membership` populated. KiCad 7 `*.kicad_pcb` length-tuning is per-net only; import as ungrouped per-net membership. KiCad 8.0+ `Net.controlled_impedance` import already covered by Batch 1 |
| D6-13 | this report | `specs/IMPORT_SPEC.md` new § (IPC-2581 Rev C high-speed metadata) | Specifies IPC-2581 Rev C `<ImpedancesProperties>` import (already cross-referenced from Batch 1's IPC-2581 export work); per-net controlled-impedance specifications populate `Net.controlled_impedance`; differential-pair identification populates `Net.length_match_membership` heuristically via P/N suffix conventions; document the lossy-import assumptions |
| **Pass 5 — architecture & guidance docs** ||||
| D6-14 | this report | `docs/STANDARDS_AUDIT_BATCH_5_GUIDANCE.md` (NEW) | Batch-5 bridging doc following the Batch-1 / Batch-2 / Batch-3 / Batch-4 pattern (must-land vs deferred, apply order, dependence on Domain 4's Batch 3 `ProjectCompliance` and Domain 5's Batch 4 `materials_posture` landing first, Pass 0 disposition refresh, Cross-Spec Update Rule compliance, advisory-exclusion fold-in framing). Recommended split: Batch 5.0 = Pass 0 + Pass 1 + Pass 2 + Pass 3 substrate tools; Batch 5.1 = Pass 4 (import) + Pass 5 (guidance + roadmap docs) + first-cohort rule packs; Batch 5.2 = SI-artifact export tools (post-M7) |
| D6-15 | this report | `docs/INTEROP_SCOPE.md` | Add "EMC & Signal Integrity (research-staged)" section: NetClass + Net + Board.length_match_groups substrate; PhyProfile + first-cohort rule packs; Impedance + LengthMatch + DiffpairGap + DiffpairSkew + ReturnPathContinuity rule types; PdnSpec data carrier; SI-artifact export tools (HyperLynx HYP, Si9000, SimBeor, Sigrity SPEED-XTRACT-IM, ADS EMX); substrate-vs-certification framing; subprocess-only rule for external-solver integration; paywall-deferred fold-in candidate list (HDMI / DisplayPort / MIPI / PCIe Gen5+ / USB4) |

**Total recommended spec edits:** **15** (3 disposition refreshes,
5 schema bedrock, 3 pool/native/library persistence, 3 MCP, 1
import. Pass 5 is 1 doc edit, but BATCH-5 GUIDANCE doc is a new
file, so total file-touching edits = 15.)

**Comparable to Batch 4's count (18 edits)** — Domain 6 is
slightly lighter than Domain 5 because much of the schema work is
on existing structs (NetClass / Net / Board / RuleType) rather
than introducing new top-level types. The user can split into
multiple PRs if the batch is judged too large for a single review
pass; suggested split is **"Pass 0 + Pass 1 + Pass 2"** as Batch
5.0 (the disposition + schema + pool work) and **"Pass 3 + Pass
4 + Pass 5"** as Batch 5.1 (the MCP, import, and guidance work).

**Dependency note.** D6-3 (extending `ProjectCompliance` with
`emc_posture`) requires Domain 4's Batch 3 to have landed first,
since Domain 4 introduces the `ProjectCompliance` struct. If
Batch 3 has not landed when Batch 5 is sequenced, the Domain-6
work either waits on Batch 3 or carries the `ProjectCompliance`
introduction itself (in which case Batch 3 will need to coordinate
when it lands). D6-3's `emc_posture` is parallel to Domain 5's
`materials_posture` (Batch 4); coordinated landing of all three
(Batch 3 + Batch 4 + Batch 5) is the cleanest sequence.

**Rule-pack landing cadence.** Rule packs are pool-content, not
spec changes. After Batch 5.0 lands the schema and Batch 5.1 lands
the engine, rule packs ship on a rolling basis with no PR
ceremony — each pack is a single JSON file in
`pool/rule-packs/`. The first-cohort packs (~20 packs) are
~30 person-days of authoring work parallelisable across the team.

## Cross-Domain Insights to Thread Forward

### To Domain 7 (PLM & lifecycle integration)

- **PHY IC vendor lookup is a PLM concern.** When a NetClass
  carries `phy_profile: Some(Ddr4_3200)`, the implicit assumption
  is that the design has a DDR4 controller IC. Domain 7's PLM
  integration should support filtering/finding parts that match
  the PHY-profile IC role (DDR4 controller, USB3 PHY, PCIe Gen4
  PHY) so the user can quickly identify the upstream-IC for IBIS
  attachment. The `lookup_part_octopart` (Batch 1) tool's
  parametric search already supports this in principle; Domain 7
  should specify the canonical PHY-role taxonomy.

- **Vendor IBIS / Touchstone refresh is a supply-chain refresh
  event.** The `refresh_supply_chain(part_uuid)` tool (Batch 1)
  populates `Part.supply_chain_offers`; the same refresh should
  check whether a newer IBIS / Touchstone has been published by
  the vendor and offer to update the attachment. Domain 7's PLM
  deep-dive should specify the consolidated refresh field map
  (Domain 2 `behavioural_models` fields, Domain 5 `compliance.*`
  fields, Domain 7 `supply_chain_offers` / `lifecycle` fields).

- **PHY-profile assignment is a project-classification attribute.**
  PLM systems may want to filter projects by PHY profiles in use
  (find all projects with DDR5 designs; find all projects with
  PCIe Gen4 designs) for resource planning. The
  `Project.compliance.emc_posture.phy_profiles_in_use` denormalised
  cache supports this.

- **Rule packs are pool content; pool layering applies.** Domain
  7's pool / vault discussion should include rule packs as a
  first-class pool entity type; user-authored packs override
  default-pool packs per the existing pool layering rules.
  Approval workflow (vault check-in / check-out) for custom
  rule-pack edits is a Domain-7 + Domain-8 cross-cutting concern.

### To Domain 8 (process & quality)

- **Length-match-group authoring is an authored op captured in
  audit-trail.** Each `CreateLengthMatchGroup` /
  `AddNetToLengthMatchGroup` / `RemoveNetFromLengthMatchGroup`
  operation is captured in the transaction log with full undo.
  The exported audit log surfaces the length-match-group authoring
  history per-project — relevant for design-review workflows
  ("when did the user assign Net X to length-match group Y?").

- **PHY-profile assignment is an authored op.** Each
  `SetPhyProfile(net_class_uuid, phy_profile)` operation is
  captured. The audit-log entry should include the profile name
  and the rule pack that applies (so the audit trail records both
  "the user picked DDR4-3200" and "the engine applied
  ddr4_3200_default_v1.json").

- **EMC waivers require sign-off.** When a `LengthMatchAudit` /
  `ImpedanceAudit` / `ReturnPathContinuityAudit` finding fires
  but the user waives it (per the existing CheckWaiver mechanism),
  the waiver is captured in the audit log with the user identity
  and rationale. Domain 8's audit-log export must surface SI/PI
  waivers as a distinct category for compliance review (the SI/PI
  waivers may need separate sign-off by an SI/PI engineer rather
  than the design lead).

- **Rule-pack version pinning is auditable.** When a rule pack is
  applied to a NetClass via `apply_rule_pack(net_class_uuid,
  pack_id)`, the pack ID and the pack's `issued` date are
  captured. The audit log surfaces "DDR4 NetClass uses rule pack
  ddr4_3200_default v1.0 (issued 2026-04-15)" — relevant for
  reproducibility (a v2.0 pack with refined tolerances should not
  silently change historical audit findings).

- **SI-artifact export is reproducible audit evidence.** Datum's
  deterministic JSON serialisation means the HyperLynx HYP /
  Si9000 / IPC-2581 Rev C `<ImpedancesProperties>` exports are
  byte-identical from byte-identical project state. Domain-8
  audit-trail-relevant: an auditor can verify SI-artifact
  integrity by re-export-and-diff. Particularly important for
  medical / automotive / defence workflows where the SI-artifact
  is part of the design dossier.

- **AI-explanation outputs are not authoritative records.** When
  the AI agent emits "the length-match tolerance is 5 mil because
  IBIS attachment X says R_pkg=2.5 nH", that text is a
  **convenience explanation**, not an authoritative engineering
  record. The authoritative record is the rule-pack JSON + the
  IBIS attachment + the rule-engine output. Domain 8's audit-log
  treatment of AI-explanation text should treat it as
  informational metadata, not as audit-trail evidence. (Strong
  guard-rail: AI agents are observers + assistants, not authority.)

### Cross-domain coordination summary

| Concern | Owner | Cross-cuts |
|---|---|---|
| `NetClass.phy_profile` + first-cohort rule packs | Domain 6 | Domain 7 (PLM filters by PHY-profile usage) |
| `Project.compliance.emc_posture` | Domain 6 | nested inside Domain 4's `ProjectCompliance`, sibling to Domain 5's `materials_posture` |
| `EmissionsStandard` / `EmcClass` enums | Domain 6 | Domain 4 (per-vertical default mapping) |
| `industry_vertical`-driven default EMC-class | Domain 4 (introduced); Domain 6 (consumes for default mapping) | Domain 5 (halogen-free flag affects laminate Dk/Df hint), Domain 7 (PLM routing by EMC class) |
| Stackup `dielectric_constant` / `loss_tangent` source-of-truth | Domain 5 (substrate framing); Batch 1 (delivered) | Domain 6 (Impedance rule consumes; SI-001 diagnostic) |
| `Part.behavioural_models` (IBIS / Touchstone) consumption | Domain 2 (Batch 1) | Domain 6 (length-match tolerance derivation; return-loss reporting) |
| Encrypted Content Handling Policy | Domain 2 (Batch 1) | Domain 6 (encrypted IBIS / Touchstone fall back to JESD worst-case + diagnostic) |
| Length-match-group + PHY-profile authoring ops | Domain 6 | Domain 8 (audit-trail capture) |
| Rule-pack version pinning | Domain 6 | Domain 8 (audit-trail; reproducibility) |
| SI-artifact export tools (subprocess-only) | Domain 6 | Domain 8 (deterministic export = reproducible audit evidence) |
| AI-explanation surface | Domain 6 (introduced); Domain 8 (audit-treatment-of) | Cross-cutting AI guard-rail |

## Sources

### Primary regulatory and standards references

- [Title 47 CFR Part 15](https://www.ecfr.gov/current/title-47/chapter-I/subchapter-A/part-15) — *Radio Frequency Devices*. US Code of Federal Regulations; **free** at ecfr.gov. Provides FCC Part 15 emissions limits and DoC / certification framework.
- [ANSI C63.4](https://webstore.ansi.org/standards/ansi/ansic63204020) — *American National Standard for Methods of Measurement of Radio-Noise Emissions from Low-Voltage Electrical and Electronic Equipment*. Test methodology referenced by FCC Part 15. Paywalled, USD 200.
- [CISPR 32:2015+A2:2024](https://webstore.iec.ch/publication/22054) — *Electromagnetic compatibility of multimedia equipment — Emission requirements*. Paywalled, ~CHF 540 from IEC Webstore.
- [CISPR 11:2024](https://webstore.iec.ch/publication/61090) — *Industrial, scientific and medical equipment — Radio-frequency disturbance characteristics*. Paywalled, ~CHF 360.
- [CISPR 25:2021](https://webstore.iec.ch/publication/64645) — *Vehicles, boats and internal combustion engines — Radio disturbance characteristics — Limits and methods of measurement for the protection of on-board receivers*. Paywalled, ~CHF 360.
- [EN 55032:2015](https://standards.cencenelec.eu/dyn/www/f?p=205:110:0::::FSP_PROJECT,FSP_ORG_ID:55687,1258544) — EU equivalent of CISPR 32. Paywalled at national standards bodies.
- [EN 55035:2017+A11:2020](https://standards.cencenelec.eu/dyn/www/f?p=205:110:0::::FSP_PROJECT,FSP_ORG_ID:60067,1258544) — EU equivalent of CISPR 35 (immunity). Paywalled.
- [IEC 61000-4 series](https://webstore.iec.ch/searchform&q=61000-4&page=1) — *EMC — Part 4: Testing and measurement techniques*. ~40 sub-parts. Paywalled, CHF 200-400 each.
- [IEC 61000-6-1:2019](https://webstore.iec.ch/publication/29915) — Generic immunity for residential / commercial environments. Paywalled.
- [IEC 61000-6-2:2016](https://webstore.iec.ch/publication/26021) — Generic immunity for industrial environments. Paywalled.
- [IEC 61000-6-3:2020](https://webstore.iec.ch/publication/65966) — Generic emissions for residential / commercial environments. Paywalled.
- [IEC 61000-6-4:2018](https://webstore.iec.ch/publication/29918) — Generic emissions for industrial environments. Paywalled.
- [IEC 60601-1-2:2014+A1:2020](https://webstore.iec.ch/publication/67007) — *Medical electrical equipment — Part 1-2: General requirements for basic safety and essential performance — Collateral Standard: Electromagnetic disturbances*. Paywalled, ~CHF 350.
- [RTCA DO-160G](https://www.rtca.org/content/standards-guidance-materials) — *Environmental Conditions and Test Procedures for Airborne Equipment*. Paywalled, USD 250.
- [MIL-STD-461G](https://everyspec.com/MIL-STD/MIL-STD-0300-0499/MIL-STD-461G_53571/) — *Requirements for the Control of Electromagnetic Interference Characteristics of Subsystems and Equipment*. **Free** at everyspec.com.
- [IEEE 802.3-2022](https://standards.ieee.org/ieee/802.3/10422/) — *IEEE Standard for Ethernet*. **Free under IEEE Get program**.
- [USB 2.0 / 3.x / 4 Specifications](https://www.usb.org/documents) — USB-IF spec set. **Free with click-through** at usb.org.
- [USB Type-C Specification Rev 2.3](https://www.usb.org/document-library/usb-type-c-cable-and-connector-specification-release-23) — **Free** at usb.org.
- [PCI Express Base Specification Rev 4.0 / 5.0](https://pcisig.com/specifications) — **Member-only**; PCI-SIG membership USD 4,000-12,000/year.
- [HDMI Specification Version 2.1a](https://www.hdmi.org/spec/index) — **Adopter-only**; USD 15,000 annual fee + per-device royalties.
- [DisplayPort Standard Version 2.1](https://vesa.org/featured-articles/displayport-standard/) — **VESA member-only**; membership starts USD 5,000/year.
- [MIPI D-PHY / C-PHY / M-PHY Specifications](https://www.mipi.org/specifications) — **MIPI Alliance member-only**; membership USD 16,000-60,000/year.
- [JEDEC JESD79-4D (DDR4)](https://www.jedec.org/standards-documents/docs/jesd79-4d) — **Free** at jedec.org with registration.
- [JEDEC JESD79-5C (DDR5)](https://www.jedec.org/standards-documents/docs/jesd79-5c) — **Free** at jedec.org.
- [JEDEC JESD209-5C (LPDDR5)](https://www.jedec.org/standards-documents/docs/jesd209-5c) — **Free** at jedec.org.
- [JEDEC JESD216F.02 (SFDP)](https://www.jedec.org/standards-documents/docs/jesd216f02) — **Free** at jedec.org.
- [IEEE Std 1149.1-2013 (JTAG)](https://standards.ieee.org/ieee/1149.1/4286/) — Paywalled, ~USD 200.
- [ISO 11898-1:2024 (CAN)](https://www.iso.org/standard/86384.html) — Paywalled, ~CHF 200.
- [IPC-2141A — Design Guide for High-Speed Controlled Impedance Circuit Boards](https://www.ipc.org/TOC/IPC-2141A.pdf) — Cross-ref IPC research; paywalled USD 285 from ipc.org.
- [IPC-2152 — Standard for Determining Current-Carrying Capacity in Printed Board Design](https://www.ipc.org/TOC/IPC-2152.pdf) — Cross-ref IPC research; paywalled USD 219.

### PHY-IC vendor layout guides (used for rule-pack content)

- [Marvell 88E1512 Layout Guide](https://www.marvell.com/products/ethernet-transceivers.html) — Marvell 1G PHY layout reference.
- [TI DP83867 Layout Guide](https://www.ti.com/product/DP83867E) — TI 1G PHY layout reference.
- [Microchip KSZ9131RNX Layout Guide](https://www.microchip.com/en-us/product/ksz9131rnx) — Microchip 1G PHY.
- [Micron TN-46-14 (DDR3 PCB Design)](https://www.micron.com/-/media/client/global/documents/products/technical-note/dram/tn4614.pdf) — DDR3 layout reference.
- [Micron TN-47-12 (DDR4 PCB Design)](https://www.micron.com/products/dram/ddr4-sdram/part-catalog) — DDR4 layout reference.
- [TI USB Type-C Layout Guide](https://www.ti.com/lit/an/slla414) — USB Type-C layout reference.
- [TI HDMI Repeater Layout Guide](https://www.ti.com/lit/an/slla320) — HDMI 2.0 layout reference.
- [Cypress / Infineon QSPI Layout Application Note](https://www.infineon.com/cms/en/product/memories/) — QSPI / OSPI layout reference.
- [TI TCAN1042 CAN-FD Transceiver Layout](https://www.ti.com/product/TCAN1042) — CAN-FD layout reference.
- [NXP TJA1043 CAN Transceiver Layout](https://www.nxp.com/products/interfaces/can-transceivers/) — CAN-FD layout reference.
- [Astera Labs PT4080 PCIe Gen5 Retimer Layout Guide](https://www.asteralabs.com/products/pcie-gen5/) — PCIe Gen5 vendor app note (used for Gen5 rule-pack content sourcing where PCI-SIG paywall constrains direct spec access).

### Reference textbooks and articles (open / public)

- Howard W. Johnson and Martin Graham, *High-Speed Digital Design: A Handbook of Black Magic* (Prentice Hall, 1993). The foundational SI textbook.
- Eric Bogatin, *Signal and Power Integrity — Simplified*, 3rd edition (Prentice Hall, 2018). Modern SI/PI textbook.
- Brian C. Wadell, *Transmission Line Design Handbook* (Artech House, 1991). Closed-form impedance formulae for microstrip / stripline / coplanar / coupled-line topologies.
- Mike Steinberger, *PCI Express Architecture* (chapter materials freely available at signalintegrityjournal.com). PCIe layout-rule reference.
- [SignalIntegrityJournal.com](https://www.signalintegrityjournal.com/) — Industry trade publication; archive of articles 2010-2026 covering controlled-impedance design, PCIe / DDR / Ethernet layout, return-path continuity, PDN design.
- [SI-LIST Mailing List archives](https://groups.io/g/si-list) — The historical SI/PI mailing list (1995-2026); searchable archive of practitioner discussion.
- [r/PrintedCircuitBoard](https://www.reddit.com/r/PrintedCircuitBoard/) — Reddit community; practitioner pain-points distilled in § "User Pain Points & Wishlist Items".
- [Saturn PCB Toolkit](https://saturnpcb.com/saturn-pcb-toolkit/) — Free PCB calculation toolkit including controlled-impedance, via-stub, current-capacity calculators. Used as a cross-check for impedance math.
- [Polar Si9000 Field Solver](https://www.polarinstruments.com/products/controlled_impedance/Si9000.html) — Mid-tier 2D field solver, USD 2k. Reference for SI-artifact export contract.
- [HyperLynx HYP File Format Documentation](https://eda.sw.siemens.com/en-US/pcb/hyperlynx/) — Mentor / Siemens; HYP-format documented in HyperLynx user guide. Reference for SI-artifact export contract.
- [JEDEC JEP106 manufacturer ID list](https://www.jedec.org/standards-documents/docs/jep-106aw) — **Free**; reference for manufacturer-name normalisation (cross-ref Domain 2).

### EDA tool documentation (used for support-matrix construction)

- [Altium Designer Documentation — High-Speed Design](https://www.altium.com/documentation/altium-designer/high-speed-design) — Altium high-speed design rule reference.
- [Altium Designer xSignals](https://www.altium.com/documentation/altium-designer/xsignals) — Altium length-match-group abstraction.
- [Cadence Allegro Constraint Manager Documentation](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis/constraint-driven-flow.html) — Cadence constraint-driven design.
- [Cadence Sigrity SI/PI](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis/sigrity-products.html) — Cadence SI/PI flagship.
- [Mentor HyperLynx](https://eda.sw.siemens.com/en-US/pcb/hyperlynx/) — Mentor / Siemens SI/PI reference tool.
- [KiCad 8 Length Tuning Documentation](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html#length-tuning) — KiCad 8 length-tuning reference.
- [KiCad 8 Net Class Editor Documentation](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html#net-classes-editor) — KiCad 8 net-class editor reference.
- [Horizon EDA documentation — Differential Pair Routing](https://horizon-eda.readthedocs.io/) — Horizon diff-pair routing reference.
- [LibrePCB — User Manual](https://librepcb.org/docs/) — LibrePCB user manual reference.

### Cross-references to prior research

- `research/standards-audit/STANDARDS_AUDIT.md` § 6 (Per-Domain Audit → 6. EMC & signal integrity) — Phase-1 inventory.
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` — Domain 2 deep-dive; the upstream IBIS / Touchstone attachment contract.
- `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md` — Domain 4 deep-dive; the `IndustryVertical` enum, `ProjectCompliance` block shape, substrate-vs-certification framing.
- `research/materials-environmental/MATERIALS_ENVIRONMENTAL_RESEARCH.md` — Domain 5 deep-dive; the Stackup-Dk/Df source-of-truth coordination, halogen-free laminate Dk/Df note.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` — IPC-2141A controlled-impedance reference, IPC-2152 current-carrying capacity reference, IPC-2581 Rev C high-speed metadata reference.
- `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.6 — current Domain 6 dispositions (post-Batch-1 baseline).
- `specs/ENGINE_SPEC.md` § 1.1a (Shared Enums for Behavioural Models), § 1.3 (Board Types — Net / NetClass / Stackup / StackupLayer), § 1.2 (Pool Types — Part), § 4 (Rules — RuleType / RuleParams) — Batch-1 contract surfaces this report builds on.
- `specs/MCP_API_SPEC.md` § Component Modelling Tools (IBIS / Touchstone / SPICE), § Encrypted Content Handling Policy — Batch-1 MCP contract surfaces this report consumes.
