# Frontend Reference Study

> **Status**: Non-normative design-research document.
> This document captures product and interaction lessons from professional
> graphical tools that matter to Datum's long-term GUI direction. It is not a
> toolkit choice document and it does not define implementation contracts.

## Purpose

Record the practical product/UI lessons that should shape Datum's frontend
foundation before the project commits to a GUI architecture or an `M7`
implementation slice.

This study treats the following tools differently:
- **Altium Designer** as the strongest public benchmark for professional PCB
  interaction expectations
- **OrCAD X** and **PADS/Xpedition-family** as references for enterprise PCB
  workflow seriousness and current commercial UX direction
- **Blender** as a reference for custom graphical application architecture and
  interaction grammar
- **KiCad** as a useful contrast point, not the product ceiling

## Overall Conclusions

The best professional interfaces are not defined by visual style. They are
defined by:
- high information density with controlled hierarchy
- persistent contextual inspection instead of modal dialog churn
- strong selection, filtering, and highlighting models
- keyboard-first canvas work for repeated expert actions
- multiple coordinated views over one underlying model
- review workflows that make intent, evidence, and consequences legible

The recurring failure mode in legacy EDA is also consistent:
- accumulated panel sprawl
- weak hierarchy between authored state, proposed state, and diagnostics
- too many modes and dialogs
- inconsistent interaction grammar between sub-editors
- enterprise or desktop-legacy baggage mistaken for professional depth

Datum should adopt the seriousness and density of the best tools while
rejecting their accumulated interaction debt.

## Altium Designer

### What It Gets Right

- The **Properties** model is the strongest public example of always-available,
  contextual inspection/editing in professional PCB tooling.
- The interface assumes expert use: dense panels, fast toggles, and
  keyboard-centric flow are normal rather than hidden.
- Filtering, highlighting, panel-driven browsing, and view control are treated
  as first-class workflow tools.
- 2D and 3D are part of one editor family, not separate products.
- The product exposes a serious professional vocabulary around rules,
  constraints, variants, outputs, and review.

### What It Gets Wrong

- The UI carries heavy legacy accumulation. Professional breadth often arrives
  as panel and settings sprawl rather than a clean interaction grammar.
- Many flows still reveal desktop-era baggage: modal dialogs, obscure toggles,
  duplicated controls, and weak discoverability.
- High density is sometimes achieved by compression rather than hierarchy.

### Adopt / Adapt / Avoid

`Adopt`
- persistent contextual inspection
- semantic filtering and highlighting
- expert-speed keyboard flow
- integrated 2D/3D product direction

`Adapt`
- dense panel systems, but with fewer overlapping control surfaces
- professional breadth, but with stronger hierarchy and less settings drift

`Avoid`
- sprawling preferences and panel duplication
- modal-over-panel routine workflows
- legacy clutter presented as professional power

### Explicit Lessons For Datum

- Datum should treat the inspector/properties surface as foundational.
- Datum should make semantic review state obvious: authored, proposed,
  selected, dimmed, diagnostic.
- Datum should inherit professional PCB vocabulary without inheriting Altium's
  accumulated UI debt.

## OrCAD X

### What It Gets Right

- Public Cadence material points toward a more modern workflow shell around
  still-serious professional PCB tasks.
- The OrCAD family reinforces that professional EDA is not just drawing; it is
  managing constraints, completion state, review, and cross-domain continuity.
- The product direction suggests continued emphasis on connected design flows
  rather than isolated editors.

### What It Gets Wrong

- Publicly available evidence is less concrete than Altium's documentation, so
  the UI lessons are clearer at the workflow level than at the interaction
  detail level.
- Cadence tooling historically carries enterprise-suite complexity and can feel
  like several subsystems sharing branding rather than one interaction model.

### Adopt / Adapt / Avoid

`Adopt`
- seriousness about full design workflows
- strong notion of continuity across capture, layout, and review

`Adapt`
- connected workflow emphasis, but with a more coherent interaction grammar

`Avoid`
- product seams surfacing directly in the UI
- enterprise complexity leaking into the first visible shell

### Explicit Lessons For Datum

- Datum should design the shell around complete review workflows, not just a
  viewport plus controls.
- Future schematic, board, and review experiences should feel like parts of one
  product, not separate applications.

## PADS / Xpedition Family

### What It Gets Right

- The Siemens family reinforces the need for high-density workflows in serious
  layout environments.
- The product line reflects the value of a growth path from smaller-team PCB
  use toward more advanced planning, constraints, and enterprise review.
- Public material consistently frames PCB work as system coordination, not only
  object manipulation.

