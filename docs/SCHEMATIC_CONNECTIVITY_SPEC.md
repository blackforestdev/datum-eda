# Schematic Connectivity — Design Rationale and Algorithm Detail

> **Status**: Non-normative design rationale.
> The controlling schematic connectivity specification is
> `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`.
> This document expands on the formal spec with detailed algorithm
> description, phase breakdown, and implementation guidance. It does not
> override the resolution rules, output graph shape, or M1 exit surface
> defined in the controlling spec.

## Purpose
Provides detailed algorithm design for schematic connectivity resolution.
The formal spec (`specs/SCHEMATIC_CONNECTIVITY_SPEC.md`) defines what must
be true about the output graph. This document describes how to compute it.

## Core Principle
Schematic connectivity is distinct from board connectivity. It is resolved
from schematic authored data alone: wires, junctions, labels, power symbols,
hierarchical ports, buses, and pin positions. The resolution algorithm must
be deterministic — same authored data produces an identical connectivity
graph on every run.

---

## 1. Authored Objects That Shape Connectivity

### Wires
- A SchematicWire connects two points on a sheet.
- Two wires that share an endpoint are electrically connected at that point.
- A wire that touches a pin endpoint connects to that pin.
- Wires do not cross-connect at intersections unless a Junction exists at
  the crossing point. This is a hard invariant — implicit cross-connects
  are a source of ERC errors in other tools and are prohibited here.

### Junctions
- A Junction at a point forces electrical connection of all wires and pins
  whose endpoints coincide with that point.
- Junctions are authored data (the user explicitly places them).
- The engine may suggest missing junctions (as a diagnostic), but never
  creates them implicitly.

### Net Labels
Four kinds, each with different scoping rules:

| Kind | Scope | Rule |
|------|-------|------|
| Local | Single sheet | Connects all wires touching labels with the same name on the same sheet. Does NOT cross sheet boundaries. |
| Global | Entire design | Connects all wires touching labels with the same name across ALL sheets. |
| Hierarchical | Sheet↔parent | Connects to a matching HierarchicalPort on the sheet's parent. See §3. |
| Power | Entire design | Semantically equivalent to a Global label, but additionally flags the net as a power net with a specific power symbol. Connects across all sheets. |

### Power Symbols
- A power symbol is a placed symbol with a single hidden power pin.
- The hidden pin is implicitly connected to a global net named after the
  power symbol (e.g., VCC, GND, +3V3).
- The power net name is the symbol's value, not its reference designator.
- Power symbols on different sheets with the same value share the same net.
- Power symbols are the mechanism by which power nets propagate globally
  without explicit global labels on every sheet.

### Buses
- A Bus is a named container for scalar net members.
- Bus naming convention: `BUS_NAME[0..7]` expands to `BUS_NAME0`,
  `BUS_NAME1`, ..., `BUS_NAME7`.
- Alternative syntax: `{NET_A, NET_B, NET_C}` defines explicit members.
- A bus is NOT an electrical net — it is a grouping mechanism. Individual
  scalar members are the actual nets.
- Bus wires carry no electrical connectivity themselves; only BusEntry
  connections create connectivity between a bus member and a wire/label.

### Bus Entries
- A BusEntry connects a single scalar member of a bus to a wire.
- The scalar member is determined by the label on the wire connected to
  the bus entry, not by position.
- A BusEntry without a connecting wire or label is a connectivity error.

### Hierarchical Ports
- A HierarchicalPort on a sheet defines a named connection point that is
  visible as a pin on the parent sheet's SheetInstance block.
- Direction (Input, Output, Bidirectional, Passive) is metadata for ERC,
  not a connectivity constraint.

### No-Connect Markers
- A NoConnectMarker targets a specific (PlacedSymbol, Pin) pair.
- It asserts: "this pin is intentionally unconnected."
- A no-connect marker suppresses the ERC unconnected-pin check for that pin.
- A no-connect pin that IS connected to a wire or net is an ERC error.

