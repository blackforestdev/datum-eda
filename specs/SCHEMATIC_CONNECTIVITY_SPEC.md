# Schematic Connectivity Specification

## 1. Purpose

Defines how authored schematic data resolves into a deterministic electrical
connectivity graph. This graph is the input to:
- schematic queries
- ERC
- schematic↔board synchronization
- constraint propagation from schematic intent to board rules

Schematic connectivity is a first-class subsystem, separate from board
connectivity.

---

## 2. Inputs

The resolver consumes authored schematic data only:
- sheets
- symbol instances
- pins on placed symbols
- wires
- junctions
- local labels
- global labels
- hierarchical labels and ports
- power symbols
- buses and bus members
- no-connect markers

It must not depend on board state, routed copper, or cached connectivity.

---

## 3. Output Graph

The target product graph is the `SchematicConnectivityGraph` shape below.
The shipped engine currently exposes deterministic read surfaces through
`schematic_net_info`, `schematic_hierarchy_info`, and
`schematic_diagnostics`; those surfaces carry the same graph intent in the
current `SchematicNetInfo`, `HierarchyInfo`, and
`ConnectivityDiagnosticInfo` API shapes.

```rust
pub struct SchematicConnectivityGraph {
    pub nets: HashMap<Uuid, SchematicNet>,
    pub pin_attachments: Vec<PinAttachment>,
    pub port_links: Vec<HierarchicalLink>,
    pub diagnostics: Vec<ConnectivityDiagnostic>,
}

pub struct SchematicNet {
    pub uuid: Uuid,
    pub name: String,
    pub members: Vec<NetMemberRef>,
    pub net_class: Option<Uuid>,
    pub semantic_class: Option<NetSemanticClass>,
}

pub struct PinAttachment {
    pub net: Uuid,
    pub component: Uuid,   // placed symbol/component instance
    pub gate: Option<Uuid>,
    pub pin: Uuid,
}

pub struct HierarchicalLink {
    pub parent_sheet: Uuid,
    pub child_sheet: Uuid,
    pub parent_port: Uuid,
    pub child_port: Uuid,
    pub net: Uuid,
}
```

The graph is derived data. Given identical authored schematic data, the
resolved graph must be byte-stable when serialized.

---

## 4. Resolution Rules

### 4.1 Sheet-Local Connectivity
- Wire endpoints that touch form a connection only if:
  - they share an endpoint, or
  - a junction explicitly joins them, or
  - the source format defines the touch as electrically connected
- Crossing wires without a junction are not connected unless the source
  format explicitly marks them as joined

### 4.2 Labels
- Local labels connect segments within the same sheet scope
- Global labels connect segments across all sheets
- Hierarchical labels connect through sheet ports only
- Conflicting labels on the same resolved segment are a connectivity error

### 4.3 Power Symbols
- Power symbols inject a named net into the sheet
- Power symbol names resolve as global nets unless the source format
  explicitly scopes them differently
- Hidden power pins connect to the resolved power net when the imported
  source format defines them that way

### 4.4 Hierarchy
- Child-sheet ports are linked to parent-sheet hierarchical labels/ports
- Missing or multiply-mapped ports produce connectivity diagnostics
- Hierarchical resolution is deterministic: same sheet graph and labels
  always produce the same net assignments

### 4.5 Buses
- Buses are containers for scalar nets, not electrical nets by themselves
- Members expand into scalar nets (`DATA[0]` → `DATA0` or equivalent source-
  format-preserving naming)
- Ambiguous bus syntax is a connectivity diagnostic, not silent coercion

Current imported KiCad subset:
- scalar member labels of the form `NAME[n]`
- simple contiguous bus-range labels of the form `NAME[a..b]`
- unambiguous geometric `bus_entry` association between one bus and one wire



**Foreign declared Bus contact (PM055, owner-resolved):** constrain a physical
Bus Run to the origin's declared semantic Bus and disclose contacting foreign
Bus declarations/occurrences separately. Equal names or scalar member sets do
not join it. Disconnected same-Bus representations widen only at the semantic
Bus tier; explicit Bus interfaces do not make local physical bridges. This is
complete derived membership within the lawful subject, independent of visibility
and context caps. Foreign contact is not co-selection or a scalar short. Missing
or ambiguous declaration/source authority is explicitly unavailable.


### 4.6 No-Connect
- A no-connect marker suppresses “unconnected pin” ERC for that exact pin
- A no-connect marker on a pin that is actually connected is an ERC error

---

## 5. Net Naming and Identity

- Human-readable net names are attributes, not identities
- Resolved schematic nets receive stable UUIDs in the canonical IR
- Imported nets derive deterministic UUIDs from source identity per
  `IMPORT_SPEC.md`
- Merged source members determine connectivity, never semantic UUID. Persisted
  NetId transitions follow PM053 C01 final-state nomination and recorded allocation.

---

## 6. Connectivity Diagnostics

These are resolver-level problems, not ERC policy decisions:
- unsupported bus/member label syntax in the current imported KiCad subset
  (`unsupported_bus_member_syntax`)
- dangling component pin (`dangling_component_pin`)
- dangling interface port (`dangling_interface_port`)
- anonymous multi-pin net without a label or port
  (`anonymous_multi_pin_net`)
- missing hierarchical port target (`missing_hierarchical_port_target`)
- multiply mapped hierarchical port (`multiply_mapped_hierarchical_port`)

Target diagnostics still include conflicting labels on the same segment,
malformed sheet hierarchy, and dangling symbol pin references. Those are
tracked as future connectivity expansion items until the resolver emits
concrete diagnostic kinds for them.

Diagnostics may block ERC if the graph cannot be resolved safely.

---

## 7. M1 Exit Surface

M1 must support:
- flat multi-sheet resolution
- local/global/hierarchical labels
- power symbols
- basic bus/member expansion
- deterministic pin-to-net graph output

M1 may defer:
- advanced bus syntax edge cases
- source-specific graphical quirks that do not affect electrical meaning

## PM053 C01–C03 owning reconciliation — 2026-10-01

[PM053](/docs/decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md) ratifies stable NetId final-state atomic nomination/fallback,
explicit logical-to-board identity relationships (pending/mismatch are lawful
incomplete states), and semantic Bus declarations with occurrence-qualified
bindings. These clauses control conflicting earlier summaries in this document:
read/cache/reopen never allocate identity; display rename preserves identity,
while a label edit changing electrical groups follows recorded transitions.
Bus drawing split/member reorder does not split its declaration; semantic split/
merge explicitly records survivor and complete redistribution. A declaration
can resolve with zero drawings. Full interface equality differs from related
subset/remapping; names/member equality never establishes Bus identity and Bus
has no board projection. Occurrence identity is source plus stable instance path.
Malformed references refuse atomically; pending/mismatch must never be published
as complete cross-domain Net selection. Authored identity/binding source is
distinct from recomputable graph/projection geometry. Canonical writes and replay
retain all recorded identities. No C04–C07, M1 numerical policy, geometry/CAM
support, selection mutation or native acceptance is ratified by this amendment.
