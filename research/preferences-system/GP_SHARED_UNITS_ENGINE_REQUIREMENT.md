# Shared Units Engine Requirement

> **Status:** Controlling owner-directed requirement; UNIT-I02R reconciliation
> is open for protected-target and owner review. UNIT-I03 implementation,
> production acceptance, and dependencies are not authorized.
>
> **Trackers:** `dat-global-preferences-engine-qcv` (ratified Preferences
> authority) and `dat-shared-units-surface-parity-s9x` (UNIT-I02R/UNIT-I03).

## Authority and current gap

Datum's canonical authored length authority is already exact: coordinates,
dimensions, and distances are signed `i64` nanometers
(`docs/CANONICAL_IR.md:59-77`; `crates/engine/src/ir/geometry.rs:3-15`). The
native-write path consumes normalized typed values, and current MCP board
schemas expose integer `_nm` fields (`mcp-server/tools_catalog_datum.py:194-205`).

The current conversion surface is not one units engine. Engine helpers convert
mm/mil/inch through `f64` (`crates/engine/src/ir/units.rs:1-26`), Gerber owns a
private fixed-six-decimal parser/formatter
(`crates/engine/src/export/formatting.rs:1-30`), and other import, preview,
save, CLI, GUI, and MCP paths retain independent conventions. There is no shared
quantity-aware suffix parser, display-unit resolver, precision policy, or
cross-surface provenance answer.

The working prototype renders the intended user model at
`docs/gui/prototypes/preferences-window.html:110-120`: one measurement system,
per-quantity display choices, display-only precision including exact nanometers,
and unit-suffixed input served by one implementation. This document governs the
engine seam only; it does not ratify every current control label or default.

## Required engine contract

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:SHARED-UNITS-ENGINE -->

1. Datum must provide one engine-owned Units service used by GUI, CLI, and MCP
   edge adapters. A caller-specific parser or formatter may wrap the service but
   cannot define rival unit semantics.
2. Canonical authored length remains signed `i64` nanometers. Measurement
   system, display unit, and precision are projections only; they never mutate,
   quantize, round, or reserialize stored geometry.
3. `MeasurementSystem` initially distinguishes `Metric` and `Imperial` and
   supplies defaults for quantity descriptors that follow the system. It is not
   storage authority.
4. Each supported physical quantity has a typed quantity kind and a typed
   display-unit choice. Per-quantity overrides may select a unit from either
   measurement system. A cross-system choice is valid but returned and rendered
   as an explicit override rather than silently normalized away.
5. `DisplayPrecision` controls formatting only and includes an exact-nanometer
   mode. Rounded display text cannot be written back as if it were the exact
   stored value.
6. The shared parser accepts locale-independent decimal length expressions with
   explicit supported suffixes, initially including `nm`, `µm`/`um`, `mm`,
   `mil`, and `in`. Inputs such as `5mm`, `200mil`, and `0.1in` use identical
   semantics in GUI, CLI, and MCP.
7. Conversion uses checked integer/rational arithmetic. Overflow, malformed or
   ambiguous suffixes, wrong quantity kinds, and non-integral-nanometer results
   are typed refusals. Any future explicit rounding mode requires a separate
   governed decision and visible provenance.
8. Bare numbers are accepted only when the caller supplies the field
   descriptor's resolved display-unit context. Automation cannot depend on an
   unstated machine preference; CLI/MCP requests must carry or resolve explicit
   context, while a suffix remains context-independent.
9. Parse and format results expose typed provenance: quantity kind, authored
   token, explicit-versus-contextual unit, resolved unit, exact canonical value,
   display precision, cross-system-override state, and refusal reason where
   applicable.
10. Existing native operations and persisted design schemas continue to receive
    integer nanometers. Adding expression input to CLI/MCP requires a compatible
    edge-adapter/schema migration; it cannot silently replace established `_nm`
    integer fields.
11. Length, angle, ratio, percentage, frequency, and other quantities remain
    distinct types. The length suffix set cannot leak into unrelated fields.
12. The final Preferences descriptor catalog must separately classify the
    measurement-system default, each quantity display override, and each
    precision policy while pointing them to this one engine service.

