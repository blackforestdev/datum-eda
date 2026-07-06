# Parametric DXF-Native Footprint Editor — Feasibility Research

> Driver: project-owner idea raised mid-conversation 2026-04-16 —
> "an EDA package that used footprints that were actually DXF compliant,
> allowing DFM audits outside the EDA seat."
> Parked in research backlog with trigger "After Phase 2 Domains 1-3
> land." Domains 1, 2, 3 are now delivered/triaged; this research is
> unblocked.
> Cross-references: Domain 1
> (`research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
> §"DXF / DWG"), Domain 2
> (`research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
> §"Pool / library integration"), Domain 3
> (`research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md`
> §"Symbol-style profiles"), and IPC research
> (`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §"PCB Libraries
> Inc. — LP Wizard / LP Calculator / Footprint Expert").
>
> Style and depth follow the prior Phase 2 reports but at shorter
> length — this is a feasibility study, not an exhaustive standards
> survey. Where authoritative information is unavailable (notably
> AutoCAD parametric-DXF round-trip behaviour and PlaneGCS
> sub-component licensing), the report says so explicitly rather than
> inventing a conclusion.

> **Update 2026-04-18 — see sibling addendum**
> `PARAMETRIC_FOOTPRINT_EDITOR_DXF_STOWAWAY_ADDENDUM.md`. The
> "defer the DXF-canonical claim indefinitely" verdict below has
> been refined under the project owner's stowaway-encoding reframe:
> ship parametric metadata into DXF anyway as standards-compliant
> XDATA + XRecord stowaway, render flat for everyone, round-trip
> losslessly Datum-to-Datum, treat external consumer adoption as a
> long-tail option rather than a launch dependency.

## Executive Summary

- **Verdict: defer with conditional pursue path.** The "parametric
  footprint" half of the idea is sound, achievable, and a real Datum
  differentiator against KiCad/Horizon at modest engineering cost.
  The "DXF as canonical interchange so DFM auditors outside the seat
  can read footprints" half is **not credible in 2026 and will not be
  credible inside Datum's planning horizon**. The two halves should
  be separated; the parametric editor is worth pursuing on its own
  merits, the DXF-native-canonical claim is not.
- **The make-or-break question — "published-and-adopted profile = standard;
  profile that only Datum reads = vendor lock-in dressed up as openness"
  — answers in the negative.** No DFM auditor (Valor MSS, CAM350, Ucamco
  UCAMX, Frontline InCAM, Genesys-2000, JLCPCB/PCBWay in-house tooling)
  consumes generic DXF for footprint-level audit; they consume Gerber,
  ODB++, and IPC-2581 because those formats already encode pad
  semantics. A new DXF profile would need adoption by at least one of
  those auditors before it had any external value, and there is no
  industry appetite for that — IPC-2581 already solves the audit-
  outside-the-seat problem with stronger semantics than DXF can carry.
- **DXF cannot losslessly carry the PCB-semantics surface Datum's IR
  already encodes.** DXF has no native concept of pad / drill / mask
  aperture / paste reduction / thermal relief / per-layer ownership.
  Encoding those semantics requires a Datum-controlled convention on
  top of stock DXF (named layers, blocks, ATTRIB, XDATA). Such a
  convention is technically writeable, but every consumer of the file
  would need a Datum-specific reader to interpret it — exactly the
  vendor-lock-in failure mode the user flagged.
- **AutoCAD's own parametric-DXF round-trip is the unspoken killer.**
  AutoCAD 2010+ supports geometric and dimensional 2D constraints
  (AcDb2dConstraints, dimensional parameters in dynamic blocks). The
  best public information indicates these constraints **are
  preserved on DXF round-trip only when the consuming tool is also
  AutoCAD or an ODA-based tool**; non-AutoCAD readers (including the
  Rust `dxf` crate and KiCad's dxflib-derived parser) treat
  constraint records as opaque and either drop them or pass them
  through unchanged but uninterpreted. That makes "parametric DXF as
  the canonical artifact" fail at the first hop into any non-AutoCAD
  consumer. Authoritative details are paywalled or absent from public
  Autodesk documentation; the report flags this as an unresolved
  blocker requiring direct AutoCAD verification.
- **The prior-art landscape is unanimous: parametric source is
  authored, footprint output is exported flat.** Every surveyed tool
  (Altium IPC Footprint Wizard, OrCAD Symbol Generator, PADS LP
  Wizard, KiCad Python footprint wizards, kicad-footprint-generator,
  Horizon EDA's parameter-program packages, PCB Libraries Footprint
  Expert) authors parametrically and *bakes* the output into a
  fixed-geometry library entry. Only Horizon preserves the parameter
  program inside the package object as a re-runnable artifact, and
  it runs only inside the Horizon engine. **Nobody ships parametric
  source as an interchange-readable canonical format.** This is
  evidence not just that the idea is novel but that the industry
  has independently concluded the value isn't there.
- **The most promising fallback is "parametric editor with strong
  IPC-2581 export and DXF mechanical-layer export," not "parametric
  DXF as canonical."** Datum can ship a SolidWorks-style parametric
  footprint authoring experience — IPC-7351-aware, constraint-driven,
  re-runnable on parameter change, family-aware — and emit
  *baked* footprints into the existing pool. For external DFM
  auditing, IPC-2581 (already targeted by Domain 1 research) carries
  pad semantics losslessly and is read by every professional DFM
  tool. For mechanical-layer DXF export (the existing Domain 1
  recommendation), `dxf-rs` covers the workflow.
- **The 2D constraint solver question has a clean answer for the
  no-copyleft rule: build from scratch in Rust under MIT/Apache 2.0,
  starting from published reference algorithms.** PlaneGCS (FreeCAD)
  is LGPL-2.1+ — the "static link forbids relicensing" rule
  effectively excludes it for a Rust crate that wants to stay
  permissive. Solvespace's solver is GPL-3 — hard exclude per the
  no-copyleft rule. D-Cubed DCM is paid commercial (~$50k/seat).
  No mature permissive-licensed Rust 2D geometric constraint solver
  exists on crates.io as of mid-2026. **A minimum viable solver for
  the parametric-footprint domain is roughly 4-6 weeks of focused
  work** — the constraint vocabulary needed (point-on-line, distance,
  parallel, perpendicular, equal-length, symmetric-about-axis, fixed,
  coincident) is small (~12 constraint types), and Newton-Raphson
  over a residual function with sparse-Jacobian solving (using
  `nalgebra-sparse`) is well-documented in the FreeCAD literature
  and academic papers cited under Sources.
- **Phasing recommendation if pursued: ship parametric-internal first,
  defer the DXF-canonical claim.** Phase A (1-2 quarters) — build
  the parametric editor with constraint solver, integrate with the
  existing pool/Package model via a new `ParametricPackage` type
  whose evaluator emits standard `Package` instances. Phase B
  (defer indefinitely) — write the DXF profile as an *export* format
  for mechanical interoperability, do not market it as canonical, and
  drop the published-and-adopted-DXF-profile claim entirely unless
  IPC or OpenEDA picks up the work independently. This separates
  the achievable value (parametric editor as Datum differentiator)
  from the unachievable value (DXF-as-DFM-interchange).

## Central Question

The single question this research answers:

> **Is parametric-footprint-with-DXF-canonical-form viable as a Datum
> product feature?**

Decomposed into two sub-questions because they have different answers:

1. **Is parametric-footprint-as-authored-form viable?** Yes — the
   prior art shows it works, the market validates the value, the
   constraint-solver build cost is manageable, and the integration
   with Datum's pool model is clean. (Areas 1, 3, 4.)
2. **Is DXF-as-canonical-form-readable-outside-Datum viable?** No —
   DXF cannot carry the necessary PCB semantics natively, a
   Datum-controlled DXF profile would still require a Datum-specific
   reader, AutoCAD's own parametric-DXF round-trip is unreliable
   into non-AutoCAD tools, and no DFM auditor would adopt the
   profile when IPC-2581 already exists and is actively gaining
   adoption. (Areas 2, 5.)

The answer to the central question is therefore: **the idea splits
into a viable half (parametric editor, ship internally) and an
unviable half (DXF as canonical interchange, drop or defer to a
mechanical-layer-only role).**

## Six Focused Areas

### Area 1 — Prior art: parametric footprint/symbol systems

#### PCB Libraries Inc. — LP Calculator / LP Wizard / Footprint Expert

The reference IPC-7351 implementation. Cross-referenced in
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` lines 1025-1054.
The parametric model is form-driven per package family (chip,
SOIC, QFP, BGA, etc.); the user fills lead dimensions (Lmin/Lmax,
Tmin/Tmax, Wmin/Wmax), tolerances (fab, placement), and density
level (A/B/C). The tool computes pad geometry from the IPC formulas
and exports to ECAD-tool-native formats (Altium .PcbLib, KiCad
.kicad_mod, OrCAD .dra, PADS .pt, etc.).

**Parametric source preservation:** None outside the tool. The output
is fixed geometry in the target ECAD format. The *parameters* used to
generate the footprint (Lmin/Lmax, density level, etc.) are not
written into the footprint file. PCB Libraries' own internal database
keeps the parameters; the consuming ECAD tool sees only the geometry.

**Output formats:** Native ECAD library formats only. No DXF export.
No interchange format that carries the parametric source.

**Net assessment:** Validates the *parametric* half of the idea — a
form-driven, IPC-aware parametric authoring tool is the gold
standard. **Disconfirms** the DXF-as-interchange half — even the
gold-standard tool exports baked geometry, not parametric source.

#### IPC-7351 itself

The standard prescribes the formulas (toe / heel / side fillet
allowances, density-level adjustments, tolerance accumulation) but
deliberately does **not** prescribe an interchange format for the
parametric *input*. IPC-7351B and the unreleased IPC-7351C stop at
"this is how you compute pad geometry given component dimensions."
There is no IPC-blessed file format for "this is the SOIC family
parametric definition" — the formulas are normative, the encoding
is implementation-defined.

IPC-2581 (the modern DPMX exchange format) carries baked geometry
plus rich metadata (controlled-impedance-by-net, differential pairs,
embedded components) but does **not** carry parametric source.
IPC-2581's `PadStackDef` element is a fixed-geometry padstack; there
is no parametric padstack element in the schema.

**Net assessment:** The standards body that owns IPC-7351 has not
specified an interchange-format home for the parametric input despite
having had 20 years to do so. This is a strong signal that the
industry value of "parametric source as exchange artifact" has been
weighed and judged low.

#### Altium Footprint Wizard / IPC Compliant Footprint Wizard

Altium's IPC Compliant Footprint Wizard (introduced AD10, refined
through AD24) is the closest commercial peer to PCB Libraries
Footprint Expert. Per-family parametric forms (Chip, SOIC, SOP,
QFP, QFN, BGA, etc.) with full IPC-7351 parameter exposure.

