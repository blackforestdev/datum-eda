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

### 4.6 No-Connect
- A no-connect marker suppresses “unconnected pin” ERC for that exact pin
- A no-connect marker on a pin that is actually connected is an ERC error

---

## 5. Net Naming and Identity

- Human-readable net names are attributes, not identities
- Resolved schematic nets receive stable UUIDs in the canonical IR
- Imported nets derive deterministic UUIDs from source identity per
  `IMPORT_SPEC.md`
- If multiple source segments merge into one resolved net, the resolved
  net UUID is derived from the deterministic set of merged source members

---

## 6. Connectivity Diagnostics

These are resolver-level problems, not ERC policy decisions:
- conflicting labels on the same segment
- missing hierarchical port target
- ambiguous bus member expansion
- malformed sheet hierarchy
- dangling symbol pin reference

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

