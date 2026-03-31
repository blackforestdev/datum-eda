# Workspace Model

> **Status**: Supporting frontend foundation document.
> This document defines Datum's application shell, panel philosophy, and lane
> coexistence model.

## Purpose

Describe how Datum should organize work inside one graphical application before
individual review or editing features are specified.

## Shell Model

Datum should use a **single primary application shell** that can host multiple
coordinated work lanes without fragmenting into separate products.

The shell should provide:
- one main window
- one active workspace layout at a time
- one or more central graphical viewports
- a stable set of persistent side or bottom panels
- an integrated command/terminal lane
- an integrated AI assistant lane

The shell should feel like a serious graphical/CAD application, not a
document-form app.

For the opening `M7` slice, this shell is now locked to:
- a viewport-centered three-column layout
- a left sidebar for `Project` and `Filters`
- a right sidebar for `Inspector` and `Review`
- a bottom dock strip for `Terminal` and `Assistant`

## First-Class Lanes

### 1. Graphical Canvas / Review Lane

The graphical lane is the main spatial workspace:
- board review and later editing
- schematic review and later editing
- future 3D review/editing

This lane owns:
- viewports
- spatial selection visualization
- geometric highlighting
- camera/navigation state

### 2. Integrated Command / Terminal Lane

The terminal lane provides:
- direct CLI-style workflows
- deterministic command invocation
- logs and reports
- artifact inspection and workflow output

This lane is part of the product, not an external escape hatch.

It must not:
- create unofficial state outside engine actions
- bypass normal design authority
- become a shadow UI model for operations that the rest of the app cannot
  understand

### 3. Integrated AI Assistant Lane

The AI lane provides:
- contextual explanation
- review assistance
- command/action discovery
- guided analysis tied to the current design context

It must not:
- become an alternate design database
- mutate design state through hidden or private channels
- hold authoritative interpretation separate from engine outputs

## Relationship Between Lanes

The three lanes should coexist as one product with shared context rules:
- they may all observe current project, selection, and review target
- they may all present information about the same deterministic engine state
- only explicit engine actions may change design state

Expected flow:
- canvas establishes spatial context
- panels refine inspection and filtering
- terminal exposes deterministic direct workflows and logs
- AI provides contextual assistance, explanation, and action guidance

No lane is allowed to redefine object identity or fabricate a competing review
state.

## Panel Philosophy

Panels should be:
- persistent
- few in number
- semantically stable
- discoverable by role rather than by feature accretion
- usable for long sessions without modal churn

Datum should prefer extending a stable panel family over introducing many
specialized panels.

Expected panel families:
- `Project / Structure`
- `Inspector / Properties`
- `Filters / Visibility`
- `Review / Evidence`
- `Output / Diagnostics`

The terminal and AI lanes are not replacements for these panels. They are
adjacent first-class lanes with distinct responsibilities.

For `M7` v1, the fixed panel taxonomy is:
- `Project`
- `Filters`
- `Inspector`
- `Review`

## Document And View Organization

Datum should distinguish:
- **document**: a board, schematic, later 3D assembly/model, or related design
  artifact
- **view**: a specific representation of that document

Examples:
- one board document with one main board view
- one schematic document with one active sheet view
- one future board document with both 2D and 3D correlated views

The product should support multiple views over one authoritative document state
without multiplying document truth.

## Review Workflow Posture

Review should feel:
- deliberate
- context-rich
- low-friction
- evidence-backed
- safe from accidental mutation

A review workspace should make the following obvious:
- what is currently under review
- what authored state provides the baseline
- what proposed or diagnostic state is being compared against that baseline
- what evidence supports the current review target
- which lane or panel can explain or inspect the current state further

## Workspace Flexibility

Datum should support task-shaped workspaces, but with guardrails.

Early product direction:
- fixed or tightly constrained layouts for initial milestones
- no arbitrary panel sprawl
- no early commitment to fully user-programmable workspace complexity

Long-term direction:
- saved workspaces by task
- coordinated board, schematic, review, terminal, and AI layouts

## M7 Implication

The first `M7` workspace should use this shell model in a narrow way:
- one central board review viewport in a locked three-column shell
- one bounded persistent panel set: `Project`, `Filters`, `Inspector`,
  `Review`
- one integrated bottom-docked terminal lane for direct deterministic
  workflows/logs
- one integrated bottom-docked AI lane for explanation and review assistance

`M7` should prove the coexistence model without broadening into a general GUI
platform.

Post-spike validation items remain open:
- exact sidebar widths and heights beyond initial implementation defaults
- exact bottom-dock open/collapsed behavior details