**Parametric source preservation:** The wizard writes a fixed-geometry
footprint into the active library (`.PcbLib`). The wizard parameters
themselves are not preserved in the library — they live in a
session dialog and are discarded on close. To regenerate at a
different density level, the user re-runs the wizard from scratch.

There is a separate Altium feature ("Component Templates") that
preserves a *template-instance* relationship for symbols, but it is
not used for footprint parametrics in the IPC Wizard flow.

**Output formats:** Altium PCB library binary (`.PcbLib`). DXF
export of the footprint is possible via the mechanical-layer export
path but is not parametric.

**Net assessment:** Same conclusion as PCB Libraries. The biggest
commercial PCB tool authors parametrically and bakes the output.

#### OrCAD Symbol Generator / OrCAD Footprint Editor

OrCAD's "Footprint Wizard" (Allegro Library Manager) supports the
same family of IPC parametric flows. Symbol Generator handles
schematic symbol creation parametrically (rectangular bodies, pin
arrays, pin-name auto-numbering).

**Parametric source preservation:** None at the library level. The
generator output is a `.dra` (footprint) or `.olb` (symbol) file
with no parametric back-reference.

**Net assessment:** Same as Altium.

#### PADS LP Wizard / Land Pattern Creator

Originally PCB Libraries' LP Wizard, rebranded as PADS Land
Pattern Creator after Mentor licensed the technology. The
parametric model is identical to PCB Libraries' Footprint Expert
(per IPC research, line 815-820). Output is a baked PADS `.pt` /
`.cae` library file.

**Net assessment:** Same as the others; output is baked.

#### KiCad in-app footprint wizards (Python)

KiCad ships a built-in Python wizard family (BGA, SOIC, S-DIP, QFP
in `kicad-footprint-wizards`). Each wizard is a Python class with a
parameter set and a `BuildThisFootprint()` method that emits
KiCad's internal footprint data structure, then serialises to
`.kicad_mod` (s-expression).

