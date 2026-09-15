# M7 Imported Board Fidelity Issue Inventory

> **Status**: Historical -- `M7-FIX-003` issue inventory, M7 spike closed-for-scope (import frozen); retained as historical evidence.
> This document records the currently known imported-board fidelity gaps with
> ticket IDs, severity, stage mapping, and concrete file evidence.

## Purpose

Provide one tracked defect inventory for the imported-board fidelity program so
the team can:
- triage in a consistent place
- map defects to the staged checklist
- justify stage exits using concrete code evidence

This inventory is the working output for:
- `M7-FIX-003` in `docs/gui/M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md`

Representation-invariance policy for this inventory:
- a ticket is not considered closed if it only works for one fixture because
  that fixture exercises a different KiCad representation path
- optional KiCad representations such as `render_cache` may not define final
  imported-board truth; for board text specifically, see
  `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
- if `render_cache` present vs absent changes effective position, orientation,
  layer meaning, or review behavior for the same authored intent, track it as
  an imported-board fidelity regression

## Usage Rules

- Severity reflects roadmap impact, not only local code cleanliness.
- `HIGH` means the defect blocks the corresponding stage exit.
- `MEDIUM` means the defect should be addressed in the current track but does
  not independently block the stage exit unless the architect says otherwise.
- `OK` means the audited area does not currently show a fidelity gap on the
  accepted evidence set.
- A ticket is not allowed to count as "done enough" if it fails the delivery
  gates in `docs/gui/M7_DELIVERY_GATES.md`, even if code exists for the slice.

## Active Gaps

| Ticket | Severity | Stage | Area | Gap | Evidence |
|--------|----------|-------|------|-----|----------|
| `M7-IMP-005` | MEDIUM | Stage 1 | import | **Parse-or-account discipline landed for board top-level objects (2026-06-10):** the importer can no longer silently discard segments, vias, zones, gr_text, or net-table blocks. Every non-imported source block emits an explicit `import dropped <form> <uuid>: missing or unparseable <fields>` warning, and a per-form conservation check (`check_form_accounting` in `import/kicad/board_objects.rs`) reports loudly if any code path discards data without accounting. First catch: 11 KiCad teardrop fill zones on DOA2526 (generated copper with no `(uuid)`) were being silently dropped; they now import under deterministic derived identities (DOA2526 zones 7 → 18). The GUI load path previously discarded the import report entirely; it now prints `datum-import warning` lines to stderr. Regression locked by `datum_test_board_import_accounts_for_every_source_object` (ungated) and `real_doa2526_import_has_no_silent_drops_or_accounting_mismatch` (env-gated). **Remaining scope:** audit pad-level numeric fallbacks (missing drill defaults to 0; geometry defaults) and apply the same accounting to footprint-nested forms and the schematic importer. | [board_objects.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/board_objects.rs), [skeleton.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs) |
| `M7-SCN-004` | MEDIUM | Stage 3 | scene-contract | Net extraction exists, but there is not yet a realistic multi-net imported-board fixture test proving `pad.net` resolves to the correct `Net` on a board like `datum-test`. | Gap identified from current fixture coverage; add realistic multi-net imported-board test in Stage 3 |
| `M7-IMP-010` | HIGH | follow-on | import | Through-hole / multi-layer pad layer identity collapses via shortcut. `parse_pad_copper_layer_anywhere` only recognizes `F.Cu` and `B.Cu` and returns the footprint placement layer (`package_layer`) otherwise, so pads whose `(layers ...)` list uses `*.Cu`, inner-copper names, or multi-layer hole semantics inherit the footprint layer instead of expressing the real copper-layer set. Observed on `datum-test`: through-hole pads render but their layer assignment is ambiguous. | [skeleton.rs parse_pad_copper_layer_anywhere](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:835) |
| `M7-IMP-011` | MEDIUM | follow-on | import + IR | `PlacedPad` carries a single primary copper layer only. Under `M7-IMP-010` Option A, multi-layer pad membership (through-hole pads spanning all copper layers, `F&B.Cu` pads, inner-layer-plus-outer combinations) is signalled implicitly by `drill > 0` and the canonical-primary-layer rule. If a downstream consumer (DRC connectivity across multilayer holes, routing layer-span, rendering of through-hole copper on every layer) needs the explicit full layer-set, the bounded rule is insufficient and the IR must grow (e.g. `PlacedPad.layers: Vec<LayerId>` or equivalent). Track this as a planned IR-expansion slice; do not retroactively expand `M7-IMP-010`. | [pad.rs PlacedPad](/home/bfadmin/Documents/datum-eda/crates/engine/src/board/pad.rs:20), [M7-IMP-010 brief IR limitation note](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_010_PAD_LAYER_IDENTITY_BRIEF.md) |
| `M7-IMP-014` | HIGH | Stage 1 follow-on | import + scene-contract | Imported KiCad text still has representation-dependent geometry quality. Research shows `render_cache` is not the general parity oracle originally assumed: KiCad emits it only for non-default TrueType / `OUTLINE_FONT` text, not for default Newstroke text. That means the current `DOA2526` vs `datum-test` quality gap is largely TrueType-authored cached outlines versus Datum's weak internal fallback, not simply cache-present versus cache-absent encoding of the same font. The fix is a Datum-owned Newstroke-equivalent generator used for all imported text in Phase 1, with `render_cache` removed as final render truth and retained only for bounded local analysis. No KiCad runtime dependency is allowed. Expected consequence: imported TrueType-authored fixtures may visibly change under Phase 1 until later TrueType support lands. | [research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md), [M7_RENDER_SEMANTIC_CONTRACT.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md), [M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md), [crates/gui-protocol/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:3168), [crates/gui-protocol/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:3249), [crates/gui-protocol/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:3498), [crates/engine/src/export/silkscreen.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/export/silkscreen.rs:13) |
| `M7-IMP-016` | HIGH | follow-on | import | KiCad format `20260206` follow-ups beyond net references. With name-form nets now importing, the env-gated external DOA2526 suite (`DATUM_RUN_EXTERNAL_DOA2526_TESTS=1`) still shows: (1) pad world-rotation deltas vs recorded expectations (0 vs -90, 180 vs 0 in `mod_tests_import_kicad_pad_rotation.rs`) — possibly a real `20260206` rotation-encoding semantic change, IMP-008 class; (2) stale expectations against the re-saved project (net names lost their `/` prefix for root nets, e.g. `/VCC` → `VCC`; `+IN` pin position moved; `#FLG01` gone); (3) `imports_real_doa2526_board_with_copper_geometry` diagnostics assert now trips on a HEALTHY import (exactly one `net_without_copper` diagnostic remains). Fixture truth needs owner confirmation before expectations are rewritten; the rotation delta needs a real-format audit, not an expectation update. | Failing env-gated tests in [mod_tests_import_kicad_doa2526.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/tests/mod_tests_import_kicad_doa2526.rs) and [mod_tests_import_kicad_pad_rotation.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/tests/mod_tests_import_kicad_pad_rotation.rs) as of 2026-06-10 |
| `TXT-ENG-002` | HIGH | post-`M7-IMP-014` | engine architecture | Datum now owns a working stroke text engine, but it remains a single-backend system. Phase 2 research confirms the next product-grade step is a three-layer text architecture (semantic model / layout engine / glyph backend) plus a hybrid backend strategy: stroke-default for `Manufacturing`, outline-default for `Annotation` / `Branding` / `Documentation` / `UiPreview`. This follow-on also carries the determinism risk for outline flattening and the bundled-font policy. Treat it as a core engine track, not an importer follow-up. | [DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md), [DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md), [DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md) |

