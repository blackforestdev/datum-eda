# S5A-C03 remaining authority — owner disposition packet

Packet: `S5A-C03-AUTHORITY-DISPOSITION-01` · prepared 2026-10-06 against `cb8f82c4`
· Frontier step S5A-C03 (`dat-uvt-s5a-build-1wv`), retained C02 execution grant.

Status: **pending owner disposition.** Nothing here is ratified, and no
implementation has started. The three positive cases below remain required. This
packet does not narrow, waive or mark any of them passed. C04 native proof, C05
independent review and C06 acceptance stay separate.

Already landed without a new grant, from existing sources:
- the governed `selection-lock-anchor` cue (`cb8f82c4`);
- the footprint-owned graphic acquisition replay (`62f0e358`).

| # | Decision | Issue | Recommendation | New mechanism? |
|---|---|---|---|---|
| D1 | Standalone board graphics G1–G3 | `dat-s5a-board-graphic-authority-h8xm` | Ratify G1–G3, with four clarifications | Yes: source model and operations |
| D2 | Placed component-Pad identity | `dat-s5a-component-pad-authority-hadv` | Persist a per-placement Pad ObjectId (option B). Reject the bare occurrence pair (option A) | Yes: persisted field and journaled adoption |
| D3 | Production-Proposal action identity | `dat-s5a-artifact-lifecycle-7g4l` | One whole-batch action per Proposal, qualified by existing fields | No new persistence. This is a scope/identity ruling |

---

## D1 — Standalone board graphics (G1–G3)

**Proposal.** `S5A-BOARD-GRAPHICS-PROPOSAL-01`,
`specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md` "Native board graphics". So far the
owner has authorized planning only.
- **G1:** a separate `BoardGraphic` family, UUID-keyed and optional in
  `board/board.json`. Variants: Line, open Polyline, three-point Arc, Rectangle,
  Circle, and Polygon (outer ring plus holes, explicit filled flag).
- **G2:** non-copper, nonconductive documentation layers only.
- **G3:** adoption through `allocate_import_identity` and ImportMap. Ambiguous or
  identifier-less adoption is refused.

**Controlling sources.**
- **Required positive rows.** UVT §2.2 defines filled graphics (authored non-zone
  shapes such as logos) and requires the "filled graphic, line/arc/outline" rows.
  Its row table lists the substrate gap as "interior hit + filled identity
  absent". The same rows are in `preparation-plan.md:215-217`.
- **One mutation path.** PM001 makes every write go through `commit()`.
- **Adoption.** PM011 says imports allocate or reuse stable ObjectIds through
  ImportMap (`import_key`).
- **Place tool deferred.** PCB Layout Tool Contract ~719-724 and Q9 defer a public
  `place_graphic_primitive`. That future model "may fold" text in. G1 keeps
  BoardText separate, which the contract permits but should be stated.
- **Precedent for one family.** Native schematic drawings are already one
  kind-tagged family (`docs/NATIVE_FORMAT.md:90-101`), and so are `SchematicPrimitive`
  and pool `Primitive`. This supports G1's one-family shape over per-primitive
  operation families.
- **Render == CAM.** Law 1 (PM020/PM023, UVT) requires that anything rendered on
  silk, fab or mechanical layers also reaches manufacturing output.

**Code facts.**
- `Board` (`crates/engine/src/board/mod.rs:30-49`) and the `Operation` enum have no
  graphic family. Text, keepout and dimension each have their own Create/Set/Delete.
- Engine KiCad import counts only `gr_line`/`gr_arc`. Edge.Cuts lines/arcs feed the
  outline. No other `gr_*` object is persisted.
- The protocol scene uses ordinal fallback keys (`kicad_scene_import/board_graphics.rs:73-82`).
- `AuthoredSelectionClass::BoardGraphic` exists, but the engine authored-source
  enumeration (`electrical_selection_query/authored_sources.rs:57`) has no graphics
  class.
- Gerber silk is built only from board text plus component silkscreen
  (`gerber/silkscreen.rs:80-202`).

**Recommendation: ratify G1–G3 with these clarifications.**
1. **Byte stability.**
   - The new `graphics` key uses `serde(default)` plus `skip_serializing_if`
     empty. Shard hashing is over raw bytes
     (`source_shard_ref_builders.rs:84`), so this keeps a board with no graphics
     byte- and revision-stable after a writer rewrite.
   - Create creates the map when absent. Its inverse removes an emptied map so
     undo restores exact bytes. Today `board_json_maps.rs:28-31` errors on a
     missing map.
2. **Edge.Cuts.**
   - One KiCad source object yields either an outline contribution or a
     `BoardGraphic`, never both.
   - Adoption discloses which one.
