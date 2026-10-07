# S5A-C03 remaining authority — owner disposition packet

Packet: `S5A-C03-AUTHORITY-DISPOSITION-01` · corrected 2026-10-06; source audit against `6321272d`
· Frontier step S5A-C03 (`dat-uvt-s5a-build-1wv`), retained C02 execution grant.

Status: **corrected packet pending independent re-review and owner disposition.**
The owner authorized this bounded correction only. D1–D3 are not ratified; their
dependent implementation has not started. Unrelated C03 execution continues. The three positive cases below remain required. This
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
1. **Byte preservation — mechanism and limits (proposed, not implemented).**
   - Read-only open, query and reopen must never write an absent `graphics` map.
     An absent key and a present empty `{}` are distinct source states. Typed
     `serde(default)`/`skip_serializing_if` cannot retain that distinction and do
     not prove raw-byte preservation.
   - The actual canonical writer stages a `serde_json::Value` through
     `to_json_deterministic` plus a newline (`substrate/journal.rs`,
     `stage_shard_write`). Map helpers require an existing map; removal retains
     it (`substrate/board_json_maps.rs`). Undo commits inverse operations through
     that same writer (`substrate/undo_redo.rs`); it is not a raw preimage restore.
     Revision integrity stores staged postimage bytes (`revision/store.rs`,
     `stage_transaction`), not an automatic inverse byte-restoration mechanism.
   - Recommended bounded mechanism: operate on the existing source JSON value;
     record the pre-operation map-presence state in the guarded operation and
     inverse. Create may insert an absent map; its inverse removes the key only
     if that operation introduced it. Undo into a previously empty map retains
     `{}`. Set/Delete retain map presence. Replay uses recorded presence and IDs,
     never a defaulted typed serialization or an emptiness heuristic.
   - For canonical preimages, the existing deterministic writer can reproduce
     the original bytes after an exact logical inverse, provided every other
     source field and the map-presence state are retained. This is an acceptance
     hypothesis requiring proof, not a guarantee already supplied by the code.
     Noncanonical whitespace/order cannot be reproduced by that mechanism.
     Recommend refusing a graphic write on such a shard before staging, with an
     explicit canonicalization/adoption obligation pending. Do not silently
     normalize it or claim arbitrary legacy raw-byte undo. Supporting those
     writes with exact original-byte restoration would require a separately
     ratified, hash-verified preimage mechanism inside `commit()`.
   - Required future proof uses existing journal/reopen fixtures where adequate:
     absent map, present empty map, populated map and noncanonical input. Hash
     raw board bytes and check unaffected shards, IDs, geometry and ImportMap
     through Create/Set/Delete, undo/redo, recorded replay and reopened undo
     state. Canonical absent/empty preimages must be restored exactly; forward
     replay/redo must reproduce committed postimages. Noncanonical refusal
     changes no source, journal or artifacts. Hash the journal separately:
     compensating undo appends history and does not restore old journal bytes or
     imply that the model revision itself returns to its former value.
     Include any schema/profile change in the raw-byte oracle. Existing Track
     feature adoption is forward-only (PM054); retaining an adopted version on
     undo cannot satisfy an exact pre-adoption board-byte claim. This proposal
     does not assume graphics need no profile change. If one is required, its
     inverse/adoption boundary must be explicitly dispositioned before execution,
     rather than silently weakening the preservation obligation.

2. **Edge.Cuts.**
   - One KiCad source object yields either an outline contribution or a
     `BoardGraphic`, never both.
   - Adoption discloses which one.
3. **CAM (Law 1): supported output, refusal and pending obligation.**
   - A rendered graphic on silk, fab or mechanical layers requires matching
     manufacturing output from the same exact source geometry. Disclosure of
     missing export does **not** satisfy Render == CAM or a required positive.
   - Supported means the existing manufacturing owner emits certified exact
     geometry on the corresponding output layer and proves render/output
     correspondence. A capability table must name supported shape/layer pairs;
     merely storing or drawing a shape cannot mark that pair supported.
   - Unsupported or uncertified export refuses atomically before replacing
     output artifacts, identifies the source and missing capability, and remains
     a pending mandatory positive obligation. A proposed narrower initial
     capability set is not permission to waive G1 variants or required layers.
   - Three-point arcs and circles do not inherit a blanket output guarantee from
     Track support. Reuse certified machinery only after equivalence is proved.
     Preserve zero intentional deviation and the open rational-centre CAM/T04
     policy; this packet neither resolves it nor extends its explicit read-only
     S5A prerequisite exception into a graphics-output acceptance waiver.