## Audited OK Areas

| Ticket | Status | Note |
|--------|--------|------|
| `M7-IMP-006` | OK | Supported pad shapes currently parse correctly for the audited subset. |
| `M7-IMP-007` | OK | Size/drill to nm extraction appears correct for the audited subset. |

## Closed Tickets

| Ticket | Status | Closeout note |
|--------|--------|---------------|
| `M7-IMP-001` | CLOSED | Unknown KiCad layer names no longer silently collapse to fallback copper. Layer resolution now returns explicit bounded import errors, call sites were updated accordingly, and six focused tests cover bounded fallback, parsed table precedence, explicit unknown-layer failure, integration failure on bad track layer, and inner-layer table capture. |
| `M7-IMP-003` | CLOSED | Outline extraction now accepts both top-level `gr_line` / `gr_arc` and footprint-embedded `fp_line` / `fp_arc` on `Edge.Cuts` under the bounded Option A ownership rule. The old silent `10mm x 10mm` placeholder success path is gone; missing or unassemblable outline now returns an empty outline plus an explicit warning instead of fake board truth. The parser helper documents the accepted contributor set directly in code, and the importer carries the warning out of skeleton construction. |
| `M7-IMP-008` | CLOSED | `PlacedPad` now carries `rotation`, KiCad import preserves pad-local authored rotation, and focused import tests cover rotated-pad behavior on both `datum-test` and DOA2526. The scene contract forwards `rotation_degrees`, and the renderer consumes it when rasterizing pad geometry. |
| `M7-IMP-009` | CLOSED | `PlacedPad` now carries `roundrect_rratio_ppm`, KiCad import parses `roundrect_rratio`, the scene contract forwards the corner ratio, and the renderer uses it to distinguish roundrect pads from plain rectangles. Focused import/render tests cover ratio preservation and geometry effect. |
| `M7-SCN-007` | CLOSED | Edge.Cuts authored-layer parity now follows Option B. Imported boards emit authored `board_graphics` primitives from real Edge.Cuts contributors, while native-project boards derive bounded board-scoped `board_graphics` from the persisted assembled outline so visibility, stacking, and picking participate in the same authored-layer lane. `scene.outline` remains the board-boundary view. Remaining limitation: native projects still do not preserve original per-contributor Edge.Cuts identity from engine storage; the native path uses stable synthetic outline-segment ids instead. |
| `M7-SCN-006` | CLOSED | `OutlinePolyline` now carries `layer_id`, defaulting to the scene Edge.Cuts lane key, so scene-level outline rendering can participate in the same layer-visibility model as other authored geometry. This follow-on is effectively superseded by the broader authored Edge.Cuts handling under `M7-SCN-007`, but the concrete layer-id gap itself is no longer open. |
| `M7-APP-001` | CLOSED | Runtime panic paths in `gui-app` were replaced with a bounded fatal-error path. Window creation, runtime creation, redraw/render failures, and internal render-scene precondition failures now emit a single-line `datum-gui error: ...` message to stderr and exit cleanly with status 1 instead of panicking with a backtrace. |
| `M7-SCN-003` | CLOSED | Imported-board `board_review_scene_v1` now carries explicit `unrouted_primitives`, separating authored, unrouted, proposed, and diagnostic lanes rather than inferring unrouted state in the renderer. |
| `M7-SCN-004` | CLOSED | Imported-board unrouted connectivity now comes from engine-backed `Board::unrouted()` truth with explicit endpoint positions, so the canonical half-routed board exposes deterministic remaining ratsnest state instead of renderer inference. |
| `M7-SCN-005` | CLOSED | The unrouted lane now has explicit visibility control and stable scene identities. `UNROUTED` is a first-class shell filter, and the renderer/app wiring treats the lane distinctly from authored and proposed geometry. |
| `M7-REN-001` | CLOSED | The opening `M7` renderer vocabulary is now written down explicitly for authored, unrouted, proposed, and diagnostic states in `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`, so later visual tweaks have a bounded semantic contract instead of drifting by taste. |
| `M7-REN-002` | CLOSED | The unrouted/ratsnest lane now follows a bounded PCB-native grammar on the canonical fixtures: solid thin airwire lines, subtle endpoint anchors that do not read as vias or drills, contrast under-stroke for legibility, and deterministic per-net color identity. The lane is now functionally and semantically distinct from both authored copper and proposed overlays; remaining improvements are polish-tier rather than fidelity-blocking. |
| `M7-REN-005` | CLOSED | Scene outline rendering now respects both `authored_visible(state)` and the outline `layer_id`, so users can intentionally hide the board-boundary view through the authored and Edge.Cuts visibility controls. The original outline-visibility bug is no longer the active follow-on; the remaining renderer frontier is the broader layer/material discipline tracked under `M7-REN-006`. |
| `M7-IMP-012` | CLOSED | Pad-level mask/paste process semantics now come from the real KiCad source instead of being derived from copper pad geometry plus board-global defaults. Import now preserves `(layers ...)` membership, board-level setup, and pad/footprint-level `solder_mask_margin`, `solder_paste_margin`, and paste-ratio overrides through to the scene and renderer. DOA2526 is the canonical proof case. This is also the first standards-bound imported-board slice that must explicitly honor the IPC observables in [research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md), especially IPC-7525 stencil reduction and IPC-7351/7352 land-pattern/process aperture intent. |
| `M7-INT-001` | CLOSED (first slice, 2026-06-09) | Authored-object selection ownership and relatedness are now stable and regression-locked on the canonical `datum-test` fixture. The first-slice proof obligations from the brief are covered by `crates/gui-render/tests/selection_ownership.rs`: selecting one component changes retained vertex state only inside that component's pad bounds (exclusive-ownership sweep across all other components), switching selection clears the prior owner, and hover is preview-only (with an explicit selection active, hovering another object leaves selection and retained world geometry unchanged). Review-target persistence across authored selection changes was already locked at the protocol level (`authored_object_selection_preserves_active_review_target`, `clearing_authored_selection_preserves_active_review_target`). The retired color-exact selected-pad test was replaced with lane-aware vertex-state assertions per the brief. Remaining interaction work (multi-select, keyboard expansion, final dimming strength tuning) is explicitly out of slice scope and belongs to later tickets. |
| `M7-IMP-017` | CLOSED (2026-06-10) | Two `20260206`-format follow-up import fixes from the teardrop render audit. (1) Zone fills now import **all** `filled_polygon` islands per layer instead of the first only (one zone per island, deterministic derived ids); a zone with no per-layer fill falls back to the authored boundary template **with an explicit warning** instead of silently. (2) `*.Cu` wildcard pad expansion no longer force-injects hardcoded layer ids 0/31 when a parsed layer table exists — on renumbered `20260206` tables id 31 is `F.CrtYd`, which was polluting pad copper sets with a courtyard layer. Pixel-level render verification on DOA2526 confirmed stack order is correct post-fix (pad annulus over teardrop root; B.Cu tail visible through F.Cu pour clearance per file truth). Residual delta vs KiCad is presentation only (opaque flat colors, no mask-ring context) — that readability concern belongs to `M7-REN-004`, not import. |
| `M7-IMP-015` | CLOSED (2026-06-10) | KiCad format `20260206` references nets by quoted name (e.g. `(net "/IN_P")`) and emits no top-level integer net table. The importer only understood integer net codes, so every segment, via, and zone on a `20260206` board was silently dropped (observed live: DOA2526 rendered pads but no traces; engine import produced tracks=0 vias=0 zones=0 nets=0 despite 148 segments and 46 vias in the source). Fix: `import/kicad/net_refs.rs` adds a format-aware `NetRef` (integer code or quoted name, parsed quote-first so names like `Net-(Q1-C)` survive) and `resolve_board_net_ref`, which derives a deterministic UUID from the net name and registers name-referenced nets on first sight. Applied at all four reference sites (segments, vias, zones, pads). After the fix DOA2526 imports tracks=148 vias=46 zones=7 nets=24 and traces render; datum-test (`20241229`) import is unchanged (tracks=32 nets=11). Regression locked by the env-gated `real_doa2526_name_referenced_nets_resolve_for_routed_copper` plus the pre-existing copper-geometry assertions. Remaining `20260206` issues are tracked as `M7-IMP-016`. |
| `M7-REN-006` | CLOSED (2026-06-09) | Layer/material render discipline is now defined and machine-enforced. The render-stack policy has exactly one encoding: `RenderStage` declaration order is the draw order, `render_stage_priority` is its discriminant, the divergent derived `Ord` (paste-before-mask) was corrected, and the three duplicated `post_copper_stages` arrays were consolidated into one shared `POST_COPPER_STAGES` walk. Known copper families construct `LayerAppearance` through a material-first constructor (`from_copper_material`) so track/pad/zone inheritance of one base material is structural. The bounded exception set (through-hole pad pass, via family, board outline/Edge overlay, selection/hover emphasis, unknown-layer fallback) is documented at the `push_retained_scene_geometry` contract header, and contract regression tests lock the declared stage ladder, the single-ordering-encoding rule, and copper material-first inheritance. Per the memo the renderer remains a deliberate stricter hybrid; deeper unification of via/through-hole-pad rendering into a generalized copper pipeline is future work outside this ticket. |
| `M7-IMP-013` | CLOSED | Imported KiCad footprint text no longer depends on KiCad `render_cache` presence for correct upright semantics. The scene contract now normalizes imported text rotation into KiCad's upright range before rendering, and `datum-test` regression coverage proves that fallback text synthesis preserves effective orientation semantics when cached text polygons are absent. Remaining future work, if needed, is fallback glyph quality rather than representation-dependent meaning. |

