# Datum S5 Selection Visual-Language Research

Status: delivered working research for `dat-s5-selection-visual-contract-zid`

## Purpose and traceability

This report audits Datum's existing selection visuals and defines the decision
surface required to complete S5 without inventing a rival style. Its derived
bridge is `docs/gui/DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md`; final visual
authority remains `docs/gui/DATUM_RENDERING_BOOK.md`, the controlling prototypes,
and owner-ratified S5 specification text.

## Established authority

- `content.selection` / application accent is `#CE5A92` across surfaces.
- Selection is screen-only consumer state: never authored, journaled, exported,
  or part of CAM/manufacturing geometry.
- The Rendering Book's complete schematic-symbol example accents the whole
  symbol—body outline, pins, terminal dots, reference/value, and text—under one
  soft glow; attached nets remain normal.
- `rendering-study.html` and `schematic-editor.html` demonstrate whole-symbol and
  selected-net/cross-workspace path glow.
- Hover is preview-only; the owner has ratified that selection wins when the same
  object is both selected and hovered.
- The magenta pane frame identifies the only GUI workspace with mutation
  authority while the shared selection may project into other panes.
- Hidden selection identity persists but hidden geometry emits no canvas
  projection. Locked objects remain selectable/inspectable but immutable.

## Incomplete contract

The current docs do not completely define:

- per-class PCB and schematic selection treatment;
- parent versus editor-child ownership visuals;
- section, connected-run, global-net, compound, and `Ctrl+A` scopes;
- selection versus related/cross-probe, proposal, diagnostic, lock, focus-member,
  and pane-focus channels;
- precise accent/glow geometry and the generic UVT `2px halo` wording;
- large-selection work budgets/LOD;
- non-color accessibility and high-contrast behavior; or
- complete conformance/golden/HUMAN evidence.

## Current implementation conflict

PCB tracks, pads, vias, and components currently bake selected recoloring or
weight changes into retained world buffers, and existing tests require retained
vertex changes. Other classes such as board text, graphics, and outline use a
lightweight projected overlay. UVT-003 requires one immediate screen-space
selection overlay path whose updates leave retained authored buffers unchanged.
The old retained-selection tests therefore encode migration debt, not the final
S5 contract.

## Research-derived compositing model

Visual states are orthogonal channels, not one winner-takes-all color enum:

1. **Visibility/eligibility gate:** hidden geometry emits no canvas state.
2. **Authored baseline:** normal material/layer/topology remains the document
   truth underneath presentation chrome.
3. **Related context:** subordinate cross-probe/relationship treatment.
4. **Proposal/review:** distinct ghost/dual-stroke proposed geometry.
5. **Diagnostic/evidence:** semantic hue plus marker shape.
6. **Hover:** lighter transient preview on unselected eligible objects.
7. **Selection:** authoritative object-shaped accent/glow projection.
8. **Lock/focus-member markers:** shape-coded orthogonal constraint/reference
   cues that coexist with selection.
9. **Workspace focus:** pane frame/header outside world geometry; sole GUI
   mutation-authority cue.

Selection wins over hover on the same object. Selection must remain legible with
proposal/diagnostic channels without erasing their semantic meaning. Pane focus
must never be inferred from a brighter/dimmer selected object.

## Object-family findings

### Schematic

| Identity/scope | Established or safest visual | Open detail |
|---|---|---|
| Symbol parent | Complete symbol accent + one soft glow; dark body fill retained; attached nets normal | Active/inactive projection explicitly ratified in final matrix |
| Pin in schematic | Parent symbol visual; pin is not independent membership | Cursor affordance only versus sub-hit marker |
| Pin in Symbol Editor | Pin stub, terminal, name/number as one child identity | Terminal distinction and exact text inclusion |
| Wire section | Exact centerline/path accent + glow; adjacent sections normal | Endpoint handles only in edit mode |
| Connected run | Every member section receives section treatment; no giant bbox | Junction/label relation treatment |
| Global net | All resolved occurrences visible across sheets/workspaces | Membership versus related pins/labels/pads/zones |
| Bus | Authored bus spine and owned name as one identity | Entry ownership/member expansion |
| Label | Border/text/pointer as one identity; dark fill retained | Related treatment under net selection |
| Junction | Preserve circular semantic core plus selection cue | Ring/glow form |
| No-connect | Complete X/flag accent + glow | Parent-pin related cue |
| Standalone text | Oriented semantic bounds and/or glyph accent | Exact glyph-versus-box treatment |
| Drawing | Exact path/perimeter; filled object keeps non-opaque interior | Editor ownership and filled treatment |

### PCB

| Identity/scope | Established or safest visual | Current gap |
|---|---|---|
| Footprint parent | Coherent owned visible presentation; connected tracks normal | Scene lacks complete lock/capability data |
| Pad in Footprint Editor | Pad-silhouette cue preserving copper/drill | Retained selected recolor loses material identity |
| Track section/run | Accent casing/glow following path with layer-colored core | Retained recolor/weight violates overlay law |
| Via | Accent outer cue preserving copper and drill void | Selected state baked into retained helper |
| Zone | Authored boundary cue; fill/islands remain one identity | No complete selected-zone overlay |
| Board text | Glyph/tight oriented treatment | Current loose bbox halo is fallback only |
| Graphic/outline | Shape-aware line/arc/perimeter cue | Magic widths and incomplete shape handling |
| Dimension | Whole annotation identity | Missing GUI projection/hit identity |
| Global net | Visible resolved member projection across workspaces | Direct selection versus related net visualization |

## Compound and scope findings

