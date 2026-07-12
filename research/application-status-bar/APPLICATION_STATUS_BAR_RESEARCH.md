# Datum Application Status Bar Research

Status: delivered working research for GUI/S5 design review

## Purpose and traceability

This report evaluates whether Datum's global bottom **Application Status Bar**
earns its permanent screen space, what information can safely live there, and
which feedback must remain near the engaged workspace. Its derived integration
bridge is `docs/gui/DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md`. Governing GUI
specifications and owner review remain authoritative; this report does not
authorize implementation.

## Question

Datum's fast-workspace ethos keeps actionable information close to the user's
field of view. A global bottom bar can provide calm application awareness, but
it can also force attention away from a precise PCB/schematic task or become
ambiguous in a split workspace. The research asks whether the bar should be
retained, reduced, made optional/contextual, or removed.

## Current Datum evidence

The HTML prototypes define a 26-pixel full-window bottom strip beneath the
terminal dock:

- Board prototype: focused pane/surface, Tool, selection/cross-pane scope, Grid,
  DRC, revision, and development copy.
- Schematic prototype: focused pane/surface, Tool, selected net/pin count, Sheet,
  Grid, ERC, revision, and development copy.
- Workspace-panes prototype: focused pane/surface, Tool, Grid, and other compact
  segmented state.

Strengths:

- stable segmented layout;
- explicit focus ownership in prototypes;
- global check/revision state separated toward the right; and
- concise semantic selection in the schematic prototype.

Gaps:

- cursor X/Y and active layer promised elsewhere are absent;
- Board and Schematic schemas diverge without an explicit global-versus-pane
  ownership model;
- `Sel U1 (both panes)` cannot spatially locate the counterpart by itself;
- development/build copy consumes production attention space; and
- rich selection already has a dedicated Inspector, inviting duplication.

The current runtime further regresses semantic identity/count, PaneId, grid,
cursor coordinates, active layer, schematic ERC context, and prototype parity.
This is evidence of unclear ownership, not an instruction to restore every field
before deciding whether it belongs globally.

## Professional-tool precedent

### EDA

- KiCad places coordinates, relative deltas/distance, zoom, grid, units, and a
  selected-object message panel at the bottom. Selection handles, marquee,
  cursor/snap, and ERC/DRC markers remain on canvas.
- Altium's global Status Bar carries coordinates, prompts, shortcuts, grid/snap,
  active mode/layer, and panel access, and can be hidden. It separately offers a
  configurable near-cursor Heads Up Display for cursor/delta/layer/grid, object
  details, shortcuts, violations, and routing state.
- Current primary OrCAD evidence was insufficient for a reliable modern field
  inventory; historical material is not treated as current law.

### Creative/CAD applications

- Blender uses its global bar for active-tool shortcuts, transient reports and
  progress, and optional resource/scene facts; it is hideable and partly
  configurable. Durable details route to the Info Editor.
- SOLIDWORKS uses the bar for editing/document state, units, coordinates,
  concise measurements, and hover help; it is hideable.
- Illustrator keeps zoom/tool/artboard navigation in a global status bar, while
  properties live in panels and important next actions can appear in a movable,
  hideable near-object Contextual Task Bar.
- Bitwig's Window Footer uses a central contextual lane for hovered controls and
  active-gesture modifiers, but selection properties live in its Inspector and
  warnings have a separate notification system.
- Ableton uses a deliberately narrow Status Bar for concise errors and
  context-specific precision; richer learning/help lives in a separate
  hideable Info View.
- AutoCAD's highly configurable bar exposes coordinates and many drafting-mode
  toggles, but its Dynamic Input feature repeats coordinate/command input near
  the crosshair—evidence that distant global state does not replace proximal
  action feedback.

## Human-factors evidence

- The proximity compatibility principle supports placing displays used together
  for the same mental operation perceptually close, while warning that maximal
  integration is not universally correct.
- Split-attention research shows costs when users must mentally integrate
  separated mutually referring information. It is supporting rationale, not a
  direct CAD-status-bar experiment.
- Change-blindness research warns against making a brief peripheral change the
  sole acknowledgement of consequential action.
- Multi-display research supports treating attention switches as a cost/risk,
  without claiming that split panes inherently reduce performance.
- Calm-technology work supports stable global information in the periphery that
  moves to the center only when needed.
- Apple feedback guidance favors integrating passive status near the item it
  describes and reserving interruption for critical/actionable situations.
- WCAG status-message guidance requires programmatic status semantics; a visual
  strip mutation alone is insufficient for accessible feedback.

## Scope-to-location model

The strongest synthesized rule is:

| Information scope | Preferred location |
|---|---|
| Global and slow-changing | Optional Application Status Bar / global chrome |
| Persistent pane-specific | Pane header or pane-owned Inspector region |
| Gesture, hover, snap, route, invalid drop | Immediate canvas/pointer-local overlay |
| Selection attributes and blockers | Inspector plus local visual projection |
| Rich violations, history, logs | Dedicated panel/dock |
| Critical global failure/progress | Persistent global indicator with details path; never peripheral-only |

