# Schematic Authoring Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012

## Driven by

- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified vocabulary and cross-doc invariants)
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
  (typed Operation/OperationBatch, single `commit()`, journal)
- `docs/decisions/PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
  (`ComponentInstance` join, relationship split, proposal-first ECO)
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
  (shared CLI/MCP/query/check/proposal/commit contract classes)
- `docs/decisions/PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
  (`CheckRun`/`CheckFinding`, waivers, proposal-first repair)
- Shared surface: `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (the seven
  shared operations — referenced, never restated here)

## Purpose & Scope

This contract specifies the minimal load-bearing tool set for authoring a
Datum schematic: placing and editing symbols, drawing connectivity, naming
nets, and reading electrical checks. It defines, per tool, the eight
implementation questions (manual UI action, Operation emitted, CLI command,
MCP/AI tool, AI query/context, validating checks, proof slice, and what is
explicitly not yet supported).

Scope is the per-sheet authoring surface over the single resolved
`DesignModel` at a known `model_revision`. It is deliberately NOT a
catalog of every native schematic verb that exists today. The current
engine exposes roughly 52 native-schematic CLI functions across three
mutation files; this contract collapses them to **ten** typed operations
by making kind, target, and edit-vs-create into parameters of existing
operations rather than separate tools.

Broader hierarchy authoring (sheet instances, sheet pins and cross-sheet net
rewiring), variant overlays and library object authoring (see `008` contract)
remain outside this ten-tool slice except for the separately bounded child-sheet
increments below. Native flat schematic document creation
already has a journaled engine/CLI operation; its ordinary GUI binding is a
separately bounded implementation increment.

### Bounded hierarchy inspection and navigation

`dat-sheet-interaction-reentry-9ee` reenters the historical OPEN-11 deferral for
read-only sheet-body and child-interface inspection and qualified navigation.
PM062 supplies occurrence-first discovery and opening; UVT §2.2.16 supplies the
typed source-backed interaction contract. Bodies use actual SheetInstance UUIDs;
interfaces preserve actual parent-port UUIDs and authored positions with explicit
instance membership and ordered occurrence qualification. Repeated definitions
and shared parent ports never collapse distinct placed memberships. Source,
definition and occurrence remain separate in inspection; missing or stale targets
show an unavailable reason. Body double-click descends and qualified breadcrumbs
return through existing shared panes. No source mutation is part of this command
family. Bounded implementation and independent ordinary GUI verification cover
selection, inspection, mouse/keyboard navigation and exact cold reopening without
design writes. Focused checks cover unavailable references; sheet/interface
authoring and cross-sheet rewiring are not supplied by that read-only increment.

Native schematic pane headers retain their own source and qualified occurrence
through focus changes and read-only navigation. The existing inline tool cluster
is shown only when both source and occurrence text fit beside it, using the same
font measurements and bounds as breadcrumb paint and pointer targets. Otherwise
the existing menu commands remain available and those header rows show context.
An unavailable pane scene is labelled unavailable from that pane's own source
lookup, independently of other visible schematics. The bounded `dat-child-pane-source-header-49i5`
App check covers Board/Root/Child focus, Fit and opening the Child again after
cold Project reopening, with authored sources and journal unchanged; this does
not add layout restoration, compact controls or a new responsive mechanism.

### Bounded blank child-sheet creation

`dat-schematic-create-child-sheet-g28d` configures `Place > Sheet`, the existing
name form, snapped placement preview and explicit Proposal review to create a
new blank child. The parent must be one complete unique occurrence; a unique
child can itself be that parent. Source, definition and placed instance are
created together through the existing three canonical operations, with stable
identities and one design acceptance. The parent and unrelated source stay intact.

Preview cancellation creates no design records. Dismissing a staged Proposal
retains its normal metadata/history but creates no sheet, definition or instance.
Undo/Redo removes/restores the complete accepted set, including both new files.
Proposal-only metadata preserves the authored revision after history; an actual
design change still invalidates stale acceptance. Destination collisions refuse
before acceptance without overwriting the existing file. Focused canonical and
real-project cold-history checks pass. Independent ordinary GUI checks cover
name/pose cancellation, Proposal dismissal, atomic acceptance, Undo/Redo, nested
creation after history, qualified navigation, Save and exact cold reopening. Existing-source reuse, repeated-parent editing, recursive subtree
creation, cycles, sheet pins and rewiring remain outside this increment. Other
authoring tools retain their actual qualified capabilities; no root substitution
makes an unsupported child edit available.

### Bounded circuit authoring inside a unique child

`dat-schematic-child-circuit-authoring-clq0` extends the existing Symbol picker,
Wire gesture and Reference/Value form to an exact unique child occurrence. The
shared captured basis carries its source, ordered path, pane and revision through
preview and invocation. Symbol and Wire requests include that path; engine
admission checks the complete sheet census, including empty sheets. Omitted paths
retain root compatibility and cannot silently address a child. Cancellation clears
only the captured preview, preserving the primary pane and canonical source.

Supported schematic-only component ownership retains the existing source-addressed
schema and shared repin/history operations. Inspection resolves a unique owner
only when all its Symbol members share the selected unique occurrence and no
physical counterpart is declared. Repeated sources and stale or mismatched paths
refuse. Other child tools, Part assignment, bulk editing, cross-sheet wiring and
physical correspondence are not enabled by this increment. Root document creation
and the delivered unique-parent child-sheet command remain available.

Focused engine and real-project protocol checks pass for bound placement, fields,
wire/history, refusal and pane isolation. Independent ordinary GUI verification
passed cancellation with no design writes, two bound symbols in a nested child,
Reference/Value editing, a pin-to-pin wire chain, four Undo/Redo steps, unchanged
parent/sibling sources, Save and exact cold reopening.

### Bounded explicit child interfaces

`dat-schematic-child-interface-binding-u073` adds Hierarchical Label placement
inside a unique child and real Port placement on the focused unique root sheet.
Selecting its child instance exposes Bind/Unbind through the shared choice form
and Proposal review. The review displays the parent/child source UUIDs, real Port
and Label UUIDs and occurrence paths; acceptance refuses until the complete card
fits. Escape cancels without changing the design.

Only explicit valid instance membership connects the real parent Port to the
exact-name child HierarchicalLabel. Unbound labels remain local and are diagnosed
with their source UUID and occurrence path. Unbind preserves both endpoints and
wire geometry. Existing typed operations and PM053 transitions preserve atomic
history; stale, wrong-path, ambiguous and repeated-source requests refuse.
No child Port placement, repeated-source editing, generated edge pins, new schema
or physical correspondence is enabled.

Focused engine/protocol/render checks and independent isolated ordinary GUI
verification cover placement, reviewed cancellation, Bind/Unbind, unrelated-net
preservation, Undo/Redo, Save and cold reopening with all 1,076 canonical files
exact. Retained run79 child placement evidence is paired with corrected run80
parent/interface evidence; run79's nonprimary-root Port refusal remains recorded.
Narrow-card acceptance refusal has automated coverage, without a narrow-viewport
visual claim. Other ERC findings and full qualification remain separate.

### Bounded Symbol manipulation inside a unique child

`dat-schematic-child-symbol-lifecycle-h3n1` admits existing Symbol Move,
Rotate/Mirror keys and Delete on an exact unique child occurrence. Acquisition,
preview, cancellation and failed-write restoration retain the captured source,
ordered instance path and pane. Shared Move geometry and canonical operations
remain the owners; other child object classes and repeated-source edits retain
their existing restrictions. Symbol-only Delete review uses the same Proposal
controls, with revision/path checks and existing dependency refusal.

Focused canonical tests cover connected child transforms, bound deletion,
unchanged parent source, stable pin/wire identities and cold history. Root and
child multi-gate tests verify surviving sibling membership, last-owner deletion
and child external-dependency refusal. Existing root group/pose regressions and
a real-project child preview/admission test pass.

Independent ordinary isolated GUI verification passed a composed Move/Rotate/
FlipX preview cancellation, acceptance and Undo/Redo on a bound nested-child
Symbol with its attached wire. Delete Proposal dismissal, acceptance, Undo/Redo
and restoration preserved the sibling and wire; Save and cold reopen retained
all 1,144 canonical files exactly. This is not a separate-transform or FlipY GUI
observation; child multi-gate behavior has focused canonical proof, not a GUI
fixture claim. Failed-write restoration has source review. Full hierarchy,
physical/ECO workflows and full qualification remain open.

### Bounded native Symbol alignment

The alignment increment in `dat-practical-schematic-capture-jsgv` adds
structured Horizontal Left/Center/Right and Vertical Top/Center/Bottom presets
for a complete homogeneous Symbol selection in one unique native occurrence.
Alignment uses authenticated placed library body extents and their union;
captions, pin labels, attached wires and anchor extrema are not the reference.
Supported body geometry currently includes rectangles, circles, round-capped
lines and zero-width polygons at cardinal poses. Missing or unsupported extents,
locks, stale source, fractional-nanometer translations or overflow refuse the
whole set. Distribution and other reference modes remain unavailable.

The existing native preview is configured as **Align Symbols** with fixed preset
geometry; coordinate editing, grid nudging and rotate/mirror changes are not
offered by that preview. Acceptance uses one canonical connected-geometry batch.
The bounded ordinary root-sheet App check covers pointer compound selection,
rendered menu traversal and acceptance, Runtime Cancel/history/Save actions,
unchanged-source cancellation and exact cold reopening. Current proof has 20
stages and five captures with all owned processes stopped. Engine and CLI checks
cover unequal body edges, attached contacts, full-scope refusal and guarded
writes. Inherited menu icon placeholders remain separately tracked; no full
pictogram, physical HUMAN, release-runtime or practical-goal completion is claimed.

### Ordinary root and child authoring — delivered development workflow

The fixed owner-selected workflow in `dat-ordinary-circuit-authoring-zvhw` has
composed development delivery: build a circuit, revise its symbols, wiring and
annotations, connect child sheets, and save/reopen it without arbitrary
differences between root and unique child occurrences. Cancellation and
canonical Undo/Redo remain required. Accepted root/child/nested authoring and
interface evidence includes one Save and cold reopening with seven sheet
objects, 1,543 canonical files and resolved nets exact; earlier receipts retain
their original candidates and limits.

The owner-reported connected-junction Select-drag failure was corrected in
`dat-schematic-connected-junction-drag-nmu2`, with copied-owner drag/cancel
and history proof. Save/cold reopening retained all 316 canonical files exactly;
authored NetIds and complete connectivity were preserved through the move.
Subsequent bounded wire editing, closed native Symbol interior picking and
source-qualified selection-cache reuse retain their separate accepted proof.
The recorded native Net query correction in
`dat-schematic-net-query-authored-identity-vbd3` preserves authored IDs and full
pin partitions across canonical geometry history without query adoption or
source writes. These corrections complete the fixed ordinary workflow at
development availability; they do not extend the earlier integrated GUI proof.
Broader owner usability QA, full schematic/release qualification, physical,
HUMAN and Wayland acceptance, and the larger native-library master-goal and
owner-only boundaries remain open. This is not editor production readiness.

Existing forms and gestures use one captured project/revision/source/ordered-path/
pane basis for unique root or child occurrences, including revalidation,
cancellation and failure restoration. The integration covers symbol fields,
compatible Part/Gate choices, named annotations, Power labels, Ports, Bus/member
forms and explicit markers through their existing operations and Proposal rules.
Bulk sheet annotation and Reference/Value tables must review and accept the same
complete qualified source they displayed; current receipts must identify that path.
The existing explicit interface-binding model also applies when a unique child
is itself the parent of another child. Repeated or ambiguous occurrences, stale
or wrong sources, locks, incompatible mappings, unsupported physical/cross-sheet
component ownership and explicit unbinding requirements remain meaningful limits.

The shared Move tool configures `datum.schematic.move` with the captured object,
ordered instance path, absolute destination and explicit `wire_edit` handle/action
for wiring. Wire endpoints reconnect deliberately; a Junction or contact vertex
reshapes incident spans while fixed attachments remain connected. An axis-aligned
selected span moves perpendicular to itself with fixed outer ends; free diagonal
spans use the requested displacement. Insert Bend, Split Wire,
Remove Segment and Remove Bend configure the same immutable geometry plan and
canonical operation batch. Free, 90-degree and 45-degree routing share the drawing
and editing path; Remove Last Vertex removes the last actual generated draft
vertex. Exact eligible pin and wire contacts take precedence over background grid
rounding. Preview and final click use the same destination and validation.

Select-drag acquires the qualified source handle after the existing drag threshold.
An unselected wire body does not start a rigid whole-wire move; a selected body
can acquire its span. A real Junction takes node precedence. Explicit Reconnect End
chooses one physical wire end even at a shared Junction. Existing unsplit source
wires expose derived contact handles without rewriting files on read or selection.
Topology edits retain the explicit-Junction deletion law; a split or off-axis
reshape that would erase an existing marker gate refuses. Bus Entry references,
locks, overflow, duplicate edges, lost fixed contacts and unintended foreign net
contacts remain guarded. Raw extensions, physical UUIDs and exact history inverse
must survive. Port Move retains its existing position-only `SetSchematicPort`
behavior and explicit membership; No-connect remains pin-owned.

The earlier wire-editing clipboard increment covers explicit Wire/Junction objects.
Copy is read-only; Cut uses reviewed canonical deletion. After the explicit
placement click, manual Paste and Duplicate assign fresh physical IDs through
one guarded direct batch. Common R/X/Y placement pose uses
checked coordinates. Existing selected wiring rotation/mirroring is a separate
shared Move configuration with an explicit pivot and fixed-contact policy: it
retains original UUIDs and raw data, never expands a derived Run into ownership,
and refuses an incomplete selection that would detach an unselected contact.
Both single edges and complete supported Wire/Junction compounds use the same
preview/canonical plan. Local visible manual placement and transforms commit
one design transaction without hidden Draft/Accepted bookkeeping in Undo.
Explicit Proposal consumers retain their immutable prepared receipts and review
policy; existing Proposal history is not rewritten. Captured Move exposes
Coordinates… with exact absolute/delta X/Y entry using captured Project units.
Valid values update the same guarded preview; invalid or refused values preserve
the preceding valid pose and editable text. Apply submits the same plan without
pointer snapping. Cancel/Escape closes the form and restores the preceding Move
preview; the next Escape cancels Move. With captured canvas ownership, plain
arrows displace the preview by the displayed exact current-grid increment; the
ordinary apply click commits it. Text, navigation, menu and pan owners retain
their inputs. Qualified selected wire grips distinguish physical ends, contact
vertices and existing Junctions, with the same captured handle used for pickup.
Component clipboard support, obstacle routing and full editor qualification were
outside that increment under `dat-schematic-wire-endpoint-edit-4368`.
The current practical-capture extension adds native same-project Symbol blocks
with explicitly selected internal Wires/Junctions to the shared Copy/Paste and
Duplicate path. Rectangle and lasso acquire the complete mixed block; Copy and
contextual Duplicate preserve that whole scope. Preview uses the captured
library body, a retained gesture seed and checked common R/X/Y pose. Fresh
Symbol/pin/field/component/binding identities and project-wide references are
allocated without copying old electrical or PCB ownership. Positioned custom
fields retain their values, visibility and relative geometry. The normal new
placement authority must still recreate the captured Part/Gate/Unit/Symbol
choices exactly; it never silently substitutes a newer source or clones an old
CI binding against current admission. Multi-gate/board-bound owners, nonstandard
owner roles or labels, local overrides, cross-project component payloads and
changed/unavailable dependencies refuse the entire block. Symbol Cut remains
unavailable; Wire/Junction placement and Cut retain their existing canonical
writers/contact rules, and ordinary Symbol Move retains its shared path.

The isolated ordinary-App exercise selected a two-resistor series block through
both rectangle and lasso, invoked production Copy/Paste shortcut actions and the
presented Duplicate command, cancelled without canonical writes, then accepted
Paste and Duplicate as separate atomic design entries. Undo/Redo and Save plus
an actual App restart retained six Symbols, three Wires, six schematic-only CIs
and three independently identified visible Tolerance fields. The retained A run
passed 18 stages in 18.51 seconds; the current wording-corrected B run passed the
same 18 stages in 9.34 seconds. Programmatic pointer input and shortcut-action
dispatch are not physical keyboard/HUMAN evidence; Cancel/history/Save used
existing Runtime commands. The preview now says “place copied block” and success
says “Copied block placed”; Wire-only wording is unchanged. All owned GUI,
daemon, clipboard and compositor processes stopped and runtime storage was
removed with no cleanup errors. That exercise alone did not establish
representative-circuit ERC composition or whole-goal completion.

### Representative amplifier integration

The subsequent ordinary-App integration revises a native common-emitter NPN
amplifier in a unique Child occurrence with Root input/output interfaces. Public
native writers prepare the circuit fixture; this setup is not manual GUI drawing.
Presented Root/Child navigation, bias-resistor alignment, complete block copying
and fresh project-wide references exercise the composed designer workflow.

Two production corrections support this case. Unordered foreign contact points
and segments receive a deterministic order before strict alignment-basis
comparison; duplicate geometry and the source-pin order used for wire movement
remain intact. Diagnostic markers over the schematic canvas yield to an active
schematic operation or Move preview, allowing a wire endpoint on a flagged pin.
Idle marker selection and diagnostic controls retain their existing routing.

The 37-stage J run passed in 51.71 seconds with ten reviewed captures. Default ERC
first reports the genuinely floating R4.2 terminal. A two-segment wire joins that
pin to the existing GND net; the old report visibly becomes stale. Wire Undo/Redo
precedes the new check-record transaction, then an explicit ERC rerun reports
zero findings, errors and warnings. Save, an actual App restart, Child navigation
and another explicit ERC run retain the repaired circuit and current clean result.
The cold Inspector reports 70 matching materialized sources. All owned GUI,
daemon, clipboard and compositor processes stopped without cleanup errors.

Input uses programmatic pointer events and production menu/shortcut dispatch;
history and Save use Runtime commands. Existing manual placement, wiring and
hierarchy proof retains its original scope. Physical HUMAN observation,
release-runtime execution and full qualification remain separate.

An interior scalar Label can slide along its existing wire while all incident
wires stay fixed and the complete physical connectivity partition is preserved.
Symbol-pin interior-contact refusal and existing endpoint Label/Power attachment
behavior remain unchanged.

Integrated source review, focused canonical/protocol regressions and independent
ordinary GUI input cover circuit creation and revision, field/annotation Proposal
review, Bus/member Entries, shared Move/history, and explicit interfaces including
a child acting as parent. One Save and cold reopening preserved all seven sheet
objects, all 1,543 canonical files and the resolved nets exactly. The
[consolidated delivery record](../reviews/workflow-delivery-rollout/native-schematic/ordinary-circuit-authoring-delivery.json)
binds the reviewed source, binaries, actual workflow evidence and retained failures.
This is programmatic ordinary GUI proof, not physical/HUMAN, performance or full
schematic qualification. The admission limits above remain in force.

The connected-junction correction retains the earlier proof at its original
scope. Independent copied-owner GUI input straightened the Q1 base branch by
moving only the junction and its incident endpoint; the through-wire and pins
stayed fixed. Escape preserved all 309 initial canonical files. Undo/Redo,
one Save and cold reopening preserved the final 316-file inventory exactly.
Direct resolved-model checks preserved all authored electrical identities,
NetIds, and complete logical and physical membership. The legacy anonymous
net query's coordinate-derived display name/UUID discrepancy is separately
tracked in `dat-schematic-net-query-authored-identity-vbd3`; those compatibility
fields are not evidence of semantic NetId stability. The delivery record's
correction entry retains the failed input/oracle attempts and actual proof.

### Current-engine grounding (verified)

Native schematic authoring has partially migrated onto the engine
OperationBatch/journal/ProjectResolver path. The old private JSON writer
inventory is retained below as historical scope evidence, but those families
are no longer allowed to be the active authority for migrated surfaces:

- `crates/cli/src/command_project_schematic_symbol_mutations.rs` (25 fns:
  place/move/rotate/mirror/delete symbol; set_/clear_ reference, value,
  lib_id, entity, part, unit, gate; set display_mode, hidden_power, pin
  override + clear; add/edit/delete field)
- `crates/cli/src/command_project_schematic_connectivity_mutations.rs`
  (16 fns: place/rename/delete label; draw/delete wire; place/delete
  junction; place/edit/delete port; create_bus/edit_bus_members;
  place/delete bus_entry; place/delete noconnect)
- `crates/cli/src/command_project_schematic_text_drawing_mutations.rs`
  (12 fns: text + drawing primitive families)

Current evidence: schematic symbol, connectivity, text/drawing, sheet
definition, sheet-instance, and hierarchy binding mutations route through
journaled native-project operations and resolver replay; canonical MCP
`datum.schematic.*` aliases bridge to the matching CLI commands. The
remaining product gap is not "no schematic substrate"; it is completion and
normalization of the contract shape: heterogeneous delete/set-field collapse,
binding evidence breadth, revision-keyed ERC/binding semantics, and
proposal boundaries for higher-level cross-domain edits. The first
contract-normalized schematic operation is now engine-owned:
`Operation::PlaceSchematicMarker { marker_kind:
Junction|NoConnect, ... }` writes the same sheet maps as the legacy
junction/no-connect operations, replays from the journal, and generates
undo inverses; the compatibility CLI verbs `place-junction` and
`place-noconnect` now author that normalized operation. Pool-backed
`place-symbol` now mints an authored symbol-first `ComponentInstance` when the
pool symbol resolves to a unique entity/gate/compatible part. It reports
explicit status for compatibility `lib_id`s, unresolved/ambiguous entity gates,
bound-without-part, ambiguous compatible parts, incompatible parts, and
bound-with-part; board-package binding remains partial.

The bounded standalone Text slice (`dat-schematic-text-krdm`) uses the existing
Place Text action, shared content/rotation form, selected Inspector edit and
shared Move/Delete paths. Notes have independent absolute anchors, preserve
literal single-line content and raw source extensions, and use canonical
journaled operations with cancellation, Undo/Redo and reopening. Ordinary note
text defaults to 1.27 mm, classified as a routine visual choice under PM059 in
that task; this does not change the Rendering Book's title/fabrication-note
ladder or add a parent tether. New rotations are cardinal; translation preserves
an unchanged imported angle. Multiline editing, typography controls, repeated
source edits and broader hierarchy authoring remain outside this slice.

The bounded root-admission correction (`dat-schematic-nonprimary-root-tools-o800`)
allows existing Text creation/editing, Bus creation/member editing and member-Wire
entry controls on any uniquely qualified root sheet. These paths reuse the
focused source and pane-scoped preview; generic tool admission is unchanged.
That increment excluded child and repeated-source editing. Text failure restoration
retains the captured pane basis instead of substituting the primary scene.

Focused real-project checks cover root requests, pane isolation, cancellation
and stale-selection refusal. Independent ordinary isolated GUI verification
passed Text creation/editing and both cancellation paths, Text Undo/Redo, Bus
creation/cancellation, explicit member Entry creation/cancellation and refused
undeclared-member attachment, Entry Undo/Redo, Save and cold reopening with all
1,105 canonical files exact. Only the intended pin group joined the chosen
member; unrelated nets and other root/child sources remained unchanged. The
existing Bus member form was inspected and canceled after reopening; changed
Bus-member submission and failed-edit restoration have source review only in
this increment. The long refusal suffix was clipped (`dat-bus-entry-refusal-details-bq5n`);
no complete-message visual or full qualification claim is made.

The bounded child Text placement extension (`dat-child-text-placement-d7ht`)
uses the same Text content/rotation form in an exact unique child occurrence.
Its captured source, ordered instance path and pane own both the immutable
preview and canonical placement request; cancellation clears that preview without
substituting the primary sheet. Literal single-line content, cardinal rotations
and the existing default text size retain their root semantics. Repeated sources,
stale bases and mismatched occurrences refuse. This increment enables placement
only: child Text editing, Move/Delete and other child tools retain their existing
restrictions. Acceptance evidence is recorded in the task; this scope definition
does not establish full hierarchy or schematic qualification.

Confirmed IR facts the matrix relies on:

- `LabelKind` (`Local|Global|Hierarchical|Power`) is already a field on
  `NetLabel` (`crates/engine/src/schematic/mod.rs:145`). Label kind is a
  parameter, not a reason for separate tools.
- `BusEntry` carries `bus: Uuid` + optional `wire: Uuid` + position
  (`schematic/mod.rs:168`). It is bus-bound, not a free point marker.
- `Schematic.sheet_instances: HashMap<Uuid, SheetInstance>` exists in the
  IR. Current native builders in `api/native_write/schematic_sheets.rs`
  author definitions, instances and bindings through canonical journaled
  operations. Complete ordinary GUI hierarchy authoring remains pending.
- ERC runs over an in-memory `Schematic`
  (`crates/engine/src/erc/mod.rs:50`,
  `run_prechecks_with_config_and_waivers`): 7 rules + a
  hierarchical-connectivity-mismatch check
  (`erc/mod.rs:291`), with severity overrides and waivers. Native check
  results have revision-qualified `CheckRun` records and stable finding
  fingerprints; the in-memory checker is not the persistence boundary.
- `ProjectResolver` and resolver-materialized schematic read/write paths exist
  for the current native-project slices; `PlacedSymbol` still carries a
  reference string + optional part/entity/gate UUIDs rather than a guaranteed
  minted `ComponentInstance` join.
- MCP exposes canonical schematic write aliases for the current journaled
  slices. These are still mostly compatibility-shaped verbs rather than the
  normalized ten-operation contract below.

## Reference-Tool Survey

**Altium Designer (primary reference).** Symbols place/move/rotate from
the Libraries panel; `Place > Wire | Bus | Net Label | Port | Sheet
Symbol | Power Port`; junctions auto-created at T-intersections;
`Tools > Annotate`; ONE SCH List/Inspector property surface drives batch
field edits; `Tools > Electrical Rules Check` with an ERC config table;
ECO is the gate for any schematic→PCB change. **Load-bearing:** one
selection+property model, ECO as the cross-domain gate, a single ERC
config table. **Ceremony Datum omits:** separate place-wire vs place-line
modes, Smart-Paste variants, per-property menu items.

**KiCad 8 (eeschema).** `Place Symbol | Power | Wire | Bus | Label |
Global Label | Hierarchical Label | Sheet | No-connect | Junction`;
junctions mostly implicit; a Symbol Fields Table for bulk field edit;
hierarchical sheets with sheet pins. **Load-bearing:** implicit junction,
global/hierarchical label net semantics, sheet-pin ↔ hierarchical-label
matching. **Ceremony:** three separate label tools instead of one
label-kind parameter; separate Junction and No-Connect placement tools
that are both zero-dimension single-point markers.

**OrCAD/Cadence Capture.** `Place Part | Wire | Bus | Net Alias | Power |
Ground | Hierarchical Port | Off-Page Connector`; netlist handoff to PCB
Editor; DRC. **Load-bearing:** off-page connector as a cross-sheet net
join, occurrence-vs-instance hierarchy model. **Ceremony:** heavy modal
property editor, session-log checks.

**Horizon EDA.** Per-block schematic; net lines + net labels + bus
rippers + power symbols; implicit junctions; ERC inside the rules engine;
every edit is a tool producing a document delta. This is the lean model
closest to Datum's operation-first intent. **Load-bearing:**
block/instance hierarchy, net-tie, pool-backed symbols.

**Lean rationale.** Across all four tools the load-bearing primitives are:
one selection+property surface (not per-field menu items), kind as a
parameter on label/marker placement (not a tool per kind), and ECO as the
gate for changes that reach the board. Datum already has `LabelKind` in
the IR, so the per-kind tool fan-out is pure ceremony. The single
principle this contract applies consistently is **parametric kind +
optional-id edit-vs-create + heterogeneous id lists** — the same principle
the reference tools converge on through their Inspector/Properties and
implicit-junction behavior.

## Tool Inventory

Each tool answers the eight questions as labeled fields. Common reads,
session/context bootstrap, the proposal lifecycle, the single `commit()`,
and global undo/redo are defined ONCE in the shared surface (see
**Shared Surface** below) and are referenced by name here, never
restated.

### 1. place-symbol

- **(1) Manual UI:** Normal component placement selects a reusable Part with
  its declared Value, Package, default Footprint and complete default PinPadMap;
  then choose the gate, reference and position on the active sheet. Value belongs
  to the selected component, rather than an independent placement edit. Explicit
  bare-Symbol placement retains its provisional/specialist scope.
- **(2) Operation:** `PlaceSymbol{sheet_id, library_ref, position,
  rotation, mirror, component_instance_id}` → one atomic
  `OperationBatch` through `commit()`. Current CLI uses the journaled
  schematic operation path and, for pool-symbol UUID `lib_id`s that resolve to
  a unique entity/gate/part, batches `CreateSchematicSymbol` with a symbol-first
  `CreateComponentInstance`.
- **(3) CLI:** `datum-eda project place-symbol <dir> --sheet <uuid>
  --reference R1 --value 10k --lib-id <id> --x-nm <nm> --y-nm <nm>
  --rotation-deg <deg> [--mirrored]`
- **(4) MCP/AI:** `datum.schematic.place_symbol` exists for the current
  symbol lifecycle slice.
- **(5) AI query/context:** Resolved `DesignModel` at a known
  `model_revision`: pool library refs, existing `sheet_id`s, occupied
  grid positions, the current reference-designator set (collision
  avoidance), target net intent. A `ComponentInstance` id is minted and
  carried so `PlacedSymbol` ↔ `PlacedPackage` can join later. Reads fold
  under the shared `datum-eda query` namespace
  (`library.query`/`schematic.query`).
- **(6) Validating checks:** ERC `unconnected_component_pin` once pins
  exist; reference-designator uniqueness; `lib_id` resolvability against
  the pool; post-commit `model_revision` recompute.
- **(7) Proof slice:** On `datum-test`, place one R symbol on the main
  sheet; assert `OperationResult.created` carries the symbol UUID + a
  freshly minted `ComponentInstance` id, the journal gained one
  `TransactionRecord`; Undo removes the placed domain objects through an
  append-only compensating transaction. The restored geometry is not a rewind
  of journal or revision identity; Redo is another recorded transaction.
- **(8) Not yet supported:** Compatibility/non-pool symbols do not mint
  ComponentInstances; in Symbol-only mode ambiguous/unresolved library bindings remain
  non-ComponentInstance placements, and board package refs attach only after
  the physical package exists.

The bounded initial Part/Gate increment (`dat-explicit-part-gate-placement-b43w`)
configures this same picker with separate named Part/gate rows alongside the
existing Symbol-only rows. Manufacturer/MPN, gate, Entity and Symbol labels
distinguish choices that share a real library Symbol UUID. The initial reference
prefix comes from the Entity and the value from the Part. The ghost retains the
chosen Part/gate context. No preferred Part or gate is inferred for an explicit
choice; Symbol-only selection retains its existing inference semantics.

The existing `datum.schematic.place_symbol` verb accepts the captured explicit
intent and independently checks the Part → Entity → gate → Unit/Symbol chain,
source identities and referenced revisions. It composes the existing schematic
Create and authored component binding in one canonical transaction, preserving
the existing component-ID derivation and Part binding role. Valid multi-gate maps
remain unchanged. Missing, incompatible, ambiguous or stale explicit intent
refuses without writes and never falls back to inferred binding.

Focused checks and the isolated ordinary App prove named choice, ghost/Cancel,
placement, Undo/Redo, Save and cold source reopening on a root and a uniquely
qualified child. Exact chosen identities, pinned component data, old schematic
payloads, library/static bytes and schematic net IDs/member runs are retained.
Canonical PM053 lineage metadata follows its recorded before/after states.
This is debug App pointer-callback/menu-dispatch proof; visual observations cover
the picker, ghost and placed states, with stored identities checked after cold
reopening. It is not physical/HUMAN or release-GUI qualification. Initial creation
of a new component is the scope; existing Assign Part and Place another Gate
remain on their current paths. Repeated/shared occurrences, cross-sheet component
grouping, placed ECO, map/library updates, approval and full qualification remain
outside this increment.

The component-first follow-up (`dat-component-first-capture-3btd`) configures the
same placement verb with captured complete-component intent. The engine checks
the declared logical and physical graph and source revisions; an incomplete or
stale choice refuses without falling back to a bare Symbol. Partial library
definitions remain editable, but do not qualify for normal complete placement.
An empty Part pool offers the existing Library acquisition path for a reusable
native Part; its supported source profile and identity/refusal rules are in the
Library Authoring Tool Contract. Initial PCB handoff consumes the established
physical choice and checks its authenticated Part-adoption source basis; pose
changes do not adopt later library definitions. A changed map, footprint or
padstack requires a reviewed update, including when the caller supplies the same
map explicitly. This adds no new persisted binding role or silent propagation.
Source `54b4413b` has 43 passing focused engine checks and an isolated ordinary
debug App run of 25 stages at 1344×1000 with current release CLI/daemon companions.
The synthetic two-component fixture covers acquisition Preview/Cancel/Add,
component selection and placement history, reviewed wired-Symbol assignment,
declared read-only transfer, Save and cold reopening. Earlier Part/Gate and
bare-Symbol receipts retain their original scope. This establishes bounded
development behavior, not component-workflow release-GUI execution, physical/HUMAN
input, manufacturer pinout qualification, owner-circuit migration or full
qualification.

### 2. transform-symbol (move / rotate / mirror unified)

- **(1) Manual UI:** Select symbol(s); drag to move, R to rotate, X/Y to
  mirror; release commits a local undoable edit.
- **(2) Operation:** `TransformSymbol{symbol_id, position?, rotation?,
  mirror?}` (one op, three optional params) → direct local `commit()`
  (local/visible/undoable, direct-commit class). Today three separate
  fns (move/rotate/mirror_native_project_symbol) now use the journaled
  schematic operation path.
- **(3) CLI:** Current CLI keeps separate `move-symbol`, `rotate-symbol`,
  and `mirror-symbol` commands.
- **(4) MCP/AI:** `datum.schematic.move_symbol`, `rotate_symbol`, and
  `mirror_symbol` exist for the current transform slice.
- **(5) AI query/context:** `symbol_id`, current position/rotation/mirror
  (`schematic.query` symbols), grid, sheet bounds, attached wire
  endpoints that must stay connected.
- **(6) Validating checks:** Geometry on grid; wires re-evaluated by
  derived connectivity (`crates/engine/src/connectivity/mod.rs`); no ERC
  severity change required to commit (local edit).
- **(7) Proof slice:** On `datum-test`, move a symbol +1mm, rotate 90,
  mirror; assert `OperationResult.modified = symbol uuid`, undo restores
  the exact prior `PlacedSymbol` bytes, derived `net_info` recomputes
  identical net membership.
- **(8) Delivery boundary:** A compatible Symbol selection uses the existing
  drag, M and selected-symbol menu Move paths, with R/X/Y and one atomic edit
  (`dat-schematic-group-transform-7a5p`). The pickup point specified by UVT
  section 2.2.12 is the displayed common group reference, translated with the
  destination. Attached wires are updated once against the complete final
  geometry; owned fields follow their Symbols. Cancel restores the source.
  Unsupported members, locks, repeated occurrences and unsupported fixed
  contacts refuse the complete group. The existing native verb retains its
  singleton absolute-pose form and adds a distinct group-delta form. Supported
  schematic-only, single-Symbol `ComponentInstance` bindings are now preserved
  by the singleton/group transform implementation in the same atomic pose,
  attached-wire and history transaction (`dat-schematic-bound-move-p8ks`).
  Supported same-sheet multi-Gate component owners also retain all sibling bindings
  through these transforms (`dat-0fom`); physical-package and stale bindings refuse
  the complete edit. Occurrence/contact limits are unchanged. Earlier independent
  GUI verification covered single-Symbol bindings during singleton/group transforms,
  attached wires, Undo/Redo and reopening. Multi-Gate GUI checks additionally covered
  new-member Move, sibling preservation, history and cold reopening (`dat-0fom`).
  Public CLI normalization and broader occurrence/contact transforms remain pending.

### 3. delete-object (any schematic object, by id list)

- **(1) Manual UI:** Select any schematic object, press Delete; removes it
  as a local undoable edit.
- **(2) Operation:** `DeleteObjects{ids:[ObjectId]}` (one op,
  heterogeneous id list) → `OperationBatch`. Batch/cross-domain deletes
  (e.g. a symbol with a bound board `ComponentInstance`) escalate to a
  Proposal. Current type-specific native delete builders emit canonical
  journaled operations; the normalized heterogeneous surface remains a target.
- **(3) CLI:** `datum-eda project delete --root <dir> --id <uuid>
  [--id <uuid>...]` (collapses delete-symbol/-wire/-junction/-label/
  -port/-bus/-bus-entry/-noconnect/-text/-drawing).
- **(4) MCP/AI:** `delete_objects` (not present). Returns
  `OperationResult` diff(deleted); a Proposal when deletion crosses into
  placed-package territory.
- **(5) AI query/context:** Object ids and their types (recoverable from
  the id in the resolved model); whether any deleted symbol has a bound
  `ComponentInstance` with a `PlacedPackage` (forces a Proposal); dangling
  wire/junction cleanup implications.
- **(6) Validating checks:** Post-delete ERC for newly
  unconnected/dangling objects; `ComponentInstance` orphan check;
  `model_revision` recompute.
- **(7) Proof slice:** On `datum-test`, delete one symbol + its attached
  wire in one batch op; assert two deleted refs, undo restores both
  atomically; deleting a board-bound symbol instead returns a Proposal.
- **(8) Delivery boundary:** Supported multi-object schematic selections use
  an inline Proposal preview, explicit Tab acceptance or Escape dismissal,
  and one undoable design deletion (`dat-schematic-batch-delete-m8zf`). The
  complete selected set is validated; owned children are deduplicated and
  surviving external dependencies refuse the operation. Dismissal preserves
  design source; Proposal review metadata may be journaled. Supported single-Symbol,
  schematic-only component bindings now use the same review, removing the owned
  binding and no-connect markers atomically while preserving unrelated wires and
  library source (`dat-schematic-bound-delete-ahmx`). Undo/Redo and reopening
  restore exact identities and valid binding pins. Supported same-sheet multi-Gate
  deletion retains surviving siblings in their current owner, removing the owner only
  with its last Symbol (`dat-0fom`). Members must share one unique source sheet
  and qualified occurrence path, empty or existing nonempty; this adds no hierarchy
  navigation. Physical or override bindings, stale or repeated sources, external
  dependencies, hierarchy-bound Ports and shared Bus restrictions remain. The public normalized id-list surface, remaining
  object classes and board-bound escalation are still pending.

### 4. draw-wire

- **(1) Manual UI:** `Place > Wire`, click start and vertices to build an
  uncommitted chain; explicit Finish/double-click commits the whole chain.
  Escape cancels the uncommitted chain with no design writes, preserving prior
  accepted wires. An authored endpoint on an existing segment creates an explicit
  junction in the same atomic batch; a mere crossing or overlap does not join.
  This applies the NCC-C03 D1/D2 PM clarification recorded on 2026-11-04 in
  `dat-schematic-connectivity-spec-copy-divergence-form-qkmo`; it is the required
  behavior, not a claim that the complete gesture is implemented.
- **(2) Operation:** `DrawWire{sheet_id, points:[Point], net_hint?}` →
  `OperationBatch`; co-emits implicit `PlaceMarker{kind:Junction}` at
  T-intersections in the SAME batch (one undo entry). The native
  `datum.schematic.draw_wire` path accepts a qualified polyline and batches
  endpoint-to-segment Junctions. The legacy CLI `draw_native_project_wire`
  remains a journaled straight-segment adapter without Junction inference.
- **(3) CLI:** `datum-eda project draw-wire <dir> --sheet <uuid>
  --from-x-nm <nm> --from-y-nm <nm> --to-x-nm <nm> --to-y-nm <nm>`
- **(4) MCP/AI:** The native `datum.schematic.draw_wire` service accepts the
  qualified polyline and returns its atomic batch, including required Junctions;
  `delete_wire` uses the shared deletion path. This does not claim that every
  public compatibility surface exposes the complete native gesture.
- **(5) AI query/context:** Sheet pin coordinates and existing
  wire/junction geometry (`schematic.query`
  wires/junctions/symbol_pins) so endpoints snap to pins and
  T-intersections auto-junction; target net name if labeling intent is
  known.
- **(6) Validating checks:** Endpoint-on-pin/grid; derived connectivity
  recompute merges/splits nets; ERC `output_to_output_conflict` / driver
  rules over the changed net.
- **(7) Proof slice:** On `datum-test`, draw a wire joining two symbol
  pins; assert derived `net_info` shows both pins on one net, an
  auto-junction op was batched at any T, undo restores prior
  connectivity.
- **(8) Not yet supported:** Arc/obstacle routing and full compatibility-surface
  parity remain open. The legacy straight-segment CLI does not infer Junctions.

### 5. place-label (kind: local | global | hierarchical | power)

- **(1) Manual UI:** `Place > Net Label` (or Global/Hierarchical Label,
  or Power Port); click on a wire/pin and type the net name.
- **(2) Operation:** `PlaceLabel{sheet_id, position, text, kind:
  Local|Global|Hierarchical|Power}` → `OperationBatch`. `LabelKind`
  already exists in the IR, so kind is a field, not separate tools.
  Rename = `SetObjectField` on label text; delete = `DeleteObjects`.
- **(3) CLI:** `datum-eda project place-label --root <dir> --sheet <uuid>
  --x <nm> --y <nm> --text <net> --kind <local|global|hier|power>`
- **(4) MCP/AI:** `datum.schematic.place_label`, `rename_label`, and
  `delete_label` exist for the current label slice. `place_label` uses one
  kind enum instead of separate local/global/hierarchical/power tools.
- **(5) AI query/context:** Existing net names (`schematic.query`
  nets/labels), wire endpoints to attach to, hierarchy port names a
  hierarchical label must match, power-net conventions (GND/+3V3).
- **(6) Validating checks:** Derived connectivity re-resolves net
  membership/merge (lowest `NetId` survives); ERC
  `power_in_without_source`, hierarchical-connectivity-mismatch;
  `unconnected_interface_port` if a hierarchical label has no matching
  explicit parent binding. Under the PM059 disposition in
  `dat-schematic-child-interface-binding-u073` and PM053's optional-interface
  amendment, an unbound child HierarchicalLabel is valid occurrence-local source,
  reported as unbound with its UUID/path. Only explicit valid instance membership
  joins a real parent Port to one exact-name child label. Unbind retains both
  endpoints; invalid referenced bindings still refuse atomically. This bounded
  workflow uses existing controls and Proposal review; focused checks and
  independent ordinary GUI binding/history/cold-reopen verification pass.
- **(7) Proof slice:** On `datum-test`, add a global label VCC on a pin;
  assert that pin's net merges into VCC in derived `net_info`,
  `power_in_without_source` clears if a source is present, undo reverts
  net membership.
- **(8) Delivery boundary:** For a uniquely qualified source occurrence, native
  GUI creation, Move/Delete and Inspector name/kind editing use canonical writes
  and journal history. Editing previews an immutable replacement at the existing
  position, with Apply/Cancel and exact undo/redo/reopen (delivery
  `dat-schematic-label-edit-yx35`). Context-menu entry remains separate work;
  contract-shaped result/normalization coverage remains pending. Label kind
  remains one parameterized capability. Existing Label/Power Move preserves raw
  label and attached-wire metadata, including nested point extensions, through
  cancellation and canonical history (`dat-schematic-label-move-payload-h1m9`).
  This does not claim the same preservation for every label/port field writer.

### 6. place-marker (kind: junction | no-connect)

- **(1) Manual UI:** `Place > Junction` at a wire crossing to force a
  connection; `Place > No-Connect` on an intentionally open pin.
- **(2) Operation:** `PlaceMarker{sheet_id, position, kind:
  Junction|NoConnect}` → `OperationBatch` (local edits). Junction is also
  auto-emitted at exact T contacts inside `DrawWire` batches. Implemented substrate slice:
  `Operation::PlaceSchematicMarker { sheet_id, marker_id, marker_kind,
  marker }` is the journal vocabulary for both point-marker placements.
  It persists into either the `junctions` or `noconnects` sheet map based on
  `marker_kind`, replays through `ProjectResolver`, and undoes by emitting the
  matching legacy delete inverse.
- **(3) CLI:** `datum-eda project place-marker --root <dir> --sheet <uuid>
  --x <nm> --y <nm> --kind <junction|noconnect>` is the normalized target.
  Current compatibility commands `place-junction` and `place-noconnect` remain
  public but now author `PlaceSchematicMarker` through the engine write
  facade.
- **(4) MCP/AI:** `place_marker` (not present). One tool, kind enum;
  returns `OperationResult`. Current MCP compatibility aliases still expose
  `datum.schematic.place_junction` and `datum.schematic.place_noconnect`.
- **(5) AI query/context:** For junction, wire-crossing coordinates
  (`schematic.query` wires/junctions). For no-connect, pins intended to
  be left open (`schematic.query` symbol_pins). Same point-placement
  query shape, parameterized by kind.
- **(6) Validating checks:** `noconnect_connected` ERC (NC on a connected
  pin = warning); `unconnected_component_pin` suppressed where NC is
  present; derived connectivity merge at a junction.
- **(7) Proof slice:** On `datum-test`, place-marker kind=noconnect on a
  deliberately open pin; assert `unconnected_component_pin` for that pin
  clears while `noconnect_connected` stays absent, undo restores the
  warning. Then place-marker kind=junction at a T; assert net merge.
- **(8) Delivery boundary:** Native GUI placement uses the normalized journal
  operation through `native.write`; preview/cancel, singleton deletion and
  history/reopen are also delivered (`dat-schematic-marker-tools-jwiv`).
  Preview/cancel writes nothing; deletion uses the canonical delete operations.
  Junction acquisition uses
  exact nearby wire contacts, including off-grid crossings; ambiguous or
  unrepresentable intersections are refused rather than rounded. No-Connect
  requires an exact unique pin. Normalized public CLI/MCP entry points and
  contract-shaped `OperationResult` remain target work; native-write registry
  availability does not establish those public entry points.

### 7. place-port (create-or-edit, hierarchical / interface port)

- **(1) Manual UI:** `Place > Port` or a sheet-pin on a hierarchical
  sheet symbol; set name + direction; double-click to edit.
- **(2) Operation:** `PlacePort{port_id?, sheet_id, position, name,
  direction}` → `OperationBatch`. With no `port_id` it creates; with a
  `port_id` it edits (collapses the separate edit verb, mirroring
  `TransformSymbol`'s optional-param edit shape). Current CLI retains
  separate journaled create/edit/delete commands.
- **(3) CLI:** `datum-eda project place-port <dir> --sheet <uuid>
  --x-nm <nm> --y-nm <nm> --name <id> --direction <input|output|...>`;
  `edit-port <dir> --port <uuid> [--name ...] [--direction ...]`; and
  `delete-port <dir> --port <uuid>`.
- **(4) MCP/AI:** `datum.schematic.place_port`, `edit_port`, and
  `delete_port` exist for the current port slice.
- **(5) AI query/context:** Parent-sheet hierarchical-label names this
  port must mirror, existing ports (`schematic.query` ports), direction
  semantics for ERC.
- **(6) Validating checks:** Hierarchical-connectivity-mismatch (parent
  label set vs child port set), `unconnected_interface_port`; derived
  connectivity refresh.
- **(7) Proof slice:** On `datum-test` (or a hierarchy fixture), add a
  port matching a parent hierarchical label; assert the
  hierarchical-connectivity-mismatch count drops, undo restores it; then
  re-issue with `--port` to rename and assert a single modified diff.
- **(8) Not yet supported:** Complete sheet-symbol/sheet-pin authoring,
  hierarchy rewiring, double-click port editing, and normalized public CLI/MCP
  entry points remain open. Native `Place > Port` now configures the shared
  name/direction form and immutable pin/grid preview. Inspector editing preserves
  the selected port's identity and position, with one Apply/Cancel action; native
  `place_port` accepts an optional existing port and qualified occurrence path.
  Singleton deletion, undo/redo and reopening use the canonical journal.
  Deleting a port still bound to a sheet instance is refused until its binding
  is explicitly removed. This bounded interface-port workflow does not establish
  complete hierarchy authoring or full schematic qualification.

### 8. create-bus (members + bus-entry as parameters)

The PM053 local scalar amendment is ratified for `dat-schematic-new-bus-members-a67z`:
new names use first-class nongraphical source declarations and explicit member entry
bindings, with no implicit label union. Unique root-sheet scope, collision admission,
member/dependent-entry refusal, lifetime and canonical history follow that amendment.
The bounded native workflow is implemented and independently verified through
ordinary GUI creation, explicit member connection, cancellation/refusal, history
and exact cold reopening. This does not establish complete schematic qualification.

- **(1) Manual UI:** `Place > Bus`, draw it; `Place > Bus Entry` to tap a
  member wire onto it; bus members named D[0..7].
- **(2) Operation:** `CreateBus{bus_id?, sheet_id, points, members:[net],
  entries:[{position, wire?}]}` → `OperationBatch`. Members and bus
  entries are parameters of the bus op (`BusEntry` carries `bus:Uuid` +
  optional `wire` — it is bus-bound, NOT a free point marker, so it stays
  in the bus tool, not place-marker). With a `bus_id`, edits
  members/geometry (collapses edit-bus-members). Current CLI retains
  separate journaled bus and bus-entry commands.
- **(3) CLI:** `datum-eda project create-bus <dir> --sheet <uuid> --name
  <bus> --member <net>...`; `edit-bus-members <dir> --bus <uuid>
  --member <net>...`; `place-bus-entry <dir> --sheet <uuid> --bus <uuid>
  --x-nm <nm> --y-nm <nm> [--wire <uuid>]`; and `delete-bus-entry <dir>
  --bus-entry <uuid>`.
- **(4) MCP/AI:** `datum.schematic.create_bus`, `edit_bus_members`,
  `place_bus_entry`, and `delete_bus_entry` exist for the current bus slice.
- **(5) AI query/context:** Existing net names that should become bus
  members (`schematic.query` nets/buses), bus geometry, entry tap points
  on member wires.
- **(6) Validating checks:** Derived connectivity expands the bus into
  member nets; ERC over each member net; bus-member name well-formedness.
- **(7) Proof slice:** On `datum-test` (or a bus fixture), create bus
  D[0..3] with one entry tapping a wire; assert derived `net_info` shows
  4 member nets and the tapped wire joins D0, undo reverts.
- **(8) Delivered scope and remaining limits:** Native `Place > Bus` uses the
  shared polyline gesture and explicit names of existing scalar Nets or previously
  unused names. New names create distinct nongraphical scalar declarations;
  ambiguous names and multiple names resolving to one scalar Net refuse atomically.
  Native `create_bus` accepts an optional existing bus and member/geometry/entry
  parameters. Bus Entry offers an existing-wire choice or an explicit declared
  member and binds a wire endpoint to an exact bus contact without shorting other
  scalar members. The Bus Inspector edits name/members and deletes qualified owned
  entries. Deleting a drawing removes its spine and owned entries in one journal
  transaction while retaining scalar source and semantic Bus lifetime. Escape and
  Cancel discard uncommitted Wire/Bus previews without writes; Finish, Enter and
  double-click retain explicit completion. Undo/redo and cold reopening preserve
  canonical identities and connectivity. Shared-declaration/hierarchy editing and
  normalized public CLI/MCP mutation adapters remain open
  (`dat-bus-public-member-parity-426y`); the older public Bus constructors do not
  yet expose this native GUI member-creation/selection behavior. Native Nets/ERC
  reads retain declaration source and recorded NetIds, refusing incomplete scalar
  adoption rather than fabricating identities. This is bounded native delivery,
  not complete bus/hierarchy authoring or full qualification.

Each semantic Bus may retain `scalar_declarations`, a set of qualified declaration
source occurrences (old absence defaults to empty; empty sets are omitted). A
declaration-backed member follows its declaration's final scalar group across
merges and every split, including last-entry removal and zero drawing lifetime.
`scalar_nets` records the resolved membership, not competing declaration authority.
Ordinary members retain existing PM053 remapping; remove only the pre-state
declaration-backed memberships before composing their final groups, preserving
other members. Net nomination and identity survival are unchanged. Missing,
wrong-context, ambiguous references or disallowed member collapse refuse. Multiple
Buses independently follow the same declaration; member removal drops only that
Bus's source binding. No names or surviving-entry heuristics reconstruct bindings.

### 9. set-symbol-field (reference/value/lib_id/part/entity/unit/gate/...)

- **(1) Manual UI:** Double-click a symbol or use the
  Inspector / Symbol Fields Table to edit reference, value, footprint,
  custom fields; assign part/library.
- **(2) Operation:** `SetSymbolField{symbol_id, field, value, clear?}` as
  ONE parametric op. The field enum covers
  `reference|value|lib_id|part|entity|unit|gate|display_mode|hidden_power|
  custom:<name>`. Current compatibility set/clear and add/edit/delete field
  builders use canonical journaled symbol operations; the normalized field
  surface remains a target.
- **(3) CLI target inventory:** `datum-eda project set-symbol-field --root <dir> --symbol
  <uuid> --field <reference|value|lib-id|part|entity|unit|gate|
  display-mode|hidden-power|custom:NAME> --value <v> [--clear]`
- **(4) MCP/AI:** `datum.schematic.set_symbol_field` exposes Reference/Value
  through one field-parameterized CLI bridge. The current response is the canonical
  commit report with transaction diff and journal length; normalized
  `OperationResult` and remaining selectors stay targets. Existing CLI/MCP
  reference/value and Custom commands remain compatibility paths.
- **(5) AI query/context:** Current symbol fields/semantics
  (`schematic.query` symbol_fields/symbol_semantics/symbol_pins), pool
  part/entity ids for assignment, reference uniqueness, gate/unit
  enumeration for multi-unit parts.
- **(6) Validating checks:** Reference uniqueness; part/entity/gate
  resolvability against the pool; pin-semantics recompute (drives ERC
  driver rules); `model_revision` recompute. Binding a part establishes
  the `ComponentInstance` → library join.
- **(7) Proof slice:** On `datum-test`, set R1 value 10k→4k7 and assign a
  pool part; assert a single `OperationResult.modified`, undo restores
  both, `symbol_semantics` reflect the new pin types.
- **(8) Delivery boundary:** The selected-Symbol form supports reference/value
  and UUID-qualified custom metadata add/edit/remove through shared controls
  (`dat-schematic-custom-fields-3bec`). New metadata starts hidden and unpositioned.
  Show/Hide and explicit Place/Move text controls author visibility and position
  (`dat-schematic-field-presentation-jvu3`); Show without a position enters
  placement rather than inventing one. Text remains owned by its Symbol, with a
  selection/gesture tether. Supported parent transforms carry positioned fields,
  including hidden ones, without extending occurrence/contact admission.
  Custom-field, reference/value and parent-transform writes preserve unrelated
  raw symbol/pin/field payload. Cancel/no-op do not write; accepted changes use
  canonical history.

  The existing native daemon `datum.schematic.set_symbol_field` verb accepts
  `field: reference|value|custom` (`dat-schematic-parametric-reference-value-kyzd`).
  Reference/value selectors reuse the selected-Symbol form's canonical builder,
  including raw payload preservation and supported component-binding repinning.
  They accept `value` or `clear: true`, rejecting custom-field parameters and
  combined value/clear requests. Clearing Value stores an empty string; clearing
  Reference refuses under existing nonempty validation. UUID-qualified Custom
  behavior is unchanged. Existing caller/provenance and Proposal requirements
  remain in force; this does not complete the normalized Operation or ten-field
  public surface.

  The CLI/MCP Reference/Value entry (`dat-schematic-field-public-callers-4dpq`)
  uses that same builder. Current CLI syntax is `project set-symbol-field --root
  <dir> --symbol <uuid> --field <reference|value> --expected-model-revision <rev>
  --expected-tip-transaction <uuid|none>` with either `--value <text>` or `--clear`.
  `--instance-path` accepts ordered instance UUIDs, defaulting to the root sheet.
  MCP requires the equivalent revision/tip parameters; the literal string `none`
  asserts an empty journal. Omission, null, malformed, blank or stale qualifications
  cannot authorize an edit. Callers obtain the revision and last transaction ID
  from `journal list` / `datum.journal.list`; an empty transaction list uses `none`.
  The entry checks the caller's snapshot and canonical commit revalidates the
  persisted revision/tip under the writer lease. MCP retains Tool/Assistant
  provenance and automated component-bound edits still require Proposal acceptance.
  The GUI and remaining field selectors are unchanged.

  The component-first follow-up (`dat-component-first-capture-3btd`) uses the same
  reviewed Assign Part path for an existing provisional Symbol. Normal choices
  require a complete reusable component and display its Value, Package and
  Footprint. Acceptance applies that Part's Value while preserving Symbol and pin
  UUIDs, anchors, wires, connectivity and other raw fields. The value comes from
  the authenticated selected Part revision, including in persisted assignment
  summaries; arbitrary unrelated field changes remain refused. This behavior has
  the bounded ordinary App evidence above; it does not broaden earlier receipts
  or deliver general library ECO.

  **Bounded mapped-component metadata revision (development availability):**
  `dat-schematic-pcb-revision-loop-cf5t` extends Reference/Value authoring only
  for qualified existing one-to-one counterparts with unchanged logical and
  physical bindings. Authenticated metadata history must retain component,
  Symbol, pin, pad and net identities and the pinned physical source; it does
  not adopt a library revision or relax other mapped-component restrictions.
  Board text changes require a separate explicit review after schematic edits.
  The existing root-schematic transfer card opens the shared inline Proposal
  review in the captured root context (study S-P), showing read-only
  Part/Footprint identity and every old/new Reference and display Value.
  Apply/Tab is available only when the complete, current, ready payload and
  acceptance hint fit visibly together, clear of the status strip; otherwise
  acceptance is unavailable with a reason. Esc dismisses without design writes.
  Accepted board changes use canonical history and preserve placement,
  footprints, pads, nets and routes through Undo/Redo, Save and reopening.
  Stale/ambiguous sources, topology or physical-library revision changes and
  unsupported hierarchy refuse without writes. This bounded exception
  neither changes the scope of earlier delivery receipts nor delivers general
  ECO, arbitrary bound edits or hierarchy synchronization.
  Focused engine proof and a separately reviewed isolated ordinary App workflow
  cover Reject, Apply with pending dismissal, Undo/Redo and Save/cold reopening.
  The 14-step workflow uses a synthetic project, programmatic input and a debug
  App with matching release CLI/daemon companions; it is not release-GUI,
  physical/HUMAN, owner-project or full PCB qualification.

  **Bounded connectivity-revision target (native delivery pending):**
  The next `dat-schematic-pcb-revision-loop-cf5t` increment reviews an already
  authored root-sheet pin reassignment between two existing stable
  board-correspondent nets on existing mapped components. The retained transfer
  card opens the same captured-context inline Proposal review (study S-Q).
  Every changed terminal shows its recorded old and proposed new pad, logical
  NetId and board Net UUID, with read-only Part/Footprint context. Display names
  never reconstruct identity. The review
  enumerates retained Track/Via/Zone source identities and any conservative
  potentially affected set, including source-bound unavailable contact reasons.
  Complete pin/pad correspondence and current physical/library source proof
  remain mandatory. An already stale zone fill is not current copper evidence:
  a complete review includes disclosed unknowns and does not certify complete
  physical connectivity, current fill or a complete Global Net.
  Explicit acceptance changes only qualified correspondence in one canonical
  design transaction. Package/pad identities, placement and physical choice stay
  fixed; authored tracks, vias and zones keep their geometry and net assignments.
  The designer explicitly refills, checks and repairs resulting shorts or
  unrouted connections using ordinary PCB tools. For affected subjects, pending
  or mismatched binding or unavailable current geometry retains the PM053 C02
  and PM054 C04/C05 unavailability rules; review fabricates no complete membership.
  The complete current review, including warnings and action hints, must fit
  visibly together before acceptance; preparing, source-stale or unfit reviews
  cannot accept, and dismissal remains reachable. Dismissing a submitted write closes
  its review without cancelling the write or reopening the review on completion.
  Net birth/retirement, ambiguous merge/split ownership, changed physical/library
  bindings and unsupported scope/contact kinds are outside this bounded review;
  these limits do not prohibit ordinary schematic topology authoring. General
  ECO, hierarchy synchronization and native delivery are not claimed by this
  target. Existing metadata-revision and initial-transfer receipts keep their
  original scope.

  Earlier bounded Part assignment implementation (`dat-schematic-part-assignment-dres`)
  adds named compatible Part/Gate choices through the selected Symbol Inspector
  and existing `Assign Part…` context command. It retains the native pool Symbol's
  Unit and geometry. Explicit shared Proposal acceptance atomically creates an
  initially missing `ComponentInstance` or updates one existing single-Symbol,
  schematic-only instance and its current library binding. Current source and
  compatible pin mappings are revalidated; pin UUIDs/anchors, fields, raw
  extensions and unaffected connectivity are preserved, with one-step design
  Undo/Redo and reopening. Physical-package bindings, unsupported shared components,
  repeated occurrences and incompatible mappings remain visibly unavailable or
  refused with reasons. Independent GUI verification covered named assignment and
  reassignment, dismissal, Undo/Redo and reopening. Supported bound-Symbol
  Move/Rotate/Mirror implementation is covered by §2. Existing Reference/Value
  and custom metadata/presentation controls also preserve supported single-Symbol,
  schematic-only bindings (`dat-schematic-bound-fields-0r9i`), including raw
  payload, library identity and connectivity through cancellation, Undo/Redo
  and reopening. Supported same-sheet multi-Gate property edits also repin the
  existing owner without losing siblings (`dat-0fom`). Physical, stale and
  repeated-source restrictions remain. Supported bound-Symbol deletion is covered by §3. The existing Inspector
  distinguishes a valid schematic binding with no declared board counterpart from
  stale, missing or ambiguous bindings (`dat-schematic-binding-inspector-a7rm`);
  owning a component does not imply cross-domain selection or a complete board graph.

  Same-sheet multi-Gate Part reassignment extends this existing flow to another
  compatible Part of the same Entity (`dat-schematic-multigate-part-reassignment-sm4i`).
  The picker and Proposal preview explicitly identify the shared component and all
  placed Gates; Gate cycling cannot substitute a Gate in this mode. Every member
  retains its Gate, Unit, native Symbol, reference/value/field payload and pins.
  The complete owner and every member's pin mapping are revalidated before one
  atomic acceptance updates all Part references and the component binding.
  Cancel/no-op do not change design source; stale, locked or incompatible siblings
  refuse the entire change. Entity/Gate migration, physical/ECO reassignment and
  broader occurrence admission remain outside this increment.

  **Place Another Gate…** configures the selected Symbol's existing picker for
  explicitly chosen unused Gates of its schematic-only component (`dat-0fom`).
  Reference starts empty and must be supplied; the chosen Gate fixes its Unit and
  native library Symbol. After snapped pose preview, a placement click stages the
  shared Proposal review; explicit acceptance creates the new Symbol/pins and
  updates the existing owner atomically. Cancel preserves design source. Members
  share one unique source sheet and qualified occurrence path, empty or existing
  nonempty, without adding hierarchy navigation. Same-sheet Move, fields and
  Delete retain sibling identities through history. Physical, override, stale and
  repeated-source cases refuse; deletion also refuses surviving external dependencies.
  Part reassignment uses the same-Entity whole-component limit above. Active-sheet annotation
  also supports validated same-sheet schematic-only multi-Gate owners
  (`dat-schematic-multigate-annotation-fy5w`), as specified below.
  Independent ordinary GUI checks covered placement preview/cancel/accept, new-member
  Move/value/Delete with sibling preservation, Undo/Redo and cold reopening. All 658
  saved files remained exact. Immediate-restart writer-lock refusal remains separately
  open (`dat-iwxe`); broader qualification is not claimed.

  **Edit > Symbol Fields Table…** edits Reference and Value across the active
  unique source sheet (`dat-schematic-bulk-fields-table-3k12`). Rows use the shared
  text controls, keyboard navigation and paging. **Review changes** freezes exact
  before/after values in the same table; explicit acceptance applies one canonical
  design batch. Simultaneous reference swaps are valid when the final set is unique.
  Empty Values are allowed; malformed, duplicate, stale, locked or unsupported
  requested edits refuse the complete batch. No-op and draft Cancel write nothing;
  cancelling a reviewed Proposal changes its metadata but preserves design source.
  Raw Symbol payload, pins, electrical identities and supported same-sheet singleton
  and multi-Gate component owners survive atomic Undo/Redo and cold reopening.
  The existing single-Symbol Fields form remains available. Custom-field columns,
  Part/library/footprint reassignment, project-wide editing and CSV exchange are
  outside this table increment.

  Automatic position presets, cross-Unit replacement,
  broader part/entity/gate reassignment, complete public-surface normalization
  and guaranteed `ComponentInstance` joins for all Symbols remain pending.

### 10. run-erc (read-only) + author-waiver

- **(1) Manual UI:** `Checks > Run ERC`; select a current finding, then
  `Checks > Waive…` or `Accept Deviation…` with a native Schematic pane
  focused. Enter a rationale in the shared form, preview the Proposal, then
  explicitly accept with Tab or dismiss with Escape.
- **(2) Operation:** ERC evaluation is read-only; explicit Run ERC persists
  revision-qualified check evidence. Findings are addressed by their actual
  `CheckRun` and stable fingerprint, never a row index. Existing
  `CreateSchematicWaiver` / `CreateSchematicDeviation` operations record durable
  dispositions. The GUI configures `native.proposal` with `CreateDisposition`,
  reusing these builders and the shared acceptance lifecycle rather than a
  direct waiver commit. Acceptance applies the disposition and Applied metadata
  atomically; dismissal leaves design source unchanged, though Proposal metadata
  may be journaled.
- **(3) CLI:** `datum-eda project query <root> erc` is the read-only report.
  Existing `project waive-finding <root> --fingerprint <id> --rationale <text>`
  and `project accept-deviation <root> --fingerprint <id> --rationale <text>`
  use canonical operations. Their direct-write policy reconciliation remains
  separate (`dat-waiver-direct-policy-hdj1`); backend availability does not
  waive the Proposal-first requirement. The normalized `erc-waive` alias remains
  pending.
- **(4) MCP/AI:** Read-only reports and existing compatibility check verbs remain
  on the shared surface. The normalized proposal-gated `author_waiver` alias is
  pending; this GUI slice does not change global write policy.
- **(5) AI query/context:** Resolved `DesignModel` + `ErcConfig` (severity
  overrides) + existing waivers; the full connectivity graph so findings
  cite stable object ids.
- **(6) Validating checks:** Self-validating: 7 rules
  (`output_to_output_conflict`, `power_in_without_source`,
  `noconnect_connected`, `input_without_explicit_driver`,
  `undriven_input_pin`, `unconnected_component_pin`,
  `unconnected_interface_port`) + hierarchical-connectivity-mismatch;
  0% FP/FN gate per M0–M5.
- **(7) Proof slice:** On `datum-test`, run ERC, assert a deterministic
  finding set + codes; introduce an output-output short via draw-wire,
  re-run, assert `output_to_output_conflict` appears, undo the wire,
  assert it clears.
- **(8) Bounded native implementation:** Current persisted ERC evidence,
  project/revision guards, unique fingerprints and nonblank rationale qualify
  the shared review (`dat-erc-disposition-gui-rs56`). Stale or already disposed
  findings refuse. Disposition changes and reopening use the existing read-only
  ERC query for reporting when current persisted evidence is absent; unchanged
  source refreshes retain that report without recomputation. These reports keep
  `persisted=false` and cannot authorize another disposition: Run ERC first.
  No extra check-evidence transaction follows acceptance, preserving one-step
  design Undo/Redo and reopening. The existing Inspector presents qualified
  human target/message, producing revision/currentness, status and available
  exact disposition rationale (`dat-erc-readable-inspector-1wof`). Technical
  identities remain expandable; shared wrapping/scrolling reaches the full
  content. Current persisted, computed read-only, stale and unavailable reports
  remain distinct; current rationale is never substituted for historical evidence.
  Removal/replacement UI, other check domains,
  normalized aliases and full qualification remain pending.

## Minimal-Set Recommendation

**Ten tools, not the ~52 native-schematic CLI functions that exist
today.** The matrix collapsed move/rotate/mirror → `TransformSymbol`, ~10
deletes → `DeleteObjects`, ~16 field setters → `SetSymbolField`, and 3–4
label kinds → `PlaceLabel{kind}`. Three further merges apply the same
parametric-kind / optional-id principle consistently:

1. place-junction + place-no-connect → `PlaceMarker{kind}` (both
   zero-dimension single-point sheet annotations).
2. edit-port folded into place-port via an optional `port_id`
   (create-or-edit, mirroring `TransformSymbol`'s optional-param edit
   shape).
3. edit-bus-members + place-bus-entry folded into create-bus as
   `members[]`/`entries[]` params with an optional `bus_id` for edit
   (`BusEntry` is bus-bound, so it correctly stays inside the bus tool
   rather than over-merging into `PlaceMarker`).

The load-bearing, non-negotiable work is NOT new tools but finishing the
contract normalization on top of the journaled schematic substrate and
minting/binding `ComponentInstance` at place-symbol and at part assignment.
Today the current schematic write slices are journaled and MCP-visible, but
the public shape is still compatibility-heavy. ERC evaluation is read-only;
explicit runs persist evidence. Existing canonical disposition operations now
serve the bounded Proposal-gated GUI in §10; normalized public aliases and
global direct-write policy reconciliation remain separate work.

Direct-commit class (local, visible, undoable): place-symbol,
transform-symbol, draw-wire, place-label, place-marker, place-port,
create-bus, set-symbol-field. Proposal-first class:
delete-of-a-board-bound-symbol, bulk delete, bulk reference reannotate,
any sheet-structure change that rewires existing nets, author-waiver.

Bounded active-sheet annotation (`dat-schematic-sheet-annotation-uy8a`) uses
**Annotate This Sheet…** in the existing blank-canvas overflow and shared Proposal
review. Every changed reference is shown as old-to-new text on the immutable
scene; acceptance changes the complete eligible sheet atomically. Prefixes come
from current references with trailing ASCII digits or a terminal `?` removed;
within each exact prefix, source y/x/UUID order receives numbers from 1, skipping
references reserved on other sheets. Active-sheet duplicates can be repaired;
other sheets retain their references, and the complete final set must be unique.
Unsupported ownership, locked or malformed source and stale/repeated occurrences
refuse the whole action. Reference-only writes preserve raw fields, pin/library
identity, connectivity and supported component bindings through Undo/Redo and
reopening. Cancel preserves design source; an already annotated sheet creates no
draft. Supported same-sheet multi-Gate annotation retains distinct per-Symbol
references and the same ordering. It updates each affected owner once, preserving
shared membership, unchanged sibling revisions and all Gate/pin identities.
Independent isolated GUI verification covered preview/cancel/accept, an unchanged
sibling, Undo/Redo and exact cold reopening. A post-Save query hit the separately
tracked writer-lock issue (`dat-iwxe`); that failed query remains recorded, with
cold verification completed on the same saved project after owner shutdown.
Physical, stale and repeated-source owners remain unsupported. The active-sheet
fields table is described in §9.

Bounded project annotation (`dat-schematic-project-annotation-ncy0`) exposes
**Annotate Project…** through Project and the blank-canvas overflow. It qualifies
every root sheet before building one atomic Proposal; it never commits a sheet at
a time or silently skips an unsupported participant. For each reference prefix,
numbering follows sheet UUID byte order, then source y/x/Symbol UUID order, starting
at 1. The existing **Annotate This Sheet…** action keeps its separate-sheet reserved
references and ordering behavior.

Project review uses the shared read-only table to show every changed reference
with its sheet name and exact before/after text, including offscreen sheets.
Duplicate sheet names are qualified with source UUIDs. Empty and unchanged sheets
remain in the complete census. Accept commits all reviewed changes together;
Cancel changes no design source, and an already annotated project creates no draft.
Reference-only edits preserve Values, raw fields, pin/library identity, geometry,
connectivity and supported component owners through Undo/Redo and reopening.
Same-sheet multi-Gate owners retain distinct per-Symbol references and are updated
once per affected owner. Any hierarchy/repeated-sheet census, cross-sheet owner,
physical binding, invalid/locked participant or stale basis refuses the whole
action. The command is available from any qualified current root-sheet pane,
including a nonprimary sheet opened through Navigator; this does not change the
editing targets of sheet-local tools. Independent isolated GUI verification covered
complete two-sheet review and cancellation, four reference changes in one design
transaction, offscreen inspection, one-action Undo/Redo, an already-annotated
no-op, Save and cold reopening with all 851 canonical files exact. Hierarchical
annotation and arbitrary project-wide field editing remain outside this increment.

## Omitted / Redundant Tools

- **rotate-symbol, mirror-symbol** — pure parameters of
  `TransformSymbol`. Altium/KiCad expose rotate/mirror as modifiers during
  a move, not separate operations. Two CLI verbs and one MCP tool removed
  with zero capability loss.
- **delete-symbol, -wire, -junction, -label, -port, -bus, -bus-entry,
  -noconnect, -text, -drawing** — all reduce to
  `DeleteObjects{ids:[ObjectId]}`; object type is recoverable from the id
  in the resolved model. Ten verbs → one. No reference tool has ten delete
  commands.
- **set-reference, set-value, set-lib-id, clear-lib-id, set-part,
  clear-part, set-entity, clear-entity, set-unit, clear-unit, set-gate,
  clear-gate, set-display-mode, set-hidden-power-behavior, set-pin-override,
  clear-pin-override, add-field, edit-field, delete-field** — all are
  `SetSymbolField{field, value, clear}`. Altium does this through one
  Inspector/Properties surface; `custom:<name>` + `value`/`clear` covers
  add/edit/delete field.
- **place-global-label, place-hierarchical-label, place-power-port as
  separate tools** — `LabelKind` already exists in the IR. KiCad's three
  label tools are ceremony; Datum keeps one `PlaceLabel{kind}`.
- **place-no-connect separate from place-junction** — both are
  zero-dimension single-point sheet annotations differing only by kind.
  Merged into `PlaceMarker{kind}`. KiCad keeps them separate (ceremony);
  the label-by-kind principle demands this merge for consistency.
- **edit-port separate from place-port** — a port edit is the same op
  shape as creation. Folded into `PlacePort` with an optional `port_id`.
- **edit-bus-members and place-bus-entry separate from create-bus** —
  members and bus entries are parameters of the bus, not independent
  objects (`BusEntry` carries `bus:Uuid`). Folded into `CreateBus` with
  `members[]`/`entries[]` and an optional `bus_id` for edit.
- **rename-label op** — rename is `SetObjectField` on the label text (a
  display change per the `NetAnchor` rule); no separate operation.
- **Sheet-structure authoring** — native sheet creation, definitions,
  instances and bindings already have canonical journaled operations. These
  structural operations remain outside this ten-tool inventory. Exposing flat
  document creation through the ordinary GUI does not deliver broader hierarchy
  editing; sheet-pin authoring and cross-sheet net rewiring retain their existing
  boundaries. See **Not-Yet-Supported**.
- **any commit/save schematic tool distinct from the global `commit()`** —
  there is exactly one `commit()` primitive in the ratified spine; a
  per-domain save tool would be a forbidden private write path (precisely
  the defect today).

## Shared Surface

This contract adds ONLY the ten schematic-specific typed Operations above.
It does NOT redefine the seven shared operations; see
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` for:

- **Session/Context** (`DatumToolSession` + `DatumContextEnvelope`,
  `datum-eda context get|refresh`) — every CLI/MCP/AI invocation opens here;
  this contract mints no session of its own.
- **Query** (`DatumQueryTool`, `datum-eda query <family>`) — all schematic
  reads (`schematic.query`, `library.query`, `object.get`,
  `relationships.query`) enter as a `--family`/`--domain` parameter and
  are the `aiQueryContext` source for every tool above. Reuse the
  existing read tools (`get_schematic_summary`/`net_info`/`symbols`/...),
  do not reinvent them per tool.
- **Run-check** (`DatumCheckTool`, `datum-eda check run|show`) — ERC is the
  schematic domain of the one check surface; `run_erc` is a thin alias.
- **Propose** (`DatumProposalTool`) — the lifecycle for board-bound-symbol
  delete, bulk delete, bulk reannotate, net-rewiring sheet changes, and
  author-waiver.
- **Apply/Commit** (`DatumCommitTool`, the single `commit()`) — the sole
  mutation gateway every op above flows through; there is no per-domain
  schematic save.
- **Artifact** (`DatumArtifactTool`) — schematic netlist export is an
  artifact projection, not a schematic authoring op.
- **Journal + Undo/Redo** (`DatumJournalTool`) — global undo/redo of any
  schematic transaction is a compensating batch through the same
  `commit()`; this contract ships no per-domain undo. Shared logical history
  tracks repeated Undo/Redo across reopening and clears pending Redo on a new
  normal transaction (`dat-journal-multistep-undo-lr67`). The journal remains
  append-only; noninvertible barriers and Proposal metadata treatment remain
  unchanged. Supported component compensation validates immutable recorded
  semantics and current source before rebasing only permitted revision pins.

