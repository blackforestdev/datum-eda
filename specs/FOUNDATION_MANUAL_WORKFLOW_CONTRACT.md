# Native doorway and exact-edit foundation contract

Status: WDQ-F02 draft for owner review; no execution or product acceptance.
Frontier: FOUNDATION-WORKFLOW-SPEC; issue `dat-manual-foundation-contracts-fsw`.
Owning route: `foundation-manual-workflow`.
Evidence: `research/process-quality/FOUNDATION_MANUAL_WORKFLOW_RESEARCH.md`
and `docs/reviews/workflow-foundation/authority-review.md`.

## Authority and delivery boundary

This is one connected manual workflow: ordinary launch → create/open native
Project → inspect a pinned local part and its electrical/physical binding →
select a board component → exact coordinate edit → cancel/commit/undo/redo →
close/reopen and recover from interruption. No AI, terminal command or imported
project is required to complete the manual path.

Existing ratifications remain controlling. Tables marked **inherited** map
them into this workflow; **proposed** choices require the WDQ-F03 disposition
and any applicable numbered decision before implementation. A draft cannot
override a Claude visual source or activate a reserved preference. The complete
schematic/PCB authoring contracts remain the native-authoring outcome; this
small scenario neither replaces their breadth nor proves enterprise readiness.

## Proposed native doorway and fixture envelope — F6/F7

Launch with no arguments into an operable native New Project/Open Project
doorway, not a process exit or hidden requirement to supply a project path.
Cancel returns to the previous valid application state without creating source.
An open failure retains the attempted location for correction and leaves any
already-open Project unchanged. Do not display technical journal history as a
mandatory formal Revision workflow; new Projects use the accepted Unmanaged
default unless independently adopted Project policy says otherwise.

New Project previews name/location and the applicable seed explanation, but
does not freeze seed values. At actual genesis, capture the immutable eligible
seed snapshot and receipt and atomically publish the complete native Project
to an absent destination. A colliding destination is refusal, never overwrite.
Genesis is not an edit transaction or undo entry. Reopen resolves source shards
and the journal, never an exported artifact or a stale workspace snapshot.

The proposed first proof fixture is explicitly native and project-local:

- A two-pin Part, Unit/Entity/Symbol, Package, Footprint, Padstack and explicit
  PinPadMap, pinned to known revisions; names are labels, not lookup identity.
- One ComponentInstance shared by the placed symbol and board realization.
  Capture actual persisted UUIDs as fixture evidence; never select by `R1` text.
- Board coordinates use the authored origin, not a camera or outline-derived
  origin. Proposed outline spans (-10,-10) to (20,20) mm so negative coordinates
  in the example do not imply an out-of-board placement.
- Component anchor initially (0,0) nm; exact target (5080000,-2540000) nm.
  A shared translation preserves all child offsets and electrical identity.
- Two connected logical pins with explicit net identities. The accepted
  PinPadMap, not equal names or visually touching geometry, binds physical pads.

This defines a fixture to author and verify in the future execution lane. It
does not claim that a bundled library, usable chooser, or fixture already exists.
Missing, ambiguous, incompatible or unavailable pinned bindings refuse placement
with object/revision details and no partial instance or journal entry. Library
updates affecting placed objects are proposal-first; no automatic latest wins.

## Exact quantities and entry — F1/F2 (inherited)

Use the shared checked-rational Units service and declared field quantity/context.
Do not create an editor-specific parser. Canonical lengths are signed i64 nm;
display unit/precision never rescales or rounds geometry. Explicit suffixes
override presentation context. Bare values need the declared Project field
context; headless calls without it refuse rather than reading machine defaults.

