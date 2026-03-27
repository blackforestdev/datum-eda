# Native File Format — Design Rationale

> **Status**: Non-normative design rationale.
> The controlling native format specification is `specs/NATIVE_FORMAT_SPEC.md`.
> The controlling serialization contract is `docs/CANONICAL_IR.md` §5.
> This document provides rationale, tradeoffs, and explanatory examples.
> If any contract here conflicts with `specs/NATIVE_FORMAT_SPEC.md`, the spec
> file wins.

## Purpose
Proposes the on-disk representation for projects created natively by this
tool. The native format is introduced in M4. Before M4, designs exist only
as imported formats (KiCad/Eagle files) with the canonical IR held in memory.

Current live slice:
- `eda project new <dir> [--name <project-name>]` creates the initial native
  scaffold for `project.json`, `schematic/schematic.json`, `board/board.json`,
  and `rules/rules.json`.
- `eda project inspect <dir>` loads that scaffold, validates the resolved file
  layout, and reports current schema/UUID/path/count summary without opening a
  mutable editing session.
- `eda project query <dir> summary` and
  `eda project query <dir> design-rules` provide the first native read surface
  directly from the on-disk scaffold, and `eda project query <dir> nets` plus
  `eda project query <dir> diagnostics` now project authored native sheet files
  through an in-memory `Schematic` bridge to expose the first native
  connectivity-aware read surface; `eda project query <dir> erc` reuses that
  same bridge to expose the first native electrical-check read surface,
  `eda project query <dir> forward-annotation-audit` now compares native
  schematic symbol intent against native board component presence by reference,
  `eda project query <dir> forward-annotation-proposal` now classifies those
  findings into deterministic add/update/remove component ECO actions,
  `eda project query <dir> forward-annotation-review` now reports persisted
  defer/reject decisions keyed by stable proposal `action_id`, and
  `eda project query <dir> check` now returns the combined schematic
  `CheckReport` shape on top of that same bridge; `eda project query <dir>
  board-diagnostics`, `eda project query <dir> board-unrouted`, and
  `eda project query <dir> board-check` now project authored native board
  files through the engine `Board` model to expose the first native board
  connectivity-aware and board-check read surface, and `eda project query
  <dir> board-texts` exposes the first authored native board-object read
  surface.
- `eda project place-symbol <dir> --sheet <uuid> --reference <text> --value <text>
  [--lib-id <text>] --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>] [--mirrored]`,
  `eda project move-symbol <dir> --symbol <uuid> --x-nm <i64> --y-nm <i64>`, and
  `eda project rotate-symbol <dir> --symbol <uuid> --rotation-deg <i32>`,
  `eda project mirror-symbol <dir> --symbol <uuid>`, and
  `eda project delete-symbol <dir> --symbol <uuid>`, `eda project set-symbol-reference <dir> --symbol <uuid> --reference <text>`, and
  `eda project set-symbol-value <dir> --symbol <uuid> --value <text>`,
  `eda project set-symbol-unit <dir> --symbol <uuid> --unit <text>`,
  `eda project clear-symbol-unit <dir> --symbol <uuid>`,
  `eda project set-symbol-gate <dir> --symbol <uuid> --gate <uuid>`, and
  `eda project clear-symbol-gate <dir> --symbol <uuid>`,
  `eda project set-symbol-lib-id <dir> --symbol <uuid> --lib-id <text>`, and
  `eda project clear-symbol-lib-id <dir> --symbol <uuid>`,
  `eda project set-symbol-entity <dir> --symbol <uuid> --entity <uuid>`,
  `eda project clear-symbol-entity <dir> --symbol <uuid>`,
  `eda project set-symbol-part <dir> --symbol <uuid> --part <uuid>`, and
  `eda project clear-symbol-part <dir> --symbol <uuid>`,
  `eda project set-symbol-display-mode <dir> --symbol <uuid> --mode <...>`,
  `eda project set-symbol-hidden-power-behavior <dir> --symbol <uuid> --behavior <...>`,
  `eda project set-pin-override <dir> --symbol <uuid> --pin <uuid> --visible <true|false>
  [--x-nm <i64> --y-nm <i64>]`, and
  `eda project clear-pin-override <dir> --symbol <uuid> --pin <uuid>` add the first native
  authored schematic symbol placement/transform/delete/semantic-selection path, and
  on the current native path `set-symbol-entity` clears any existing `part`, while
  `set-symbol-part` clears any existing `entity`.
  `eda project add-symbol-field <dir> --symbol <uuid> --key <text> --value <text>
  [--hidden] [--x-nm <i64> --y-nm <i64>]`,
  `eda project edit-symbol-field <dir> --field <uuid> ...`, and
  `eda project delete-symbol-field <dir> --field <uuid>` extend that path to
  native symbol field authoring, while `eda project query <dir> symbols`,
  `eda project query <dir> symbol-fields --symbol <uuid>`, and
  `eda project query <dir> symbol-semantics --symbol <uuid>`, and
  `eda project query <dir> symbol-pins --symbol <uuid>` read back the
  persisted symbol slice, including any stored per-pin override state.