Selection is consumer-side state, never an Operation and never journaled;
it is read-only via `selection.describe`. There is no schematic
select/deselect/filter mutation tool.

## Proof Slice & Fixture

**Primary fixture:** the `datum-test` KiCad fixture at
`~/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_sch` (a
real schematic, per the `reference_datum_test_fixture` memory). Import it,
then exercise place-symbol / transform-symbol / draw-wire / place-label /
place-marker / set-symbol-field / run-erc round-trips with undo.

**Hierarchy/native proofs** that `datum-test` cannot cover (port ↔
hierarchical-label matching, native authoring): use the in-repo native
fixtures under
`crates/test-harness/testdata/quality/native_validation_cases_v1/` and the
hierarchy-mismatch-demo golden referenced in
`crates/cli/src/main_tests_query_goldens.rs`.

If a clean multi-sheet hierarchical native fixture is needed for any
broader hierarchy proof, ASK the owner to provide a real one rather than
fabricating sheet JSON (per the no-synthetic-kicad / ask-for-fixtures
memory).

**End-to-end gate for this contract:** a place-symbol → set-symbol-field
(part bind) → draw-wire → place-label → run-erc sequence on `datum-test`
that records each accepted design edit as one atomic design transaction with
provenance, and restores domain state through append-only compensating Undo/Redo
transactions rather than reverting revision identity. Read-only ERC evaluation
does not mutate design source; explicit Run ERC persists separate
revision-qualified check evidence. Proposal metadata and check evidence are
distinguished from the atomic design transaction instead of imposing one journal
row on every UI action. For the supported pool-backed fixture, assert a
`ComponentInstance` is minted at place-symbol and bound at Part assignment;
the unresolved/ambiguous compatibility cases in §1 require their own explicit
status and refusal checks rather than a fabricated binding.