| Input / context | Exact result or refusal |
| --- | --- |
| `5.08mm`, `200mil`, `0.2in`, `5080000nm` | Each is 5080000 nm, regardless of current display unit |
| `-2.54mm`, `-100mil`, `-0.1in` | Each is -2540000 nm; sign applies to the whole value |
| `5.08e-3m` | Refuse unsupported suffix; `m` is not one of V1's length suffixes |
| `5.08e0mm` | 5080000 nm |
| `1um`, `1µm` | 1000 nm |
| `0.000001mm` | 1 nm |
| `0.0000005mm` | Refuse sub-nanometer value; no round-to-nearest |
| `9223372036854775807nm` | Exact i64 maximum at the scalar service boundary |
| `-9223372036854775808nm` | Exact i64 minimum at the scalar service boundary |
| `9223372036854775808nm`, `-9223372036854775809nm` | Refuse overflow, never wrap or saturate |
| `1,5mm`, `NaN`, `1mm+2mm`, `1e`, `-`, empty text | No successful commit; preserve correctable draft text |
| Bare `5.08` in mm context | 5080000 nm; same text in inch context means 129032000 nm |

Scalar representability is not permission to place geometry at an extreme:
the owning operation must also validate transforms, dimensions and constraints
with checked arithmetic. An extreme valid scalar that overflows a translated
child coordinate refuses the entire operation.

At decimal_3 mm display, canonical 5080001 nm displays rounded `5.080 mm`
with rounded-state metadata. Focus regenerates exact `5.080001mm` from canonical
state; it never uses the rounded label as input. If the selected unit has no
finite exact representation, expose exact suffixed nm text. Unchanged Enter or
blur is a semantic no-op. Escape restores the pre-edit value without reparsing
the rounded label. Invalid input remains editable and does not replace the last
valid value. Explicit display precision survives unit/system changes; Automatic
alone re-resolves. Rendering spacing is a visual contract, not parser authority.

Angles are a separate quantity: decimal degrees with explicit `deg`/`°` or a
declared bare-angle context. Exact conversion uses the field's declared integer
angle scale; precision 0–3 is display, not a new canonical scale. Refuse missing
scale, nonintegral canonical result, overflow, radians/DMS and length suffixes.

## Selection, coordinate frames and snap — F3/F4 (inherited)

Board pad clicks select the parent footprint/component subject; pad-definition
editing belongs to its editor. Capture selected stable IDs, owning pane and
resolver revision together. Switching focus does not replace shared selection;
identity-preserving edits retain it, deletion drops it with notice, and undo
does not silently resurrect consumer selection. S5A remains read-only.

View zoom, pan, rotation, device scale and grid visibility do not change model
coordinates. Invert the view transform before model-space snapping; measure
object-snap eligibility in screen space under the existing 10-pixel rule.
Eligible object snap wins over grid; exclude moved/hidden/ineligible targets.
Hiding the grid does not disable snapping or alter selection. The accepted
metric-first grid defaults remain board 0.5 mm and schematic 2.5 mm, not historic
imperial-derived 0.635/2.54 mm substitutes. Fine-grid behavior is the shared
tooling rule, not a preference identity invented for this contract.

Exact field entry must preserve its submitted canonical value, not secretly
quantize it to the displayed grid. Gesture reference is invocation-specific;
the focused selected member is not automatically a pivot. Quantize is an align
parameter. A locked, stale, incompatible, constrained or invalid member refuses
the entire selected operation with exact blockers and zero journal change.

## Transaction and history timeline — F5 (inherited unless marked)

Let J be the number of committed transactions before the action; record the
starting source hashes, object revisions and model revision separately.

| Stage | Expected source/journal result |
| --- | --- |
| Focus, type incomplete/invalid input, show preview | Original canonical state; J unchanged |
| Escape before commit | Draft discarded, original canonical state; J unchanged |
| Submit canonically unchanged value in another unit | No source rewrite; J unchanged |
| Submit valid changed coordinate | One validated typed batch, one journaled transaction; J+1 |
| Multi-selection preflight fails | Whole refusal, original selection and source retained; J unchanged |
| Undo committed edit | Original geometric values restored by compensating transaction; J+2, history not rewound |
| Redo | Target geometric values restored through the same durable history mechanism; J+3 |
| Close/reopen after each committed state | Exact corresponding values/identities and durable history; no new edit solely from reopening |
| Read-only check evaluation | No fabricated authored edit transaction; check evidence classified separately |

The GUI must observe one correlated action → typed request → commit/proposal →
journal outcome → resolver refresh. A process exit or lost response after the
commit point cannot be reported as a proved no-op or blindly retried as a fresh
edit. Reconcile journal state and show the actual outcome first.

