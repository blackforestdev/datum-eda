# Library Authoring Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012.

## Driven by

- `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`
  (the library/component primitive set, identity, provenance, binding,
  and proposal-first update propagation)
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified mechanism vocabulary: `DesignModel`, `ProjectResolver`,
  `ObjectId`/`object_revision`, `ComponentInstance`, single `commit()`
  + journal, Import Map `import_key`, derived-state honesty)
- Decisions `000`/`001` for the operation/commit/journal substrate that
  every tool below depends on.

This contract is an IMPLEMENTATION-SPEC tool layer, not product
philosophy. It enumerates the concrete library-domain authoring tools,
the typed Operations they emit, and the leanest set that satisfies
Decision 008. It references the seven shared operations by name and does
not restate them (see Shared Surface).

## Purpose & Scope

Scope: native authoring and editing of every library `DomainObject` in
the Decision 008 set — `Symbol`, `Footprint`, `Package`, `Padstack`,
`Part`, `PinPadMap`, `ModelAttachment` — plus library provenance/approval
state, the `ComponentInstance` library binding, proposal-first library
update (ECO) propagation, and standards-driven footprint synthesis.

Out of scope (owned by other domains, consumed here only as inputs or
provenance): import sessions and Import Map construction (import domain,
Decision 011); schematic/PCB placement geometry (schematic/PCB domains,
Decision 002/003); manufacturing projection/artifacts (Decision 000B);
rule/check authoring (Decision 009). Read access to the pool and the
session/query/check/proposal/commit/artifact/journal machinery are the
shared surface and are referenced, not redefined.

## Current Implementation Reality

The shared write substrate now exists and is not the blocker. The engine has
`Operation`/`OperationBatch`, the single journaled commit path, revision guards,
`ObjectId`/`object_revision` discovery, `ComponentInstance` shards, Import Map
sidecars, resolver-visible pool shards, proposal policy for automated library
mutations, and broad CLI/MCP library command surfaces. Current implementation
evidence includes:

- `crates/engine/src/pool/mod.rs`: canonical pool structs for `Unit`,
  `Symbol`, `Entity`, `Footprint`, `Padstack`, `Package`, `Part`,
  `PinPadMap`, lifecycle, primitives, part metadata, symbol fields/style
  assertions, model attachments, model references, and a simple pool
  search/index.
- `crates/engine/src/substrate/operation.rs`: generic
  `CreatePoolLibraryObject` / `SetPoolLibraryObject` /
  `DeletePoolLibraryObject` operations plus legacy typed package/padstack and
  model attach/detach operations.
- `crates/engine/src/substrate/pool_journal_ops.rs`: write-time schema and
  path/kind/UUID validation for authored pool shards.
- `crates/engine/src/substrate/project_resolver.rs`: resolver discovery of
  `units`, `symbols`, `entities`, `parts`, `packages`, `footprints`,
  `padstacks`, and `pin_pad_maps` pool directories.
- `crates/cli/src/command_project_library.rs` and
  `mcp-server/tools_catalog_library.py`: broad typed library-authoring
  facades for units, symbols, entities, padstacks, packages, parts, package
  graphics, metadata, model attachment, and resolver-backed list/show.
- `crates/cli/src/command_project_library_footprint.rs` and the matching MCP
  aliases now provide the first typed first-class `Footprint` authoring slice:
  create a footprint tied to an existing `Package` and set/update footprint pad
  records tied to existing `Padstack`s, plus rectangular/polygon
  `Footprint.courtyard` geometry and first `Footprint.silkscreen`
  line/rectangle/circle/polygon primitives, through the same journaled
  pool-object gate. The manual CLI also targets `Footprint.fab` with the same
  four graphic actions using `--target fab`; `--target silkscreen` or an omitted
  target retains silkscreen. Generic graphic aliases retain the legacy command
  names. This CLI target parameter does not imply fabrication-target proposal,
  MCP or GUI delivery.

The manual CLI also edits an unplaced Symbol definition’s fields through
`set-pool-library-object --kind symbols --section fields --from-json <array>`.
It validates existing field shape, preserves the original ordered array and
changes only fields through canonical Set/history. Add, change, remove and clear
workflows preserve sibling geometry, pin anchors and Unit links. Omitted section
retains whole-object Set. This does not imply field-section MCP, proposal or GUI
delivery, placed-Symbol propagation, or new field identity/uniqueness policy.

Manual CLI Symbol graphic correction, removal, reordering and clearing use
`set-pool-library-object --kind symbols --section drawings --from-json <array>`.
The existing Primitive schema admits the original ordered array, including
repeated entries and unknown metadata; only drawings change through canonical
Set/history, preserving sibling content. Existing append checks and whole-object
Set remain intact. This adds no geometry threshold, graphic identity policy,
drawings-section MCP/proposal/GUI delivery or placed-Symbol propagation.

Manual CLI Footprint graphic correction, removal, reordering and clearing use
`set-pool-library-object --kind footprints --section silkscreen|fab --from-json <array>`.
Existing Primitive schema admission preserves the raw ordered array; only the
selected collection changes through canonical Set/history. Pads, courtyard,
process/model data, unknown metadata and the other collection remain intact.
Existing append checks and whole-object Set remain separate. This adds no numeric
policy, graphic identity rule, section MCP/proposal/GUI delivery, placed
propagation or manufacturing qualification.

Manual CLI Padstack copper-aperture editing uses
`set-pool-library-object --kind padstacks --section aperture --from-json <value>`.
Resize, shape change, explicit null clearing and restoration preserve the raw
Circle/Rect value, sibling metadata and absent defaults through canonical
Set/history; a new Footprint pad can reference the restored Padstack UUID.
Existing whole-object schema admission and typed Create checks remain separate.
This adds no numeric/drill/process policy, normalization, placed propagation,
section MCP/proposal/GUI delivery or manufacturing qualification.

Manual CLI Footprint pad-collection editing uses
`set-pool-library-object --kind footprints --section pads --from-json <map>`.
Existing `HashMap<Uuid, Pad>` admission preserves the raw map for add, edit,
removal and empty-object clearing through canonical Set/history. Retained pad
IDs, leaf metadata and non-pad siblings remain intact; the single-pad command
keeps its separate checks. This adds no identity, numeric, layer, name, PinPadMap,
placed-propagation, source/standards or section MCP/proposal/GUI policy.

Manual `datum-eda project process <root>` provides `profiles`, `set-profile`,
`delete-profile`, `select-basis`, `inspect`, `placements`, `pin-pad`, `adopt` and
`set-associations` over canonical engine primitives. Profile/basis writes use
journaled history; selecting `--profile` captures its current revision/hash.
Inspection distinguishes absent policies, explicit zero, inherited basis and
unresolved inputs. Existing `proposal create-board-component-replacements
--replacement <json>` carries explicit `process_update` intent through normal
review/application. These CLI capabilities do not establish MCP or GUI authoring,
standards certification or broader qualification.

The PM062 GUI delivery at `61b5b409` exercises named browsing and typed
relationships for all eight kinds, plus manual Footprint/Padstack create/edit
with named existing Package/Padstack references, supported Circle/Rect geometry,
process inspection, preview/cancel, canonical Undo/Redo, Save and ordinary App
reopen. Actual App, engine and daemon evidence and distinct reviews apply only
at their recorded source pins and scopes; bounded source landing is recorded in
`docs/reviews/workflow-delivery-rollout/library-gui-authoring-delivery.json`.
Definition edits
preserve existing placed users. Full eight-kind authoring, Package editing,
models, approval, generation and general placed updates remain targets; seed,
source rights, standards/manufacturing and HUMAN/assistive qualification remain
separate. PM062 does not establish those capabilities through this candidate.

The creation-only Package GUI increment adds named Package creation using
existing family/code/mounting metadata, optional nominal body dimensions/heights
and terminal identity/name/role. The shared library form uses the canonical writer
and history; the new Package is available to the named Footprint picker. Actual
ordinary App proof covers an initially empty native Project, preview/cancel,
malformed-input refusal, Undo/Redo, Save/reopen and normal/narrow keyboard controls.
Source pins, execution and distinct review receipts are recorded in
`docs/reviews/workflow-delivery-rollout/package-gui-creation.json`.
The preview is a dimensional envelope; terminal positions and exact outline are
unavailable. Saved-Package geometry/topology editing, tolerance/model authoring,
placed updates and broader qualification remain open.

Saved Package metadata editing (`dat-saved-package-metadata-edit-fztr`) adds name,
family, code and mounting text to the same Library form. Dimensions and terminal/
pad topology remain read-only. Only changed text fields enter the canonical raw
patch; identity, unedited fields, optional absence and unknown metadata survive.
Existing Package-role bindings and frozen legacy/Footprint pad users retain their
exact historical definitions, source pins, geometry and connected Nets. The
existing authenticated history qualifies those revisions. Counter reuse that
would make a retained revision ambiguous, including metadata Undo/Redo collisions,
refuses before publication. Preview and
Cancel leave canonical sources and journal unchanged. Seventeen focused engine/
form checks and the ordinary debug App's 67 stages and six inspected captures
cover explicit commit, history, Save/cold persistence and unchanged placed root/
child rendered identity. The App uses a body-only Package; engine cases supply
the nonempty frozen-pad and historical-binding proof. Presented controls and
logical keys supply no physical/HUMAN, release-GUI execution, full Package
geometry/topology authoring or broader library qualification claim.

The creation-only Unit and Symbol GUI increment uses the same canonical writer
and shared library form. A Unit has named logical pins and electrical types; a
separate Symbol links to a named Unit and supplies one explicit rectangular body
and named pin anchors with positions, directions and stub lengths. The form shows
the pin table, declared geometry preview and change review before Commit; Cancel
discards the draft. Created Symbols are available to the schematic placement
picker without requiring a physical Part binding. That creation increment does
not establish saved-definition or general graphics editing.

The saved Unit/Symbol property increment (`dat-saved-unit-symbol-edit-qwni`)
extends the shared form with saved names, existing logical-pin names/electrical
types and existing anchor positions, directions and stub lengths. Named pin and
anchor rows select the data to edit; preview, explicit Commit/Cancel, canonical
Undo/Redo and Save/reopen use the same writer. UUIDs, retained Unit links,
arbitrary source graphics/field arrays, unknown metadata and absent defaults
remain intact. Existing placed pin metadata, body and pin graphics remain frozen
through source edits and history. Declared Symbol revisions and native placement
transaction before-models supply authenticated source reads. An unavailable frozen
basis refuses edits that require it; qualified read errors retain per-object
unavailable inspection. Unbound legacy display does not establish a frozen binding.
Edits that would
make an existing pinned revision ambiguous are refused before mutation. Actual
focused engine/protocol and isolated ordinary-App proof is recorded in the task;
the App proof uses presented controls and logical keys in a debug test profile,
not physical/HUMAN qualification or release-GUI execution. General graphics and
pin-topology authoring, relinking, Package/Entity/Part/PinPadMap editing, approval,
models and placed-definition updates remain outside this increment.

Saved Unit pin alternate names and swap group
(`dat-saved-unit-pin-alternates-swap-i57e`) use the shared Library form.
Designers can add, select, edit and remove alternate names of a selected
saved pin and edit its swap group, with preview, cancel, canonical
Undo/Redo and Save/cold reopening. Alternate names retain their ordered
literal name and kind strings, including duplicates. Captured indices and
exact raw preimages identify the original rows. Edits preserve the pin
UUID, unrelated row metadata and other pins. The existing unsigned
32-bit swap group uses zero for no group. This increment stores authored
metadata and leaves pin swapping, alternate-function use and placed ECO
to their existing separate authority.

The isolated native App passed 79 stages in 24.40 seconds with three
inspected captures. Add preview showed two alternate names and group 7.
Edit preview and the reopened form showed one alternate and group 12.
Six sole Set Unit operations preserved the original 72 journal transactions
byte-for-byte. Their diffs identify only the Unit and selected nested pin
as modified. The other 32 canonical sources stayed byte-for-byte identical,
including placed circuit and mapping data. The final project had 33 sources
and 78 transactions. There were 86 scoped Rust passes, with existing
`dat-17ld` ignored. The App evidence comes from programmatic controls
in an isolated Wayland session.

Saved Unit pin addition (`dat-saved-unit-add-pin-g2dm`) now exposes Add pin in
the shared form. Each explicit new row retains one fresh UUID, a name and an
explicit electrical type across selection and invalid input. Remove draft pin
discards only a new row; that addition increment preserved existing saved pins. Pin controls on
other Library kinds refuse before changing the draft. Sparse
`pin_additions` compose after supported old-pin metadata edits, authenticate the
original raw membership and source, and reject nil, owned, duplicate or aliased
identity collisions. The existing creation builder supplies new-pin defaults;
untouched original keys, values, unknown metadata and child revisions remain exact.
No new minimum-pin or unique-caption policy is introduced.

Preview/Cancel, canonical nested-Pin registration, Undo/Redo and Save/cold reopen
use the existing writer. Old placed names, types, positions and membership remain
in their actual authored SymbolPin records; this does not invent a placed Unit
binding or a new history reader. An actual connected eight-pad Complete certificate
and held/fresh membership survive successful addition/history/cold, with existing
logical-pin counters and mapping sources unchanged.