## Not-Yet-Supported

- **Contract-normalized schematic operations.** Current schematic write slices
  use journaled operations and resolver replay, but they remain exposed mostly
  as compatibility-shaped per-kind/per-field commands. The ten-operation
  contract now has its first normalized journal operation:
  `PlaceSchematicMarker` for junction/no-connect placement. It still needs
  normalized `DeleteObjects`, `SetSymbolField`, richer `CreateBus`, public
  `place-marker`/`datum.schematic.place_marker` aliases, and consistent
  `OperationResult` / `model_revision` reporting.
- **ComponentInstance mint/bind from schematic placement.** Pool-backed
  `place-symbol` now stores part/entity/gate UUIDs and mints an authored
  symbol-first `ComponentInstance` with a `part_ref` when the resolved
  entity/gate has one compatible part. Compatibility `lib_id`s, unresolved or
  ambiguous entity-gate matches, ambiguous compatible parts, incompatible part
  mappings, and later board-package attachment remain explicit partial states.
- **Hierarchy authoring.** Sheet definition, sheet instance create/delete/move,
  and parent-port bind/unbind operations are implemented and journaled.
  Sheet-pin authoring and broader net-rewiring hierarchy edits remain
  proposal-first target work.
- **Remaining disposition surfaces and normalization.** Revision-qualified
  ERC records, fingerprint targeting, canonical waiver/deviation operations
  and the bounded Proposal-gated GUI exist (§10). Normalized `erc-waive` /
  `author_waiver` aliases, removal/replacement UI and global direct-write policy
  reconciliation (`dat-waiver-direct-policy-hdj1`) remain pending.
