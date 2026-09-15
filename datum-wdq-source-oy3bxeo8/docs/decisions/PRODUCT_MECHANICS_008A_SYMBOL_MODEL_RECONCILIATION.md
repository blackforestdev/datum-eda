# Product Mechanics 008A: Symbol / Library Model Reconciliation

> **Status**: Proposed addendum to `PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM`.
> **Purpose**: Reconcile Datum's symbol/library model against (a) a professional
> "minimum seriousness" capability baseline and (b) the current Rust pool types,
> and record the precise deltas to close. Not a rebuild — Datum already meets or
> exceeds most of the bar.
> **Evidence base**: an adversarially-verified research pass (25/25 claims
> confirmed, all primary sources). **Caveat**: evidence is Altium- and
> KiCad-strong; Cadence is thin (one source) and Siemens/Zuken largely
> uncaptured — treat as an Altium/KiCad-grounded baseline.

## A. Minimum-seriousness baseline (verified)

| # | Capability | Evidence (tool) |
|---|---|---|
| 1 | Component = container linking to symbol + footprint models (symbol is graphical only) | Altium ("a Component… links to these models"); KiCad symbol `Footprint` field |
| 2 | Pin **electrical-type** taxonomy (Input/Output/I-O/Passive/Power(In/Out)/OpenCollector/OpenEmitter/HiZ/Tristate) | Altium fixed 8 (4 docs); KiCad adds Power-In/Out, Bidirectional, Tri-state |
| 3 | Pin electrical type **drives ERC** | Altium ("used when compiling… to detect electrical connection errors") |
| 4 | Pin **number (designator)** distinct from pin **name** | Altium pin properties; SnapEDA |
| 5 | Heterogeneous **multi-unit/gates** + shared power across units ("Part Zero") | Altium "Part Zero… power pins in all parts"; KiCad multi-unit + unit letter |
| 6 | Pin→pad map is **not 1:1** (multiple pads → one logical pin) | Altium SOT223 "two pads sharing designator '2'" |
| 7 | **Padstack** = distinct, reusable named entity beneath the footprint | Cadence (named padstacks from PADPATH) |
| 8 | Immutable **versioned revisions** + **lifecycle status** | Altium (released revision "closed"; Planned→Production→Deprecated→Obsolete) |
| 9 | Pin **graphic styles** (clock, inverted/dot, active-low) + hidden/implicit power pins | KiCad alternate styles; invisible Power-Input → global net |
| 10 | Per-pin **alternate functions** (alt name/type/graphic at placement) | KiCad alternate pin functions |
| 11 | Pin **swap groups** (and gate swap) | Altium/Cadence swap |
| 12 | **Parametric / MPN / supplier / inheritance** on the part | Altium component params; SnapEDA |

## B. What Datum already covers (verified against `crates/engine/src/pool/mod.rs`)

