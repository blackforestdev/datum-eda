# IPC Standards for PCB Layout and Library Compliance — Industry Survey

> Scope: the IPC standards landscape that bears on PCB layout, library
> footprint geometry, padstack data, design rule definition, and design
> data exchange. Purpose: to ground Datum EDA's M7 GUI and library
> direction in the same standards that commercial professional EDA tools
> measure themselves against.
>
> Companions: `research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md`
> and `research/copper-rendering/COPPER_RENDERING_RESEARCH.md`.

## Executive Summary

- **The flagship land-pattern standard, IPC-7351, is officially "No
  Longer Maintained".** IPC's own standards revision table lists
  IPC-7351 (last revision IPC-7351B from June 2010) as no longer
  maintained, with no IPC-7351C ever released. After roughly six years
  of work on a planned IPC-7351C, the IPC Land Pattern Committee
  changed direction, scrapped the in-progress C revision, and produced
  **IPC-7352** ("Generic Guideline for Land Pattern Design", Rev A
  released July 2023) as the unified successor that incorporates both
  surface-mount (formerly IPC-7351) and through-hole (IPC-7251) land
  pattern guidance. Despite this formal status, **IPC-7351B is still
  the lingua franca** every EDA tool, footprint vendor, and designer
  community refers to in 2026; IPC-7352 has had limited uptake so far.
- **Land pattern density levels are settled industry vocabulary.**
  IPC-7351 / IPC-7352 codify three Density Levels (Most / Nominal /
  Least, with naming-suffix letters M / N / L corresponding to the
  user's environment-condition selection), expressed as toe (Jt),
  heel (Jh), and side (Js) fillet allowances added to component lead
  span and width. Every serious tool exposes this choice. Datum needs
  it as a first-class library concept, not a magic number per
  footprint.
- **"Open-source EDA does not adhere to IPC" is half-true at best.**
  KiCad's KLC (KiCad Library Convention) is heavily IPC-7351-derived —
  pin 1 location, courtyard, silkscreen line widths, and chamfered-pad
  conventions all cite IPC-7351/C explicitly. Horizon EDA ships
  IPC-7351B-based footprint generation. LibrePCB explicitly mandates
  three-density-level package variants with IPC-7351 naming. The
  problem is *not* that open tools ignore IPC; it is that they (a)
  hard-code one density level, (b) don't surface the choice in the
  footprint object, and (c) lack the equivalent of Altium's "IPC
  Compliant Footprint Wizard" with full per-package family parameter
  forms. Commercial tools (Altium, OrCAD/Allegro, Pulsonix, DipTrace)
  ship the wizard; open tools generally don't.
- **The reference implementation is not an EDA tool — it is PCB
  Libraries Inc.'s LP Wizard / Footprint Expert.** Tom Hausherr (CEO
  of PCB Libraries) co-developed the IPC-7351 calculator with IPC
  itself; the LP Calculator and LP Wizard are the de-facto golden
  reference that other tools' wizards are validated against. A free
  LP Calculator is downloadable; the LP Wizard / Footprint Expert is
  paid. Footprint Expert exports to every major EDA format — Altium,
  Allegro, Cadence, Eagle, KiCad, OrCAD, PADS, Pulsonix.
- **IPC-2221 (generic design) and IPC-2152 (current capacity) are the
  load-bearing electrical-rule standards.** IPC-2221C (December 2023)
  is the active revision, with substantially revised conductor-spacing
  tables (Table 6-1) and new clearance guidance for vacuum, altitude,
  back-drilling, and compliant pins. IPC-2152 (2009, no longer
  maintained but still authoritative) replaced IPC-2221's old current
  charts with empirically-tested data showing internal traces tolerate
  far higher currents than the IPC-2221 "halve-the-external-value"
  rule of thumb claimed.
