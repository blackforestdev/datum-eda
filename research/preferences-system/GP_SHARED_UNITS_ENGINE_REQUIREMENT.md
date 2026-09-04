# Shared Units Engine Requirement

> **Status:** Controlling owner-directed requirement; UNIT-I02R was reopened by
> the owner on 2026-09-03 for six industrial-readiness corrections. The revised
> contract and its protected visual targets require fresh UNIT-I02V review.
> Runtime implementation, production acceptance, and dependencies are not
> authorized.
>
> **Trackers:** `dat-global-preferences-engine-qcv` (ratified Preferences
> authority) and `dat-shared-units-surface-parity-s9x` (UNIT-I02R/UNIT-I03A–C).

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
5. Every independently selectable length quantity owns an independent typed
   display-precision choice. Precision controls formatting only and includes
   automatic quantity/unit defaults and an exact-nanometer mode. Rounded display
   text cannot be written back as if it were the exact stored value.
6. The shared parser accepts locale-independent decimal and scientific-notation
   length expressions with explicit supported suffixes, initially including
   `nm`, `µm`/`um`, `mm`, `mil`, and `in`. Inputs such as `5mm`, `2e2mil`, and
   `0.1in` use identical semantics in GUI, CLI, and MCP.
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
    V1 exposes only decimal-degree angle display and exact decimal-degree input;
    DMS and radians remain deferred until they have a complete governed input,
    edit, rounding, and canonical-scale contract.
12. The final Preferences descriptor catalog must separately classify the
    measurement-system default, each quantity display override, and each
    precision policy while pointing them to this one engine service.

## Required later proof

Implementation planning must include:

- exact equivalence such as `5.08mm == 200mil == 0.2in == 5_080_000nm`;
- negative values, signed limits, bounded scientific exponents, overflow at
  every rational stage, whitespace/case policy, Unicode `µm` and ASCII `um`,
  malformed input, and sub-nanometer refusal;
- format/parse round trips wherever the selected representation is exact;
- focus/edit/no-op/cancel proof that rounded view text never becomes input;
- proof that display precision and system changes leave stored bytes unchanged;
- identical explicit-suffix results through GUI, CLI, and MCP adapters;
- contextual bare-number proof naming the resolved field/unit;
- migration compatibility for existing integer `_nm` callers, retired Units
  descriptor values, and Projects predating `ProjectDisplayUnits`.

The intentionally different examples `5mm`, `200mil`, and `0.1in` demonstrate
suffix acceptance; they are not asserted to be equivalent values.

## Industrial Units execution contract

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R -->

This section translates the twelve owner requirements and seven proof families
above into the exact boundary that must be reviewed before UNIT-I03A may start.
It does not replace or weaken those requirements. Planning evidence, an HTML
render, or a passing isolated core test is not implementation authorization or
production acceptance.

### Ratified authority model

Product Mechanics 040 makes the authority split exact. `UnitsProfile` is one
shared value schema, not one shared owner:

| Layer | Owner | Effect |
|---|---|---|
| Global Units defaults | Global Preferences repository/resolver | defaults for future Projects only; no open or existing Project follows |
| Project Working Units | Project `ProjectDisplayUnits` authority | editor display, measurement readouts, and contextual bare numeric input for that Project |
| Publish/document units | `AdoptedDraftingStandard` and Publish/document authority | dimension and issued-document presentation; independent of working units |

The Project-facing product name is **Project Working Units**; the persisted
schema identity remains `ProjectDisplayUnits`. A Project Working Units change
is a canonical journaled Project mutation. It changes Project settings and the
journal, but never rescales or rewrites canonical geometry. Global edits leave
all existing Project bytes, journals, dirty state, and views unchanged.

This mapping follows Onshape's modern account-default → workspace-owned →
drawing-owned separation. SOLIDWORKS, Inventor, and Fusion corroborate the
document/template ownership boundary. KiCad is low-weight comparison evidence
only and cannot establish this architecture. Product Mechanics 040 records the
reviewed primary sources and owner decision.

### Typed profile and resolution

The preference-facing value is one typed `UnitsProfile`, composed from exactly
eight registered `datum.units.*` descriptors. Unit and precision ownership are
paired per quantity; one shared length precision is forbidden because the three
quantities permit different units and require different useful defaults.