4. **Copper and identifier-less sources.**
   - Copper `gr_*` (logos, antennas) and identifier-less sources are refused at
     adoption with a disclosed loss report. They are not silently dropped.
   - Each would get its own later semantics.
   - This is stricter than BoardText today, which falls back to a content hash
     (`skeleton.rs:529-535`). Say whether BoardText should later converge.

**Effects.**
- **Persistence:** a new optional `board.json` map. Read-only reopening leaves
  bytes unchanged; writes and inverse bytes require the mechanism and proof above.
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
  land-pattern pad UUID (Footprint, or the legacy Package fallback) into every placement's
  `board.json#component_pads[packageUuid]`.
- Positions are land-pattern-local, and there is no net. Set Part currently
  changes only the Part reference; it does not record an explicit stale
  materialization status. The stale disclosure below is proposed work.
- The frozen S5A recipe shows the same pad UUIDs `80000000…1–8` under two Packages.

**Lifetime of the candidate pair (Package UUID, library pad UUID).**

| Event | Pair preserved? | Evidence |
|---|---|---|
| Move / rotate | yes | `board_journal_ops.rs:93-99`; `board_package_move.rs:5-35` |
| Flip | yes (mirrored in place) | `board_package_move.rs:100-112` |
| Undo / redo; undo of delete | yes (payload snapshot) | `board_journal_ops.rs:180-234` |
| Set Part binding | kept but **stale**: no rematerialization | `board_journal_ops.rs:51-56` |
| Set Package / rematerialize | only if the new footprint reuses the same library pad UUIDs | `crates/cli/src/commands/board/component_mutations.rs:192-245` |
| Library pad add/remove/renumber | not propagated; frozen snapshot | `library/footprint.rs:262-319` |
| Delete then re-place | no (new Package UUID) | `crates/cli/src/commands/board/component_mutations.rs:52` |
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
- **Authoritative correspondence.** Record project/Board, placed Package and
  its explicit ComponentInstance relationship, pinned Part/Package and the selected land-pattern owner
  (Footprint, or explicitly recorded legacy Package fallback), the library Pad and pinned Padstack, and the applicable pinned PinPadMap with
  gate/pin occurrence. A library-pad UUID is a reference, not sufficient proof
  that two materializations describe the same placed Pad. Reference strings,
  pad numbers, coordinates and equal Net names never establish correspondence.
  The proposed journal payload records each old placed ID, old pinned source
  binding and its explicit new pinned library-Pad target, plus additions and
  retirements. Validation accounts for every old/new pad exactly once, rejects
  duplicate targets or foreign placements, and verifies the referenced source
  revisions and library-object history. Same library UUID after deletion/rebirth
  is not continuity. When history cannot prove survival, require explicit
  replacement rather than synthesizing lineage. PinPadMap changes are validated
  as electrical rebinding separately; equal pin numbers do not preserve Pad ID.

- **Rematerialization within the same binding.** Preserve a placed Pad ID only
  when an engine-validated, one-to-one correspondence proves continuity of that
  Pad under the same placed Package and pinned land-pattern lineage. A surviving
  library reference may support the proof but cannot replace it. Changed source
  revisions require explicit validated lineage or the replacement rule below.
  Geometry/pad-number changes do not themselves mint or identify a Pad. The
  source owner validates continuity; scene keys or renderer observations never
  supply it.
- **Replacement and retirement.** A package/footprint replacement retires old
  Pad IDs and allocates fresh IDs unless explicit authoritative one-to-one
  correspondence is supplied and validated in the canonical batch. Added pads
  allocate recorded IDs; removed pads retire with provenance. Unrelated reuse
  of a library UUID, deletion/re-placement and future copying cannot recycle
  retired IDs. Undo restores the recorded prior IDs and binding snapshot; redo
  and replay reuse recorded allocations, never mint on read. This does not
  implicitly resurrect a dissolved selection.
- **Existing boards.** Boards with no minted IDs need one explicit, journaled,
  provenance-carrying adoption. Minting never happens on read or reopen (PM053).
  Refuse ambiguous source correspondence atomically; a deterministic seed does
  not prove it.
  - A UUIDv5 of (package, library pad) is acceptable only as that adoption's seed,
    once persisted.
  - The legacy name-keyed `deterministic_component_pad_uuid` in the KiCad
    namespace must not be reused.
- **Selection.** Selection then uses the plain `{class: Pad, id}`, the same shape
  as imported pads. Strict pad-majority uses board-transformed pad anchors.
- **Stale Part binding recommendation.** Set Part keeps the frozen geometry and
  explicitly reports its materialization/correspondence as Stale until an
  explicit journaled rematerialization or replacement validates the new binding.
  Do not guess a new Pad mapping or rematerialize merely on read. Valid local
  Object inspection may continue with that stale status; it cannot certify a
  complete cross-domain Net.