**Parametric source preservation:** **None.** The wizard runs in
the KiCad PCB editor process; on save, only the resulting
`.kicad_mod` is written. The Python wizard code is a tool, not a
library citizen. Long-running stability issues — multiple wizards
have thrown unhandled Python exceptions for several KiCad releases
(GitLab issue #4896 is the canonical bug). Cross-referenced in
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` line 869-875.

**Net assessment:** KiCad has parametric *authoring* but not
parametric *libraries*. This is the same shape as Altium / PCB
Libraries / OrCAD / PADS.

#### kicad-footprint-generator (community Python project)

The Python script library used to bulk-generate KiCad's official
library footprints (per IPC research, line 877-883). YAML-driven —
each package family has a YAML file (`scripts/Packages/ipc_definitions.yaml`)
holding lead dimensions, density levels, tolerances; a Python
generator reads the YAML and emits `.kicad_mod` files in batch.

**Parametric source preservation:** The YAML *is* the parametric
source, but it is not part of the footprint library — it is a
build-time input that lives in a separate repo. End-users of the
official KiCad library see only the generated `.kicad_mod` files;
the YAML is a contributor-only artifact.

This is the *closest* surveyed model to "parametric source as a
first-class library citizen" because the YAML is openly published
and any tool can in principle re-run the generator. But the YAML is
KiCad-specific, the generator is KiCad-specific Python, and no
non-KiCad tool reads either.

**Net assessment:** Validates that "parametric source preserved as
a separate file alongside the baked output" is a workable pattern —
this is what Datum should consider for its parametric pool entries
(see Area 4). Does **not** validate "parametric source as universal
interchange" — the source is KiCad-tool-specific even when openly
published.

#### Horizon EDA's parametric package model

The closest open-source structural match for parametric footprints
inside the canonical IR. Studied directly from
`research/horizon-source/src/parameter/` and
`research/horizon-source/src/gen-pkg/`.

Horizon's model has three pieces:

1. **`ParameterSet`** (`src/parameter/set.hpp`) — a small enum of
   typed parameter IDs (`PAD_WIDTH`, `PAD_HEIGHT`, `PAD_DIAMETER`,
   `SOLDER_MASK_EXPANSION`, `PASTE_MASK_CONTRACTION`, `HOLE_DIAMETER`,
   `HOLE_LENGTH`, `COURTYARD_EXPANSION`, `VIA_DIAMETER`, etc., 13
   parameters total) keyed to int64 values (Horizon stores all
   geometry in nanometers). The set is finite and hard-coded —
   there is no facility to add new parameter IDs without editing
   the C++ enum.
2. **`ParameterProgram`** (`src/parameter/program.hpp` and
   `program.cpp`) — a stack-based RPN micro-language compiled
   inline by Horizon. Tokens are integers, command words (`+xy`,
   `-xy`, `dup`, `chs`, `get-parameter`, `set-polygon`), strings,
   and UUIDs. Programs run inside the Horizon engine to mutate
   polygons of a Package object (e.g., the courtyard polygon's
   coordinates are computed from `COURTYARD_EXPANSION` plus
   authored pad coordinates).
3. **`Package.parameter_program`** field (visible in
   `src/gen-pkg/gen-pkg.cpp` line 51-59) — the program text is
   stored as a string inside each package JSON. The pad-level
   parameter set is stored on each `Pad` object
   (`pad.parameter_set[ParameterID::HOLE_DIAMETER] = 1_mm` etc.).

**Parametric source preservation:** **Yes** — the program text and
parameter sets are preserved in the package JSON file alongside the
baked geometry. Horizon re-runs the program when the parameter set
changes, regenerating the polygon coordinates.

**Interchange:** None outside Horizon. The `parameter_program` text
is meaningless to any non-Horizon tool (the RPN syntax is
Horizon-specific). The parameter set enum is Horizon-specific.

**Net assessment:** **The right structural shape for Datum** —
parametric source as a first-class field on the Package object,
re-runnable, deterministic. **Disconfirms the DXF-canonical idea**:
Horizon's parameter format is opaque to outsiders by design; if the
Horizon team had thought the parametric source had universal
interchange value, they would have chosen a less Horizon-specific
encoding. Their choice of an internal RPN language is evidence that
they considered the parametric source a tool-internal concern.

#### MCAD parametric 2D sketch reference (SolidWorks / Inventor / FreeCAD)

The user's stated reference. SolidWorks 2D sketches have the
following structural shape:

- **Sketch entities** — points, lines, arcs, circles, splines,
  rectangles, polygons; positioned by user input or constrained
  by relations.
- **Geometric constraints** — coincident, collinear, parallel,
  perpendicular, tangent, equal, symmetric, fixed, midpoint,
  concentric, horizontal, vertical (~15 types in SolidWorks; FreeCAD
  Sketcher has a similar set).
- **Dimensional constraints** — distance, angle, radius, diameter
  with named parameters that can be driven by an equation
  (`pad_pitch = 0.5mm`, `pad_count_x = 4`, `pad_count_y = 4`,
  `body_width = pad_pitch * (pad_count_x + 1)`).
- **Solver** — D-Cubed DCM (SolidWorks, Inventor commercial),
  PlaneGCS (FreeCAD open source), Solvespace's internal solver
  (Solvespace).
- **Pattern operations** — linear pattern, circular pattern, mirror
  — applied to sketch geometry to multiply entities along an axis or
  around a centre. (For PCB footprints this is exactly the
  primitive needed for "rectangular array of N×M pads".)

**File format preservation:** SolidWorks sketches are preserved
inside SolidWorks part files (`.SLDPRT`) — a binary, proprietary,
license-locked format. DXF export of a parametric sketch from
SolidWorks emits **flat geometry** — the constraints and dimensional
parameters are not written. AutoCAD claims to round-trip parametric
constraints in DXF when the consumer is also AutoCAD; the
SolidWorks→DXF→anywhere-else path is universally lossy.

**Net assessment:** Validates the *editor UX* the user wants — a
constraint-and-dimension model with named parameters and pattern
operations. **Disconfirms the DXF-canonical idea once more**: even
SolidWorks, the user's reference tool, does not preserve parametric
source on DXF export.

#### OASIS Open / OpenEDA initiatives

Searched. **No active OASIS or OpenEDA initiative on parametric
component formats exists.** OASIS Open hosts several EDA-related
technical committees historically (e.g., the Open Device Format
TC, EDA Tooling Standards Discussion Group), but none have produced
a parametric-component standard. OpenEDA (the loose grouping around
KiCad / LibrePCB / Horizon / Datum-class open-tool collaboration)
has no published spec on parametric interchange. The IEEE 1685
(IP-XACT) standard for parametric IC IP is the closest analogue but
applies to ASIC/FPGA IP cores, not PCB footprints.

**Net assessment:** No standards-body shoulder for Datum to lean
against. A parametric DXF profile would be a unilateral Datum
contribution.

#### Verdict for Area 1

> **Nobody currently ships parametric-footprint-as-interchange-format.**
> Every tool surveyed authors parametrically and bakes the output.
> Horizon comes closest to "parametric source as first-class library
> citizen" but the source format is tool-internal and was deliberately
> not designed for cross-tool interchange. The market signal — across
> commercial and open-source tools, across 20 years of IPC-7351
> existence — is unanimous: parametric source has internal value
> (regeneration, family-aware editing) but not external interchange
> value.

### Area 2 — DXF as a PCB-semantics carrier

#### Existing DXF-to-PCB conventions

Cross-referenced from
`research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
lines 631-686. The existing DXF-to-PCB use case is **board outline
import** — the mechanical CAD team hands off "this is the board
cutout polygon" as a DXF file. PCB tools handle this by:

1. Reading the DXF, walking entities (LINE, ARC, CIRCLE,
   LWPOLYLINE).
2. Asking the user to pick which DXF layer holds the board outline
   (no semantic mapping — DXF layer names are arbitrary; the user
   selects).
3. Importing the geometry onto the PCB tool's "Edge.Cuts" or
   equivalent layer as raw polygon data with no PCB-specific
   metadata.

**There is no convention for DXF carrying pad / drill / silkscreen
/ mask semantics on import.** DXF is treated as raw outline geometry
on a single user-mapped layer. Every surveyed tool (Altium, KiCad,
OrCAD, PADS, Eagle, Horizon, LibrePCB) implements DXF import this
way.

#### Existing DXF-from-PCB conventions

When EDA tools export DXF (typically for the MCAD team's
mechanical-fit drawing), conventions are **per-tool ad hoc**:

- **Altium DXF Export** writes mechanical layers to numbered DXF
  layers (`Mechanical1`, `Mechanical2`, ...). Per-pad geometry can
  optionally be exported but as flat LWPOLYLINE arrays on a
  user-chosen layer; no semantic mapping.
- **KiCad DXF export** writes one DXF layer per KiCad layer
  (`F.Cu`, `B.Cu`, `F.SilkS`, `Edge.Cuts`, etc.) using the KiCad
  layer name verbatim. Pads emit as filled circles or polygons —
  the *shape* is in the geometry but the *semantic* (pad vs
  silkscreen vs courtyard) is encoded in the layer name only.
- **Cadence Allegro** has IPC-defined mechanical-layer DXF naming
  but is Allegro-specific.
- **OrCAD / PADS** export DXF with vendor-defined layer names.

There is no de facto cross-tool DXF-PCB layer convention. Every
auditing tool that consumes EDA-emitted DXF requires per-tool
configuration to map "this layer name means pad copper, that layer
name means silkscreen."

**Net assessment:** The existing DXF-from-PCB landscape is exactly
the opposite of "published-and-adopted profile." It is ad hoc per
tool. Datum publishing a *new* profile adds one more ad-hoc
convention to the landscape unless industry adopts it broadly.

#### DXF entity capability survey for PCB semantics

Stock DXF entities mapped against the PCB-semantics surface
Datum's IR encodes:

| PCB semantic | DXF entity | Mapping quality |
|---|---|---|
| Pad outline (rectangle) | `LWPOLYLINE` (closed) | Full |
| Pad outline (circle) | `CIRCLE` or `LWPOLYLINE` arc | Full |
| Pad outline (rounded-rect) | `LWPOLYLINE` with arc segments | Full geometry, not parametric |
| Pad outline (oblong / stadium) | `LWPOLYLINE` with arcs | Full geometry, not parametric |
| Pad outline (chamfered rect) | `LWPOLYLINE` | Full geometry |
| Drill hole | `CIRCLE` on a designated layer | **Convention required** |
| Slot drill | `LINE` segment + width on layer | **Convention required** |
| Plated vs non-plated drill | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Pad number / pin name | `ATTRIB` on `INSERT` of a `BLOCK` | **Convention required** |
| Pad-stack reference | `XDATA` with custom application name | **Convention required** |
| Per-layer copper geometry | DXF layer per PCB layer | Full but ad hoc layer names |
| Solder mask aperture | DXF layer per side | **Convention required** |
| Solder mask expansion (numerical) | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Paste mask aperture | DXF layer per side | **Convention required** |
| Paste reduction (numerical) | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Thermal relief settings | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Courtyard polygon | `LWPOLYLINE` on courtyard layer | Full geometry |
| Silkscreen geometry | `LINE` / `ARC` / `LWPOLYLINE` / `TEXT` | Full geometry |
| Assembly drawing geometry | Same as silkscreen | Full geometry |
| Reference designator | `TEXT` or `ATTRIB` | Full geometry, semantic via convention |
| 3D model reference | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Pad rotation per pin | `INSERT` rotation field | Full |
| Density level (IPC-7351) | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Toe / heel / side fillet (Jt/Jh/Js) | DXF has no encoding | **Out-of-band metadata required (XDATA)** |
| Component lead dimensions (Lmin/Lmax etc.) | DXF has no encoding | **Out-of-band metadata required (XDATA)** |

Of 21 PCB-semantic categories, **9 require Datum-controlled
conventions or XDATA**. Stock DXF carries the geometry and a few
attributes (block instances, ATTRIBs); it carries none of the
PCB-specific *semantics* without convention.

DXF's `XDATA` (extended data) feature attaches application-keyed
arbitrary data to any entity. The application key is registered with
a unique name (e.g., `DATUM_EDA`); other tools see the XDATA but
ignore it unless they recognise the application key. This is
technically the cleanest place to put per-pad metadata
(`datum:pad_shape=roundrect`, `datum:pad_size_mm=1.6x0.8`,
`datum:roundrect_ratio_pct=25`, `datum:density_level=B`).

**Verdict on DXF-as-carrier:** The geometry round-trips. The
semantics do not, except via Datum-specific extensions that no
other tool understands.

#### Parametric DXF round-trip status

AutoCAD 2010+ supports 2D parametric constraints (geometric:
coincident, parallel, perpendicular, etc.; dimensional: distance,
angle, radius with named parameters). Internally these are stored
in `AcDb2dConstraints` and `AcDbAssoc` records.

Public information on DXF round-trip preservation is sparse and
contradictory. The Autodesk DXF Reference documents the
constraint-related entity types (e.g., `ACDBASSOCDIMDEPENDENCYBODY`,
`ACDBPERSSUBENTMANAGER`) under "Object section" but the
documentation is incomplete relative to AutoCAD's internal model.
Multiple Autodesk Discussion-Forum threads from 2018-2024 report
that constraint preservation on DXF round-trip into and back out of
non-AutoCAD tools (Inventor, Civil 3D, BricsCAD, FreeCAD,
LibreCAD) is partial-to-broken — geometric constraints are often
silently dropped, dimensional constraints are sometimes preserved
as fixed-value text annotations rather than re-evaluatable
parameters.

**The Rust `dxf` crate (`dxf-rs`) does not implement constraint
records.** The crate's coverage is general entities (lines, arcs,
polylines, blocks) plus tables (layers, linetypes); the
`ACDBASSOC*` family is in the "not yet supported" bucket. KiCad's
internal dxflib-derived parser similarly does not parse constraints.

**This is the unspoken killer for the DXF-canonical idea.** Even if
Datum encoded its parametric source as AutoCAD-style constraint
records on DXF export, no Rust DXF library would round-trip them,
no non-AutoCAD consumer would understand them, and AutoCAD's own
behaviour is reportedly inconsistent. The "DXF round-trip
preserves the parametric source" assumption fails at the first
hop into any non-AutoCAD reader.

**Open question:** AutoCAD's internal behaviour cannot be verified
from public documentation alone. If the user wants stronger
confidence, the next step is a hands-on AutoCAD experiment —
author a parametric block, save as DXF, re-open in AutoCAD,
verify constraint preservation; then re-open in BricsCAD / FreeCAD
to verify cross-tool behaviour. This research flags it as
unresolved rather than asserting either way.

#### DWG vs DXF for parameter preservation

DWG is binary, carries more internal metadata than DXF, and may
preserve constraints more reliably (since AutoCAD is the canonical
DWG reader). However:

- DWG is Autodesk-controlled with no published spec.
- The only path for non-Autodesk DWG support is the Open Design
  Alliance (ODA) Teigha SDK at ~$15,000/year corporate licensing,
  cross-referenced from
  `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
  line 651-653.
- LibreDWG (GNU/GPLv3) is the open alternative; quality is uneven
  for AutoCAD 2010+ files.
- Datum's no-copyleft rule excludes LibreDWG from direct linkage.
- Domain 1 research already excluded DWG from Datum's roadmap.

**DWG is not a viable carrier for Datum.** This forecloses the
"use DWG instead, since DXF is too lossy for parameters" workaround.

#### Proposed Datum-EDA DXF profile (sketch)

For completeness, a minimal Datum DXF profile that *would* work for
mechanical-layer interchange (not parametric source). The intent is
to make the profile concrete enough to evaluate, not to commit to
shipping it.

**Layer names** (registered with `DATUM_` prefix, all uppercase):

- `DATUM_PAD_TOP`, `DATUM_PAD_BOTTOM` — copper pad geometry per side
- `DATUM_DRILL_PLATED`, `DATUM_DRILL_NONPLATED` — drill positions as
  CIRCLE entities; diameter encoded in CIRCLE's radius
- `DATUM_DRILL_SLOT_PLATED`, `DATUM_DRILL_SLOT_NONPLATED` — slot
  drills as LWPOLYLINE rectangles
- `DATUM_MASK_TOP`, `DATUM_MASK_BOTTOM` — solder mask aperture
  geometry per side
- `DATUM_PASTE_TOP`, `DATUM_PASTE_BOTTOM` — paste mask aperture
  geometry per side
- `DATUM_SILK_TOP`, `DATUM_SILK_BOTTOM` — silkscreen geometry per
  side
- `DATUM_ASSY_TOP`, `DATUM_ASSY_BOTTOM` — assembly drawing
  geometry per side
- `DATUM_FAB_TOP`, `DATUM_FAB_BOTTOM` — fabrication layer
  (component body outline) per side
- `DATUM_COURTYARD_TOP`, `DATUM_COURTYARD_BOTTOM` — courtyard
  polygons per side
- `DATUM_OUTLINE` — package outline (single layer, reference only)
- `DATUM_ORIGIN` — origin marker (single POINT entity at 0,0)
- `DATUM_PIN_1` — pin-1 marker (single POINT entity)

**Block convention for pad instances:**

Each pad is an `INSERT` of a Datum-published BLOCK (e.g., `DATUM_PAD_ROUNDRECT`,
`DATUM_PAD_CIRCLE`, `DATUM_PAD_OBLONG`). The INSERT carries an
`ATTRIB` set:

- `PAD_NAME` (string — pin name, e.g., "1", "GND", "VCC")
- `PAD_NUMBER` (integer — sequential pad number)
- `PAD_NET` (string — net assignment if known; usually empty in
  library context)

The BLOCK definition contains the parametric pad shape (a rounded
rectangle inscribed in the block's bounding box, sized via the
`INSERT`'s X/Y scale factors).

**XDATA convention for per-entity metadata:**

XDATA application key: `DATUM_EDA`. Per pad-INSERT XDATA carries:

- `pad_size_mm` (real pair — width, height)
- `pad_shape` (string — `roundrect`, `oblong`, `circle`,
  `chamfered`, `polygon`)
- `roundrect_ratio_pct` (integer — for roundrect pads)
- `chamfer_corners` (string — `tl,tr,bl,br` subset)
- `mask_expansion_um` (integer — solder mask expansion, default 0)
- `paste_reduction_pct` (integer — paste mask reduction, default 0)
- `padstack_uuid` (string — Datum-pool padstack UUID, opaque to
  outside readers)
- `density_level` (string — `A`, `B`, or `C`)

**Per-package XDATA on the `DATUM_OUTLINE` polygon:**

- `package_uuid` (string — Datum-pool UUID)
- `package_name` (string)
- `ipc_compliance` (string — `IPC-7351B`, `IPC-7352`, `none`)
- `density_level` (string)
- `source_lead_dims_mm` (key-value list — Lmin, Lmax, Tmin, Tmax,
  Wmin, Wmax)
- `solder_fillet_mm` (key-value — Jt, Jh, Js)
- `body_height_mm` (real)

**Parametric source storage:** Out of scope for the DXF profile.
The parametric source (constraint definitions, named parameters,
pattern operations) cannot be encoded in stock DXF and is not
encoded by AutoCAD's DXF export reliably. The recommendation is:
parametric source lives in a sidecar JSON file alongside the DXF,
referenced from the DXF's package-level XDATA (`source_uri`).

#### Verdict for Area 2

> **The Datum DXF profile is technically writeable for the geometry
> half — every PCB layer maps to a DXF layer and every pad shape
> maps to a DXF entity.** The semantic half — pad-vs-silkscreen-vs-
> mask-vs-courtyard, plated-vs-nonplated drill, mask expansion,
> paste reduction, IPC density level, source lead dimensions —
> requires Datum-published conventions (named layers, named blocks,
> XDATA application key) that no other tool understands without
> implementing the profile.
>
> **The parametric source half is unsolvable in DXF.** Stock DXF
> has no parametric primitives; AutoCAD's parametric primitives
> are not preserved on cross-tool round-trip; no Rust DXF library
> implements them. The realistic path is "parametric source in a
> sidecar JSON file, baked geometry in DXF" — at which point the
> DXF is just the export and the canonical artifact is the JSON,
> which is exactly the situation Datum is already in with native
> JSON files (per `docs/NATIVE_FORMAT.md`).
>
> The "DFM auditor reads our DXF without a Datum seat" promise is
> achievable for *geometry* audits (a competent DXF-aware auditor
> can verify "all pads are inside the courtyard" or "no silkscreen
> overlaps a pad") but **not** for *parametric* audits ("verify this
> footprint matches IPC-7351B Density B for SOIC-8") because the
> auditor needs the source dimensions and the IPC formulas, which
> are XDATA the auditor has to choose to implement.

### Area 3 — 2D constraint solver options

#### License-aware survey

The no-copyleft rule (per `feedback_no_copyleft_integration` memory)
forbids GPL-class code in the Datum engine binary. The matrix:

| Solver | License | Suitability for Datum engine | Notes |
|---|---|---|---|
| **D-Cubed DCM** (Siemens) | Commercial paid (~$50k/seat) | Excluded by cost | Industry standard. SolidWorks, Inventor, Solid Edge use it. |
| **PlaneGCS** (FreeCAD) | LGPL-2.1+ | Excluded by static-link relicensing risk | Mature 2D solver. C++. Rust binding would need dynamic-link wrapper; LGPL static-link relicensing problem makes this borderline-to-forbidden per the no-copyleft rule. |
| **Solvespace solver** | GPL-3 | Hard exclude | The solver was extracted as a separate library at one point but the entire Solvespace codebase is GPL-3. |
| **CHOCO solver** | BSD-3 | Wrong domain | Java constraint propagation library; integer / finite-domain constraints. Not built for continuous geometric constraints. Embedding via JNI is operationally infeasible for a Rust engine. |
| **Z3 SMT solver** (Microsoft Research) | MIT | Wrong tool for the job | Z3 can theoretically express geometric constraints but is overkill, slow for interactive editing, and has 30+ MB binary footprint. Usable as a fallback for unusual constraint problems but not as the everyday solver. |
| **Cassowary** (Apple) | Various ports — most permissive | Wrong domain | Linear constraint solver (used in macOS Auto Layout). Geometric constraints are nonlinear; Cassowary cannot express equal-distance, perpendicular, etc. |
| **OptaPlanner** (Red Hat) | Apache 2.0 | Wrong domain | Java meta-heuristic optimisation; not geometric. |
| **Sympy geometry** | BSD-3 | Wrong runtime | Python symbolic geometry. Useful as a reference for algorithm correctness during Datum solver development; not embeddable. |
| **NLopt** (Steven G. Johnson) | LGPL or MIT (per algorithm) | Partial | Nonlinear optimisation library. Could form the backend for a constraint-as-residual-function solver. License is per-algorithm — some MIT, some LGPL. Use only the MIT-licensed algorithms. C library; Rust bindings exist via `nlopt-rs` (MIT). |
| **`argmin` Rust crate** | MIT/Apache 2.0 | Suitable as building block | Mature Rust optimisation framework with Newton-Raphson, Gauss-Newton, Levenberg-Marquardt. Would form the numerical backend for a from-scratch geometric solver. |
| **`nalgebra-sparse`** | Apache-2.0 | Suitable as building block | Rust sparse linear algebra. Needed for the sparse-Jacobian solve at each Newton iteration. |
| **`good_lp`** | MIT | Wrong domain | Linear programming wrapper. Same problem as Cassowary — geometric constraints are nonlinear. |

**No mature permissive-licensed Rust 2D geometric constraint solver
exists on crates.io as of mid-2026.** Searches across crates.io for
"constraint solver", "geometric constraint", "sketch solver", "2d
constraints" return either Cassowary-family linear solvers, generic
optimisation crates, or domain-specific solvers (e.g., for typesetting).

#### Build-from-scratch effort estimate

The constraint vocabulary needed for parametric footprint authoring
is small:

| Constraint type | Algebraic form (residual) | Notes |
|---|---|---|
| Coincident (point-on-point) | `(p1.x - p2.x)² + (p1.y - p2.y)² = 0` | 2 residuals |
| Distance (point-to-point) | `(p1.x - p2.x)² + (p1.y - p2.y)² - d² = 0` | 1 residual |
| Horizontal (line) | `p1.y - p2.y = 0` | 1 residual |
| Vertical (line) | `p1.x - p2.x = 0` | 1 residual |
| Parallel (two lines) | cross-product = 0 | 1 residual |
| Perpendicular (two lines) | dot-product = 0 | 1 residual |
| Equal length (two lines) | `len(L1) - len(L2) = 0` | 1 residual |
| Symmetric (two points about an axis) | reflected coordinate = 0 | 2 residuals |
| Fixed (point at coords) | `p.x - x0 = 0`, `p.y - y0 = 0` | 2 residuals |
| Midpoint | `2·m - (p1 + p2) = 0` | 2 residuals |
| Concentric (two circles) | `c1 - c2 = 0` | 2 residuals |
| Tangent (line to circle) | `dist(line, c) - r = 0` | 1 residual |

**12 constraint types total.** This covers everything a parametric
footprint editor needs: pin pitch (distance), pad arrays (linear
pattern → equal-length + parallel), rectangular packages (horizontal
+ vertical + perpendicular), symmetric arrangements (symmetric),
fixed origin (fixed), pad-to-pad distance equality (equal length).

The solver loop:

1. Collect all constraints into a residual vector `r(x)` where `x` is
   the vector of free coordinates (DOFs).
2. Compute the Jacobian `J = ∂r/∂x` analytically (each constraint's
   derivative is closed-form).
3. Iterate Newton-Raphson: `x_{n+1} = x_n - J^+ · r(x_n)` where `J^+`
   is the pseudoinverse (for over- or under-constrained systems).
4. Use Levenberg-Marquardt damping for stability when the Jacobian
   is near-singular.
5. Detect over-constrained systems (rank deficiency) and emit a
   "constraint conflict" diagnostic naming the conflicting
   constraints.
6. Detect under-constrained systems (DOF > 0 after constraint
   accounting) and emit a "free DOFs remaining" diagnostic.

Reference algorithms are well-documented:

- **FreeCAD PlaneGCS** documentation (LGPL — read for algorithm
  reference, do not copy code) describes the same residual /
  Jacobian / LM-damped Newton approach.
- **Hoffmann, Lomonosov, Sitharam (2001)** — *Geometric constraint
  solving — the witness configuration method*. ACM SIGGRAPH paper;
  algorithm is in the public domain.
- **Foufou, Michelucci, Jurzak (2005)** — *Numerical decomposition
  of geometric constraints*. INRIA technical report; public.
- **Sitharam et al. (FRONTIER project)** — multiple papers on
  geometric constraint solving; reference implementations in C are
  permissive-licensed (BSD).
- **Solvespace's algorithm description** in its design doc is
  readable; reading the *description* is fine, copying code is
  forbidden by the GPL.

**Effort estimate:** A minimum viable solver for the 12-constraint
vocabulary, with Newton-Raphson + LM damping + sparse Jacobian
solve via `nalgebra-sparse` + over/under-constrained diagnostics,
is **roughly 4-6 weeks of focused engineering** for one Rust
engineer with prior numerical-methods experience. This is a
one-time engineering cost; the solver does not need ongoing
maintenance once it works on the regression corpus.

The solver does not need to be world-class. PlaneGCS handles
problems with thousands of free variables; a footprint editor needs
to handle ~100 free variables in the worst case (a 64-pin BGA
needs 64×2 = 128 pad-position DOFs plus a handful of body / silk
DOFs). Even a naive O(n³) dense Jacobian solve is fast enough at
that scale. Sparse handling is a nice-to-have for the largest
parts (1000-ball BGA) but not required for v1.

#### Verdict for Area 3

> **Build from scratch in Rust under MIT/Apache 2.0 is the right
> answer.** PlaneGCS is LGPL with relicensing-on-static-link risk;
> Solvespace is hard-excluded GPL-3; no permissive Rust solver
> exists; commercial DCM is out by cost. The constraint vocabulary
> is small (~12 types), the algorithm is well-documented in the
> public-domain literature, the numerical backend (`argmin`,
> `nalgebra-sparse`, `nlopt-rs` MIT subset) is already in the Rust
> ecosystem, and the scale is modest. **4-6 weeks engineering
> estimate** for a v1 solver.
>
> Risk: under-constrained-system diagnostics and constraint-conflict
> reporting are the parts that take the longest to get right
> (Solvespace and PlaneGCS have spent years polishing the user-
> facing error messages). Datum should plan for an additional
> 2-3 weeks of polish on diagnostics in v1.1.

### Area 4 — Pool / library integration

The current Datum data model (per `specs/ENGINE_SPEC.md` § 1.2) is:

```rust
pub struct Pad {
    pub uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub padstack: Uuid,   // → Padstack
    pub layer: LayerId,
}

pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub models_3d: Vec<ModelRef>,
    pub body_height_nm: Option<i64>,
    pub body_height_mounted_nm: Option<i64>,
    pub tags: HashSet<String>,
}
```

All baked geometry. No parametric source.

#### Recommended ParametricPackage design

A new pool entry type that **generates** `Package` instances:

```rust
pub struct ParametricPackage {
    pub uuid: Uuid,
    pub name: String,
    pub family: PackageFamily,                    // Chip, SOIC, QFP, BGA, Custom, ...
    pub parameters: BTreeMap<String, Parameter>,  // named, ordered by author
    pub constraints: Vec<Constraint>,             // 12-constraint vocabulary
    pub geometry_template: GeometryTemplate,      // skeleton with placeholder coords
    pub pad_template: PadTemplate,                // pad shape derived from parameters
    pub ipc_metadata: Option<IpcMetadata>,        // IPC-7351 lead dims, density, fillets
    pub provenance: Provenance,                   // who authored, when, source standard
}

pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,                      // Length, Count, Angle, Enum, Bool
    pub default: ParameterValue,
    pub constraint: Option<ParameterConstraint>,  // min/max/enum-of
    pub description: String,
}

pub struct ParametricPackageInstance {
    pub uuid: Uuid,
    pub source: Uuid,                             // → ParametricPackage
    pub parameter_values: BTreeMap<String, ParameterValue>,
    pub baked: Package,                           // generated; cached for fast load
}
```

**Authored vs derived split:**

The user authors `ParametricPackage`. The engine derives
`ParametricPackageInstance` whose `baked: Package` field is the
flat geometry ready for board placement. This matches the existing
Datum invariant from `docs/CANONICAL_IR.md` — authored fields are
hand-edited; derived fields are recomputed from authored sources.

**Re-evaluation:** When a parameter value changes, the engine
re-runs the constraint solver and re-bakes `Package`. This is a
deterministic operation (same parameter values always produce the
same baked geometry — needed for the existing import-determinism
gate per the M1/pre-M7 commit `9517a74`).

**Multi-instance parameterisation (chip families):**

The 0402/0603/0805/1206 chip-resistor family is one
`ParametricPackage` (family = Chip) plus four
`ParametricPackageInstance` records, each with different lead
dimensions in the parameter set. The pool storage layout:

```
pool/
  parametric-packages/
    chip/
      <uuid>.json                 — ParametricPackage (the family definition)
  packages/
    chip-0402/<uuid>.json         — ParametricPackageInstance baked Package
    chip-0603/<uuid>.json
    chip-0805/<uuid>.json
    chip-1206/<uuid>.json