```text
UnitsProfile {
  system: MeasurementSystem,
  board_length: LengthUnitChoice<BoardLayout>,
  board_length_precision: LengthPrecisionChoice,
  drill_hole: LengthUnitChoice<DrillHole>,
  drill_hole_precision: LengthPrecisionChoice,
  schematic_geometry: LengthUnitChoice<SchematicGeometry>,
  schematic_geometry_precision: LengthPrecisionChoice,
  angle_precision: DecimalDegreePrecision,
}

LengthUnitChoice<Q> = FollowSystem | Explicit(Q::AllowedUnit)
LengthPrecisionChoice = Automatic | DecimalPlaces(0..6) | ExactNanometer
DecimalDegreePrecision = DecimalPlaces(0 | 1 | 2 | 3)
```

The generic notation above is a type relationship, not a requirement for Rust
generics. Invalid quantity/unit pairs must be unrepresentable after descriptor
validation. The service returns a `ResolvedUnitsProfile` containing concrete
units, resolved per-quantity display precisions, decimal-degree precision, and
an `OverrideState` for each quantity. It must not consult ambient preferences
internally; callers pass the eight already-resolved descriptor values as one
immutable-at-read snapshot.

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

Each of the three precision descriptors has the same structural token inventory:

| Stored token | Engine value | Meaning |
|---|---|---|
| `automatic` | quantity/unit default below | recompute when the resolved unit changes |
| `decimal_0` … `decimal_6` | `DecimalPlaces(0..6)` | explicit places in the resolved display unit |
| `exact_nm` | `ExactNanometer` | signed canonical integer plus `nm` |

`Automatic` is resolved by the complete V1 table; the stored token stays
`automatic` and the resolved precision is returned in provenance:

| Quantity | `mm` | `um` | `mil` | `inch` |
|---|---:|---:|---:|---:|
| Board/layout | 3 places (`0.001 mm`) | 1 place (`0.1 um`) | 1 place (`0.1 mil`) | 5 places (`0.00001 in`) |
| Drill/hole | 3 places (`0.001 mm`) | — | 1 place (`0.1 mil`) | 4 places (`0.0001 in`) |
| Schematic geometry | 2 places (`0.01 mm`) | — | 0 places (`1 mil`) | — |

An explicit precision never changes when the measurement system or unit changes;
only `Automatic` re-resolves. Decimal display rounds half away from zero and
returns `rounded=true` whenever information is hidden.

The retired `datum.units.length_precision` value is migration input only. A
valid legacy token maps once and atomically into all three new precision keys:
`0.1`→`decimal_1`, `0.01`→`decimal_2`, `0.001`→`decimal_3`,
`0.0001`→`decimal_4`, and `exact_nm`→`exact_nm`. Explicit contributions already
present on any new key win for that key; an invalid legacy value is preserved
as unreadable evidence and cannot partially seed the new profile. The retired
key is never written again and is not a ninth active descriptor.

### Lossless display and edit contract

Formatted view text is never an edit buffer and never becomes geometry merely
because focus enters or leaves a control. A numeric field must obey all of the
following:

1. Unfocused display may use the selected precision and may be rounded.
2. On edit focus, the buffer is regenerated from the exact canonical value, not
   from the rounded label. If the value has a finite exact representation in the
   resolved unit, Datum shows it with its suffix; otherwise it shows the exact
   signed nanometer value with `nm`.
3. Focus then blur, Enter, or navigation without a semantic edit is a no-op: no
   journal entry, dirty-state change, normalization write, or geometry rewrite.
4. Escape restores the pre-edit canonical value and leaves geometry unchanged.
5. An edited commit parses once through the shared service, compares canonical
   values, and journals only an actual canonical change. Caret increments and
   steppers operate on the canonical value using a declared exact step, never by
   reparsing the rounded display string.

Thus additional exact digits can appear on focus without implying that display
precision controls accuracy.

### V1 angle service

V1 deliberately exposes one honest angle model: decimal degrees. The active
descriptor `datum.units.angle_precision` stores `decimal_0`, `decimal_1`,
`decimal_2`, or `decimal_3`, with factory `decimal_1`. Formatting consumes the
owning field's declared canonical integer angle scale, converts through checked
rational arithmetic, appends `deg`, and reports whether display rounding hid
information.

Angle input accepts signed locale-independent decimal or scientific notation
with explicit `deg` or `°`; a bare number is accepted only in a declared angle
field context. Conversion to the field's declared canonical integer scale must
be exact. A non-integral result, overflow, wrong suffix, absent scale, or absent
bare-number context is a typed refusal; V1 never silently rounds angle input.