- **Electrical registration recommendation.** Register each adopted placed ID
  as a Board Pad source in the existing source-owned object/occurrence index,
  with placement transform and layer/span from its pinned materialization.
  Board occurrences have no schematic `instance_path`. The ComponentInstance,
  pinned PinPadMap and schematic pin occurrence establish an explicit logical
  Net ↔ board-local Net relationship per PM053, separately from geometric Run
  contact. Complete Run/Net membership includes native pads only when geometry,
  source registration and the required electrical binding are authoritative.
  Missing/ambiguous/stale mappings yield the existing source-bound unavailable
  or inconsistent result for the dependent aggregate, never successful empty
  membership or a partial Global Net advertised as complete. Unrelated complete
  subjects remain available. Allocation, registration, relationship changes and
  retirement must be journaled atomically with the placement/adoption batch.
- **Required future proof.** Reuse the two-placement recipe to distinguish
  shared library pads; verify same-binding materialization, changed revision
  with/without proved lineage, footprint replacement with reused library UUIDs,
  added/removed pads, move/rotate/flip, stale Set Part, delete/re-place, adoption,
  inverse/replay/reopen and complete versus refused electrical membership.
  No proposed continuity or registration rule is implementation authority yet.
- **Larger convergence (option C), not required now.** Materialize native placements
  into the canonical `pads` class with owner, net and transformed geometry. This
  unifies drill/Gerber/zone-fill/routing.

**Effects.**
- **Persistence:** placed ID plus library/provenance binding and explicit adoption;
  the exact schema and retirement record remain proposed.
- **Revision:** existing payload snapshots provide a reuse point; allocation,
  correspondence, registration and inverse coverage still require implementation.
- **Adoption:** selection references share the plain Pad-ID shape; native and
  imported storage are not claimed to have converged.
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

## Independent-review finding dispositions

These are the three findings forwarded by the owner on 2026-10-06. No reviewer
identity, re-review outcome or ratification is inferred.

| Finding | Corrected disposition | Acceptance still pending |
|---|---|---|
| 1 — Render == CAM weakened to disclosure | D1 clarification 3 now distinguishes exact supported output, atomic refusal and mandatory pending positive output. Missing export cannot pass. | Shape/layer capability and matching output proof; CAM/T04 unchanged. |
| 2 — Placed-Pad continuity underspecified | D2 now proposes explicit pinned correspondence, validated rematerialization, replacement/retirement, stale binding and source/electrical registration. | Independent re-review, owner disposition and canonical implementation/proof. |
| 3 — Unsupported byte-stability guarantee | D1 clarification 1 names the actual Value writer/inverse path, records absent/empty distinction, bounds canonical-byte claims and refuses unsupported noncanonical writes. | Presence-aware operations and actual undo/replay/reopen raw-byte proof; no arbitrary-byte guarantee. |

## Owning-contract reconciliation and review pin

The foundation completion plan links this corrected packet as its pending
G1–G3/Pad/Proposal decision packet. Native-format determinism concerns canonical
serialization, not preservation of arbitrary original formatting. ENGINE_SPEC
continues to own `commit()` and compensating inverse batches; PM053 owns explicit
relationship/occurrence authority and incomplete states. IMPORT_SPEC/PM011 own
stable import provenance. The manufacturing contract/PM054 preserve exact output,
atomic refusal and mandatory T04. The foundation source and every listed consumer
were reviewed at these affected scopes; unchanged historical research/approval
pins and source-health policy are retained. No controlling normative contract is
amended to ratify the proposed fields or lifecycle rules.

This packet is registered in the existing foundation evidence route. The review
handoff supplies the exact repository commit and this file's Git blob after the
correction commit; neither a mutable filename nor an implementer's verification
is independent re-review. Re-review is requested, not claimed passed. C03/C02
execution and current ownership survive; D1–D3 implementation stays blocked only
where dependent. Inspector layout, graphics placement UI, manufacturing CAM/T04,
performance budgets, deferred editors and the Claude prototype lane are unchanged.

## Response format

Reply per decision:
- `D1: ratify G1–G3 + clarifications 1–4 | amend: … | defer`
- `D2: option B + validated correspondence/replacement/retirement; Set Part: report stale; Run/Net: authoritative inclusion only | amend: …`
- `D3: ratify tuple (+ Deferred: keep|dissolve; stale: keep-flagged|dissolve) | amend: …`

The owner's choice will be recorded as numbered decision record(s) or a PM026
amendment, with same-change Frontier/PROGRESS/manifest governance. Execution then
proceeds within the resulting grant. The "dense compound" padlock threshold is
also open: today every compound is treated as dense. That one is a visual
disposition for the HUMAN re-review, not part of this packet.