## Current Stage Blocking Read

### Stage 1 blockers

- none at the previously recorded silent-corruption level

Stage 1 remains blocked until the team can show:
- supported outline behavior is explicit under the chosen Option A ownership
  rule
- unsupported ambiguous ownership cases do not silently pass as imported truth
Current read:
- the highest-risk Stage 1 silent outline corruption path is closed
- remaining Stage 1 work is validation hardening (`M7-IMP-002`, `M7-IMP-004`)
  plus the broader fallback audit in `M7-IMP-005` and imported-board text
  normalization in `M7-IMP-014`

### Stage 2 blockers

- none at the previously recorded pad-rotation / roundrect-semantics level

Stage 2 remains blocked until the team can show:
- rotated non-circular pads no longer lose required geometry semantics
- roundrect pads preserve the corner-semantics needed for credible review
Current read:
- those two specific blockers are now closed in code and test coverage
- the remaining Stage 2 frontier is footprint-context credibility and scene
  companion work (`M7-SCN-001`, `M7-SCN-002`)

## Recommended Immediate Work Order

After `M7-FIX-001` and `M7-FIX-002` are closed, the next highest-value
implementation order is:

1. `M7-REG-001`..`M7-REG-003` — fixture-backed import/scene/visual regression
   coverage for the supported `M7` view (owner-ordered 2026-06-11: pulled
   ahead of remaining import/scene work so the readability slices get locked
   by fixtures immediately)