Twenty-three unique focused Rust cases and one ordinary isolated debug-App case pass:
110 stages, 22.55 seconds and nine inspected captures. The App uses the original
22-source/20-transaction circuit without setup writes. Unit addition, Undo and Redo
each contain only Set Unit and create/delete/restore the new nested Pin; all other
21 sources, old root/child scenes, CIs, pins, references, Maps and Nets remain exact.
A later shared Symbol creation selects the added pin by name, previews and commits
its explicit body/anchor, then survives cold reopen. That separate transaction adds
only one Symbol definition, ending at 23 sources and 24 transactions.

The initial engine test's empty Symbol body was correctly refused by the existing
one-rectangle creation rule; only that test fixture changed. The new wrong-kind
test first needed a named Unit in its fixture; that setup was corrected. The
pre-landing Symbol-control regression is fixed by the early Unit-kind check,
with direct Add/Remove/select refusal tested without draft mutation. Earlier
App proof remains at its own candidate; the final corrected build supplies the
current captures. The existing ignored rectangle case remains uncredited. App controls and logical input are programmatic
in owned Wayland. That addition proof supplied no saved-pin removal/rekeying,
Symbol-topology edit, implicit placed update, new reader/schema/controller, approval,
physical/HUMAN observation, release-GUI execution, full qualification or broader-goal
acceptance.

Saved unused Unit pin removal (`dat-saved-unit-remove-pin-bu3g`) now exposes
Remove pin for a selected saved row in the shared form. Preview/Cancel, canonical
Undo/Redo and Save/cold reopening use the existing writer. Sparse `pin_removals`
authenticate original parsed raw identities, embedded UUIDs and registered source
ownership, including the existing `pins` kind fallback. Only selected raw rows
are deleted; remaining spellings, metadata, unknowns and child counters stay exact.
Removal discards only the removed row's pending name/type. Other metadata and
distinct additions compose; adding the same original ID in that draft refuses.
New-row discard remains separate, with existing zero-pin and caption semantics.

Current Symbol anchors, default/nondefault and legacy mappings, actual placed
pins and mappings retained by authored component declarations prevent removal.
Existing authenticated revision reads preserve distinct historical Part contexts,
including registered declarations omitted from the resolved component projection.
Relevant unavailable or ambiguous membership refuses, including duplicate sources
that hide an exact pin reference; unrelated duplicate sources do not blanket-block
an unused pin. Mapped pins must belong to their authenticated Unit source. A new
canonical anchor or mapping after preparation invalidates the stale removal.

Thirteen engine and nineteen protocol cases pass, including successful removal
through connected eight-pad Complete/history/cold and held/fresh membership.
The ordinary isolated debug App passes 85 stages in 16.62 seconds with eight
inspected captures: readable used-pin refusal/Cancel, unused Preview/Cancel,
commit/history/cold, and unchanged root/child rendered scenes after cold reopen.
Starting from 28 sources and 32 transactions, one canonical AddPin fixture supplies
the unused pin. Three sole Set Unit history transactions end at 36 transactions;
all other 27 canonical files remain byte-exact and the final Unit raw payload
matches its pre-setup original. Actual App assertions retain original named pins
and exclude the retired pin from later Symbol choices; that creation form is
canceled, with no Symbol transaction. The choices capture is not an expanded list.
Earlier fixture, validation-stage and QA setup failures remain uncredited at their
recorded candidates. Programmatic controls establish no physical/HUMAN or release
GUI execution, implicit placed update, rekeying, new reader/schema/controller,
approval, full qualification or broader-goal acceptance.

Saved Symbol anchor addition (`dat-saved-symbol-add-anchor-b6o1`) uses the same
form to select an existing named pin of the retained Unit and set physical
position, direction and stub length. Old/new row selection retains intent and
invalid text; Remove draft anchor discards only a new row. This addition increment
did not remove or relink saved anchors. Sparse `anchor_additions` compose after supported
old-pose edits and append to the ordered raw array. Old anchors, mixed graphics,
fields and unknown metadata remain exact. The existing creation admission and
unique parsed-UUID pin lookup are shared with later materialization; ambiguous
membership, unsupported Unit schema, wrong-kind controls and stale sources refuse.
Qualified Unit reference reads confer no canonical saved-write authority. Consumed
Unit/pin and Symbol revisions are guarded; existing frozen-placement readers and
retained-counter checks preserve old users.

Thirty-one unique focused Rust cases and the corrected ordinary App run pass: 99 stages,
21.63 seconds and nine inspected captures. Preview/Cancel, new-only discard,
canonical Undo/Redo and Save/cold reopen use the existing writer. Starting from
the prior 23-source/24-transaction circuit, no setup write is needed. Three sole
Set Symbol history transactions preserve all other 22 files and old root/child
scenes and captures. A later separate explicit Part/gate placement uses the
expanded Symbol and two actual pins, creates one CI and two electrical identities,
and permits only the exact existing initial Net lineage transition. Final cold
state has 26 sources and 28 transactions. The capture named `cold-child` depicts
Root after reopening; Child rendering assertions and frozen/source checks supply
their own evidence. Raw compile/reference-fixture failures and the existing
ignored case remain uncredited. The initial acceptance was withdrawn before landing
after a valid compact numeric UUID exposed an existing `pins` registry fallback.
The corrected guard admits that legitimate class under the same identity/source
checks; append, materialization, history and cold regression cover it. The 13
unchanged protocol cases retain their prior proof; the corrected engine and App
supply matching current checks. Original candidate artifacts retain their own scope.
This addition proof supplies no saved-anchor removal, automatic
placed update, new history reader/schema/controller, physical/HUMAN observation,
release-GUI execution, full qualification or broader-goal acceptance.

Saved Symbol anchor removal (`dat-saved-symbol-remove-anchor-6pf0`) uses the
selected saved row's existing Remove anchor control; Remove draft anchor still
discards only a pending new row. Sparse removals authenticate original parsed
Pin UUIDs and their retained Unit membership, preserve remaining raw anchor order,
graphics, fields and unknown metadata, and leave the logical Unit pin intact.
Selected removed-row invalid pose text is discarded with that row; other invalid
buffered intent still refuses. Distinct additions, removals, old poses and metadata
compose. Re-adding the same original Pin removed in this draft refuses clearly
without normalizing its typed choice or implicitly restoring the row. Cancel
discards the draft. Zero anchors remain allowed under existing source semantics.

Thirty-eight focused Rust cases and the ordinary App proof pass: 99 stages,
22.11 seconds and nine inspected captures. Engine cases include successful
removal with connected eight-pad Complete, held/fresh membership and counters,
and removal of all five authentic bound anchors followed by a new zero-pin
explicit Part placement and cold reopen. The App starts from the prior
26-source/28-transaction circuit without setup writes. Three sole Set Symbol
removal/Undo/Redo transactions preserve the other 25 sources, logical Unit pin,
independent Supply Symbol and old users. Four old placements retain their exact
captured definitions and pin counts of one, one, one and two. A later separate
explicit Part/Gate B placement consumes the revised Symbol with one retained
Input pin, creates one CI and one Net, and permits only the two existing Nets'
exact recorded initial-lineage transitions. Final Save/cold state has 28 sources
and 32 transactions. All four private process owners stop, with no cleanup errors.
The raw initial fixture failure assumed one anchor in a real five-anchor fixture;
only that test assumption was corrected. Existing ignored17ld remains uncredited.
Controls and shared logical input are programmatic in owned Wayland. This proof
supplies no logical-pin removal/rekey, anchor relinking, implicit placed update,
new schema/reader/controller/minimum policy, physical/HUMAN observation,
release-GUI execution, full qualification or broader-goal acceptance.

Native Symbol pin-label clearance (`dat-7b2y`) positions the name
and number against the actual projected Symbol body, transformed pin stub
and terminal ink. The Library preview shares that layout and retains its
fitted text size and terminal markers. Schematic rendering retains its
existing stroke classes and widths. Label roles and pin source identity
remain stable. Authored geometry, attachment points and electrical data
stay intact.

The bounded narrow-body correction (`dat-opposed-pin-label-crowding-q0y5`)
keeps a name on its owning side when full body clearance would carry it
through the far outline. It uses the existing own stub, terminal and number
ink for the minimum feasible inward adjustment, retaining full text and pin
identity. Body clearance is best effort when the authored Symbol is too
small; opposed names can still overlap each other or the body. No text
shrinking, wrapping, outside-body fallback or automatic body resize occurs.

The saved TERU circuit passed a read-only isolated App run with 41 stages
and five inspected captures: Library preview and Root/Child open and cold
views retain the source-bound names on their owning sides. The existing fit
projection now uses bounds without the displaced names; its Inspector preview
fits the text, while narrow-body canvas overlap remains visible. All 367
source/history files and 104 journal entries remain byte-for-byte identical.
Twenty focused native-library checks passed, including cardinal/mirrored
poses and the retained thick-stroke and fitted-marker cases. The initial
strict-anchor marker regression and subsequent stroke-radius test-assumption
failures remain recorded. This is programmatic debug App proof, with no
physical/HUMAN, release-GUI execution or broader-goal acceptance.

The original thick-stroke Library case and the XQ2 schematic case passed
in two isolated native App runs. The 14-stage Library run shows IN and
OUT clear of the 0.254 mm body stroke in the enlarged preview and
Inspector. The 23-stage schematic run shows S17 above the XQ2 stub in
open and cold Fit views and above the uncommitted XQ3 stub. Nearby labels
can still be crowded in the Fit view. Both runs preserve their canonical
sources and journals byte-for-byte, with 9 sources and 9 transactions
in the Library case and 34 sources and 86 transactions in the schematic
case. No placement was committed by this proof. There were 27 scoped
Rust passes. The updated native regression compares qualified loaded and
gesture label layouts at the same pose while retaining its exact source,
terminal, non-pin text and bounds checks.

Saved Symbol schematic pin designators
(`dat-saved-symbol-anchor-designator-8ebg`) use the shared saved anchor
form. Designers can set a literal string or clear the optional property,
preview or cancel, commit, use canonical Undo/Redo, and reopen the saved
definition. Captured indices and exact raw preimages identify the selected
anchor. Edits compose with the existing pose and relink paths and preserve
unrelated raw data. Clear removes the property. A later bare placement uses
the logical pin name when the designator is absent, and retains a literal
empty string when explicitly authored. This value belongs to the schematic
pin. Physical pad designators and PinPadMaps retain their existing authority.

The isolated App passed 91 stages in 32.48 seconds with five inspected
captures. The form and review show S17, and the Clear review shows the
logical pin name fallback. A separate XQ2 placement uses S17 on its new pin.
The warm and cold Root scenes contain source-bound S17 text and glyph
geometry. At the captured Fit zoom the small canvas number partly overlaps
the terminal, within the existing pin-label issue `dat-7b2y`. The prior
Omega glyph gap remains tracked under `dat-ui-filename-glyph-coverage-o53p`.
The original 78 transactions remain byte-for-byte identical through seven
sole Set Symbol operations followed by the new placement. Existing Root
and Child geometry remains exact through definition edits and history.
Placed data and physical maps are preserved. The final project has 34
sources and 86 transactions. There were 89 scoped Rust passes, with
`dat-17ld` ignored. The App evidence uses programmatic input in an
isolated Wayland session.

Saved Symbol anchor pin correction (`dat-saved-symbol-relink-anchor-1p7a`)
exposes the retained Unit's named pin chooser for a selected saved anchor. The
original captured row identity remains stable while the target changes. Sparse
`anchor_relinks` changes only the pin reference at the original array position;
raw metadata, unedited geometry, legacy style and other rows remain intact.
Same-target choice preserves the original spelling and emits no relink intent.
Authenticated same-Unit membership and existing source/revision guards refuse
foreign, occupied, ambiguous or stale targets. Same-row removal/addition conflicts
refuse explicitly; distinct row edits compose without inferring swaps.

Preview/Cancel, canonical Undo/Redo and Save/cold reopening use the existing
writer and history. Old root/child placed pins and frozen definitions retain
their previous association. A later explicit new Symbol placement consumes the
corrected definition. This corrects a Symbol reference without creating or
rekeying a logical pin, changing the Unit or applying a placed-component ECO.

Fifty-six relevant focused Rust cases and the ordinary isolated App case pass
(89 stages, 21.99 seconds, nine inspected captures). With no setup transaction,
the original 28-source/36-transaction circuit receives three sole Set Symbol
history transactions; all other 27 sources remain byte-exact in that core phase.
One later bare Symbol placement creates U5 with Auxiliary supply/PowerIn at the
retained anchor geometry. It adds one electrical identity and records only the
existing initial Net's exact lineage transition among old identities; the other
25 original files remain byte-exact, with 29 sources and 40 transactions after
cold reopening. The original journal prefix is byte-exact. The new-placement
ghost and committed captures have a narrow schematic pane; exact association,
type and position come from executed App/source assertions, with the new Symbol
visible in the cold capture. Programmatic controls establish no physical/HUMAN,
release-GUI or full-qualification result. A broader test filter also selected an
unresolved board-track failure (`dat-exact30-track-ci-lg26`); that batch is not a
PASS and does not widen this increment's scope.

