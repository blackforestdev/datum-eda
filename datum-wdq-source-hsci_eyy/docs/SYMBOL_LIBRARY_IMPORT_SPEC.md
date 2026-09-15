# Symbol / Library Import (KiCad · Horizon → native) — Spec

> **Status**: Proposed spec for the per-user symbol-library import on-ramp.
> **Decision context**: implements distribution **path B (import-per-user)** —
> Datum bundles **no** third-party library content; each user imports a library
> into *their own* pool. See `PRODUCT_MECHANICS_008A` for the model deltas this
> spec depends on (notably D2, pin graphic style).
> **Companion to**: `specs/IMPORT_SPEC.md`, decision-008.
>
> Current implementation boundary: Datum now has a checked-in authored native
> library baseline fixture (`crates/test-harness/testdata/library/native_authored_baseline_v1`)
> that resolves and validates Unit/Symbol/Entity/Package/Footprint/Padstack/
> Part/PinPadMap records without import-derived content. That fixture proves
> the native pool substrate; this import spec remains the proposed per-user
> compatibility on-ramp.

## 1. Intent & North-Star posture

Professionals will not hand-draw thousands of symbols. This import is the
**content on-ramp** that populates a native pool by **normalizing** foreign
symbols into Datum's model — not a raw geometry lift, and not a viewer. The
output is **native Datum library objects**; the import is a compatibility path,
not the library's foundation. Existing machinery to build on:
`import/kicad/symbol_helpers.rs`, `import/eagle/pool_builder.rs`, and the
substrate **Import Map**.

## 2. Licensing posture (why path B)

- **User-initiated, into the user's pool.** This is "use of library data in a
  project" — KiCad and Horizon libraries (both CC-BY-SA 4.0) **explicitly waive
  share-alike for designs/projects that use them**. The user incurs no
  obligation; their designs are unencumbered.
- **Datum distributes nothing.** No converted library ships in the box, so no
  CC-BY-SA collection obligation attaches to Datum.
- **Provenance preserved.** Each imported pool object records source tool,
  source license + attribution, `source_path`, and `source_hash` in its
  `LibraryProvenance` — honest provenance, and the basis for re-import
  reconciliation.

## 3. Scope

**In:** KiCad symbol libraries (`.kicad_sym`) and Horizon pool
units/symbols/entities/parts (JSON). Produces native `Unit`, `Symbol`, `Gate`,
`Entity`, and (where present) `Part`/`Package`/`Pad`/`Padstack` pool objects.
**Out:** bundling content; footprint copper/courtyard/3D fidelity beyond the
model; full schematic/board import (separate); cloning a library editor.

## 4. Target = native pool types

The converter populates `crates/engine/src/pool/mod.rs` types
(`Unit`/`Symbol`/`Gate`/`Entity`/`Package`/`Pad`/`Padstack`/`Part` + `pad_map`)
through the **operation/commit/journal** path — import is a journaled
`OperationBatch` with `provenance = import`, fully undoable. **No private writer.**

## 5. Field mapping — KiCad `.kicad_sym`

| KiCad | → Datum | Notes |
|---|---|---|
| symbol (top) | `Entity` (+ default `Part`) | multi-unit symbol → multiple `Gate`s under the entity |
| symbol unit *n* | `Unit` + `Symbol` + `Gate` | KiCad "unit 0" (common) pins → a shared/power gate (008A-D6) |
| pin `name` | `Pin.name` | logical name |
| pin `number` | `Pad.name` via `pad_map` | physical designator; many-pads→one-pin supported |
| pin `electrical_type` | `Pin.direction` (electrical type) | via the mapping table in §7 |
| pin graphic `style` / `length` / orientation | `SymbolPinAnchor.style` | **requires 008A-D2** (clock, inverted-dot, active-low…) |
| pin alternate functions | `Pin.alternates` | name (+ type/graphic per 008A-D4) |
| hidden power pin | `Pin` + implicit-power flag | connectivity convention (008A-D5) |
| symbol graphics (rect/polyline/text) | `Symbol.drawings: Vec<Primitive>` | `Primitive::{Line,Rect,Circle,Arc,Text}` |
| `Reference` prefix / `Value` / `Datasheet` | `Symbol` fields / `Part.{value,datasheet}` + `default_refdes_prefix` | fields per 008A-D3 |
| `Footprint` field + footprint filters | `Part.package` binding + filter metadata | may be unresolved at symbol-only import |
| `ki_keywords` / custom props | `Part.parametric` / `tags` | |

## 6. Field mapping — Horizon pool

Horizon's model is the lineage of Datum's, so the structural map is near 1:1
(`Unit`→`Unit`, `Symbol`→`Symbol`, `Gate`→`Gate`, `Entity`→`Entity`,
`Part`→`Part`, `Package`/`Padstack`→same). Work is identity remap (assign Datum
`ObjectId`/UUIDs, record Import Map keys) + field copy + electrical-type
normalization (§7). Highest-fidelity source; smallest converter.

## 7. Canonical electrical-type mapping (normalization)

Map each source pin type onto Datum's canonical electrical type (008A-D1),
which is the value ERC consumes:

| KiCad type | Datum |
|---|---|
| `input` | Input |
| `output` | Output |
| `bidirectional` | Bidirectional |
| `tri_state` | TriState |
| `passive` | Passive |
| `power_in` | PowerIn |
| `power_out` | PowerOut |
| `open_collector` | OpenCollector |
| `open_emitter` | OpenEmitter |
| `unspecified` / `free` | Passive *(with a warning)* |
| `no_connect` | NoConnect |

Horizon types map directly (same taxonomy lineage). Any source type without a
canonical target is imported as `Passive` **and logged as a normalization
warning** — never silently dropped.

## 8. Identity & re-import (idempotency)

Use the substrate **Import Map**: each imported object gets an `import_key`
derived from the source identity + `source_hash`. Re-importing the same library
is idempotent; changed source objects update in place; objects absent from the
latest source are marked `missing_in_source` (reuse the existing KiCad/Eagle
Import-Map lifecycle). This makes the on-ramp repeatable and reviewable, not a
one-shot dump.

## 9. UX

One command, into the user's pool, through the normal mutation path:
`datum-eda pool import-symbols --from {kicad|horizon} <path>` (CLI) — plus the
equivalent engine op and MCP tool. Reports: imported counts, normalization
warnings, unresolved footprint bindings.

## 10. Proof gate (fidelity)

Import a pinned real KiCad symbol library fixture and assert, per symbol:
- pin **count**, **name**, **number/designator**, and **electrical type** preserved;
- **multi-unit** structure preserved (units → gates);
- **pin graphic style** preserved (after 008A-D2);
- pad-map cardinality preserved (incl. a many-pads→one-pin case);
- **ERC-equivalence** on a small sample net (the imported pin types drive the
  same ERC verdict as the source tool's intent).
Normalization warnings are part of the expected output, not failures.

## 11. Non-goals

Bundling/redistributing library content; footprint copper/3D fidelity; full
schematic or board import; preserving source-tool-specific rendering pixel-for-pixel.
