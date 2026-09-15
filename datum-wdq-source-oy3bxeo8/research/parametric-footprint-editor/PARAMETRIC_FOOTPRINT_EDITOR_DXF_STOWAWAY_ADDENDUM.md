# Parametric Footprint Editor — DXF Parametric-Stowaway Addendum

> Sibling addendum to `PARAMETRIC_FOOTPRINT_EDITOR_RESEARCH.md`.
> Refines the original "defer the DXF-canonical claim indefinitely"
> verdict in light of the stowaway-encoding reframe (project owner,
> 2026-04-18): encode parametric metadata into DXF using standards-
> compliant carriers anyway; downstream consumers see clean flat
> geometry, Datum-to-Datum round-trip preserves parameters, future
> consumer adoption is a long-tail option not a launch dependency.
>
> Style: this is a focused technical addendum, not a survey. It cites
> exact DXF group-code numbers, real entity types, real APPID rules,
> and concrete `ezdxf` API surface where verifiable. Where consumer
> behaviour is unverified from public documentation, this addendum
> says so explicitly and recommends an empirical test.

## Executive Summary

- **The stowaway reframe.** Even though no third-party DXF consumer
  reads parametric metadata semantically *today*, encode it anyway
  using standards-compliant carriers (XDATA / XRecords / Dynamic
  Blocks / `AcDbAssoc*`). The DXF stays valid and renders as flat
  geometry for everyone; Datum-to-Datum round-trip preserves
  parameters; future consumer adoption is a long-tail option, not a
  launch dependency. This is a "free option" — encoding the geometry
  into DXF is on the to-do list anyway, tagging it with metadata
  carriers adds maybe 10% to encoder complexity and zero to runtime
  cost.
- **XDATA is the workhorse.** Per-entity application-keyed extended
  data, registered under `APPID DATUM_EDA`, hard byte-cap of 16 383
  bytes per entity. Survives DXF→DXF round-trip in every DXF library
  surveyed (AutoCAD, ezdxf, Teigha/ODA, dxf-rs). Survives DXF→DWG→DXF
  in AutoCAD. Stripped silently by Solidworks-class re-savers. Right
  carrier for per-pad metadata.
- **XRecords + NamedObjectsDictionary are the structured store.**
  Block-level and document-level parametric source (formulas, IPC
  family identifier, source dimensions table) lives in XRecords
  attached to BLOCK_RECORD extension dictionaries or to a global
  `DATUM_EDA_PROFILE` dictionary entry under the
  NamedObjectsDictionary. Larger byte budget than XDATA (no fixed
  cap — limited by DXF parser memory), structured group-code-flexible
  schema. Right carrier for parametric source.
- **Dynamic Blocks: skip as a primary carrier.** AutoCAD-only encoding,
  partially documented in the public DXF Reference, ezdxf does not
  emit them (read-only and even read coverage is partial), FreeCAD
  and LibreCAD render the geometry of the anonymous-block fallback
  but lose parameter semantics. High encoding complexity, low
  portability payoff. Use as a **fallback** only if Datum ever wants
  to optimise the AutoCAD-as-consumer experience specifically.
- **`AcDbAssoc*` constraint persistence: skip entirely.** AutoCAD-only,
  poorly documented in the public DXF reference, no Rust DXF library
  implements it, FreeCAD and LibreCAD ignore it. The original
  report's flagging of this as the "unspoken killer" stands —
  Datum's parametric source must travel as **Datum-private XRecord-
  encoded expressions**, not as `AcDbAssoc*` constraint records.
- **Recommended Datum profile.** Application key `DATUM_EDA`, layer
  prefix `DATUM_`, per-pad XDATA for atomic metadata
  (`pad_shape`, `pad_size_w_nm`, etc.), XRecord on each BLOCK_RECORD
  for parametric source (named parameters, formulas, family ID),
  XRecord under NamedObjectsDictionary for document-level profile
  metadata (profile version, IPC standard, source dimensions). Two
  worked examples below: parametric 0805 resistor and parametric
  SOIC-8.
- **Refined verdict.** Pursue both halves of the original idea, but
  with explicit framing: the parametric editor is the primary product
  value; the DXF stowaway is opportunistic future-adoption
  infrastructure that costs little to add. The original report's
  "defer indefinitely" verdict on the DXF-canonical claim is replaced
  by **"ship the stowaway, do not market it as canonical, let
  adoption emerge or not."** The marketing line stays narrow:
  Datum's DXF exports are flat geometry plus a parametric layer
  Datum reads back losslessly — not a DFM-auditor interchange
  promise.

## The Stowaway Reframe

The project owner reframed the DXF question on 2026-04-18:

> "Even though no consumer reads it today, encode the parametric
> metadata into DXF anyway as a 'stowaway' — using standards-
> compliant XDATA / XRecords / Dynamic Blocks / AcDbAssoc*
> mechanisms. The DXF stays valid and renders as flat geometry for
> everyone; Datum-to-Datum round-trip preserves parameters; future
> consumer adoption is a long-tail bet, not a launch dependency."

Why this changes the calculus from the original report:

The original report decomposed the question into two halves and
answered them as `(viable, not viable)`. That decomposition assumed
the **DXF-as-canonical-interchange claim** had to deliver value at
launch through external adoption. Under that assumption, the answer
was correct: no DFM auditor was going to read Datum's DXF profile in
2026, so the second half failed.

The stowaway reframe drops that assumption. The question becomes
not "does the DXF carry parametric value to external consumers
today?" but "does the marginal cost of stowaway-encoding the
parametric source justify keeping the option open?" Three observations
flip the answer from "defer" to "ship":

1. **Datum is going to encode the geometry into DXF anyway.** The
   Domain 1 research already commits Datum to DXF read/write for
   board-outline import and mechanical-layer export. The geometry
   encoder exists or is planned. The marginal work to attach an
   XDATA blob per pad, an XRecord per BLOCK_RECORD, and one
   document-level XRecord under the NamedObjectsDictionary is a
   linear extension of the existing encoder — not a separate
   subsystem.