| Baseline | Datum | Status |
|---|---|---|
| 1 separation | `Part → Entity/Unit/Symbol` + `Package/Pad/Padstack` + first-class `PinPadMap` | ✅ **exceeds** — `Unit` (electrical interface) is separated from `Symbol` (drawing), more granular than Altium |
| 2 electrical type | `pool::PinDirection` aliases `LibraryPinElectricalType` (10 values, ⊇ Altium's 8); schematic `PinElectricalType` is now the same type alias | ✅ partial — library taxonomy is the schematic/ERC authority; persisted CheckRun evidence still needs explicit binding/taxonomy revision context (see D1) |
| 4 number vs name | `Pin.name` (logical) + `Pad.name` (designator) linked by `PinPadMap` | ✅ architectural |
| 5 multi-gate | `Entity.gates: HashMap<_, Gate>`, `Gate { unit, symbol }` | ✅ (shared-power = convention, see D6) |
| 6 many-pads→one-pin | `PinPadMap { part, footprint?, mappings: pad → {gate, pin} }`; `Part.pad_map` is fallback/import compatibility only | ✅ partial — first-class authority exists and is gate-aware; full migration policy remains |
| 7 padstack | `Padstack { aperture, drill_nm, plated, layer_span, mask_policy, paste_policy, annular_ring_nm, thermal, antipad }` | ✅ partial — schema depth exists; downstream consumption remains incomplete |
| 8 lifecycle | `Part.lifecycle = Active/Nrnd/Eol/Obsolete/Unknown` | ✅ partial (revision guards/object revisions exist; release immutability policy still needs library-level closure) |
| 10 alternates | `Pin.alternates: Vec<AlternateName>` | ✅ partial (see D4) |
| 11 swap | `Pin.swap_group: u32` | ✅ (gate swap — confirm) |
| 12 parametric/MPN/etc. | `Part.{mpn, manufacturer_jep106, parametric, base, orderable_mpns, supply_chain_offers, behavioural_models, thermal, packaging_options}` | ✅ **exceeds** |

**Headline:** the model is professional-grade. The bar is largely met. The
remaining work is narrow, targeted deltas — not a redesign.

## C. Deltas to close

**D1 — Reconcile the two pin-type enums (table-stakes).**
The duplicate enum has been removed: `pool::PinDirection` is now a compatibility alias, `schematic::PinElectricalType` is now the pool-owned
`LibraryPinElectricalType` alias, and ERC tests lock the richer library roles
(`OpenCollector`, `OpenEmitter`, `TriState`, `NoConnect`) as the
classification input. The remaining target is evidence depth, not enum
ownership: persisted CheckRun findings should expose enough model revision,
library binding revision, and taxonomy context to prove which library pin type
was checked. *Acceptance:* a placed library pin's canonical electrical type is
the value ERC checks, including persisted CheckRun evidence tied to the resolved
binding.

**D2 — Pin graphic style on the symbol (table-stakes; the converter needs it).**
Compatibility summary: `SymbolPinAnchor` now carries `orientation` semantics via
the nested `style.orientation` field, not as a new top-level authoring field.
`SymbolPinAnchor` now carries `position` plus an engine-owned nested `style`
with `orientation`, optional `length_nm`, and `decoration`
(`none | inverted | clock | inverted_clock`). Legacy top-level style fields are
accepted on read, but new CLI-authored anchors write the converged `style`
shape and placed-symbol pin queries expose it for pool-backed symbols. This
closes the first schema slice, but the target remains broader:
preserve/import/export active-low and alternate-function style semantics without
collapsing distinct source styles into one decoration.

**D3 — Converge the Rust `Symbol` type to the decision-008 schema (convergence-debt).**
decision-008 `Symbol` specifies `fields[]`, `style_profile_assertions[]`
(IEEE-315 / IEC-60617 / JIS-C-0617), `standards_basis[]`,
`default_refdes_prefix`, and `check_state`. The Rust `Symbol` now carries these
fields plus provenance, drawings, and pin anchors. Remaining convergence debt is
behavioral: rendering/check/import/export parity must consume these fields as
first-class schema, not only store them.

**D4 — Alternate-function parity (differentiator, trending table-stakes).**
`AlternateName { name, kind }` → optionally carry per-alternate **electrical type
and graphic style** (KiCad alternate pin functions for modern MCUs).

**D5 — Hidden/implicit power pins (behavior).** Specify the
invisible-power-input → same-named-global-net convention as ERC/connectivity
behavior (a model flag + a connectivity rule).

## D. Classification

- **Table-stakes (close):** D1, D2, D3.
- **Differentiator (schedule):** D4; gate-swap completion.
- **Already spec'd in 008 (implement, not decide):** IEEE/IEC/JIS style profiles
  (`style_profile_assertions`), parametric/MPN/supplier/lifecycle.
- **Genuine product judgment (owner):** how many style profiles to ship in v1;
  depth of alternate-function support in v1; De Morgan alternate symbols.

## E. Confirm items (verify before marking done)
- Library **revision immutability** semantics vs object revisions/model revision.
- Whether decision-008 §Symbol `pins[]` should name active-low separately from
  the current `inverted` decoration (D2).
- Gate-swap representation (pin swap via `swap_group` is present).
