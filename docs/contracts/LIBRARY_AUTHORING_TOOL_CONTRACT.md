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
  pool-object gate.

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
2. Operation it emits: typed variants in a NEW
   `crates/engine/src/api/write_ops/library_author.rs` —
   `CreateLibraryObject{kind: Symbol|Footprint|Package|Padstack|Part|
   PinPadMap|ModelAttachment, fields}` and
   `EditLibraryObjectField{object_id, patch}`. Both route through the
   single `commit()` primitive (shared surface; committed at HEAD
   `5fe3016`), mint or bump `ObjectId` + `object_revision`, and append a journal
   `TransactionRecord`. PinPadMap reference resolution, process-aperture
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
   density_level, source_dimensions}`.
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
  Footprint courtyards plus Footprint silkscreen lines/rectangles/circles/polygons directly. Importers and compatibility
  materialization still preserve legacy package land-pattern fields in some
  paths, and footprint fab/silkscreen primitives beyond lines/rectangles/circles/polygons, assembly/mechanical/model/
  process-policy authoring remains pending. The target schema separates
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
- Padstack process-policy consumption. Current padstacks model layer spans,
  aperture, drill, annular, plated/non-plated, thermal/anti-pad, and explicit
  mask/paste policy; the remaining gap is systematic consumption by footprint
  generation/materialization, checks, standards repairs, and fabrication
  outputs, including unknown/import-preserved states.
- `ModelAttachment` as governed library data across targets. Existing package
  `ModelRef`, content-addressed model blobs, and part behavioural-model
  attachments are useful slices, but the target needs one hash/provenance/role/
  review-state contract consistently across Part, Package, and Footprint
  targets.
- `LibraryBinding` / update-binding operations. `ComponentInstance` exists,
  but library bindings are not yet the single first-class join that pins
  revisions for Part, Symbol, Package, Footprint, PinPadMap, and models.
- Pool layering and override policy. Priority ordering exists, but duplicate
  UUID/shadowing, same-UUID override versus fork semantics, writable pool
  targets, conflict diagnostics, and project-local override records need one
  normative rule set.
- Validation tiers. Commit-time rejection, resolver diagnostics, and
  `project validate` findings are not yet a single documented contract.
- IPC footprint generation. Standards basis/deviation recording is specified,
  but full IPC-7351/7352 generation, naming, thermal/anti-pad synthesis,
  density family coverage, and standards-derived pad/mask/paste checks remain
  future work.
- Multi-format symbol style rendering (IEEE 315 / IEC 60617), automatic
  diff-pair/bus pin-group inference, and cross-project ECO fan-out.

## Open Owner Questions

1. Which library pool types ship first: bundled-Datum, user-local,
   organization, project-local, imported, vendor-derived? This sets the
   `pool_ref` enum and `LibraryBinding` layering.
2. Which `approval_state` values gate placement, and is the placement
   gate a hard block or a warning by default?
3. IPC-7351B vs IPC-7352 as the default generated-footprint naming/basis,
   and which first package family the generator supports?
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