- Every actual visible member receives its class-specific selection treatment.
- Do not draw a permanent giant compound bounding box; show bounds/reference only
  while a transform requiring them is armed.
- Optional focus member must not gain stronger mutation authority. If shown, use
  a small shape/anchor marker rather than another glow or arbitrary color.
- Locked members retain normal selection plus a non-color lock cue; they expose
  no transform handles.
- Hidden members remain in the selection set but produce no canvas cue.
- A large/global selection needs deterministic viewport culling, batching, an
  emission budget, and an explicit aggregate fallback; never silently truncate
  membership.

## Glow/halo reconciliation

The prototypes implement accent geometry plus a Gaussian soft glow. UVT's
`Selection highlight | A | 2.0 px halo` is too generic to govern every class.
The final decision must define:

- whether 2 physical pixels describes a mandatory crisp accent boundary/casing,
  a minimum visible footprint, or a glow extent;
- whether material-bearing PCB cores remain visible beneath an accent casing;
- which schematic presentation primitives may become accent-colored;
- dark-fill retention for symbols/labels; and
- exact high-contrast fallback when soft glow is unavailable.

Regardless of styling, selection geometry remains class-A screen-space chrome,
zoom invariant, pane clipped, and excluded from hit/qualification bounds.

## Accessibility requirements

- State is never color alone: selection has continuous geometry, lock uses a
  glyph/hatch/notch, related uses pattern/secondary form, diagnostics use marker
  shapes, and workspace focus uses frame plus header/focus dot.
- Meaningful non-text state graphics target WCAG 3:1 contrast against adjacent
  board/schematic/material colors.
- High-contrast mode replaces subtle aura/tint with crisp boundaries/patterns.
- Selection never pulses. Reduced motion preserves static state meaning.
- Tiny point objects receive a minimum screen-space cue without enlarging
  authored geometry.
- Programmatic selection/focus/hidden/locked/refusal state is available without
  announcing continuous pointer hover motion.

## Performance and retention requirements

- Selection/hover/cursor changes update only dedicated post-world interaction
  buffers: no retained scene reconstruction, model re-resolution/import, static
  authored/grid rebuild, or unrelated-pane upload.
- Every pane projects the same selection through its own live camera.
- Overlay lookup uses retained indexed metadata; pointer motion never scans the
  full scene.
- Multi-selection overlay generation is viewport-culled, deterministic, bounded,
  and capability-honest. Exact cap/fallback remains an owner/spec decision.
- No animation may invalidate retained geometry continuously.

## Required conformance evidence

1. State reducer tests for selected+hovered, locked, hidden, related, focus,
   capture/focus loss, and no arbitrary focus-member promotion.
2. Exact physical-pixel/scale/zoom tests for selection and hover cues.
3. Per-class overlay coverage for all honestly supported S5 identities.
4. Retained-world byte identity/static-buffer stability across selection changes.
5. Duplicate-pane independent-camera projection.
6. Large-design candidate/emission budget and deterministic fallback.
7. Non-text contrast, grayscale/color-vision, high-contrast, and reduced-motion
   snapshots.
8. Explicit golden state manifests and HUMAN reference review at multiple zooms,
   scales, dense designs, and maximum state collisions.

## Owner decisions still required

Owner decision closed 2026-07-12: slightly brighten the complete owned visible
symbol/footprint/object presentation while preserving semantic/material hues,
then add the selection-accent internal glow and crisp object-shaped 2px channel.

Owner decision closed 2026-07-12: the actual shared selection projects at full,
identical strength in active and inactive workspaces. Only the magenta pane
frame/focus header/tool availability communicates GUI mutation authority.

Owner decision closed 2026-07-12: triple-click selects one semantic Global Net
subject. All visible resolved electrical projections (wire/label/port/pin
terminal, track/via/pad connection/zone/airwire) receive full selection; parent
symbol/footprint bodies remain related rather than selected.

Owner decision closed 2026-07-12: merely related objects retain exact authored
appearance—no brightening, recolor, glow, or selection accent. During explicit
relationship context, unrelated geometry may dim mildly while related geometry
stays baseline; durable explanation belongs in Inspector.

Owner decision closed 2026-07-12: an optional compound focus member receives no
extra persistent canvas treatment. Inspector/session state identifies it;
commands that need a reference render a temporary command-owned marker.

Owner decision closed 2026-07-12: locked objects are slightly neutral-greyed,
retain full selection, suppress transform handles, use a locked cursor, and show
a small anchor padlock when selected/hovered. The glyph is gated by icon-set
declaration, the Rendering Study contact sheet/style, and HUMAN review; dense
compounds avoid repeated-icon clutter and report counts in Inspector.

Owner decision closed 2026-07-12: authored base, proposal ghost/dual stroke,
selection cue, then topmost semantic diagnostic marker. Selecting a proposal or
diagnostic adds selection without erasing proposal/severity identity; no channel
is flattened into plain magenta.

Owner decision closed 2026-07-12: bus click depth is local section → connected
run → semantic hierarchical bus. Spine, owned name/label, and attached entries
project as one subject; scalar member nets remain independent.

Still required:

1. Standalone text and point-object treatments.
2. Dense-selection attenuation/budget fallback.

## Primary Datum sources

- `docs/gui/DATUM_RENDERING_BOOK.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/DATUM_GUI_DESIGN_SPEC.md`
- `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`
- `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`
- `docs/gui/prototypes/rendering-study.html`
- `docs/gui/prototypes/schematic-editor.html`
- `crates/gui-render/src/render/retained.rs`
- `crates/gui-render/src/render/overlay.rs`
- `crates/gui-render/src/render/interaction_overlay.rs`
- `crates/gui-render/tests/selection_ownership.rs`