Local visible undoable editing uses direct commit. Automation, cross-domain
intent changes and library propagation use the applicable proposal policy.
Proposal preview/rejection changes no authored design state; explicit acceptance
validates the current base and commits atomically with provenance. Stale proposals
are not silently rebased. Board movement alone does not rewrite electrical intent.

**Proposed wire gesture:** clicks extend one uncommitted wire-chain preview;
explicit Finish commits one chain batch, Escape cancels the whole uncommitted
chain. This requires owner reconciliation of the current “Esc to finish” text.
Native endpoint-on-segment T formation and crossing junction policy must be
resolved separately; UUID net identity does not answer formation semantics.

## Persistence and write ownership — F6

PM001/000D require staged bytes flushed before a durable journal append, then
shard promotion and directory synchronization. The durable journal append is
the edit commit point; Preferences head promotion is not Project authority.

| Failure / state | Required observable outcome |
| --- | --- |
| Invalid/unwritable destination or creation collision | Clear refusal, existing projects untouched, no success/genesis claim |
| Missing/incompatible shard or dangling required identity | Explicit resolver diagnostic/recovery state; not a partially accepted editable model |
| Interrupted edit before durable journal record | Reopen original committed state; discard only identified uncommitted staging |
| Interrupted edit after durable journal record, valid staged bytes | Roll forward the identified transaction and resolve its exact committed state |
| Journaled transaction with corrupt/missing recovery bytes | Preserve evidence and expose diagnostic recovery; never silently mix old/new shards |
| Storage/sync failure with uncertain commit outcome | Do not claim success or zero-write refusal until journal/recovery reconciles outcome |
| Second writer or stale base | Refuse before mutation under the ratified ownership protocol; no lock stealing or partial rename |
| Workspace restore failure | Recover source independently; stale cameras/panes never replay edits |

The writer protocol is **unresolved** in `dat-project-write-ownership-lock-0ne`.
Recommended for numbered decision: one engine-owned exclusive guard shared by
all writers/recovery; GUI uses daemon dispatch, headless writers the same guard.
Alternative: daemon-only ownership with explicit lifecycle/client rules. The
research lists descriptor, path-alias, lock-target and filesystem consequences.
This contract neither chooses the mechanism nor authorizes a dependency.

## Settings isolation and timing — F8 (inherited)

| Action with Projects A and B already open | Required scope |
| --- | --- |
| Change Global Units default | Future Project seeding only; A/B values, geometry and journals unchanged |
| Change A's Project Working Units | A's owned setting/context changes through its authorized path; no A geometry rescale, no B or Global mutation |
| Create C after changing Global defaults | Capture current eligible defaults once at actual genesis and retain immutable receipt |
| Reset A Working Units | A's immutable genesis/migration receipt; never today's Global values |
| Reset when required receipt unavailable | Typed unavailable reason; no guessed replacement |
| Change Publish presentation | Output/view context only according to its owner; not a replacement for board Working Units |
| Cancel numeric edit | No new remembered default, no settings write and no geometry write |

Project settings may themselves be authored Project state; zero geometry change
does not imply zero Project transaction. Keep that journal observation separate
from machine preferences and workspace-only changes. A missing active consumer
does not authorize reserved grid/autosave/library settings to become visible.

## Proof and owner-review work still required

Proof must correlate actual native input, visible/focus/accessibility states,
typed request, stable target IDs, canonical values, journal count/hash/revisions,
source-shard isolation and reopen results. Include all rows above, explicit
units and bare-context multi-surface parity, stale/mixed selection and pinned
library failures. Build/source/environment provenance and independent replay
are mandatory; screenshots, parser tests and clean unchanged reopen alone fail.

WDQ-F03 still needs a bounded visual-owner reconciliation list, exact consuming
owner handoffs and proposed prerequisite edits. Writer mechanism, native wire
gesture/formation, doorway design and fixture-specific latency budgets remain
explicit unapproved choices. This draft is not a complete WDQ-F02 handoff packet
and does not permit execution, delivery enrollment or successor selection.