### Pin Positions
- Each placed symbol has pins at computed positions (from the Symbol
  graphics, transformed by the PlacedSymbol's position, rotation, mirror).
- A pin endpoint that coincides with a wire endpoint (within zero
  tolerance — integer coordinates, exact match) creates a connection.

---

## 2. Net Resolution Algorithm

### Phase 1: Local Connectivity (per sheet)

For each sheet, build a connectivity graph of points:

```
1. Collect all connection points:
   - Wire endpoints (from, to)
   - Pin endpoints (computed from placed symbol position + pin offset)
   - Junction positions
   - Label positions (Local, Global, Hierarchical, Power)

2. Build adjacency:
   - Two points are connected if they share exact coordinates AND:
     a. Both are wire endpoints (wire-to-wire), OR
     b. One is a wire endpoint and the other is a pin endpoint, OR
     c. A Junction exists at that position (forces connection of all
        points at that position)
   - Wire-to-wire connections at shared endpoints are always valid.
   - Pin-to-pin connections without an intervening wire are NOT valid
     (two pins at the same position don't connect without a wire).

3. Compute connected components:
   - Union-find on the adjacency graph.
   - Each connected component is a "local net segment."

4. Assign net names to segments:
   - If a segment contains a Local label → net name = label name
   - If a segment contains a Global label → net name = label name (global scope)
   - If a segment contains a Power label/symbol → net name = power name (global scope)
   - If a segment contains a Hierarchical label → net name = label name (resolved in §3)
   - If a segment contains multiple labels with different names → connectivity
     diagnostic: ambiguous net name
   - If a segment contains no labels → unnamed net (auto-generated name: "Net-<sheet>-<index>")

5. Attach pins to segments:
   - Each pin whose endpoint is in a segment's connected component is
     attached to that segment's net.
   - Record: (PlacedSymbol UUID, Pin UUID) → Net UUID
```

### Phase 2: Global Net Merging

After all sheets have local connectivity:

```
1. Global labels:
   - All local net segments across all sheets that contain a Global label
     with the same name are merged into a single net.

2. Power symbols:
   - All local net segments that contain a Power symbol with the same value
     are merged into a single net.
   - Power nets are implicitly global.

3. Net class inheritance:
   - If the design assigns a net class to a net name, all merged segments
     inherit that net class.
```

### Phase 3: Hierarchical Resolution

```
For each SheetInstance in the design:
  1. Identify the child sheet (from SheetDefinition → root_sheet).
  2. Identify Hierarchical labels on the child sheet.
  3. For each Hierarchical label:
     a. Find the matching HierarchicalPort on the child sheet
        (name must match exactly).
     b. Find the corresponding port pin on the parent sheet's
        SheetInstance block.
     c. The wire connected to the SheetInstance port pin on the parent
        sheet determines the parent-side net.
     d. Merge the child-side net segment (containing the Hierarchical label)
        with the parent-side net.

  4. Unmatched ports:
     - A HierarchicalPort on a child sheet with no matching label inside
       the sheet → connectivity diagnostic (unused port).
     - A SheetInstance port on the parent with no wire connected →
       connectivity diagnostic (floating port).

  5. Multi-instance sheets:
     - If the same SheetDefinition is instantiated multiple times, each
       instance gets independent net resolution. The same Hierarchical
       label in the child sheet resolves to DIFFERENT parent nets for
       each instance.
     - Local labels inside a multi-instance sheet are instance-scoped
       (each instance has its own copy of the local net).
     - Global labels inside a multi-instance sheet are STILL global
       (they connect across all instances). This is intentional — a
       power symbol inside a reused sub-sheet should connect to the
       global power net.
```

### Phase 4: Bus Expansion

```
For each Bus on each sheet:
  1. Parse the bus name to extract scalar members:
     - "DATA[0..7]" → ["DATA0", "DATA1", ..., "DATA7"]
     - "{SDA, SCL}" → ["SDA", "SCL"]

  2. For each BusEntry connected to this bus:
     a. Identify the wire connected to the BusEntry.
     b. Identify the label on that wire (if any).
     c. The label name must match one of the bus members.
     d. If no label → connectivity diagnostic (unlabeled bus entry).
     e. If label doesn't match any member → connectivity diagnostic
        (bus entry label not a member of the bus).

  3. Bus-to-bus connections across hierarchy:
     - A bus on a child sheet connected to a HierarchicalPort propagates
       all its members through the hierarchy, following the same rules
       as scalar Hierarchical labels but applied per-member.
```

### Phase 5: Output

The final connectivity graph:

```
SchematicConnectivity {
    nets: HashMap<Uuid, ResolvedNet>,
}

ResolvedNet {
    uuid: Uuid,
    name: String,
    class: Option<Uuid>,           // net class, if assigned
    is_power: bool,
    pins: Vec<(Uuid, Uuid)>,       // (PlacedSymbol UUID, Pin UUID)
    labels: Vec<(Uuid, LabelKind)>, // labels contributing to this net
    sheets: Vec<Uuid>,             // sheets this net appears on
    segments: Vec<NetSegment>,     // per-sheet wire segments
}

NetSegment {
    sheet: Uuid,
    wires: Vec<Uuid>,
    junctions: Vec<Uuid>,
    labels: Vec<Uuid>,
}
```

This graph is the input to ERC (§see ERC_SPEC.md) and to schematic query
tools (get_schematic_net_info, get_connectivity_diagnostics, etc.).

---

## 3. Connectivity Diagnostics

The connectivity resolver produces diagnostics for ambiguous or problematic
situations. These are NOT ERC violations — they are connectivity-layer
issues that must be resolved before ERC can run meaningfully.

| Diagnostic | Severity | Description |
|-----------|----------|-------------|
| `ambiguous_net_name` | Error | A wire segment has multiple labels with different names |
| `unlabeled_bus_entry` | Warning | A BusEntry has no labeled wire connected |
| `bus_member_mismatch` | Error | A BusEntry label doesn't match any bus member |
| `floating_port` | Warning | A SheetInstance port on the parent has no wire |
| `unused_port` | Warning | A HierarchicalPort on a child sheet has no matching label |
| `orphan_wire` | Info | A wire segment connects to no pins and has no label |
| `pin_overlap_no_junction` | Warning | Two different-net wires cross at a point with no Junction |
| `duplicate_label` | Warning | Same label name appears multiple times on one segment (redundant) |

---

## 4. Determinism Requirements

1. Net UUIDs are deterministic: derived from net name via UUID v5 with a
   connectivity namespace. Same authored data → same net UUIDs.
2. Auto-generated net names for unnamed segments use a deterministic naming
   scheme based on sheet UUID and sorted wire UUIDs, not insertion order.
3. The order of pins in a ResolvedNet.pins list is sorted by
   (PlacedSymbol UUID, Pin UUID) for deterministic serialization.
4. Multi-instance sheets produce instance-qualified net names for local
   nets: `<instance_path>/<local_net_name>`.

---

## 5. Incremental Recomputation

After an operation modifies schematic authored data:

1. Identify affected sheets (from OpDiff: which sheets had objects
   created, modified, or deleted).
2. Re-run Phase 1 for affected sheets only.
3. Re-run Phase 2 (global merge) for nets that include segments on
   affected sheets.
4. Re-run Phase 3 (hierarchy) for instances whose parent or child sheet
   was affected.
5. Re-run Phase 4 (bus expansion) for buses on affected sheets.
6. Diff the new connectivity graph against the previous one.
7. Emit change notifications for nets that changed (for ERC re-check,
   query cache invalidation, and GUI update).

Full recomputation is always correct but slow. Incremental recomputation
is an optimization — if it produces different results than full recompute,
that is a bug.

---

## 6. Relationship to Board Connectivity

Schematic connectivity and board connectivity are parallel systems:

| Aspect | Schematic | Board |
|--------|-----------|-------|
| Source | Wires, labels, ports, pins | Tracks, vias, pours, pads |
| Output | ResolvedNet (which pins should connect) | CopperIsland (which pads are connected) |
| Purpose | Design intent | Physical realization |
| Used by | ERC | DRC |
| Cross-check | Forward/backward annotation | Same |

The schematic connectivity graph defines WHAT should be connected.
The board connectivity graph defines WHAT IS connected.
The difference between them is the airwire list (unrouted connections)
and the ECO (engineering change order).

---

## Milestone Position
- Net resolution algorithm implemented in M1
- Full hierarchy support in M1
- Bus expansion in M1
- Connectivity diagnostics in M1
- Stable query/ERC input graph required by M2
- Incremental recomputation in M3 (required for operation support)
