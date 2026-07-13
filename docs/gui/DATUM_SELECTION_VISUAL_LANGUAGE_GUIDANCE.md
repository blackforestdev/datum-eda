# Datum Selection Visual-Language Guidance

Status: governed research-derived guidance; owner decisions in progress

Research basis:
`research/selection-visual-language/SELECTION_VISUAL_LANGUAGE_RESEARCH.md`.
Tracked by `dat-s5-selection-visual-contract-zid`. Final visual authority remains
the Rendering Book, controlling prototypes, UVT S5 contract, and final numbered
selection-identity decision.

## Preserve the Datum identity

- Selection uses the established cross-surface `#CE5A92` accent and soft
  internal-glow language.
- Owner-ratified construction: slightly brighten every owned visible
  presentation primitive while preserving its semantic/material hue, then add
  the accent internal glow and crisp object-shaped screen-space cue.
- Do not replace it with a generic bounding-box-only CAD selection style.
- Selecting an owned compound identity highlights its complete visible
  presentation coherently; connected or merely related geometry is excluded
  unless the selected scope explicitly includes it.
- Selection is immediate screen-space presentation state only.

## State channels

Treat authored material, related context, proposal, diagnostics, hover,
selection, lock/focus markers, and workspace focus as orthogonal compositing
channels. Selection wins over hover on the same object. Workspace focus remains
the pane frame/header mutation-authority cue and must not be inferred from object
highlight intensity.

Owner-ratified projection rule: an actual selection has identical full treatment
in every resolving active/inactive workspace. Only pane chrome/tool availability
changes with GUI mutation authority. Related cross-probe geometry is a distinct
subordinate role, not a dimmed form of selection.

Owner-ratified related rule: preserve the related object's exact authored
appearance. Do not brighten/recolor/glow it. An explicit relationship context
may mildly dim unrelated geometry, leaving related objects at baseline; the
Inspector carries durable relationship explanation.

Owner-ratified compound-focus rule: do not render a second persistent canvas
state. All selected members look equally selected; Inspector identifies optional
focus, and reference-requiring commands own a temporary explicit marker.

Owner-ratified locked rule: persistent slight neutral desaturation/greying plus
full selection when selected, no transform handles, locked cursor, and a small
anchor padlock for selected/hovered objects. The glyph cannot ship until declared
in `icon_set.json`, aligned with and added to the Rendering Study contact sheet,
and visually reviewed; dense compounds rely on greying plus Inspector count.

Owner-ratified collision rule: authored base → proposal ghost/dual stroke →
selection cue → topmost diagnostic marker. Selecting proposal/diagnostic adds
selection without erasing proposal status or semantic severity; shape/pattern
keeps every channel distinct beyond hue.

Owner-ratified small-object rule: standalone text treats actual glyphs without a
persistent box; junction/via retain semantic/material core and add a ring/glow;
no-connect treats its full X; Symbol Editor pin treats complete pin presentation.
Low-zoom/high-contrast fallbacks use minimum crisp screen-space cues.

## Architecture correction

S5 must migrate PCB selected recoloring/weight out of retained authored buffers
and into one typed post-world overlay path shared by PCB and schematic. Selection
changes must preserve retained-world bytes, prepared/static buffers, CAM/export,
and unrelated-pane uploads. Existing retained-selection tests must be replaced
with overlay-retention evidence rather than treated as final behavior.

## Object and scope honesty

- Parent selection follows workspace authority: PCB pad → footprint and
  schematic pin → symbol outside their dedicated definition editors.
- Sections, connected runs, global nets, compounds, persistent groups, and
  related cross-probe projections are distinct semantic scopes and must not be
  conflated by one visual role.
- Owner-ratified Global Net scope is one semantic selection subject whose full
  visible electrical projection receives selection treatment across schematic
  and PCB. Connected parent symbol/footprint bodies remain related, not selected.
- Owner-ratified Bus scope is independent: section → connected bus run → semantic
  hierarchical bus. Spine, owned name, and entries project together; scalar
  member nets remain separate selection subjects.
- Hidden selected geometry remains hidden. Locked selected geometry retains
  selection plus a non-color constraint cue and no transform handles.
- Derived zone fills, projected graphics, dimensions, groups, locks, and other
  states without typed scene authority cannot be claimed visually complete.

## Accessibility and performance

- Selection, related, lock, diagnostic, proposal, and focus states require
  geometry/pattern/shape channels in addition to hue.
- High contrast uses crisp outlines/patterns; reduced motion never removes state
  meaning and selection does not pulse.
- Selection chrome is zoom invariant and pane clipped and does not alter hit
  geometry.
- Visible overlay emission must be deterministic and bounded; large selections
  cannot silently truncate.

## Decision posture

The research report lists the remaining owner decisions. Amend the Rendering
Book and prototypes section-by-section as those decisions are ratified, then add
machine/HUMAN conformance evidence before closing the bead or authorizing S5.
