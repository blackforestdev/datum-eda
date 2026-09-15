# Publish Space Visual Study Brief

> **Status:** Governed DOC-C04 visual-research brief; no implementation
> authorization.
>
> **Visual owner:** Claude owns the HTML source of truth under
> `docs/gui/prototypes/*.html`. Codex owns research/specification/governance and
> must not edit or stage Claude's prototype.
>
> **Target artifact:** `docs/gui/prototypes/publish-space-study.html`.

## Purpose

Extend Datum's ratified one-window recursive-pane shell with the Publish Space
experience required by DOC-C03. The study must make the semantic contract
visible and test the remaining choreography choices. It must not invent a
global Design/Publish mode, replace the Project Navigator, detach content into
native windows, or recreate an AutoCAD command console.

## Read before drawing

Review these sources together; later runtime drift never overrides them:

- `research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md`, especially
  the 20-question ledger and DOC-C01..DOC-C03;
- `docs/gui/prototypes/board-editor.html`;
- `docs/gui/prototypes/schematic-editor.html`;
- `docs/gui/prototypes/workspace-panes.html`;
- `docs/gui/prototypes/title-block-study.html` and `title-block-sizes.html`;
- `docs/gui/DATUM_GUI_DESIGN_SPEC.md` and `DATUM_GUI_CONFORMANCE_SPEC.md`;
- Product Mechanics 019, 020, 021, 023, and 033;
- `research/documentation-system/TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md` and
  `PRODUCT_REVISION_ENGINE_RESEARCH.md`.

## Locked elements to carry forward

1. One cohesive native Datum window only.
2. Persistent left Project Navigator above the focused-pane context panel,
   central recursive pane tree, right Inspector, bottom Terminal dock, and the
   output-only pane-local Datum Console.
3. Board, Schematic, specialist Design editors, Publish content, and auxiliary
   views may coexist in arbitrary internal splits. Design/Publish are content
   classifications, never a global pane-evicting mode.
4. Pane focus owns tools, menus, Inspector, context panel, and Console placement.
5. Navigator single-click selects; double-click or Enter opens beside the
   focused Pane. Navigator scrolling and local right-click Search behavior stay
   exactly as recorded in question 18.
6. `Sheet` means a Publish page only. The schematic Design surface is continuous;
   no `Schematic · Sheet` label may appear in the new study.
7. Workspace Pane and Publish Viewport must read as visibly different objects.
8. Viewport frame move, frame resize, and content scale are separate controls.
9. Publish cannot edit projected Design geometry. The explicit local action is
   `Open Source in Design`; double-click remains unassigned.
10. `New Viewport`, `Use Existing`, `Edit Saved Viewport`, and `Make Unique`
    must express the ViewportDefinition/ViewportInstance link model. Rename is
    label-only.
11. Publish dimensions are associative reference documentation only. Design
    dimensions alone may drive geometry.
12. Existing title-block direction and token/color/typography laws remain
    controlling.
13. Released configurations are immutable. Detailed allocation, approval, and
    release controls are now governed by Product Mechanics 034 and
    `specs/PRODUCT_REVISION_ENGINE_SPEC.md`; this Publish visual study does not
    own or redraw them.

## Required visual frames

The HTML may use one interactive study or clearly labeled candidate panels, but
must include all of these states:

### A. Project discovery and opening

- Project Navigator hierarchy for a scalable project containing multiple Design
  contexts, Publish assets, several SheetSets, shared Sheets, and saved named
  ViewportDefinitions without forcing product/assembly hierarchy.
- A local context menu on the relevant Project/Publish node for creating a blank
  Sheet, creating from template, and local Search.
- Single-click selection versus double-click/Enter adjacent opening.
- Long-tree scrolling with the right-edge scrollbar and a local incremental
  search state showing matches in collapsed descendants.

### B. Coexisting Design and Publish

- Schematic Design Pane beside a Publish Sheet Pane in the inherited shell.
- PCB + Schematic + Publish nested tiling at one useful size.
- Focus moving between Design and Publish with pane-local tools, context panel,
  Inspector, and Console ownership following focus.
