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

The canonical pin electrical taxonomy is `LibraryPinElectricalType:v1`.
It is owned by the library/pool model in
`crates/engine/src/pool/pin.rs`; schematic pins consume the same taxonomy
through `schematic::PinElectricalType`, which is an alias to
`LibraryPinElectricalType`. ERC consumes this library-owned type directly
and emits the canonical snake-case names in pin evidence:

```rust
pub enum PinElectricalType {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    TriState,
    NoConnect,
}
```

```text
input
output
bidirectional
passive
power_in
power_out
open_collector
open_emitter
tri_state
no_connect
```

`OpenCollector`, `OpenEmitter`, and `TriState` are explicit drivers but are
not treated as strong push-pull conflicting outputs by current ERC
classification. `NoConnect` is intentionally supported both as a pin
electrical type and as a separate no-connect marker; either representation
can express intentional disconnection, and ERC keeps both paths explicit.

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

2. `power_in_without_source`
- power-input pins are attached to a net with no valid power source

3. `noconnect_connected`
- a pin marked `NoConnect`, or carrying a no-connect marker, is attached to
  a resolved net

4. `input_without_explicit_driver`
- a net has input pins without an explicit driver, but passive biasing or
  attached components make the current severity informational

5. `undriven_input_pin`
- input pins are attached to a net with no valid driver/source

6. `unconnected_component_pin`
- a non-optional pin has no net attachment and no waiver/no-connect marker

7. `unconnected_interface_port`
- an interface port is isolated from component pins and labels

8. `undriven_power_net`
- a power-like named net has no connected component pins

9. `undriven_named_net`
- a non-power named net has no connected component pins

10. `hierarchical_connectivity_mismatch`
- parent/child port mapping is incomplete or inconsistent

The target `passive_only_net` rule remains unimplemented as a distinct
finding code. Current ERC folds passive-only/biasing context into
`input_without_explicit_driver` severity decisions rather than emitting a
separate passive-only finding.

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

The compatibility matrix is authored in the engine classification helpers
and versioned by `LibraryPinElectricalType:v1`. The current implementation
classifies `Output` and `PowerOut` as strong conflicting outputs, while
`OpenCollector`, `OpenEmitter`, and `TriState` count as explicit drivers
without triggering output-output contention by themselves.

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
