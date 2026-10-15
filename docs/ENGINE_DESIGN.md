# Engine Architecture

> **Status**: Non-normative architecture rationale.
> The controlling API and type contracts are `specs/ENGINE_SPEC.md`,
> `specs/PROGRAM_SPEC.md`, and `specs/MCP_API_SPEC.md`.
> The [ownership map](../specs/PROGRAM_SPEC.md#ownership-and-precedence)
> identifies inherited precedence; [CANONICAL_IR](CANONICAL_IR.md) retains its
> governed ownership. This rationale does not promote examples to requirements.

## Principle: Headless-First

The engine is a Rust library with no GUI dependencies. Headless consumers
use it without linking windowing or rendering code. The original architecture
sketch also included optional Python embedding; the sketch does not claim
implemented bindings or authorize a new dependency.

The GUI is a separate crate that consumes the engine API, just like the
CLI and MCP server do. Every committed design change flows through the
operation model. Interactive behaviors (drag, hover, route-in-progress)
are consumer-specific — they produce operations, they are not operations.

For the opening `M7` slice, the GUI should consume versioned deterministic
review-scene and review-payload contracts rather than reading ad hoc engine
state directly. The frontend may own workspace, viewport, hover/focus,
selection, terminal-lane, and AI-lane presentation state, but it must not own
parallel design truth or unofficial write semantics.
The locked `M7` shell and lane decisions do not change that authority
boundary; they only constrain how the frontend presents deterministic review
state.

Illustrative original consumer topology:

```
engine (lib crate)
├── No GUI dependencies
├── No rendering dependencies
├── No windowing dependencies
├── Compiles to: library, CLI binary, Python module (via PyO3)
│
├── Consumed by:
│   ├── gui (bin crate) — wgpu renderer + editor
│   ├── cli (bin crate) — batch operations
│   ├── mcp-server (Python) — AI agent interface
│   └── python module — scripting
```

## Data Model

[Canonical IR](CANONICAL_IR.md) owns identity, precision, authored/derived
boundaries, transaction invariants and rule representation until formally
promoted or subsumed. This rationale does not duplicate its definitions.

| Contract | Existing owner |
| --- | --- |
| Resolved model and source-shard authority | [ENGINE_SPEC §0.2](../specs/ENGINE_SPEC.md#02-target-authority-model) and [§0.3](../specs/ENGINE_SPEC.md#03-source-shards-as-persistence-partitions) |
| Pool field/type contracts | [ENGINE_SPEC §1.2](../specs/ENGINE_SPEC.md#12-pool-types) |
| Board field/type contracts | [ENGINE_SPEC §1.3](../specs/ENGINE_SPEC.md#13-board-types) |
| Schematic field/type contracts | [ENGINE_SPEC §1.4](../specs/ENGINE_SPEC.md#14-schematic-types) |
| Native file layouts and schemas | [NATIVE_FORMAT_SPEC](../specs/NATIVE_FORMAT_SPEC.md) |

Read the engine specification's [status labels](../specs/ENGINE_SPEC.md#01-status-labels)
and owning reconciliation sections: target, current implementation and
compatibility shapes have different scopes. The old field diagrams in this
rationale are replaced by those links; they are not another schema authority.

### Pool (Component Library)

The architectural inspiration is Horizon's pool concept, redesigned for AI
queryability. Individual JSON objects support human inspection and source
control; a rebuildable SQLite index supports fast lookup. This is a structural
principle, not adoption of Horizon's schema. The Part connects electrical and
physical library references with purchasable-component metadata. Exact fields,
relationships, storage and supported authoring are owned by the contracts above.

### Board

The board is a physical projection/partition of the resolved model. Its native
file representation is a persistence choice, not a second project authority.
Use the linked board type and persistence owners for exact fields and layouts.

### Schematic

Sheets and hierarchy partition electrical presentation and intent. Their files
remain inputs to the engine-owned resolver, not independent design truth.
Exact source types and storage belong to the linked owners.

## Operation System

The typed operation/batch, proposal, commit and compensating undo contracts live
in [ENGINE_SPEC §0.8](../specs/ENGINE_SPEC.md#08-operations-proposals-transactions-and-commit).
The [IR transaction rationale](CANONICAL_IR.md#4-transaction-model) explains why
validation, serializable intent, identity diffs and affected-data invalidation
belong together. Compound actions share the atomic batch/journal path.

The former dynamic `Operation` trait example is retired from this rationale;
it must not be read as an alternative mutation gateway. The exact owner links
preserve the distinction between target substrate and compatibility APIs.

### Operation Categories

Illustrative workflow families from the original architecture sketch, not an
implemented command/variant inventory or a qualification claim:

```
Pool Operations:
  CreatePart, UpdatePart, ImportLibrary

Board Operations:
  PlaceComponent, MoveComponent, RotateComponent, DeleteComponent
  AddTrack, DeleteTrack, SetTrackWidth
  AddVia, DeleteVia
  ProposeRoute (AI-generated route for a net, validated before commit)
  ProposeRouteDiffpair (AI-generated diff pair route)
  AddTrackManual (point-to-point with clearance-aware pathfinding)
  TuneLength
  AddZone, UpdateZone, PourCopper
  SetDesignRule, DeleteDesignRule

Schematic Operations:
  PlaceSymbol, Wire, PlaceLabel, PlacePowerSymbol
  Annotate, AssignPart

Cross-Domain:
  SyncSchematicToBoard (ECO — generates list of atomic operations)
  SyncBoardToSchematic (backannotation)

Export:
  ExportGerber, ExportDrill, ExportBOM, ExportSTEP, ExportPDF
```

## Rule Engine

The canonical expression representation belongs to
[CANONICAL_IR §6](CANONICAL_IR.md#6-rule-representation). Evaluator support and
explicit unsupported-node errors belong to the
[ENGINE_SPEC support matrix](../specs/ENGINE_SPEC.md#16-rulescope-support-matrix).
The examples below illustrate the design motivation, not current parser syntax
or a replacement support inventory.

Examples (informed by Altium's query language):

```
clearance {
    scope: InNetClass("HighSpeed") And OnLayer("Top")
    value: 0.15mm
    priority: 10
}

track_width {
    scope: InNet("USB_D+") Or InNet("USB_D-")
    min: 0.1mm
    preferred: 0.127mm  // 90Ω differential
    max: 0.2mm
    priority: 20
}

clearance {
    scope: All
    value: 0.1mm
    priority: 1  // default, lowest priority
}
```

The canonical representation and support matrix above own node names, priority
and evaluation semantics; this rationale adds no separate rule contract.

## API Surface

Use [ENGINE_SPEC §5 API Contract Mode](../specs/ENGINE_SPEC.md#api-contract-mode)
to distinguish the target substrate, current implementation, compatibility API
and legacy Target M2+ surface. The
[current API inventory](../specs/ENGINE_SPEC.md#51-current-implemented-engine-api)
identifies its implementation owners. New design follows the substrate contract
in [§0](../specs/ENGINE_SPEC.md#0-product-mechanics-integration-and-migration-target),
including the single commit gateway linked above.

The former duplicate `Engine` method sketch and `execute_batch` example are
retired. Their original bytes remain in `62819c9b:docs/ENGINE_DESIGN.md`; they
do not establish implemented support, public tool names or an alternative write
path. Cross-consumer requirements remain in the existing formal subject owners
and domain contracts, not this architectural sketch.
