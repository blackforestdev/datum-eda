# ERC — Design Rationale and Domain Analysis

> **Status**: Non-normative design rationale.
> The controlling ERC specification is `specs/ERC_SPEC.md`.
> The controlling waiver and severity model is `specs/CHECKING_ARCHITECTURE_SPEC.md`.
> This document provides domain context, design reasoning, and reference
> material that informed the formal specs. It does not define contracts.

## Purpose
Provides domain analysis for the electrical rule checking subsystem:
pin electrical semantics, net driving theory, compatibility reasoning,
and check design rationale. Read this to understand *why* the ERC spec
says what it says.

---

## 1. Pin Electrical Types

Every pin in the pool has a PinElectricalType (defined canonically in
`specs/ERC_SPEC.md` §3.1). The types and their domain meanings:

| Type | Symbol | Meaning |
|------|--------|---------|
| Input | `I` | Consumes a signal. Must be driven by something. |
| Output | `O` | Drives a signal. Produces a defined logic level. |
| Bidirectional | `B` | Can drive or receive (e.g., data bus, I2C SDA). |
| Passive | `P` | No driving or receiving semantics (resistor, capacitor leads). |
| PowerIn | `W` | Consumes power. Must be connected to a power source. |
| PowerOut | `E` | Provides power. Drives a power net. |
| OpenCollector | `C` | Open-drain/open-collector output. Can sink but not source. Multiple allowed on one net. |
| OpenEmitter | `X` | Open-emitter output. Can source but not sink. |
| TriState | `T` | Three-state output. Can drive, receive, or float. |
| NoConnect | `N` | Must not be connected to anything. |