## Required later proof

Implementation planning must include:

- exact equivalence such as `5.08mm == 200mil == 0.2in == 5_080_000nm`;
- negative values, signed limits, overflow, whitespace/case policy, Unicode
  `µm` and ASCII `um`, malformed input, and sub-nanometer refusal;
- format/parse round trips wherever the selected representation is exact;
- proof that display precision and system changes leave stored bytes unchanged;
- identical explicit-suffix results through GUI, CLI, and MCP adapters;
- contextual bare-number proof naming the resolved field/unit;
- migration compatibility for existing integer `_nm` callers.

The intentionally different examples `5mm`, `200mil`, and `0.1in` demonstrate
suffix acceptance; they are not asserted to be equivalent values.

## UNIT-I03 reconciliation contract

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R -->

This section translates the twelve owner requirements and seven proof families
above into the exact boundary that must be reviewed before UNIT-I03 may start.
It does not replace or weaken those requirements. Planning evidence, an HTML
render, or a passing isolated core test is not implementation authorization or
production acceptance.

### Typed profile and resolution

The preference-facing value is one typed `UnitsProfile`, composed from exactly
the six registered `datum.units.*` descriptors:

```text
UnitsProfile {
  system: MeasurementSystem,
  board_length: LengthUnitChoice<BoardLayout>,
  drill_hole: LengthUnitChoice<DrillHole>,
  schematic_geometry: LengthUnitChoice<SchematicGeometry>,
  length_precision: LengthPrecision,
  angle_format: AngleFormat,
}

LengthUnitChoice<Q> = FollowSystem | Explicit(Q::AllowedUnit)
LengthPrecision = DecimalPlaces(1 | 2 | 3 | 4) | ExactNanometer
AngleFormat = DecimalDegrees(DegreePlaces) |
              DegreesMinutesSeconds(SecondPlaces) |
              Radians(RadianPlaces)
```

The generic notation above is a type relationship, not a requirement for Rust
generics. Invalid quantity/unit pairs must be unrepresentable after descriptor
validation. The service returns a `ResolvedUnitsProfile` containing concrete
units, the unchanged precision and angle policies, and an `OverrideState` for
each quantity. It must not consult ambient preferences internally; callers pass
the six already-resolved descriptor values as one snapshot.

`FollowSystem` resolves by this complete V1 table:

| Quantity | Metric | Imperial | Explicit choices |
|---|---|---|---|
| Board/layout length | `mm` | `mil` | `mm`, `um`, `mil`, `inch` |
| Drill/hole size | `mm` | `inch` | `mm`, `mil`, `inch` |
| Schematic geometry | `mm` | `mil` | `mm`, `mil` |

An explicit unit never changes when the measurement system changes. If its
system differs, resolution succeeds with `CrossSystemOverride`; GUI,
provenance, CLI, and MCP query output must preserve that fact. `nm` remains an
explicit expression suffix and exact-display unit, but is not a selectable V1
per-quantity override.

The descriptor tokens map to the formatter without inference:

| `datum.units.length_precision` | Engine value | Meaning |
|---|---|---|
| `0.1` | `DecimalPlaces(1)` | one place in the resolved display unit |
| `0.01` | `DecimalPlaces(2)` | two places in the resolved display unit |
| `0.001` | `DecimalPlaces(3)` | three places in the resolved display unit |
| `0.0001` | `DecimalPlaces(4)` | four places in the resolved display unit |
| `exact_nm` | `ExactNanometer` | signed canonical integer plus `nm` |

Decimal display may round half away from zero and must return `rounded=true`
when information was hidden. Display text is never a mutation input unless the
user explicitly edits and reparses it.

### Angle-format service

V1 angle formatting consumes the owning field's canonical integer angle and
does not establish a new canonical angle representation. The adapter names its
source resolution (for example tenths of a degree or millidegrees), converts it
to an exact rational degree value, and requests one of these descriptor tokens:

| Notation | Allowed precision tokens | Output |
|---|---|---|
| `decimal_degrees` | `1`, `0.1`, `0.01`, `0.001` | signed decimal plus `deg` |
| `dms` | `1s`, `0.1s` | signed degrees, minutes, seconds |
| `radians` | `0.001`, `0.000001` | signed decimal plus `rad` |

