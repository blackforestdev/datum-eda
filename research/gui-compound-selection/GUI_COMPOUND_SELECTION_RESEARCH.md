# GUI Compound Selection And Batch-Attribute Research

Status: integrated working research for the S5 specification design phase

## Purpose and traceability

This report preserves the research used to design S5 compound selection and its
Inspector edit surface. Its derived integration bridge is
`docs/gui/DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md`, which feeds
`docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2. Research conclusions
do not authorize implementation by themselves; the governed specification,
the pending selection-identity decision record, and final owner review control
S5.

This study intentionally treats do-not-populate (DNP) as one example of the
larger compound-attribute problem. It is not the organizing feature or the
complete inventory.

## Questions studied

1. How do mature EDA/CAD tools present attributes for heterogeneous selections?
2. Which attributes are safe common Inspector edits, and which require a
   dedicated domain tool?
3. How should mixed values, incompatible members, locks, and partial failure be
   represented?
4. Which desired surfaces already have honest Datum model/operation authority?
5. What substrate must exist before persistent groups and cross-domain locks can
   be claimed?

## External precedent

### Common properties and mixed values

KiCad's PCB and schematic Properties Managers expose properties shared by all
selected object types; changes apply to each selected item. Altium's Properties
and List panels support multi-edit and expose a simultaneous edit only where the
attribute is editable for the declared targets. AutoCAD exposes common
properties, shows divergent values as `*VARIES*`, and filters the property view
by object type without changing the underlying selection.

Datum conclusion:

- use `All N` plus explicit per-type Inspector scopes without changing canvas
  membership;
- show divergent values as `Mixed`, never blank or an arbitrary member value;
- match fields by typed semantic identity, units, value domain, and mutation
  verb—not display label; and
- preflight the complete declared scope and commit atomically. Datum does not
  copy undocumented or silent partial-application behavior.

### Dedicated bulk tools

KiCad separates track/via bulk editing, zone management, text/graphics bulk
editing, symbol fields, and variants from the generic Properties Manager.
Altium similarly uses List/Smart Edit, variant management, rules, and specialized
component/library surfaces. These products demonstrate that a large selection
does not make every shared-looking field a safe generic property.

Datum conclusion: connectivity, rule resolution, manufacturing intent,
annotation uniqueness, library authority, hierarchy, or generated geometry are
signals for a dedicated typed tool rather than a generic compound field.

### Groups and transforms

KiCad groups can contain heterogeneous and nested editor objects and preserve
relative placement during transforms. AutoCAD groups and SolidWorks blocks also
act as persistent compound objects, but their overlap, auto-expansion, scaling,
and relation-breaking behaviors are not automatically appropriate for EDA.

Datum conclusion:

- ordinary multi-selection remains ephemeral consumer/session state;
- `Group` is reserved for a persistent authored object created explicitly;
- persistent groups need stable identity, revisioned membership, name, lock,
  reference/pivot, document ownership, nesting/overlap policy, diagnostics, and
  typed create/set/delete operations; and
- a group never replaces electrical connectivity, component-instance, library,
  hierarchy, variant, or manufacturing relationships.

## Datum substrate audit

### Capabilities already represented

- Board packages carry part/package binding, reference, value, position,
  rotation, side/layer, and lock, with several field-specific operations.
- Board tracks, vias, zones, dimensions, pads, and text have whole-object Set
  operations.
- Placed schematic symbols, labels, ports, buses, text, and drawings have
  whole-object Set operations; several topology objects remain create/delete
  only.
- Variant state currently models fitted components, not a complete independent
  DNP/exclusion/alternate-part vocabulary.
- Selection itself remains consumer state and must never be journaled.

### Gaps exposed by the desired UX

- No persistent authored Group model or Create/Set/DeleteGroup operation exists.
- Lock is not a universal cross-domain property; many schematic and PCB object
  families do not carry it.
- Whole-object setters are not safe substitutes for typed field-level batch
  patches unless fresh-state resolution preserves every untouched field and the
  complete guarded batch is atomic.
- PCB projected graphics are not uniformly backed by authored engine identities.
- Schematic wires, junctions, no-connects, and bus entries lack the narrow
  topology-preserving mutation surface a generic Inspector would imply.
- Independent DNP, Exclude BOM, Exclude Board, Exclude Simulation, alternate
  part, and variant parameter overrides are not one interchangeable boolean.
- The existing parametric-tooling statement that locked alignment members are
  skipped conflicts with the ratified S5 whole-operation refusal and must be
  reconciled before implementation.

## Compound attribute inventory

The classifications below describe product placement, not current implementation
status. Every editable field still requires typed operation authority, complete
preflight, one commit, and one undo step.

### Universal compound surface

| Attribute or action | Placement | Notes |
|---|---|---|
| Member count/types and inventory | Inspector, read-only | Stable deterministic identity list |
| Combined bounds and reference | Inspector, read-only/command input | Must name the reported reference |
| Workspace/document coverage | Inspector, read-only | Mutation remains active-workspace-only for GUI users |
| Layer/net/side coverage | Inspector, read-only | Derived coverage is not a writable aggregate |
| Hidden/locked/incompatible counts | Inspector, read-only/preflight | Explain exact blockers |
| Position X/Y | Compound transform | Editing computes a delta; it does not assign one absolute coordinate to every member |
| Rotation | Compound transform | Explicit reference; compatible objects only |
| Schematic mirror H/V | Homogeneous symbol transform | Domain-specific; not generic geometry mirror |
| Lock/unlock | Compound property after substrate | Requires universal typed lock authority |
| Group | Explicit authored operation after substrate | Creates persistent `Group XXX`; never implicit |

### PCB objects

| Scope | Candidate common fields | Dedicated tools / derived or deferred fields |
|---|---|---|
| Parts/packages | value, rotation, lock | side/flip, replacement, annotation, DNP/variant, fields, assembly attributes |
| Pads (footprint editor) | later typed net/mask/paste/rotation fields | shape, padstack, drill, layers, number, geometry, parent/library authority |
| Tracks | later typed width/layer fields | net/topology, rule reset, endpoint geometry, cleanup/merge, impedance |
| Vias | later typed diameter/drill fields | span/type, net, tenting/protection, padstack and stackup validation |
| Zones | later typed thermal fields | net/layer/priority, polygon, clearance, refill; fill is derived geometry |
| Text | style, height, stroke, alignment, visibility where typed | literal-content formulas, layer/side consequences, source/render intent |
| Graphics/outline/keepouts | none until authored identities converge | geometry, layer, keepout kind, outline vertex tools |
| Dimensions | later typed layer/text override | endpoints, units, precision, style; measured value is derived |
| Nets/netclasses/rules | display effective values | dedicated rule/netclass manager and inheritance provenance |
| Stackup/layers | display valid destinations | component side, track migration, via span, graphics reassignment are distinct operations |
| Production/assembly | display effective fitted/output state | variants, BOM, PnP, panel/manufacturing plans and outputs use dedicated authority |

### Schematic and library objects

| Scope | Candidate common fields | Dedicated tools / derived or deferred fields |
|---|---|---|
| Placed symbols | value, transform/display fields where typed | reference uniqueness, bindings, unit/gate, fields, variants, hidden-power policy |
| Symbol fields | visibility/position after typed ops | field table for key/value/add/delete/reorder and bulk component data |
| Wires | none as generic properties | topology-aware endpoint/segment drag, split/merge, delete/redraw |
| Buses/entries | display membership/attachment | bus name/members/segments and entry attachment in dedicated tools |
| Labels/ports | homogeneous typed fields later | rename/kind/direction with hierarchy/connectivity validation |
| Junction/no-connect | position/status display | connectivity-aware placement/removal/relocation |
| Text/graphics | formatting after schema exists | dedicated bulk formatting and geometry tools |
| Library pins | read placed projection | pin table/library authoring for name, number, electrical type, swap/alternates, style |
| Library symbols/parts | read binding/provenance | proposal-first library authority, validation, and propagation |

### Manufacturing and variant examples

DNP demonstrates why the inventory cannot stop at geometric attributes. Other
valid compound edits include fitted/unfitted state, BOM/board/simulation
exclusions, alternate parts, component parameters, procurement data, assembly
side, and output inclusion. They do not share one authority:

- base design attributes and variant overlays remain distinct;
- DNP does not imply Exclude from BOM;
- Exclude from Board does not mean the component is merely unpopulated;
- alternate parts require binding/pin compatibility; and
- generated artifacts and run evidence remain read-only projections.

These belong in a dedicated variant/assembly scope or table over stable
ComponentInstance identity, not a generic PCB-object boolean.

## Unsafe generic patterns

- Silent partial edits or skipping locked/incompatible members.
- Treating same-labeled fields such as `width`, `layer`, or `rotation` as the
  same semantic operation across unrelated types.
- Arbitrary scale of mixed EDA geometry.
- Generic PCB mirror that ignores side transfer, layer mapping, text
  readability, courtyard, 3D orientation, and routing consequences.
- Breaking connectivity or external relationships as an incidental transform
  option.
- Batch-assigning one reference designator or net name without uniqueness and
  connectivity planning.
- Shallow-copying stable IDs, revisions, derived fills, relationships, or
  generated evidence.
- Letting selection or persistent grouping grant authority over locked,
  library-owned, hierarchy-owned, or generated objects.

## Recommended phase boundary

### S5A — selection and compound inspection

- Ratified click, modifier, rectangle/lasso, auto-pan, eligibility, visibility,
  projection, and clearing behavior.
- Typed stable-identity set with optional focus member.
- Ephemeral compound Inspector subject with `All N` and per-type scopes.
- Common/`Mixed`/unavailable property presentation and exact affected counts.
- Read-only derived coverage and complete blocker reporting.

### S5B — selection authority substrate

- Persistent authored group identity, membership, operations, and undo/replay.
- Universal typed object-lock vocabulary or an explicit capability matrix.
- Typed field-level batch operations/patch contracts with stale-revision guards.
- Atomic preflight, one commit, one undo step, and no silent skipping.
- Narrow transforms already ratified for compatible active-workspace targets.

### Later dedicated domain tools

- Track/via bulk editor, zone manager/refill, padstack editor, text/graphics
  bulk editor, symbol fields, variants/assembly, rules/netclasses, stackup/layer
  transfer, annotation, align/distribute, copy/delete closure, and library tools.

This division is a sequencing boundary, not a product-scope rejection. S5A must
preserve the typed seams that S5B and later domain tools consume.

## Sources

- KiCad PCB Editor manual: <https://docs.kicad.org/master/en/pcbnew/pcbnew.html>
- KiCad Schematic Editor manual: <https://docs.kicad.org/master/en/eeschema/eeschema.html>
- Altium, Editing Multiple Design Objects:
  <https://www.altium.com/documentation/altium-designer/editing-multiple-design-objects>
- Altium, Design Object Selection:
  <https://www.altium.com/documentation/altium-designer/design-object-selection>
- Altium, PCB Placement and Editing Techniques:
  <https://www.altium.com/documentation/altium-designer/pcb/placement-editing-techniques>
- Altium, Variant Manager:
  <https://www.altium.com/documentation/altium-designer/variant-manager>
- AutoCAD, work with object properties:
  <https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-LT-DidYouKnow/files/GUID-94C065AB-FF9E-4752-B778-23D2FBB87E18.htm>
- AutoCAD, groups:
  <https://help.autodesk.com/view/ACD/2026/ENU/index.html?guid=GUID-C20C24AD-81F5-4E4A-A414-B1A2BC71E041>
- SOLIDWORKS, sketch blocks:
  <https://help.solidworks.com/2026/English/SolidWorks/Sldworks/HIDD_DVE_SKG_ADDREMOVE.htm>
- Datum PCB layout tool contract: `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`
- Datum schematic authoring tool contract:
  `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`
- Datum schematic connectivity spec: `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`
- Datum operation inventory: `crates/engine/src/substrate/operation.rs`
- Datum board types: `crates/engine/src/board/board_types.rs`
- Datum schematic types: `crates/engine/src/schematic/mod.rs`

## Open research and review items

- Exact persistent-group nesting, overlap, broken-member, and naming policy.
- Universal lock capability matrix and whether some authorities remain
  immutable rather than unlockable.
- Compound transform reference visualization and connectivity preview.
- Exact first implementation inventory after field-level operation audit.
- Variant/assembly vocabulary beyond the current fitted-components overlay.
- Copy/delete closure and hierarchy-aware duplication semantics.
