# Library Bootstrap & Content Strategy — Open Discussion

> **Status**: Open discussion — NOT ratified. Captured to resume later; when
> resolved, ratify into a numbered `docs/decisions/PRODUCT_MECHANICS_0xx`
> record and wire it into governance (PROGRESS row + gate).
> **Owner decisions still pending**: marked **[OWNER]** below.

## The problem
Datum deliberately ships **no third-party library content** (ratified path-B, to
avoid CC-BY-SA/copyleft entering Datum's distributable). That is correct, but it
exposes a bootstrap gap: with no shipped library, an incomplete importer, no
symbol generator, and no visual library editor, a user cannot yet sit down and
design a board from scratch. This document captures the strategy discussion for
how Datum's library gets populated — and to Datum's strict quality bar.

## What the research established (commercial EDA library sourcing)
Deep-research pass across Altium, Cadence/OrCAD, Siemens/PADS, the third-party
content ecosystem, and KiCad/Horizon (22 sources, adversarially verified):

- **No commercial tool ships a giant static library as the primary answer.** The
  model is a **hybrid**: a modest shipped baseline + built-in generation + heavy
  reliance on cloud/vendor content and the cross-tool per-part ecosystem.
- **Generation is a complement, not the primary path.** The claim that Altium
  positions its IPC Footprint Wizard as the primary workflow was *refuted* in
  verification — Altium's primary path is **Manufacturer Part Search (cloud) +
  managed user libraries**; the wizard fills gaps.
- **Built-in IPC generators are table stakes**: Altium IPC-Compliant Footprint
  Wizard, PADS (free) IPC Land Pattern Creator, OrCAD wizards.
- **The dominant real-world workflow is per-part fetch by MPN** from SamacSys /
  Component Search Engine / Library Loader, Ultra Librarian, SnapEDA, Octopart,
  and distributor downloads (Digi-Key, Mouser).
- **Altium replaced the local Content Vault with cloud Manufacturer Part Search /
  Altium 365 components** — linked cloud content, not a big local static set.
- **KiCad/Horizon are the outliers** — they ship large community-maintained
  bundled libraries because they are open-source (KiCad libs: CC-BY-SA with a
  design-use exception; Horizon pool: its own license).
- **Licensing**: third-party content (SnapEDA/UL/SamacSys) is generally
  free-to-use in commercial designs but **not redistributable/bundleable** — it
  must be **user-fetched**, never shipped by a tool vendor. This confirms
  path-B's "don't bundle" instinct.

**Bottom line:** professionals do NOT expect a huge shipped baseline; they expect
a working *population mechanism* (generate + fetch + own libraries) plus a small
generic starter set. Datum's instinct is industry-aligned; the gap is execution.

## The three-leg model (derived)
- **Leg A — per-part acquisition (PRIMARY):** import an existing library file
  *and/or* fetch-by-MPN. This is the dominant real workflow and the #1 gap
  (KiCad symbol/schematic bulk import is stubbed; only Eagle `.lbr` fully works).
- **Leg B — IPC generation (FALLBACK):** generate from datasheet dimensions when
  a part isn't available. Datum has the IPC-7351B two-terminal chip generator;
  expand family-by-family (SOIC/SOT → QFN → BGA).
- **Leg C — first-run baseline (SEED):** a small Datum-authored/generated set so
  the tool isn't empty on launch. All Datum-owned IP → bundleable.

## Derived decisions (proposed)
- **D1** — Sourcing is three-legged; the PRIMARY leg is acquisition, not
  generation (correcting the earlier over-emphasis on generation).
- **D2** — Licensing rule: Datum ships ONLY content it generates/authors itself;
  third-party content is user-fetched at runtime, never bundled.
- **D3** — One normalization pipeline (`acquire → normalize → commit()` with the
  Import Map), not N bespoke importers. Finish the KiCad importer first.
- **D4 [OWNER]** — Offline-first; online fetch-by-MPN (SnapEDA/Octopart/Digi-Key)
  is an optional, explicit, provenance-tracked layer built after the file path.
  *Recommendation: yes to optional online, never required/core. Veto → strictly
  offline-file-only.*
- **D5** — Ship a minimal license-clean baseline: IPC-generated chip footprints
  (0201–1206 R/C/L, density A/B/C) + ~12 native generic symbols (R, C, L,
  diode/LED, BJT, MOSFET, op-amp, connector, generic IC, power/ground ports).
- **D6** — Generator expands family-by-family; each family is a tracked item.

## The compliance-normalization "forge" (owner suggestion — strong)
Rather than trusting imported parts as-is, every library object — imported,
converted, OR generated — passes through a **standards gate** that either
**normalizes it to compliance or rejects it**, and stamps a compliance
attestation. This is the durable, license-clean asset (it operates on
Datum-native objects) and the quality spine across all three legs. It enforces:

- **Completeness**: a footprint must have pads, courtyard, silkscreen, assembly
  outline, refdes, solder-**mask** expansion, solder-**paste** aperture — reject
  if a required layer is missing.
- **Geometry to IPC + DFM**: pad corner radius = `clamp(ratio · min(w,h),
  min_r, max_r)` (Datum already models `roundrect_rratio`, 25% default);
  courtyard excess per density; silk-to-pad clearance; mask expansion; paste
  ratio. (This also finally wires up the mask/paste-in-export governance gap.)
- **Symbols to IEEE-315 / IEC-60617**: pin electrical types present, refdes
  prefix, style-profile conformance.

### Content-sourcing fork for the forge **[OWNER]**
- **Path A — oracle + re-derive (cleanest, matches path-B):** use open libraries
  (KiCad/Horizon/Eagle) as a *coverage map + parameter source + validation
  oracle* (dimensions, pin counts, pinouts are facts, not copyrightable), then
  **re-derive geometry natively** from IPC. Output is Datum-owned IP, no
  copyleft. Symbols generated from the pin-list facts in Datum's own style.
- **Path B — convert verbatim, ship as a separate attributed share-alike data
  package** (KiCad's own model: unencumbered engine, separately-licensed libs).
  Fast large corpus, but the shipped *library package* carries CC-BY-SA
  (attribution + share-alike); the engine/native format stay clean.
- *Recommendation: Path A for anything Datum ships. Path B only if a
  separately-licensed data package is acceptable.*
- **Hard prerequisite:** a real licensing determination before shipping any
  content derived from CC-BY-SA/third-party sources. Conversion does NOT launder
  share-alike; a derivative of CC-BY-SA content remains CC-BY-SA.

## Sequencing (proposed reprioritization)
1. KiCad importer (Leg A file path) — the unblocker. Highest priority.
2. Compliance-normalization forge — needed on every path.
3. Baseline starter set (Leg C) — cheap, high first-run value.
4. Generator families (Leg B) — SOIC/SOT next.
5. Optional online fetch (Leg A cloud, D4) — later.
6. Visual library editor — GUI phase; authoring-via-operations covers it until then.

This library work outranks the enforcement/governance plumbing that is now in
place — it is what actually blocks the product from being usable.

## Open questions to resolve before ratifying
- **[OWNER] D4**: optional online fetch, or strictly offline-file-only?
- **[OWNER] Content path A vs B** for the forge.
- **Licensing determination** on any CC-BY-SA-derived content (gates Path B).
- Baseline scope confirmation (D5 list).