- `eda project place-text <dir> --sheet <uuid> --text <text> --x-nm <i64>
  --y-nm <i64> [--rotation-deg <i32>]`, `eda project edit-text <dir> --text <uuid> ...`,
  and `eda project delete-text <dir> --text <uuid>` add the first native
  non-electrical schematic text object family, while
  `eda project query <dir> texts` reads back the persisted text slice.
- `eda project place-drawing-line <dir> --sheet <uuid> --from-x-nm <i64>
  --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>`,
  `eda project place-drawing-rect <dir> --sheet <uuid> ...`,
  `eda project place-drawing-circle <dir> --sheet <uuid> ...`,
  `eda project place-drawing-arc <dir> --sheet <uuid> ...`,
  `eda project edit-drawing-line <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-rect <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-circle <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-arc <dir> --drawing <uuid> ...`, and
  `eda project delete-drawing <dir> --drawing <uuid>` add the first native
  schematic drawing primitive family, while `eda project query <dir> drawings`
  reads back the persisted drawing slice with kind-tagged objects.
- `eda project place-label <dir> --sheet <uuid> --name <text> --x-nm <i64>
  --y-nm <i64>` is the first native authored schematic mutation and writes
  directly to a referenced sheet file.
- `eda project rename-label <dir> --label <uuid> --name <text>` and
  `eda project delete-label <dir> --label <uuid>` complete the first native
  schematic object family on the same deterministic sheet-file mutation path.
- `eda project draw-wire <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64>
  --to-x-nm <i64> --to-y-nm <i64>` and `eda project delete-wire <dir> --wire <uuid>`
  add the first connectivity-bearing native schematic geometry mutation path.
- `eda project place-junction <dir> --sheet <uuid> --x-nm <i64> --y-nm <i64>`
  and `eda project delete-junction <dir> --junction <uuid>` extend that path
  to authored net-topology join points.
- `eda project place-port <dir> --sheet <uuid> --name <text> --direction <...>
  --x-nm <i64> --y-nm <i64>`, `eda project edit-port <dir> --port <uuid> ...`,
  and `eda project delete-port <dir> --port <uuid>` add the first cross-sheet
  connectivity-interface object family on the same deterministic mutation path.
- `eda project create-bus <dir> --sheet <uuid> --name <text> --member <text>...`,
  `eda project edit-bus-members <dir> --bus <uuid> --member <text>...`,
  `eda project place-bus-entry <dir> --sheet <uuid> --bus <uuid> [--wire <uuid>]`,
  and `eda project delete-bus-entry <dir> --bus-entry <uuid>` extend that path
  to structured multi-net connectivity objects.