```

The `.json` for each instance carries `source: <parametric-uuid>` so
the pool indexer can show "which instances derive from this
parametric source."

**Pool search and MCP surface:**

`ParametricPackage` records appear as a separate top-level entity
in the pool index (alongside `Symbol`, `Padstack`, `Package`,
`Part`). They are searchable by `family`, `parameters` (key-value
search), `ipc_metadata.density_level`, and `provenance.author`.

MCP tool surface:

- `list_parametric_packages` — search and filter
- `get_parametric_package` — fetch ParametricPackage with parameter
  schema
- `instantiate_parametric_package` — create a
  ParametricPackageInstance with given parameter values; returns
  baked Package
- `re_instantiate_parametric_package` — rebake an existing instance
  with updated parameter values
- `solve_parametric_constraints` — diagnostic; runs the solver
  without baking, returns DOF / conflict report

This integrates with the existing operation/transaction model from
`specs/ENGINE_SPEC.md` § 1.4 (684-731). Each parameter change is a
transaction; the rebake is a derived effect.

**Schematic symbol parametric:**

The symbol equivalent (per `research/schematic-drawing-conventions/`
— the IEEE 315 vs IEC 60617 symbol-style profiles already require
runtime symbol selection per project) is a strong fit for the same
parametric mechanism, but with a smaller scope:

- **Parametric input is mostly count-based**: pin count, pin spacing,
  body width, signal-vs-power-vs-ground groupings.
- **Constraint vocabulary is even smaller**: equal-spacing,
  symmetric-about-axis, fixed-pin-grid.
- **Pattern operations dominate**: linear pattern (pins along a
  side) and mirror (pins on opposite sides).

Recommend **shared infrastructure (constraint solver, parameter
model)** between `ParametricPackage` and a new `ParametricSymbol`,
with separate type-level definitions so the `family` enum and IPC
metadata only apply to packages.

**Backwards compatibility with imported (frozen) packages:**

Existing imported packages (KiCad, Eagle, IPC-2581) are not
parametric. The recommendation:

- Add a `Package.origin: PackageOrigin` enum:
  `Imported(format) | Authored | DerivedFromParametric(uuid)`.
- Imported packages are **always frozen** — there is no path to
  retroactively parameterise them automatically. The package
  geometry is the user's authored truth.
- Users can manually create a `ParametricPackage` that *resembles*
  an imported package and use it for new placements; the imported
  package remains unchanged.
- Re-import (e.g., updated KiCad library version) replaces the
  imported package's geometry; if a `ParametricPackage` was
  hand-modelled to match it, the relationship is informational
  only (no automatic re-sync).

This is the same pattern as Altium's "Migrate from imported to
managed library" workflow — manual, opt-in, one-way.

#### Verdict for Area 4

> **The pool integration is clean and incremental.** A new
> `ParametricPackage` type alongside `Package` rather than embedded
> in `Package`; instances are baked into standard `Package` records
> for board consumption; the constraint solver runs at instantiation
> and re-instantiation; the existing operation/transaction model
> covers parameter changes; MCP surface is a small additive set of
> tools. No invasive refactor of `Package` / `Pad` / `Padstack`.
> Symbol parametric reuses the constraint and parameter
> infrastructure with a smaller scope.

### Area 5 — Published-profile viability (the make-or-break question)

The user identified this as the central viability question:
*"published-and-adopted profile = standard; profile that only
Datum reads = vendor lock-in dressed up as openness."*

#### Adoption strategies

The realistic paths to get an industry-watched DXF-PCB profile
adopted:

1. **IPC route.** Submit to the IPC EDGE (Electronic Designs
   Group / Engineering) committee chain. Realistic timeline for
   an IPC standard: 3-7 years from initial submission to
   published revision. IPC has historically not chartered DXF-related
   work (the PCB-mechanical interchange standards in IPC's
   portfolio are IPC-2581 for ECAD-to-fab and IPC's IDF / EDMD
   work for ECAD-to-MCAD; both are XML-family, not DXF-derived).
   No appetite signal exists for IPC adopting a parametric-DXF
   profile.
2. **OASIS Open route.** OASIS hosts technical committees on
   demand if a sponsor pays the membership fee (~$5k/year for
   small companies). A Datum-led TC is technically possible but
   would need at least three industry sponsors to launch and
   ongoing engagement to maintain. Datum is one company; without
   recruited co-sponsors (KiCad team, LibrePCB, Horizon, perhaps
   one PCB Libraries / SnapEDA / SamacSys partner), the TC would
   stall.
3. **OpenEDA loose-grouping route.** No formal standards body;
   informal coordination among open EDA tool maintainers. Lowest
   barrier to publishing a profile but lowest binding force —
   "we've all agreed to read this format" only matters if the
   tools actually ship readers. The risk: a published profile that
   nobody implements is functionally identical to a Datum-only
   profile.
4. **GitHub spec route.** Publish the profile spec on GitHub under
   a permissive license (CC-BY-4.0 or CC0). Fastest to publish,
   weakest endorsement. Useful as documentation-of-decision but
   not as standards adoption.
5. **De facto adoption via reference implementation.** Datum
   ships a reference DXF profile reader in Rust (the natural
   build); separately commits to publishing a Python reference
   reader (broader audience for tool integrators). If at least one
   non-Datum tool implements the profile (KiCad import,
   LibrePCB import, a SnapEDA/SamacSys export path), the de facto
   adoption argument starts.

**Realistic assessment:** Path 5 (de facto via reference impl) is
the only path with non-trivial probability of success. Even there,
the probability of getting a second tool to implement the profile
is low — KiCad, LibrePCB, and Horizon have their own native
formats and IPC-2581 export; their incentive to add a Datum-DXF
import path is weak unless Datum has user share large enough to
justify the engineering cost on the consuming side.

#### DFM tool integration prospects

The auditing tools that matter:

- **Valor MSS / Process Preparation** (Siemens) — reads ODB++
  natively, IPC-2581 with conversion, Gerber + drill. **Does not
  natively consume DXF for footprint-level audit.** Adding a
  Datum-DXF reader would require Siemens engineering effort with
  no commercial driver.
- **CAM350 / DFMStream** (Downstream Technologies) — reads Gerber,
  ODB++, IPC-2581. **No DXF footprint audit path.**
- **Frontline InCAM** (Orbotech, now ASMPT) — same shape.
- **Ucamco UCAMX** (Ucamco) — same shape.
- **Genesys-2000** (Frontline / ASMPT) — same shape.
- **JLCPCB / PCBWay in-house DRC** — proprietary tools that
  consume Gerber + drill. **No public DXF audit path; no
  business reason for them to add one.**

**No DFM auditor consumes generic DXF for footprint-level audit
because Gerber + drill + IPC-2581 already encode everything
needed.** The "DFM audit outside the EDA seat" promise is
**already solved** by the existing fab-tool ecosystem reading
IPC-2581 and ODB++. A new DXF profile would be solving a problem
the industry considers solved.

#### "First 5 years" interim-value risk

Even in the optimistic case where a published Datum DXF profile
gets traction, no DFM tool would implement reading it for at
least 3-5 years. During that window, the marketing claim "DFM
audits outside the EDA seat" would be false in practice — the
auditor would still need either Datum or a Datum-aware DXF
reader.

This window is long enough that the marketing positioning
collapses. Datum's user base in the first 5 years would be
hearing "ship to fab via Gerber/IPC-2581" anyway; the parametric
DXF profile would be inert.

#### Internal-only fallback value

Stripping the "external auditor reads our DXF" claim: is the
parametric editor still worth shipping for Datum's own users?

**Yes.** Internal-only value comes from:

- **Re-parameterisation under design change.** "We need this
  footprint at IPC density A instead of B" — re-runnable
  parametric source rebakes the geometry deterministically.
- **Family-aware editing.** "Generate the 0402, 0603, 0805,
  1206 versions of this chip footprint" from one parametric
  source.
- **AI-tractable footprint authoring.** AI agents can author
  footprints by setting named parameters rather than emitting
  raw geometry — parameter authoring is far more tractable for
  an LLM than "place 8 pads at these 16 specific coordinates."
  This is a real differentiator for Datum's AI-native positioning.
- **Constraint-checked authoring.** The solver catches
  over-constrained ("this footprint has conflicting symmetric +
  fixed constraints") and under-constrained ("this footprint has
  3 free DOFs") situations at author time, not at fab.
- **IPC compliance verification.** Parametric source carrying
  IPC metadata makes "is this footprint IPC-7351B Density B for
  SOIC-8?" answerable inside Datum without external tool calls.

These internal benefits stand even if no external tool ever reads
a Datum-emitted DXF.

#### Verdict for Area 5

> **The "published-and-adopted-DXF-profile" angle is marketing
> fiction in 2026 and stays fiction inside Datum's planning
> horizon.** No DFM auditor consumes DXF for footprint-level
> audit; the industry settled this question by adopting IPC-2581
> and ODB++ for the audit path. A Datum-published profile would
> need adoption by either IPC (3-7 year timeline, no appetite
> signal), an OpenEDA-style coalition (no formal binding force),
> or a willing peer tool (no incentive). The first-5-years
> interim value is zero externally.
>
> **The internal-only fallback is genuinely valuable** —
> parametric authoring, family generation, AI-tractable
> parameter editing, IPC compliance verification, constraint-
> checked authoring. **The recommendation is to ship the
> internal value and drop the external-DXF claim.**

### Area 6 — Architectural recommendation

#### Verdict

> **Pursue the parametric editor; defer the DXF-canonical claim
> indefinitely.**

The two halves of the original idea separate cleanly:

- **Pursue (Phase A):** Parametric footprint and symbol authoring
  with constraint solver, integrated into the existing pool model
  via new `ParametricPackage` / `ParametricSymbol` types that
  bake into standard `Package` / `Symbol` records for downstream
  consumption. Internal Datum value, AI-tractable, IPC-compliance-
  aware.
- **Defer indefinitely (Phase B):** DXF-as-canonical-interchange.
  No DFM auditor reads it; no published-and-adopted standards body
  is in motion; AutoCAD's own parametric DXF round-trip is
  unreliable; the industry already solved the audit-outside-the-
  seat problem with IPC-2581 and ODB++. Drop the marketing claim
  rather than build the unviable feature.

#### Scope (Phase A)

In:

- `ParametricPackage` and `ParametricSymbol` pool types (new) with
  parameter schema, constraint vocabulary, geometry template,
  IPC metadata.
- Constraint solver (build from scratch in Rust under MIT/Apache
  2.0; ~4-6 weeks).
- IPC-7351 / IPC-7352 family templates for the must-have package
  families (Chip, SOIC, QFP, QFN, BGA, SOT, DIP) — extend as the
  user community drives.
- MCP surface for parametric package authoring and instantiation.
- Pool storage layout under `pool/parametric-packages/`.
- DXF mechanical-layer export (already a Domain 1 recommendation
  at ~1 week effort) — useful for MCAD interop, not marketed as
  parametric.

Out:

- DXF-as-canonical-interchange profile.
- Datum-published DXF profile (write-up as documentation only if
  Datum chooses to commit to mechanical-layer export with named
  layers; do not push for adoption).
- AutoCAD parametric-constraint round-trip (technically
  infeasible per Area 2).
- DFM auditor partnerships (no realistic prospects per Area 5).

#### Phasing

**Quarter 1 (M8 prep):**

1. Constraint solver MVP in a new `crates/parametric-solver` crate
   under MIT/Apache 2.0. Vocabulary: 12 constraint types per Area 3.
   Algorithm: Newton-Raphson + Levenberg-Marquardt damping +
   sparse Jacobian (`nalgebra-sparse`). Diagnostics: over- and
   under-constrained detection with naming.
2. `ParametricPackage` / `ParametricSymbol` data model in
   `specs/ENGINE_SPEC.md` § 1.2 extension.
3. Pool storage and indexing per Area 4.
4. MCP surface (5 tools per Area 4).

**Quarter 2 (M8 delivery):**

1. IPC-7351-family templates: Chip, SOIC, QFP, QFN, BGA, SOT, DIP.
2. CLI for parametric package CRUD.
3. Internal regression corpus: every IPC-7351-family template
   produces baked footprints that match a hand-authored golden.
4. Documentation: parametric authoring tutorial; AI agent guide.

**Quarter 3+ (post-M8):**

1. ParametricSymbol templates for IEEE 315 / IEC 60617 standard
   symbol shapes.
2. GUI integration (M7+ work) — parametric authoring UI in the
   footprint editor.
3. Optional: Datum DXF mechanical-layer export with named-layer
   convention (per Area 2 sketch, but as a documented Datum
   extension not a published standard).

#### Top 3 risks if pursued

1. **Constraint solver diagnostic quality.** Building a numerically
   correct solver is bounded; making the over/under-constrained
   diagnostics clear enough that users (and AI agents) can fix
   their footprints without expert help is the part that
   PlaneGCS and Solvespace have spent years polishing. Datum
   should plan an extra 2-3 weeks of post-MVP polish on
   diagnostics. **Severity: medium. Mitigation: regression
   corpus of bad-author cases with expected diagnostic strings.**
2. **IPC-7351 family template completeness.** The 7-family v1
   list (Chip, SOIC, QFP, QFN, BGA, SOT, DIP) covers the 80% of
   parts. The long tail (custom packages, exotic BGA pitches,
   QFN with thermal pads, high-power TO-220 derivatives) is open-
   ended. PCB Libraries Footprint Expert covers ~200 families;
   Datum will not match that in v1. **Severity: medium.
   Mitigation: ship the 7-family v1, document the
   custom-template authoring path so users can extend, and
   accept the long tail will lag commercial tools.**
3. **AI-agent ergonomics.** Parametric authoring is more
   tractable for AI than raw-geometry authoring, but the AI agent
   still needs to know the family-specific parameter vocabulary
   ("for SOIC, set lead_count, body_width, lead_pitch, lead_length,
   density_level"). Documenting this for the LLM at MCP-tool
   description level is critical. **Severity: low-medium.
   Mitigation: each `instantiate_parametric_package` MCP tool
   call returns a parameter schema with examples; AI guide
   documentation includes worked examples per family.**

#### Revisit triggers (for Phase B and the DXF-canonical claim)

The DXF-canonical-interchange decision should be revisited only if
**all** of the following occur:

- IPC charters a parametric-footprint-interchange working group
  with public minutes, **and**
- A non-Datum DFM auditor (Valor, CAM350, Frontline, Ucamco) ships
  a Datum-DXF reader, **and**
- AutoCAD's parametric-DXF round-trip becomes verified-reliable in
  non-AutoCAD readers (by independent testing or Autodesk
  documentation update).

None of these are predictable; the recommendation is to **drop the
DXF-canonical claim from the product narrative entirely** and
revisit only if external events force the question.

#### If skip (formal exclusion text)

If the user chooses to skip Phase A entirely (not the
recommendation, but provided for completeness):

> **Datum does not ship a parametric footprint or symbol authoring
> system in v1.** Footprint and symbol authoring is fully manual or
> imported from external libraries (KiCad, SnapEDA, UltraLibrarian,
> SamacSys, PCB Libraries Footprint Expert). The pool's `Package`
> and `Symbol` records are baked geometry only. Users who need
> IPC-7351 parametric authoring use external tools (Footprint
> Expert, KiCad in-app wizards) and import the baked output. This
> exclusion is documented in `docs/INTEROP_SCOPE.md` and in
> `specs/PROGRAM_SPEC.md` § Out-of-Scope.

This text would land in `docs/INTEROP_SCOPE.md` "Out-of-scope for
v1" if the skip path is chosen. The recommendation in this report
is **not** to skip — the parametric editor's internal value is
real — but the exclusion text is provided so the decision survives
either way.

## Comparison Tables

### Table 1 — DXF profile capability matrix

| PCB semantic category | Stock DXF support | With Datum profile | With sidecar JSON | External tool understands without Datum profile? |
|---|---|---|---|---|
| Pad outline geometry (rect / round / oblong / chamfered) | Full (LWPOLYLINE, CIRCLE) | Full | n/a | Yes — geometry only, no semantic |
| Drill hole position + diameter | Convention required | Full (named layer + CIRCLE radius) | n/a | Only if external tool implements convention |
| Plated vs non-plated drill | None | Via XDATA | n/a | No |
| Pad number / pin name | Convention required | Full (ATTRIB on INSERT) | n/a | Yes if tool reads ATTRIB |
| Pad-stack reference | None | Via XDATA | n/a | No |
| Per-layer copper geometry | Convention required | Full (named layer per side) | n/a | Only if external tool implements convention |
| Solder mask aperture | Convention required | Full (named layer per side) | n/a | Only if external tool implements convention |
| Solder mask expansion (numerical) | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Paste mask aperture | Convention required | Full (named layer per side) | n/a | Only if external tool implements convention |
| Paste reduction (numerical) | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Thermal relief settings | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Courtyard polygon | Full | Full (named layer) | n/a | Yes — geometry only |
| Silkscreen geometry | Full | Full (named layer per side) | n/a | Yes |
| Reference designator | Full (TEXT / ATTRIB) | Full | n/a | Yes |
| 3D model reference | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| IPC density level | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Toe / heel / side fillet (Jt/Jh/Js) | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Component lead dimensions (Lmin/Lmax etc.) | None | Via XDATA | Full | No (DXF) / Yes (sidecar) |
| Parametric source (constraints, named parameters) | None | None reliably | Full | No (DXF lossy) / Yes (sidecar) |
| Pattern operations (linear/circular/mirror) | None reliably | None reliably | Full | No / Yes |
| Solver state (DOF, conflicts) | None | None | Full | No / Yes |

**Net:** DXF carries geometry. Sidecar JSON carries semantics. The
"DXF-as-canonical" framing is wrong; the canonical artifact is
the JSON, the DXF is one of several baked exports. (This is the
situation Datum is already in with `docs/NATIVE_FORMAT.md` —
canonical JSON, derived exports.)

### Table 2 — Constraint solver license matrix

| Solver | License | Static-link with permissive Rust binary? | Subprocess option? | Verdict |
|---|---|---|---|---|
| D-Cubed DCM (Siemens) | Commercial | Allowed (paid) | Allowed | Excluded by cost (~$50k/seat) |
| PlaneGCS (FreeCAD) | LGPL-2.1+ | Borderline (relicensing risk) | Possible (run FreeCAD as subprocess) | Excluded for direct linkage; subprocess is theoretically allowed but FreeCAD-as-subprocess is operationally absurd for a footprint editor |
| Solvespace solver | GPL-3 | **No** (hard exclude per no-copyleft rule) | Allowed | Excluded for linkage; subprocess is allowed but adds 30 MB binary, GUI dependency |
| CHOCO solver | BSD-3 | Yes (via JNI) | Yes | Wrong domain (integer/finite-domain, not geometric) |
| Z3 SMT solver (Microsoft) | MIT | Yes | Yes | Wrong tool for the job (overkill, slow for interactive editing) |
| Cassowary | Various permissive | Yes | n/a | Wrong domain (linear constraints only) |
| OptaPlanner (Red Hat) | Apache 2.0 | Yes (via JNI) | Yes | Wrong domain (meta-heuristic optimisation) |
| Sympy geometry | BSD-3 | n/a | Yes (Python) | Wrong runtime (Python symbolic) — useful as algorithm reference only |
| NLopt | LGPL or MIT (per algorithm) | MIT subset only | Yes | Suitable as numerical backend; partial license care needed |
| `argmin` Rust crate | MIT/Apache 2.0 | **Yes** | n/a | Suitable as numerical backend |
| `nalgebra-sparse` | Apache 2.0 | **Yes** | n/a | Suitable as sparse linear algebra backend |
| `good_lp` | MIT | Yes | n/a | Wrong domain (linear programming only) |
| **Build from scratch in Rust** | **MIT/Apache 2.0** | **Yes** | n/a | **Recommended.** ~4-6 weeks effort, 12-constraint vocabulary, well-documented algorithms |

**Net:** No off-the-shelf permissive Rust geometric constraint solver
exists. Build from scratch using `argmin` + `nalgebra-sparse` as
the numerical backend.

## Risks and Open Questions

### Risks (if pursued)

1. **AutoCAD parametric-DXF round-trip behaviour is unverified.**
   Public information is contradictory. If the user wants to push
   harder on the DXF-canonical angle, the next step is hands-on
   AutoCAD experimentation. This research does not have that
   evidence.
2. **Constraint solver diagnostics quality.** Bounded but high-
   touch engineering. Plan for 2-3 extra weeks of polish.
3. **IPC family template long tail.** v1 ships 7 families;
   commercial tools cover ~200. Communicate this scope clearly
   so users know to expect "use Footprint Expert for exotic
   packages" for several years.
4. **AI-agent parameter vocabulary documentation.** Per-family
   parameter schemas need clear documentation at MCP-tool level
   so LLMs can author parametrically without trial and error.

### Risks (if dropped, i.e. skip-Phase-A)

1. **Datum loses the AI-tractable footprint authoring
   differentiator.** Raw-geometry footprint authoring is not
   well-suited to LLMs; competitors (KiCad-with-Copilot, Altium
   AI features) will eat this space.
2. **No path to family-aware footprint editing.** Users who want
   "the same footprint at IPC density A instead of B" must
   manually re-author.
3. **Internal IPC-compliance verification stays hand-rolled.**
   Without parametric source carrying IPC metadata, "is this
   footprint IPC-7351B Density B compliant?" is answerable only
   by re-deriving from inspection — slow and error-prone.

### Open questions

1. **AutoCAD parametric DXF round-trip:** does it actually
   preserve constraints when consumer is also AutoCAD? When
   consumer is BricsCAD? When consumer is FreeCAD? Hands-on
   verification needed before any commitment to DXF as
   parametric carrier. Recommendation: do not commit to DXF
   parametric carrier; the question becomes moot under the
   defer recommendation.
2. **Symbol vs footprint scope priority:** which goes first if
   both are pursued? Recommendation: footprint first (clearer
   IPC anchor, more obvious differentiation), symbol second
   (smaller scope, reuses infrastructure).
3. **Should the DXF mechanical-layer export use the proposed
   Datum profile (named layers) or just per-PCB-layer naming
   like KiCad?** The KiCad-style ad hoc convention is sufficient
   for current MCAD interchange use cases. Recommendation: ship
   KiCad-style ad hoc naming for the M7+ DXF mechanical export
   (already a Domain 1 recommendation); do not formalise the
   Datum profile until and unless a peer tool commits to
   reading it.
4. **Pool migration of imported packages:** is there ever a path
   from imported (frozen) → derived-from-parametric? Recommendation:
   no automatic migration; manual re-author into parametric
   form by user choice; document as Altium-style "Migrate from
   imported to managed library" workflow.

## Recommendation

> **Pursue Phase A (parametric editor with constraint solver, pool
> integration, IPC-7351 family templates). Defer Phase B (DXF as
> canonical interchange) indefinitely; revisit only if IPC, OpenEDA,
> or a peer tool drives external adoption.**

**Rationale:** The parametric editor's internal value is real
(AI-tractable authoring, family-aware editing, IPC compliance
verification, constraint-checked authoring) and the engineering
cost is bounded (~4-6 weeks solver, ~1 quarter for full Phase A).
The DXF-as-canonical-interchange value depends on external
adoption that no DFM auditor or standards body shows appetite for
in 2026; IPC-2581 and ODB++ already solve the audit-outside-the-
seat problem with stronger semantics than DXF can carry. Shipping
Phase A without Phase B captures the achievable value without
betting on the unachievable.

**Phasing if pursued:**

- Quarter 1: constraint solver, ParametricPackage data model,
  pool storage, MCP surface.
- Quarter 2: IPC-7351 family templates (7 families), CLI,
  regression corpus, documentation.
- Quarter 3+: ParametricSymbol templates, GUI integration,
  optional DXF mechanical-layer export (KiCad-style ad hoc, not
  Datum profile).

**Revisit trigger if deferred:**

> The DXF-canonical-interchange decision should be revisited only
> if all of the following occur: (a) IPC charters a parametric-
> footprint-interchange working group, (b) a non-Datum DFM auditor
> ships a Datum-DXF reader, (c) AutoCAD's parametric-DXF round-
> trip is verified reliable in non-AutoCAD consumers.

**Constraint solver license path:** Build from scratch in Rust
under MIT/Apache 2.0 using `argmin` + `nalgebra-sparse` as the
numerical backend. PlaneGCS (LGPL), Solvespace (GPL-3), and DCM
(commercial paid) are all excluded for the reasons in Area 3.

**Formal exclusion text if Datum chooses to skip Phase A entirely:**
provided in Area 6 ("If skip" subsection) for documentation in
`docs/INTEROP_SCOPE.md` "Out-of-scope for v1".

## Sources

### Prior art (Area 1)

- **PCB Libraries Inc. — LP Wizard / Footprint Expert** —
  pcblibraries.com (no public docs); cross-referenced in
  `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` lines
  1025-1054 and source URLs cited there
  (https://www.pcblibraries.com/forum/, the PCBL Naming Convention
  PDF at cskl.de).
- **IPC-7351B / IPC-7352** — IPC store
  (https://shop.ipc.org/), paywalled. Public formula summaries at
  Sierra Circuits, Altium Resources, Ultra Librarian, PCBSync.
- **Altium IPC Compliant Footprint Wizard** — Altium documentation
  (https://www.altium.com/documentation/altium-designer/footprints-component-pcb-library-pcblib).
- **OrCAD / Allegro Footprint Wizard** — Cadence documentation
  (https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis.html).
- **PADS Land Pattern Creator** — Siemens PADS documentation
  (https://eda.sw.siemens.com/en-US/pcb/pads/professional/).
- **KiCad in-app footprint wizards** — KiCad source
  (https://gitlab.com/kicad/code/kicad/-/tree/master/pcbnew/python/plugins);
  bug tracker GitLab issue #4896
  (https://gitlab.com/kicad/code/kicad/-/issues/4896).
- **kicad-footprint-generator** — official GitLab
  (https://gitlab.com/kicad/libraries/kicad-footprint-generator);
  IPC parameter table at
  `scripts/Packages/ipc_definitions.yaml`.
- **Horizon EDA parametric package model** — direct source at
  `research/horizon-source/src/parameter/{set,program}.{hpp,cpp}`
  and `research/horizon-source/src/gen-pkg/gen-pkg.cpp`
  (specifically lines 35-60 showing parameter program embedded
  in package).
- **Horizon EDA pool_parametric** — `research/horizon-source/src/pool/pool_parametric.{hpp,cpp}`.
- **SolidWorks 2D sketch model** — SolidWorks help
  (https://help.solidworks.com/, paywalled; public summaries at
  multiple SolidWorks community wikis).
- **FreeCAD Sketcher** — FreeCAD wiki
  (https://wiki.freecad.org/Sketcher_Workbench).
- **OASIS Open EDA initiatives** — searched
  (https://www.oasis-open.org/committees/) — none active on
  parametric component formats.

### DXF carrier (Area 2)

- **Autodesk DXF Reference** — current
  (https://help.autodesk.com/view/OARX/2026/ENU/?guid=GUID-235B22E0-A567-4CF6-92D3-38A2306D73F3).
- **`dxf-rs` crate** — https://crates.io/crates/dxf
  (MIT licence, Rust DXF read/write, mature for general entities,
  no constraint records).
- **`dxflib`** — KiCad's internal parser, GPL.
- **AutoCAD parametric constraints (AcDb2dConstraints)** —
  Autodesk Discussion-Forum threads 2018-2024 (search "DXF
  parametric constraints round trip"); contradictory; no
  authoritative single source.
- **Open Design Alliance Teigha SDK** — https://www.opendesign.com/
  (commercial, ~$15k/year corporate).
- **LibreDWG** — https://www.gnu.org/software/libredwg/ (GPL-3,
  uneven AutoCAD 2010+ compatibility).
- Cross-referenced from
  `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
  lines 619-686.