Saved Symbol definition fields and reference prefix
(`dat-saved-symbol-fields-prefix-rmdc`) use the shared Library form. Designers
can add, select, edit and remove ordered field rows, including duplicate keys.
A row stores its literal key and value, optional position and visibility.
Captured indices and exact raw preimages identify saved rows. Coordinate edits
preserve Point metadata. Clearing a position removes its member. Untouched
legacy values and raw metadata retain their representation.

The prefix editor stores the string as typed. Clear removes the property and
the existing bare Symbol allocator falls back to `U`. An authored empty
string produces a numeric reference. Existing placed instances retain their
data. A later bare placement uses the saved prefix and the existing allocator,
takes its Value from the Symbol name, and starts with empty placed fields.

The isolated native App passed 111 stages in 35.57 seconds with six inspected
captures. The add and edit previews showed the field review. Clear showed
Default U. After cold reopening the form showed XQ and MPN, and the test
asserted the remaining hydrated row data. The MPN value retained Greek Omega
in the source. The preview showed a replacement box for it, within the
existing glyph-coverage gap (`dat-ui-filename-glyph-coverage-o53p`).
The original 61 transactions remained byte-exact through ten sole Set Symbol
operations and one separate XQ1 placement. XQ1 was readable in the placement
ghost and after reopening. The final project had 33 sources and 72
transactions. The focused Rust suites passed 82 tests, with the existing
`dat-17ld` case ignored. The App evidence comes from programmatic input in
an isolated Wayland session.

Native electrical preview status (`dat-f43w`) uses the existing refusal and
process state. Valid Unit and Symbol previews show their validated state without
a false refusal action or irrelevant Drill line. The actual refusal action stays
available, including in the Tab sequence, when the draft or browser supplies a
reason. Footprint and Padstack process inspection remains available; the generic
Drill line applies to Padstacks, with existing Footprint and Package summaries
retained. No authoring operations, notification state or status vocabulary change.
Five focused renderer checks pass. The ordinary read-only App passes 31 stages
in 12.01 seconds with four inspected captures, including actual Tab cycles and
a zero-radius Circle refusal. The full refusal reaches Console state; its visible
strip truncates the message. All 392 persisted source/history files, including
49 canonical sources and the 111-transaction journal, remain byte-exact. GUI
terminal runtime contexts change and are excluded by the existing source audit
scope; whole-project byte equality is not claimed. This supplies no physical/HUMAN,
release-GUI execution, full qualification or broader-goal acceptance.

Saved Symbol Circle graphics (`dat-saved-symbol-circle-qngv`) use the shared
ordered graphic-row and numeric form. Designers can add, select, correct and
remove center/radius/stroke, then Preview/Cancel, commit, Undo/Redo and reopen.
New and changed Circles require positive radius/stroke and representable integer
extents. Untouched admitted legacy rows gain no geometry gate. Captured indices
and raw preimages distinguish identical rows; corrections preserve center
metadata and unrelated graphics, and typed additions retain append order without
persistent graphic IDs. Circle schema and native projection already exist;
this adds no canvas Circle tool, writer, public verb or placed-user update.

The isolated App passed 113 stages in 38.06 seconds with nine inspected captures.
It explicitly removes the original Rect, authors a 1.5 mm Circle, corrects it to
2 mm, exercises invalid-input discard/removal/history and cold reopening, then
places a separate explicit Part as TC3. Old Root/Child definitions, pin/net
membership and actual body/pin/text/glyph ink remain frozen, including warm
and placement previews. The original 104 journal rows remain byte-exact through
six sole Set Symbol operations and that placement; the final project has 49
sources and 111 transactions. Old Root members, the complete Child, original CIs
and other pool sources remain exact. One old Net's predecessor list contracts
to itself and its revision increments during placement; no universal unchanged
identity-record claim is made.

There are 96 unique scoped Rust passes, with existing `dat-17ld` ignored. The
initial stale editable-row assertion failure remains recorded. The cold capture
shows center X, while all four cold values are asserted. The placement HUD partly
covers the ghost's top; the complete Circle is visible after cold reopening.
The original 1.5 mm and 2 mm preview captures contain identical pin-name
pixels in the compared 96 × 19 region. Independent inspection disproved the
apparent disappearance reported in `dat-circle-preview-pin-context-i7bx`;
no product correction was needed. Small preview text and existing canvas
overlap remain explicit limitations.
Programmatic debug App proof supplies no pin-text readability, physical/HUMAN,
release-GUI execution, full qualification or broader-goal acceptance.

Saved Symbol literal Text graphics (`dat-saved-symbol-text-graphics-w0k9`) share
the Line and Rect form. Designers can add, select, edit and remove text, including
its position and rotation, then preview, cancel, commit, undo, redo, save and reopen.
Captured indices and raw preimages identify saved rows. New text follows the
existing CLI trim and nonempty rule. Saved text preserves literal spelling and
the existing signed 32-bit rotation range. Edits preserve other raw graphics,
Point metadata, anchors and old placements. Typed appends retain insertion order.

Library preview bounds include the recorded world text layout and available
legacy fill and stroke extents. The first run exposed clipped Inspector labels
and an occluded placement ghost. In the corrected isolated App run, REMOVE and
READY fit in the preview. READY is readable on the placement ghost after View
Zoom Out at the same world pose and after reopening. Rotated DRAFT crosses the
body at its authored position. The cold form shows the literal and shared
controls, and the workflow asserts position and rotation.

The 142-stage App run passes in 35.69 seconds, with eight inspected captures.
The original 54 transactions remain byte-exact. Six sole Set Symbol operations
preserve the other 30 core sources, followed by a separate U8 placement with the
revised label. The final project has 32 sources and 61 transactions. There are
82 unique scoped Rust passes, including actual rotated text projection and
preview bounds. The GUI evidence comes from programmatic controls in an isolated
Wayland session.

Saved Symbol rectangle lifecycle (`dat-saved-symbol-rect-lifecycle-6ksv`) extends
the shared graphics form to add, select, correct and remove individual Rects,
including multiple identical rectangles. Lines and Rects share one captured row
collection, selection, numeric buffer and action/navigation list. Original array
indices and exact raw preimages identify saved rows. Rect corrections change only
min/max coordinates and stroke; ordered raw siblings, Point metadata, anchors and
fields remain intact. New rows serialize typed graphics only, with no persistent
graphic UUID. Mixed new Line/Rect rows retain append order. The legacy unique-Rect
request derives from the same buffer; conflicting legacy/sparse intent for one
row refuses. Removing the selected row discards only its invalid fields.

New and changed Rects retain the existing manual producer's ordered-corner and
positive-stroke checks. Untouched admitted legacy Rects gain no geometry gate;
existing Line append versus saved-edit admission remains intact. No additional
primitive, canvas, controller, schema, history reader or placed-user ECO is added.
The previous shared preview fix continues to preserve old placed definitions.

The isolated App passes 146 stages in 32.72 seconds with eight inspected captures.
No setup is required: the original 47 transactions stay byte-exact, six sole Set
Symbol history operations preserve the other 29 sources, and a later explicit U7
placement alone consumes the revised rectangle. Its separate consumer records one
new Net and the exact old initial-Net lineage transition; the other 27 original
files remain byte-exact. Final cold state has 31 sources and 54 transactions.
Actual Root body, pin, text and glyph geometry remains exact before/during the
ghost, after commit and cold; the new ghost and placement use revised geometry.
All five fields are visible in preview; the cold form shows the selected row,
shared actions and minimum X, with the other four cold values asserted. Selected
Rust checks pass (45 GUI-electrical engine, 30 protocol and one new frozen-scene
case); the broader engine batch remains failed on known `dat-exact30-track-ci-lg26`,
and `dat-17ld` remains ignored. Initial fixture and transport-launch failures stay
recorded. Controls are programmatic, with no physical/HUMAN, release-GUI,
full-qualification or broader-goal acceptance.

Saved Symbol Line graphics (`dat-saved-symbol-line-graphics-l2f7`) now supports
explicit Add line, selected saved-row endpoint/stroke correction and removal in
the shared form, with Preview/Cancel, canonical Undo/Redo and Save/cold reopening.
An original captured array index and raw preimage distinguish even identical
Lines. Corrections patch only five numeric leaves; removals retain all other
ordered raw graphics, Point metadata, anchors and fields. Distinct edits compose
with the existing rectangle and anchor controls. Removing a selected row discards
only its invalid buffered fields; new-row discard remains separate. New Lines
reuse existing append admission (distinct endpoints, positive stroke), while
saved corrections retain existing Primitive schema admission. New rows serialize
only typed Line fields, preventing arbitrary nested UUID extension manufacture.
No persisted graphic identity, new geometry policy or general graphics editor is
introduced.

Shared schematic gesture previews now capture each old instance's authenticated
Symbol definition through the existing model reader. Existing instances use that
local definition at their requested pose; new placement ghosts use the selected
current definition. Missing or unauthenticated old captures use the existing
source-unavailable presentation, with no latest-definition substitution. Legacy
unbound sources retain their ordinary current-source display semantics. This
fixes the observed warm-preview leak of revised Lines into old instances without
adding a history reader, persistent cache or separate renderer.

There are 112 unique passing scoped Rust cases, including preservation during
placement, wire, marker and moved/rotated previews. The affected pose batch also
retains one known annotation-admission failure (`dat-project-annotation-root-assertion-i6m9`),
so it is not an overall PASS. The current ordinary isolated App passes 140 stages
in 21.63 seconds, with eight inspected captures. No setup is needed: six sole
Set Symbol transactions exercise two history sequences while preserving the
other 28 sources and original 40-transaction journal prefix. A later explicit U6
placement uses the revised Line; only one exact old initial-Net lineage transition
and one new Net accompany that consumer, leaving the other 26 original files
byte-exact. Final cold state has 30 sources/47 transactions. Old Root body,
terminal, text and glyph geometry are checked before and during the ghost, after
commit and cold; only U6 displays the new Line. Earlier failed fixtures, compile
attempts, the wrong-scene App assertion and the visually incorrect prior preview
remain recorded. Controls are programmatic; no physical/HUMAN, release-GUI,
full-qualification or broader-goal acceptance is supplied.

The earlier rectangle-correction increment (`dat-saved-symbol-rectangle-iltr`) exposed the
five existing body min/max coordinate and stroke fields for exactly one existing
raw Rect, at its retained array position. Other known graphics can coexist. Only
those five integer leaves change: all other graphics, nested unknown metadata,
Unit/UUID, anchors and fields remain intact. Ordered corners and positive stroke
retained the creation validation. At that earlier baseline, zero or multiple Rects
disabled body correction while metadata/anchor flows remained available; no
primitive was guessed, added, removed or reordered. The shared graphics lifecycle
above now permits explicit Rect selection, addition and removal. Unknown primitive variants retain the
existing typed refusal rather than being dropped or adopted.

The body request qualifies its unique canonical source and captured revision
before the existing Set writer. Authenticated declared Symbol revisions and
unique placement birth before-models keep old bodies, pins, CI and Nets frozen.
The existing final retained-root guard now includes Symbol bindings and override
root references: different payloads at a retained counter, including Undo/Redo,
refuse atomically; equal authenticated payloads retain their admission. Symbol-only
checks avoid unrelated physical-pad scans. Part/Package guards and their strict
prospective-source rules remain unchanged.

Forty-one unique focused Rust checks pass, with the existing unrelated `dat-17ld`
case still ignored. The ordinary isolated debug App passes 110 stages and eight
inspected captures for five-field editing, Cancel, history, Save/cold reopening,
visible zero/multiple-Rect forms and later explicit Part/gate placement using the
corrected mixed body with current source guards. The actual pre-setup old bodies
remain authenticated; setup is not relabelled as old placement evidence. Core
correction leaves old circuit sources and Net identities exact. After the separate
new placement, old membership stays exact and only the existing canonically
recorded initial Net-lineage transition is qualified. Earlier compile, fixture,
source-guard assertion and redundant post-placement Cancel failures are retained;
a zero-match invocation supplies no proof. The plain Symbol producer is unchanged.
Existing preview-affordance issue `dat-f43w` remains open separately. This supplies
no general graphics canvas/picker, topology/relinking, placed update/ECO, approval,
physical/HUMAN, release-GUI execution, full qualification or broader goal acceptance.

The new Entity GUI increment (`dat-entity-gate-gui-create-zcp0`) adds project-local
creation in the shared Library form: explicit name, reference prefix,
manufacturer, tags and one or more named gates. Each gate selects an existing
named Unit and a compatible Symbol. The table separates gate, Unit, Symbol and
reference state; preview lists the resulting fields and links before Commit.
The shared `datum.library.create` engine adapter independently checks every
reference, Symbol/Unit compatibility and gate key/UUID identity. Preview and
Cancel leave source and journal unchanged. One canonical Create transaction,
Undo/Redo and Save/cold reopen retain Entity/gate identities and links while
existing definitions and placed circuits remain unchanged. Focused engine/form
checks and the isolated ordinary App run are recorded in the task; the App uses
presented controls and logical keys in a debug test profile. This establishes
neither physical/HUMAN qualification nor universal raw-operation validation.
Generic raw Entity gate-link admission is separate intake
`dat-entity-raw-links-ysdr`. Saved Entity gate name/topology/Unit-link editing, saved Part Entity/Package relinking,
saved PinPadMap editing,
multi-gate placement, approval, models and broader library completion remain open.