DMS and radians are useful future presentation modes but are not active V1
preferences. The retired draft `datum.units.angle_format` value may migrate only
when its notation is `decimal_degrees` and its precision maps exactly to the new
descriptor. A stored `dms` or `radians` draft is preserved as unsupported legacy
evidence and the factory decimal-degree precision is used for the session; it is
not silently reinterpreted. Activating DMS or radians requires a later governed
contract covering entry grammar, exact canonical conversion, editing, rounding,
clipboard/automation behavior, and migration. No V1 surface may advertise them.

### Exact expression grammar and future expression boundary

The V1 numeric grammar is
`[+-]?(digits(\.digits?)?|\.digits)([eE][+-]?digits)?`, followed by optional
ASCII whitespace and an allowed suffix. Internal whitespace, grouping marks,
locale commas, hexadecimal forms, `NaN`, and infinities are refused. Exponents
are parsed as bounded powers of ten through checked `i128` rational arithmetic;
exponent construction, numerator/denominator reduction, unit scaling, and final
integer conversion each check overflow. A mathematically non-integral canonical
length or angle result is refused, not rounded.

This grammar is a scalar-with-unit seam, not a general formula language. A later
expression engine may own arithmetic, references, functions, and dimensional
analysis, but it must consume this service's quantity types, unit-token registry,
and checked rational conversion. It may return an exact typed scalar to this
service; it may not introduce a second suffix table or rival conversion policy.

### Surface and adapter boundaries

The owner-review Global Preferences Units surface must use the GP-F05 native owned
window, immediate-save repository, resolver, typed presentation catalog,
search, focus, reset, provenance, accessibility, responsive, and non-color
patterns. It adds one **Units** category and exactly eight registered descriptor
rows and value selectors: system; unit and precision for Board, Drill, and
Schematic; and decimal-degree precision. A quantity's Unit and Precision may be
visually paired, but remain separately named, focused, stored, and resettable.
It does not expose document-unit or Project-policy editing in Global Preferences.
Each descriptor row states `Global · this device`, while seed-capable provenance
also states that the effective value is a default copied only at New Project
with a receipt. The category and each descriptor row must say **Defaults for
new Projects** and **Open Projects are unaffected**. Changing a Global Units value changes the
future seed only. It never changes interactive display, contextual bare-number
defaults, or any other state in an open or existing Project.

Production acceptance also requires a real Project Preferences **Units** slice.
Its terminal menu command is `Edit > Preferences > Project Preferences…`; it is
disabled without an open Project and opens the window directly. **Units** is the
selected category inside the window, never a third menu/submenu level. The native
window is owned/input-modal to Datum, and no setting appears in the Navigator.
The available and disabled doorway states are separate examples; the same menu
must never show duplicate enabled and disabled Project Preferences commands.
The visible scope is `Project · <project name>`. The slice contains the same eight typed values under
Project Working Units authority, not Global provenance, plus the durable seed
receipt as read-only provenance. Immediate control commits use the canonical
journaled Project mutation, are undoable, and never touch geometry. The initial
window contains only the real Units category—no empty, planned, Revision,
Publish, or document-unit category.

Projects created before `ProjectDisplayUnits` exists migrate deterministically
from the versioned Datum factory profile embedded in the migration, never from
the current machine's Global values. Migration atomically writes the Project
profile and a migration receipt through Project authority, leaves geometry
byte-identical, and is idempotent. An unreadable or invalid legacy profile is
preserved and refused rather than partially guessed.

A Project row's Reset action means **Reset to this Project's recorded seed or
migration value**. It reads the durable receipt, submits that value through the
same journaled mutation, and is undoable. It never re-resolves the current Global
default and never changes the receipt. Reset is unavailable with a visible typed
reason when the applicable receipt/value cannot be read; it never guesses or
falls back to a machine preference.

GUI numeric fields in a Project pass a typed `QuantityContext` and that
Project's immutable-at-read `ResolvedUnitsProfile` snapshot to the service.
Explicit suffixes are independent of that snapshot. A bare value is refused
when the field has no declared quantity/Project-profile mapping. A no-Project
preview may explicitly use `GlobalUnitsDefaults`, but it must identify itself as
a preview and cannot become editor or mutation context. Search terms include
labels, stable keys, supported unit names, and suffixes without making the
search index a semantic authority.

CLI and MCP compatibility follows one additive rule:

- existing integer `_nm` request and response fields remain accepted and exact;
- a new expression sibling is optional and mutually exclusive with its `_nm`
  sibling; a request supplying both is refused;
- an explicit suffix needs no profile; a bare expression must carry an explicit
  quantity/unit context or identify a Project and field whose Project Working
  Units can be resolved, never the machine's Global preference;
- normalized results and refusals expose canonical `_nm` plus typed provenance;
- legacy commands that already use an explicitly named unit may retain their
  spelling as adapters, but their conversion delegates to the shared exact
  service.

Machine Preferences are therefore not an automation input. GUI, CLI, and MCP
share semantics without making headless results depend on who last opened the
desktop window.

Interchange readers and writers also do not read Global defaults or Project
Working Units. A file-format adapter owns only that format's documented grammar,
unit declarations, coordinate scale, and deterministic serialization. Its
numeric conversion must delegate to the shared checked rational primitives,
but import/export output is a function of file bytes, format version, explicit
operation options, and canonical model state—not a local profile. Display
precision never truncates or rounds an imported or exported canonical value.

### Known runtime contradictions to remove

The reopened audit found implemented code that cannot be treated as authority or
accepted in place:

- `preferences/catalog_part_one.rs` classifies Units as live-and-seed and names
  live display/input consumers. It must classify all eight active keys as
  future-Project seeds only and remove the unsupported angle-parser claim.
- `preferences/schema.rs` accepts a permissive numeric angle structure. It must
  validate the exact decimal-degree precision enum and retired-key migration
  rules; arbitrary positive floating precision is forbidden.
- `ir/units.rs` and call sites still expose floating `mm_to_nm`, `mil_to_nm`, and
  `inch_to_nm` paths. Production callers must move to checked rational
  conversion; no saturating, truncating, or independently rounded rival path may
  remain.
- the runtime registry, catalog, persisted `UnitsProfile`, Project schema, GUI
  presentation metadata, CLI/MCP schemas, and migration fixtures must carry the
  same eight keys and token inventories. Key-prefix inference and GUI hard-coding
  remain forbidden.

UNIT-I03A must begin with assertions that fail against these contradictions and
finish with an inventory proving no production caller retains them.

### Legacy conversion disposition

UNIT-I03A must inventory every length and angle conversion call site before changing one
and classify it under this table. Unclassified conversions block acceptance.

| Existing path | Required disposition |
|---|---|
| GUI numeric entry/readout | use quantity context plus the owning Project's resolved working profile; no Global fallback |
| CLI fixed-unit/floating input | preserve syntax through a compatibility adapter; replace conversion with checked parsing and typed refusal |
| MCP `_nm` fields | preserve as canonical compatibility path; add only mutually exclusive expression siblings where authorized |
| KiCad and other import readers | keep format grammar local; replace floating conversion with checked rational primitives |
| Save/export serializers | keep deterministic format policy local; format canonical integers without Preferences |
| Artifact preview/readout | use the display formatter; no fixed-six-decimal truncation |
| Public `mm_to_nm`/`mil_to_nm`/`inch_to_nm` floating helpers | remove after all production callers migrate, or make an explicitly deprecated compatibility boundary that returns typed refusal rather than saturation/truncation |

No adapter may silently round a non-integral nanometer, saturate overflow, use
locale-sensitive decimal parsing, or create a second suffix table.

### Project seed, working-unit, and receipt ownership

The eight descriptors are Global defaults and eligible `ProjectPolicySeed`
inputs; they are not live display/parser preferences for existing Projects.
At New Project, the Preferences resolver produces one immutable, validated
`UnitsProfile` seed snapshot. An eligible explicit
`datum.projects.unit_policy_seed` aggregate wins; otherwise the eight resolved
typed descriptors compose the snapshot. The Project mutation authority—not the
Preferences repository and not the Units service—atomically writes
`ProjectDisplayUnits` and the durable itemized `ProjectSeedReceipt`. Existing
Projects never follow later Global changes.

After creation, `ProjectDisplayUnits` is the sole Project Working Units
authority. Its typed Project Preferences mutation changes the settings shard
and journal with provenance, diff, undo, and refusal semantics. It can change
subsequent display and bare-input interpretation, but canonical geometry shards
must remain byte-identical. Publish/document units remain separately owned by
`AdoptedDraftingStandard` and Publish/document authority; neither direction
silently follows the other.