3. `M7-IMP-005` (remaining pad-level numeric fallbacks)
4. `M7-SCN-001`
5. `M7-SCN-002`

(`M7-INT-001` first slice and `M7-REN-006` closed 2026-06-09; `M7-REN-003`
and `M7-REN-004` closed 2026-06-12 — proposed overlays verified copper-like
in both states, the per-vertex diagnostic marker violation removed, zone
copper tone-separated as a derived material shade, and dim-unrelated
readability verified on the canonical fixture. Stage 4 is closed; the
frontier is Stage 5 regression coverage.)

Rationale:
- the previously documented Stage 1 / Stage 2 import-semantic blockers have
  materially landed and should stop being treated as the active execution
  frontier
- the delivery gates now point at interaction stability and layer/material
  readability as the next real blockers for credible review
- broader fallback cleanup should still happen, but it is no longer the only
  high-impact frontier
- footprint companion work remains necessary for Stage 2 credibility after the
  lower-level pad semantics are in place

## Active Implementation Briefs

- `M7-IMP-001`:
  [docs/gui/M7_IMP_001_LAYER_FALLBACK_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_001_LAYER_FALLBACK_BRIEF.md)
- `M7-IMP-003` decision memo:
  [docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_DECISION.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_DECISION.md)
- `M7-IMP-003` implementation brief:
  [docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_BRIEF.md)