Saved Entity metadata editing (`dat-saved-entity-metadata-edit-3y3v`) now uses the
same Library form for name, reference prefix, manufacturer and tags. Existing
gates appear in a named Gate / Unit / Symbol / reference-state table; this
metadata increment does not change their links.
Preview and explicit Commit use `datum.library.edit` and the canonical Set writer;
the engine independently refuses gate topology, identity, package and unsupported fields,
ambiguous sources and noncanonical ownership. Only changed metadata fields enter
the patch. Unedited raw fields, ordered collections, unknown metadata and absent
defaults remain intact. Prefix revision does not renumber placed references or
update placed bindings. Cancel leaves all canonical files and journal unchanged;
Undo/Redo, Save and cold reopening preserve the corrected Entity and unchanged
gate/link IDs, linked definitions, Part and placed root/child circuitry. Twenty-two
focused checks and the isolated ordinary debug App's 67 steps and six inspected
captures establish this bounded behavior. The App uses presented controls and
programmatic logical keys; no physical/HUMAN, release-GUI execution, gate topology
policy or full library qualification is claimed.

Saved Entity gate Symbol correction (`dat-saved-entity-gate-symbol-0bmd`) adds
selection of an existing gate row and another compatible named Symbol for its
unchanged Unit. Only changed Symbol links and explicitly edited metadata enter the
canonical raw patch. Entity/gate UUIDs, Unit links, unedited raw fields, Part and
PinPadMap definitions and existing placed circuitry remain intact. Existing users
retain their authenticated effective placement or explicit assignment chain,
including supported Undo/Redo restoration; missing, ambiguous or mismatched history
and incompatible or stale references refuse before mutation. Later explicit
Part/gate placement uses the corrected current Symbol with revision guards.
Eight engine and five protocol checks and an isolated ordinary debug App's 80 steps
and seven inspected captures cover Preview, keyboard Cancel, commit, Undo/Redo,
Save/cold reopening, unchanged old root/child geometry and corrected later placement.
The isolated later-placement case retains existing Net IDs and membership, with
only its established canonical initial electrical lineage transition permitted. Gate names,
addition/removal, Unit or logical-pin topology, implicit placed updates/ECO and
approval remain outside this increment; no physical/HUMAN, release-GUI execution,
full qualification or broader-goal acceptance is claimed.

Public native Part JSON admission (`dat-part-raw-footprint-pair-yjc4`) now checks
that an explicit default Footprint belongs to the Part's selected Package in
`project create-pool-library-object` and `project set-pool-library-object` with
manual CLI provenance. The existing reference checks, canonical source-blocker
and shape-error priority remain. Matching defaults, explicit null and omitted
defaults, unrelated raw metadata and supported history/cold reopening are retained.
This corrects the existing manual admission helpers; raw canonical builders,
import/replay and other provenance semantics are not expanded or tightened.
Nine scoped engine/CLI checks pass. The original public CLI reproduced both
mismatches; the matching updated CLI refuses both with all 36 persisted files
unchanged per refusal. Compatible public creation/editing, Undo/Redo, fresh query,
null/omission and metadata checks finish with 14 CLI-sourced journal transactions.
Existing source-ownership errors can retain failed staging payloads; only those
legacy-error assertions exclude staging, while new mismatch refusals use the full
persisted snapshot. Earlier fixture, stack-environment and snapshot failures remain
recorded. PinPadMap ownership is covered by the subsequent
`dat-part-raw-map-owner-nb91` correction below; the earlier Footprint/Package proof
retains its original scope.
No GUI replay, universal raw-writer integrity or full qualification is supplied.

Public native Part default PinPadMap ownership (`dat-part-raw-map-owner-nb91`)
now requires the referenced Map's Part UUID to equal the candidate Part UUID for
manual CLI JSON Create and Set. This follows the existing LibraryGraph ownership
rule after the unchanged reference and default Footprint/Package checks. Raw
canonical/import/replay and other-provenance behavior remain unchanged. It adds
no Map topology or Footprint compatibility rule. Map Footprint/Package
compatibility is covered by the subsequent `dat-part-raw-map-physical-x00t`
correction below; this earlier ownership proof retains its original scope.
Fourteen scoped engine/CLI checks pass, including existing Part/Footprint
regressions and refusal of a stale Part batch after its Map owner changes while
the Part revision stays fixed. Actual public CLI Create and Set reproduced the
old foreign-owner admission; the updated normal CLI refuses both with all 50
persisted files unchanged, backed by saved before/after hash maps. Matching
existing Set, Undo/Redo, fresh queries, raw metadata and null/omitted new-Part
defaults pass, ending at 18 CLI-sourced journal rows. The Map source stays
byte-exact. Matching Map plus new-Part Create is not assumed as a circular
creation path. Old fixture operations/assertions are reused with visibility-only
exports. Existing source-blocker staging scope, GUI proof and full-qualification
boundaries remain; no GUI replay or universal raw-writer integrity is claimed.

Public native Part default Map physical compatibility
(`dat-part-raw-map-physical-x00t`) now checks a declared Map Footprint against
an explicit Part default Footprint and the selected Part Package, after the
unchanged reference, Footprint/Package and Map-owner checks. This follows existing
LibraryGraph rules using the existing materialized readers. Absent/null Map
Footprints remain optional; no topology, raw canonical/import/replay or other
provenance policy is added. Nineteen focused engine/CLI checks pass, including
existing reference/owner/Footprint regressions and a referenced Map change that
stales a prepared Part without changing the Part revision.
Actual public CLI Set reproduced both conflicts; the updated normal CLI refuses
both with all 58 persisted files unchanged per refusal. Matching Set, Undo/Redo,
cold queries, metadata and optional Part defaults pass. A separate continuation
uses existing public full-object Package Set with a fresh Package-owned pad UUID,
then real Map mappings with omitted/null Footprints and Part Set. Both pass, with
Map and Package source bytes restored exactly, ending at 25 CLI-sourced journal
rows. The primary 18-row result retains its original scope; the composite result
includes the continuation. The earlier legacy Package-pad adapter recipe failed
and remains recorded, without a whole-run PASS. An unexecuted test-fixture path
correction and driver-assembly failure are retained. Canonical empty-map fixtures
are supplemental, not public empty-map creation proof. No circular matching-Map
new-Part creation, GUI replay or full qualification is claimed.

The creation-only Part GUI increment (`dat-part-gui-create-6zs4`) adds explicit
manufacturer/MPN, value, description, datasheet reference and tags, with named
Entity/Package choices and an optional compatible default Footprint. A Package
change clears an incompatible Footprint selection; no default remains a genuine
absent link. The shared engine adapter independently checks kinds, references,
Footprint/Package compatibility and referenced revisions before canonical Create.
MPN supplies the Part display identity; no separate name field is introduced.
New Parts use the existing `Unknown` lifecycle, displayed read-only, without an
approval or Active claim. Existing map/model/parametric/packaging/inheritance
fields retain their schema-supported empty/absent creation representation.
Preview/Cancel, canonical Undo/Redo and Save/cold reopen preserve the exact Part
identity, entered metadata and chosen links while existing definitions and placed
circuit sources stay unchanged. Focused engine/form and isolated ordinary-App
proof covers both absent and compatible Footprint choices. That App proof uses
debug builds, presented controls and logical keys, not physical/HUMAN or release
qualification. Saved Part Entity/Package relinking, inheritance, placed pin-pad updates,
placement policy, models, approval and broader library completion remain open.
Generic raw Part/Footprint pair validation is separate intake
`dat-part-raw-footprint-pair-yjc4`; this increment supplies no universal raw-write
integrity claim.

Saved Entity gate-name correction (`dat-saved-entity-gate-name-8tdk`) adds the
nonempty display-name field for an existing selected gate. Explicit changes remain
available when navigating between gate rows, and Preview/Cancel, canonical
Undo/Redo and Save/cold reopening preserve that intent. Sparse `gate_names`
patches authenticate unique parsed UUID membership, retain original raw key
spelling and unknown data, and guard the actual Unit/Symbol sources. Only requested
name leaves change; gate UUIDs, membership and Unit/Symbol links stay intact.
Creation's nonempty-name semantics apply without a new uniqueness or normalization
policy. Saved gate Symbol correction and Entity metadata/creation remain supported.

Nineteen focused Rust cases and one ordinary isolated debug-App case pass.
The App completes 82 stages in 17.39 seconds with seven inspected captures,
including two-row editing, back-navigation, Preview/Cancel, history, cold reopening
and revised current gate-choice captions followed by Cancel. All existing
root/child bodies, pins, references, CIs, Maps and electrical source bytes remain
exact, with no Move or new-placement compensation. An actual connected eight-pad
`Complete` certificate and held/fresh membership remain unchanged through
successful rename, Undo/Redo and cold reopening.

The original test-index borrow compile error, duplicate-alias negative-fixture
assumption and App-test UUID parse-type compile error remain recorded. The general
writer can retain legacy gate-key aliases; the saved editor refuses ambiguous
parsed membership without rewriting that source. The Inspector path-label
report `dat-library-gate-inspector-path-06xw` was retracted after actual metadata,
shared-font and raster diagnostics: both original name rows are pixel-identical
to independent cold renders of their correct text. The original visual reading
was wrong; no production fix or renderer workaround was needed. That name-only
increment does not supply gate add/remove or Unit relinking, inferred placed
updates/ECO, approval, physical/HUMAN observation, release-GUI execution, complete
Inspector labeling, full library qualification or broader-goal acceptance.

Saved Entity gate Unit/Symbol correction (`dat-saved-entity-gate-unit-glgy`)
adds an explicit named Unit choice beside the compatible named Symbol choice for
an existing gate. Changing Unit retains the old Symbol visibly until the designer
chooses a compatible pair. Edits persist across rows; preview and the pending table
reflect both links. The engine composes the complete candidate before validation,
then applies sparse raw leaves with actual source revision guards. Untouched UUID
spellings, gate identity/membership and unknown data remain intact; a reviewed
counterpart-normalization defect is corrected with two alias-preservation controls.
Existing metadata, gate-name, Symbol-only and creation behavior remains supported.

The existing materialized graph validates every affected Part's retained default,
nondefault and legacy mappings against candidate Unit pins. Incompatible mappings
refuse atomically; compatible mappings on unchanged gates remain valid. Old placed
users retain their authenticated original Unit/Symbol chain across repeated relinks
and remain editable. Later explicit placement uses the revised pair. There is no
pin migration, implicit placed update, Map/default clearing, new writer, controller,
schema or historical Entity binding.

Twenty-three focused Rust cases and one ordinary isolated debug-App case pass.
The App completes 96 stages in 20.06 seconds with eight inspected captures. Its
canonical setup adds a distinct map-free Entity/Part, real original root/child
placements and a fresh compatible Unit/Symbol; before/after setup inventories and
journals distinguish the original 22 sources from the protected 30-source baseline.
During correction, Cancel, Undo/Redo and cold reopening, only the existing gate's
Unit/Symbol leaves change and the other 29 source/CI/pin/reference/Map/Net paths stay
byte-exact. A later explicit placement adds its actual Symbol/CI/Net and exact
canonical initial lineage transition separately. Real connected eight-pad Complete
refusals and repeated old-member editability have their focused engine proof.

Raw zero-match, obsolete/mapped-target fixture, pre-GUI missing-freeze and invalid
capture-label failures remain recorded; the mapped baseline App attempt correctly
refuses without changing its original 22 sources. Shared focus/reveal makes Commit
reachable, although the Preview-focused capture does not show that button. This
supplies no gate membership editing, placed migration/ECO, approval, physical/HUMAN
observation, release-GUI execution, full library qualification or broader-goal
acceptance.

Saved Entity gate addition (`dat-saved-entity-add-gate-3pjh`) exposes Add gate in
the existing shared form. A new draft gate has a stable fresh UUID and explicit
name, named Unit and compatible named Symbol. Intent persists across old/new row
selection; Preview/Cancel, canonical Undo/Redo and Save/cold reopening retain it.
The sparse `gate_additions` patch rejects globally owned, duplicate or aliased
collisions, preserves every existing raw gate and other untouched field, and uses
the canonical Set registration for new nested IDs. Existing name/Unit/Symbol edits,
creation and shared keyboard focus/reveal remain supported. This increment does
not enable removal of existing saved gates.

The complete candidate uses the existing materialized graph and revision guards;
old default/nondefault and legacy mappings retain authority. A successful addition
preserves an actual connected eight-pad Complete certificate and held/fresh old
membership through history and cold reopening. Twenty-six focused Rust cases and
one ordinary isolated debug-App case pass: 102 stages, 23.21 seconds and nine
inspected captures. The original 22-source/20-transaction circuit needs no setup.
During the library addition, only one new Gate enters the Entity and all other 21
source/CI/Map/Net paths, old gate links, pins and references stay byte-exact.

