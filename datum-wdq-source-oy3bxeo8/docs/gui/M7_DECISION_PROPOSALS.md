# M7 Decision Proposals

> **Status**: Proposal history/reference for the opening `M7`
> slice.
> This document turns the approved frontend foundation and `M7` spec into
> concrete UI/UX proposals that informed the locked `M7` decisions. The active
> authoritative text now lives in `specs/M7_FRONTEND_SPEC.md` and the governing
> `docs/gui/*.md` foundation documents.

## Purpose

Define the concrete UI decisions that the first `M7` route-proposal review
workspace should follow.

This document is grounded in:
- `specs/M7_FRONTEND_SPEC.md`
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/REFERENCE_STUDY.md`

The strongest professional references for these proposals are:
- **Altium Designer** for contextual inspection, semantic review, and
  professional PCB density
- **Blender** for shell/workspace/editor structure and interaction grammar
- **OrCAD-family** and **PADS/Xpedition-family** for workflow seriousness and
  review posture

## 1. Initial Workspace Layout Proposal

### Reference Pattern

`Adopt`
- Altium: persistent side panels and dense review-oriented shell
- Blender: central editor dominance with supporting side regions

`Adapt`
- enterprise PCB tool seriousness without panel sprawl

`Avoid`
- ribbon-heavy top chrome
- multi-pane clutter before the product proves the first workflow

### Recommended Datum Choice

Use a **viewport-centered three-column shell with one bottom lane strip**:
- center: board review viewport
- left sidebar: `Project` above `Filters`
- right sidebar: `Inspector` above `Review`
- bottom dock strip: tabbed `Terminal` and `Assistant`

Default dimensions:
- left sidebar: `280 px`
- right sidebar: `340 px`
- bottom lane strip collapsed height: `44 px`
- bottom lane strip expanded height: `220 px`
- viewport minimum share: `>= 55%` of window width at default layout

Always visible:
- viewport
- left and right sidebars
- one bottom lane tab strip

Collapsible:
- left sidebar as a whole
- right sidebar as a whole
- bottom lane body, but not the lane tab strip

Terminal lane placement:
- bottom dock, default collapsed, opens into the bottom lane body

AI assistant lane placement:
- shares the same bottom dock as a second tab next to `Terminal`

### Plausible Alternative

Use a **right-heavy shell**:
- viewport center-left
- all panels stacked in a single right dock
- bottom terminal/AI dock unchanged

### Tradeoffs

Recommended layout advantages:
- preserves viewport dominance
- prevents all metadata from collapsing into one overloaded right rail
- keeps route review evidence close to the viewport while preserving filtering
  on the opposite side

Alternative advantages:
- simpler to implement
- closer to some commercial PCB tool layouts

Alternative drawbacks:
- higher risk of right-side density collapse
- weaker separation between filtering, inspection, and review roles

### Human Approval Needed

`yes`

## 2. Panel Taxonomy Proposal

### Reference Pattern

`Adopt`
- Altium: persistent contextual properties and browsing panels
- Blender: clear editor/region role boundaries

`Adapt`
- commercial EDA panel seriousness, but with a smaller fixed taxonomy

`Avoid`
- every feature becoming its own panel
- duplicated ownership between panels

### Recommended Datum Choice

#### `Project`

Owns:
- project identity
- board identity
- current review source summary
- current artifact/live-review provenance

Must not own:
- visibility controls
- detailed object properties
- evidence row browsing

#### `Filters`

Owns:
- layer visibility
- authored/proposed visibility toggles
- dim-unrelated controls
- bounded review visibility presets

Must not own:
- object metadata
- proposal evidence interpretation
- command execution

#### `Inspector`

Owns:
- metadata for the current single selection
- authored object fields
- proposal action fields when selected
- stable IDs and source identity display

Must not own:
- evidence list sequencing
- filtering logic
- mutation controls in `M7`

#### `Review`

Owns:
- proposal summary
- route review status
- contract/profile/source display
- ordered evidence list
- next/previous review item navigation

Must not own:
- general project structure
- low-level layer filtering
- direct geometry editing

Why this is the right M7 minimum:
- it preserves professional role clarity
- it keeps review state distinct from object metadata
- it avoids the commercial-EDA failure mode where one panel tries to do
  browsing, filtering, inspection, and explanation at once

### Plausible Alternative

Merge `Inspector` and `Review` into one right-side `Details` panel.

### Tradeoffs

Recommended advantages:
- cleaner separation between "what is selected" and "why this proposal is under
  review"
- better future growth path into diagnostics and explain surfaces

Alternative advantages:
- simpler first implementation

Alternative drawbacks:
- faster panel overload
- weaker review workflow structure

### Human Approval Needed

`yes`

## 3. Route Review Workflow Proposal

### Reference Pattern

`Adopt`
- Altium: selection-driven inspection and high-density review posture
- Blender: explicit active target and contextual navigation

`Adapt`
- OrCAD/PADS seriousness about workflow continuity and review framing

`Avoid`
- modal wizard-style review flows
- review that depends on hidden state

### Recommended Datum Choice

Proposed user flow:
1. User opens a board review workspace with one already-loaded proposal review.
2. Initial active review target defaults to the first proposal action.
3. The `Review` panel lists proposal actions and evidence in deterministic
   order.
4. Selecting a row in `Review` focuses the corresponding proposal overlay and
   related authored context in the viewport.
5. Clicking proposal overlay geometry focuses the corresponding `Review` row.
6. Clicking authored geometry shifts the `Inspector` to authored-object detail
   while preserving the current route review context.
7. The user steps through actions/evidence with keyboard or panel controls.
8. The user exits by closing the review, changing selection, collapsing the
   workspace, or leaving the project. No mutation path is presented.

Key inspect motions:
- select evidence row -> zoom or highlight target
- hover authored geometry -> preview relation to active review target
- click proposal path segment -> inspect exact action/evidence linkage
- toggle dim-unrelated -> isolate proposal context

### Plausible Alternative

Default initial selection to the first evidence row rather than the first
proposal action.

### Tradeoffs

Recommended advantages:
- starts from the spatial object under review
- keeps route geometry primary and explanation secondary

Alternative advantages:
- explanation-led review may help early debugging

Alternative drawbacks:
- weaker visual anchoring
- greater risk that review feels like reading a report instead of inspecting a
  board

### Human Approval Needed

`yes`

## 4. Selection And Focus Proposal

### Reference Pattern

`Adopt`
- Blender: clear distinction between hover, active editor focus, and selected
  target
- Altium: selection as the main gateway to detailed context

`Adapt`
- professional CAD precision without Blender's broader mode complexity

`Avoid`
- ambiguous focus transfer
- hidden multi-selection behavior in the opening slice

### Recommended Datum Choice

Exact `M7` v1 grammar:
- `hover`: transient preview only
- `focus`: which lane receives keyboard input
- `selection`: the current single explicit object or review item
- `active review target`: the proposal/evidence context currently driving route
  review highlighting

Canvas focus behavior:
- click viewport background or geometry -> canvas gains focus
- keyboard navigation shortcuts act on review traversal and viewport only when
  canvas has focus

Panel focus behavior:
- clicking `Project`, `Filters`, `Inspector`, or `Review` gives that panel
  keyboard focus
- panel focus never silently changes selection unless the user activates a row
  or control

Bottom lane focus behavior:
- terminal and AI input are explicit focus owners
- opening those lanes does not change current canvas selection

### Plausible Alternative

Tie active review target directly to selection at all times, with no separate
state.

### Tradeoffs

Recommended advantages:
- lets the user inspect authored context without losing the current route
  review target
- keeps the model clearer across canvas, panels, terminal, and AI

Alternative advantages:
- simpler state machine

Alternative drawbacks:
- harder to inspect related authored geometry while preserving review context

### Human Approval Needed

`yes`

## 5. Authored / Proposed / Diagnostic Visual Differentiation Proposal

### Reference Pattern

`Adopt`
- Altium: explicit visual distinction of working states during routing/review
- KiCad contrast lesson: avoid letting alternate state look like baseline

`Adapt`
- professional PCB color density, but with stronger semantic discipline

`Avoid`
- relying on color alone
- letting proposal geometry disappear inside dense copper scenes

### Recommended Datum Choice

#### Authored

Use:
- normal layer/domain color treatment
- solid strokes and fills
- full opacity for in-scope geometry

#### Authored Related

Use:
- same base color family as authored
- brighter outline or stronger edge contrast
- minimal glow or emphasis halo only when necessary

#### Authored Dimmed

Use:
- reduced opacity and reduced contrast
- keep geometry legible, not ghosted out completely

#### Proposed Overlay

Use:
- fixed semantic proposal accent color, independent of copper-layer color
- thicker stroke than authored tracks
- dashed or segmented line pattern for paths not yet authored
- optional subtle translucent corridor fill only where it improves legibility

#### Proposed Focus

Use:
- stronger proposal stroke
- outer outline or highlight ring
- anchor markers emphasized

#### Diagnostic / Evidence

Use:
- bounded markers, badges, or segment emphasis
- severity/evidence accent separate from proposal accent
- no full-screen noise fields

What should use which channel:
- color: semantic state families and severity
- line style: authored vs proposed distinction
- fill: zones and bounded review emphasis only
- opacity: dimming unrelated context
- outline: active selection and proposal focus

### Plausible Alternative

Use solid proposal lines with no dashed differentiation, relying only on color
and thickness.

### Tradeoffs

Recommended advantages:
- remains readable in dense PCB scenes
- survives monochrome or color-fatigue conditions better
- reduces risk that proposed geometry is mistaken for authored geometry

Alternative advantages:
- visually cleaner
- simpler renderer rules

Alternative drawbacks:
- weaker semantic separation
- more likely to fail in dense copper scenes or for color-deficient users

### Human Approval Needed

`yes`

## 6. Density And Hierarchy Proposal

### Reference Pattern

`Adopt`
- Altium/PADS/Xpedition: professional information density
- Blender: viewport dominance and supporting side regions

`Adapt`
- pro density with cleaner prioritization than commercial panel clutter

`Avoid`
- consumer-app emptiness
- enterprise-style everything-visible clutter

### Recommended Datum Choice

Target density for `M7`:
- compact professional density
- one-scan panel sections
- low-padding lists and metadata rows
- no large decorative margins

Visual dominance:
- viewport first
- `Review` panel second
- `Inspector` third
- `Filters` and `Project` quieter
- terminal and AI lanes visually subordinate until opened

Visually quiet:
- project metadata
- inactive panel sections
- unrelated authored geometry under review focus

How to avoid panel clutter:
- only four side panels
- one clear owner per information type
- progressive disclosure inside panels rather than new panels
- default collapsed subsections for rarely used metadata

### Plausible Alternative

Slightly reduce density for `M7` to improve first-pass readability while the
workspace is still new.

### Tradeoffs

Recommended advantages:
- aligned with professional EDA expectations
- avoids a later redesign from "friendly" to "serious"

Alternative advantages:
- easier first-run comprehension

Alternative drawbacks:
- risks drifting into generic app spacing
- wastes valuable board-review space

### Human Approval Needed

`no`

## 7. Color Strategy Proposal

### Reference Pattern

`Adopt`
- professional dark-workspace expectations from PCB/CAD tools
- Altium lesson: semantic state matters more than theme fashion

`Adapt`
- Linux-native theme discipline with fixed semantic colors for review state

`Avoid`
- treating theme choice as pure preference with no semantic rules

### Recommended Datum Choice

Launch `M7` with **dual themes**, but design the opening workspace **dark-first**
for the primary review screenshots and spike tuning.

Rationale:
- dense PCB scenes generally read better on a dark field
- long-session visual fatigue is often lower for board review on dark
  backgrounds
- Linux desktop quality still benefits from a legitimate light theme path

State/severity/highlight roles:
- proposal state color: fixed semantic accent across themes
- diagnostic/evidence severity accents: fixed semantic palette across themes
- authored layer colors: theme-adjusted but semantically stable
- panel/chrome neutrals: theme-dependent

Accessibility rules:
- selection and proposal state must remain distinguishable without hue alone
- contrast ratios must remain adequate in both themes
- avoid over-saturated neon accents on dark backgrounds

### Plausible Alternative

Launch light-first to align more closely with traditional desktop expectations.

### Tradeoffs

Recommended advantages:
- better fit for dense PCB spatial review
- closer to professional CAD/EDA usage patterns

Alternative advantages:
- simpler visual integration with some Linux desktops
- easier text rendering perception for some users

Alternative drawbacks:
- weaker board-scene emphasis
- more difficult to keep dense copper/layer scenes calm

### Human Approval Needed

`yes`

## 8. Typography Proposal

### Reference Pattern

`Adopt`
- dense commercial EDA readability
- Blender lesson: compact UI typography can still be functional if hierarchy is
  disciplined

`Adapt`
- stronger differentiation between labels, values, IDs, and metadata

`Avoid`
- large generic desktop text
- decorative or fashionable type choices

### Recommended Datum Choice

UI text direction:
- one compact sans UI family for shell, panels, and lane chrome
- one monospace family for IDs, coordinates, net names when code-like, paths,
  command output, and exact contract metadata

Hierarchy:
- labels: medium emphasis, smaller size
- values: slightly stronger emphasis
- IDs/UUIDs/paths: monospace, visually quieter unless explicitly selected
- metadata/status: smaller and quieter still

Compactness/readability targets:
- default UI text tuned for dense panel scanning
- line height tight but not cramped
- tabular alignment preferred in metadata-heavy panels

### Plausible Alternative

Use a single sans family everywhere except the terminal lane.

### Tradeoffs

Recommended advantages:
- better distinction between human-oriented and machine-oriented data
- stronger scanning in review-heavy panels

Alternative advantages:
- simpler visual system

Alternative drawbacks:
- less clarity around IDs and exact engine-derived metadata

### Human Approval Needed

`no`

## 9. Terminal Lane Proposal

### Reference Pattern

`Adopt`
- Blender-like idea of multiple serious work regions
- Datum-specific foundation rule that the terminal is a first-class lane

`Adapt`
- CLI seriousness without turning the whole product into a shell wrapper

`Avoid`
- hidden console
- terminal as shadow authoring path

### Recommended Datum Choice

Role in `M7`:
- deterministic workflow and logs surface
- place to view exact command/report output associated with the current review
- bounded place to re-run or inspect machine-native review commands later

Relation to current selection/review context:
- shows current project and current review target context in its header
- may expose copyable exact request arguments for the active review
- does not silently track hover; it only consumes explicit selection or active
  review target context

Default display:
- collapsed by default
- when opened, show recent review-related logs/output first
- clear separation between command invocation history and passive logs

Deferred:
- arbitrary terminal session management
- shell multiplexing
- rich scripting UX
- write workflows

### Plausible Alternative

Keep the terminal lane hidden until a later milestone and rely on external CLI
for `M7`.

### Tradeoffs

Recommended advantages:
- proves the lane model early
- reinforces Datum's machine-native identity inside the GUI

Alternative advantages:
- smaller opening scope

Alternative drawbacks:
- delays an important differentiator
- weakens shell coherence around deterministic workflows

### Human Approval Needed

`yes`

## 10. AI Assistant Lane Proposal

### Reference Pattern

`Adopt`
- professional review support posture from EDA tools, but with Datum's AI-first
  product identity
- Blender lesson: contextual assistance should respect active editor context

`Adapt`
- AI as explain/support layer, not as autonomous hidden operator

`Avoid`
- chat-app-in-a-pane behavior
- ambiguous authority

### Recommended Datum Choice

Role in `M7`:
- explain current route review target
- summarize current evidence
- answer "why is this selected?" and "what does this contract/payload mean?"
- suggest deterministic next inspection steps

Context received:
- current project identity
- current board identity
- current active review target identity
- current single selection identity
- current review payload metadata already visible elsewhere in the workspace

Relation to selection/review context:
- explicit context banner at top of the lane
- context changes only on explicit selection/review-target changes
- AI responses should reference the same IDs and names the user sees in the
  rest of the workspace

What it must not do in `M7`:
- apply changes
- synthesize hidden state
- invent unofficial proposal semantics
- act as a replacement for the `Review` panel

Deferred:
- broad design assistance
- mutation proposals flowing directly into execution
- autonomous multi-step workflows

### Plausible Alternative

Keep the AI lane hidden by default and expose it only as a summoned assistant
surface.

### Tradeoffs

Recommended advantages:
- proves the three-lane product model early
- keeps AI contextual and disciplined rather than ornamental

Alternative advantages:
- lower opening complexity

Alternative drawbacks:
- risks AI becoming an afterthought rather than a first-class lane

### Human Approval Needed

`yes`

## 11. Future 2D / 3D Coexistence Proposal

### Reference Pattern

`Adopt`
- Altium: 2D/3D as one product direction
- Blender: shared editor grammar and context discipline across different view
  types

`Adapt`
- 2D-first opening slice with 3D-safe assumptions

`Avoid`
- 2D assumptions that require a different selection or state model in 3D

### Recommended Datum Choice

`M7` should remain purely 2D in UI, but preserve these assumptions:
- object identity is not 2D-specific
- selection grammar is not 2D-specific
- authored/proposed/diagnostic state families are not 2D-specific
- layer visibility and review-target concepts can later map into 3D view rules

Practical implication:
- avoid naming or structuring review state in a way that assumes "board review"
  always means "flat top-down only"
- keep viewport controls and view-model naming compatible with future secondary
  view types

### Plausible Alternative

Treat `M7` as fully board-2D-specific and defer all coexistence concerns until
3D actually starts.

### Tradeoffs

Recommended advantages:
- preserves future flexibility with little cost now
- avoids repainting selection/review foundations later

Alternative advantages:
- narrower opening language

Alternative drawbacks:
- higher chance of accidental 2D-only assumptions hardening into contracts

### Human Approval Needed

`no`

## Highest-Priority Approval Questions

These decisions most need human review before the architecture spike:
- initial workspace layout
- panel taxonomy
- route review flow starting point
- selection vs active review target separation
- visual differentiation of authored vs proposed vs diagnostic state
- dark-first dual-theme launch strategy
- terminal lane visibility and role in `M7`
- AI lane visibility and role in `M7`

## Stable Enough To Proceed If Unchanged

These proposals are stable enough to carry into the architecture spike unless
the product direction changes:
- viewport-dominant shell
- four-panel `M7` minimum
- single-selection `M7` model
- collapsed bottom dock for terminal and AI lanes
- dense professional hierarchy rather than generic desktop spacing
- two-family typography split between UI text and machine-oriented text
- 2D assumptions that remain compatible with later 3D