The descriptor schema stores `precision` as one of those strings; its factory
value is `{"notation":"decimal_degrees","precision":"0.1"}`. Validation
rejects a precision not valid for the selected notation. Decimal-degree and DMS
formatting use checked rational arithmetic. Radians are necessarily a rounded
presentation of most exact degree values, must report `rounded=true`, and may
not be parsed back or written to geometry as exact truth. UNIT-I03 does not add
a general angle-expression parser: existing canonical integer angle fields and
their compatibility names remain authoritative until a separately governed
input and rounding contract exists. This removes the catalog's unsupported
`angle parser` consumer claim.

### Surface and adapter boundaries

The approved Global Preferences Units surface must use the GP-F05 native owned
window, immediate-save repository, resolver, typed presentation catalog,
search, focus, reset, provenance, accessibility, responsive, and non-color
patterns. It adds one **Units** category and exactly the six controls above; it
does not expose document-unit or Project-policy editing in Global Preferences.
Each control states `Global · this device`, while seed-capable provenance also
states that the effective value is copied only at New Project with a receipt.
Changing a Global Units value immediately changes that user's interactive
display and contextual bare-number defaults. It never mutates an open or
existing Project.

GUI numeric fields pass a typed `QuantityContext` and the current immutable
`ResolvedUnitsProfile` snapshot to the service. Explicit suffixes are independent
of that snapshot. A bare value is refused when the field has no declared
quantity/profile mapping. Search terms include labels, stable keys, supported
unit names, and suffixes without making the search index a semantic authority.

CLI and MCP compatibility follows one additive rule:

- existing integer `_nm` request and response fields remain accepted and exact;
- a new expression sibling is optional and mutually exclusive with its `_nm`
  sibling; a request supplying both is refused;
- an explicit suffix needs no profile; a bare expression must carry an explicit
  quantity/unit context in the request, never the machine's Global preference;
- normalized results and refusals expose canonical `_nm` plus typed provenance;
- legacy commands that already use an explicitly named unit may retain their
  spelling as adapters, but their conversion delegates to the shared exact
  service.

Machine Preferences are therefore not an automation input. GUI, CLI, and MCP
share semantics without making headless results depend on who last opened the
desktop window.

Interchange readers and writers also do not read Global or Project display
preferences. A file-format adapter owns only that format's documented grammar,
unit declarations, coordinate scale, and deterministic serialization. Its
numeric conversion must delegate to the shared checked rational primitives,
but import/export output is a function of file bytes, format version, explicit
operation options, and canonical model state—not a local profile. Display
precision never truncates or rounds an imported or exported canonical value.

### Legacy conversion disposition

UNIT-I03 must inventory every length conversion call site before changing one
and classify it under this table. Unclassified conversions block acceptance.

| Existing path | Required disposition |
|---|---|
| GUI numeric entry/readout | use quantity context plus resolved profile |
| CLI fixed-unit/floating input | preserve syntax through a compatibility adapter; replace conversion with checked parsing and typed refusal |
| MCP `_nm` fields | preserve as canonical compatibility path; add only mutually exclusive expression siblings where authorized |
| KiCad and other import readers | keep format grammar local; replace floating conversion with checked rational primitives |
| Save/export serializers | keep deterministic format policy local; format canonical integers without Preferences |
| Artifact preview/readout | use the display formatter; no fixed-six-decimal truncation |
| Public `mm_to_nm`/`mil_to_nm`/`inch_to_nm` floating helpers | remove after all production callers migrate, or make an explicitly deprecated compatibility boundary that returns typed refusal rather than saturation/truncation |

No adapter may silently round a non-integral nanometer, saturate overflow, use
locale-sensitive decimal parsing, or create a second suffix table.

### Project seed and receipt ownership

The six descriptors are simultaneously Global live display/parser preferences
and eligible `ProjectPolicySeed` inputs. Those roles do not create live linkage.
At New Project, the Preferences resolver produces one immutable, validated
`UnitsProfile` seed snapshot. An eligible explicit
`datum.projects.unit_policy_seed` aggregate wins; otherwise the six resolved
typed descriptors compose the snapshot. The Project mutation authority—not the
Preferences repository and not the Units service—atomically writes
`ProjectDisplayUnits` and the durable itemized `ProjectSeedReceipt`. Existing
Projects never follow later Global changes.