UNIT-I03A owns the typed Units snapshot schema, aggregate validation/composition,
and the Units side of a real New-Project integration proof. It may call the one
canonical Project mutation transaction; it may not write Project files itself.
The later GP-CM03 step retains ownership of the general seed pipeline and all
non-Units seed descriptors, and must reuse this Units integration rather than
create a second path. A Global preference edit must produce byte-identical open
Project files. UNIT-I03B owns the Project Preferences Units-only slice and its
mutation UX under this contract and the corresponding Units-specific completion
of PPS-C01 through PPS-C03. Broader Project Preferences categories remain
separately specified and unauthorized.

Authored expression text is request provenance, not persisted design authority.
UNIT-I03A stores canonical integers and the already-ratified seed/receipt facts;
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
| 2. Exact nm truth | Global changes leave every existing Project byte-identical; Project Working Units changes alter only settings/journal authority while geometry shards remain byte-identical; P4 |
| 3. Measurement system | typed profile tests cover Metric and Imperial plus every follow-system cell |
| 4. Typed quantities/overrides | compile/runtime refusal of invalid quantity-unit pairs; every explicit cross-system choice remains visible in GUI and query provenance |
| 5. Display precision | each quantity owns `automatic`, `decimal_0..6`, or `exact_nm`; every automatic table cell resolves exactly; explicit precision survives unit/system changes; focus/edit/no-op/cancel/commit never feeds rounded text back to canonical truth; decimal-degree precision maps exactly; P3 |
| 6. Length suffixes | decimal and scientific-notation vectors, including P1, pass identically through direct engine, GUI, CLI, and MCP; P2 and P5 |
| 7. Checked arithmetic | signed limits, bounded exponents, overflow at every rational stage, malformed input, sub-nm, case/whitespace, Unicode `µm` and ASCII `um` refusals/results are identical; P2 |
| 8. Explicit context | GUI bare values name Project/field/quantity/unit; CLI/MCP bare values either name equivalent explicit context or a Project+field, otherwise refuse; no path consults Global defaults for an existing Project; P6 |
| 9. Provenance | success and refusal snapshots assert every required provenance field, rounded state, and cross-system state across GUI/CLI/MCP |
| 10. `_nm` compatibility | old MCP/CLI vectors remain byte-for-byte compatible; dual-field requests refuse; expression results include canonical `_nm`; P7 |
| 11. Quantity separation | length suffixes refuse for angle/ratio/percentage/frequency; decimal-degree angle input requires angle context and exact canonical-scale conversion; DMS/radians remain absent and deferred |
| 12. Descriptor classification | registry/catalog/profile schema agree on eight Global-default keys, types, defaults, presentation metadata, eight rows/selectors, legacy fan-out/refusal rules, copy-once seed role, Project Working Units owner, and separate Publish/document owner |
| P1. Equivalence | `5.08mm`, `200mil`, `0.2in`, and `5080000nm` resolve to the same canonical value in every adapter |
| P2. Negative/boundary corpus | one shared conformance corpus runs against engine and all edge adapters |
| P3. Round trip | exact representations round trip; rounded length/angle output is marked; focus reveals exact canonical text; focus/blur and unchanged commit are byte-identical no-ops |
| P4. Authority isolation | Global edits leave existing Project bytes, journal, dirty state, and views unchanged; Project Working Units edits journal exactly one settings mutation and leave geometry bytes unchanged |
| P5. Suffix parity | GUI, CLI, and MCP return the same canonical value and provenance for every explicit-suffix vector |
| P6. Bare context | accepted GUI and CLI/MCP contexts name the same Project/field/resolved quantity/unit; explicit request context remains supported; absent context refuses |
| P7. Migration | established `_nm` callers and deterministic interchange fixtures remain compatible; old shared precision fans out atomically; old decimal-degree angle values map; unsupported angle drafts preserve evidence and refuse reinterpretation; pre-feature Projects receive the versioned factory profile, never current Global values; legacy conversion inventory has no unreviewed path |

Production acceptance additionally requires the owner-approved protected Units
Global and Project targets, running-app captures for ordinary/changed/search/
refusal/cross-system/narrow/non-color states, keyboard and screen-reader
evidence, real New-Project seed/receipt proof, legacy-Project migration proof,
lossless edit proof, a zero-geometry-mutation proof, and the full governed test
suite. No single screenshot or isolated module test can satisfy this matrix.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R-CONTRACT-RECONCILED -->

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R-AUTHORITY-MODEL-APPROVED -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:SHARED-UNITS-ENGINE -->