The new gate is then placed on an existing child component through the actual
marking-menu More action, remaining-gate picker, reference field, ghost and
Proposal review/acceptance. Three separate canonical transactions record Proposal
creation, acceptance and application; they add the child Symbol, Net identity and
applied Proposal while extending the existing CI. Old members, roles and Part
bindings remain exact; the sole old Net change is its actual canonical initial
lineage transition. There is no implicit placed update or Map/default migration.

The first App run remains a recorded failure: its test input incorrectly marked
fresh key presses as held-key focus replay, which production intentionally ignores.
The corrected test preserves the captured F6 template's fresh-event semantics and
asserts the entered reference; the production replay guard is unchanged. F6 proves
only a native template, and subsequent public WindowEvent dispatch is programmatic.
No physical/HUMAN observation, release-GUI execution, new controller/history binding,
full qualification or broader native-library acceptance is supplied.

Saved Entity gate removal (`dat-saved-entity-remove-gate-azc3`) now uses the
shared form to remove a selected unused saved gate or discard a new draft row.
Sparse `gate_removals` resolve one unique parsed UUID to its original raw key and
embedded identity, preserving all remaining raw gates, links and unknown fields.
Missing, duplicate or ambiguous identities refuse. Actual authored placements
refuse removal even after the current Part is relinked to another Entity; retained
default, nondefault and legacy mappings also refuse without writes. The existing
materialized source scan and graph validation supply these checks and revision
guards; authenticated history is never used to bypass a placed reference.

A draft may temporarily contain no gates, but removal Preview/Commit requires the
existing manual-authoring minimum in the complete prospective candidate. Explicit
removal and a distinct valid addition can compose a replacement. This does not
change the generic Entity schema, legacy empty data, raw Set or metadata editing.
Preview/Cancel, canonical nested-ID deletion/restoration, Undo/Redo and Save/cold
reopening preserve the other gates and circuit. Creation, name, Unit/Symbol and
Add gate workflows remain supported.

Thirty-three unique focused Rust cases and one ordinary isolated debug-App case
pass: 73 stages, 16.92 seconds and seven inspected captures. An actual connected
eight-pad Complete certificate and held/fresh membership survive successful unused
removal/history/cold; independent mapping refusals preserve the same authority.
The App circuit retains its original 22 files and 20-transaction prefix. One
separate canonical Add Gate fixture transaction supplies an unused gate; removal,
Undo and Redo then each contain only Set Entity, ending at 24 transactions. All
other 21 source/CI/Map/Net files remain byte-exact, with old root/child scenes
unchanged and the removed row absent after cold reopening. Presented controls and
the existing logical-key handler provide programmatic proof in owned Wayland.
No physical/HUMAN observation, release-GUI execution, implicit placed update, pin
migration, approval, full qualification or broader-goal acceptance is supplied.

Saved Part metadata editing (`dat-saved-part-metadata-edit-8xfy`) uses the same
Library form for manufacturer, MPN, value, description, datasheet reference and
tags. That metadata-only increment kept Entity, Package, default Footprint,
PinPadMap and lifecycle read-only.
Only changed metadata fields patch the raw Part through the canonical Set writer;
unedited fields, ordered collections, unknown metadata and absent defaults remain
intact. Identity, relationships and policy fields are refused independently by
the engine, along with ambiguous or noncanonical source ownership. Existing
component instances retain their exact declared Part revisions, values,
annotations and mappings; accepted canonical history supplies the exact pinned
Part payload, with source identity, kind, incarnation and recorded hashes still
checked. No latest-revision substitution or automatic placed update occurs.
New placement uses the current Part revision and value through the ordinary
explicit Part/gate choice. Physical materialization and its Undo/Redo retain
their existing separate explicit authority; pending net and pad validation uses
the unchanged accepted Part basis alongside the actual staged operations.
Forty-six focused checks cover metadata, historical references, forms and
connected physical materialization. The isolated ordinary debug App passes 99
steps with eight inspected captures, including byte-exact Cancel, metadata and
placement Undo/Redo, Save and cold reopening with old and new placements. An
earlier run stopped at a capture-label assertion before new placement; its
partial result is retained. Presented controls and programmatic logical input
establish this bounded behavior, without physical/HUMAN, release-GUI execution,
relinking, approval or full library qualification claims.

Saved Part default Footprint correction (`dat-saved-part-default-footprint-vsrh`)
adds a named same-Package choice and an explicit empty choice to that shared form.
Entity, Package, default PinPadMap, map definitions and lifecycle remain fixed.
Untouched absent, null or declared values retain their raw representation; an
explicit empty choice removes only `default_footprint`. Unavailable or ambiguous
declared identities stay visible and refuse resubmission rather than becoming an
implicit clear. Existing LibraryGraph rules validate every retained mapping
against the candidate effective Footprint or Package, with mapped Gate/Unit
dependencies and consumed source revisions qualified before commit. Missing
optional pad collections mean empty membership, not a validation exemption.
Conflicting retained Part counter reuse, including Undo/Redo compensation,
refuses before model, source or journal publication. Existing Package revision
and strict prospective Part-basis safeguards remain unchanged.

Nonempty engine proof preserves eight frozen pads, connected Net membership,
CI identity, mappings and source pins through change, clear, history and cold
reopening, including authenticated physical users without a CI. The actual CLI
materializes later placements from A, corrected B, then existing fallback A after
clear, while earlier placements remain frozen. The isolated ordinary debug App
passes 94 stages with seven inspected captures: both Cancel paths preserve files
and journal; change/clear, Undo/Redo, Save/cold reopening and old root/child scenes
retain exact circuit sources and Net membership. Earlier fixture, dependency
validation, revision-collision and App label-assertion failures retain their raw
results. This supplies no Entity/Package relinking, default-Map edit, implicit
placed update, approval, physical/HUMAN, release-GUI or full library qualification.

Saved Part default PinPadMap correction (`dat-saved-part-default-map-j4cc`)
adds named existing same-Part Map selection and explicit clear to the shared form.
A draft can change Footprint and Map together when their resulting effective
Footprint/Package pair passes the existing graph rules. Unavailable or incompatible
declared selections stay visible until explicitly corrected; neither selector
silently clears or infers the other. Clear removes only `default_pin_pad_map`;
omitted fields retain their raw absent, null or declared representation. Entity,
Package, UUIDs, Map payloads and every other unedited field remain fixed.
Mapped Gate/Unit and source revision guards, retained Part/Map history and atomic
counter-collision refusals protect existing users. Map creation's separate
assignment-on-creation restriction to unplaced Parts remains unchanged.

Thirty-five unique focused Rust checks pass, including eight connected frozen
pads, CI/mapping/source identity, Nets, history, cold reads and current-default
consumer coverage. The ordinary isolated debug App passes 123 stages with five
inspected captures for selection, composed defaults, all three Cancel paths,
Undo/Redo, Save/cold reopening and exact old root/child sources and scenes.
The CLI case captures the current Part, checks actual gate compatibility, then
explicitly declares that selected Map through existing CI authority and the mapped
materialization command; A/B/clear preserve earlier users and Map payloads.
Initial Part placement remains Part-only, with no automatic Map binding. Legacy
mapping precedence and consumer-specific empty/footprintless Map refusals remain;
no consumer inference or universal materialization guarantee is added. The first
CLI fixture run lacked a sheet; its failed log is retained, and the fixture now
creates one canonically. This supplies no placed update/ECO, Entity/Package
relinking, approval, physical/HUMAN, release-GUI or full library qualification.

Saved Part Entity correction (`dat-saved-part-entity-lo01`) adds a named existing
Entity choice to the shared form while keeping Package fixed. Explicit Entity,
Footprint and default-Map choices compose only when the resulting candidate passes
the existing graph rules. The engine qualifies unique typed Entity gates and their
Unit/Symbol sources, with consumed-source revision guards. Part identity, unedited
raw fields and Map definitions remain intact. Every retained same-Part Map,
including nondefault Maps, and legacy `pad_map` rows remain relevant. Incompatible
gate references refuse atomically even when the default Map is explicitly cleared;
this supplies no successful mapped-Part Entity migration.

Existing schematic components retain the Entity named by their declared Part
revision. Old multi-gate move, rotate, delete and remaining-gate choices use that
authenticated chain; later explicit initial Part/gate choices use the corrected
current Entity. Plain source correction, Cancel and Part Undo/Redo preserve all
old root/child source and CI bytes exactly. The App's subsequent child Move and
its history qualify actual CI/member revision counters only after the real Move
phase, preserving the declared Part pin, library bindings, roles and other data.

Forty-one focused Rust checks and one ordinary isolated debug-App case pass. The
App completes 86 stages with eight inspected captures, covering Preview/Cancel,
canonical Undo/Redo, Save/cold reopening, old component editing and new explicit
Entity placement. A connected eight-pad `Complete` certificate remains unchanged
on incompatible mapped-switch refusal. The gate regression batch retains the
known `dat-2glp` annotation assertion failure, with six other cases passing;
earlier fixture, shortcut, capture and assertion-review attempts retain their
original results. Package relinking, implicit placed updates/ECO, approval,
physical/HUMAN, release-GUI execution and full library qualification remain outside
this increment.

Saved Part Package correction (`dat-saved-part-package-4w84`) adds an explicit
existing named Package choice to the shared form. A compatible default Footprint
or an explicit empty choice can accompany it; incompatible declared Footprint
and Map choices stay visible until the designer corrects them. Only requested
leaves change. Every retained default/nondefault Map and legacy mapping remains
authoritative. An incompatible Map Footprint/Package relationship or absent
candidate Package pad membership refuses atomically, including when default
Footprint and Map are explicitly cleared. A connected eight-pad `Complete`
certificate, source and journal remain unchanged; no mapped migration is supplied.

Preview/Cancel and Part Undo/Redo preserve old root/child source and CI bytes
exactly. Old components retain their declared Part revision and editability.
The subsequent real child Move and four history transactions qualify only actual
CI/member counters and intended coordinates; cold reopening retains the original
Part pin and bindings. Later explicit placement uses current Part revision 3 and
its selected Package, without updating old users.

Thirty-two focused Rust cases and one ordinary isolated debug-App case pass.
The App completes 90 stages in 23.46 seconds with eight inspected captures.
The first form attempt retains its raw-versus-canonical UUID assertion failure;
the first App attempt retains its whole-Net-set assertion failure after legitimate
new placement. The corrected App checks every baseline Net membership individually
and preserves the strict source/CI comparisons. Known `dat-2glp` and `dat-6l8l`
boundaries remain separate. This supplies no implicit placed update/ECO, approval,
physical/HUMAN observation, release-GUI execution, full library qualification or
broader-goal acceptance.

The creation-only PinPadMap GUI increment (`dat-pin-pad-map-gui-create-emck`)
adds named Part and compatible explicit Footprint choices and a table assigning
Footprint pads to gate-qualified Unit pins. Unmapped pads remain visible;
nonempty partial maps are valid under the existing rules. Duplicate pads or
gate/pin pairs are refused, while the same Unit pin under distinct gates is valid.
Changing source choices clears incompatible draft relationships. The shared
engine adapter independently validates membership, kinds and referenced revisions
before canonical Create; it introduces neither name matching nor a separate map
name. Preview and Cancel leave source and journal unchanged.

Library authoring free-text and numeric fields (`dat-library-form-caret-editing-4x34`)
use the existing shared single-line editor and painter: pointer caret placement,
keyboard movement, selection, insertion/deletion and clipboard operations act at
the current position. Field text remains authoritative through the existing draft
change/validation route; caret-only changes retain previews and do not dirty the
draft. Field switches preserve their own selection, and rebinding a different
pin, terminal, pad or gate resets its transient editor state. Selectors retain
choice behavior even when their candidates are unavailable or empty. Existing
Preview/Commit/Cancel, canonical history and Save remain the mutation authority.
Focused checks cover Unicode boundaries and numeric correction; the ordinary App
checks name/numeric correction, selection/clipboard, cancellation, committed raw
payload, Undo/Redo and exact cold reopening. Its Greek U+03BC name round-trips
losslessly but currently displays a missing-glyph box in both field and generic
detail text; that separate rendering defect supplies no Unicode glyph-coverage
claim or new restriction on authored metadata. No separate text controller,
authored schema, default policy or live-source write path is added.

Saved PinPadMap browser captions (`dat-pin-pad-map-result-label-r0ic`) derive
readable Part and Footprint identities from their declared references. The same
caption serves result rows, selected detail and existing search. Exact source
kind/UUID candidates and existing warnings distinguish missing, malformed,
wrong-kind, ambiguous and unavailable relationships; an undeclared optional
Footprint stays explicitly absent. Captions never supply a preferred map or a
Part default as a substitute. Equal captions retain separate source keys, rows,
order and relationship targets. This disposable presentation adds no authored
map name or source write. Focused reference/search checks and ordinary isolated
App browsing, both relationship roles and cold reopening preserve all authored
files and journal; approval, compatibility and saved-map editing are unchanged.