- **IPC-2581 (DPMX) is the modern data-exchange format and is shipped
  by every Tier-1 tool, including KiCad 8+.** Revision C (December
  2020) brought controlled-impedance-by-net, differential-pair
  identification, embedded-component support, edge-plating, and
  rigid-flex enhancements. Cadence Allegro, Altium Designer, Siemens
  Xpedition, Mentor PADS, and KiCad all support IPC-2581 export
  natively (Eagle/Fusion does not, as of M7's research date). Datum
  should treat IPC-2581C export as a M7-or-M8 requirement, not a
  nice-to-have.
- **Datum's existing Padstack-as-first-class data model is well
  positioned, but is missing IPC-specific metadata.** The current
  Padstack carries geometry per layer; it does not carry the
  IPC-derived parameters (density level used to generate it, source
  J-fillet values, source component lead/body tolerances, IPC-7351
  naming-convention components). Without those, Datum cannot validate
  a footprint against IPC, regenerate it for a different density
  level, or export an IPC-compliant name. This is a small data-model
  addition with large downstream payoff.
- **A small number of standards do most of the work, the rest are
  reference material.** For a credible M7-era PCB tool the irreducible
  set is: IPC-7351B (or IPC-7352) for land patterns, IPC-7093 for
  BTC/QFN thermal pads, IPC-7525 for stencils, IPC-2221 for clearance
  rules, IPC-2152 for trace current, IPC-4761 for via protection
  vocabulary, IPC-2581 for export, IPC-D-356 for fab netlist. Datum
  should treat these as the must-encode set; the rest (IPC-A-600,
  IPC-A-610, IPC-J-STD-001, IPC-CM-770) are vocabulary references the
  user community expects you to *speak*, not standards Datum needs to
  validate against in the engine.

## IPC Standards Catalog

> Revision dates and "no longer maintained" status confirmed against
> the IPC document revision table at electronics.org. Where IPC has
> retired a standard but not replaced its content, designers and
> tooling continue to treat the last-published revision as
> authoritative; this is noted per standard.

### Land pattern / footprint family

#### IPC-7351 / IPC-7351B / IPC-7351C (and the IPC-7352 succession)

**Full title:** Generic Requirements for Surface Mount Design and Land
Pattern Standard.

**Revision history:**
- Original: Feb 2005
- Rev A: Feb 2007
- Rev B: June 2010 — current de-facto reference
- Rev C: never released. Withdrawn after ~6 years of committee work
- **Successor:** IPC-7352 (see below)

**Status:** Officially "No Longer Maintained" per IPC's revision
table. In practice it is still the standard every commercial and
open-source tool, every footprint vendor, and every designer
community refers to.

**Scope:** Defines surface-mount land pattern geometries (pads,
courtyard, fillet allowances) for every common SMT package family —
chip components (resistors, capacitors, MELFs), gull-wing leaded
(SOIC, SOP, QFP, SOT), J-leaded (PLCC), inward-formed leads, BGA, CGA,
LGA, no-lead (DFN, QFN, SON), aluminum electrolytics, flat-lead
devices, and resistor/capacitor arrays.

**Key parameters Datum must encode:**

- **Density Level** — the three environment conditions:
  - Level A — Most material condition (large pads, low-density boards)
  - Level B — Nominal material condition (median pads, default)
  - Level C — Least material condition (small pads, high-density)
- **Solder fillet goals** for each lead type, three values:
  - Jt — toe fillet (outside the lead)
  - Jh — heel fillet (inside the lead, under body for J-lead/molded)
  - Js — side fillet (each side of the lead)
- **Component tolerances** — Lmax/Lmin (overall body length across
  leads), Tmin/Tmax (lead length), Wmin/Wmax (lead width), Smax/Smin
  (calculated lead-to-lead inner gap as Smax = Lmax − 2·Tmin)
- **Land pattern dimensions** computed from the above:
  - Z = land pattern outer extent
  - G = land pattern inner gap
  - X = land width
  - Y = land length
- **Tolerances** — fabrication tolerance, placement tolerance, both
  added to the calculation
- **Naming convention** — see "Naming convention adherence" below

**Indicative dimensions** (Density Level B / Nominal, as published in
multiple references derived from the IPC-7351B tables):

| Package | G (gap) | Y (pad len) | X (pad wid) | Z (overall) |
|---------|--------:|------------:|------------:|------------:|
| 0402 chip | 0.40 mm | 0.55 mm | 0.60 mm | 1.50 mm |
| 0603 chip | 0.70 mm | 0.90 mm | 1.00 mm | 2.50 mm |
| 0805 chip | 1.00 mm | 0.90 mm | 1.45 mm | 2.80 mm |

**For SOIC at Level B (per Altium's wizard documentation of the IPC
defaults):** Jt = 0.35 mm, Jh = 0.35 mm, Js = 0.03 mm. Level A: Jt =
0.55, Jh = 0.45, Js = 0.05. Level C: Jt = 0.15, Jh = 0.25, Js = 0.01.
The Js value being smallest for the largest level is correct — the
Level A "Most" name refers to environment robustness not numerical
size of every J value.

**Availability:** Paywalled (IPC store). Public summaries at Sierra
Circuits, Altium Resources, Ultra Librarian, PCBSync are extensive
and contain the formula structure if not the full per-package tables.
PCB Libraries' free LP Calculator implements the exact algorithms.

**Cross-references:** IPC-7251 (through-hole), IPC-7352 (unified
successor), IPC-7093 (BTCs), IPC-J-STD-001 (the soldering standard
the J-fillets are sized to satisfy), IPC-A-610 (assembly
acceptability — what a "good" fillet looks like in inspection).

**Implementation implications for Datum:**

1. The Padstack record needs a `density_level` enum (Most/Nominal/Least)
   plus the source J values it was generated with. Without these the
   footprint is not regeneratable and not validatable.
2. The Package record needs the source component tolerances (Lmin/Lmax,
   Tmin/Tmax, Wmin/Wmax) so the footprint can be re-derived if the
   density level changes.
3. Library tooling needs an IPC-7351 calculator-style footprint wizard
   per package family — this is the single largest open-tool gap
   versus Altium / OrCAD / DipTrace.
4. Naming should follow the IPC-7351B convention by default with the
   density-level suffix (`M`/`N`/`L`) preserved.

#### IPC-7251 (through-hole sibling of IPC-7351)

**Full title:** Generic Requirements for Through-Hole Design and Land
Pattern Standard.

**Status:** Original release the only published revision; not listed in
IPC's revision table. PCB Libraries forum threads suggest it is also no
longer being independently maintained — the through-hole content was
folded into IPC-7352.

**Scope:** The through-hole counterpart of IPC-7351. Sub-numbered by
component family — IPC-7252 (discretes), IPC-7253 (DIP), IPC-7254
(three-leaded semis like TO-220), IPC-7255 (PGA), IPC-7256 (multi-
function), IPC-7257 (connectors and headers), IPC-7258 (SIP resistor
networks), IPC-7259 (mounting hardware).

**Key parameters:** Same three-density-level framework as IPC-7351
(Levels A/B/C), with hole size, lead extension, and pad-around-hole
calculations. Specifies pad-to-hole annular ring requirements that map
to IPC-A-600 acceptance criteria.

**Implementation implications:** Same data-model needs as IPC-7351 but
adds drill geometry and hole-to-pad annular ring as the IPC-rule input
(annular ring is also covered by IPC-A-600 / IPC-6012).

#### IPC-7352 (the new unified land-pattern standard)

**Full title:** Generic Guideline for Land Pattern Design.

**Revision:** Rev A, July 2023 (per IPC's revision table). The
official store also lists "Revision 0" (May 2023); the table entry
"Rev A 7/23" is the current published version.

**Scope:** Unified land pattern standard intended to replace both
IPC-7351 (SMT) and IPC-7251 (through-hole) under one cover. Carries
the three-density-level framework forward. Adds support for "mixed"
mounting technology where SMT and through-hole share the same
package. Provides recommended pad shapes, fillet/courtyard
allowances, mask/stencil considerations, and naming conventions.

**Naming convention difference from IPC-7351B:** IPC-7352 keeps the
component-type prefix at the front and pin count at the end. Earlier
draft IPC-7351C had moved pin count to the front; this was rolled
back when 7351C was scrapped. Footprint Expert v23 ships three
selectable conventions (PCB Libraries / IPC-7351B / IPC-7352) — a
useful indicator of how unsettled the naming question still is in
practice.

**Adoption status:** Slow. As of mid-2026, most tool wizards still
identify themselves as "IPC-7351 compliant"; the IPC-7352 label
appears mostly in PCB Libraries' Footprint Expert and in vendor
marketing.

**Datum implication:** Plan for both naming conventions in the
library-name policy, with the IPC-7351B form as the default and
IPC-7352 as a per-library or per-pool toggle. Treat the underlying
Density-Level / J-fillet model as identical for both.

#### IPC-7525 / IPC-7525B / IPC-7525C (stencils)

**Full title:** Stencil Design Guidelines.

**Revision history:**
- Original: May 2000
- Rev A: Feb 2007
- Rev B: October 2011
- Rev C: November 2021 — current

**Scope:** Defines aperture sizing, area ratio, aspect ratio, and
paste-mask-reduction recommendations for solder paste stencils.
Determines what the paste-layer aperture for a pad should look like
(it's almost never the same as the copper pad).

**Key parameters Datum must encode:**

- **Area ratio** = aperture area / aperture-wall area. For a
  rectangular aperture of width W, length L and stencil thickness T:
  AR = (W·L) / (2·(W+L)·T). For a circular aperture of diameter D:
  AR = D / (4·T). **Recommended minimum: 0.66.** Below ~0.50 the
  paste won't release reliably.
- **Aspect ratio** = aperture width / stencil thickness. **Recommended
  minimum: 1.5** (for rectangular apertures). Less critical than area
  ratio for irregular shapes.
- **Paste mask reduction** for QFN/DFN thermal pads: typically
  segmented into a 2x2, 3x3, 4x4 array of smaller apertures to
  achieve the IPC-7093-recommended 50–80% paste coverage on the
  thermal pad without one giant aperture that floods solder under the
  package.
- **Standard stencil thicknesses:** 0.10, 0.12, 0.13, 0.15, 0.20 mm
  (4, 5, 5, 6, 8 mil). Datum's paste mask should know which stencil
  thickness the area-ratio check is against.

**Availability:** Paywalled. Free public summaries (Aivon,
BlueRingStencils, PCBSync) cover the area-ratio formula and minimum
0.66 threshold extensively.

**Cross-references:** IPC-7351 (the pads the stencil is for),
IPC-7093 (the thermal-pad-aperture special case for BTCs).

**Implementation implications:**
- Datum's PadStack `aperture` should track the paste-mask shape
  separately from the copper shape (it already has `PadAperture` —
  good; needs the paste / mask split formalised).
- A DRC-style "stencil sanity" check that flags pads with area ratio
  < 0.66 against an assumed stencil thickness is a high-value
  manufacturability check that no open-source tool ships well.
  Worth a Datum differentiator.

#### IPC-7093 / IPC-7093A (BTC / QFN / DFN / LGA)

**Full title:** Design and Assembly Process Implementation for Bottom
Termination Components.

**Revision history:**
- Original: March 2011
- Rev A: October 2020 — current

**Scope:** The dedicated standard for "BTCs" (Bottom Termination
Components) — components whose external connections consist of
metallized terminals on the bottom surface of the package body, with
no leads emerging from the side. Covers QFN (Quad Flat No-lead), DFN
(Dual Flat No-lead), SON, LGA (Land Grid Array), MLP, MLF, and the
thermal pad / exposed pad sub-class that comes with most of these.

**Key parameters Datum must encode:**

- **Perimeter pad sizing** — for most BTCs, NSMD (Non-Solder-Mask-
  Defined) pads are preferred; SMD (Solder-Mask-Defined) is a
  per-pad property worth carrying. NSMD reliability is materially
  better than SMD for perimeter terminals.
- **Thermal pad design** — the thermal/ground pad is its own
  geometry, not a copy of the perimeter pad. Carries its own
  via-in-pad placement, paste-mask segmentation, and solder-mask
  treatment.
- **Paste coverage on thermal pad** — IPC-7093A recommends 50–80%
  paste coverage of the thermal pad area, achieved through a
  segmented stencil aperture (NOT a solid stencil opening) so that
  during reflow the package can settle without the paste pumping out
  to the perimeter pads.
- **Via-in-pad treatment** — for thermal pads with via-in-pad,
  IPC-4761 fill type (typically Type VII filled-and-capped) becomes
  a binding requirement to avoid solder voiding through the via.
- **Solder mask defined thermal pad** option — uses solder mask
  rings around vias to dam solder flow during reflow.

**Availability:** Paywalled. The 7093A table of contents is publicly
visible at electronics.org and shows extensive coverage of stencil,
inspection, and rework guidance.

**Cross-references:** IPC-7351 (the underlying land-pattern math),
IPC-7525 (stencil aperture design), IPC-4761 (via fill / via-in-pad),
IPC-A-610 (acceptance criteria for QFN solder joints).

**Implementation implications:**
- The Package model needs a separate `thermal_pad` concept; it can't
  be just "one of the pads".
- Paste-mask segmentation pattern should be a property the library
  authoring tool can generate from a single setting (segmentation
  pitch, percentage coverage).
- DRC: BTC packages with via-in-pad but without IPC-4761 Type V/VI/VII
  via fill specified should warn.

#### IPC-CM-770

**Full title:** Component Mounting Guidelines for Printed Boards.

**Revision:** Rev E, January 2004 — current. **Status: No Longer
Maintained.**

**Scope:** Covers component mounting techniques — manual and
machine assembly, SMT, BGA, flip chip, through-hole. Treats the
process side (placement, soldering, cleaning, coating) more than the
land-pattern side.

**Implementation implications:** Vocabulary reference only. Datum
does not need to validate against IPC-CM-770; it does need to use
its terminology when surfacing assembly-side warnings.

### Generic design / fabrication family

#### IPC-2221 / IPC-2221B / IPC-2221C

**Full title:** Generic Standard on Printed Board Design.

**Revision history:**
- Original: Feb 1998 (supersedes IPC-D-275)
- Amendment 1: Jan 2000
- Rev A: May 2003
- Rev B: November 2012
- **Rev C: December 2023 — current**

**Scope:** The generic design rule standard. Defines clearance,
conductor spacing, electrical strength, dielectric breakdown, and
the structural rules every PCB inherits. It is what every "PCB
clearance calculator" on the internet implements.

**Key tables Datum must encode:**

- **Table 6-1** — minimum conductor spacing as a function of peak
  working voltage, by conductor environment. The classic columns:
  - **B1** — Internal conductors
  - **B2** — External conductors, uncoated, sea level to 3050 m
  - **B3** — External conductors, uncoated, above 3050 m
  - **B4** — External conductors with permanent polymer coating
  - **A5** — External conductors with conformal coating over assembly
  - **B5 (new in 2221C)** — explicit vacuum operation case
- **Voltage-to-spacing examples** (per multiple public summaries):
  - 0–15 V internal: 0.05 mm
  - 0–15 V external uncoated: 0.10 mm
  - 16–30 V internal: 0.05 mm
  - 31–50 V external uncoated: 0.10 mm (B2)
  - 100 V external uncoated: 0.20 mm (B2)
  - 300 V external uncoated: 1.25 mm (B2)
- **Spacing above 500 V** — formula: spacing = base + (V − 500) × δ.
  For uncoated tracks IPC-2221B: spacing(mm) = 2.5 + (V−500)·0.005.
  IPC-2221C revised the per-volt step value (the 0.005); Datum
  should encode both formulas selectable by IPC revision target.
- **Conductor sizing for current** — historically Table 6-4
  (external) and 6-5 (internal). IPC-2221's old rule of "internal
  current = ½ external" is the value that IPC-2152 disproved; for
  current calculations IPC-2152 is the modern reference (see below).

**Availability:** Paywalled, but every detail of Table 6-1 has been
discussed and tabulated publicly. Sierra Circuits, Siemens (Apr 2025
blog), Altium calculators, smpspowersupply.com, and PCBSync all
reproduce the relevant numbers.

**Cross-references:** IPC-2222 (rigid sectional), IPC-2226 (HDI
sectional), IPC-2152 (current capacity), IPC-2615 (dimensions and
tolerances). IPC-D-275 was the predecessor superseded by IPC-2221.

**Implementation implications for Datum DRC:**
- Datum needs an `IPC-2221 clearance rule` configurable by
  voltage class (per net or per net class) and conductor environment
  (per layer assignment plus a global "altitude / coating" project
  setting). The lookup is from (peak voltage, environment) → min
  spacing.
- Should be selectable IPC-2221B vs IPC-2221C (the new 2221C
  altitude treatment, vacuum case, and sub-500V/super-500V values
  all differ slightly).
- The Datum project file needs an `intended_environment` field
  (sea-level uncoated / sea-level conformal / conformal / encapsulated
  / vacuum / above-3050m). Without it the clearance check has no
  defensible default.

#### IPC-2222 / IPC-2222B (rigid sectional)

**Full title:** Sectional Design Standard for Rigid Organic Printed
Boards.

**Revision history:** Original 1998, Rev A 2010, **Rev B October 2020
— current**.

**Scope:** Sectional supplement to IPC-2221 for rigid organic boards.
Covers laminate and material properties, hole specifications,
mechanical tolerances, layer-stack assumptions, and rigid-board-
specific structural rules.

**Implementation implications:** Datum should be aware of the
distinction; most rigid-board designs apply IPC-2222B alongside
IPC-2221C. For M7 the IPC-2222 content is largely fab-side and Datum
can defer it.

#### IPC-2226 (HDI)

**Full title:** Sectional Design Standard for High Density
Interconnect (HDI) Printed Boards.

**Revision history:** Original April 2003, Rev A September 2017.
**Status: No Longer Maintained** per IPC's table.

**Scope:** Defines six HDI construction Types based on microvia
configuration and core construction:
- **Type I** — single microvia layer on one or both sides of a
  conventional core
- **Type II** — Type I + buried vias in the core
- **Type III** — two or more microvia layers on at least one side
  (the most common modern HDI build, including the popular `2+N+2`
  stackup that requires two sequential lamination cycles per side)
- **Types IV / V / VI** — exotic constructions for specific
  applications (passive substrate, etc.)

**Key parameters:** Microvia diameter (typically 0.10–0.15 mm), pad
sizes for microvias (typically 0.30 mm for a 0.15 mm laser drill),
sequential lamination step counts, core thickness for HDI.

**Implementation implications:** Datum's via type taxonomy should
include `microvia` distinct from `via` (it already does). The HDI
construction Type is a stack-up-level property; relevant for fab
output and impedance modelling but not strictly needed for M7
rendering.

#### IPC-2152 (current capacity)

**Full title:** Standard for Determining Current Carrying Capacity
in Printed Board Design.

**Revision:** Original August 2009. **Status: No Longer Maintained**
per IPC's table, but it is still the authoritative document for
current capacity — IPC-2221's old current charts have not been
superseded by anything else.

**Scope:** Replaced IPC-2221's decades-old current-versus-cross-
section formula with empirically tested data. The IPC-2221 formula
i = K × ΔT^0.44 × Ac^0.725 (with K = 0.048 for outer, 0.024 for
inner) was based on 1950s tests; IPC-2152 tested hundreds of board
configurations and showed the old "internal = half external" rule
was simply wrong — internal traces handle current much closer to
external.

**Key parameters:** Trace current capacity is a function of
(temperature rise, conductor cross-section, board thermal conductivity,
copper plane presence, board mounting, ambient environment, board
thickness). Output is a family of charts (not a closed-form formula
— IPC-2152 deliberately presents look-up curves rather than a
regression formula).

**Reference figures:** Figure 5-2 (universal 3 oz copper polyimide
chart), with finer-resolution variants Figures 5-3..5-5. Internal
and external traces share the same baseline chart in the modern
formulation.

**Availability:** Paywalled. Calculator implementations are
everywhere (Sierra Circuits, smps.us, Altium); the underlying data
is well-documented in IPC's own conference papers (Mike Jouppi).

**Implementation implications:**
- Datum's current-capacity check should support both IPC-2221 (legacy
  formula) and IPC-2152 (chart-based) modes. IPC-2152 results require
  per-net or per-net-class current targets and a board-thermal-class
  setting.
- This is appropriately deferred from M7 (it's a DRC capability, not
  a rendering concern); flagging as future M8 work.

#### IPC-2615

**Full title:** Printed Board Dimensions and Tolerances.

**Revision:** Original July 2000. **Status: No Longer Maintained.**

**Scope:** Geometric Dimensioning and Tolerancing (GD&T) for PCBs
— positional, profile, orientation, form tolerances; the geometric
symbology used on fabrication drawings. Companions IPC-2221 and the
6012 fabrication-acceptance standards.

**Implementation implications:** Output-side concern; relevant for
Datum's fab-drawing export, not for M7 rendering.

#### IPC-4761 (via protection)

**Full title:** Design Guide for Protection of Printed Board Via
Structures.

**Revision:** Original July 2006. **Status: No Longer Maintained**,
but the Type I–VII vocabulary is still universally used.

**Scope:** Defines seven via protection Types, taxonomy of via
treatments:
- **Type I** — Tented (one side covered with dryfilm solder mask)
- **Type II** — Tented and Covered (Type I + overprinted with normal
  solder mask)
- **Type III** — Plugged (partially filled with non-conductive paste)
- **Type IV** — Plugged and Covered
- **Type V** — Filled (completely filled with non-conductive paste)
- **Type VI** — Filled and Covered
- **Type VII** — Filled and Capped (filled, planarised, copper-plated
  over — flat solderable surface, the gold standard for via-in-pad)

**Implementation implications for Datum:**
- The via type taxonomy in Datum needs to expose IPC-4761 Type I–VII
  as a property of the via (not just `tented: bool`). The current
  `via_type` flag noted in the copper guidance research document is
  the right place; map it to the IPC-4761 Type letter.
- IPC-4761 Type is a fabrication input, not a rendering choice; but
  the renderer needs to know whether to draw the mask aperture
  (Types III, V — show aperture; Types I, II, IV, VI — no aperture;
  Type VII — flat copper, no aperture, cap visible).

#### IPC-A-600

**Full title:** Acceptability of Printed Boards.

**Revision:** Rev M, May 2025 — current. (Previous: Rev K, July 2020;
Rev J 2016; Rev H 2010; Rev G 2004.) Originally published 1964.

**Scope:** Fab-side visual acceptance criteria — annular rings,
plating voids, dewetting, copper thickness, edge plating, solder
mask coverage. Defines what is "acceptable" for Class 1 / 2 / 3
products. The fabrication-side counterpart to IPC-A-610.

**Implementation implications:** Reference vocabulary only. Datum's
DRC can use IPC-A-600's annular ring class definitions (Class 1 /
2 / 3) to scope its own annular-ring checks.

#### IPC-A-610

**Full title:** Acceptability of Electronic Assemblies.

**Revision:** Rev J, March 2024 — current. (Previous: Rev H 2020;
Rev G 2017; Rev F 2014.) Originally published 1983.

**Scope:** Assembly-side visual acceptance criteria. The most-cited
IPC standard; defines the famous **Class 1 / Class 2 / Class 3**
quality classifications used everywhere in the industry:
- **Class 1** — General Electronic Products (consumer, short life,
  cosmetic imperfections OK)
- **Class 2** — Dedicated Service Electronic Products (industrial,
  brief downtime acceptable, no cosmetic-impact functional defects)
- **Class 3** — High Performance / Harsh Environment (life-critical,
  no failures acceptable, strictest fillet/joint criteria)

A defect at Class 1 implies a defect at Class 2 and 3 — the classes
are nested.

**Implementation implications:** Datum's project metadata should
have an `ipc_class` field (1/2/3); DRC rule defaults should derive
from it. Pin 1 marking, polarity marking, courtyard size all have
Class-dependent expectations from IPC-A-610.

#### IPC-J-STD-001

**Full title:** Requirements for Soldered Electrical and Electronic
Assemblies.

**Revision:** Rev J, April 2024 — current. (Previous: Rev H 2020;
Rev G 2017.) Originally published April 1992.

**Scope:** The soldering process standard. Defines materials,
methods, and acceptance criteria for soldered joints. Co-published
with IPC-A-610 and shares the Class 1/2/3 hierarchy.

**Implementation implications:** Vocabulary-only. The IPC-7351
fillet allowances exist in part to satisfy J-STD-001; Datum need not
validate against J-STD-001 directly.

### Data exchange family

#### IPC-2581 (DPMX — Digital Product Model Exchange)

**Full title:** Generic Requirements for Printed Board Assembly
Products Manufacturing Description Data and Transfer Methodology.

**Revision history:**
- Original: March 2004
- Amendment 1: May 2007
- Rev A: May 2012
- Rev B: September 2013
- Rev B Amendment 1: January 2017
- **Rev C: November 2020 — current**

**Scope:** XML-based, single-file format that bundles everything
needed to fabricate and assemble a board:
- Copper image data per layer
- Board layer stack-up
- Drill information
- Netlist (bare-board test + populated-board)
- Component BOM with placement
- Fabrication and assembly notes
- (Rev C) Controlled-impedance specifications per net / layer / stack
- (Rev C) Differential pair identification
- (Rev C) Embedded components, cavities, coins, edge plating
- (Rev C) Bidirectional DFx — manufacturer feedback can flow back
  through the same file

**Status:** Active and widely supported. The IPC-2581 Consortium
includes Cadence, Altium, Mentor/Siemens, Zuken, and others. Free
public viewers exist (e.g., Sierra Circuits' viewer).

**Tool support (April 2026 status):**
- **Altium Designer** — full IPC-2581 export (member of consortium)
- **Cadence Allegro / OrCAD** — full export and import; can generate
  difference reports between revisions
- **Siemens Xpedition** — full IPC-2581 import/export via the BluePrint
  workflow
- **Mentor PADS** — IPC-2581B export via the ODB++ export dialog
  (checkbox option)
- **KiCad 8.0+** — native IPC-2581 export. Default revision is B in
  KiCad 8; later releases default to C
- **Eagle / Fusion Electronics** — no native IPC-2581 export
  (consistent gap)
- **DipTrace, EasyEDA, LibrePCB, Horizon EDA** — no native IPC-2581
  export

**Comparison to ODB++ and Gerber X3:**
- **ODB++** — Mentor/Siemens proprietary (now Valor MSS), open spec,
  also bundles fab+assembly. Wider real-world fab adoption than
  IPC-2581 historically. Folder-of-files vs IPC-2581's single XML.
- **Gerber X3** — extension of Gerber X2 with assembly attributes.
  Multi-file like classic Gerber. Less complete than IPC-2581 for
  netlist / BOM / impedance. Most fab houses still accept Gerber as
  the universal interchange.

**Cross-references:** IPC-D-356 (the netlist piece can be exported
separately), IPC-2581's RF / impedance content overlaps with parts
of IPC-2141 (impedance).

**Implementation implications for Datum:**
- IPC-2581C export should be on Datum's near-term roadmap. It is
  table-stakes for any tool that wants to be taken seriously by
  Tier-1 fabricators. Implementing C (not B) gives access to the
  controlled-impedance schema Datum will eventually want.
- The schema is XSD-based; consortium publishes the schemas openly.
  No reverse-engineering needed.

#### IPC-D-356 / IPC-D-356A / IPC-D-356B

**Full title:** Bare Substrate Electrical Test Data Format.

**Revision history:**
- Original: March 1992
- Rev A: January 1998 — most-supported by CAD tools
- Rev B: October 2002 — current revision
- **Status:** No Longer Maintained.

**Scope:** ASCII fixed-width 80-column format (one record per line)
that defines the netlist, pad coordinates, and via positions for
bare-board electrical test. The fab uses it to program flying-probe
or bed-of-nails testers. Predates IPC-2581 by a decade and is still
universally accepted.

**Format:** 80-column fixed-width text. Record types include 327
(net point), 317 (via), 378 (test point). Each record contains
net name, reference designator, pin number, X/Y, hole/SMT flag,
layer.

**Tool support:** Universal — every Tier-1 EDA tool exports it,
every fab accepts it.

**Implementation implications for Datum:**
- IPC-D-356A export is an essential M7-or-M8 deliverable. It is
  small, parseable, well-documented (free-to-view IPC-D-356A
  reference at MSU and pcb-rnd-aux). KiCad ships it; not having
  it would be a credibility gap.
- Carries net names, so Datum's stable-net-ID model needs a clean
  serialization to net-name strings.

### Other relevant

#### IPC-T-50 (the dictionary)

**Full title:** Terms and Definitions for Interconnecting and
Packaging Electronic Circuits.

**Revision:** Rev N, November 2021 — current. (Rev M May 2015 had
~220 new/revised terms; Rev N had ~550, including major updates to
via terminology, soldering vocabulary, and fab-process definitions.)

**Scope:** The master IPC glossary. All other IPC standards
reference T-50 for definitions. Designed to support second-language
English readers — terse, precise, no idiom.

**Implementation implications:** Vocabulary baseline. Datum's UI
labels, AI-prompt vocabulary, and documentation should use IPC-T-50
terms wherever applicable. "Annular ring", "land", "fillet",
"courtyard", "padstack" are all T-50 terms.

#### IPC-1752 (materials declaration — note only)

**Full title:** Materials Declaration Management.

**Revision:** Rev B, July 2020 — current. (Rev A Feb 2010 with
amendments through Feb 2014; Rev B is the SCIP / REACH-aligned
version.)

**Scope:** XML data exchange format for declaring materials in
products — RoHS, REACH, conflict minerals, SCIP database. Three
declaration classes:
- Class A — yes/no compliance assertion
- Class B — declarable substance groups present
- Class C — full composition breakdown to ~100 ppm

**Implementation implications:** Out of scope for M7. Worth noting in
Datum's component / Part metadata model that BOM-level material
declaration is the eventual integration point — Part records should
have room for an MPN-keyed lookup to vendor IPC-1752 data. Datum
itself does not need to consume IPC-1752; downstream BOM / supply-
chain tools do.

## EDA Tool Implementation Survey

### Commercial

#### Altium Designer

The visual reference point for IPC-compliant footprint authoring.

- **IPC Compliant Footprint Wizard** — full per-package-family form
  with explicit Density Level (A/B/C) selection on its own dialog
  page (the "SOIC Solder Fillets Window" for SOIC; equivalent for
  every other family). Uses IPC's published Jt/Jh/Js defaults per
  level, with a "Use default values" checkbox or full override.
  Builds 2D footprint, optionally 3D body, optionally mechanical /
  silk / courtyard layers all in one pass. The wizard ships standard
  in Altium Designer; no add-on cost. The "IPC Compliant Footprints
  Batch Generator" runs the same engine over a CSV / library file
  for bulk generation.
- **Naming convention** — generates IPC-7351B-style names by default
  (e.g., `SOIC127P600X175-8N`).
- **DRC rule presets** — Altium's rules engine can encode IPC-2221
  clearance, with rule scope expressions per net / net class /
  layer. No literal "IPC Class 2 preset" button, but the manual
  encodes the equivalent rule expressions.
- **IPC-2581 export** — full Rev C support; Altium is an active
  consortium member.
- **IPC-D-356** — exported from the Output Job File system.
- **Forum sentiment:** Wizard is broadly trusted; chief complaint is
  that some package families (notably newer BTC subtypes) lag
  IPC-7093A updates.

#### Cadence Allegro / OrCAD X Presto

- **OrCAD Library Builder** — the IPC-driven land pattern generator.
  Includes PDF datasheet extraction (proprietary OCR-style tooling),
  IPC-7351 calculator-driven GUI, automated symbol generation, STEP
  3D model generation. Calls itself "IPC-7351-driven" and aligns
  closely with PCB Libraries' algorithms.
- Allegro itself does not ship a built-in IPC wizard at the same
  level as Altium; users typically run Library Builder or external
  PCB Libraries Footprint Expert (which has Allegro export) and
  bring patterns in.
- **IPC-2581 export** — full, both directions; can generate diff
  reports between revisions and export stack-up data separately.
- **DRC rule presets** — Allegro's Constraint Manager handles IPC-
  2221 and IPC-6012 class rules through preset templates.

#### Siemens Xpedition / PADS (formerly Mentor)

- **PADS Land Pattern Creator** (formerly LP Wizard, originally
  PCB Libraries' product) — the most-pedigreed IPC-7351 wizard in
  the commercial space. PADS bundled the full PCB Libraries LP
  Wizard for many years; the relationship and product continuity
  is documented in Siemens' own electronic-systems-design blog
  series. Ships with Xpedition's library subsystem.
- **Pin Pair / topology protection** — relevant to authoring more
  than to IPC, but PADS's library workflow is unusually mature.
- **IPC-2581 export** — Xpedition supports both IPC-2581 and
  ODB++. PADS exports IPC-2581B via the ODB++ dialog.

#### Pulsonix

- **Pulsonix Library Expert** — exposes IPC-7351 "B" and "C"
  revision generation; uses PCB Libraries' Footprint Expert
  engine under the covers (per PCB Libraries' own product page
  for the Pulsonix Enterprise Edition). Also references IPC-7352
  and IPC-J-STD-001 for joint acceptance.
- Modest market share but a credible IPC story.

### Open-source

#### KiCad (KLC, official library, footprint wizards)

KiCad is the highest-profile open-source EDA tool and the place
where the "open source ignores IPC" myth most needs correcting.

**KiCad Library Convention (KLC):** A formal, versioned document at
klc.kicad.org that governs official KiCad library contributions.
IPC-7351 is cited explicitly throughout:

- **F4.2** — Pin 1 location: upper left corner per IPC-7351.
- **F5.1** — Silkscreen line widths: 0.10 mm (high density) /
  0.12 mm nominal / 0.15 mm (low density), citing IPC-7351C
  (despite IPC-7351C never having been formally released — KLC was
  written against an IPC-7351C draft).
- **F5.3** — Courtyard rules per IPC-7351C; line width 0.05 mm,
  default offset 0.25 mm for SMT (0.15 mm for sub-1.5 mm parts),
  0.5 mm for connectors / capacitors / crystals, 1.0 mm for BGA.
- **F2.1 / F2.2** — Naming convention: hierarchical with
  underscores, e.g., `SOIC-8_3.2x5.7mm_P1.27mm`. **This is not
  the IPC-7351B `SOIC127P600X175-8N` form** — KiCad chose a more
  human-readable variant. Forum discussion of this divergence is
  long and explicit; the KiCad team's stated rationale is
  readability and grep-ability, with an explicit acceptance that
  it is non-conforming with IPC-7351B's naming.
- **Density level:** KiCad's official library is built at Density
  Level B (Nominal) by default, but **the level is not encoded in
  the footprint name** and the footprint object carries no
  metadata indicating which level it was generated at. This is the
  largest single IPC-compliance gap in the official KiCad library.
- **Pad shape:** rounded-rect by default, with a 25%-of-shorter-edge
  radius (or 0.25 mm cap, whichever is smaller) — citing IPC-7351C.

**KiCad footprint wizards (in-app, Python):** A built-in family of
parametric footprint wizards (BGA, SOIC, S-DIP, QFP, etc.) shipped
with the KiCad PCB editor. Use IPC-7351B and IEC 61188-7 for
calculations. Long-running stability issues — the SOIC and S-DIP
wizards have thrown Python exceptions for multiple KiCad releases
(GitLab issue #4896 is the canonical bug). Useful for one-off
generation; not the primary path for the official library.

**kicad-footprint-generator (community + GitLab official):** The
Python script library used to generate the official KiCad library
footprints in bulk. Lives in the official KiCad GitLab as
`kicad/libraries/kicad-footprint-generator`. Each package family
has a YAML-driven generator with IPC-7351 default parameters
(`scripts/Packages/ipc_definitions.yaml`). This is where the
"density level B" choice is encoded for the official library.

**Forum criticism:** The KiCad community has documented multiple
specific deviations:
- 0603 capacitor and 0603 resistor pads not identical (per IPC-7351
  they should be — same package, different component class is not a
  reason to differ);
- Real-world manufacturing reports that IPC-7351B Nominal sizes
  performed badly for some processes, suggesting that the IPC
  defaults themselves should be treated as a starting point not a
  destination. This is a common professional view.

**IPC-2581 export:** KiCad 8.0 (Feb 2024) shipped native IPC-2581
export for the first time. Both B and C revisions selectable;
default has shifted from B (in 8.0) to C in newer builds.

**IPC-D-356 export:** Available since KiCad 4.x.

**Net assessment:** KiCad's *intent* is heavily IPC-aligned through
KLC. The *gaps* are: (1) no in-tool density-level switch on the
footprint object itself; (2) naming-convention divergence from
IPC-7351B; (3) flaky in-app wizards; (4) no per-footprint metadata
indicating IPC compliance level.

#### Horizon EDA

Studied directly from `research/horizon-source/` and the public
horizon-pool-convention repo.

- **Pool padstacks:** Horizon's Padstack is a JSON file with
  per-layer geometry (paste, mask, copper, drill). Custom padstacks
  live in the package-local `padstacks/` directory; generic ones in
  the pool root. Parameter programs control mask expansion / paste
  contraction. This is the closest open-source structural match to
  Datum's Padstack-as-first-class model.
- **IPC compliance language in pool conventions:** Explicitly
  forbids "IPC compliant" as a name modifier — the convention
  expects all pool entries to *be* IPC-compliant by default, so it
  shouldn't need to be marked.
- **Footprint generator:** Horizon shipped an interactive
  footprint generator (PR #287, merged July 2019, by `endofexclusive`)
  built on a separate `footag` library. Implements IPC-7351B
  calculations from component lead dimensions, lead tolerances,
  fab tolerance, placement tolerance, density level. The PR
  description notes: "IPC-7351B does not specify what artwork
  shall look like. So things like silkscreen is not covered" —
  i.e. the generator does the math but leaves silkscreen / courtyard
  / artwork to the user.
- **Naming:** Horizon's pool convention is non-IPC-7351B (more
  similar to KLC's verbose form than IPC's compact form).
- **IPC-2581 / IPC-D-356:** Not supported.

**Net assessment:** Horizon has the strongest structural match to
Datum's library model, the cleanest separation of padstack as a
first-class object, and a credible (if narrow) IPC-7351B generator.
Where Datum can advance is: (a) carry density-level metadata on the
generated padstack, (b) ship more package families in the
generator, (c) generate the silkscreen / courtyard the IPC standard
declines to specify.

#### LibrePCB

LibrePCB has the most explicit IPC-7351 alignment in its written
package conventions of any open-source tool.

- **Naming:** "Generally follow IPC-7351 when naming packages" for
  covered standards. Metric units mandatory; American English;
  pin counts appended without leading zeros.
- **Density levels:** "Recommends implementing all three IPC density
  levels (A, B, C) as separate footprint variants when applicable."
  Default footprint is Density Level B. Encourages combining density
  level × mounting style as separate variants — a package supporting
  three density levels and two mounting variants would have six
  footprint variants.
- **Pin 1:** Always at top-left, citing IPC-7351C "Level A, slide 22".
- **Courtyard:** Mandatory polygon on Top Courtyard layer; 0.0 mm
  line width; 0.2 mm offset for SMT (citing IPC-7351 if applicable),
  0.4 mm for THT.
- **Legend (silkscreen):** Single polygon, 0.2 mm typical line width
  (0.1 mm minimum); per IPC-7351C; should remain visible after
  assembly.
- **Pad rules:** Each lead requires its own package pad. "Use pad
  names according to IPC-7351 (if applicable)" or
  counterclockwise / functional naming.
- **IPC-2581 / IPC-D-356:** Not supported as of this research.

**Net assessment:** LibrePCB is the closest written articulation of
IPC compliance in any open-source tool. The footprint-variant model
(one Package, multiple footprints for density / mounting) is a
different architectural answer than KiCad's (one Package per
density level) — Datum's Package model could go either way and
should pick consciously.

### Consumer / cloud

#### Eagle / Fusion 360 Electronics

- **Fusion package generator:** Inside the Fusion library editor,
  "by design compliant with IPC-7351" per Autodesk documentation.
  Will generate compliant 2D footprints and 3D models. Less
  feature-complete than Altium's wizard.
- **Eagle DRC:** Not configured against IPC-7351 footprint quality
  out of the box. Per an Autodesk forum thread: Eagle DRC rules
  themselves are not IPC-aligned by default; users must configure.
- **Eagle library:** Heritage Eagle library is *not* IPC-7351-
  derived in any rigorous sense; many footprints predate the
  standard. Modern Fusion-era libraries are better.
- **IPC-2581:** Not supported. Long-standing user wishlist item.
- **IPC-D-356:** Supported via legacy Eagle exporters.

#### DipTrace

- **IPC-7351 Pattern Generator** — a built-in wizard that creates
  IPC-7351-compliant patterns and 3D models from per-package-family
  parameter forms. Bulk-edit tools and templates available. Recent
  releases (3.x → 4.0) added CQFP, DFN (2/3/4-pin), SOD, SODFL,
  Radial LED package families. Standard library reportedly contains
  >29,000 IPC-7351-built patterns out of >164,000 components. This
  is one of the more capable wizards in the consumer-tier tools.
- **IPC-2581:** Not supported.
- **IPC-D-356:** Supported.

#### EasyEDA

- **Footprint editor:** Exists; no formal IPC-7351 wizard is
  documented. Heavy reliance on the LCSC component library
  (EasyEDA's vendor-supplied library) which is supplied by JLCPCB
  for assembly compatibility. Footprints in the LCSC library are
  designed for JLCPCB's process; they may or may not align with
  IPC-7351 depending on the entry. No documented controversy or
  systematic audit.
- **IPC-2581 / IPC-D-356:** Not supported.

#### Quadcept

- Limited public documentation on IPC-7351 wizard support. Quadcept
  positions itself as a Japanese commercial tool with a moderate
  footprint library; no specific IPC-7351 wizard is publicly
  documented.

### Reference implementations & third-party libraries

#### PCB Libraries Inc. — LP Wizard / LP Calculator / Footprint Expert (the gold standard)

PCB Libraries' tooling — and Tom Hausherr personally — is the
de-facto reference implementation of IPC-7351.

- **LP Calculator** — free, downloadable from PCB Libraries.
  Implements the IPC-7351 algorithms exactly as published; PCB
  Libraries was IPC's collaborator on the standard's development.
  Per Electronic Design's coverage of the launch: "The calculator
  relies on data underlying IPC-7351, including fabrication,
  assembly, and component tolerance-based mathematical algorithms."
- **LP Wizard** — paid, the full per-family form-driven wizard.
  Predecessor to PADS Land Pattern Creator (which was effectively
  rebranded LP Wizard for many years).
- **Footprint Expert** (the modern flagship) — full library
  authoring tool with IPC-7351B / IPC-7352 / "PCB Libraries"
  selectable naming conventions. **Exports natively to Altium,
  Allegro, Cadstar, Eagle, KiCad (yes), OrCAD, PADS, Pulsonix,
  Pantheon, P-CAD.** "PCB Footprint Expert for KiCad" is a
  specific paid product.
- **IPC role:** Tom Hausherr has chaired and contributed
  extensively to the IPC Land Pattern Committee; he is the source
  of the authoritative public commentary on why IPC-7351C was
  scrapped in favor of IPC-7352.

**Net assessment:** Datum cannot ship a free in-tool IPC-7351
wizard that matches Footprint Expert in package coverage. What
Datum *can* do is implement the core IPC-7351B / IPC-7352
algorithms for a useful subset of families and let users import
Footprint Expert outputs as a parallel path.

#### SnapEDA / SnapMagic

- Rebranded SnapMagic in 2024. Provides community-uploaded and
  vendor-uploaded symbols, footprints, and 3D models for ECAD
  tools. Stated alignment: IPC-7351B Nominal density (Level B).
- Quality story is community-driven; SnapMagic's "Verified"
  workflow runs a programmatic check (silkscreen/pad overlap,
  courtyard sanity) on every footprint and exposes the result on
  the part page.
- Reputation: useful for fast prototypes, less trusted for
  production-class libraries. Most enterprise users treat SnapEDA
  footprints as drafts to be reviewed.

#### UltraLibrarian

- Tagline: "Components based on IPC-7351B standard where
  appropriate." Built through a templating system and verified
  internally before publication.
- Direct manufacturer partnerships with TI, Analog Devices, TE
  Connectivity, etc. — vendor-supplied data is verified by the
  vendor or by Ultra Librarian's internal team before release.
- Higher trust score than SnapEDA in practice. Used by many
  enterprises as the primary external library source.

#### Component Search Engine / SamacSys (Supplyframe)

- 14M+ components. "PCB Footprints created using component
  manufacturer specifications or comply with IPC-7351B. Where a
  component category is not catered for by the IPC standard, they
  use rules developed in collaboration with leading PCB
  manufacturers."
- Drag-and-drop into all major ECAD tools. Generally well-regarded;
  similar trust profile to UltraLibrarian.

#### Octopart libraries / Common Parts Library (CPL)

- Curated subset (the "Common Parts Library") of high-volume
  components. Per Octopart: "All PCB footprints conform to IPC-7351
  standards." Licensed Creative Commons ShareAlike. Library
  development is done by SnapEDA / SnapMagic on Octopart's behalf.
- Smaller than SnapEDA's full catalog; intended as a quality-curated
  baseline.

## Cross-Cutting Patterns

### How tools encode density levels

| Tool | Density-level UI | Encoded in footprint object | Naming suffix |
|------|------------------|-----------------------------|----------------|
| Altium | Wizard dialog page per package; defaults box | No (post-generation, info lost) | Yes (M/N/L if generated by wizard) |
| OrCAD Library Builder | Calculator GUI | Vendor-dependent | Yes |
| Pulsonix LP Expert | LP Expert form | Yes (Footprint Expert metadata) | Yes |
| PADS Land Pattern Creator | Form per family | Yes | Yes |
| KiCad (official library) | None — Level B hardcoded | No | No |
| KiCad (in-app wizards) | Form parameter | No (post-generation) | No |
| Horizon EDA (footag) | Generator parameter | Partially (pool metadata) | No |
| LibrePCB | Multiple footprint variants per package | Yes (variant identity) | Yes (variant name) |
| DipTrace | Pattern Generator form | Pattern templates | Optional |
| Fusion 360 Electronics | Generator form | No | No |

The pattern: commercial tools encode density level at generation time
*and* preserve it in the footprint object's metadata; open-source
tools generally encode it at generation time but lose it in the
footprint object. **LibrePCB is the open-source outlier** — its
multi-variant approach explicitly preserves the choice as part of
the package identity.

**Datum recommendation:** Adopt LibrePCB-style multiple-footprint-
per-package, with density level a first-class identity attribute on
the variant. Do not lose the J-fillet inputs after generation.

### How tools expose IPC parameters in the footprint editor

- **Altium / OrCAD / Pulsonix** — explicit form-driven dialogs at
  package-family granularity. The user sees Jt / Jh / Js as named
  parameters in the dialog.
- **DipTrace** — same, but coverage thin in some BTC families.
- **KiCad in-app wizard** — form-driven but per-family rather than
  unified; some wizards broken.
- **KiCad footprint-generator (Python)** — IPC parameters live in
  YAML files (`ipc_definitions.yaml`); not surfaced in the GUI.
- **Horizon EDA (footag)** — parameters are exposed in the
  generator GUI; preserved in the JSON pool entry.
- **LibrePCB** — variant identity captures the choice; per-variant
  pad geometry is hand-edited rather than parametrically calculated.

The chasm here is exposure: commercial tools make IPC parameters
*editable per footprint*; most open tools make them *editable at
generation time and forgotten thereafter*.

### Naming convention adherence

- **IPC-7351B canonical form:** `<TYPE><PITCH>P<LEAD-SPAN>X<HEIGHT>-<PINS><DENSITY>`
  — e.g., `SOIC127P600X175-8N` = SOIC, 1.27 mm pitch, 6.00 mm lead
  span, 1.75 mm height, 8 pins, Nominal density.
- **IPC-7352 canonical form:** Same prefix-and-suffix structure;
  some packages renamed for unification with through-hole patterns.
- **PCB Libraries' own form:** Pin count moved to the front in some
  variants. PCB Libraries Footprint Expert ships all three as
  selectable defaults.

| Tool | Naming convention |
|------|-------------------|
| Altium IPC Wizard | IPC-7351B canonical |
| OrCAD Library Builder | IPC-7351B canonical |
| Pulsonix Library Expert | Selectable B / 7352 / PCB Libraries |
| Footprint Expert (PCB Libraries) | Selectable |
| PADS LP Creator | IPC-7351B canonical |
| KiCad official library | KLC custom (`SOIC-8_3.2x5.7mm_P1.27mm`) |
| Horizon EDA | Verbose ("DIP8, 10 pins, 2 rows, 2.54 mm pitch") |
| LibrePCB | "Generally follow IPC-7351"; variant suffix |
| DipTrace | IPC-7351-derived |
| SnapEDA / UltraLibrarian | IPC-7351B canonical (vendor-published) |

The KiCad / Horizon divergence here is *intentional* and rooted in
designer-readability arguments. It is also the single biggest
interoperability friction when migrating between tools; designers
who learned IPC-7351B form on Altium / OrCAD do not recognize
`SOIC-8_3.2x5.7mm_P1.27mm` and vice versa.

**Datum recommendation:** Default to IPC-7351B canonical form for
generated footprints; allow the user to choose KLC-style or other
variant per pool. This is a one-line setting that pays huge
interop dividends.

### Validation / DRC alignment to IPC

- **Altium** — rules engine; IPC-2221, IPC-6012 class rules
  encodable; no built-in "IPC class 2 preset" but the rule
  expressions are the standard idiom.
- **OrCAD Constraint Manager** — preset templates for IPC-2221,
  IPC-6012; high-class rules for HDI per IPC-2226.
- **Pulsonix / Cadstar** — class-driven DRC.
- **KiCad** — Custom Design Rules system can express IPC
  clearance; no out-of-the-box preset for IPC-class. Community-
  authored rule sets for JLCPCB and OSH Park exist.
- **LibrePCB** — DRC rules per design rule profile; not
  explicitly IPC-2221-tabular.
- **Horizon EDA** — rule-based DRC; no IPC-class presets.

The pattern: DRC engine capability is comparable across tools;
**preset libraries that map "Class 2" → concrete numerical rules
are a gap in every open tool**. Datum could ship IPC-2221C-derived
clearance presets per Class 1/2/3 with conductor-environment scopes
as a real differentiator.

## Datum EDA Implications

### Data model implications

The most important finding for Datum is that the **canonical IR /
Padstack / Package / Part chain needs additional IPC-derived
metadata fields** to be a credible professional library subsystem:

**Padstack additions:**
- `density_level: enum { Most, Nominal, Least, Custom }` — drives
  regeneration and validation
- `source_jt_mm`, `source_jh_mm`, `source_js_mm: f64` — the three
  fillet allowances used to compute this padstack (preserved so the
  pad can be re-derived if density level changes)
- `ipc_4761_via_type: Option<enum I..VII>` — for vias only; drives
  paste / mask rendering and fab output annotation
- `paste_aperture: Aperture` distinct from `copper_shape` — already
  partially present (`PadAperture`); should be split formalised so
  paste / mask / copper are independent

**Package additions:**
- `source_component_dims: ComponentDims { l_min, l_max, t_min,
  t_max, w_min, w_max, body_x, body_y, height_max }` — preserved
  source dimensions so density-level changes regenerate correctly
- `ipc_naming: NamingComponents { type_prefix, pitch_um,
  lead_span_um, height_um, pin_count, density_letter, special }`
  — structured naming so all conventions can be derived from one
  source of truth
- `ipc_thermal_pad: Option<ThermalPad>` — for BTCs, distinct from
  the perimeter pads

**Project additions:**
- `ipc_class: enum { Class1, Class2, Class3 }` — drives DRC defaults
- `intended_environment: enum { ConsumerSeaLevel, IndustrialSealed,
  Aerospace, Vacuum, HighAltitude }` — drives IPC-2221 column
  selection
- `target_ipc_revisions: { ipc_2221: B|C, ipc_7351: B|7352 }` —
  selectable IPC revisions for backward compatibility

### DRC rule implications

Datum's existing DRC framework (7 rules per CLAUDE.md) should grow
to cover the IPC tables explicitly:

| Datum DRC rule | IPC source |
|----------------|------------|
| Conductor-to-conductor clearance | IPC-2221C Table 6-1 (B1..B5 columns) |
| Annular ring | IPC-A-600 / IPC-6012 Class 1/2/3 |
| Trace current capacity | IPC-2152 Figure 5-2 family |
| Stencil aperture sanity | IPC-7525C area-ratio ≥ 0.66 |
| Via fill required | IPC-4761 (Type V/VI/VII for via-in-pad on BTC) |
| BTC thermal-pad paste coverage | IPC-7093A (50–80% of pad area) |
| Footprint fillet allowance | IPC-7351B / IPC-7352 (Jt/Jh/Js per density level) |

The clearance check is the largest single addition; it is also the
most visible to users (every Altium / KiCad user expects a
voltage-aware clearance check). IPC-2221C selectability versus
IPC-2221B matters because the per-volt step value above 500 V
changed.

### Library tooling implications

- **Footprint wizard family.** Datum needs at minimum:
  - Chip-component wizard (resistor, capacitor, MELF, inductor)
  - Gull-wing wizard (SOIC, SOP, QFP, SOT)
  - J-leaded wizard (PLCC)
  - No-lead BTC wizard (DFN, QFN, SON, LGA — IPC-7093A-aware)
  - BGA wizard (with row/col, pitch, ball diameter)
  - Through-hole DIP / SIP / TO wizard (IPC-7251 / IPC-7352-TH)

  These are the minimum to cover ~95% of common parts. Each must
  expose density level selection, J-fillet override, and component-
  tolerance fields. The PCB Libraries LP Calculator is the
  reference implementation behavior; Horizon's `footag` library is
  a working open-source starting point Datum could study.

- **Naming policy.** Default to IPC-7351B canonical; allow
  KLC-style and IPC-7352 as per-pool overrides. Ship a name-
  validation tool that reports drift from the configured policy.

- **Density level on import.** When importing KiCad/Eagle libraries,
  the density level is unknown. Mark imported padstacks
  `density_level: Custom` with a `provenance: imported_from: <tool>`
  flag rather than guessing.

### Output format implications

- **IPC-2581C export** — should be on Datum's roadmap as a M7-tail
  or M8 deliverable. Schema is published openly; KiCad 8/9 source
  is a reference implementation. Single XML file; covers everything
  Gerber+drill+netlist+BOM+impedance does in one drop.
- **IPC-D-356A export** — small, well-documented, universally
  accepted. Should ship in M7 or shortly after. Format is simple
  enough to write by hand; KiCad's exporter is ~600 lines.
- **Gerber X3 attributes** — adjacent concern; X3's component
  attributes overlap with parts of IPC-2581. Datum's existing
  Gerber export should grow X2 attribute support as a stepping
  stone to X3.
- **Stack-up file** — IPC-2581C has a separable stack-up XML
  schema; useful for impedance vendor exchange.

### Standards Datum should validate against vs reference vs ignore (for M7 vs later)

**M7 must-comply (encode in engine):**
- IPC-7351B / IPC-7352 land pattern math (for the wizard family)
- IPC-7525C stencil aperture area-ratio check
- IPC-2221C Table 6-1 clearance rule
- IPC-4761 Type taxonomy on vias
- IPC-7093A thermal-pad paste-coverage check
- IPC-T-50 vocabulary baseline (everywhere user-visible)

**M7 must-reference (UI labels, project metadata):**
- IPC-A-600 / IPC-A-610 Class 1/2/3 (project setting)
- IPC-J-STD-001 (referenced in fillet-allowance derivation)
- IPC-2222B (rigid-board sectional)

**M8 (future):**
- IPC-2581C export
- IPC-D-356A export
- IPC-2152 chart-based current capacity
- IPC-2226 HDI Type classification (for the stack-up subsystem)

**Out of scope (note only, not validated):**
- IPC-1752 materials declaration (BOM integration, downstream)
- IPC-CM-770 (process-side; vocabulary only)
- IPC-2615 (GD&T; output drawing concern)

## User Pain Points & Wishlist Items

Distilled from KiCad forums, PCB Libraries forum, KLC discussion
threads, Cadence Community, Altium forums, EDABoard, and
DipTrace forum.

1. **"Which density level was this footprint generated at?"** Once
   a footprint is in the library, the IPC parameters are typically
   lost. Users can't tell whether a part is Level A / B / C without
   re-measuring against the datasheet. KLC explicitly does not
   encode density in the name, and the official KiCad library is
   uniformly Level B without that fact being discoverable inside
   the tool.
2. **"Why are 0603 R and 0603 C different?"** Recurring KiCad bug
   report; per IPC-7351 the same package should produce the same
   pads regardless of component class. KiCad has multiple
   historical inconsistencies here.
3. **"The IPC default doesn't actually solder well in our line."**
   Real-world manufacturing experience varies; the IPC defaults
   are starting points, not destinations. Tools should make per-
   shop overrides easy, not bury them. Altium's wizard "Use
   default values" checkbox + explicit override is the right
   pattern.
4. **"Hand-editing footprints is fragile."** Once you tweak a pad
   in the editor, regeneration from the wizard overwrites your
   changes. Wizards that produce "live" parameterised footprints
   (Horizon's parameter programs, OrCAD Library Builder's
   re-runnable templates) solve this; pure one-shot wizards
   don't.
5. **"How do I export IPC-2581 to my fab?"** Increasingly common
   request. KiCad 8.0 unblocked this for the open-source side;
   Datum needs to follow.
6. **"Why is my via-in-pad failing in assembly?"** Recurring BTC
   thermal-pad issue. IPC-7093A's via-fill (Type V/VI/VII)
   requirement is poorly surfaced in most tool defaults; users
   discover it only after a failed prototype.
7. **"My clearance fails at 500 V but passes at 499 V."** IPC-2221's
   step in the spacing formula above 500 V is real, and the
   formula changed in 2221C. Tools should be explicit about which
   IPC revision their clearance rule represents.
8. **"Naming convention drift between tool and library vendor."**
   Altium / SnapEDA / UltraLibrarian use IPC-7351B canonical
   names; KiCad's library uses KLC names. Importing across the
   boundary loses the IPC-encoded information in the name.
9. **"Footprint Expert is great but $$$$."** PCB Libraries'
   Footprint Expert is the gold standard but commercial-only. No
   open-source equivalent at the same package coverage. (This is
   a real gap Datum could partially close.)
10. **"Class 1/2/3 is project-level, not rule-level."** IPC class
    affects every clearance, fillet, annular ring, and silkscreen
    rule. Tools that make it a per-rule setting (every rule has
    its own IPC-class dropdown) get it wrong; the right model is
    project-level Class with rule defaults derived from it. Altium
    handles this awkwardly; Datum could do better.

## Recommendations for Datum EDA

### Must-comply now (M7 must)

1. **Encode density level on every Padstack and Package.** This is
   the single biggest IPC-quality lever. Without it, no footprint
   in Datum can be validated, regenerated, or audited for IPC
   compliance. Add `density_level` enum and source-J fillet fields
   to the Padstack record now while the data model is still
   maturing.
2. **Add `ipc_class` to project metadata** with Class 2 default and
   make every DRC rule's defaults derive from it. Surface it
   prominently in the new-project wizard.
3. **Add `intended_environment` to project metadata** for IPC-2221
   column selection (sea-level uncoated / sea-level conformal /
   vacuum / above-3050m). Default sea-level uncoated.
4. **Adopt IPC-7351B canonical naming as the library default,**
   with KLC-style and IPC-7352 as per-pool overrides. Ship a
   library lint tool that reports name drift.
5. **Use IPC-T-50 vocabulary in all UI labels.** "Annular ring",
   "land", "fillet", "courtyard", "padstack", "trace". This is
   free professional credibility.
6. **Implement IPC-4761 Type I–VII as a Via property.** Drives
   mask/paste rendering and fab annotation. Default Type I
   (tented).
7. **Implement IPC-7525C area-ratio check** on every padstack
   paste aperture against a configurable assumed stencil thickness
   (default 0.12 mm). Warn at < 0.66.

### Library tooling (M7 if possible, M8 if not)

8. **Footprint wizard family** — at minimum, chip / gull-wing /
   no-lead-BTC / BGA / DIP wizards with explicit density-level
   forms. The IPC-7351B math is well-documented; Horizon's `footag`
   is a usable open-source starting point. Each wizard must produce
   regeneratable padstacks (parameter-program-style) so users can
   change one tolerance and refresh.
9. **IPC-7093A BTC wizard with thermal-pad paste segmentation,
   via-in-pad sanity, and IPC-4761 Type V/VI/VII sanity check.**
   This single wizard would put Datum ahead of every open-source
   tool today.
10. **Library import provenance.** Imported padstacks (KiCad,
    Eagle) should be marked `density_level: Custom`,
    `provenance: imported_from: <tool>`, never silently classified
    as Level B.

### Output formats (M7 / M8)

11. **IPC-D-356A export in M7.** Small, table-stakes, no excuse not
    to ship.
12. **IPC-2581C export in early M8.** Schema is published; KiCad
    8/9 is the reference open-source implementation. Single XML
    drop is the modern fab-data answer.
13. **Gerber X2 attributes** as a stepping stone if X3 isn't yet
    feasible.

### Differentiators (where Datum can lead)

14. **Density-level switching as a live operation.** Selecting a
    Package and applying "Recompute at Density Level A" should
    regenerate the padstack from preserved source dimensions. No
    other open tool ships this; Altium does only via rerun-the-
    wizard which discards manual tweaks.
15. **IPC compliance lint report per Project.** A single report
    that shows: every Package's density level, every Padstack
    failing area-ratio, every BTC missing IPC-4761 via-fill, every
    clearance violation against IPC-2221 column for the project
    environment. This is the kind of audit report that gets
    enterprise users' attention.
16. **AI-explanation of IPC violations.** "This pad fails IPC-7525C
    area ratio (0.41 vs 0.66 minimum) because the paste aperture
    is too small for a 0.12 mm stencil — increase aperture to at
    least 0.30 × 0.30 mm or use a 0.10 mm stencil." This plays
    directly to Datum's MCP / AI-first positioning.
17. **IPC-2221C clearance presets per Class 1/2/3 + environment.**
    Ship them as the default rule-set on a new project. Every
    other open tool makes the user write these from scratch.

### Out of scope for M7 (defer or skip)

- IPC-2152 chart-based current capacity — defer to M8 / a thermal
  module
- IPC-2226 HDI Type classification — defer until stack-up
  subsystem matures
- IPC-1752 materials declaration — out of scope; downstream BOM
  concern
- IPC-CM-770 process-side checks — vocabulary only; no validation
- Full IPC-7352 transition — keep it as a per-pool option; do not
  force it ahead of the industry

## Sources

### Primary IPC standards documentation and revision tracking

- [IPC Document Revision Table (electronics.org)](https://www.electronics.org/ipc-document-revision-table) — authoritative revision dates, current/superseded status, and "no longer maintained" flags for every IPC standard
- [IPC Status of Standardization](https://www.ipc.org/Status) — in-progress revision status for IPC standards
- [IPC Recently Released Standards](https://www.electronics.org/recently-released-ipc-standards-and-documents) — current release announcements

### IPC-7351 / IPC-7352 land pattern family

- [IPC-7351B preview pages (ANSI Webstore)](https://webstore.ansi.org/preview-pages/IPC/preview_IPC+7351B-2010.pdf) — official preview of IPC-7351B
- [IPC-7351 product page (IPC Store)](https://shop.ipc.org/ipc-7351/ipc-7351-standard-only/Revision-b/english) — current revision, paywalled
- [The IPC-7351 Standard in PCB Footprints and Land Patterns (Altium Resources)](https://resources.altium.com/p/pcb-land-pattern-design-ipc-7351-standard) — comprehensive Altium reference summary
- [IPC-7351 Complete Guide (PCBSync)](https://pcbsync.com/ipc-7351/) — public-summary covering density levels, naming, fillet calculations
- [IPC 7351 Standards (Sierra Circuits)](https://www.protoexpress.com/blog/features-of-ipc-7351-standards-to-design-pcb-component-footprint/) — public summary
- [IPC-7352 product page (IPC Store)](https://shop.ipc.org/ipc-7352/ipc-7352-standard-only/Revision-0/english) — current IPC-7352 standard
- [IPC-7351 Naming Convention (CSKL via PCBL)](https://www.cskl.de/fileadmin/csk/dokumente/produkte/pcbl/ipc_standard_pcb_library_expert_Land_Pattern_Naming_Convention.pdf) — PCB Libraries reference document
- [SOIC Footprint Naming Decoded (SnapEDA blog)](https://blog.snapeda.com/2018/06/05/deciphering-our-footprint-naming-convention/) — IPC-7351B name structure decode
- [IPC-7351 Footprint Standard (UltraLibrarian)](https://www.ultralibrarian.com/2023/12/21/ipc-7351-explaining-the-standard-for-pcb-footprints-ulc/) — vendor explanation
- [IPC-7251 Generic Requirements for Through-Hole](https://azitech.dk/wp-content/uploads/2023/05/IPC-7251-req-for-Through-Hole-Designs.pdf) — through-hole sibling standard
- [Do We Have a New Release of IPC-7351C? (PCB Libraries Forum)](https://www.pcblibraries.com/forum/do-we-have-a-new-release-of-ipc7351c_topic3150.html) — Tom Hausherr's thread documenting IPC-7351C abandonment in favor of IPC-7352
- [IPC-7351C Land Pattern Overview by Tom Hausherr](https://www.scribd.com/document/689023959/What-is-New-in-IPC-7351C-by-Tom-Hausherr-CEO-Founder-of-PCB-Libraries-Inc-1) — the planned-but-scrapped IPC-7351C content
- [PCB Design Perfection Starts in the CAD Library (IPC E6&S17_01)](https://www.ipc.org/system/files/technical_resource/E6%26S17_01.pdf) — Tom Hausherr conference paper on IPC-7351 implementation

### IPC-7525 (stencils) and IPC-7093 (BTC)

- [IPC-7525B preview pages (ANSI Webstore)](https://webstore.ansi.org/preview-pages/IPC/preview_IPC+7525B-2011.pdf) — official IPC-7525B preview
- [IPC-7525C Table of Contents](https://www.ipc.org/TOC/IPC-7525C_TOC.pdf) — current revision TOC
- [IPC-7525 Explained (PCBSync)](https://pcbsync.com/ipc-7525/) — area ratio, aspect ratio explained
- [What is Area Ratio (BlueRing Stencils)](https://blueringstencils.com/wp-content/uploads/2019/04/What-Is-Area-Ratio-8-24-23.pdf) — area-ratio formula and ≥ 0.66 minimum
- [IPC-7525 Stencil Printing of Small Apertures (Coleman, IPC)](https://www.circuitinsight.com/pdf/stencil_printing_small_apertures_ipc.pdf) — IPC conference paper
- [IPC-7093 Explained (PCBSync)](https://pcbsync.com/ipc-7093/) — BTC standard summary
- [IPC-7093A Table of Contents](https://www.electronics.org/TOC/IPC-7093A-toc.pdf) — current revision TOC
- [Assembly Challenges of Bottom Terminated Components (IPC paper)](https://www.circuitinsight.com/pdf/assembly_challenges_bottom_terminated_ipc.pdf) — BTC manufacturing detail
- [BTC Footprint Guidelines (Cadence Community PDF)](https://community.cadence.com/cfs-file/__key/communityserver-discussions-components-files/28/BTC-footprint-guidelines.pdf) — practical BTC design notes
- [Body of Knowledge for QFN (NASA NEPP)](https://nepp.nasa.gov/files/26515/2013-18_CL14-4477_QFN_Ghaffarian.pdf) — Class-3 perspective on QFN reliability
- [PCB Design Principles for QFN and Other BTCs (IPC)](https://www.electronics.org/system/files/technical_resource/E6&S17_02.pdf) — IPC conference paper

### IPC-2221 (generic design) and IPC-2152 (current)

- [IPC-2221B Generic Standard (electronics.org TOC)](https://www.electronics.org/TOC/IPC-2221B.pdf) — current Rev B (until 2221C uptake) TOC
- [IPC-2221C Standard (IPC Store)](https://shop.ipc.org/ipc-2221/ipc-2221-standard-only/Revision-c/english) — December 2023 revision
- [IPC-2221 Explained (PCBSync)](https://pcbsync.com/ipc-2221/) — public summary covering Table 6-1
- [IPC-2221 Calculator (Altium Resources)](https://resources.altium.com/p/using-an-ipc-2221-calculator-for-high-voltage-design) — clearance calculator with column definitions
- [IPC-2221 PCB Trace Spacing Calculator (SMPS Power Supply)](https://www.smpspowersupply.com/ipc2221pcbclearance.html) — published Table 6-1 spacing values
- [PCB High Voltage Spacing (Siemens, April 2025)](https://blogs.sw.siemens.com/electronic-systems-design/2025/04/29/pcb-high-voltage-spacing-what-every-engineer-should-know/) — IPC-2221C column changes
- [PCB Conductor Spacing Calculator (Sierra Circuits)](https://www.protoexpress.com/tools/pcb-conductor-spacing-and-voltage-calculator/) — IPC-2221 implementation
- [IPC-2152 Standard TOC](https://www.electronics.org/TOC/IPC-2152.pdf) — current revision TOC
- [The Value of IPC-2152 (Mike Jouppi, IPC paper)](https://www.electronics.org/system/files/technical_resource/E7&S22_03.pdf) — author's own discussion of why IPC-2152 superseded IPC-2221's current charts
- [IPC-2152 Calculator (Altium Resources)](https://resources.altium.com/p/using-ipc-2152-calculator-designing-standards) — chart-based current capacity
- [IPC-2152 Explained (PCBSync)](https://pcbsync.com/ipc-2152/) — public summary

### IPC-2226 (HDI), IPC-4761 (vias), IPC-2615 (dimensions), IPC-2222 (rigid sectional)

- [IPC-2226 Explained (PCBSync)](https://pcbsync.com/ipc-2226/) — Type I–VI HDI classifications
- [HDI Design Guidelines (Netvia)](https://www.netviagroup.com/solutions/hdi-pcb-design-guidelines.html) — IPC-2226 practical
- [2+N+2 PCB Stackup (Altium / Zach Peterson)](https://resources.altium.com/p/2n2-pcb-stackup-design-hdi-boards) — sequential lamination explained
- [IPC-4761 PDF (elepcb)](https://www.elepcb.com/wp-content/uploads/2024/04/IPC-4761.pdf) — full document copy
- [IPC-4761 Explained (PCBSync)](https://pcbsync.com/ipc-4761/) — Type I–VII via protection summary
- [IPC-4761 Via Types (Altium)](https://resources.altium.com/p/IPC-vias) — applied per via type
- [Via Plugging / Tenting / Filling (Würth Elektronik)](https://www.we-online.com/files/pdf1/design-guide-plugging-filling-tenting-cbt-en.pdf) — fab-side perspective
- [IPC-2615 Standard TOC](https://www.electronics.org/TOC/IPC-2615.pdf) — printed board dimensions / GD&T
- [IPC-2222 PDF (Internet Archive copy)](https://ia600602.us.archive.org/16/items/ipc_20220718/IPC-2222%20eng_text.pdf) — sectional rigid board standard

### Acceptability and process standards

- [IPC-A-600 Acceptability of Printed Boards (Super Engineer)](https://www.superengineer.net/blog/ipc-a-600) — Rev M (2025) summary
- [IPC-A-610J Announcement (ANSI Blog)](https://blog.ansi.org/ansi/acceptability-electronic-assemblies-ipc-a-610j-2024/) — current Rev J announcement
- [IPC-A-610 Class 1/2/3 (PCBOnline)](https://www.pcbonline.com/blog/ipc-a-610-pcb-assembly.html) — class definitions
- [IPC J-STD-001J](https://www.nextpcb.com/blog/ipc-j-std-001) — current Rev J summary
- [J-STD-001 Class hierarchy (IPC Training)](https://soldertraining.net/blog/three-product-classes-identified-under-jstd001-standard/) — Class 1/2/3 nesting
- [IPC-CM-770 (PCBSync)](https://pcbsync.com/ipc-cm-770/) — component mounting guidelines
- [IPC-T-50 (PCBSync)](https://pcbsync.com/ipc-t-50/) — terms and definitions reference

### Data exchange standards

- [IPC-2581 Consortium homepage](https://www.ipc2581.com/) — official consortium
- [IPC-2581 Support Status](http://www.ipc2581.com/support-status/) — list of tools claiming support
- [IPC-2581 Revision C announcement (IPC)](https://www.electronics.org/news-release/ipc-releases-ipc-2581-revision-c-generic-requirements-printed-board-assembly-products) — Dec 2020 release notes
- [IPC-2581 Rev C Hemant Shah interview (Sierra Circuits)](https://www.protoexpress.com/blog/ipc-2581-revision-c-facilitates-pcb-data-management-by-hemant-shah/) — Rev C feature deep-dive
- [Why IPC-2581 (PCD&F)](https://www.pcdandf.com/pcdesign/index.php/editorial/menu-features/12532-why-ipc-2581-is-the-cad-data-exchange-format-of-today-and-tomorrow) — adoption argument
- [ODB++ vs Gerber X3 vs IPC-2581 (Altium)](https://resources.altium.com/p/pcb-production-file-format-wars) — format comparison
- [KiCad v8 IPC-2581 Q&A with Seth Hillebrand](https://www.ipc2581.com/kicad-v8-exports-ipc2581-qa-with-seth-hillebrand-key-developer-for-kicad/) — KiCad 8 implementation interview
- [KiCad 8.0 Release Announcement](https://www.kicad.org/blog/2024/02/Version-8.0.0-Released/) — IPC-2581 export in v8
- [IPC-D-356A Reference (MSU ATLAS L1Calo)](https://web.pa.msu.edu/hep/atlas/l1calo/hub/hardware/components/circuit_board/ipc_356a_net_list.pdf) — file format specification
- [IPC-D-356 Simplified (Rich Nedbal)](https://web.pa.msu.edu/hep/atlas/l1calo/hub/hardware/components/circuit_board/ipc_356_netlist_format.pdf) — practitioner reference
- [IPC-D-356B TOC](https://www.electronics.org/TOC/IPC-D-356B.pdf) — current revision TOC
- [IPC-D-356 from pcb-rnd-aux](http://repo.hu/projects/pcb-rnd-aux/pool/ipc-d-356/Body.html) — full body text reference
- [Do You Need an IPC-D-356 Netlist (Altium)](https://resources.altium.com/p/do-you-need-ipc-d-356-netlist) — practical use
- [IPC-1752A Standard for Materials Declaration (AcquisCompliance)](https://www.acquiscompliance.com/blog/ipc-1752a-fmd-standard-disclosure-material-declaration-management/) — RoHS/REACH context
- [IPC-1752 (Assent)](https://www.assent.com/resources/knowledge-article/what-is-the-ipc-1752a-standard/) — declaration class A/B/C

### EDA tool implementation

**Altium:**
- [IPC Compliant Footprint Wizard (Altium documentation)](https://www.altium.com/documentation/altium-designer/footprintwizard-dlg-form-footprintwizardipc-compliant-footprint-wizard-ad?version=22) — official wizard documentation
- [IPC Compliant Footprints Batch Generator](https://www.altium.com/documentation/altium-designer/footprintwizard-dlg-form-footprintwizardbatchipc-compliant-footprints-batch-generator-ad?version=22) — batch wizard
- [Create a Footprint using IPC Wizard - Part 1 (PCB-3D)](https://www.pcb-3d.com/tutorials/create-a-footprint-using-ipc-compliant-footprint-wizard-part-1/) — practical walk-through
- [Create a Footprint using IPC Wizard - Part 2 (PCB-3D)](https://www.pcb-3d.com/tutorials/create-a-footprint-using-ipc-compliant-footprint-wizard-part-2/) — wizard density level dialog
- [IPC Compliant Footprint Wizard Features (Altium)](https://resources.altium.com/p/ipc-compliant-footprint-wizard-features-adscvid) — feature overview
- [Working with IPC Compliant Footprint Models (Altium)](https://resources.altium.com/p/working-ipc-compliant-footprint-models) — wizard usage
- [IPC Classes & Complying with IPC Standards for PCB Design (Altium)](https://resources.altium.com/p/complying-with-ipc-standards-for-pcb-design) — Class-1/2/3 design implications
- [Altium Manufacturing Rule Types](https://www.altium.com/documentation/altium-designer/pcb-manufacturing-rules) — DRC engine

**Cadence Allegro / OrCAD:**
- [Cadence IPC-2581 page](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis/pc-design-flows/product-creation/ipc2581.html) — IPC-2581 support
- [IPC-2581 Consortium Update (Cadence)](https://www.cadence.com/content/dam/cadence-www/global/en_US/documents/tools/pcb-design-analysis/pcb-west-2016-f4-update-on-2581-cp.pdf) — consortium presentation
- [OrCAD Library Builder (Parallel-systems)](https://www.parallel-systems.co.uk/orcadlibrarybuilder/) — IPC-driven library tool
- [Allegro Package Symbol Wizard discussion](https://community.cadence.com/cadence_technology_forums/pcb-design/f/pcb-design/34349/allegro-package-symbol-wizard) — Allegro library workflow
- [IPC Web Component Downloads (Cadence Resources)](https://resources.pcb.cadence.com/blog/2020-ipc-web-component-downloads-and-available-footprints) — IPC component story

**PADS / Xpedition:**
- [How the Free Land Pattern Creator in PADS (Siemens)](https://blogs.sw.siemens.com/electronic-systems-design/2014/05/01/how-the-free-land-pattern-creator-in-pads-will-save-you-time-part-1/) — PADS LP Creator
- [PADS LP Viewer download (Siemens)](https://www.pads.com/downloads/lp-viewer-download/) — official PADS landing page
- [Siemens Xpedition IPC-2581 question](https://community.sw.siemens.com/s/question/0D54O00006eo70oSAA/ipc2581) — Xpedition IPC-2581 support
- [Export IPC-2581 from Siemens PADS (CircuitHub)](https://docs.circuithub.com/en/articles/6240954-export-ipc-2581-from-siemens-mentor-pads) — PADS IPC-2581 procedure

**Pulsonix:**
- [PCB Footprint Expert Enterprise for Pulsonix](https://www.pcblibraries.com/products/fpx/Pulsonix.asp) — PCB Libraries' Pulsonix integration
- [Pulsonix News (IPC-7352 support)](https://pulsonix.com/news?ID=12) — IPC-7352 announcement

**KiCad:**
- [Full KLC - KiCad Library Conventions](https://klc.kicad.org/) — KLC homepage with all sections
- [Why are KiCad Library conventions non-IPC compliant (forum)](https://forum.kicad.info/t/why-are-the-kicad-library-conventions-non-ipc-compliant/3678) — community discussion of divergence
- [SMT footprints and IPC-7351B (kicad-users archive)](https://kicad-users.yahoogroups.narkive.com/wduHIdla/smt-footprints-ipc-7351b-software-etc-what-i-learned-so-far) — historical thread on IPC compliance
- [IPC-7351 Footprint Wizard (KiCad forum)](https://forum.kicad.info/t/ipc-7351-footprint-wizard/5856) — wizard discussion
- [KiCad Footprint Wizards GitHub](https://github.com/KiCad/kicad-footprint-wizards) — Python wizard source
- [kicad-footprint-generator on GitLab](https://gitlab.com/kicad/libraries/kicad-footprint-generator) — official footprint generator
- [pointhi/kicad-footprint-generator GitHub](https://github.com/pointhi/kicad-footprint-generator) — community footprint generator
- [ipc_definitions.yaml in kicad-footprint-generator](https://github.com/pointhi/kicad-footprint-generator/blob/master/scripts/Packages/ipc_definitions.yaml) — IPC parameter table source
- [KiCad Wizard Python Errors (GitLab #4896)](https://gitlab.com/kicad/code/kicad/-/issues/4896) — bug report for SOIC/S-DIP wizards
- [SMD Footprints: 0603 R vs C consistency (issue #1137)](https://github.com/KiCad/kicad-library/issues/1137) — IPC-7351 violation report
- [How to Export IPC-2581 from KiCad (PCBSync)](https://pcbsync.com/how-to-export-ipc-2581-files-from-kicad/) — practical guide
- [How to Set Up KiCad Design Rules (Sierra Circuits)](https://www.protoexpress.com/blog/how-to-set-up-design-rules-kicad/) — KiCad DRC capability
- [KiCad Custom DRC Rules for JLCPCB (tinfever GitHub)](https://github.com/tinfever/KiCAD-Custom-DRC-Rules-for-JLCPCB-with-Unit-Tests) — community IPC-style rule sets
- [PCB Footprint Expert for KiCad (PCBL)](https://www.pcblibraries.com/Products/FPX/KiCad.asp) — commercial Footprint Expert KiCad export
- [Adding IPC high density chip footprints (KiCad GitLab #1836)](https://gitlab.com/kicad/libraries/kicad-footprints/-/work_items/1836) — community RFC for IPC density coverage

**Horizon EDA (sources read directly from `research/horizon-source/`):**
- [Horizon Pool Convention GitHub](https://github.com/horizon-eda/horizon-pool-convention) — pool naming and padstack conventions
- [Horizon Footprint Generator PR #287](https://github.com/horizon-eda/horizon/pull/287) — IPC-7351B generator merge
- [Horizon Pool Elements documentation](https://docs.horizon-eda.org/en/latest/pool-elements.html) — official Horizon pool docs
- [Horizon Custom Padstack Discussion](https://horizon-eda.discourse.group/t/custom-padstack-parameters/79) — community thread

**LibrePCB:**
- [LibrePCB Package Conventions](https://librepcb.org/docs/library-conventions/packages/) — IPC-7351 compliance language

**Consumer / cloud:**
- [DipTrace IPC-7351 Pattern Generator (forum)](https://diptrace.com/forum/viewtopic.php?t=13940) — built-in generator discussion
- [DipTrace Pattern Editor Help (PDF)](https://diptrace.com/books/help/PattEditHelp.pdf) — official help including pattern generator
- [DipTrace What's New](https://diptrace.com/diptrace-software/whats-new/) — recent IPC-7351 wizard family additions
- [EasyEDA Footprint editing](https://docs.easyeda.com/en/PCBLib/PCBLib-Edit/) — footprint editor docs
- [Fusion Electronics package generator (Autodesk)](https://forums.autodesk.com/t5/eagle-forum/ipc-7351-compliant-design-rules-for-eagle/td-p/12634399) — IPC compliance discussion

**Reference implementations & vendors:**
- [PCB Libraries Tom Hausherr bio](https://www.pcblibraries.com/aboutus/Tom.asp) — author of IPC-7351 commercial implementation
- [Free IPC-7351 Land Pattern Calculator (PCB Libraries)](https://www.pcblibraries.com/forum/ipc7351-smd-pth-reference-calculators_topic785.html) — free LP Calculator
- [LP Wizard and Footprint Expert Comparison (PCB Libraries)](https://www.pcblibraries.com/products/compare/lpwizard-libraryexpert/) — product line comparison
- [Free IPC-7351 LP Calculator (Electronic Design)](https://www.electronicdesign.com/technologies/industrial/boards/article/21776916/free-ipc-7351-land-pattern-calculator-aids-board-designers) — launch coverage with Hausherr commentary
- [IPC Releases Free Land Pattern Calculator (I-Connect007)](https://iconnect007.com/index.php/article/44012/ipc-releases-free-land-pattern-calculator/44015) — IPC's own announcement
- [Zuken / PCB Libraries CADSTAR LP Wizard launch (CIMdata)](https://www.cimdata.com/newsletter/2006/11/03/11.03.17.htm) — historical context
- [SnapEDA Symbol & Footprint Standards](https://www.snapeda.com/standards/) — SnapEDA's IPC-7351B Nominal commitment
- [SnapEDA IPC-7351 SOIC Specification](https://blog.snapeda.com/2015/07/13/the-ipc-7351-specification-explained-soic-components/) — public SOIC explanation
- [Ultra Librarian Standards page](https://www.ultralibrarian.com/about/standards) — vendor IPC commitment
- [SamacSys PCB Library Standards](https://www.samacsys.com/pcb-library-standards/) — vendor library standards page
- [Octopart CPL on GitHub](https://github.com/octopart/CPL-Data) — Common Parts Library data
- [Octopart CPL KiCad Library](https://github.com/octopart/CPL-KiCad-Library) — KiCad-format CPL
- [Why Symbols, Footprints & 3D Models Matter (Octopart)](https://octopart.com/pulse/p/symbols-footprint-3d-models) — IPC-7351 commitment statement

### IPC training and education

- [IPC Edge Training](https://education.ipc.org/) — IPC's online training platform
- [IPC PCB Design Curriculum](https://education.ipc.org/ipc-pcb-design-curriculum) — formal design course
- [IPC Designer Certification (eptac)](https://www.eptac.com/class/ipc-designer-certification) — Certified Interconnect Designer training
- [Is IPC CID Training Important (Altium)](https://resources.altium.com/p/is-ipc-cid-training-important-for-your-design-career) — designer certification context