### Constraint solvers (Area 3)

- **D-Cubed DCM** — https://www.plm.automation.siemens.com/global/en/products/parasolid/dcm-products.html
  (commercial paid).
- **PlaneGCS / FreeCAD** — https://wiki.freecad.org/Sketcher_PlaneGCS;
  source at https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/Sketcher/App/planegcs
  (LGPL-2.1+).
- **Solvespace** — https://solvespace.com/ (GPL-3); solver
  description at https://github.com/solvespace/solvespace/blob/master/exposed/SOLVER_INTERFACE.md.
- **CHOCO** — https://choco-solver.org/ (BSD-3).
- **Z3** — https://github.com/Z3Prover/z3 (MIT).
- **Cassowary** — original (Apple) and various ports.
- **OptaPlanner** — https://www.optaplanner.org/ (Apache 2.0).
- **NLopt** — https://nlopt.readthedocs.io/ (mixed LGPL/MIT
  per algorithm).
- **`argmin` Rust crate** — https://crates.io/crates/argmin
  (MIT/Apache 2.0).
- **`nalgebra-sparse` Rust crate** — https://crates.io/crates/nalgebra-sparse
  (Apache 2.0).
- **Hoffmann, Lomonosov, Sitharam (2001)** — *Geometric constraint
  solving — the witness configuration method*. ACM SIGGRAPH paper.