- `eda project place-noconnect <dir> --sheet <uuid> --symbol <uuid> --pin <uuid>
  --x-nm <i64> --y-nm <i64>` and `eda project delete-noconnect <dir> --noconnect <uuid>`
  add the no-connect marker object family on the same deterministic sheet-file path.
- `eda project query <dir> nets` and `eda project query <dir> diagnostics`
  derive native connectivity output from the persisted sheet objects without
  requiring an imported-design session.
- `eda project query <dir> erc` derives native ERC precheck findings from the
  persisted sheet objects without requiring an imported-design session.
- `eda project query <dir> check` derives the combined native schematic check
  report from the persisted sheet objects without requiring an imported-design
  session.
- `eda project place-board-text <dir> --text <text> --x-nm <i64> --y-nm <i64>
  [--rotation-deg <i32>] --layer <i32>`, `eda project edit-board-text <dir>
  --text <uuid> ...`, and `eda project delete-board-text <dir> --text <uuid>`
  add the first native board-authored object family directly on
  `board/board.json`, while `eda project query <dir> board-texts` reads back
  that persisted board slice.
- `eda project place-board-keepout <dir> --kind <text> --layer <i32>...
  --vertex <x:y>...`, `eda project edit-board-keepout <dir> --keepout <uuid> ...`,
  and `eda project delete-board-keepout <dir> --keepout <uuid>` extend the
  same native board path to keepout polygons, while
  `eda project query <dir> board-keepouts` reads back that persisted keepout
  slice.
- `eda project place-board-dimension <dir> --from-x-nm <i64> --from-y-nm <i64>
  --to-x-nm <i64> --to-y-nm <i64> [--text <text>]`,
  `eda project edit-board-dimension <dir> --dimension <uuid> ...`, and
  `eda project delete-board-dimension <dir> --dimension <uuid>` extend the
  same native board path to persisted dimension annotations, while
  `eda project query <dir> board-dimensions` reads back that dimension slice.
- `eda project set-board-outline <dir> --vertex <x:y>...` replaces the
  persisted native board outline polygon directly on `board/board.json`, while
  `eda project query <dir> board-outline` reads back that canonical outline
  slice.
- `eda project set-board-stackup <dir> --layer <id:name:type:thickness_nm>...`
  replaces the ordered native board stackup layer list directly on
  `board/board.json`, while `eda project query <dir> board-stackup` reads back
  that persisted stackup slice.
- `eda project place-board-net <dir> --name <text> --class <uuid>`,
  `eda project edit-board-net <dir> --net <uuid> ...`, and
  `eda project delete-board-net <dir> --net <uuid>` add the first native board
  net lifecycle on `board/board.json`, while `eda project query <dir>
  board-nets` reads back that persisted net slice.
- `eda project query <dir> board-components` reads back the persisted native
  placed-package inventory from `board/board.json`, establishing the read-side
  contract for future native package placement work.
- `eda project export-bom <dir> --out <path>` writes a deterministic CSV BOM
  directly from that persisted native board-component inventory, establishing
  the first native manufacturing-export slice without depending on pool lookup,
  manufacturer metadata, or variant expansion.
- `eda project compare-bom <dir> --bom <path>` now compares that BOM CSV
  against the current native board-component inventory by reference, value,
  part/package identity, layer, position, rotation, and locked state, so BOM
  drift is reviewable semantically instead of only by file generation.
- `eda project export-pnp <dir> --out <path>` writes a deterministic CSV
  pick-and-place export directly from that same persisted native
  board-component inventory, establishing the second native manufacturing-
  export slice without depending on feeder setup, origin calibration, or
  variant expansion.
- `eda project compare-pnp <dir> --pnp <path>` now compares that PnP CSV
  against the current native board-component inventory by reference, placement,
  rotation, side/layer, part/package identity, value, and locked state, so PnP
  drift is reviewable semantically instead of only by file generation.
- `eda project export-drill <dir> --out <path>` writes a deterministic CSV
  drill export directly from the persisted native via inventory, establishing
  the first native drill-export slice without claiming full Excellon tooling,
  slot support, or fabrication-ready hole tables yet.
