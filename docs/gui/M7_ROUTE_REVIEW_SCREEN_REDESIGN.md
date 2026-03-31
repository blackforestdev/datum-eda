# M7 Route Review Screen Redesign

> **Status**: Approval-ready screen redesign spec for the current `M7`
> route-proposal review workspace.
> This document is screen-specific guidance for the existing locked `M7`
> workflow. It does not change `M7` scope or semantics.

## Purpose

Define how the current `M7` route-review screen should be redesigned so it
looks and feels like a credible early professional EDA review surface rather
than a technically functional prototype.

This document is grounded in:
- `specs/M7_FRONTEND_SPEC.md`
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/M7_DECISION_PROPOSALS.md`
- `docs/gui/REFERENCE_STUDY.md`

Primary reference posture:
- **Altium** for professional PCB density, contextual inspection, and panel
  seriousness
- **Blender** for editor-first composition and deliberate region hierarchy
- **OrCAD/PADS/Xpedition** for workflow posture and non-consumer visual
  seriousness

## Reference-Screen Notes

These notes capture only the lessons from the attached comparison screenshots
that materially affect the current `M7` route-review screen redesign.

### Altium Reference: Split Schematic / PCB Workspace

#### What It Gets Right Structurally

- The shell is dense, but the working surfaces still dominate.
- The left project rail is visually quiet and clearly subordinate.
- The central editors feel like real professional surfaces rather than generic
  canvases inside a window.
- The PCB pane uses distinct object classes and high scene density without
  losing the active work target.

#### What It Gets Wrong Or Carries As Baggage

- The top chrome is heavy and legacy-dense.
- Toolbar and icon accumulation creates visible product debt.
- The shell is more operationally crowded than Datum should copy for `M7`.

#### What Datum Should Adopt

- Quiet project/navigation rail behavior.
- Strong editor/viewport dominance.
- Professional panel seriousness without oversized chrome.

#### What Datum Should Adapt

- High-density shell discipline, but with fewer competing toolbar and command
  surfaces.
- Contextual panel behavior, but expressed through Datum's smaller `M7`
  taxonomy.

#### What Datum Should Avoid

- Ribbon/toolbar accumulation.
- Shell clutter that competes with the route-review viewport.

### Altium Reference: Focused PCB View

#### What It Gets Right Structurally

- The board itself is the visual field.
- Copper, substrate, pads, vias, component outlines, and active route all read
  as separate object classes.
- The active route is unmistakable, but still feels integrated with the board.
- The properties panel is dense and calm instead of noisy.

#### What It Gets Wrong Or Carries As Baggage

- The overall shell still carries commercial EDA accumulation.
- Some of the density comes from years of feature layering rather than a
  cleaner interaction grammar.

#### What Datum Should Adopt

- Board-as-field treatment.
- Strong object vocabulary for pads, vias, copper, outline, and bodies.
- Review overlay that feels laid over authored copper rather than floating
  separately.

#### What Datum Should Adapt

- Dense PCB visual hierarchy, but tuned for one narrow route-review workflow.
- Strong panel density, but with fewer control surfaces than Altium exposes.

#### What Datum Should Avoid

- Copying Altium's full shell complexity into the `M7` opening slice.
- Treating commercial visual clutter as a sign of professionalism.

### KiCad Reference: PCB Editor

#### What It Gets Right Structurally

- The board fills the viewport as a real PCB surface.
- Zones, traces, pads, vias, labels, and edge cuts are immediately legible as
  different classes.
- The appearance/layer panel is practical and supports fast visibility
  reasoning.
- The board feels grounded because the substrate, copper, and markings all
  participate in the same scene vocabulary.

#### What It Gets Wrong Or Carries As Baggage

- The shell is functional, but not especially refined.
- The overall product feel is more utilitarian than carefully composed.
- Some density comes from accumulation rather than stronger hierarchy.

#### What Datum Should Adopt

- Practical PCB object readability.
- Useful board-field treatment where substrate and copper are both visible.
- Layer/object clarity that supports review immediately.

#### What Datum Should Adapt

- Visibility/filter practicality without copying KiCad's full editor chrome.
- Dense object rendering, but with a calmer and more deliberate visual system.

#### What Datum Should Avoid

- GUI clutter justified by familiarity.
- Treating KiCad's current shell polish as the product ceiling.

## 1. Critique Of The Current Screen

### What Looks Primitive

- The viewport still reads like a debug canvas with a route line drawn on top,
  not like a board review surface.
- Component bodies and pads are rendered as simple boxes with very limited
  object vocabulary.
- The route overlay is legible, but still visually closer to a line test than
  to a professional route-review annotation.
- Panel text is readable but still feels like rasterized status text rather
  than a deliberate inspection interface.
- The bottom dock strip reads like a placeholder tab bar, not part of a
  cohesive workstation shell.

### What Looks Unprofessional

- The board field is too flat and uniform; it lacks a strong sense of material
  baseline, layer/copper context, and intentional scale.
- The viewport grid is currently generic and application-wide rather than
  clearly subordinate to the board itself.
- Component bodies, pads, and board outline do not yet form a convincing PCB
  composition.
- The screen still lacks a strong visual center of gravity around the review
  target; the route line is visible, but the scene around it does not carry
  enough supporting information.
- Panel cards are structurally correct, but their typography and rhythm remain
  closer to a tool spike than to a polished professional inspection surface.

### What Is Visually Unclear

- The distinction between board substrate, component body, and copper object is
  still weaker than it should be.
- The current proposal overlay is visible, but the authored/proposed/
  diagnostic hierarchy is not yet strong enough to feel authoritative.
- The current screen does not communicate enough about why the user should
  trust the route as a review object instead of reading it as a random line.
- The right sidebar has the correct information categories, but not yet enough
  hierarchy to separate summary, metadata, and the active actionable review row
  at a glance.

### What Can Stay

- The locked three-column shell and bottom dock strip.
- The fixed panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`.
- The viewport-centered layout.
- The read-only review workflow and first-action default target.
- The current general dark-neutral direction.
- The current compact density target.