An explicit default-assignment choice starts off. It may add a raw-preserving
Part Set to the same transaction only when the Part's explicit default Footprint
matches and canonical ownership evidence proves no authored component, package
or symbol declarations reference that Part, including unresolved persisted owners.
Missing, unsupported or ambiguous ownership evidence refuses assignment while
leaving map-only creation available. Canonical component-instance loading now
reports unavailable inventory, including unreadable entries and nonrepresentable
JSON paths, rather than treating those cases as proof of no users.
Focused engine/form and loader checks plus the isolated ordinary App prove
map-only and eligible atomic default creation, refusals, cancellation, Undo/Redo,
Save and cold reopening with stable map identities and links. Existing circuit
sources and unrelated definitions remain unchanged; optional assignment changes
only the selected Part's default-map field. The App proof uses debug builds,
presented controls and logical keys. Map relinking, placed updates, legacy
Package-pad authoring, full-coverage/large-map qualification, physical/HUMAN and
release qualification, models, approval and broader library completion remain open.

Saved Footprint-bound mapping revision (`dat-saved-pin-pad-map-edit-je1j`) reuses
the named pad / gate / logical-pin table and explicit unmap control. Part,
Footprint, UUID and Part defaults remain fixed; the engine refuses identity,
relationship and policy edits independently. A map with no declared Footprint
remains available for inspection, without substituting a Part default. Missing,
malformed and ambiguous row sources refuse editing. Nonempty partial maps retain
visible unmapped pads; no automatic assignment or manufacturing-ready claim is
introduced. Canonical Set patches assignments while preserving unrelated raw
fields, unknown cell metadata and unchanged UUID spellings; explicit unmap removes
only that selected cell.

Existing declared Map bindings, recorded physical source pins, logical terminals
and Net membership retain their exact authenticated map revisions through edits,
history and cold reopening. Held query snapshots retain their captured mapping.
Ordinary pending-source frames refuse declared Map reads rather than label
replacement cells as old revisions;
accepted frames still resolve their exact old and current payloads.
The current-map public engine producer is separately exercised with a new
component and explicit eight-pad materialization, a revised connected
correspondence, and subsequent populated-pad rematerialization Undo/Redo; the old
component and Net remain unchanged. This producer proof is not a manual App/CLI
materialization action. The bounded manual CLI follow-up is recorded below as
`dat-manual-map-materialization-bxrt`. Initial materialization Undo with an absent
prior pad collection failed in the original map-revision proof and was not
credited there; its follow-up correction is recorded below. Forty-five focused
checks establish these engine/form/history scopes. The isolated ordinary debug
App passes 69 steps and six inspected captures for assignment/unmap, Preview,
byte-exact Cancel, map-only commit, Undo/Redo, Save and cold reopening, including
unchanged root/child circuit rendering and resolved component identities. It uses
presented controls and logical keys, not physical/HUMAN or release-GUI execution.
Relinking, implicit placed updates, approval and full library-to-PCB qualification
remain open.

Initial mapped materialization history
(`dat-component-pad-initial-undo-absent-sxdi`) now preserves a genuinely absent
prior pad collection without inserting `null`, and preserves an explicitly empty
collection as empty. Redo restores the same eight requested Pad identities,
geometry and pinned source choices with only the expected compensation revision
advancement. Two repeated Undo/Redo cycles and cold reopening at both states
retain the other bound component/Net, source history and authored unknown
siblings, with no phantom Pad registrations. Malformed null, object and string
collections still refuse without source or journal writes. Eleven focused
engine cases cover this correction and affected Map/Part/materialization
regressions. This is public engine producer/history proof, without a manual
map-aware surface, App/CLI action, physical/HUMAN or release-GUI execution claim;
the original map-revision failure remains valid at its earlier candidate.

Manual mapped-pad materialization (`dat-manual-map-materialization-bxrt`) uses
the existing CLI Bind/Set ComponentInstance commands with an explicit
`--pin-pad-map`, and `project set-board-component-package` with paired
`--component-instance` and `--pin-pad-map` choices. The selected Map's exact
Footprint reference supplies the land pattern; no Part default or inferred
component/mapping substitutes for an absent choice. A Map without a Footprint
refuses on this route. Mapped Set with an omitted `--part` preserves the authored
Part reference, pinned revision and binding metadata. Existing placed Map target
or revision replacements refuse here and retain the reviewed library-update
boundary; absent pads do not make a component unplaced.

Explicit `--pad-correspondence <placed-pad>=<library-pad>` values capture target
pins through the public engine query and retain Pad identities through the
existing materialization producer. Referenced pads require valid continuity;
`--adopt-existing` retains the producer's explicit adoption rules. Profile 4 is
a precondition, with separately reported preparation through the existing CLI
placement path. Thirteen focused native cases and two actual CLI subprocess
cases pass. The CLI proof covers first materialization, two initial Undo/Redo
cycles and cold resolution, connected explicit refresh and its Undo/Redo,
preservation of the other bound component/Complete Net, exact omission pins,
new-component current-source selection, parameter help and no-write refusals.
The earlier engine-only proofs retain their original scopes; this adds headless
manual CLI evidence, without App, physical/HUMAN, release-GUI execution or full
library-to-PCB qualification.

The initial schematic handoff (`dat-explicit-part-gate-placement-b43w`) adds named
project-local Part/gate choices to the existing Symbol picker. Distinct choices
retain the actual shared Symbol UUID and display manufacturer/MPN, gate, Entity
and Symbol names. The native placement verb checks the exact selected chain and
captured revisions before one canonical Symbol/component transaction; explicit
choices never fall back to automatic Part inference. Root and unique-child
pointer/ghost/cancellation, placement, history and Save/cold-source checks retain
the chosen identities and pinned binding while existing library definitions,
default Footprint/maps and unrelated circuit data stay unchanged. Valid multi-gate
maps are preserved. This debug App proof does not establish physical/HUMAN,
release-GUI, approval, placed-update or broader library completion.

The remaining blocker is not "no substrate." It is that the library-specific
authority model is still split across engine structs, generic substrate
operations, CLI-side JSON constructors, MCP bridge methods, and ad hoc
validation. This contract therefore defines the implementation target:

- The engine must own a typed `LibraryGraph`/pool dependency contract rather
  than leaving dependency semantics in CLI validation only.
- `Package` is not a `Footprint`. `Package` carries the physical component
  body/terminal family; `Footprint` carries the PCB land pattern, courtyard,
  paste/mask process apertures, fabrication/assembly graphics, and pad
  geometry.
- `PinPadMap` is first-class library data. Its canonical mapping is
  gate-aware `pad -> {gate, pin}`, not `pin -> pad`, so multi-gate parts can
  bind the same unit pin to distinct physical pads. The legacy `Part.pad_map`
  shape is a compatibility/import convenience until migrated behind explicit
  `PinPadMap` objects.
- `Padstack` now has model fields for layer spans, copper aperture, drill,
  annular ring, thermal/anti-pad, plated state, and mask/paste policy; the
  remaining target is systematic consumption in footprint materialization,
  checks, standards repair, and fabrication outputs.
- `ModelAttachment` now exists for Part behavioural models with hash/provenance
  and review-state fields, but the product target is a governed attachment
  contract across Part/Package/Footprint targets. Bare path-only 3D `ModelRef`
  remains insufficient for that full target.
- Pool layering, duplicate UUID/shadowing, project-local overrides,
  automated-authoring proposal policy, binding materialization, and
  validation tiers must be defined once in the engine contract and consumed by
  CLI/MCP/GUI.

## Reference-Tool Survey (with the lean rationale)

The survey establishes what is load-bearing in professional library
tooling versus ceremony, and confirms the leanest write primitive.

- Altium Designer (primary reference). A unified `Component`
  (DbLib/SVNDb/Vault) links one `Symbol` + one-or-more `Footprint`s +
  `Parameters` + `Models` through the Models tab. A separate Footprint
  editor carries the IPC Compliant Footprint Wizard (IPC-7351); a Pad
  Stack editor carries per-layer copper/mask/paste and plated/non-plated
  state; symbols can be multi-part; managed content runs a lifecycle
  (New -> In Production -> Deprecated -> Obsolete) with release and
  where-used impact. Load-bearing: explicit symbol/footprint/pad-map
  separation, an IPC generator, a per-layer padstack, model attach,
  revision + lifecycle, and where-used impact. Ceremony: the SVN/vault
  server and the separate DbLib/IntLib/SchLib/PcbLib file zoo.
  Critically, Altium exposes ONE "Add Model" affordance on the Models
  tab (one verb, many model roles) — direct evidence that attach-model
  is NOT a top-level tool.
- KiCad. Symbol Editor (`.kicad_sym`, multi-unit, pin electrical type),
  Footprint Editor (`.kicad_mod`, F.Cu/F.Mask/F.Paste, pad shapes, 3D
  attach), Python Footprint Wizards (IPC-ish generators), symbol ->
  footprint via the `Footprint` field + association tables, and the
  Database Library (`.kicad_dbl`). Load-bearing: the same symbol/
  footprint/pin-map separation, layer-per-aperture pads, and scripted
  generators. Gap vs Decision 008: no first-class part revision,
  lifecycle, provenance, or approval.
- OrCAD / Cadence (Allegro). Part Developer with a cell + symbol +
  package split; a padstack editor with explicit regular/thermal/
  anti-pad plus drill; a package symbol with pin-to-pad map.
  Load-bearing: rigorous padstack semantics (anti-pad/thermal) and an
  explicit package-vs-symbol-vs-part split. Ceremony: heavyweight
  separate editors, license-gated.
- Horizon EDA. Pool `Unit` -> `Symbol` -> `Entity(Gate)` ->
  `Package(Padstack)` -> `Part` with UUID identity, reusable padstacks,
  and part inheritance via a base — the exact lineage of
  `crates/engine/src/pool/mod.rs`. Note: Horizon FUSES the land pattern
  into `Package`; Decision 008 instead SPLITS `Footprint` from
  `Package` (the Altium/KiCad/OrCAD posture).

Lean rationale extracted from the survey: every tool above ships
separate EDITORS (UI surfaces) per object kind, but the underlying write
primitive is uniform — create or edit a typed library object. Altium's
single "Add Model" verb and the universal symbol/footprint/pad-map split
confirm that per-kind and per-field tools are UI ceremony, not distinct
write capabilities. Datum therefore collapses authoring to ONE
parameterized create/edit verb, keeps the genuinely distinct surfaces
(approval guardrail, ComponentInstance binding, standards synthesis) as
their own verbs, and omits the rest.

## Tool Inventory

The shared verbs (session/context, query, run-check, propose, commit,
artifact, journal/undo-redo) are NOT restated here. Each tool below
lists ONLY its domain-specific typed Operation variants and the
domain-specific AI query context. The eight questions are labeled
fields.

### Tool 1 — `library.author` (create/edit ALL library DomainObjects)

Covers `Symbol`, `Footprint`, `Package`, `Padstack`, `Part`,
`PinPadMap`, `ModelAttachment` — the seven equal Decision 008
DomainObjects, each with `ObjectId`/`object_revision`.

1. Manual UI action: a library projection with one create-or-edit
   surface per object kind. Symbol editor (pins/units/graphics/fields/
   style-profile assertions); Footprint editor (pads referencing
   padstacks; copper/mask/paste/courtyard/fab/silk/mech geometry; origin;
   process-aperture policy); Package form (family/dims/tolerances/body
   height/mounting/model refs); Padstack editor (per-layer copper plus
   mask/paste/drill/annular policy); Part form (MPN/lifecycle/parametrics/
   default symbol + footprint + pinmap); a Pin/Pad map table
   (`logical_pin_id` -> `footprint_pad_id` [+ `package_terminal_id`/
   `electrical_role`/`variant_condition`]); and a Models-tab row
   (role/format/content + hash/transform). Each field edit is a local,
   visible, undoable edit. Editing an UNPLACED object commits directly;
   editing an object that has any `LibraryBinding` to a placed
   `ComponentInstance` is forced proposal-first (Decision 008
   lines 99-100, 431-434).
2. Current engine mechanism: `api/native_write/library.rs` supplies
   `PoolLibraryObjectTarget` and `PoolLibraryOperationSpec` create/set/delete
   and Part-model attach/detach inputs through `BatchComposer`/`PreparedWrite`
   and the canonical commit/journal path. Footprint and gate-aware map builders
   use that same owner; no new writer module is required. The normalized
   `CreateLibraryObject{kind, fields}` / `EditLibraryObjectField{object_id, patch}`
   vocabulary below is the target authoring surface, not a claim those exact
   API variants already exist. Current operations preserve UUID ownership,
   bump applicable revisions and record canonical inverses/history. PinPadMap
   reference resolution, process-aperture
   (mask/paste) policy, and model hash-on-create are kind-specific
   validation INSIDE these two ops — not separate verbs. Unplaced edit
   = direct commit; placed-binding edit = proposal-first via the shared
   Propose verb.
