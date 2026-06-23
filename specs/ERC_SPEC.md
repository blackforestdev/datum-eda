# ERC Specification

## 1. Purpose

Defines electrical rule checking on the schematic connectivity graph.
ERC validates electrical intent and symbol/pin semantics. It is distinct
from DRC, which validates physical implementation on the board.

---

## 2. Inputs

ERC consumes:
- schematic authored data
- resolved `SchematicConnectivityGraph`
- pin electrical types
- symbol/component metadata
- ERC waivers

ERC does not consume:
- copper geometry
- routed length
- board clearance data

---

## 3. Electrical Semantics

### 3.1 Pin Electrical Types

The 10-variant taxonomy below is the **target**. The shipped
`PinElectricalType` (`crates/engine/src/schematic/mod.rs`, verified
2026-06-22) narrows it to six variants — it omits `TriState`,
`OpenCollector`, `OpenEmitter`, and `NoConnect` (no-connect is modeled
separately via no-connect markers, not as a pin electrical type):

```rust
// Target taxonomy:
pub enum PinElectricalType {
    Input,
    Output,
    Bidirectional,
    Passive,
    TriState,       // target — not in shipped enum
    OpenCollector,  // target — not in shipped enum
    OpenEmitter,    // target — not in shipped enum
    PowerIn,
    PowerOut,
    NoConnect,      // target — modeled as no-connect markers, not a pin type
}

// Shipped (six variants):
//   Input, Output, Bidirectional, Passive, PowerIn, PowerOut
```

### 3.2 Net Semantic Classes

```rust
pub enum NetSemanticClass {
    Power,
    Ground,
    Signal,
    Clock,
    Analog,
    DifferentialMember,
}
```

Net semantic class may be:
- explicitly imported from source constructs
- inferred from power symbols / labels
- assigned by rules later in the pipeline

---

## 4. M2 ERC Rule Set

The following checks are required for `M2`:

1. `output_to_output_conflict`
- two or more strong outputs drive the same net

2. `undriven_input`
- input or power-in pin is attached to a net with no valid driver/source

3. `power_without_source`
- a power net contains only `PowerIn` pins and no valid power source

4. `noconnect_connected`
- a pin marked `NoConnect` is attached to a resolved net

5. `unconnected_required_pin`
- a non-optional pin has no net attachment and no waiver/no-connect marker

6. `passive_only_net`
- a net contains only passive pins and no explicit source

7. `hierarchical_connectivity_mismatch`
- parent/child port mapping is incomplete or inconsistent

M2 may emit `warning` rather than `error` for `passive_only_net`
depending on project defaults.

---

## 5. Compatibility Model

ERC evaluates pin compatibility by net:

| Driver/Sink Combination | Default Result |
|-------------------------|----------------|
| Output + Output | Error |
| Output + Input | OK |
| Output + Passive | OK |
| PowerOut + PowerIn | OK |
| PowerIn + PowerIn | Error if no source on net |
| NoConnect + anything connected | Error |
| Passive + Passive | Warning |
| TriState + Output | Warning or OK, configurable |

Rows that reference target-only pin types (`TriState`, and the pin-type
form of `NoConnect`; see § 3.1) are part of the target matrix, not the
shipped six-variant taxonomy.

The full compatibility matrix is authored in the engine and versioned in
the canonical ruleset.

---

## 6. Results

The location-bearing form below is the **target** result shape (a typed
`rule: ErcRuleKind`, a positional `index`, and a `location:
SchematicLocation`). It is not the shipped shape — see the implemented
shape that follows it.

```rust
// Target (not yet implemented in this exact form):
pub struct ErcReport {
    pub passed: bool,
    pub violations: Vec<ErcViolation>,
    pub summary: CheckSummary,
}

pub struct ErcViolation {
    pub index: u32,
    pub rule: ErcRuleKind,
    pub severity: CheckSeverity,
    pub message: String,
    pub location: SchematicLocation,
    pub objects: Vec<Uuid>,
    pub waived: bool,
}
```

**Shipped shape (`crates/engine/src/erc/mod.rs`, verified 2026-06-22):**
findings are emitted as `ErcFinding`, which differs from the target above —
there is no `index`, no `rule: ErcRuleKind` (the rule is a string `code`),
and no `location: SchematicLocation` (object identity is carried by
`ErcObjectRef` plus resolved `object_uuids`):

```rust
pub struct ErcFinding {
    pub id: Uuid,
    pub code: &'static str,         // rule code, e.g. "output_to_output_conflict"
    pub severity: ErcSeverity,
    pub message: String,
    pub net_name: Option<String>,
    pub component: Option<String>,
    pub pin: Option<String>,
    pub objects: Vec<ErcObjectRef>, // { kind: &'static str, key: String }
    pub object_uuids: Vec<Uuid>,
    pub waived: bool,
}
```

ERC and DRC share:
- severity levels
- summary shape
- waiver concept
- explanation/reporting surface

They do not share the same location type or checking engine.

---

## 7. Waivers

Waivers are authored data. The canonical `CheckWaiver` and `WaiverTarget`
definitions live in `CHECKING_ARCHITECTURE_SPEC.md`.

ERC-specific waiver expectations:
- ERC waivers match schematic-object UUIDs and ERC rule/object tuples
- waived findings remain visible but do not fail the check
- waiver matching must not depend on human-readable net or sheet names

---

## 8. Exit Surface

For `M2`, ERC is complete enough when:
- the M2 rule set runs on imported KiCad/Eagle schematics
- results are exposed via CLI and MCP
- findings can be explained in natural language
- waivers are persisted and honored deterministically