- `eda project export-excellon-drill <dir> --out <path>` now writes a narrow
  Excellon drill file directly from the persisted native via inventory through
  `eda_engine::export`, establishing the first real drill writer on the native
  board path.
- `eda project validate-excellon-drill <dir> --drill <path>` now re-renders
  that expected native Excellon drill file and reports byte-for-byte
  match/mismatch with CI-usable exit status, establishing the first validation
  surface for a real drill writer.
- `eda project inspect-excellon-drill <path>` now reads that narrow Excellon
  drill file and reports the parsed tool table plus hit counts, establishing
  the first Excellon read/inspection surface without claiming broader drill
  interchange support.
- `eda project compare-excellon-drill <dir> --drill <path>` now compares that
  narrow Excellon file against the current native via inventory by normalized
  drill diameter and hit count, reporting matched, missing, extra, and hit-
  drift drill buckets without requiring byte-for-byte identity.
- `eda project report-drill-hole-classes <dir>` now groups the current native
  via inventory into explicit through/blind/buried hole classes using copper
  layer span from the persisted native stackup, reporting per-class tool tables
  and hit counts.
- `eda project export-gerber-outline <dir> --out <path>` now writes a narrow
  RS-274X Gerber file for the persisted native board outline through
  `eda_engine::export`, establishing the first real native Gerber writer on
  the board-outline slice.
- `eda project validate-gerber-outline <dir> --gerber <path>` now re-renders
  that expected native board-outline Gerber and reports byte-for-byte
  match/mismatch with CI-usable exit status, establishing the first native
  validation surface for a real Gerber writer.
- `eda project export-gerber-copper-layer <dir> --layer <id> --out <path>`
  now writes a narrow RS-274X Gerber file for persisted native board tracks,
  vias, and zones on one selected copper layer through `eda_engine::export`,
  establishing the first real copper-layer writer without claiming pads or full
  layer-set emission yet.
- `eda project validate-gerber-copper-layer <dir> --layer <id> --gerber
  <path>` now re-renders that expected native copper-layer Gerber and reports
  byte-for-byte match/mismatch with CI-usable exit status, establishing the
  first native validation surface for the copper-layer writer.
- `eda project plan-gerber-export <dir> [--prefix <text>]` reports the
  deterministic broader Gerber artifact set implied by the current native
  board outline and stackup, without claiming that the full planned layer set
  is writable yet.
- `eda project compare-gerber-export-plan <dir> --output-dir <path>
  [--prefix <text>]` compares that deterministic planned artifact set against a
  directory and reports matched, missing, and extra files, establishing the
  first native Gerber drift/comparison slice without claiming aperture or
  geometry-level validation yet.
- `eda project query <dir> board-pads` reads back the persisted native
  placed-pad inventory from `board/board.json`, establishing the first native
  package-linked pad read surface.
- `eda project query <dir> board-tracks` reads back the persisted native routed
  track inventory from `board/board.json`, establishing the first native copper
  read surface.
- `eda project query <dir> board-vias` reads back the persisted native via
  inventory from `board/board.json`, extending the first native copper read
  surface beyond tracks.
- `eda project query <dir> board-zones` reads back the persisted native copper
  zone inventory from `board/board.json`, completing the first native copper
  read surface across tracks, vias, and zones.
- `eda project query <dir> board-diagnostics`, `eda project query <dir>
  board-unrouted`, and `eda project query <dir> board-check` derive native
  board connectivity findings, airwires, and the board-side `CheckReport`
  shape from the persisted package/pad/track/via/zone/net state without
  requiring an imported-board session.
- `eda project query <dir> forward-annotation-audit` derives a read-only
  schematic-vs-board comparison report from the persisted native symbol and
  board component inventories without attempting ECO mutation yet.
