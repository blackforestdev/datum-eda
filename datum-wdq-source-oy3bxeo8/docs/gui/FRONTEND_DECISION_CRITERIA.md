# Frontend Decision Criteria

> **Status**: Non-normative decision rubric.
> This document defines how Datum evaluates frontend architecture choices. It
> exists to prevent premature or shallow framework selection and to keep the
> GUI aligned with long-term product goals.

## Purpose

Datum's frontend architecture must be chosen as infrastructure for a serious
EDA product, not as a convenience layer for a small viewer.

This rubric should be cited by future frontend foundation, architecture, and
`M7` planning documents.

## Evaluation Rule

Candidates should be judged against the full product direction:
- professional board and schematic canvases
- future 3D review/editing
- CAD-style interaction
- native Linux desktop expectations
- engine-first ownership boundaries

No option should be accepted merely because it is easy to start.

## Core Criteria

### 1. Custom 2D Canvas Suitability

The frontend must support a first-class custom board/schematic canvas with:
- precise control over rendering layers and draw ordering
- reliable picking and highlight identity
- overlays for authored, proposed, and diagnostic states
- dense technical rendering without fighting toolkit assumptions

Questions:
- Can the stack support a serious custom canvas rather than a generic scene?
- Can Datum control draw order, hit testing, visibility, and overlay behavior?
- Will the stack remain workable once board and schematic views both exist?

### 2. Future 3D Path

The frontend must leave a credible path to integrated 3D review and later
editing.

Questions:
- Can 2D and 3D share identity, selection, and visibility state?
- Does the rendering model leave room for a correlated 3D view?
- Does the stack force a separate 3D subsystem that will later fracture the
  product?

### 3. CAD Interaction Model Fit

Datum needs a CAD-style interaction model, not a form-app interaction model.

Questions:
- Can the stack support low-latency pan/zoom/orbit and precise pointer work?
- Can it support persistent tool modes and contextual input handling cleanly?
- Does it let the viewport remain first-class within the shell?
- Will keyboard-centric expert workflows feel natural or bolted on?

### 4. Linux Desktop Quality

Datum is Linux-first. Desktop quality matters.

Questions:
- Does the stack behave well on modern Linux desktops?
- Is Wayland/X11 support credible?
- Are font rendering, DPI handling, input methods, and window behavior strong?
- Can Datum deliver a professional-feeling Linux application without extensive
  platform workarounds?

### 5. Engine / Frontend Boundary

The engine remains the source of truth. The frontend is a consumer.

Questions:
- Does the stack make it easy to keep design truth outside the GUI?
- Can contracts flow cleanly from engine/daemon into a rendering/view-model
  layer?
- Does the architecture tempt the team into frontend-owned semantics or
  parallel state?

### 6. Performance and Rendering Control

The frontend must not cheapen or bottleneck a high-performance engine.

Questions:
- How much control does Datum have over rendering behavior?
- Can the frontend handle dense geometry, overlays, and frequent interaction
  without obvious ceiling issues?
- Will the shell or rendering layer introduce avoidable extra passes,
  translation layers, or framework-imposed constraints?

### 7. UI / UX Ceiling

Datum is aiming above hobbyist defaults.

Questions:
- Can this architecture support a serious long-session professional interface?
- Does it allow a disciplined panel model, strong hierarchy, and high-density
  workflow design?
- Will the architecture age well as the product grows from review into editing?

### 8. Maintainability and Lock-In Risk

Frontend choices should not create hidden strategic debt.

Questions:
- How much product-specific work will be trapped inside framework-specific
  patterns?
- How hard would it be to evolve or replace parts of the stack later?
- Does the stack impose language, deployment, licensing, or ecosystem
  constraints that meaningfully narrow future choices?

## Secondary Criteria

These matter, but they are subordinate to the core criteria above:
- packaging and deployment complexity
- team familiarity
- documentation quality
- ecosystem maturity for supporting pieces such as text, docking, dialogs, and
  asset loading
- test harness practicality

They should break ties, not determine the decision on their own.

## Disqualifying Failure Modes

An option should be treated as strategically weak if it:
- treats the custom canvas as a second-class citizen
- makes 3D an awkward afterthought
- encourages GUI-owned semantic state
- locks Datum into generic desktop-app interaction patterns
- imposes obvious performance or rendering ceilings
- produces a Linux application that feels compromised
- is attractive only because it accelerates a throwaway first viewer

## Decision Guidance

When comparing candidates, write down:
- where each option is strong
- where each option is risky
- which risks are temporary implementation costs
- which risks are structural product limitations

Prefer the option with higher long-term product ceiling when the cost is
primarily execution effort. Reject the option whose convenience comes from
structural limitations that Datum would later have to unwind.

## Expected Output Of Future Architecture Studies

Any later frontend architecture recommendation should explicitly score or
discuss each candidate against this rubric:
- custom 2D canvas suitability
- future 3D path
- CAD interaction model fit
- Linux desktop quality
- engine/frontend boundary
- performance/rendering control
- UI/UX ceiling
- maintainability and lock-in risk