3. CLI command: `datum-eda library create
   <symbol|footprint|package|padstack|part|pinpadmap|model>
   --pool <ref> [--from-json <file>] [--from <object_id>]`;
   `datum-eda library edit <object_id> --set <field>=<value> |
   --patch <file>`.
4. MCP/AI tool: `library_create_object{kind, pool_ref, fields, from?}`;
   `library_edit_object{object_id, patch}`. MCP write tools land ONLY
   behind `commit()` so AI authoring is never an unjournaled surface.
5. AI query/context needed: target pool ref (reusable vs project-local),
   the schema for the kind, current `model_revision`, and the resolved
  `object_id`s referenced — a Footprint needs valid `padstack_ref`s + a
  `Package` ref; a Part needs default symbol/footprint context and
  `Part.default_pin_pad_map`; a
  PinPadMap needs symbol pin ids + names, footprint pad ids + names, and
  package terminal ids; a ModelAttachment needs `target_object_id` plus
   declared format/role and a data-egress/encrypted-content policy.
   Sourced via the existing read tools (`search_pool`/`get_part`/
   `get_package`/`get_symbols`) folded under `datum.query.library`.
   AI-authored content (especially name-matched PinPadMap rows or
   datasheet-extracted models) is marked `provenance=AI`,
   `review_state=NeedsReview`, and accepted only via a proposal.
   The engine commit gateway enforces this boundary for current pool
   operations: `Tool`/`Assistant` provenance cannot directly create/delete
   typed pool packages or padstacks, create/set/delete generic pool objects,
   or attach/detach pool part models; those batches fail with
   `proposal_required_for_automated_library_operation` before shard staging.
   MCP-backed library authoring launches the CLI with
   `DATUM_COMMIT_SOURCE=tool`, so the bridge cannot accidentally downgrade an
   AI/tool write into manual CLI provenance. The positive review path starts
   with raw library objects through `datum-eda proposal
   create-pool-library-object` / `datum.proposal.create_pool_library_object`
   and the first semantic typed producers through `datum-eda proposal
   create-pool-unit` / `datum.proposal.create_pool_unit` and
   `datum-eda proposal create-pool-symbol` /
   `datum.proposal.create_pool_symbol`, then
   `datum-eda proposal create-pool-entity` /
   `datum.proposal.create_pool_entity`, and typed padstacks through
   `datum-eda proposal create-pool-padstack` /
   `datum.proposal.create_pool_padstack`, then typed packages through
   `datum-eda proposal create-pool-package` /
   `datum.proposal.create_pool_package`; legacy package-pad compatibility
   remains callable through
   `datum-eda proposal set-pool-package-pad` /
   `datum.proposal.set_pool_package_pad`, first-class Footprints through
   `datum-eda proposal create-pool-footprint` /
   `datum.proposal.create_pool_footprint`, and new land-pattern pads through
   `datum-eda proposal set-pool-footprint-pad` /
   `datum.proposal.set_pool_footprint_pad`, Footprint courtyards through
   `datum-eda proposal set-pool-footprint-courtyard-rect` /
   `datum.proposal.set_pool_footprint_courtyard_rect` and
   `datum-eda proposal set-pool-footprint-courtyard-polygon` /
   `datum.proposal.set_pool_footprint_courtyard_polygon`, Footprint
   silkscreen lines/rectangles/circles/polygons through `datum-eda proposal
   add-pool-footprint-silkscreen-line`,
   `add-pool-footprint-silkscreen-rect`,
   `add-pool-footprint-silkscreen-circle`, and
   `add-pool-footprint-silkscreen-polygon` /
   `datum.proposal.add_pool_footprint_silkscreen_line`,
   `datum.proposal.add_pool_footprint_silkscreen_rect`,
   `datum.proposal.add_pool_footprint_silkscreen_circle`, and
   `datum.proposal.add_pool_footprint_silkscreen_polygon`, plus compatibility
   package courtyards through
   `datum-eda proposal set-pool-package-courtyard-rect` /
   `datum.proposal.set_pool_package_courtyard_rect` and
   `datum-eda proposal set-pool-package-courtyard-polygon` /
   `datum.proposal.set_pool_package_courtyard_polygon`; all persist only
   proposal metadata until accept/apply.
6. Validating checks: per-kind schema/required-field validation;
   referential integrity (Footprint pads resolve to existing Padstacks
   + a Package; Part default refs resolve; every PinPadMap
   `logical_pin_id` resolves in `symbol_ref` and every `footprint_pad_id`
   in `footprint_ref`, unmapped pins warned, duplicate pad targets error,
   `variant_condition` references only authored `Fitted`/`Unfitted`
   overlay and never derived `NotApplicableForVariant`; ModelAttachment
   `content_hash` computed + stored, format/role enum validated,
   encrypted content flagged `ImportPreserved` and never decrypted);
   `ObjectId` uniqueness; `object_revision` monotonicity; geometry
   sanity (closed courtyard polygon, finite pad positions; mask/paste
   policy explicit per Decision 008 lines 300-306). The `ProjectResolver`
   re-validates references and placed-binding impact at `commit()`. Runs
   through the shared run-check verb.
7. Proof slice: First Proof Slice steps 1-2 — create one Part + Symbol +
   Package + Footprint + one Padstack family + one PinPadMap + (optional)
   one ModelAttachment; store IPC/process basis + provenance on the
   footprint/padstack; edit a field; confirm the journal records each
   transaction, `object_revision` bumps, and undo restores the prior
   revision.
8. Explicitly not-supported-yet: IPC density-level auto-generation (that
   is Tool 4); model execution/decryption; multi-format symbol style
   rendering (IEEE 315 / IEC 60617 — `style_profile_assertions` are
   stored, not rendered); automatic differential-pair/bus pin-group
   inference and pin-swap-group-driven PinPadMap remap.

### Tool 2 — `library.set_provenance_state` (approval / review / lifecycle / unknown-basis marking)

Survives as a distinct verb ONLY because it carries a hard approval
guardrail that a routine field edit lacks.

1. Manual UI action: a status control on any library object — set
   `approval_state` (`Approved|Deprecated|Imported|UnknownBasis|
   NeedsReview`), `lifecycle`, and unknown-basis flags. Shown as a badge
   in part-assignment pickers so designers see approved vs imported vs
   unknown before placing.
2. Operation it emits: `SetLibraryReviewState{object_id, approval_state,
   review_state, lifecycle?, unknown_basis_flags[]}` typed Operation via
   `commit()`; bumps `object_revision`, journaled, records the approval
   actor in provenance. Kept distinct from `EditLibraryObjectField`
   because it carries the hard approval guardrail (AI may NEVER
   auto-approve; state-transition legality) absent from a routine field
   edit.
3. CLI command: `datum-eda library set-state <object_id>
   --approval <approved|deprecated|imported|unknown|needs-review>
   [--lifecycle <...>]`.
4. MCP/AI tool: `library_set_review_state{object_id, approval_state,
   review_state?, lifecycle?}` — `Approved` requires explicit human
   acceptance; AI auto-approve is hard-blocked.
5. AI query/context needed: `object_id`, current provenance/
   `review_state`, and whether the actor may approve. AI surfaces a
   proposal via the shared Propose verb; human acceptance is the
   transaction.
6. Validating checks: state-transition legality (cannot `Approve` an
   object with unresolved `UnknownBasis` flags without an accepted
   deviation); a placement gate (configurable check warns/blocks placing
   non-`Approved` parts — strictness is an Open Owner Question);
   approval actor recorded in provenance; AI auto-approve hard-blocked.
7. Proof slice: First Proof Slice step 4 — a seeded imported object
   retains `Imported`/`UnknownBasis` markers; an approval proposal flips
   it only on human acceptance.
8. Explicitly not-supported-yet: org-level approval workflow / sign-off
   chains; where-used release gating across multiple projects.

### Tool 3 — `library.bind` / `library.update_binding` (ComponentInstance join + ECO propagation)

The `ComponentInstance` electrical-to-physical join and proposal-first
library update.

1. Manual UI action: placing a part creates a `ComponentInstance` and a
   `LibraryBinding` pinning the resolved `object_revision`; both
   `PlacedSymbol` and `PlacedPackage` reference that `ComponentInstance`.
   "Update from library" shows a reviewable before/after-revision diff —
   never silent latest-wins.
2. Operation it emits:
   `BindComponentInstance{component_instance_id, part_ref, symbol_ref,
   footprint_ref, pinpadmap_ref, pinned_object_revision}` commits
   directly for a fresh placement (local, undoable).
   `UpdateLibraryBinding{binding_id, from_revision, to_revision}` is
   PROPOSAL-FIRST (cross-domain: touches placed boards/checks/artifacts),
   applied via `commit()` only on acceptance. Both depend on the
   `ComponentInstance` + `commit()`/journal substrate, which now exists; the
   remaining gap is the library-specific binding operation family above it.
3. CLI command: `datum-eda library bind <component_instance_id>
   --part <id> [--symbol <id> --footprint <id> --pinmap <id>]`;
   `datum-eda library update-binding <binding_id> --to-revision <rev>`
   (emits a proposal).
4. MCP/AI tool: `library_bind_instance{component_instance_id, part_ref,
   ...}`; `library_propose_binding_update{binding_id, to_revision}`.
5. AI query/context needed: the `ComponentInstance` id (the canonical
   electrical-to-physical join — never refdes/name/path), the current
   pinned revision, the target object + latest revision, and the
   where-used set so the proposal shows impact across boards/panels/
   checks/artifacts. Sourced via `datum.query.relationships` /
   `provenance.query`.
6. Validating checks: the `ComponentInstance` is referenced by both
   `PlacedSymbol` and `PlacedPackage`; `pinned_object_revision` exists;
   the binding-update proposal runs affected-object checks (ERC/DRC/
   footprint-fit) via the shared run-check verb before acceptance; no
   silent global mutation — propagation is journaled per accepted
   proposal.
7. Proof slice: First Proof Slice steps 3 + 6 — place the Part as a
   `ComponentInstance` binding schematic + PCB to one identity; show a
   reviewable proposal for a library correction that mutates placed
   objects only on acceptance.
8. Explicitly not-supported-yet: cross-project ECO fan-out; automatic
   bulk re-bind without per-board review (withheld by the proposal-first
   invariant).

### Tool 4 — `library.generate_ipc_footprint` (standards-driven synthesis)

Genuine geometry synthesis from dimensions + density level, not a field
edit.

1. Manual UI action: an IPC footprint wizard — choose a package family +
   density level (A/B/C), enter source body dimensions/tolerances/lead
   geometry, generate a Footprint + Padstack family with recorded
   `standards_basis`, courtyard/mask/paste policy, and J-values.
   Generated content is `review_state=Generated`.
2. Operation it emits: `GenerateIpcFootprint{package_ref, density_level,
   source_dimensions}` emits an `OperationBatch` (Footprint + Padstacks +
   provenance) as a PROPOSAL (batch + standards-bearing); on acceptance
   it commits through `commit()`/journal. This is synthesis of new
   geometry, not a field edit, so it stays distinct. Process-aperture
   CORRECTIONS (mask/paste policy on an EXISTING footprint/padstack) are
   NOT here — they are `EditLibraryObjectField` on Tool 1 (proposal-first
   when placed).
3. CLI command: `datum-eda library gen-ipc-footprint --package <id>
   --density <A|B|C> --dims <file.json>` (emits a proposal).
4. MCP/AI tool: `library_propose_ipc_footprint{package_ref,
   density_level, source_dimensions}`. Current public slices expose
   `datum.library.generate_ipc7351b_soic` for direct journaled authoring and
   `datum.proposal.generate_ipc7351b_soic` for proposal-first review, alongside
   the existing two-terminal chip slice.
5. AI query/context needed: package family + body dimensions/tolerances/
   lead geometry, the target IPC basis (7351B vs 7352 is an Open Owner
   Question), and the density level.
6. Validating checks: generated geometry validated against the declared
   basis; compliance state recorded (`Compliant |
   CompliantWithDeviation | NonCompliant | UnknownBasis`); a generated
   pad with copper but no mask/paste basis is flagged unknown-basis;
   `deviation_refs` recorded when intentionally off-basis. Runs through
   the shared run-check verb.
7. Proof slice: First Proof Slice steps 2 + 5 — store IPC/process basis
   on the generated footprint/padstack; the check reports one pad/mask/
   paste policy mismatch.
8. Explicitly not-supported-yet: full IPC-7351 package-family coverage
   (start one family per Open Owner Question); IPC-7352 alternative
   naming; thermal-relief/anti-pad solver-driven optimization.

## Minimal-Set Recommendation

FOUR load-bearing tools cover the entire Decision 008 surface — down
from a naive six-or-more catalog:

1. `library.author` — ONE create/edit verb parameterized by object kind.
   Decision 008 (lines 157-374) makes `Symbol`, `Footprint`, `Package`,
   `Padstack`, `Part`, `PinPadMap`, AND `ModelAttachment` equal
   DomainObjects with `ObjectId`/`object_revision`; they share one
   operation shape (`CreateLibraryObject{kind,fields}` /
   `EditLibraryObjectField`), so they share one tool. Per-kind validation
   (PinPadMap ref-resolution, ModelAttachment hashing, Footprint
   `padstack_ref`s) is create/edit logic, not a reason for a separate
   verb. Process-aperture corrections are mask/paste-policy field edits =
   `EditLibraryObjectField`, proposal-first when placed.