- `eda project query <dir> forward-annotation-proposal` derives a read-only
  ECO proposal from that same persisted native symbol and board component
  inventory without attempting apply/reject mutation yet; each proposed action
  now carries a stable `action_id` plus grouped add/remove/update views so the
  later review/apply layer can target actions deterministically.
- `eda project defer-forward-annotation-action <dir> --action-id <sha256:...>`,
  `eda project reject-forward-annotation-action <dir> --action-id <sha256:...>`,
  `eda project clear-forward-annotation-action-review <dir> --action-id <sha256:...>`,
  and `eda project query <dir> forward-annotation-review` now persist, clear,
  and expose deterministic review decisions by stable proposal key without
  applying the ECO action itself.
- `eda project apply-forward-annotation-action <dir> --action-id <sha256:...>`
  now applies the currently supported proposal kinds by stable key:
  `remove_component`, `update_component` when the reason is `value_mismatch`,
  and `add_component` when the caller supplies explicit
  `--package <uuid> --x-nm <i64> --y-nm <i64> --layer <i32>` input; unresolved
  add proposals also require `--part <uuid>` when the proposal does not carry a
  resolved schematic part. `part_mismatch` now applies through the same stable
  action key when the caller supplies an explicit `--package <uuid>` target;
  unresolved part-mismatch proposals also require `--part <uuid>` when the
  proposal does not carry a resolved schematic part.
- `eda project set-board-component-part <dir> --component <uuid> --part <uuid>`
  and `eda project set-board-component-package <dir> --component <uuid>
  --package <uuid>` now expose the board-side reassignment semantics directly,
  so forward-annotation `part_mismatch` apply reuses the same native component
  mutation surface instead of writing bespoke ECO-only state.
- `eda project apply-forward-annotation-reviewed <dir>` now batch-applies the
  current self-sufficient proposal actions while honoring persisted
  `deferred`/`rejected` review decisions. In the current slice that means
  `remove_component` and `value_mismatch` updates apply automatically, while
  `add_component` and `part_mismatch` remain visible in the batch report as
  `requires_explicit_input` until the caller supplies per-action package or
  placement input through the keyed single-action command.
- `eda project export-forward-annotation-proposal <dir> --out <path>` now
  writes a versioned `native_forward_annotation_proposal_artifact` file
  containing the current proposal actions plus persisted review decisions, so
  the forward-annotation review package can be preserved and moved independently
  of the live native project state.
- `eda project export-forward-annotation-proposal-selection <dir> --action-id
  <sha256:...>... --out <path>` now writes that same artifact kind with only
  the selected proposal actions and matching review records, so scoped review
  exchange does not require exporting the entire live proposal package.
- `eda project select-forward-annotation-proposal-artifact --artifact <path>
  --action-id <sha256:...>... --out <path>` now narrows an existing artifact
  down to the selected proposal actions and matching review records without
  consulting live project state, so scoped artifact exchange can stay entirely
  artifact-local when needed.
- `eda project inspect-forward-annotation-proposal-artifact <path>` now loads
  that artifact and reports version, project identity, action counts, and
  review counts through a deterministic read-only inspection surface.
- `eda project compare-forward-annotation-proposal-artifact <dir> --artifact <path>`
  now compares the exported artifact against the current live project proposal
  and classifies each artifact action as `applicable`, `drifted`, or `stale`
  before any artifact-driven apply path exists.
- `eda project filter-forward-annotation-proposal-artifact <dir> --artifact <path>
  --out <path>` now writes a new versioned artifact containing only the exported
  actions that are still `applicable` against the current live project plus the
  retained review records for those actions, so later artifact-driven apply can
  start from a deterministic prefiltered package instead of the original stale
  export.
- `eda project plan-forward-annotation-proposal-artifact-apply <dir> --artifact
  <path>` now reports which artifact actions are currently `self_sufficient`
  versus `requires_explicit_input`, including the exact caller inputs still
  needed for `add_component` and `part_mismatch`, so artifact-driven apply can
  preflight mutation readiness without touching live project state.