- **Foufou, Michelucci, Jurzak (2005)** — *Numerical decomposition
  of geometric constraints*. INRIA technical report
  (https://hal.archives-ouvertes.fr/).
- **Sitharam et al. FRONTIER project** — public papers on
  geometric constraint solving (https://www.cise.ufl.edu/~sitharam/).

### Pool integration (Area 4)

- **Datum `Package` and `Pad`** — `specs/ENGINE_SPEC.md` § 1.2
  lines 217-235.
- **Datum `Part`** — same source, lines 242-263.
- **Datum pool architecture** — `docs/POOL_ARCHITECTURE.md`.
- **Datum canonical IR (authored vs derived invariant)** —
  `docs/CANONICAL_IR.md`.
- **Datum operation/transaction model** — `specs/ENGINE_SPEC.md`
  § 1.4 lines 684-731.
- **Datum native format** — `docs/NATIVE_FORMAT.md`.
- **Horizon parametric pool entries** — direct source as cited in
  Area 1.

### Published-profile viability (Area 5)

- **IPC standards committee process** — https://www.ipc.org/standards/standards-development.
- **OASIS Open committee chartering** — https://www.oasis-open.org/policies-guidelines/.
- **Valor MSS** — https://eda.sw.siemens.com/en-US/pcb/valor/.
- **CAM350 / DFMStream** — https://www.downstreamtech.com/.
- **Frontline InCAM** — https://www.frontline-pcb.com/ (now under
  ASMPT).
- **Ucamco UCAMX** — https://www.ucamco.com/.
- **IPC-2581 adoption status** — cross-referenced from
  `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` lines 64-71
  and `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
  lines 110-124.

### Project policy

- **No-copyleft-integration rule** —
  `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/feedback_no_copyleft_integration.md`.
- **Research-only mode rule** —
  `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/feedback_research_only_mode.md`.
- **Datum attribution policy** — `/CLAUDE.md` § Attribution Policy.
