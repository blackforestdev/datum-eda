# M7 Imported Board Fidelity Issue Inventory

> **Status**: Active issue inventory for `M7-FIX-003`.
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
| `M7-IMP-005` | MEDIUM | Stage 1 | import | Additional silent fallbacks remain: unknown imported pad shape defaults to `Circle`; missing drill defaults to `0`. | [skeleton.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:662), [skeleton.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:674) |
| `M7-SCN-004` | MEDIUM | Stage 3 | scene-contract | Net extraction exists, but there is not yet a realistic multi-net imported-board fixture test proving `pad.net` resolves to the correct `Net` on a board like `datum-test`. | Gap identified from current fixture coverage; add realistic multi-net imported-board test in Stage 3 |
| `M7-IMP-010` | HIGH | follow-on | import | Through-hole / multi-layer pad layer identity collapses via shortcut. `parse_pad_copper_layer_anywhere` only recognizes `F.Cu` and `B.Cu` and returns the footprint placement layer (`package_layer`) otherwise, so pads whose `(layers ...)` list uses `*.Cu`, inner-copper names, or multi-layer hole semantics inherit the footprint layer instead of expressing the real copper-layer set. Observed on `datum-test`: through-hole pads render but their layer assignment is ambiguous. | [skeleton.rs parse_pad_copper_layer_anywhere](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:835) |
| `M7-IMP-011` | MEDIUM | follow-on | import + IR | `PlacedPad` carries a single primary copper layer only. Under `M7-IMP-010` Option A, multi-layer pad membership (through-hole pads spanning all copper layers, `F&B.Cu` pads, inner-layer-plus-outer combinations) is signalled implicitly by `drill > 0` and the canonical-primary-layer rule. If a downstream consumer (DRC connectivity across multilayer holes, routing layer-span, rendering of through-hole copper on every layer) needs the explicit full layer-set, the bounded rule is insufficient and the IR must grow (e.g. `PlacedPad.layers: Vec<LayerId>` or equivalent). Track this as a planned IR-expansion slice; do not retroactively expand `M7-IMP-010`. | [pad.rs PlacedPad](/home/bfadmin/Documents/datum-eda/crates/engine/src/board/pad.rs:20), [M7-IMP-010 brief IR limitation note](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_010_PAD_LAYER_IDENTITY_BRIEF.md) |
| `M7-REN-006` | MEDIUM | follow-on | renderer contract | The renderer remains a hybrid of layer-driven and object-class-driven behavior. `LayerAppearance` is still shaped around primitive families (`authored_track`, `pad_copper`, `zone_fill`, `via_outer`, `via_inner`), `push_retained_scene_geometry` still uses hand-authored per-class passes, and special cases such as vias, board graphics, and outline sit outside a generalized layer/material pipeline. This does not mean import items are individually styled ad hoc, but it does mean the renderer can still drift away from the product rule that authored geometry should inherit visibility and base appearance from owning layer/material semantics. Track as a renderer-contract discipline ticket; do not treat it as a purely cosmetic concern. **Architectural direction (set 2026-04-15, tightened 2026-04-16):** (1) define a render-material model per layer role; (2) make primitives inherit visibility + base appearance from that model; (3) keep only a small explicit exception set (e.g. board-frame outline overlay, selection state); (4) enforce a declared render-stack policy of layer-type first, then front/back side, then stable tie-breakers; (5) eliminate special-case visibility/color paths unless product semantics truly require them. Tactical fixes (TH pad color following the visible copper layer, outline layer-id alignment, process-layer front/back ordering, etc.) are steps inside this broader concern, not substitutes for it. | [gui-render/src/lib.rs LayerAppearance](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:361), [gui-render/src/lib.rs push_retained_scene_geometry](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1575), [M7_RENDER_LAYER_DISCIPLINE_MEMO.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md) |
| `M7-INT-001` | HIGH | cross-stage prerequisite | interaction substrate | Selection, hit ownership, focus/relatedness, and dimming behavior are still capable of making user-facing `M7` features ambiguous or effectively untestable. This is not "extra polish"; it is a prerequisite interaction-stability track for any slice that claims selectable/focusable behavior. A feature depending on these pillars must not be advanced on a low-resolution basis if the tester cannot intentionally trigger it, understand what state it is in, or rely on consistent ownership. | Repeated `datum-test` audits across Stage 4 work; see delivery rule in [M7_DELIVERY_GATES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_DELIVERY_GATES.md) |

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
  plus the broader fallback audit in `M7-IMP-005`

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

1. `M7-INT-001`
2. `M7-REN-006`
3. `M7-REN-003`
4. `M7-REN-004`
5. `M7-IMP-005`
6. `M7-SCN-001`
7. `M7-SCN-002`

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