- `eda project apply-forward-annotation-proposal-artifact <dir> --artifact
  <path>` now applies a filtered artifact only when it still targets the same
  native project UUID and every retained action remains both `applicable` and
  `self_sufficient`; retained `deferred` and `rejected` review decisions still
  suppress execution inside that artifact-driven apply path.
- `eda project import-forward-annotation-artifact-review <dir> --artifact
  <path>` now merges artifact review decisions back into live project state
  only for `action_id`s that still exist in the current live proposal, so the
  artifact can restore active review intent without reviving stale decisions.
- `eda project replace-forward-annotation-artifact-review <dir> --artifact
  <path>` now replaces the live review map with that same validated artifact
  subset, so review import can be either additive or authoritative depending on
  whether the caller wants to preserve or discard unrelated existing live
  review state.
- `eda project place-board-component <dir> --part <uuid> --package <uuid>
  --reference <text> --value <text> --x-nm <i64> --y-nm <i64> --layer <i32>`
  plus `eda project move-board-component <dir> --component <uuid> --x-nm <i64>
  --y-nm <i64>`, `eda project rotate-board-component <dir> --component <uuid>
  --rotation-deg <i32>`, `eda project set-board-component-locked <dir>
  --component <uuid>`, and `eda project clear-board-component-locked <dir>
  --component <uuid>`, and `eda project delete-board-component <dir>
  --component <uuid>` add the first native placed-package full lifecycle on
  `board/board.json`, while `eda project query <dir> board-components` verifies
  the persisted component state.
- `eda project set-board-pad-net <dir> --pad <uuid> --net <uuid>` and
  `eda project clear-board-pad-net <dir> --pad <uuid>` plus
  `eda project edit-board-pad <dir> --pad <uuid> [--x-nm <i64> --y-nm <i64>]
  [--layer <i32>]` plus `eda project place-board-pad <dir> --package <uuid>
  --name <text> --x-nm <i64> --y-nm <i64> --layer <i32> [--net <uuid>]` and
  `eda project delete-board-pad <dir> --pad <uuid>` add the first
  package-linked pad lifecycle/edit path on `board/board.json`, while `eda
  project query <dir> board-pads` verifies the persisted pad inventory,
  placement, and net assignment state.
- `eda project draw-board-track <dir> --net <uuid> --from-x-nm <i64>
  --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64> --width-nm <i64> --layer
  <i32>` and `eda project delete-board-track <dir> --track <uuid>` add the
  first native copper authoring path on `board/board.json`, while `eda project
  query <dir> board-tracks` verifies the persisted routed geometry.
- `eda project place-board-via <dir> --net <uuid> --x-nm <i64> --y-nm <i64>
  --drill-nm <i64> --diameter-nm <i64> --from-layer <i32> --to-layer <i32>`
  and `eda project delete-board-via <dir> --via <uuid>` extend that native
  copper authoring path to vias, while `eda project query <dir> board-vias`
  verifies the persisted via geometry.
- `eda project place-board-zone <dir> --net <uuid> --vertex <x:y>... --layer
  <i32> --priority <u32> --thermal-relief <bool> --thermal-gap-nm <i64>
  --thermal-spoke-width-nm <i64>` and `eda project delete-board-zone <dir>
  --zone <uuid>` extend that native copper authoring path to zones, while
  `eda project query <dir> board-zones` verifies the persisted copper fill
  geometry and thermal settings.
- `eda project place-board-net-class <dir> ...`,
  `eda project edit-board-net-class <dir> --net-class <uuid> ...`, and
  `eda project delete-board-net-class <dir> --net-class <uuid>` add the first
  native board net-class lifecycle on `board/board.json`, while
  `eda project query <dir> board-net-classes` reads back that persisted rule
  slice.

## Design Principle
The native format is a direct serialization of the canonical IR. No
translation layer, no lossy conversion. What is in memory is what is on
disk, serialized as deterministic JSON.