- **Auto-junction inference** during `draw-wire`.
- **Remaining MCP schematic write tools.** The first journaled MCP
  schematic write aliases now exist for connectivity, labels, ports, buses,
  schematic text/drawing, and symbol lifecycle edits. Remaining authoring
  tools must land only behind the same `OperationBatch`/journal substrate so
  AI authoring is never an unjournaled surface.

## Open Owner Questions

1. **Substrate scope.** Current schematic native mutations for
   connectivity, labels, ports, buses, schematic text/drawing, and symbol
   lifecycle edits route through engine `OperationBatch` + journal +
   `ProjectResolver` replay rather than private JSON writes. The remaining
   scope question is which higher-level heterogeneous/bulk schematic edits
   graduate next, and which must be proposal-first.
2. **ComponentInstance timing.** Is `ComponentInstance` minted at
   place-symbol, or only at part assignment? Does its absence block any
   schematic authoring before the join exists?
3. **Direct-commit vs proposal boundary.** Confirm direct =
   place/transform/label/marker/field-edit/port/bus; proposal-first =
   delete-of-a-board-bound-symbol, bulk delete, bulk reference reannotate,
   and any sheet-structure change that rewires existing nets.
4. **Hierarchy GUI breadth.** Current native definition/instance/binding
   operations exist. Disposition complete GUI hierarchy/sheet-pin authoring
   and proposal-first net-rewiring scope without treating backend availability
   as ordinary designer delivery or creating a new deferral automatically.
5. **Disposition normalization.** Reconcile the remaining public aliases and
   global direct-write policy with the `009` rules/checks contract. Existing
   canonical operations and the bounded GUI in §10 do not resolve those gaps.
6. **MCP write timing.** First schematic MCP writes now exist only for
   journaled connectivity primitives (`draw_wire`, `delete_wire`,
   compatibility `place_junction`, `delete_junction`, `place_noconnect`,
   `delete_noconnect`; junction/no-connect placement now journals as
   `PlaceSchematicMarker`) plus labels (`place_label`, `rename_label`,
   `delete_label`), ports (`place_port`, `edit_port`, `delete_port`), and
   buses (`create_bus`, `edit_bus_members`, `place_bus_entry`,
   `delete_bus_entry`), and symbol lifecycle edits (`place_symbol`,
   `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`,
   `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override,
   and symbol field aliases). Remaining schematic write tools, especially
   unified heterogeneous `delete_objects`, should land only behind the same
   journaled substrate.

## PM053 C01–C03 owning reconciliation — 2026-10-01

[PM053](../decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md) ratifies stable NetId final-state atomic nomination/fallback,
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