- `M7-IMP-014` implementation brief:
  [docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md)
- `M7-IMP-014` implementation plan:
  [docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md)
- PCB text rendering research:
  [research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md)
- Phase 2 text engine research:
  [research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md)
- Phase 2 text engine brief:
  [docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md)
- Phase 2 text engine implementation plan:
  [docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md)
- `M7-INT-001` first-slice brief:
  [docs/gui/M7_INT_001_INTERACTION_STABILITY_BRIEF.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_INT_001_INTERACTION_STABILITY_BRIEF.md)
- render-architecture memo:
  [docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md)

## Standards Impact On M7

The standards/compliance specification changes the imported-board fidelity track
in one bounded way:
- `M7` must preserve and credibly present standards-relevant imported
  observables once they are exposed in the review surface
- `M7` may add bounded import-audit diagnostics for recognized standards-aware
  observables
- `M7` may not silently heal imported geometry toward an inferred IPC result

Near-term standards-facing work that remains in-scope for opening `M7`:
- preserving imported copper/drill/mask/paste/thermal-via truth
- exposing structured review findings when imported geometry differs from a
  declared or inferred standards basis, where the rule basis is explicit enough

Work that remains out of scope for opening `M7`:
- full IPC footprint wizard / generator
- broad library-wide standards validation
- product-level compliance claims