2. `library.set_provenance_state` — survives ONLY because it carries the
   hard approval guardrail (AI-never-auto-approves; state-transition
   legality) that a plain field edit does not.
3. `library.bind` / `library.update_binding` — the `ComponentInstance`
   electrical-to-physical join plus proposal-first ECO propagation.
4. `library.generate_ipc_footprint` — genuine standards SYNTHESIS (new
   geometry from dimensions + density, a batch proposal), not a field
   edit.

Everything reduces to typed Operations on the single `commit()`/journal
path; batch/cross-domain/standards/approval actions are proposals; local
unplaced field edits commit directly. This is the minimum that answers
every professional library question in Decision 008 without a per-field
or per-kind tool catalog. Decision 008 (lines 50, 67-74, 246-277)
explicitly SPLITS `Footprint` from `Package`, so `library.author` has
SEVEN kinds (not five or six) and the pool schema must add a `Footprint`
object.

## Omitted / Redundant Tools

Each omission is a deliberate defect-avoidance cut, justified against
real-world practice.

- `library.pin_pad_map` as its own tool. PinPadMap is one of the seven
  equal DomainObjects (Decision 008 lines 308-343) with the same
  `ObjectId`/`object_revision` as every other kind. Its "distinct
  validation" (ref-resolution, `variant_condition`) is per-kind
  create/edit logic — every kind has distinct validation. Product-level AI/MCP
  authoring should fold it into `library.author` as `kind=PinPadMap`; the
  current CLI also exposes typed `create-pool-pin-pad-map` /
  `set-pool-pin-pad-map` helpers as transitional ergonomic commands over the
  same first-class pool object.
- `library.attach_model` as its own tool. `ModelAttachment` is a
  first-class DomainObject (Decision 008 lines 345-374) whose
  `target_object_id` points at a Part/Package/Footprint. Attaching =
  `CreateLibraryObject{kind:ModelAttachment, fields}`; `content_hash`
  computation is kind-specific create logic. Altium itself exposes ONE
  "Add Model" affordance for all model roles. Folded into
  `library.author`.
- Process-aperture correction (`SetPadProcessAperture` /
  `ApplyFootprintProcessPolicy`) bundled into the IPC generator.
  Mask/paste policy is a field on Padstack/Footprint (Decision 008
  lines 290-306, 458-462). Correcting it is `EditLibraryObjectField`
  (proposal-first when placed), not synthesis, and does not belong with
  the geometry generator. Folded into `library.author` edit; the
  generator keeps only true synthesis.
- Separate `create_symbol` / `create_footprint` / `create_package` /
  `create_padstack` / `create_part` tools. Collapsed into
  `library.author` with a `kind` parameter — identical operation shape.
  Altium/KiCad/OrCAD have separate EDITORS (UI), but the write primitive
  is one.
- `duplicate_library_object` / clone. Duplicate = `library.author`
  create seeded from an existing object via `--from <object_id>` (read
  via `get_part`/`get_package`, mint a new `ObjectId`). Identity is never
  the name (Decision 008 lines 62-65).
- `rename` / move-between-pools. Pool path and name are searchable
  labels, not identity (Decision 008 lines 62-65). Rename =
  `EditLibraryObjectField` on `display_name`; pool change = a
  `LibraryBinding` `pool_ref` edit. `ObjectId` preserves identity, so no
  dedicated tool.
- `delete_library_object`. Soft-delete = set `approval_state=Deprecated`
  via `library.set_provenance_state`; hard delete is the generic
  engine-wide `DeleteObjects{ids}` op (shared surface) gated by
  referential-integrity checks. Where-used is a query, not a write tool.
  A library-specific delete adds nothing.
- `set_lifecycle` as its own tool. Folded into
  `library.set_provenance_state`; lifecycle and approval/review are the
  same guardrailed status surface on one object.
- `import_library` as a library-domain tool. Imported-library handling
  is the IMPORT domain's session / Import-Map (`import_key`) mechanism
  (Decision 008 lines 40-45, 403-409; Decision 011), surfaced to library
  only as provenance + proposals. The implemented project-local Eagle
  `.lbr` seed path is `datum-eda project import-eagle-library`, which
  journals pool objects plus Import Map provenance; it is not a separate
  library-authoring verb. Library needs only `set_provenance_state` +
  `bind` to consume import results.
- `search_pool` / `get_part` / `get_package` / `get_symbols` as NEW
  tools. Already implemented (`project_surface.rs`; `dispatch.rs`). They
  are the AI query-context source for the authoring tools and fold under
  the shared query namespace — not new tools.

## Shared Surface

This contract uses the seven shared operations defined ONCE in
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` and does not restate them.
Library-specific use:

- Session/Context (`datum-eda context get|refresh`): every library
  invocation runs inside a `DatumToolSession` carrying the
  `model_revision` snapshot; `library.author`/`bind`/`update_binding`
  refresh before propose/apply when stale.
- Query (`datum-eda query --family library` / `datum.query.library`,
  `relationships`, `provenance`): the AI query-context source for all
  four tools; folds the existing `search_pool`/`get_part`/`get_package`/
  `get_symbols` reads. Cross-domain joins key on `ComponentInstance`;
  imported identity resolves via `import_key`.
- Run-check (`datum-eda check run`): per-kind library validation, binding
  affected-object checks, and IPC compliance checks run here, not as
  library-private verbs.
- Propose (`datum-eda proposal create|show|validate|apply`): the apply path
  for placed-binding edits, `UpdateLibraryBinding`, `GenerateIpcFootprint`
  batches, and all AI-originated library mutations. The four tools above
  are proposal PRODUCERS, not new mutation primitives.
- Commit (`commit()` — the only mutation gateway): every library
  Operation/OperationBatch flows through it; there is no library-private
  save/write path.
- Artifact: library objects are not artifacts; they are source. Where-used
  and BOM consumption of library identity are artifact/query reads
  keyed by `ComponentInstance`.
- Journal + Undo/Redo (`datum-eda journal`): library transactions appear in
  the global journal; removal of a created object is undo of the
  `CreateLibraryObject` transaction, not a per-domain delete/reversal
  verb.

## Proof Slice & Fixture

Two fixtures.

1. Existing read / board-consume path (current capability, verified):
   an Eagle `.lbr` imported into the in-memory pool exercises
   `search_pool`/`get_part`/`assign_part`
   (`crates/engine/src/import/eagle/pool_builder.rs` +
   `crates/engine-daemon/src/dispatch.rs`). This proves what works today.
2. Decision 008 First Proof Slice (native authoring + ComponentInstance
   bind + import re-association): use the canonical `datum-test` fixture
   at `~/Documents/kicad_projects/Datum-eda/datum-test/` (`datum-test.
   kicad_pcb` + `.kicad_sch` + `.kicad_pro`). Author one native Part +
   Symbol + Package + Footprint + Padstack + PinPadMap (+ optional
   ModelAttachment), place it as a `ComponentInstance` into `datum-test`,
   then re-import the KiCad source and confirm re-association via Import
   Map `import_key` (NOT `source_hash`) with preserved provenance /
   unknown-basis markers.

Fixture 2 is now partially executable: the shared journal/commit substrate,
`ComponentInstance`, Import Map, resolver pool-shard discovery, raw library
object operations, typed CLI producers, MCP bridges, real `Footprint` pool
payload validation, body-oriented `Package` fields, richer `Padstack` fields,
and Part model attachments exist. The fixture is not complete until
`PinPadMap` authority is fully migrated beyond the current runtime-preferred
`Part.default_pin_pad_map` slice and footprint-first runtime board pad
regeneration, legacy package land-pattern compatibility is migrated behind
explicit policy, model attachments are governed consistently across library
targets, and the first engine-owned `LibraryGraph` diagnostic seam expands into
one resolver/commit/validate dependency contract instead of only projecting
engine diagnostics through `project validate`.

## Not-Yet-Supported / Not-Yet-Systematic

- Full `Footprint` authority. `Footprint` is now a real pool type and
  `project validate` checks footprint package refs plus footprint-pad padstack
  refs. Runtime board pad regeneration now prefers first-class `Footprint`
  land-pattern pads through `Part.default_footprint`, `PinPadMap.footprint`, or
  a unique package-matching footprint before falling back to legacy
  `Package.pads`. Typed CLI/MCP authoring can now create first-class
  Footprints, set Footprint pads directly, and set rectangular/polygon
  Footprint courtyards plus Footprint silkscreen lines/rectangles/circles/polygons
  directly. Manual CLI graphics additionally target fabrication for those four
  primitives through the same actions and canonical history. Importers and
  compatibility materialization still preserve legacy package land-pattern
  fields in some paths, and footprint fab/silkscreen primitives beyond
  lines/rectangles/circles/polygons, assembly/mechanical/model
  authoring remains pending. The target schema separates
  component body (`Package`) from board land pattern (`Footprint`) without
  relying on compatibility fallback.
- Engine-owned `LibraryGraph` authority. The code has pool structs,
  resolver-discovered shards, write-time shape validation, and a first
  engine-owned dependency diagnostic seam consumed by `project validate`, but
  no single engine-level resolved graph governing resolver, commit-time,
  CLI/MCP/GUI, duplicate/shadowing, and materialization policy.
- First-class gate-aware `PinPadMap` authority. `pool/pin_pad_maps` now has direct typed
  CLI authoring and validation, and runtime part-compatibility signatures plus
  component-pad net remapping prefer a valid `Part.default_pin_pad_map`
  resolved through `Pool.pin_pad_maps`. Legacy-named part pad-map commands now
  bridge compatibility inputs into that default first-class map and do not
  write `Part.pad_map`. `Part.pad_map` still coexists for legacy imports and
  fallback when a first-class map is absent or unusable; it does not override
  usable `Footprint` / `PinPadMap` authority. The remaining target is to retire
  `Part.pad_map` to migrated import input behind explicit policy.
- Padstack process-policy consumption. PM061 supplies bounded native effective
  process inspection, explicit placed materialization/update, checks, retained
  workspace rendering and exact supported fabrication consumption. Broader
  generation/repair coverage, unsupported output geometries and full
  qualification remain open.
- `ModelAttachment` as governed library data across targets. Existing package
  `ModelRef`, content-addressed model blobs, and part behavioural-model
  attachments are useful slices, but the target needs one hash/provenance/role/
  review-state contract consistently across Part, Package, and Footprint
  targets.
- `LibraryBinding` / update-binding operations. `ComponentInstance` now carries
  Rust `LibraryBinding` records that can pin Part, Symbol, Package, Footprint,
  PinPadMap, and model roles, with resolver/commit validation and
  backward-compatible `part_ref` projection. The remaining target is the public
  update-binding operation family and proposal-first ECO propagation for placed
  bindings.
- Pool layering and override policy. Priority ordering exists, but duplicate
  UUID/shadowing, same-UUID override versus fork semantics, writable pool
  targets, conflict diagnostics, and project-local override records need one
  normative rule set.
- Validation tiers. Commit-time rejection, resolver diagnostics, and
  `project validate` findings are not yet a single documented contract.
- IPC footprint generation. IPC-7351B two-terminal chip and SOIC vertical slices
  now exist through engine generators, journaled CLI writes, proposal creation,
  MCP aliases, and LibraryGraph basis/process-policy validation. Full IPC-7351/
  7352 family coverage, naming policy alternatives, thermal/anti-pad synthesis,
  check-run findings, deviations, export consumption, and import audit remain
  future work.
- Multi-format symbol style rendering (IEEE 315 / IEC 60617), automatic
  diff-pair/bus pin-group inference, and cross-project ECO fan-out.

## Open Owner Questions

1. Which library pool types ship first: bundled-Datum, user-local,
   organization, project-local, imported, vendor-derived? This sets the
   `pool_ref` enum and `LibraryBinding` layering.
2. Which `approval_state` values gate placement, and is the placement
   gate a hard block or a warning by default?
3. IPC-7351B vs IPC-7352 as the default generated-footprint naming/basis
   beyond the landed IPC-7351B two-terminal and SOIC slices, and which next
   package family the generator supports?
4. How strict should default checks be for imported `UnknownBasis`
   library data (warn vs fail)?
5. Boundary for `EditLibraryObjectField`: the decision says library
   changes touching placed objects are proposal-first (lines 99-100,
   431-434). Confirm the rule: editing an UNPLACED library object commits
   directly (local); editing an object that has any `LibraryBinding` to a
   placed `ComponentInstance` is forced proposal-first. Is "has a placed
   binding" the right trigger, or should it be "the edit changes a field
   the binding depends on"?
6. How far should the first engine-owned `LibraryGraph` diagnostic seam be
   promoted before any more CLI/MCP library authoring commands land? This
   contract recommends continuing the migration: CLI and MCP should call the
   engine graph, and resolver/commit-time validation should share that same
   semantic policy rather than duplicate it.