## Candidate Application Status Bar roles

### Option A — remove it

Move pane context to pane headers, selection to Inspector/canvas, checks/jobs to
global header indicators or docks, and coordinates/snap to local overlays.

Benefits: maximum canvas focus and no ambiguous global pane state.
Risk: no consistent low-interruption place for background progress, dirty/model
state, or terse global health.

### Option B — minimal calm global bar

Keep only project/global state such as dirty/model revision, aggregate check
severity, background task/progress, connectivity/recompute state, and an
accessible details affordance. Make it independently hideable.

Benefits: peripheral awareness without rapid pointer-driven churn.
Risk: permanent space for information some users rarely need.

### Option C — hybrid contextual bar

Use a stable global cluster plus one replacement contextual lane for the active
workspace/tool/gesture. Never accumulate unrelated fields; local overlays remain
the authoritative action feedback.

Benefits: discoverable terse hints and progress.
Risk: still encourages eye travel and can duplicate pane-local information.

### Option D — configurable CAD-style dashboard

Expose many coordinates/grid/snap/layer/tool toggles and allow customization.

Benefits: familiar and flexible.
Risk: highest clutter/ambiguity cost and weakest fit with Datum's Lean ethos.

## Design criteria before ratification

1. No consequential commit, refusal, invalid placement, lock conflict, selection
   scope change, or check failure may rely on the bar alone.
2. Pointer coordinates, snap target, dx/dy, marquee/lasso mode, route preview,
   and hover identity should remain in the pointer-containing pane when needed.
3. Tool/command authority follows the magenta-framed active workspace.
4. A global field derived from a pane must name its pane/surface; otherwise it is
   ambiguous and should not be shown.
5. Rich selection belongs in Inspector; cross-probe belongs on corresponding
   objects in both panes.
6. Warnings/progress need persistent severity/state and a route to durable
   detail. Color or transient text alone is insufficient.
7. Routine pointer motion must not generate accessibility announcement spam;
   commit/refusal/check completion must be programmatically available.
8. Narrow-window priority must preserve critical global health before
   development/version or low-value fields.
9. If the bar is hideable, safety truth must remain visible/accessible elsewhere.
10. Validate with task tests: missed feedback, wrong-pane commands, blocker
    comprehension, check-state interpretation, and split-view focus prediction.

## Recommendation pending owner review

The research favors Option B (minimal calm global bar) or removing the bar over
a CAD-style dashboard. If retained, active-work details should migrate toward
the pane/cursor/Inspector rather than being duplicated globally. This is a
research recommendation, not a ratified decision.

## Primary sources

- KiCad PCB Editor: <https://docs.kicad.org/master/en/pcbnew/pcbnew.html>
- KiCad Schematic Editor: <https://docs.kicad.org/master/en/eeschema/eeschema.html>
- Altium environment elements: <https://www.altium.com/documentation/altium-designer/design-environment-elements>
- Altium Board Insight: <https://www.altium.com/documentation/altium-designer/pcb/board-insight-system>
- Blender Status Bar: <https://docs.blender.org/manual/en/5.0/interface/window_system/status_bar.html>
- SOLIDWORKS Status Bar: <https://help.solidworks.com/2026/english/swtutorialonline/c_tut_status_bar.htm>
- Illustrator workspace: <https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/workspace-overview.html>
- Illustrator Contextual Task Bar: <https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/contextual-task-bar-overview.html>
- Bitwig Window Footer: <https://www.bitwig.com/userguide/latest/the_window_footer>
- Ableton Live concepts: <https://www.ableton.com/en/live-manual/12/live-concepts/>
- AutoCAD Status Bar: <https://help.autodesk.com/cloudhelp/2022/ENU/AutoCAD-Core/files/GUID-C5C9380F-5469-4858-B306-B1BFFC19C0A9.htm>
- AutoCAD Dynamic Input: <https://help.autodesk.com/cloudhelp/2022/ENU/AutoCAD-DidYouKnow/files/GUID-683349C0-E5C2-4E16-8846-5523E71172A9.htm>
- Proximity compatibility: <https://journals.sagepub.com/doi/10.1518/001872095779049408>
- Split attention: <https://bpspsychub.onlinelibrary.wiley.com/doi/10.1111/j.2044-8279.1992.tb01017.x>
- Change blindness: <https://www2.psych.ubc.ca/~rensink/publications/abs.00.12.html>
- Calm technology: <https://calmtech.com/papers/designing-calm-technology>
- Apple feedback guidance: <https://developer.apple.com/design/human-interface-guidelines/feedback>
- WCAG status messages: <https://www.w3.org/WAI/standards-guidelines/wcag/new-in-21/#413-status-messages>

## Open decisions

- Remove versus minimal calm retained bar.
- If retained, fixed minimum fields, hiding policy, and narrow-window priority.
- Whether any contextual lane is justified or all active-work hints stay local.
- Global check/progress placement if the bar is removed.
- Pane-local coordinate/readout form and accessibility behavior.