### What It Gets Wrong

- Public UI evidence is more marketing-oriented than documentation-oriented, so
  detailed interaction lessons are weaker than for Altium or Blender.
- Enterprise product families tend to surface capability through product and
  mode expansion, which risks visible UI fragmentation.

### Adopt / Adapt / Avoid

`Adopt`
- seriousness about planning, review, and design-scale workflow
- expectation that PCB tools must support sustained expert density

`Adapt`
- structured review and planning patterns without adopting enterprise bloat

`Avoid`
- product-family seams and over-specialized shell complexity

### Explicit Lessons For Datum

- Datum should design early GUI decisions so they can scale to more advanced
  review and planning tasks later.
- The opening slice should stay narrow, but the shell should not dead-end.

## Blender

### What It Gets Right

- Blender proves the value of a **custom graphical application model** rather
  than forcing a domain-specific editor into generic desktop assumptions.
- The Areas/Editors/Workspaces model is a strong example of task-shaped UI
  organization.
- Blender has a coherent operator/tool mindset: the active editor, current
  selection, and current mode define what input means.
- Viewport interaction is treated as a first-class product concern, not as a
  secondary canvas inside a generic app shell.

### What It Gets Wrong

- Flexibility can become overload. Blender often exposes too many pathways,
  settings, and conventions for the same outcome.
- Discoverability debt is real, especially for users who do not already think
  in Blender's interaction grammar.

### Adopt / Adapt / Avoid

`Adopt`
- editor-first shell design
- workspace/task framing
- coherent operator and mode model
- serious investment in viewport interaction quality

`Adapt`
- custom-application architecture, but with stricter discipline and fewer ways
  to do the same thing

`Avoid`
- overexposing internal complexity
- configurability that outruns product clarity

### Explicit Lessons For Datum

- Datum should design the GUI as a serious graphical application, not a generic
  desktop app with a drawing widget.
- The shell should support task-specific workspaces, but Datum should begin
  with tighter guardrails than Blender.
- Input behavior should be deliberate and contextual, especially once edit
  tools exist.

## KiCad (Contrast Reference)

### What It Gets Right

- KiCad provides a useful baseline for what open-source users expect in board
  and schematic tooling.
- Appearance and visibility controls are practical, and the 3D viewer provides
  evidence that linked alternate views are valuable even when they are not yet
  fully integrated.
- KiCad demonstrates the value of approachable defaults and a broad ecosystem.

### What It Gets Wrong

- The overall product still reflects GUI-first architecture, with automation
  and headless use treated as secondary.
- Interaction consistency across editors remains uneven.
- Dialog-heavy and tool-fragmented workflows remain more common than they
  should be in a modern professional interface.
- The 3D path is useful, but it still reads more as an adjacent view than as a
  first-class part of one interaction model.

### Adopt / Adapt / Avoid

`Adopt`
- useful visibility controls and pragmatic defaults
- practical lessons from real open-source user expectations

`Adapt`
- familiarity where it reduces unnecessary user friction, but not at the cost
  of stronger long-term architecture

`Avoid`
- GUI-first product assumptions
- dialog-heavy routine workflows
- inconsistent editor feel
- treating 3D as auxiliary rather than coordinated

### Explicit Lessons For Datum

- Datum should remain aware of KiCad expectations while aiming clearly above
  KiCad's current product coherence.
- Datum should not inherit KiCad's architecture or interaction compromises just
  because they are familiar.

## Cross-Reference Summary For Datum

Patterns worth inheriting:
- always-visible contextual inspection
- strong selection/filter/highlight vocabulary
- keyboard-centric expert workflow
- coordinated multiple views over one design state
- high information density with readable hierarchy
- review as a first-class workflow, not an afterthought

Patterns Datum should explicitly reject:
- modal-dialog dependence for routine work
- panel proliferation without stable taxonomy
- inconsistent interaction grammar between canvases
- low-density generic desktop-app aesthetics
- shell designs that bury the viewport
- a 3D path that feels disconnected from 2D state

## Directional Conclusions For Frontend Work

1. Datum should design for **professional review/editing coherence**, not
   casual desktop simplicity.
2. Datum should treat **selection, filtering, and contextual inspection** as
   core product infrastructure.
3. Datum should begin with a narrow opening slice, but the shell must already
   reflect a long-term serious graphical application model.
4. Datum should build an interface that is visually disciplined enough to avoid
   legacy EDA clutter while remaining dense enough for expert work.
5. Datum should make authored state, proposed state, and diagnostic state
   unmistakable.