These types come from:
- Library pin definitions (KiCad .kicad_sym, Eagle .lbr)
- Pool author assignments
- Overrides per Part (a Part can override its Entity's pin types)

---

## 2. Net Driving Analysis — Design Rationale

Every resolved net is analyzed for its driving state. A net's driver is
determined by the pin types connected to it.

### Driver Classification

| Pin type on net | Drives? | Notes |
|-----------------|---------|-------|
| Output | Yes | Strong driver |
| PowerOut | Yes | Strong driver (power) |
| Bidirectional | Yes (weak) | Can drive, but defers to strong drivers |
| TriState | Yes (weak) | Can drive, but may be floating |
| OpenCollector | Yes (weak) | Needs pull-up to define high state |
| OpenEmitter | Yes (weak) | Needs pull-down to define low state |
| Input | No | Needs a driver |
| PowerIn | No | Needs a power source |
| Passive | Neutral | Neither drives nor requires driving |
| NoConnect | N/A | Must not be on a net at all |

### Net Drive States

After examining all pins on a net:

| State | Condition | Rationale for ERC result |
|-------|-----------|--------------------------|
| Driven | At least one strong driver | OK — net has a definite source |
| Weakly driven | At least one weak driver, no strong driver | Warning — may be intentional (wired-OR, tri-state bus) |
| Passive only | Only Passive pins | Warning — net exists but does nothing |
| Undriven | Has Input or PowerIn pins but no driver | Error — definite design mistake |
| Multi-driven | Multiple strong drivers (Output↔Output) | Error — contention |
| Float risk | TriState or OpenCollector without pull resistor inference | Warning — may float |
| No connect | Has NoConnect pin and also other connections | Error — contradictory intent |
| Empty | Net exists but has no pins | Warning — orphaned label |

---

## 3. Pin-to-Pin Compatibility — Reference Matrix

This matrix documents the domain reasoning behind the compatibility
model defined in `specs/ERC_SPEC.md` §5.

Legend: ✓ = OK, W = Warning, E = Error, blank = symmetric

|              | Input | Output | Bidir | Passive | PowerIn | PowerOut | OpenCol | OpenEm | TriState | NoConn |
|-------------|-------|--------|-------|---------|---------|----------|---------|--------|----------|--------|
| **Input**   | W     | ✓      | ✓     | ✓       | W       | ✓        | ✓       | ✓      | ✓        | E      |
| **Output**  |       | E      | ✓     | ✓       | ✓       | E        | W       | W      | W        | E      |
| **Bidir**   |       |        | ✓     | ✓       | ✓       | ✓        | ✓       | ✓      | ✓        | E      |
| **Passive** |       |        |       | ✓       | ✓       | ✓        | ✓       | ✓      | ✓        | E      |
| **PowerIn** |       |        |       |         | ✓       | ✓        | W       | W      | W        | E      |
| **PowerOut**|       |        |       |         |         | E        | W       | W      | W        | E      |
| **OpenCol** |       |        |       |         |         |          | ✓       | E      | ✓        | E      |
| **OpenEm**  |       |        |       |         |         |          |         | ✓      | ✓        | E      |
| **TriState**|       |        |       |         |         |          |         |        | ✓        | E      |
| **NoConn**  |       |        |       |         |         |          |         |        |          | E      |

Key reasoning for non-obvious combinations:
- **Output↔Output** (E): Two drivers fighting. Always an error.
- **PowerOut↔PowerOut** (E): Two power sources on the same net. Always error.
  Exception: if both PowerOut pins belong to the same component (e.g.,
  multiple VCC pins on one IC), this is OK — resolved by checking whether
  both pins belong to the same component.
- **Input↔Input** (W): Multiple inputs with no driver. Warning, not error,
  because the driver may be off-sheet or via a global label.
- **OpenCollector↔OpenCollector** (✓): Wired-OR/wired-AND topology. Valid.

---

## 4. M2 Check Design Rationale

The seven checks required for M2 (enumerated in `specs/ERC_SPEC.md` §4)
were chosen because they catch the most common and dangerous schematic
errors:

### output_to_output_conflict
Two strong drivers on one net is always a hardware error. In real circuits
this causes contention current, potential damage, and undefined logic
levels. No legitimate design has this.

### undriven_input
An input with no driver is either a missing connection or a schematic
that was left incomplete. This is the single most common schematic error.

### power_without_source
A power net (PowerIn pins) with no PowerOut source means an IC will not
receive power. This catches missing power symbol connections.

### noconnect_connected
A no-connect marker is an explicit assertion. If the pin IS connected,
the assertion is wrong. This catches stale no-connect markers after
design changes.

### unconnected_required_pin
A pin that should be connected but isn't. Warning level because some
unused pins are legitimately left floating (e.g., unused op-amp outputs).

### passive_only_net
A net where nothing drives and nothing receives. This usually indicates
an incomplete schematic (components placed but not yet wired into the
circuit).

### hierarchical_connectivity_mismatch
A port that doesn't connect through hierarchy means the sub-sheet is
electrically disconnected from the parent. This breaks the design intent.

---

## 5. Severity Configuration Rationale

The formal checking architecture (`specs/CHECKING_ARCHITECTURE_SPEC.md`)
defines three severity levels. The rationale for making severity
configurable per check is that different projects have different
engineering contexts:

- An audio amplifier project may promote `passive_only_net` to error
  (passive-only nets are always wrong in analog circuits)
- A digital project may demote `unconnected_required_pin` to info
  (many digital ICs have pins that are legitimately unused)

Severity configuration is authored data (part of project settings).

---

## 6. Waiver Design Rationale

Waivers are documented in `specs/CHECKING_ARCHITECTURE_SPEC.md` §4.
The design rationale:

- Waivers target specific objects, not blanket check suppressions, because
  blanket suppression hides new violations that appear after design changes.
- Waivers are authored data because they represent engineering decisions
  ("I know this looks wrong but it's intentional") that should survive
  across sessions and be visible in design reviews.
- Waived findings remain visible in reports to ensure waivers are reviewed
  periodically — a waiver from six months ago may no longer be valid.
- Waiver matching is UUID-based, not name-based, because object names
  can change (re-annotation) while identity should be stable.

---

## 7. Future ERC Checks (beyond M2)

These are documented for planning purposes. They do not appear in the
M2 exit criteria in `specs/PROGRAM_SPEC.md`.

| Check | Milestone | Description |
|-------|-----------|-------------|
| Bus width mismatch | M4 | Bus connects to port/label with different member count |
| Duplicate reference | M4 | Two symbols share the same reference designator |
| Missing annotation | M4 | Symbols with placeholder references (R?, U?) |
| Schematic-board mismatch | M4 | Schematic net not present in board, or vice versa |
| Unused sheet | M4 | SheetDefinition with no SheetInstances |
| Bidirectional contention | M6 | Multiple Bidirectional pins on a net with no protocol analysis |