- Pane Zoom and Full-Screen Stage applied to Publish without changing source
  authority or destroying the tile tree.

### C. Sheet creation and composition

- A truly blank Sheet after media/orientation choice.
- A template-seeded Sheet carrying the accepted title block and editable starter
  content.
- A heterogeneous Sheet containing schematic, board, photograph/image, note,
  and projected table examples without implying the first implementation ships
  all of them.
- Sheet/media boundary, printable/standards guidance, overflow finding, grid/
  snap, align/distribute, z-order, and locked item cues.

### D. Viewport manipulation and reuse

- Selected ViewportInstance with handles that make frame resize unmistakable and
  do not suggest scale-by-drag.
- Local menu/Inspector scale presets, custom exact scale, fit-to-frame, crop,
  visibility, render style, and lock.
- Per-Viewport scale label.
- `New Viewport` versus `Use Existing` insertion flow.
- Linked definition state, instance override state, `Edit Saved Viewport` impact
  preview, and `Make Unique` with next numbered name plus inline rename.
- Broken/stale associative reference and missing-source states with actionable,
  object-local feedback.

### E. Annotation and source navigation

- Static Sheet note and an associative Publish dimension attached through a
  Viewport to stable Design geometry.
- Local Viewport menu containing `Open Source in Design`.
- Two explicitly labeled choreography candidates for C05 review:
  1. open the source in a new adjacent Pane while preserving Publish context
     (**recommended**, consistent with specialist-editor entry and Project
     Navigator opening);
  2. retarget the focused Pane with a deterministic Back/return affordance.
- No edit-through state and no double-click shortcut.

### F. SheetSets, sharing, and forks

- At least two packages, such as Fabrication and Customer, referencing one
  unchanged shared Sheet.
- `Fork Sheet` producing a distinct Sheet while inherited ViewportInstances
  remain linked to their definitions.
- `Make Unique` applied only to the Viewport that diverges.
- A conspicuous statement that hidden layers/content are not verified redaction;
  security-grade redaction remains a separate contract.

### G. Working versus released context

- Quiet working/default state.
- Immutable released-baseline viewing state with exact baseline identity.
- A later Design or Publish change producing an affected/stale successor-work
  indication without rewriting the released Sheet.
- The study may show status, impact, and navigation only. It must not invent
  revision allocation, approval roles, signatures, or release commands before
  REV-C01..REV-C08.

### H. Responsive and accessibility matrix

- Wide, ordinary, and narrow application widths.
- Terminal closed and open.
- Single, tiled, Zoom, and Stage arrangements.
- Keyboard focus visibility, icon+text redundancy for consequential state,
  non-color-only link/override/release cues, and readable overflow behavior.

## Owner-review questions the study must isolate

The study must make these choices independently reviewable rather than bundling
them into one approval:

1. Project Navigator hierarchy and Publish creation placement.
2. Blank-versus-template Sheet entry choreography.
3. Adjacent-versus-retarget behavior for `Open Source in Design`.
4. Viewport linked/override/unique visual language.
5. SheetSet sharing/fork presentation.
6. Working/released/affected status presentation, limited to the approved
   Revision Engine boundary.
7. Overall visual disposition across normal and responsive states.

## Acceptance evidence

Before DOC-C04 can complete:

- the HTML must render without network dependencies;
- `scripts/shot_html.sh docs/gui/prototypes/publish-space-study.html ...` must
  produce reviewed screenshots at representative wide, ordinary, and narrow
  sizes;
- every required frame above must be traceable by a stable HTML anchor or visible
  panel label;
- the prototype must be added to exactly one evidence route and cross-reviewed
  against its governed consumers;
- Claude must provide a concise consistency report naming carried-forward
  elements, deliberate candidates, exclusions, and any unresolved conflict;
- the owner must review the candidates in DOC-C05 before DOC-C06 ratifies
  mechanism.