3. **CAM (Law 1).** Graphics on silk, fab or mechanical layers must reach the
   matching Gerber or be explicitly disclosed as not yet exported. The proposal
   currently says only "no CAM approximation".
   - Arcs and circles inherit the open rational-centre CAM/T04 policy.
   - They do not resolve it.
4. **Copper and identifier-less sources.**
   - Copper `gr_*` (logos, antennas) and identifier-less sources are refused at
     adoption with a disclosed loss report. They are not silently dropped.
   - Each would get its own later semantics.
   - This is stricter than BoardText today, which falls back to a content hash
     (`skeleton.rs:529-535`). Say whether BoardText should later converge.

**Effects.**
- **Persistence:** a new optional `board.json` map. Existing projects reopen
  byte-identical.
- **Revision:** Create/Set/DeleteBoardGraphic through `commit()` with
  journal/undo; payload registration follows the keepout pattern.
- **Adoption:** ImportMap keys per source UUID and lifecycle `missing_in_source`,
  like other board objects.
- **Selection:** a graphics source class in `authored_sources`, a filled flag in
  the scene primitive, interior/hole qualification, and Inspector properties.
  `BoardGraphic` properties currently return none.
- **Unblocks:** UVT filled-graphic and line/arc rows (N03/N15). Dimensions stay
  excluded (`dat-dimension-selection-reentry-kxk`). No placement UI is implied.

**Alternative, not recommended.** Fold everything into the deferred
`place_graphic_primitive` model. That is larger and reopens text.

## D2 — Placed component-Pad identity

**Current source.**
- Native placement materialization
  (`crates/cli/src/commands/pool/materialization.rs:297-308`) copies each library
  Footprint pad UUID into every placement's
  `board.json#component_pads[packageUuid]`.
- Positions are footprint-local, and there is no net.
- The frozen S5A recipe shows the same pad UUIDs `80000000…1–8` under two Packages.

**Lifetime of the candidate pair (Package UUID, library pad UUID).**

| Event | Pair preserved? | Evidence |
|---|---|---|
| Move / rotate | yes | `board_journal_ops.rs:93-99`; `board_package_move.rs:5-35` |
| Flip | yes (mirrored in place) | `board_package_move.rs:100-112` |
| Undo / redo; undo of delete | yes (payload snapshot) | `board_journal_ops.rs:180-234` |
| Set Part binding | kept but **stale**: no rematerialization | `board_journal_ops.rs:51-56` |
| Set Package / rematerialize | only if the new footprint reuses the same library pad UUIDs | `board/component_mutations.rs:192-245` |
| Library pad add/remove/renumber | not propagated; frozen snapshot | `library/footprint.rs:262-319` |
| Delete then re-place | no (new Package UUID) | `component_mutations.rs:52` |
| Copy / duplicate | not implemented | — |
| KiCad adoption | not applicable: imported pads get their own per-occurrence UUID in canonical `pads`, reused through ImportMap `kicad:board-pad:…` | `import/kicad/placed_pads.rs:38-56`; `board_objects.rs:133` |

**Why not the bare pair (option A).**
- **PM003 identity.** PM003 (~180-186) requires a stable persisted ObjectId for
  every authored domain object, and pin–pad relationships target pad IDs.
- **Validator refuses it.** The engine occurrence validator accepts only
  `tracks|vias|pads|zones` on the board. It refuses any board occurrence with an
  `instance_path` (`electrical_basis.rs:120-133`), because `instance_path` means
  schematic SheetInstances.
  - Overloading `instance_path` with a Package UUID would amend PM053 occurrence
    semantics.
- **Two Pad shapes.** Option A would leave Datum with two Pad identity shapes
  (imported: plain id; native: pair).
- **Swap aliasing.** The pair silently aliases across a package swap whenever two
  footprints share pad UUIDs.

**Recommendation (option B).**
- **Mint at materialization.** Persist a per-placement Pad ObjectId, minted once at
  materialization. Store it beside a `library_pad` back-reference in each
  `component_pads` entry, created through the journaled create/set-package
  payloads.
- **Rematerialization.** Carry existing IDs over by `library_pad`. New library pads
  get new IDs. Removed pads retire and are never reused.
- **Existing boards.** Boards with no minted IDs need one explicit, journaled,
  provenance-carrying adoption. Minting never happens on read or reopen (PM053).
  - A UUIDv5 of (package, library pad) is acceptable only as that adoption's seed,
    once persisted.
  - The legacy name-keyed `deterministic_component_pad_uuid` in the KiCad
    namespace must not be reused.
- **Selection.** Selection then uses the plain `{class: Pad, id}`, the same shape
  as imported pads. Strict pad-majority uses board-transformed pad anchors.
- **Rulings needed alongside:**
  - Does Set Part rematerialize, or report the stale snapshot?
  - Do Run/Net members include native pads? That needs a net source:
    ComponentInstance → PinPadMap → schematic pin net, per PM053.
