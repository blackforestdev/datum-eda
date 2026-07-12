# Datum Application Status Bar Guidance

Status: governed research-derived guidance; owner decision open

Research basis:
`research/application-status-bar/APPLICATION_STATUS_BAR_RESEARCH.md`.
The governing GUI specifications and Active Frontier remain authoritative. This
guidance does not ratify retention, removal, or field contents.

## Naming and identity

**Application Status Bar** is the canonical name for the thin full-window strip
at the absolute bottom of Datum, beneath the terminal dock. It is global
application chrome—not a pane header, terminal tab strip, Inspector, canvas
overlay, stdout lane, or future action console.

## Research-derived placement law

`global/stable → optional application chrome; pane-persistent → pane header;
gesture/hover/snap → canvas-local; rich/detail/history → panel`.

Consequential active-work feedback MUST NOT depend on the Application Status Bar
alone. Selection projection, snap engagement, gesture mode, invalid drop, lock
refusal, route preview, and cross-probe correspondence require feedback at or
near the affected pane/object, with durable detail in Inspector/panels as
appropriate.

## Candidate retained role

If retained, the bar should be calm and low-churn. Candidate global fields are:

- model dirty/revision state;
- aggregate check severity/completion;
- background commit/check/output-job progress;
- connectivity/recompute state; and
- a details affordance.

Focused-pane identity, tool, grid, layer/sheet, coordinates, selection summary,
and active-gesture hints are not automatically global merely because prototypes
currently place them there. They must pass proximity, ambiguity, duplication,
and narrow-window review. Any pane-derived global field must identify its owner.

## Safety and accessibility

- Critical failure/progress remains discoverable if the bar is hidden or
  removed.
- Important state uses text/icon plus severity, not color alone.
- Transient changes are not the sole acknowledgement of consequential actions.
- Status events carry programmatic scope/severity semantics without moving
  focus; high-frequency cursor motion is not announced continuously.
- Full detail remains in the authoritative Inspector, diagnostics, activity, or
  job surface.

## Decision posture

Research favors either removal or a minimal independently hideable global bar,
not a dense CAD-style toggle dashboard. Owner review must choose the bar's
retention and contents before S5 status reporting is specified or implemented.