UNIT-I03 owns the typed Units snapshot schema, aggregate validation/composition,
and the Units side of a real New-Project integration proof. It may call the one
canonical Project mutation transaction; it may not write Project files itself.
The later GP-CM03 step retains ownership of the general seed pipeline and all
non-Units seed descriptors, and must reuse this Units integration rather than
create a second path. A Global preference edit must produce byte-identical open
Project files. Project Preferences remains separately specified and is not
authorized by UNIT-I03.

Authored expression text is request provenance, not persisted design authority.
UNIT-I03 stores canonical integers and the already-ratified seed/receipt facts;
it does not add an authored-unit-intent field to geometry. The comparative
`units-and-grid-model.html` statement that every number remembers how it was
written is not ratified for this step and requires its own schema/ownership
decision before implementation.

### Clause-by-clause acceptance matrix

Every row is mandatory. The seven original proof families are identified as
P1–P7 in their listed order above.

| Requirement | Required cross-surface proof |
|---|---|
| 1. One service | call-site inventory shows GUI, CLI, MCP, preview, and interchange adapters delegate to the engine service; no rival suffix/conversion semantics remain |
| 2. Exact nm truth | before/after Project-byte comparison for every system, unit, precision, and angle-format preference change; P4 |
| 3. Measurement system | typed profile tests cover Metric and Imperial plus every follow-system cell |
| 4. Typed quantities/overrides | compile/runtime refusal of invalid quantity-unit pairs; every explicit cross-system choice remains visible in GUI and query provenance |
| 5. Display precision | all five descriptor tokens map exactly as specified; rounded flag and no-write-back proof; exact round trips where representable; P3 |
| 6. Length suffixes | common vectors including P1 pass identically through direct engine, GUI, CLI, and MCP; P2 and P5 |
| 7. Checked arithmetic | signed limits, overflow, malformed input, sub-nm, case/whitespace, Unicode `µm` and ASCII `um` refusals/results are identical; P2 |
| 8. Explicit context | GUI bare values name field/quantity/unit; CLI/MCP bare values without request context refuse; P6 |
| 9. Provenance | success and refusal snapshots assert every required provenance field, rounded state, and cross-system state across GUI/CLI/MCP |
| 10. `_nm` compatibility | old MCP/CLI vectors remain byte-for-byte compatible; dual-field requests refuse; expression results include canonical `_nm`; P7 |
| 11. Quantity separation | length suffixes refuse for angle/ratio/percentage/frequency; radians remain display-only and cannot write geometry |
| 12. Descriptor classification | registry/catalog/profile schema agree on six keys, types, defaults, presentation metadata, consumers, live role, and copy-once seed role |
| P1. Equivalence | `5.08mm`, `200mil`, `0.2in`, and `5080000nm` resolve to the same canonical value in every adapter |
| P2. Negative/boundary corpus | one shared conformance corpus runs against engine and all edge adapters |
| P3. Round trip | exact representations round trip; rounded length/angle output is marked and never used as canonical write-back |
| P4. No mutation | Global edits and readout-format changes leave existing Project bytes, journal, and dirty state unchanged |
| P5. Suffix parity | GUI, CLI, and MCP return the same canonical value and provenance for every explicit-suffix vector |
| P6. Bare context | accepted GUI and explicit CLI/MCP contexts name the same resolved quantity/unit; absent automation context refuses |
| P7. Migration | all established `_nm` callers and deterministic interchange fixtures remain compatible; legacy conversion inventory has no unreviewed path |

Production acceptance additionally requires the owner-approved protected Units
target, running-app captures for ordinary/changed/search/refusal/cross-system/
narrow/non-color states, keyboard and screen-reader evidence, real New-Project
seed/receipt proof, a zero-existing-Project-mutation proof, and the full governed
test suite. No single screenshot or isolated module test can satisfy this matrix.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:SHARED-UNITS-ENGINE -->