---

## 1. Project Structure

A native project is a directory containing:

```
myproject/
├── project.json              # Project manifest
├── schematic/
│   ├── schematic.json        # Schematic metadata + variants + waivers
│   ├── sheets/
│   │   ├── <uuid>.json       # One file per sheet
│   │   └── ...
│   └── definitions/
│       ├── <uuid>.json       # One file per SheetDefinition
│       └── ...
├── board/
│   └── board.json            # Complete board state
├── pool/                     # Project-local pool (optional)
│   ├── units/
│   ├── entities/
│   ├── symbols/
│   ├── packages/
│   ├── padstacks/
│   ├── parts/
│   └── pool.sqlite           # Pool index (rebuildable)
├── rules/
│   └── rules.json            # Design rules (shared by ERC + DRC)
├── settings/
│   ├── checking.json         # ERC/DRC severity overrides, waiver config
│   └── output.json           # Export presets (M4+: Gerber, BOM, etc.)
└── .ids/                     # Import sidecar data (if project was imported)
    └── <original_filename>.ids.json
```

### Why multiple files instead of one

- **Diffability**: `git diff` shows which sheets changed, not a monolithic blob
- **Merge-friendliness**: two people editing different sheets don't conflict
- **Incremental save**: only write files for modified objects
- **AI-friendliness**: read one sheet without loading the entire project
- **Performance**: large boards serialize/deserialize faster in isolation

---

## 2. File Schemas

### 2.1 project.json

```json
{
  "schema_version": 1,
  "uuid": "project-uuid",
  "name": "My Project",
  "created": "2026-03-24T12:00:00Z",
  "modified": "2026-03-24T14:30:00Z",
  "pools": [
    { "path": "pool", "priority": 1 },
    { "path": "/shared/team-pool", "priority": 2 }
  ],
  "schematic": "schematic/schematic.json",
  "board": "board/board.json",
  "rules": "rules/rules.json",
  "settings": {
    "checking": "settings/checking.json",
    "output": "settings/output.json"
  }
}
```

### 2.2 schematic.json

```json
{
  "schema_version": 1,
  "uuid": "schematic-uuid",
  "sheets": {
    "sheet-uuid-1": "sheets/sheet-uuid-1.json",
    "sheet-uuid-2": "sheets/sheet-uuid-2.json"
  },
  "definitions": {
    "def-uuid-1": "definitions/def-uuid-1.json"
  },
  "instances": [
    {
      "uuid": "instance-uuid",
      "definition": "def-uuid-1",
      "parent_sheet": null,
      "position": { "x": 0, "y": 0 },
      "name": "Main Sheet"
    }
  ],
  "variants": {
    "variant-uuid": {
      "name": "Standard",
      "fitted_components": {}
    }
  },
  "waivers": []
}
```

### 2.3 Sheet file (sheets/<uuid>.json)

```json
{
  "schema_version": 1,
  "uuid": "sheet-uuid",
  "name": "Power Supply",
  "frame": null,
  "symbols": {
    "sym-uuid": {
      "uuid": "sym-uuid",
      "part": "part-uuid",
      "entity": "entity-uuid",
      "gate": "gate-uuid",
      "reference": "U1",
      "value": "LM3671",
      "fields": [],
      "position": { "x": 25400000, "y": 19050000 },
      "rotation": 0,
      "mirrored": false,
      "unit_selection": null,
      "display_mode": "LibraryDefault",
      "pin_overrides": [],
      "hidden_power_behavior": "SourceDefinedImplicit"
    }
  },
  "wires": {},
  "junctions": {},
  "labels": {},
  "buses": {},
  "bus_entries": {},
  "ports": {},
  "noconnects": {},
  "texts": {},
  "drawings": {}
}
```

### 2.4 board.json

The board is a single file because board objects are heavily
cross-referenced (tracks reference nets, nets reference components,
zones reference nets and interact geometrically). Splitting the board
across files would create complex cross-file references.