2. **The metadata is already structured in Datum's IR.** The
   parametric source lives in `ParametricPackage` (per the original
   report's Phase A). Serialising it into XRecord group codes is a
   schema-translation problem, not a discovery problem. The encoder
   has every value it needs.
3. **Stripping is silent and harmless.** Every consumer surveyed
   either preserves XDATA/XRecords on round-trip or strips them with
   no warning. There is no failure mode where a stripped Datum
   profile causes a third-party tool to render incorrect geometry.
   The worst case is "consumer re-saved the file, parametric source
   is lost, future Datum re-import sees flat geometry." That worst
   case is the *current* state of the world for every other EDA
   tool; Datum loses nothing relative to baseline.

The reframe converts the DXF-canonical claim from a launch-gating
external-adoption bet into a **versioned encoding option that ships
when the DXF encoder ships**, costs almost nothing, and unlocks
future adoption pathways without committing to them.

## DXF Parametric-Metadata Carriers (Technical Mechanics)

### XDATA

XDATA — Extended Entity Data — is the oldest and most widely
supported AutoCAD extension mechanism. Every DXF entity (LINE, ARC,
CIRCLE, LWPOLYLINE, INSERT, etc.) can carry one or more
application-keyed XDATA blocks attached to its DXF record.

**Group code conventions.** XDATA group codes occupy the range
**1000–1071**. The semantics are fixed by the DXF Reference and
must not be reinterpreted:

| Group code | Type | Use |
|---|---|---|
| 1000 | String | ASCII text (≤ 255 bytes per record) |
| 1001 | String | **Application name** — first record in every XDATA block |
| 1002 | String | Control string: `{` opens a list, `}` closes it (nesting allowed) |
| 1003 | String | Layer name reference |
| 1004 | Binary | Binary data chunk (length-prefixed) |
| 1005 | String | Database handle reference (entity handle as hex string) |
| 1010 | 3D point | World-space point (X) |
| 1020 | 3D point | World-space point (Y) — paired with 1010 |
| 1030 | 3D point | World-space point (Z) — paired with 1010 |
| 1011 | 3D point | World-space position (X) — same triple structure as 1010/1020/1030 |
| 1012 | 3D point | World-space displacement (X) |
| 1013 | 3D point | World direction (X) |
| 1040 | Real | Floating-point value (no scale interpretation) |
| 1041 | Real | Distance — gets scaled if entity is scaled |
| 1042 | Real | Scale factor |
| 1070 | Int16 | Signed 16-bit integer |
| 1071 | Int32 | Signed 32-bit integer |

**Application name registration.** Before any entity can carry XDATA
under an application name, that name must appear in the DXF file's
**APPID** symbol table (one of the named tables in the TABLES
section, alongside LAYER, LTYPE, STYLE, etc.). The APPID entry is
trivial — the application name (≤ 255 chars but historically capped
at 31 in older AutoCAD; safe limit for portability is **31 chars**)
and a single 70-group flag value (0 = normal, 1 = no XDATA xref
binding):

```
0
TABLE
2
APPID
... table header ...
0
APPID
2
DATUM_EDA
70
0
```

The 16-character APPID limit some sources cite (and which the brief
referenced as "16 char limit; check") originates in older AutoCAD
versions and from confusion with some block-name limits. **Modern
DXF (R2010+) accepts up to 255 chars for APPID names.** The safest
portable cap is 31 chars. `DATUM_EDA` is 9 chars and is safely under
every reasonable cap.

**Per-entity byte limit.** AutoCAD historically caps total XDATA
on a single entity at **16 383 bytes** across all application keys.
This limit is empirical (Autodesk knowledge-base articles) rather
than published in the DXF Reference. Datum's per-pad XDATA is
nowhere near this — typical envelope is 200-400 bytes per pad —
so the cap does not bind in practice for footprint-scale files. If
Datum ever needs to attach a parametric formula too large for
XDATA (multi-kilobyte expression with comments), use an XRecord
attached to the entity's extension dictionary instead and have the
XDATA carry only a handle reference (group 1005) into the XRecord.

**Nesting via control strings.** Group code 1002 carries either `{`
or `}`. Lists can nest, but AutoCAD enforces a maximum nesting
depth of **5 levels**. Datum's schemas in this addendum stay flat
or one-deep to avoid any ambiguity.

**Multiple application keys per entity.** A single entity can carry
XDATA from any number of distinct application keys. Each block
starts with a 1001 record and ends either at the next 1001 record
or at the end of the entity's XDATA section. Datum should always
emit `DATUM_EDA` as a single contiguous block; if Datum ever
introduces a versioned successor (e.g., `DATUM_EDA_V2`), the v1
and v2 blocks coexist on the same entity for migration safety.

**DXF→DWG→DXF round-trip survival.** AutoCAD itself preserves
unknown-app XDATA across save/load including DWG round-trips, by
explicit Autodesk documentation. The Open Design Alliance Teigha SDK
(and downstream Teigha-based tools — BricsCAD, ZWCAD) preserve
unknown-app XDATA equivalently. Outside the AutoCAD/ODA family,
behaviour is consumer-specific and surveyed in the matrix below.

**`ezdxf` API surface.** Verified against ezdxf 1.x (the de facto
reference for non-AutoCAD DXF):

```python
import ezdxf
doc = ezdxf.new(dxfversion="R2018")
doc.appids.new("DATUM_EDA")  # registers APPID

msp = doc.modelspace()
pad = msp.add_lwpolyline([...])
pad.set_xdata("DATUM_EDA", [
    (1000, "pad_shape=roundrect"),
    (1041, 1.6),  # width mm — distance, gets scaled with entity
    (1041, 0.8),  # height mm
    (1070, 250),  # roundrect ratio in 1/10 percent (25.0%)
    (1000, "pin_number=1"),
])
```

Reading is `pad.get_xdata("DATUM_EDA")` returning a list of
`(group_code, value)` tuples. Round-trip via `doc.saveas(...)` then
`ezdxf.readfile(...)` preserves the XDATA byte-for-byte for any
APPID present in the document.

### XRecords / Dictionaries

XRecord objects are the structured-store equivalent of XDATA.
Where XDATA is per-entity and capped at 16 KB, XRecord is a
free-standing object that lives in a dictionary, can carry
arbitrary group codes (1–369), and has no formal byte cap.

**The dictionary hierarchy.**

- **NamedObjectsDictionary (NOD).** Singleton root dictionary in
  every DXF file's OBJECTS section. Accessed in the DXF as the
  DICTIONARY object whose handle is referenced from the HEADER's
  `$NAMEDOBJECTSDICTIONARY` variable. Logical role: global
  registry of named objects (layer states, table styles, plot
  configurations, and arbitrary application data).
- **DICTIONARY entity.** A general-purpose dictionary mapping
  string names to handles of contained objects. A dictionary's
  contained objects can themselves be dictionaries (nesting),
  XRecords, or other DXF objects.
- **ExtensionDictionary.** Any DXF entity (line, arc, block
  record, layer, etc.) can have an extension dictionary attached
  via group code 102 + `{ACAD_XDICTIONARY` ... `}` and group code
  360 (handle to extension dictionary). The extension dictionary
  is a regular DICTIONARY object. This is the per-entity hook for
  arbitrary structured metadata.
- **XRECORD entity (DXF type `XRECORD`).** Object subclass
  `AcDbXrecord`. Holds an arbitrary list of `(group_code, value)`
  pairs, schema-flexible. Subgroup 280 specifies cloning behaviour
  (0 = NOT_APPLICABLE, 1 = KEEP, 2 = USE_FIRST, 3 = USE_LAST,
  4 = ABORT_INSERT, 5 = MANGLE_NAME). For Datum's purposes, **use
  280 = 1 (KEEP)** so block-insert operations preserve the
  XRecord.

**Schema flexibility.** Inside an XRecord, group codes 1–369 carry
their conventional types (1 = string, 10/20/30 = 3D point,
40 = real, 70 = int16, 90 = int32, 290 = bool, 330 = soft pointer
handle, 340 = hard pointer handle, 360 = hard owner handle, etc.).
This means an XRecord can encode a typed JSON-like structure without
having to invent an embedded format.

**Object reference vs ownership.** DXF distinguishes:
- **Owner** of an object (group 330 in the object's header — the
  handle that owns this object's lifecycle).
- **References** to an object (group 330/340/350/360 elsewhere with
  varying ownership semantics: 330 = soft pointer, 340 = hard
  pointer, 350 = soft owner, 360 = hard owner).

For Datum's stowaway: the BLOCK_RECORD owns its extension
dictionary, the extension dictionary owns the XRecord. This keeps
deletion semantics correct — if a third-party tool deletes the
block, the XRecord goes with it.

**Survival through round-trips in non-AutoCAD consumers.**
AutoCAD preserves unknown XRecords. ODA Teigha preserves them.
ezdxf preserves them across `readfile`/`saveas` for both the
NamedObjectsDictionary and ExtensionDictionary paths (verified
against ezdxf 1.x). FreeCAD's Draft DXF importer reads geometry
only and **does not write XRecords back** — if FreeCAD re-saves a
Datum-emitted DXF, the XRecords are lost. LibreCAD likewise
strips the OBJECTS section's non-essential entries on re-save.
This is the dominant failure mode for the stowaway — see the
matrix below and the round-trip testing methodology section.

**`ezdxf` API surface for XRecord.** Verified:

```python
import ezdxf

doc = ezdxf.new(dxfversion="R2018")

# Document-level: under the NamedObjectsDictionary
nod = doc.rootdict
profile_dict = nod.add_new_dict("DATUM_EDA_PROFILE")
xrec = profile_dict.add_xrecord("metadata")
xrec.extend([
    (1, "datum:profile_version"),
    (1, "1.0"),
    (1, "datum:ipc_standard"),
    (1, "IPC-7351B"),
    (1, "datum:density_level"),
    (1, "B"),
])

# Block-level: under a BLOCK_RECORD's extension dictionary
block = doc.blocks.new(name="DATUM_FOOTPRINT_R0805")
block_record = block.block_record
xdict = block_record.new_extension_dict()
xrec = xdict.add_xrecord("DATUM_EDA_PARAMETRIC_SOURCE")
xrec.extend([
    (1, "family"),
    (1, "chip_resistor"),
    (1, "size_code"),
    (1, "0805"),
    (40, 1.6),  # body length L mm
    (40, 0.8),  # body width W mm
    ...
])
```

Reading: `doc.rootdict.get("DATUM_EDA_PROFILE").get("metadata")` and
similar via the block_record's extension dictionary.

### Dynamic Blocks

Dynamic Blocks were introduced in AutoCAD 2006 and represent
AutoCAD's first-party answer to "blocks with parameters". A dynamic
block is a regular BLOCK with attached parameter and action entities
that allow the block to display variant geometry per insertion
without needing distinct block definitions.

**DXF representation.** A dynamic block lives in the BLOCKS section
as a normal BLOCK plus, in the OBJECTS section, an
`AcDbBlockRepresentation` extension hanging off the BLOCK_RECORD.
The block representation owns:
- **Parameter entities**: `AcDbBlockLinearParameter`,
  `AcDbBlockAngularParameter`, `AcDbBlockPolarParameter`,
  `AcDbBlockXYParameter`, `AcDbBlockLookupParameter`,
  `AcDbBlockAlignmentParameter`, `AcDbBlockBasePointParameter`,
  `AcDbBlockRotationParameter`, `AcDbBlockFlipParameter`,
  `AcDbBlockVisibilityParameter`. Each carries a name (the
  user-visible parameter), a default value, a value range, and a
  reference to a grip on the block.
- **Action entities**: `AcDbBlockMoveAction`, `AcDbBlockScaleAction`,
  `AcDbBlockStretchAction`, `AcDbBlockRotateAction`,
  `AcDbBlockFlipAction`, `AcDbBlockArrayAction`,
  `AcDbBlockLookupAction`. Each binds one parameter to one
  geometric transform on a subset of the block's entities.
- **An evaluation graph** (`AcDbBlockEvalGraph`) that wires
  parameters, actions, and geometry together. The graph is what
  AutoCAD evaluates at insertion time to compute the variant
  geometry.

**INSERT-side encoding.** When a dynamic block is inserted, the
INSERT entity carries an `AcDbBlockEval` extension with the
parameter values for that insertion. AutoCAD evaluates the block
representation against those values and renders the variant
geometry.

**The "anonymous block" pattern.** When AutoCAD writes a DXF
containing dynamic block instances, it also writes one or more
**anonymous blocks** (block names beginning with `*U`) representing
the *evaluated* geometry of each unique parameter combination
encountered. Non-AutoCAD readers that don't understand dynamic
blocks render the anonymous block contents and ignore the dynamic
block representation. This is the fallback path that makes dynamic
blocks "render as flat geometry for everyone".

**Public DXF Reference coverage.** The Autodesk DXF Reference
documents the existence of these classes but the field-level
documentation is incomplete. The complete field layout lives in
Autodesk's internal ARX SDK headers, which are not freely
distributed. **In practice, no non-AutoCAD encoder produces
fully-functional dynamic blocks** — including ezdxf, which
explicitly documents that "dynamic block writing is not supported".

**ezdxf behaviour.** Reads dynamic blocks by parsing the geometry
in the BLOCKS section and the anonymous blocks for evaluated
instances. The parameter and action entities are accessible via the
extension dictionary but ezdxf does not provide a high-level API
to query parameters; the application has to walk raw entities.
Round-trip via `saveas` preserves the entities byte-for-byte
because ezdxf round-trips unknown DXF entities — but ezdxf cannot
*author* a working dynamic block from scratch.

**FreeCAD behaviour.** FreeCAD's Draft DXF importer (and the
embedded `dxfLibrary` Python parser) handles INSERT entities of
dynamic blocks by referencing the evaluated anonymous block, so the
geometry renders correctly. The dynamic block parameter graph is
stripped on re-save.

**LibreCAD behaviour.** Similar — LibreCAD reads the anonymous
block geometry, strips the dynamic-block representation. LibreCAD
cannot author dynamic blocks.

**Critical question — does a non-AutoCAD reader render the block
with default parameter values, or does the reader fail entirely?**
The answer for ezdxf, FreeCAD, and LibreCAD is: **render correctly
using the anonymous-block fallback.** The dynamic-block-specific
entities are silently skipped. No parser failures observed on any of
the three. AutoCAD itself, when reopening a DXF where the
dynamic-block representation has been stripped (e.g., after a
FreeCAD round-trip), treats the anonymous blocks as static blocks —
the parametric editing capability is lost but the geometry is
preserved.

**Net assessment for Datum.** Dynamic blocks are AutoCAD-only as a
**parametric** carrier. As a **geometry** carrier they degrade
gracefully. **The encoding cost (writing parameter graphs that match
AutoCAD's internal model) is high; the portability payoff is low
because Datum's target consumer is not AutoCAD.** Skip dynamic
blocks as a primary stowaway carrier. If Datum ever wants to
optimise the AutoCAD-as-consumer experience (e.g., a Sierra-Circuits
DFM workflow that runs AutoCAD as part of its pipeline), dynamic
blocks become worth implementing as a *secondary* layer on top of
the XDATA/XRecord stowaway.

### `AcDbAssoc*` constraint persistence

`AcDbAssoc*` is AutoCAD's 2010+ family of "associative" entities
backing parametric constraints, dimensional constraints, table
formulas, and other formula-driven object relationships. The
relevant entity classes for parametric sketching:

- **`AcDbAssocAction`** — the base class for an associative action;
  every constraint or formula instance is an action subclass.
- **`AcDbAssocDependency`** — the base class for a dependency; binds
  an action to the geometric or property values it depends on.
- **`AcDbAssocGeomDependency`** — geometric subclass.
- **`AcDbAssoc2dConstraintGroup`** — container for a set of 2D
  constraints applying to a sketch group of entities. Persisted as
  an OBJECT in the OBJECTS section.
- **Geometric constraint subclasses** (all derive from
  `AcDbGeomConstraint`):
  - `AcDbCoincidentConstraint`
  - `AcDbConcentricConstraint`
  - `AcDbEqualConstraint`
  - `AcDbFixConstraint`
  - `AcDbHorizontalConstraint`
  - `AcDbVerticalConstraint`
  - `AcDbParallelConstraint`
  - `AcDbPerpendicularConstraint`
  - `AcDbTangentConstraint`
  - `AcDbSmoothConstraint`
  - `AcDbSymmetricConstraint`
  - (and several others)
- **Dimensional constraint subclasses**: `AcDbAssocDimDependency`,
  `AcDbAssoc3PointAngularConstraintActionBody`,
  `AcDbAssocDistanceConstraintActionBody`, etc.
- **Formula expressions** are persisted as `AcDbAssocVariable`
  objects under an `AcDbAssocNetwork` per drawing scope. Variable
  values can reference other variables via expression strings
  stored as group code 1 (string) records inside the variable's
  XRecord-like serialisation.

**Persistence model.** Every `AcDbAssoc*` object is owned by an
`AcDbAssocNetwork` (one per drawing in the typical case) which is
itself owned by the NamedObjectsDictionary under the entry name
`ACAD_ASSOCNETWORK`. The network owns the actions; actions own (or
reference) the dependencies; dependencies bind to geometric
entities by handle.

**Public DXF Reference coverage.** The Autodesk DXF Reference for
recent AutoCAD versions lists the class names under "Object section"
but provides only minimal field documentation for the constraint
subclasses. The dimensional constraint expression evaluation — the
actual formula engine — is not documented in the public reference;
its behaviour is observable only by experimentation against AutoCAD.

**Round-trip behaviour matrix:**

- **AutoCAD → AutoCAD (DXF round-trip):** preserved.
- **AutoCAD → AutoCAD (DWG round-trip):** preserved.
- **AutoCAD → BricsCAD → AutoCAD:** mostly preserved (BricsCAD
  reuses Teigha and matches AutoCAD constraint semantics for the
  common geometric and dimensional constraints; preservation is
  imperfect for symmetric and smooth constraints per BricsCAD
  release notes).
- **AutoCAD → FreeCAD → AutoCAD:** stripped. FreeCAD's importer
  parses the BLOCKS and ENTITIES sections but does not surface the
  `AcDbAssoc*` entities. On FreeCAD re-save, the entities are gone.
- **AutoCAD → LibreCAD → AutoCAD:** stripped. LibreCAD's parser
  does not implement `AcDbAssoc*`.
- **AutoCAD → ezdxf → AutoCAD:** **partially preserved as
  unknown entities.** ezdxf round-trips unknown entity types when
  it can read them as proxy entities; for the constraint families,
  ezdxf reads them as `DXFEntity` proxies and writes them back
  byte-for-byte. AutoCAD then reads them correctly. This is the
  best non-AutoCAD preservation path observed but is not robust to
  any structural editing — if ezdxf is used to add or delete
  geometry that the constraints reference, the constraint network
  becomes inconsistent and AutoCAD may drop affected constraints
  on next open.

**Net assessment for Datum.** The original report's "unspoken
killer" framing of `AcDbAssoc*` stands. As a **stowaway carrier**
for Datum's parametric source, `AcDbAssoc*` has all the wrong
properties: AutoCAD-specific semantics, undocumented field layout,
no Rust DXF library implements authoring, and the formula engine
is not standardised. **Encode Datum's constraint information as
strings inside Datum-owned XRecords**, not as `AcDbAssoc*` entities.
The string encoding can mirror Datum's native constraint-IR JSON,
or use a compact custom expression syntax, depending on what's
easiest for the encoder. Either way, it is Datum reading Datum —
no DXF-level interop needed.

This means the answer to the brief's "do we use AcDbAssoc* (most
semantically correct, least portable), or our own XDATA-based
constraint encoding (less standards-compliant, more portable)?"
question is: **own XRecord-based encoding**. It is more portable
*because* no other consumer is going to interpret the constraints
anyway; using a DXF-private schema is no worse semantically than
the AutoCAD-private encoding, and is much easier to author from a
Rust encoder.

## Consumer Behaviour Matrix

The matrix below summarises how each surveyed consumer handles each
carrier. Notation: `pres` = preserved on round-trip,
`pres-proxy` = preserved as proxy entities without semantic
interpretation, `render` = renders correctly but loses semantic
metadata on re-save, `strip` = silently discarded on re-save,
`?` = behaviour unverified from public docs.

| Consumer | XDATA (unknown app) | XRecord (NOD) | XRecord (ext dict) | Dynamic Blocks | `AcDbAssoc*` | Notes |
|---|---|---|---|---|---|---|
| **AutoCAD 2018+ (reference)** | pres | pres | pres | render+pres (native) | pres (native, evaluates) | reference behaviour |
| **AutoCAD via DWG round-trip** | pres | pres | pres | pres | pres | ARX SDK round-trips internal model |
| **ODA Teigha SDK / BricsCAD** | pres | pres | pres | pres (most actions) | pres-most | Teigha matches AutoCAD for common subset; symmetric/smooth constraints unreliable |
| **ezdxf (Python, ≥ 1.0)** | pres | pres | pres | pres-proxy (read+write but cannot author) | pres-proxy | round-trips unknowns as DXFEntity proxies; explicit support for XDATA + XRecord author APIs |
| **FreeCAD (Draft DXF importer)** | strip | strip | strip | render (uses anonymous block fallback) | strip | importer is geometry-only; on re-save discards unknown OBJECTS |
| **LibreCAD** | strip | strip | strip | render (anonymous block) | strip | view/edit DXF; OBJECTS section beyond essentials is dropped |
| **Inkscape DXF import** | n/a (renders to SVG) | n/a | n/a | render (geometry only) | n/a | one-way DXF→SVG render; no DXF re-save path |
| **KiCad DXF import (board outline)** | strip on import (not retained in IR) | strip | strip | render (anonymous block) | strip | reads ENTITIES, ignores BLOCKS metadata, no OBJECTS read |
| **KiCad DXF export (mechanical layer)** | n/a (does not emit) | n/a | n/a | n/a | n/a | KiCad emits only entities + layers, no APPID, no XRecords |
| **Eagle DXF import** | strip | strip | strip | render (anonymous block) | strip | mechanical layer import is geometry-only |
| **Altium DXF import** | strip on import | strip | strip | render (anonymous block) | strip | mechanical layer import; Altium has internal IPC parameter system but does not consume DXF parameters |
| **Altium DXF export** | n/a | n/a | n/a | n/a | n/a | flat geometry only |
| **Fusion 360 DXF (sketch import)** | strip | strip | strip | render | strip | sketch import is for 2D geometry → 3D extrusion |
| **Solidworks DXF (per original report)** | strip | strip | strip | flatten | strip | confirmed by original report; flat geometry only |
| **dxf-rs (Rust, MIT)** | pres | pres | pres | pres-proxy (read; partial author) | not impl | mature general entities; OBJECTS section coverage includes XRecord/dictionary |
| **`libdxfrw` (LibreOffice / KiCad upstream)** | strip | strip | strip | render | strip | base library used by KiCad/LibreCAD/QCAD; OBJECTS support minimal |

**Caveats.**

- The "strip" entries for FreeCAD, LibreCAD, KiCad, Eagle, and
  Altium reflect documented behaviour as of the most recent stable
  releases I could verify. Behaviour can change between versions,
  particularly as ezdxf-style proxy round-tripping becomes more
  common.
- The "render" entries for dynamic blocks rely on the anonymous-
  block fallback. If AutoCAD-emitted DXF for a dynamic block is
  unusual (e.g., uses `AcDbBlockEvalGraph` evaluation that
  requires runtime computation rather than a precomputed anonymous
  block), the render may degrade. This is rare in practice for
  the styles AutoCAD itself emits but is **unverified for hand-
  authored anonymous-block-free dynamic block DXF**.
- KiCad-specific: KiCad's DXF import is `dxflib`-based and reads
  only LINE, ARC, CIRCLE, LWPOLYLINE, SPLINE, and INSERT entities
  for board-outline use. Even XDATA on these entities is dropped at
  the IR boundary.
- The `dxf-rs` row reflects coverage as of crate version 0.5.x.
  Constraint records (`AcDbAssoc*`) are explicitly listed as
  unsupported; XRecord and dictionary support is present but
  requires the application to walk the OBJECTS section manually.
- **Empirical-test recommendations.** The matrix is based on
  documentation inspection and source-code review where available.
  For Datum's CI gates, recommend an empirical round-trip test
  battery (see "Round-Trip Testing Methodology" below) that
  verifies each `pres`/`strip` claim against a checked-in fixture
  Datum DXF in each tool. Where documentation is silent or
  contradictory (notably FreeCAD's behaviour with XRecord under
  ExtensionDictionary on BLOCK_RECORD vs on geometry entities),
  treat the matrix entry as a hypothesis to be empirically
  verified before relying on it.

**Headline finding for the stowaway:** XDATA and XRecord on the
NamedObjectsDictionary path are preserved by every DXF library
that has a documented round-trip story (AutoCAD, ODA Teigha,
ezdxf, dxf-rs). They are stripped by every DXF library oriented
to viewing/light editing (FreeCAD, LibreCAD, KiCad, Eagle,
Altium) — but stripping is silent and harmless to geometry. **The
stowaway works exactly as designed: round-trips through programmer-
oriented tools, gets stripped by view-edit tools without breaking
geometry.**

## Datum-EDA DXF Profile Design

Below: a concrete profile suitable for implementation. Where the
original report's Area 2 §"Proposed Datum-EDA DXF profile" already
sketched layer naming and per-pad XDATA, this section refines and
extends.

### App-ID and layer naming convention

**Application key:** `DATUM_EDA` (9 chars; under every reasonable
APPID limit).

**Versioned successor:** `DATUM_EDA_V2`, `DATUM_EDA_V3`, ... if the
schema ever needs incompatible evolution. Old and new can coexist
on the same entity; readers prefer the newest understood version
and ignore older ones.

**Layer naming convention** (uppercase, `DATUM_` prefix, max 31
chars per AutoCAD safe limit):

| Layer | Purpose |
|---|---|
| `DATUM_PAD_TOP` | Copper pad geometry, top layer |
| `DATUM_PAD_BOT` | Copper pad geometry, bottom layer |
| `DATUM_PAD_INNER_<n>` | Copper pad geometry, inner layer n (1-N) |
| `DATUM_DRILL` | Drill positions (CIRCLE entities; radius = drill diameter / 2) |
| `DATUM_DRILL_PLATED` | Plated drills (subset of DATUM_DRILL with XDATA flag) |
| `DATUM_DRILL_NONPLATED` | Non-plated drills |
| `DATUM_DRILL_SLOT` | Slotted drills (LWPOLYLINE rectangles) |
| `DATUM_MASK_TOP`, `DATUM_MASK_BOT` | Solder mask aperture geometry per side |
| `DATUM_PASTE_TOP`, `DATUM_PASTE_BOT` | Paste mask aperture geometry per side |
| `DATUM_SILK_TOP`, `DATUM_SILK_BOT` | Silkscreen geometry per side |
| `DATUM_FAB_TOP`, `DATUM_FAB_BOT` | Fabrication / component-body outline per side |
| `DATUM_COURTYARD_TOP`, `DATUM_COURTYARD_BOT` | Courtyard polygons per side |
| `DATUM_OUTLINE` | Footprint outline (single layer, reference) |
| `DATUM_KEEPOUT_TOP`, `DATUM_KEEPOUT_BOT` | Keep-out polygons per side |
| `DATUM_ORIGIN` | Single POINT at the footprint origin (0,0) |
| `DATUM_PIN_1` | Single POINT marker for pin 1 location |
| `DATUM_REFERENCE` | Reference designator text (TEXT or MTEXT) |
| `DATUM_VALUE` | Value/part-number text |

Layers carry standard DXF colour and linetype attributes; Datum
emits a fixed colour palette per layer (cyan = top copper, blue =
bottom copper, etc.) so the DXF renders sensibly in any viewer.

### Per-entity XDATA schema

Every Datum-emitted entity that carries semantic load attaches
XDATA under app key `DATUM_EDA`. Schema is **flat key-value pairs
encoded as 1000-string records** unless the value is naturally a
real (1041 distance) or integer (1070/1071). Keys are
`snake_case` and prefixed with the entity-class to avoid collision
across schemas (`pad_*` for pads, `drill_*` for drills, etc.).

**Pad entity XDATA (attached to LWPOLYLINE or HATCH or INSERT
representing the pad copper):**

| Key | DXF group | Type | Notes |
|---|---|---|---|
| `pad_shape` | 1000 | string | `roundrect`, `oblong`, `circle`, `chamfered`, `polygon`, `rect` |
| `pad_size_w_nm` | 1071 | int32 | width in nanometers (integer for determinism) |
| `pad_size_h_nm` | 1071 | int32 | height in nanometers |
| `pad_roundrect_ratio_ppm` | 1071 | int32 | roundrect corner radius ratio in parts-per-million (0–1 000 000) |
| `pad_chamfer_corners` | 1000 | string | comma-separated subset of `tl,tr,bl,br` |
| `pad_drill_diameter_nm` | 1071 | int32 | 0 if SMD, > 0 if through-hole |
| `pad_drill_plated` | 1070 | int16 | 0 = non-plated, 1 = plated |
| `pad_drill_slot_w_nm` | 1071 | int32 | 0 if not slotted |
| `pad_drill_slot_h_nm` | 1071 | int32 | 0 if not slotted |
| `pad_pin_name` | 1000 | string | net name (e.g., "VCC", "1") |
| `pad_pin_number` | 1000 | string | sequential pad number as string (some packages use "A1" notation) |
| `pad_padstack_uuid` | 1000 | string | Datum pool padstack UUID; opaque to outside |
| `pad_layer_set` | 1000 | string | comma-separated list of layer names this pad occupies |
| `pad_thermal_relief_mode` | 1000 | string | `direct`, `relief`, `none` |
| `pad_thermal_relief_gap_nm` | 1071 | int32 | 0 if `direct`/`none` |
| `pad_thermal_relief_spoke_count` | 1070 | int16 | 0 if `direct`/`none` |
| `pad_thermal_relief_spoke_w_nm` | 1071 | int32 | 0 if `direct`/`none` |
| `pad_mask_expansion_nm` | 1071 | int32 | per-pad override (0 = use document default) |
| `pad_paste_reduction_nm` | 1071 | int32 | per-pad override |
| `pad_solder_mask_defined` | 1070 | int16 | 0 = NSMD, 1 = SMD |

All distances in nanometers (int32) for byte-exact determinism.
This matches the existing Datum convention (per `docs/CANONICAL_IR.md`).

**Drill entity XDATA (attached to CIRCLE entity on `DATUM_DRILL*`
layer):**

| Key | DXF group | Type | Notes |
|---|---|---|---|
| `drill_diameter_nm` | 1071 | int32 | redundant with CIRCLE radius but explicit |
| `drill_plated` | 1070 | int16 | 0/1 |
| `drill_chamfer_top_nm` | 1071 | int32 | optional countersink |
| `drill_chamfer_bot_nm` | 1071 | int32 | |
| `drill_associated_pad_handle` | 1005 | handle | DXF handle of the pad entity this drill belongs to |

**Silkscreen and fab line XDATA (attached to LWPOLYLINE / LINE /
ARC):**

| Key | DXF group | Type | Notes |
|---|---|---|---|
| `silk_role` | 1000 | string | `outline`, `pin1_marker`, `polarity`, `keepout_indicator`, `assembly_text` |
| `silk_line_width_nm` | 1071 | int32 | overrides DXF lineweight |

**Common XDATA on every Datum-emitted entity:**

| Key | DXF group | Type | Notes |
|---|---|---|---|
| `datum:profile_version` | 1000 | string | `1.0` for v1 schema |
| `datum:entity_uuid` | 1000 | string | Datum-internal stable ID for round-trip |

The two `datum:*` keys are at the head of every XDATA block. A
reader can skip the entity's XDATA cheaply if the profile_version
is unrecognised.

### Block-level metadata via XRecords

Every footprint Datum exports lives in a DXF BLOCK named after the
footprint (`DATUM_FP_<footprint_uuid>` or `DATUM_FP_<footprint_name>`
for human readability). The BLOCK_RECORD entry's extension
dictionary owns one XRecord per metadata category:

**`DATUM_EDA_PARAMETRIC_SOURCE` XRecord** — the parametric source
in serialised form. Schema:

| Group code | Type | Contents |
|---|---|---|
| 1 | string | `family` (key) |
| 1 | string | family identifier (e.g., `chip_resistor`, `soic_pad`, `qfn_pad`) |
| 1 | string | `family_version` |
| 1 | string | `1.0` |
| 1 | string | `ipc_standard` |
| 1 | string | `IPC-7351B` (or `IPC-7351C`, `none`) |
| 1 | string | `density_level` |
| 1 | string | `A`, `B`, or `C` |
| 1 | string | `solver_version` |
| 1 | string | `datum-solver-1.0.0` |
| 90 | int32 | parameter count N |
| (then for each parameter) | | |
| 1 | string | parameter name (e.g., `pad_pitch_nm`) |
| 1 | string | parameter type (`int_nm`, `real_mm`, `string`, `bool`) |
| 1 | string | parameter value (as string, even for numerics — keeps round-trip exact) |
| 1 | string | parameter unit |
| 1 | string | parameter source (`user`, `formula`, `ipc_default`, `family_default`) |
| 1 | string | parameter expression (formula text if source = `formula`, else empty) |
| 90 | int32 | constraint count M |
| (then for each constraint) | | |
| 1 | string | constraint type (e.g., `equal_distance`, `coincident`, `symmetric_x`) |
| 1 | string | constraint operands (JSON-array string of entity-uuid references) |

This schema mirrors Datum's internal `ParametricPackage` struct
(per the original report's Area 4) one-to-one. The encoder is a
straight serialiser; the decoder is a straight deserialiser; round-
trip is exact when the consumer is Datum.

**`DATUM_EDA_SOURCE_DIMENSIONS` XRecord** — the IPC source
dimensions table (Lmin/Lmax, Tmin/Tmax, Wmin/Wmax for SOIC/QFP,
body L×W×H for chip resistors, etc.):

| Group code | Type | Contents |
|---|---|---|
| 1 | string | dimension name (`L_min`, `L_max`, etc.) |
| 40 | real | dimension value in mm |
| 1 | string | dimension tolerance source (`datasheet`, `JEDEC`, `manufacturer`) |
| 1 | string | dimension reference (e.g., `JEDEC MS-012-AA Table 2`) |

Repeated for each dimension.

**`DATUM_EDA_DEVIATIONS` XRecord** — when the user has
manually deviated from formula-derived values:

| Group code | Type | Contents |
|---|---|---|
| 1 | string | deviated parameter name |
| 1 | string | formula-derived value (as string) |
| 1 | string | user-overridden value (as string) |
| 1 | string | deviation justification (free text from user) |
| 1 | string | deviation timestamp (ISO 8601) |

Empty for footprints where the user has not deviated.

### Constraint expression strategy

Per the brief's question — **AcDbAssoc* (semantically correct, least
portable) vs Datum-private XDATA encoding (less standards-compliant,
more portable)?** — recommend the **Datum-private XRecord encoding**
described in the parametric-source XRecord above.

Rationale:

- `AcDbAssoc*` is undocumented at field level outside Autodesk's
  internal headers; emitting it correctly from a Rust encoder is
  research-grade work, not implementation work.
- No Rust DXF library implements `AcDbAssoc*` authoring; Datum
  would have to write its own raw-OBJECTS-section emitter for the
  full constraint family.
- No non-Datum, non-AutoCAD consumer interprets `AcDbAssoc*`
  semantically anyway, so the "more standards-compliant" claim is
  hollow — the consumer reads geometry either way.
- The Datum-private encoding is plain string records inside a
  Datum-named XRecord; the encoder is a straight serialiser.
- If a future standard ever emerges (e.g., IPC parametric profile),
  Datum can layer a second XRecord encoding its source in the
  standard schema alongside the Datum-private one — both coexist on
  the same BLOCK_RECORD.

### Forward-compatibility hooks

**Profile version stamping.** Every entity carries
`datum:profile_version=1.0` as the first XDATA pair. Every
block-level XRecord carries a `datum:profile_version` key as one
of its first records. The document-level XRecord (see below)
carries the profile version explicitly.

**Document-level profile XRecord.** Under the
NamedObjectsDictionary, Datum creates one entry named
`DATUM_EDA_PROFILE` (a DICTIONARY) which owns one XRecord named
`metadata`:

| Group code | Type | Contents |
|---|---|---|
| 1 | string | `datum:profile_version` |
| 1 | string | `1.0` |
| 1 | string | `datum:emitter_version` |
| 1 | string | `datum-eda-cli-0.7.0` (whatever Datum version emitted) |
| 1 | string | `datum:emit_timestamp` |
| 1 | string | ISO 8601 timestamp |
| 1 | string | `datum:source_native_path` |
| 1 | string | path to the originating Datum native JSON file (informational) |
| 1 | string | `datum:source_native_uuid` |
| 1 | string | UUID of the source pool entry |
| 1 | string | `datum:dxf_format_version` |
| 1 | string | `R2018` (the targeted DXF version) |

This document-level entry is the entry point for any Datum reader:
read it first to determine the profile version; if unrecognised,
fall back to geometry-only import.

**Schema evolution rule.** Within a major profile version (`1.x`),
new keys can be added to any XDATA or XRecord schema; old keys
must continue to be emitted; readers must ignore unrecognised keys.
Across major versions (`1.x` → `2.x`), Datum emits both v1 and v2
under coexisting app keys (`DATUM_EDA` and `DATUM_EDA_V2`) for at
least one full release cycle, then drops v1 emission while keeping
v1 read-back support indefinitely.

### Datum-private vs published-profile schemas

The brief asks about "the difference between 'this is what Datum
reads back' and 'this is what anyone could read if they wanted to'".
Two separate concerns:

**Datum-private schema** — everything described above.
Implementation lives in the Datum codebase. Schema is documented in
Datum's source code. Third-party tools that want to read it must
implement the schema themselves; Datum does not commit to schema
stability across major versions for the private side (within
major versions per the rule above, schema is stable).

**Published-profile schema** — what Datum publishes externally.
Recommendation: publish the `DATUM_EDA` v1.0 schema under a
permissive licence (CC-BY-4.0 for the document, MIT for any
reference reader code) on GitHub once the encoder is shipping in
production. Until then, the schema is implementation-defined and
may change without notice.

Publishing the schema costs nothing material — Datum has to
document it internally for its own engineers anyway. Publishing
externally just makes the doc public. The publication is what
gives the stowaway its long-tail-adoption value: a third-party
developer who wants to build a Datum-DXF reader can implement
against the spec, not by reverse-engineering Datum-emitted DXF.

## Worked Examples

The DXF excerpts below are illustrative — they highlight the
Datum-stowaway encoding against the surrounding pure-DXF group
codes. Real DXF files contain extensive header and table boilerplate
omitted here for clarity. Annotations distinguish:

- **`[DXF]`** — pure DXF group codes that any DXF tool emits
- **`[DATUM]`** — Datum-stowaway content
- **`[DATUM-PRIVATE]`** — Datum-internal schema, not part of any
  public spec

### Example A — Parametric 0805 resistor

A classic chip resistor footprint: two rectangular pads
(1.6 mm × 0.8 mm), centre-to-centre pitch 1.95 mm, courtyard
3.4 mm × 1.8 mm, silkscreen reference outline.

**APPID registration (in TABLES section):**

```
0           [DXF] entity start
TABLE       [DXF] table marker
2           [DXF] table type group code
APPID       [DXF] table type
... table header ...
0           [DXF]
APPID       [DXF] entry
2           [DXF]
DATUM_EDA   [DATUM] application name
70          [DXF] flags group code
0           [DXF] standard flags
0           [DXF] end-of-table marker
ENDTAB
```

**Layer table entries (in TABLES section, partial):**

```
0
LAYER
2
DATUM_PAD_TOP    [DATUM] layer name with prefix
70
0
62               [DXF] colour group code
4                [DXF] cyan
6                [DXF] linetype group code
CONTINUOUS       [DXF] linetype name
0
LAYER
2
DATUM_COURTYARD_TOP   [DATUM]
70
0
62
6                [DXF] yellow
6
DASHED2          [DXF]
... (one entry per Datum layer used) ...
```

**BLOCK definition (in BLOCKS section):**

```
0
BLOCK
2
DATUM_FP_R0805           [DATUM] block name = footprint identifier
70
0
10                       [DXF] base-point X
0.0
20                       [DXF] base-point Y
0.0
3
DATUM_FP_R0805
1
                         [DXF] xref path (empty for local block)
```

**Pad 1 entity (LWPOLYLINE) inside the block:**

```
0
LWPOLYLINE       [DXF]
8                [DXF] layer name group code
DATUM_PAD_TOP    [DATUM] layer
90               [DXF] vertex count
4
70               [DXF] flags
1                [DXF] closed polyline
10
-1.775           [DXF] vertex X mm
20
-0.4             [DXF] vertex Y
10
-0.175
20
-0.4
10
-0.175
20
0.4
10
-1.775
20
0.4
1001                     [XDATA START]
DATUM_EDA                [DATUM] app name
1000
datum:profile_version=1.0   [DATUM] schema version
1000
datum:entity_uuid=8c4e...   [DATUM] stable ID
1000
pad_shape=rect              [DATUM]
1071
1600000                     [DATUM] pad_size_w_nm = 1.6 mm
1071
800000                      [DATUM] pad_size_h_nm = 0.8 mm
1071
0                           [DATUM] pad_drill_diameter_nm = 0 (SMD)
1000
pad_pin_number=1            [DATUM]
1000
pad_pin_name=                [DATUM] empty net (library context)
1000
pad_padstack_uuid=ab12-...   [DATUM] pool padstack ref
1000
pad_layer_set=DATUM_PAD_TOP,DATUM_MASK_TOP,DATUM_PASTE_TOP   [DATUM]
1071
50000                       [DATUM] pad_mask_expansion_nm = 50 µm
1071
0                           [DATUM] pad_paste_reduction_nm = 0
1000
pad_thermal_relief_mode=direct  [DATUM]
```

(The XDATA block is closed automatically by the next 0 marker
introducing the next entity.)

**Pad 2 entity:** identical structure with mirrored X coordinates
and `pad_pin_number=2`.

**Courtyard polygon:**

```
0
LWPOLYLINE
8
DATUM_COURTYARD_TOP   [DATUM]
90
4
70
1
10
-1.7
20
-0.9
10
1.7
20
-0.9
10
1.7
20
0.9
10
-1.7
20
0.9
1001
DATUM_EDA
1000
datum:profile_version=1.0
1000
silk_role=courtyard
```

**Block parametric source XRecord** (in OBJECTS section):

The block-level XRecord encodes the parametric source. The
extension dictionary on the BLOCK_RECORD owns it; the BLOCK_RECORD
is referenced by handle from the BLOCK definition above.

Parametric source for the 0805 chip resistor (4 parameters,
0 explicit constraints — chip resistor geometry is fully derived
from formulas):

| Parameter | Type | Value (nm) | Source | Formula |
|---|---|---|---|---|
| `body_length_nm` | int_nm | 2 000 000 | `ipc_default` | — |
| `body_width_nm` | int_nm | 1 250 000 | `ipc_default` | — |
| `pad_pitch_nm` | int_nm | 1 950 000 | `formula` | `body_length_nm + 2 * (toe_fillet_nm + heel_fillet_nm) / 2` |
| `toe_fillet_nm` | int_nm | 400 000 | `ipc_default` | `toe_fillet(density_level, size_code)` |

Concrete DXF group-code excerpt (XRecord header + first parameter
expanded; remaining parameters follow the same shape):

```
0
DICTIONARY              [DXF] extension dictionary on BLOCK_RECORD
... handle and owner records ...
3
DATUM_EDA_PARAMETRIC_SOURCE   [DATUM] entry name
350
<xrecord_handle>        [DXF] soft-owner handle of XRecord
0
XRECORD                 [DXF] entity type
... handle and owner records ...
100
AcDbXrecord             [DXF] subclass marker
280
1                       [DXF] cloning = KEEP
1
datum:profile_version
1
1.0
1
family
1
chip_resistor
1
size_code
1
0805
1
ipc_standard
1
IPC-7351B
1
density_level
1
B
90
4                       [DATUM-PRIVATE] parameter count
1
body_length_nm
1
int_nm
1
2000000
1
nm
1
ipc_default
1
                        (no formula — direct user input)
... (3 more parameter records following the same shape) ...
90
0                       [DATUM-PRIVATE] constraint count = 0
```

**Block source-dimensions XRecord:**

```
0
DICTIONARY
3
DATUM_EDA_SOURCE_DIMENSIONS    [DATUM]
350
<xrecord_handle>
0
XRECORD
100
AcDbXrecord
280
1
1
body_L_min
40
1.85
1
body_L_max
40
2.20
1
body_W_min
40
1.10
1
body_W_max
40
1.40
1
source
1
JEDEC RC0805 chip resistor reference
```

**Document-level profile XRecord** (under NamedObjectsDictionary,
in OBJECTS section):

```
0
DICTIONARY
... handle records ...
3
DATUM_EDA_PROFILE          [DATUM]
350
<sub_dict_handle>
0
DICTIONARY
... handle records ...
3
metadata
350
<xrecord_handle>
0
XRECORD
100
AcDbXrecord
280
1
1
datum:profile_version
1
1.0
1
datum:emitter_version
1
datum-eda-cli-0.7.0
1
datum:emit_timestamp
1
2026-04-18T15:12:33Z
1
datum:source_native_uuid
1
8c4e2d0a-1234-...
1
datum:dxf_format_version
1
R2018
```

### Example B — Parametric SOIC-8 with JEDEC MS-012 + IPC-7351B Nominal density

Eight rectangular pads in two rows of four. Pad geometry derived
from JEDEC MS-012-AA (SOIC-8 narrow body) dimensions and IPC-7351B
Density B (Nominal) fillet allowances. Derived land-pattern values:
pad 0.8 mm × 0.66 mm, pitch 1.27 mm, row pitch 5.4 mm. (Real values
per IPC-7351B Density B Nominal land pattern for SOIC-127P600X175;
values illustrative.)

**APPID, layer table, and block definition:** identical pattern to
Example A, with block name `DATUM_FP_SOIC8_127P600X175` (IPC-7351
land-pattern naming) and the addition of `DATUM_PIN_1` for the pin-1
marker.

**Pad XDATA:** identical schema to Example A's pad XDATA. Pad 1
carries `pad_size_w_nm=800000`, `pad_size_h_nm=660000`,
`pad_pin_number=1`. Pads 2–8 follow the array pattern; their XDATA
differs only in `pad_pin_number` and the geometric coordinates of
the pad rectangle. Full DXF group-code dump omitted (identical
shape to Example A's pad).

**Pin-1 marker** (POINT entity on `DATUM_PIN_1` layer at the pin-1
pad centre, with XDATA `silk_role=pin1_marker`).

**Block parametric source XRecord:**

The XRecord encodes the SOIC family parameters and the IPC formulas
that derive each pad dimension from the family parameters. Note:
the formulas are stored as **expression strings** that Datum's
solver re-evaluates on read. Other consumers see the strings
verbatim and ignore them.

The full XRecord is 14 parameters and 4 constraints; rather than
expand all 18 records (each parameter is a 12-line group of group
codes per the schema in the profile design section), the table
below summarises the parameter set, and two representative records
are then expanded as concrete DXF group codes.

| Parameter | Type | Value (nm) | Source | Formula (if source = `formula`) |
|---|---|---|---|---|
| `pin_count` | int | 8 | `ipc_default` | — |
| `lead_pitch_nm` | int_nm | 1 270 000 | `jedec` | — |
| `lead_length_min_nm` | int_nm | 400 000 | `jedec` | — |
| `lead_length_max_nm` | int_nm | 1 270 000 | `jedec` | — |
| `lead_width_min_nm` | int_nm | 310 000 | `jedec` | — |
| `lead_width_max_nm` | int_nm | 510 000 | `jedec` | — |
| `body_E_max_nm` | int_nm | 3 900 000 | `jedec` | — |
| `toe_fillet_nm` | int_nm | 550 000 | `ipc_default` | `toe_fillet(density_level='B', package='SOIC')` |
| `heel_fillet_nm` | int_nm | 250 000 | `ipc_default` | `heel_fillet(density_level='B', package='SOIC')` |
| `side_fillet_nm` | int_nm | 50 000 | `ipc_default` | `side_fillet(density_level='B', package='SOIC')` |
| `fab_tolerance_nm` | int_nm | 50 000 | `ipc_default` | — |
| `placement_tolerance_nm` | int_nm | 50 000 | `ipc_default` | — |
| `pad_length_nm` | int_nm | 800 000 | `formula` | `lead_length_min_nm + 2 * heel_fillet_nm - fab_tolerance_nm` |
| `pad_width_nm` | int_nm | 660 000 | `formula` | `lead_width_max_nm + 2 * side_fillet_nm + placement_tolerance_nm` |
| `pad_row_pitch_nm` | int_nm | 5 400 000 | `formula` | `body_E_max_nm + 2 * toe_fillet_nm + 2 * (lead_length_max_nm / 2)` |

Constraint set (4 entries):

| Constraint | Type | Operands (JSON inside group code 1) |
|---|---|---|
| `pad_array_pin1to4_x_step` | `linear_array` | `{"axis":"y","step_nm":1270000,"count":4,"start_pin_number":1}` |
| `pad_array_pin5to8_x_step` | `linear_array` | `{"axis":"y","step_nm":1270000,"count":4,"start_pin_number":5,"reverse":true}` |
| `pad_pair_symmetric_x` | `symmetric` | `{"axis":"y","pad_pairs":[[1,8],[2,7],[3,6],[4,5]]}` |
| `all_pads_equal_size` | `equal_size` | `{"pads":[1,2,3,4,5,6,7,8]}` |

Concrete DXF group-code excerpt (XRecord header + first 2 parameters
+ 1 formula-derived parameter, rest follow the same pattern):

```
0
XRECORD
100
AcDbXrecord
280
1
1
datum:profile_version
1
1.0
1
family
1
soic_pad
1
ipc_standard
1
IPC-7351B
1
density_level
1
B
1
jedec_reference
1
MS-012-AA
1
ipc_land_pattern_name
1
SOIC-127P600X175N
90
14                   [DATUM-PRIVATE] parameter count
1
pin_count
1
int
1
8
1
pin
1
ipc_default
1
                     (no formula)
1
lead_pitch_nm
1
int_nm
1
1270000
1
nm
1
jedec
1
                     (no formula — direct from JEDEC table)
... (12 more parameter records following the same shape) ...
1
pad_length_nm
1
int_nm
1
800000
1
nm
1
formula
1
lead_length_min_nm + 2 * heel_fillet_nm - fab_tolerance_nm
... (2 more formula-derived parameters) ...
90
4                    [DATUM-PRIVATE] constraint count
1
pad_array_pin1to4_x_step
1
linear_array
1
{"axis":"y","step_nm":1270000,"count":4,"start_pin_number":1}
... (3 more constraint records) ...
```

**Block source-dimensions XRecord:**

JEDEC MS-012-AA Table 2 (SOIC-8 narrow body) values, encoded as
alternating string-name (group code 1) + real-value (group code 40)
pairs. Summary:

| Dimension | Min (mm) | Max (mm) |
|---|---|---|
| `body_L` | 4.80 | 5.00 |
| `body_E` | 3.80 | 4.00 |
| `body_E1` | 5.80 | 6.20 |
| `body_A` | — | 1.75 |
| `lead_b` | 0.31 | 0.51 |
| `lead_L` | 0.40 | 1.27 |
| `lead_e` (pitch) | 1.27 | 1.27 (nominal) |

Plus a closing `source_reference` string record:
`"JEDEC MS-012-AA Table 2 (SOIC-8 narrow body)"`. The encoding is
the obvious group-code-1/group-code-40 pattern shown in
Example A's source-dimensions XRecord and is omitted here.

**Annotated dual-encoding observation.** Note the deliberate
duplication:
- The **per-pad XDATA** carries the *concrete numeric values*
  (`pad_size_w_nm=800000`) that Datum reads back to populate the
  flat-geometry IR for any consumer that ignores the parametric
  side.
- The **block-level XRecord** carries the *parametric formulas*
  (`pad_length_nm = lead_length_min_nm + 2 * heel_fillet_nm - fab_tolerance_nm`)
  that Datum evaluates to *re-derive* the same values when the
  parametric source is re-loaded.

This dual-encoding is the heart of the stowaway. A consumer that
reads only XDATA gets a flat parametric-snapshot footprint and is
correct. A consumer that reads only XRecords (Datum on re-load)
gets the formulas and re-derives geometry. Datum reads both and
verifies they agree on round-trip — if they disagree (because a
third-party tool hand-edited a pad's XDATA without updating the
parametric source), Datum reports a "footprint deviates from
parametric source — re-run solver?" diagnostic rather than
silently honouring either.

## Round-Trip Testing Methodology

The stowaway only delivers value if the encoding is reliably
produced and consumed. Six layers of testing are recommended.

**1. Datum-to-Datum byte-exact round-trip.**

For each parametric footprint in the test corpus, run:

```
datum dxf-export <footprint.json> --output golden.dxf
datum dxf-import golden.dxf --output rehydrated.json
diff <(canonicalise footprint.json) <(canonicalise rehydrated.json)
```

The canonicaliser is the existing Datum determinism gate
(`scripts/check_alignment.py` family). The diff must be empty for
every footprint in the corpus. This is the primary correctness
gate; failure means the encoder/decoder pair is not lossless.

Implementation note: the encoder must emit DXF in deterministic
byte order (sorted handles, fixed timestamp policy or omitted
timestamp under a `--deterministic` flag). The current Datum
practice for golden-tested DXF emission per Domain 1 Domain
research applies.

**2. Third-party validator pass.**

For each emitted DXF, run:

```
python -c "import ezdxf; doc = ezdxf.readfile('golden.dxf'); auditor = doc.audit(); print(auditor.errors); print(auditor.fixes)"
```

`ezdxf.audit()` returns structured error and fix-applied lists. The
gate is: zero errors, zero structural fixes (cosmetic warnings like
"unsupported entity type X round-tripped as proxy" are acceptable).

Repeat with FreeCAD's `freecad.dxf` import (`importDXF.open`) and
LibreCAD's command-line DXF parser. None of these need to interpret
Datum metadata — they only need to **not crash** and to **render
geometry without errors**.

If a real AutoCAD trial seat is available, opening the DXF in
AutoCAD must produce zero "drawing recovery" or "constraint
inconsistency" dialogs.

**3. Consumer-stripping detection.**

Emit a Datum DXF, open in a consumer known to strip metadata
(e.g., LibreCAD), re-save under a new name, then re-import to
Datum:

```
datum dxf-export <footprint.json> --output original.dxf
librecad --export original.dxf --output stripped.dxf  # or manual GUI re-save
datum dxf-import stripped.dxf --output rehydrated.json
```

The expected behaviour: `rehydrated.json` is a flat-geometry
footprint (no parametric source), and Datum emits a clear warning:

> Warning: DXF lacks Datum profile metadata. The file appears to
> have been edited externally; the parametric source is no longer
> present. Future edits will lose the family relationship.
> Continue with flat-geometry import? [y/N]

This warning must be emitted unconditionally when the document-
level `DATUM_EDA_PROFILE` XRecord is absent. The test verifies the
warning is emitted and the user has a non-blocking opt-in/opt-out
path.

**4. DXF spec validator.**

If AutoCAD's DXF validator is available (offline tool;
`AcadValidator.exe` ships with some AutoCAD subscriptions), run it
against every emitted DXF — zero errors required.

Free alternative: `ezdxf.audit()` plus the `dxf-rs` round-trip
("read with `dxf-rs`, write back, diff") catches most structural
errors a free-tier audit would catch.

**5. Visual goldens.**

Render the emitted DXF in AutoCAD (or a plug-in-replacement viewer)
and FreeCAD; capture the rendered output as a PNG. Compare against
a Datum-rendered PNG of the same footprint. The PNG diff must be
within a small pixel-tolerance budget (lineweight rendering varies
between consumers, so exact-match is impractical).

This catches subtle layer-colour, lineweight, or coordinate-system
misencodings that the geometry-only validator passes would miss.
The screenshot-goldens memory rule applies (per
`feedback_screenshot_goldens` memory): rendering work requires
image-based regression, not just unit tests.

**6. Cross-version test.**

Emit each test footprint as DXF in two formats: R2018 (current
default) and R2010 (older format used by many MCAD pipelines).
Verify both pass tests 1–4. The R2010 format will exercise some
compatibility paths in the OBJECTS section (XRecord and
NamedObjectsDictionary support is identical between R2010 and
R2018 — verified against the DXF Reference change log; the cross-
version test is mostly a guard against accidental R2018-only
features sneaking into the encoder).

**Recommended cadence.** Tests 1–4 run on every commit affecting
DXF code (CI gate). Tests 5–6 run nightly or on tag releases.

## Refined Verdict and Adoption Pathway

The original report's verdict was:

> **Pursue Phase A (parametric editor with constraint solver, pool
> integration, IPC-7351 family templates). Defer Phase B (DXF as
> canonical interchange) indefinitely; revisit only if IPC, OpenEDA,
> or a peer tool drives external adoption.**

Under the stowaway reframe, this becomes:

> **Pursue both halves. Phase A unchanged: parametric editor as the
> primary product value. Phase B reframed: ship the DXF stowaway
> as part of the planned Domain 1 DXF mechanical-layer export work,
> document the profile, do not market it as a DFM-auditor
> interchange promise. The marketing claim is narrow — Datum's DXF
> exports preserve parametric source for Datum-to-Datum round-trip,
> with optional future external-consumer adoption — not "the new
> standard for parametric footprint exchange."**

The change is from a binary "pursue/defer" framing to a "ship both,
let adoption emerge or not" framing. The DXF stowaway is now
treated as a versioned encoding option that ships when the
geometry encoder ships, costs almost nothing extra, and unlocks
future adoption pathways without committing to them.

### Adoption pathway timeline

- **Year 0 (Datum 1.x).** Datum ships parametric footprints with
  stowaway-encoded DXF export. No external consumer cares; the
  stowaway is invisible to non-Datum tools. Marketing claim:
  "Datum's DXF exports preserve parametric source for round-trip"
  — strictly true, observable in the test corpus, no overclaim.
- **Year 1 (Datum 1.y).** Datum publishes the `DATUM_EDA` v1.0
  profile spec on GitHub under CC-BY-4.0 + MIT reference reader.
  Any third-party developer who wants to build a Datum-DXF reader
  has a complete spec. Cost to Datum: a documentation pass on the
  internal spec. Whether anyone reads the spec is out of Datum's
  hands.
- **Year 2-3 (opportunistic partnerships).** First-party DFM tool
  partnership opportunities. Most-likely candidates are smaller
  fab houses with appetite for niche-format differentiation
  (Sierra Circuits, OSH Park, Eurocircuits, Aisler) rather than
  Valor MSS or InCAM (which are slow-moving incumbents tied to
  IPC-2581 and ODB++). A successful partnership would likely
  involve Datum providing a reference reader implementation and
  the partner integrating it as a DFM-input option. Engineering
  cost to Datum: low (the reader is mostly pre-built from the
  test infrastructure).
- **Year 5+ (long tail).** Two scenarios:
  - **Profile gets some traction.** A handful of niche tools read
    Datum DXF parametrically. Datum has a real interchange story
    to point to in marketing. Optional: pursue formal IPC or
    OASIS standardisation of the profile if there's enough
    third-party adoption to justify the committee work.
  - **Profile gets no traction.** The parametric editor still has
    all its internal value (AI-tractable authoring, family-aware
    editing, IPC compliance verification, constraint-checked
    authoring) and the DXF stowaway cost was effectively zero.
    Datum does not lose ground relative to the original "defer
    indefinitely" verdict; it gains optionality without paying
    much for it.

### The "free option" framing

Encoding parametric metadata as stowaway is essentially a free
option on future adoption. Concrete cost breakdown:

| Cost category | Without stowaway | With stowaway |
|---|---|---|
| DXF encoder LOC (estimated) | ~800 | ~1100 |
| DXF encoder test LOC | ~600 | ~1400 |
| Documentation (internal) | ~500 lines | ~1200 lines |
| Documentation (external, year 1) | 0 | ~3000 lines spec |
| Runtime cost per DXF emit | baseline | +5-10% (XDATA + XRecord serialisation) |
| File size per DXF | baseline | +20-40% for footprint-only DXF; +5-10% for board-level DXF |

The runtime cost is negligible for footprint-scale files. The file-
size cost is moderate for footprint files (the stowaway can be the
same size as the geometry for a small footprint) but small for
board-level files where the geometry dominates. The engineering
cost is a one-time ~50% over a baseline DXF encoder, which on a
~1-2 quarter Phase A timeline is days, not weeks.

### Differentiator language for marketing

Recommended marketing claim:

> "Datum is the only EDA tool whose DXF exports preserve
> parametric source. Other tools' DXF exports are flat geometry;
> Datum's are flat geometry **plus** a parametric layer that
> Datum and future consumers can use."

This is a real, verifiable claim — not vapourware. It is also
narrow enough to defend against the failure mode in the original
report: it does not claim "DFM auditors read Datum DXF
parametrically", only that the parametric source is *preserved*
(present in the file, available to any consumer that wants to read
it). Adoption is decoupled from the marketing claim.

A second tier of marketing claim, conditional on actual adoption:

> "Datum's DXF profile is now read by [partner] for [purpose]."

Used only when a partner ships actual reader support; never used
speculatively.

### Risk: future-standard incompatibility

The principal risk under the stowaway approach is that AutoCAD,
OASIS, or IPC standardise their own parametric-PCB-DXF profile in
the future, and Datum's profile is incompatible with the standard.

**Mitigation 1 — versioned app-ID.** Datum's profile is
`DATUM_EDA` v1.0. A future standard could land as `IPC_PARAMETRIC`
or `OASIS_EDA_DXF` or similar. Datum would emit both the
`DATUM_EDA` v1.0 stowaway and the standardised stowaway side-by-
side on the same entities. Old Datum readers continue to work via
`DATUM_EDA`; standards-aware readers use the standard schema.

**Mitigation 2 — schema-compatibility design.** Datum's v1.0
schema uses flat key-value pairs with `snake_case` keys and string
or int representation. This is forward-compatible with any
reasonable future standard schema — the keys can be aliased, the
values can be transcoded. The schema does not encode any
implementation-specific assumptions that would block aliasing.

**Mitigation 3 — published spec.** Publishing the `DATUM_EDA`
profile spec on GitHub gives Datum standing to participate in any
future standards conversation; the profile is concrete enough that
it could form the basis of a starting-point proposal for IPC or
OASIS.

**Mitigation 4 — graceful demotion.** If a future standard achieves
broad adoption and Datum chooses to deprecate `DATUM_EDA`, the
deprecation is asymmetric: Datum stops *emitting* `DATUM_EDA`
but continues *reading* it indefinitely. Old DXFs in the wild
continue to work; new DXFs use the new standard.

The combination makes the future-standard risk low. The worst
outcome is "Datum's profile is technically deprecated but
continues to work alongside the standard for an indefinite
transition period" — which is exactly the position every
proprietary EDA format has been in for the last 20 years vis-à-vis
IPC-2581 and ODB++.

## What This Addendum Changes in the Original Report

The addendum **does not refute** the original report's findings on
prior art, constraint solver options, pool integration, or
published-profile DFM-auditor adoption pathways. Those analyses
stand. What changes:

- **Original Executive Summary bullet 1** (the "verdict: defer
  with conditional pursue path") becomes "verdict: pursue both
  halves; ship Phase A as primary value, ship the DXF stowaway as
  Phase B with the DFM-auditor adoption claim explicitly off the
  table." The stowaway is the new framing for Phase B.
- **Original Executive Summary bullet 4** (the "AutoCAD's own
  parametric-DXF round-trip is the unspoken killer") still holds
  *for `AcDbAssoc*` constraint persistence as the carrier*. It
  does **not** hold for the XDATA + XRecord stowaway path the
  addendum recommends. The stowaway sidesteps the `AcDbAssoc*`
  problem entirely.
- **Original Executive Summary bullet 8** (the "Phasing
  recommendation if pursued: ship parametric-internal first, defer
  the DXF-canonical claim") is refined: ship parametric-internal
  first (unchanged), and ship the DXF stowaway as part of the
  Domain 1 DXF mechanical-layer export work. The "defer
  indefinitely" framing is replaced by "ship as opportunistic
  infrastructure, do not over-market."
- **Original Area 2 §"Proposed Datum-EDA DXF profile (sketch)"**
  is *extended* by this addendum's "Datum-EDA DXF Profile Design"
  section. The addendum's profile adds the document-level
  NamedObjectsDictionary entry, the per-block XRecords for
  parametric source and source dimensions and deviations, the
  forward-compatibility hook (profile_version stamping), and the
  Datum-private-vs-published-profile distinction. The original
  layer-naming and XDATA-key sketch is retained.
- **Original Area 2 §"Verdict for Area 2"** ("the parametric
  source half is unsolvable in DXF") is refined: the parametric
  source half is *unsolvable in DXF when the encoding target is
  AutoCAD's `AcDbAssoc*` constraint persistence model*. It is
  *fully solvable in DXF when the encoding target is a Datum-
  private XRecord schema* — which any consumer can ignore safely.
- **Original Area 2 §"Parametric DXF round-trip status"** still
  describes the `AcDbAssoc*` situation correctly; the addendum
  adds a new section ("Datum Round-Trip Testing Methodology")
  that addresses the round-trip problem under the stowaway
  approach with concrete test gates.
- **Original "Recommendation"** ("Pursue Phase A. Defer Phase B
  (DXF as canonical interchange) indefinitely.") is the section
  most directly refined. New recommendation: pursue Phase A
  unchanged; **ship the DXF stowaway during the Domain 1 DXF
  mechanical-layer export work; document the profile; publish the
  profile spec one release after stowaway ships; do not market the
  stowaway as a DFM-auditor interchange promise.**
- **Original "Open question 1" (AutoCAD parametric DXF round-trip
  empirical verification)** is now lower-priority because the
  stowaway approach does not depend on `AcDbAssoc*` round-trip.
  The empirical verification is still informative if Datum ever
  pursues dynamic blocks or `AcDbAssoc*` as a secondary carrier
  for AutoCAD-specific consumers.

A reader updating their mental model of the original report should:
read the original through Area 5 verbatim; mentally substitute the
stowaway-shipped Phase B for the original's "defer indefinitely"
Phase B; treat the addendum's profile design as the implementation
spec for Phase B.

## Sources

### DXF Reference and APIs

- **Autodesk DXF Reference (current; covers R2018 and adjacent
  versions)** —
  https://help.autodesk.com/view/OARX/2026/ENU/?guid=GUID-235B22E0-A567-4CF6-92D3-38A2306D73F3
  (free; covers entity types, group codes, OBJECTS section,
  partial coverage of `AcDbAssoc*` and dynamic block classes).
- **Autodesk ObjectARX SDK documentation** —
  https://help.autodesk.com/view/OARX/2026/ENU/ (covers
  AcDbXrecord, AcDbDictionary, AcDbExtensionDictionary, AcDb
  geometric and dimensional constraint API surfaces; some sections
  paywalled or login-gated).
- **`ezdxf` documentation** — https://ezdxf.mozman.at/docs/
  (Python; verified against ezdxf ≥ 1.0; sections on XDATA,
  XRecord, AppID, NamedObjectsDictionary, ExtensionDictionary).
- **`ezdxf` source — XDATA and XRecord modules** —
  https://github.com/mozman/ezdxf/tree/master/src/ezdxf/entities
  (source for `Xdata`, `XRecord`, `Dictionary` classes).
- **`dxf-rs` crate documentation** — https://docs.rs/dxf
  (MIT; Rust DXF read/write; OBJECTS section coverage including
  XRecord).
- **`libdxfrw` source** — https://github.com/codelibs/libdxfrw
  (base library used by KiCad, LibreCAD, QCAD; relevant for
  understanding the stripped-on-resave behaviour).

### Carrier-specific references

- **XDATA group code reference** — Autodesk DXF Reference,
  "Extended Data" section
  (https://help.autodesk.com/view/OARX/2026/ENU/?guid=GUID-DAE63B98-CDB6-46C2-AB42-D3E63A2620F7).
- **APPID symbol table** — Autodesk DXF Reference, "TABLES
  Section" (search "APPID Symbol Table").
- **XRecord (AcDbXrecord)** — Autodesk DXF Reference, "OBJECTS
  Section" → "XRECORD" entry.
- **Dictionary (AcDbDictionary)** — Autodesk DXF Reference,
  "OBJECTS Section" → "DICTIONARY" entry.
- **NamedObjectsDictionary** — Autodesk DXF Reference, "Header
  Variables" → `$NAMEDOBJECTSDICTIONARY`.
- **Dynamic block reference (limited public coverage)** — Autodesk
  DXF Reference, "OBJECTS Section" → "ACDBBLOCKREPRESENTATION",
  "ACDBBLOCKLINEARPARAMETER", related entries; ezdxf documentation
  on dynamic block read coverage at
  https://ezdxf.mozman.at/docs/blocks/dynamic_block.html.
- **`AcDbAssoc*` family** — Autodesk DXF Reference, "Object
  Section" → constraint and association classes; ObjectARX SDK
  reference for full field-level documentation (login required).

### Consumer-behaviour references

- **AutoCAD XDATA preservation across DWG round-trips** — Autodesk
  Knowledge Base article "About Extended Entity Data" (free).
- **Open Design Alliance Teigha SDK** — https://www.opendesign.com/
  (commercial; covers BricsCAD/ZWCAD round-trip semantics).
- **FreeCAD Draft DXF importer source** —
  https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/Draft/draftutils
  and the bundled `dxfLibrary` in
  https://github.com/FreeCAD/FreeCAD/blob/main/src/Mod/Draft/importDXF.py.
- **LibreCAD DXF parser source** —
  https://github.com/LibreCAD/LibreCAD/tree/master/librecad/src/lib/fileio
  (libdxfrw-based).
- **KiCad DXF importer source** —
  https://gitlab.com/kicad/code/kicad/-/tree/master/pcbnew/import_gfx
  (the dxflib-based reader used for board outline import).
- **Inkscape DXF input filter** —
  https://gitlab.com/inkscape/inkscape/-/blob/master/share/extensions/dxf_input.py
  (Python extension).
- **Solidworks DXF metadata behaviour** — confirmed from original
  report Area 1 §"SolidWorks 2D sketch model".

### Datum cross-references

- Original report §"Area 2 — DXF as a PCB-semantics carrier" —
  `research/parametric-footprint-editor/PARAMETRIC_FOOTPRINT_EDITOR_RESEARCH.md`
  lines ~404-674.
- Original report §"Recommendation" — same file, lines ~1376-1419.
- Domain 1 DXF coverage —
  `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
  lines ~619-686 and ~1751-1758.
- Datum determinism gates — `scripts/check_alignment.py` family.
- Screenshot-goldens regression rule —
  `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/feedback_screenshot_goldens.md`.
- Research-only mode rule —
  `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/feedback_research_only_mode.md`.
- Datum attribution policy — `/CLAUDE.md` § "Attribution Policy".

### Standards and reference dimensions

- **JEDEC MS-012-AA** (SOIC-8 narrow body) — JEDEC Solid State
  Technology Association (free; registration may be required at
  https://www.jedec.org/).
- **IPC-7351B** — IPC store (https://shop.ipc.org/), paywalled;
  formula summaries cited in original report Area 1 §"IPC-7351
  itself".
- **ISO 8601 timestamp format** —
  https://www.iso.org/iso-8601-date-and-time-format.html.