## 2. Concrete Redesign Direction For This Exact Screen

### 2.1 Layout Refinements

Keep the existing shell structure, but refine proportions and internal spacing:
- preserve the current left / center / right composition
- visually reduce the dominance of panel chrome
- make the board field feel inset and intentionally framed inside the viewport
- treat the viewport header band as a compact status strip, not a text area

Screen-level target:
- viewport should own the eye immediately
- sidebars should read as disciplined instruments, not as equal-weight columns
- the bottom dock strip should remain quiet until opened

### 2.2 Panel Composition

#### Left Sidebar

`Project`
- one compact identity block
- one highlighted current-net line
- remove any feeling of “label dump”

`Filters`
- present as toggles/status rows with stronger alignment
- use one subdued divider between toggle block and active review status

#### Right Sidebar

`Inspector`
- top: current selection identity
- middle: authored/proposal metadata rows
- bottom: one compact state block for layer/segment status

`Review`
- top: review-source and action-count summary
- middle: one strong divider
- bottom: active action row list with clear selected-state treatment

The right sidebar should visually echo professional PCB tools:
- summary first
- current target second
- evidence/action list third

### 2.3 Typography Hierarchy

The current bitmap text is acceptable for a spike, but the hierarchy needs to
be explicit:
- panel titles: small uppercase, low-contrast, role-defining
- primary values: compact but clearly brighter
- metadata labels: muted and aligned
- active route/proposal text: warm accent, only where action state is primary
- viewport labels: sparse, not decorative

For this screen specifically:
- component reference labels should be small and quiet
- the active route identifier in the viewport header should remain visible but
  not dominate the scene
- review row titles should be stronger than review row subtitles by a visible
  step, not a subtle one

### 2.4 Scene Styling

The current viewport should be redesigned around **board field**, **authored
objects**, and **review overlay** as distinct visual strata.

#### Board Field

- the board interior should read as the main field, slightly separated from the
  outer viewport background
- the board outline should be crisp and structurally authoritative
- the grid should be subtle enough to support scale without competing with
  geometry
- the grid should feel like a drafting aid, not a wallpaper texture

#### Component Bodies

- bodies should read as placement envelopes, not bright rectangles
- body fill should stay subdued
- body outline should be more important than body fill
- selected or related components may brighten at the border, not by flooding
  the whole body

#### Pads

- pads should read as copper contact objects
- they need stronger material distinction from component bodies
- pad centers/rings should imply electrical contact rather than “small boxes”
- anchor pads should be recognizably related to the active proposal

#### Tracks And Vias

- when present, tracks should read as authored copper with layer-aware color
  and thin material-like line weight
- vias should read as plated transitions, not generic points
- authored tracks/vias should remain visually quieter than the active proposal

### 2.5 Authored / Proposed / Diagnostic Appearance

#### Authored

- cool, low-saturation copper/object vocabulary
- visible enough to establish trustworthy board context
- restrained emphasis for selected or related authored objects

#### Proposed

- warm overlay color family
- center highlight or dual-stroke treatment
- slightly thicker than authored tracks
- proposal must feel laid **over** the board, not merged into it

#### Diagnostic / Evidence

- thinner and less dominant than the proposal
- separate hue family from proposal
- clearly supportive rather than primary

The key screen-level rule:
- authored explains “what exists”
- proposed explains “what is being reviewed”
- diagnostic explains “why it matters”

### 2.6 Color Relationships