```json
{
  "schema_version": 1,
  "uuid": "board-uuid",
  "name": "My Board",
  "stackup": {
    "layers": [
      { "id": 1, "name": "Top", "type": "Copper", "thickness_nm": 35000 },
      { "id": 2, "name": "Dielectric", "type": "Dielectric", "thickness_nm": 1600000 },
      { "id": 3, "name": "Bottom", "type": "Copper", "thickness_nm": 35000 }
    ]
  },
  "outline": {
    "vertices": [
      { "x": 0, "y": 0 },
      { "x": 53800000, "y": 0 },
      { "x": 53800000, "y": 37500000 },
      { "x": 0, "y": 37500000 }
    ],
    "closed": true
  },
  "packages": {},
  "tracks": {},
  "vias": {},
  "zones": {},
  "nets": {},
  "net_classes": {},
  "keepouts": [],
  "dimensions": [],
  "texts": []
}
```

All coordinates are `i64` nanometers. All angles are `i32` tenths of degree.
All UUIDs are lowercase hyphenated strings.

### 2.5 rules.json

```json
{
  "schema_version": 1,
  "rules": [
    {
      "uuid": "rule-uuid",
      "name": "Default Clearance",
      "scope": "All",
      "priority": 1,
      "enabled": true,
      "rule_type": "ClearanceCopper",
      "parameters": { "Clearance": { "min": 100000 } }
    }
  ]
}
```

---

## 3. Serialization Contract

All native format files follow the deterministic serialization contract
from CANONICAL_IR.md §5:

- Map keys sorted alphabetically
- UUIDs lowercase hyphenated
- Coordinates as integer nanometers (never floating point)
- Arrays ordered semantically (vertices in polygon order, layers in
  stackup order) or by UUID (for unordered collections)
- Schema version at document root
- UTF-8 encoding, no BOM
- No timestamp or random data in serialized design objects

**Guarantee**: Same authored data → byte-identical file content on every
save, every platform, every run. Verified by golden tests.

---

## 4. Schema Versioning

Every file includes `"schema_version": N`. Version rules:

- The engine can **read** any version ≥ 1
- The engine always **writes** the current version
- Migration logic is explicit per-version (not implicit)
- Migration is forward-only: version 1 → 2, never 2 → 1
- If a file has an unknown version (higher than the engine knows),
  the engine returns an error with a clear message

### Migration example
```
Version 1: { "stackup": { "layers": [...] } }
Version 2: { "stackup": { "layers": [...], "material_db": "iec-61249" } }

Migration 1→2: add "material_db": null to stackup
```

Migrations are collected in a module, one function per version transition.
They are tested against saved golden files of each version.

---

## 5. Imported Project Persistence

Before M4 (native format), imported projects persist through:

1. **Source files**: original .kicad_pcb / .kicad_sch / .brd / .sch
2. **Sidecar .ids.json**: UUID mapping for stable identity across sessions
3. **In-memory canonical IR**: loaded from source files on each open

After M4, imported projects can optionally be converted to native format:
- `tool convert <kicad_project> --to native`
- This creates a native project directory from the imported data
- The original source files are NOT modified or deleted
- Conversion is one-way (native → KiCad export is a separate operation)

---

## 6. File Locking and Concurrency

The engine assumes single-writer access to a project directory. It does
not implement file locking or concurrent-write detection in v1.

For multi-user scenarios:
- Use git branching (each user works on a branch)
- Merge at the git level (per-file structure makes this practical)
- Conflict resolution: git merge conflicts on JSON files are
  human-resolvable (one object per file, sorted keys)

Future: file-level locking or operational transform for real-time
collaboration (M8+ if ever).

---

## Milestone Position
- M0-M3: No native format. Designs exist as imported source files +
  sidecar .ids.json + in-memory canonical IR
- M4: Native format introduced. Schema version 1.
- M4: Convert-from-import tool
- M4+: Schema migrations as format evolves