- **Larger convergence (option C), not required now.** Materialize native placements
  into the canonical `pads` class with owner, net and transformed geometry. This
  unifies drill/Gerber/zone-fill/routing.

**Effects.**
- **Persistence:** one field per component pad, plus an adoption operation.
- **Revision:** package payload snapshots already journal it.
- **Adoption:** native and imported pads share one shape.
- **Selection:** removes the catalog refusal
  (`selection_geometry_catalog.rs:93-101`) and enables the 4-of-8 / 5-of-8 oracle.

**Adjacent defects filed (not repaired here).**
- `dat-component-pad-validate-duplicate-jpg0`: `validate.rs:494` scopes pad UUID
  uniqueness board-wide.
- `dat-component-pad-cam-transform-ab4v`: copper Gerber context uses footprint-local
  `component_pads` positions with rotation 0, while silk/mechanical transform them.
  Not yet confirmed by an actual Gerber run; interacts with the open CAM/T04
  obligation.

## D3 — Production-Proposal action identity

**Controlling sources.**
- **Subject.** UVT ~1596-1606: "a click owns the whole proposal ACTION", "one
  action = one subject".
- **Lifetime.** UVT §8: the subject dissolves on commit or discard, and its
  lifetime is bound to the producing artifact, not the journal.
- **Explicit-only.** UVT OPEN-12 (~295-305) and conformance row A7.
- **Merely related to geometry.** PM026 ~24-29 and ~40-42: Proposal is a
  lifecycle-bound, explicit-only subject. Proposal-to-produced-geometry is
  "merely related", so identity is the action, never its geometry.
- **Stable ID and stale status.** PM004 ~238-252 requires a stable proposal ID and
  stale status. The tool surface ~321-338 says the proposal owns its typed
  `OperationBatch`.
- **No renderer inference.** The FCCP forbids renderer inference of action IDs
  (~2363, ~2410).

**Present state.**
- `composition.rs:352` emits `proposal_action_id = proposal_id`.
- `selection_artifacts.rs:74` refuses production UUIDs as action authority.
- `ProposalPayload` decoding (`gui-protocol/src/production_proposals.rs:54-68`)
  drops `project_id` and `batch_id`.
- The engine `Proposal` (`substrate/proposal.rs:44-59`) is one atomically applied
  `OperationBatch`. It has no sub-action concept.

**Recommendation.**
- **Identity.** One Proposal is exactly one action: `{project_id, proposal_id,
  batch_id, prepared_against}`.
  - All four fields already exist and persist in `.datum/proposals/{id}.json`,
    which is outside the model-revision hash.
  - No new persisted field.
- **Producer change.**
  - Decode the fields in `ProductionProposalSummary`.
  - Carry the typed tuple from the protocol producer, so scene code only passes
    it through.
  - Add a production branch in the artifact authority that checks project,
    `batch_id`, `prepared_against` and the batch guard against
    `model.proposals[proposal_id]`.
- **Lifetime.**
  - Missing proposal (deleted, or create undone): drop and report, no
    resurrection.
  - Applied or Rejected: dissolve.
  - Model revision moved past `prepared_against`: the subject persists, flagged
    stale-blocked as `validate_proposal_apply` reports. Only commit or discard
    dissolves (UVT §8).
- **Route proposals.** Route action IDs remain finer sub-keys qualified under the
  same producer tuple.
- **Rulings needed.**
  - Does Deferred count as discard? Recommended: no. It keeps the subject; only
    Applied or Rejected dissolves.
  - Is a stale subject kept-and-flagged (recommended) or dissolved?
- **UVT text.** UVT ~1613 / ~1625 ("production proposals have no world-scene
  identity", "ABSENT") describe current state. If this is ratified, the UVT's
  production-proposal row changes from absent to this typed identity.

**Effects.**
- **Persistence:** none.
- **Revision:** identity is stable across unrelated edits; staleness is reported,
  not used as identity churn.
- **Adoption:** not applicable.
- **Selection:** explicit pointing only; PM026's closed mapping is unchanged. The
  generic non-authored-channels positive (`preparation-plan.md:63`) becomes
  implementable.

Option B (chrome-only) is **not** proposed. It would reduce required scope.

## Response format

Reply per decision:
- `D1: ratify G1–G3 + clarifications 1–4 | amend: … | defer`
- `D2: option B (+ Set Part: rematerialize|report; Run/Net: include|exclude) | amend: …`
- `D3: ratify tuple (+ Deferred: keep|dissolve; stale: keep-flagged|dissolve) | amend: …`

The owner's choice will be recorded as numbered decision record(s) or a PM026
amendment, with same-change Frontier/PROGRESS/manifest governance. Execution then
proceeds within the resulting grant. The "dense compound" padlock threshold is
also open: today every compound is treated as dense. That one is a visual
disposition for the HUMAN re-review, not part of this packet.