Use one restrained neutral base plus two role accents:
- neutral slate/charcoal shell and board field
- cool authored/copper family
- warm proposal family
- limited evidence/diagnostic accent

Do not use bright white as the main authored object color except for tiny
high-contrast details.

For this screen:
- component bodies should be dark slate
- pads should be pale copper/off-metal rather than flat white
- board outline should be brighter than the grid, but dimmer than active
  proposal focus
- selected review row and selected proposal should share one accent family

### 2.7 Spacing And Density

Keep professional density, but make rhythm clearer:
- panel interior spacing should use a tighter and more consistent vertical step
- cards should feel deliberate, not merely boxed
- viewport header should be compact enough not to steal height from the board
- row height in `Review` should remain compact but should not collapse title
  and subtitle into one texture

### 2.8 Grid / Outline / Board Field Treatment

For this exact screen:
- the outer viewport region should be darker than the board field
- the board field should be one restrained tone with subtle grid support
- the board outline should be a thin but crisp perimeter
- the board field should not feel like an empty container; it should feel like
  a PCB review plane

### 2.9 Bottom Dock Treatment

The bottom dock should remain present but quiet:
- inactive-tab strip only
- darker and slightly lower contrast than sidebars
- no visual competition with the main route review
- tabs should read as available work lanes, not as buttons demanding attention

## 3. Prioritized Redesign Plan

### Must Change First

1. **Viewport field hierarchy**
   - board field, outline, grid, and route overlay must feel intentional and
     distinct
2. **PCB object vocabulary**
   - component bodies, pads, and labels must stop reading as placeholder boxes
3. **Proposal overlay language**
   - the route must read as a deliberate review overlay with stronger authored
     contrast
4. **Right-sidebar hierarchy**
   - `Inspector` and `Review` need clearer summary/action grouping

Reference impact:
- the added reference screenshots reinforce this order rather than changing it
- Altium and KiCad both show that board-field treatment and object vocabulary
  must be fixed before panel polish can carry the screen

### Can Wait

- final font stack
- more nuanced theme tuning
- richer layer coloring once more complex board scenes are available
- animated highlight transitions
- expanded terminal/AI dock treatment beyond basic visual cohesion

### Should Not Change In This Slice

- shell structure
- panel taxonomy
- review workflow
- read-only posture
- bottom dock role
- selection semantics

## 4. Visual Acceptance Criteria

### Credible Enough For Early M7 Review

The screen is credible enough when:
- the viewport reads immediately as a board review surface, not a generic
  canvas
- component bodies, pads, and the route overlay look like intentional object
  classes
- the proposal path is visually primary without drowning out authored context
- the right sidebar clearly communicates current selection, review source, and
  active action
- the overall screen feels calm, technical, and dense rather than improvised

### Still Acceptable As Spike-Level Roughness

- bitmap text remains in use temporarily
- geometry is still simple and mostly rectilinear
- object styling is limited to solid fills, borders, and simple line weights
- terminal and AI lanes remain visually minimal

### Still Unacceptable

- viewport still reads like a debug scene
- proposal path still feels like a test line rather than review overlay
- authored objects still read as anonymous boxes without category distinction
- panels still feel like raw diagnostic text dumps
- shell chrome competes with the board for attention

## 5. Bounded Implementation Sequence

### Pass 1: Board Field And Overlay Hierarchy

- refine viewport background/board-field separation
- tune grid, outline, and board inset treatment
- finalize proposal path dual-stroke and authored/proposed contrast

### Pass 2: PCB Object Vocabulary

- redesign component body treatment
- redesign pad treatment
- add quiet component reference labels
- ensure selected/related authored objects use border/emphasis rather than fill
  flooding

### Pass 3: Panel Hierarchy Pass

- tighten panel typography rhythm
- refine card headers/dividers
- align metadata rows more deliberately
- improve selected review-row emphasis

### Pass 4: Color And Density Tuning

- calibrate neutral base tones
- reduce accidental low-contrast gray-on-gray areas
- tune authored, proposed, and diagnostic accents to work together across the
  full screen

### Pass 5: Bottom Dock And Shell Cohesion

- make the bottom dock quieter and more integrated
- ensure sidebars and viewport feel like one workstation, not three separate
  blocks

### Pass 6: Final Acceptance Pass

- review against this document and the screenshot-derived criteria
- fix only the remaining screen-level problems that prevent credibility
- stop before broadening scope or adding workflow

## Summary Decision

This screen should evolve toward:
- **Altium-like seriousness in density and inspection**
- **Blender-like editor-first clarity**
- **Datum-owned authored/proposed/diagnostic vocabulary**

It should not evolve toward:
- a brighter generic desktop app
- a decorative “modernized” UI
- a visually busy enterprise EDA clone
- a viewer that looks polished but still fails to communicate trustworthy board
  review context
