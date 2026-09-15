# Authoring Tool Semantics — Design Rationale

> **Status**: Non-normative design rationale.
> The controlling schematic editor specification is `specs/SCHEMATIC_EDITOR_SPEC.md`.
> The controlling operation model is `specs/ENGINE_SPEC.md` §3.
> The controlling M3 operation list is `specs/PROGRAM_SPEC.md` §M3.
> The controlling M4 operation list is `specs/PROGRAM_SPEC.md` §M4 and
> `specs/SCHEMATIC_EDITOR_SPEC.md` §4.
> This document provides design reasoning for tool semantics — what each
> authoring action means, what it validates, and what edge cases it
> handles. It does not define operation names or API surfaces.

## Purpose
Documents the domain semantics of authoring actions — the engineering
meaning behind placing, moving, wiring, and deleting design objects.
This reasoning informs the formal operation specifications but does not
replace them.

## Design Principles

These principles are derived from the architecture decisions in
`CLAUDE.md` and `docs/ENGINE_DESIGN.md`:

### Atomicity
Each authoring action resolves to one or more atomic Operations (as
defined in `specs/ENGINE_SPEC.md` §3). The Operations are the smallest
undoable units. Actions that appear to do multiple things (e.g., "delete
component and its tracks") are compound — they emit multiple Operations
grouped in a Transaction.

### No Implicit Side Effects
Operations do not silently modify objects they weren't asked to modify.
Moving a component does not move tracks. Deleting a symbol does not
delete wires. Consumers (GUI, AI agent) compose atomic operations to
achieve compound behaviors.

### Connectivity is Computed, Not Authored
No operation directly modifies the connectivity graph. Operations modify
authored data (wires, tracks, pads, labels). Connectivity is recomputed
from authored data after each transaction.

### Grid and Snap are Consumer Concerns
The engine accepts exact coordinates. Grid snapping, pin snapping,
alignment guides — these are GUI/CLI behaviors that quantize coordinates
before emitting operations. The engine does not enforce any grid.

---

## 1. Schematic Tool Semantics

The M4 schematic operations are listed in `specs/SCHEMATIC_EDITOR_SPEC.md`
§4.1. This section documents the domain semantics behind them.

### Placing a Symbol
When a symbol is placed on a sheet, it creates a new PlacedSymbol with
pin endpoints computed from the symbol graphics plus the placement
position, rotation, and mirror. No net connections exist until wires
are drawn to pin endpoints.

For multi-gate entities, the first unplaced gate is used unless a specific
gate is specified. Placing when all gates are already placed is an error.

### Moving a Symbol
Moving a symbol updates its position. Pin endpoints recompute. Connected
wires do NOT move with the symbol — they remain at their original
positions. This may create disconnects that the user must fix.

GUI consumers may implement rubber-banding (wires stretch during drag)
as a preview, but the final result is a position change plus separate
wire endpoint adjustments. This is consistent with the "no implicit side
effects" principle and with `specs/SCHEMATIC_EDITOR_SPEC.md` §6.1.

### Drawing a Wire
Creating a wire between two points establishes connectivity if either
endpoint coincides with a pin endpoint or existing wire endpoint (exact
integer coordinate match). If a wire endpoint lands mid-segment on
another wire, a Junction must be explicitly placed — no implicit
T-connections. This is documented in `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`
§4.1.

### Placing a Junction
A Junction forces electrical connection of all wires and pins at its
position. The engine may suggest missing junctions as diagnostics but
never creates them implicitly.

### Labels and Net Naming
A label assigns a name to the wire segment at its position. Label scoping
follows the rules in `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` §4.2:
- Local: same sheet only
- Global: all sheets
- Hierarchical: through sheet ports
- Power: global, flags net as power

### No-Connect Markers
A no-connect marker targets a specific (symbol, pin) pair. It is an
assertion that the pin is intentionally unconnected. If the pin IS
connected, ERC flags it (per `specs/ERC_SPEC.md` §4, check 4).

### Annotation
Assigns reference designators deterministically: sorted by sheet order,
then by position (top-to-bottom, left-to-right). Existing non-provisional
references are not changed. Determinism requirement per
`specs/SCHEMATIC_EDITOR_SPEC.md` §5.

---

## 2. Board Tool Semantics

The M3 board operations are listed in `specs/PROGRAM_SPEC.md` §M3.
Board creation operations are in `specs/PROGRAM_SPEC.md` §M4.

### Moving a Component
Updates position (and optionally rotation). All pads recompute positions.
Tracks and vias connected to pads do NOT move — connections may break.
Airwires recompute. GUI may implement drag-with-tracks as a compound
transaction.

### Flipping a Component
Toggles the layer (Top↔Bottom). Mirrors pad positions. Swaps layer
assignments. Tracks on the old layer become disconnected.

### Deleting a Component
Removes the PlacedPackage. Does NOT automatically delete connected
tracks/vias. Those become orphaned segments. A user-facing "delete
component and tracks" is a compound transaction.

### Adding a Track
Creates a track segment. If endpoints coincide with pad centers, via
positions, or existing track endpoints on the same net and layer,
connectivity is established. DRC checks width and clearance rules.

GUI-level interactive routing emits sequences of track and via operations,
potentially with clearance-aware pathfinding. The engine operation is
"add this segment."

### Adding a Zone
Creates a zone with an authored polygon boundary. Fill geometry is
derived data — computed by the copper pour engine (obstacle subtraction,
thermal relief, island removal). Zone fill is a derived-data computation,
not an authored-data operation.

### Design Rules
Rules are authored data. Adding or modifying a rule may change DRC results
on the next run. Rule scope expressions use the AST defined in
`specs/ENGINE_SPEC.md` §1.5.

---

## 3. Cross-Domain Tool Semantics

### Forward Annotation (ECO)
Compares current schematic state to current board state. Produces a
per-change list (new component, deleted component, changed part, changed
value, new net, deleted net). Each change is individually accept/reject.
Accepted changes become Operations in a transaction.

### Backward Annotation
Propagates board-side changes (reference renames, value changes, pin
swaps) back to the schematic. Same accept/reject flow.

---

## 4. Layout Tool Semantics (M5+)

Layout tools (placement suggestions, routing proposals) produce
LayoutProposals as defined in `docs/LAYOUT_ENGINE.md` §6. They do NOT
directly modify the design. The proposal/review workflow means no
change commits without explicit acceptance.
