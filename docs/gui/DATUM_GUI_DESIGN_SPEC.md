# Datum GUI Design Spec — The "How"

> **Status**: Governed active design spec.
> **Authority**: Realizes the *visual/interaction execution* ("how") of the GUI
> that `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` (the "what") and decision 019 leave
> open, on the systems ratified by decisions **014** (layout geometry) and **015**
> (Design Book tokens). Design authored by the project owner; this doc records the
> ratified design decisions and points at the controlling visual prototype.
> **Reference prototype**: `docs/gui/prototypes/board-editor.html` (design pass 3 — split PCB|Schematic view).
> **Scope**: The concrete visual composition, density, and interaction craft of
> the Datum desktop GUI — starting with the board editor.

## Why this exists

The product spec, menu bindings, and even the Design Book specify *what* the GUI
contains and *why* — a menu tree, a component checklist, a token palette. None of
them is a **design**: the composition, visual hierarchy, density rhythm, and
interaction choreography that decide whether the interface is clean, attractive,
and intuitive. That craft is the "how," and in UI it is often more decisive than
the what/why. This spec captures it, developed together with a live HTML
prototype (the vehicle) so the two directly convey the intended interface.

## Design thesis (the pinned direction)

Datum's GUI is a **professional instrument in the pro-audio idiom** — the visual
language of Bitwig / Ableton (and plugins in that lineage: Vital, iZotope RX,
FabFilter). This is not an aesthetic accident: pro-audio apps render everything
custom (no OS-native widgets) — the exact architecture Datum chose with wgpu — so
they are Datum's real peer class. The governing rules:

1. **Flat, dark, restrained chrome. Zero decoration.**
2. **Color is meaning, never decoration.** Chrome is the Design Book's monochrome
   ramp; saturated color appears *only* on canvas content (copper/nets/layers)
   and as the single `#CE5A92` magenta selection accent. (Ableton's gray chrome +
   colored clips; Bitwig's semantic modulation color.)
3. **The canvas is the protagonist**, and controls act *on* it (Altium's Properties
   Panel + direct manipulation — the anti-dialog-per-object thesis).
4. **Dense, not cramped** — ruthless hierarchy and uniform rhythm, generous where
   it counts.

## Reference prototype

`docs/gui/prototypes/board-editor.html` is the **controlling visual reference**
for the board editor. It is built entirely from the Design Book chrome/content
tokens. This document narrates the decisions it embodies; when they disagree, the
prototype wins for visuals and this doc wins for rationale/rules — reconcile in
the same change.

## Locked decisions — board editor v1

- **Shell composition** (left→right, top→bottom): menu bar · left column
  (Project tree over Layers) · **central board/schematic pane field
  (protagonist)** with per-pane header tools · right column (Inspector) · bottom
  dock (Terminal only — multi-tab, 32px collapsed) · Application Status Bar. Approximate
  widths: left ~228px, right ~300px.
- **Color-application law**: chrome uses only `bg/surface.01–03/border/text`
  tokens; the only chrome color allowed onto the canvas is `--acc` (#CE5A92) as
  selection. Copper/nets/pads/vias/ratsnest use the content tokens.
- **Selection → Inspector binding**: single selection is the primary model;
  selecting an object (magenta outline + glow on canvas) drives the Inspector and
  cross-highlights panel ↔ canvas.
- **Inspector = Properties Panel**: context-sensitive to the selection;
  inline-editable rows (no dialogs); collapsible grouped sections (Identity /
  Placement / Nets / Checks); key-value rows with mono tabular values for
  coordinates.
- **Layers panel**: Ableton colored-track-row idiom — swatch + name + visibility,
  active layer accented.
- **Type & density**: dense but legible — ~25px rows, chrome text ~13.5–14.5px,
  uppercase section labels ~11.5px with letter-spacing, `tabular-nums` on all
  coordinates/IDs. **Production UI font = IBM Plex Sans** (the prototype uses a
  system stand-in; the sandbox can't embed the binary).
- **Icon scale/style**: line/stroke glyphs — pane tools ~16–20px, tree/panel
  ~15px; every tool carries a single-key accelerator; tooltips mandatory on
  icon-only controls; never icon-only for critical actions.
- **Status bar**: cursor X/Y (mm) · active tool · selection · grid · active layer
  · DRC count · model revision.

## Workspace & Mode Model

Datum is a **single unified viewport**, not a set of separate editor windows. The
user opens a document from the Project pane and the viewport enters that
document's **mode**; the mode carries its own toolset and menus.

- **Documents / modes**: schematic, PCB/board, footprint editor, symbol editor
  (library-object modes), plus read views (rules/check report, manufacturing).
  Selecting `project → Schematic` / `Board` / a library `Footprint` / `Symbol`
  switches the mode.
- **Mode-specific tools (the SolidWorks pattern)**: each focused pane owns a
  header tool strip and the active-editor-gated menus (schematic: wire / symbol /
  label / bus / junction; PCB: route / via / zone / place; footprint & symbol:
  their drawing tools) — exactly the gating `DATUM_GUI_MENU_BINDINGS.md` already
  assumes.
- **Tiling ("tmux for EDA")**: the viewport splits into panes; each pane is a
  **(document, view) pair**. This one abstraction covers both "2D + 3D of the same
  board" and "schematic here, PCB there, footprint in a PIP." Panes tile or float
  (picture-in-picture).
- **Context follows focus**: the focused pane owns the pane-header tools and menus, and
  the Inspector / Layers / Filters panels bind to the focused pane's document and
  selection.
- **Cross-probe over one model**: selection is engine-level, so selecting an
  object in one pane highlights its counterparts in every other pane showing
  related objects (schematic symbol ↔ board footprint ↔ net). This falls out of
  tiled panes over one shared `DesignModel` — Altium's cross-probe / Horizon's
  message bus, for free.
**Ratified as decision 021** (`docs/decisions/PRODUCT_MECHANICS_021_WORKSPACE_PANE_TILING.md`):
the workspace is a **recursive binary tile tree**, **tile-first**, View-menu-managed,
with two bounded overlay modes on top — **Zoom/maximize** a pane (the "get the others
out of the way" need) and deliberate **Float/detach** (PIP over the others). Panes hold
`(document, view)` pairs over the one `DesignModel`; the layout is **consumer/workspace
state, never journaled**. Distinct from decision-020 **paper-space viewports** (a
projection window onto a sheet) — a **workspace pane** is the interactive editor tiling.
The former "PIP vs tile-only" sub-decision is resolved there (tile foundation + zoom +
float escape hatch).

Reference prototypes: `docs/gui/prototypes/board-editor.html` (pass 3, PCB|Schematic
split with cross-probe) and `docs/gui/prototypes/workspace-panes.html` (the recursive
tiling + View-menu model).

## Command Surfaces — console, terminal, AI (three, not one)

Three distinct input surfaces got muddled in the M7 spike. They are **separate by
design**, and correcting this is a decision-level fix (clarifies 005 terminal,
006 assistant, 019 shell).

1. **Command Console (Eagle / AutoCAD).** A single-line command input in the
   **lower-left of the viewport**. Type a command (`mo`, `route`, `net`…), hover
   the target, press Enter → it acts on the object under the cursor. **Every verb
   is a console command**, and the console language *is* the scripting language
   (Eagle's ULP → Datum's Python/verbs). It is the **typed twin of the marking
   menu** — identical verbs, different muscle memory (type vs. flick) — a
   manual-first power surface. It **drives the editor, not the OS**; it is not a
   shell. *(This is the original intent that the spec distorted into "a terminal.")*
2. **Native Terminal (VS Code-style).** A real PTY system terminal
   (Alacritty/Ghostty/Konsole-grade), **fully integrated, not a bolt-on**, with
   **multiple tabs/sessions**. It runs any OS task and is where **code agents run**
   — Claude Code, Codex, or a **local model (vLLM / Ollama)** — with the full Datum
   surface exposed. Because it's native, a Linux user drives agents through their
   **own subscription or local models — no forced API credits**. Multiple tabs let
   **different agents work different aspects in parallel** (schematic / PCB /
   footprint). Decision 005, clarified.
3. **AI collaboration = agent-in-terminal + inline ghost overlay.** There is **no
   separate "Assistant/Agent tab."** The agent runs in the native terminal; its
   *proposals* render as the inline ghost overlay on the canvas (Tab / Esc). This
   corrects the M7 distortion (a bolted-on assistant lane — same family as the
   vacated 013 misfire). **There is no Output tab either** — CAM/export results are **files in the
   project working directory** (produced via the terminal), viewed in the gerber/drill
   viewer or a paperspace viewport (decision 020), not a supervision lane (the same
   meta-supervision pattern as the vacated 013 misfire).

**Why it got muddled:** the spec collapsed the *console* (drive the editor by
typing verbs) into the *terminal* (run OS commands), then split "AI" into its own
lane — a surface that was neither a real command line nor a real shell, plus a
redundant agent tab. Two surfaces pretending to be one; one split into two. Net
result on the shell: the console is viewport-anchored (lower-left); the bottom dock
is **the terminal alone (multi-tab)** — no Assistant tab, no Output tab. CAM/export
output is files in the working directory (produced via the terminal), viewed in the
gerber/drill viewer or a paperspace viewport (decision 020).

**The unifying law — five doorways, one vocabulary.** menu bar (discovery) ·
marking menu (gesture) · **command console (typed)** · scripting (verbs in a file)
· AI (intent). All five drive the **same verb registry**, so **every program action
must be a verb** — a *complete* set (capability stays a parameter of a small verb
set, but nothing is unreachable). A missing verb = an action the AI can't do, the
console can't type, and a script can't call. **Verb-registry completeness is
first-order**; the menu_model `not_built` entries are the visible gap.

**Future:** SPICE / behavioral-model integration (deferred) benefits directly from
the multi-tab terminal + parallel-agent model.

## Context Menu System (right-click) — the speed surface

Designed 2026-07-06 from a two-part research pass: EDA right-click systems
(Altium / KiCad / Horizon / Eagle) for *content*, and marking/radial menus
(Autodesk Maya, Sketchbook Pro, Blender, Fluxbox; Kurtenbach & Buxton HCI work)
for *form*. **Full research record with citations:**
`research/gui-context-menus/CONTEXT_MENU_RESEARCH.md`.

**Paradigm** — **select-first, object-verb** (KiCad / Horizon), NOT Eagle's modal
verb-first. The menu is **filtered to only the actions valid for the current
selection** (Horizon's anti-bloat discipline). Every item **emits a typed
`Operation` through the one commit path** — so the context menu is also the
natural home for AI **"Propose…"** variants and undo/provenance affordances (kept
in one submenu to preserve the short-menu discipline).

**Form — hybrid marking menu + linear overflow:**
- Right-click opens a per-object-type **marking menu** (radial). *Same gesture,
  two modes*: **hold ~280 ms → labeled wheel draws** (novice); **flick immediately
  in a direction → command fires, nothing draws** (expert, eyes stay on the
  board). The novice drag *is* the expert flick — continuous skill transfer.
  Right-click/right-drag is exclusively owned by this menu on every 2D editor;
  it MUST NOT fall through to viewport pan. Pan uses `Space`+primary-button drag,
  while middle-button drag is reserved for 3D-view rotation.
- **≤8 items per wheel; the 4 cardinal (N/E/S/W) carry the most-frequent verbs**
  (only cardinals are reliably reproducible blind); diagonals carry secondary
  actions. **Never place a destructive/irreversible action on a diagonal — keep
  Delete cardinal.**
- **Nesting ≤2 levels**; any sub-wheel is **4-wide** (cardinal-only).
- **Compound marks — the exponential speedup.** Selecting *through* a submenu is
  one continuous stroke: flick toward the parent verb, then toward the child —
  e.g. up-then-right traces an inverted "L". Experts draw the whole shape blind,
  and it becomes a single muscle-memory gesture (Kurtenbach & Buxton hierarchic
  marking menus; Zhao & Balakrishnan simple-vs-compound marks). To support it, **a
  submenu spawns as a new radial at the location of the parent wedge you flicked
  toward** (offset outward in that direction — the Sketchbook Pro model), *not*
  re-centered. The parent wheel stays put (faded/blurred), so the compound stroke
  flows outward and the gesture path stays constant per object type → the whole
  drill-down becomes one memorized flick-shape rather than a slow menu descent.
  Pure compound marks degrade when *deep* (Zhao & Balakrishnan, UIST 2004), so any
  level past the depth-2 cap would use **simple marks** (a separate stroke per
  level) instead of one long zig-zag.
- One wedge is **"More…" → a conventional linear, scannable list** (the
  magenta-group-delineated menu already mocked) for the long tail — parameterized
  actions (net classes, track widths, value pickers). Support **tear-off** so a
  chosen list floats as a palette during repetitive work.
- **Every item carries a keyboard accelerator** (manual-first hedge; a third
  eyes-free path).
- **Frozen geometry**: slot positions are constant per object type; unavailable
  actions grey **in place**, never reflow; never auto-sort by recency.
  **User-editable** (hoist your own verbs onto cardinals).

**Adoption is progressive — the gesture is never required.** The floor stays as
easy as any CAD package: a beginner *holds* to get a normal labeled menu, or uses
the menu bar / command palette / keyboard — nothing forces the flick. The flick
is an opt-in speed *reward* that pays off with repetition, so the ceiling is
faster than any competitor. This deliberately optimizes for intermediate-to-
advanced users (Datum's target segment) without a beginner penalty — the concern
"it might take getting used to" is real but opt-in, never a wall.

**Linear ordering** (the overflow and any non-radial fallback), top→bottom with
magenta group separators: object-special (frequent-first) → transform block →
grouped submenus (`Select ▸` / `Net ▸` / `Convert ▸`) → clipboard/lifecycle →
lock/visibility toggles → **Properties last (`E`)**.

**Empty-canvas menu is different** — space-level only: Paste, Select All,
`Grid ▸`, `View ▸`, `Place ▸`, global toggles (router/ratsnest/highlight). No
object verbs.

**Multi-select** — menu = the **intersection** of actions valid for all selected
objects; singular labels become "…Selected…"; Properties opens a multi-edit.

**Per-object cardinal proposals** (owner-tunable, user-editable):

| Object | N | E | S | W | diagonals |
|---|---|---|---|---|---|
| Component | Move | Rotate | Delete | Properties | Flip · Lock · Find in schematic · More… |
| Track | Route/Continue | Change Width | Delete | Change Layer | Drag · Highlight Net · Add Via · More… |
| Via | Change Size | Assign Net | Delete | Properties | Move · Highlight Net · More… |
| Net | Highlight | Hide Ratsnest | Assign Netclass | Assign Color | Select All · More… |
| Zone | Fill | Unfill | Edit Border | Properties | Repour · Add Cutout · Duplicate to Layer · More… |
| Empty | Place ▸ | Paste | Grid ▸ | View ▸ | — |

**Full per-object menu content** (every object type in PCB + schematic —
cardinal-4 / secondary / overflow / sub-wheels, each leaf mapped to an engine op,
with destructive / tool-start / cross-probe flags): **`docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md`**.

**Tool depth is the moat.** A menu item is a shallow doorway; the capability lives
in the operation's *parameter space*, reached three ways that emit one op —
marking-menu presets / tool inspector (full schema) / AI intent (natural language)
/ CLI. The flagship deep tools (Align as a discipline, parametric array/pattern
placement, thermal-via arrays, impedance-aware diff-pair routing) and this tri-modal
contract are specified in **`docs/gui/DATUM_GUI_PARAMETRIC_TOOLING.md`**.

**Still to prove:** the marking-menu interaction in the prototype (delayed popup,
expert flick drawing nothing), and per-object cardinal tuning with the owner.
Reference visual: `docs/gui/prototypes/context-menu-marking-menu.html` (contiguous
semi-transparent wedges over a blurred board, auto-scaling, nested radial submenus
with parent-layer fade/blur). The AI dock-vs-overlay comparison is illustrated in
`docs/gui/prototypes/open-decisions.html`.

## Open design decisions (resolve before broader build-out)

1. **AI surface — RESOLVED (2026-07-06): both, by role.** Proposed *changes*
   render as an **inline ghost overlay on the canvas** (dimmed geometry in place,
   `Tab` accept / `Esc` dismiss) — never a chat panel stealing the canvas. The
   **conversational agent lives in the terminal/assistant lane** — the "code-agent
   for EDA" model: converse, or `Esc` and redirect, exactly like driving Claude
   Code — reconciling decisions 004/006 (assistant surface) + 005 (terminal). The
   two are complementary, not alternatives: overlay = proposal presentation on the
   canvas; terminal lane = the agent conversation. Both ship. (Note the AI need not
   be one large model — e.g. a small GPU-resident routing model handles routing;
   the surfaces above are model-agnostic.)
2. **Fonts**: embed IBM Plex Sans; choose the data-mono face (Design Book leaves
   it open).
3. **Context menus — RESOLVED (2026-07-06)** into the "Context Menu System"
   section above (hybrid marking menu + linear overflow, select-first object-verb,
   per-object cardinal verbs). Remaining: validate the marking-menu interaction in
   the prototype (delayed popup, expert flick, mark trail) and tune the per-object
   cardinal assignments with the owner.
4. **Datum visual identity**: reference frames, origins, fiducials, measurement
   styling — no Datum-specific identity yet (generic dark theme). Owner call.

## Working method

Spec ⇄ prototype, co-developed with tight back-and-forth until the pair directly
conveys the interface. The prototype is the visual source of truth; this spec is
the ratified rationale + rules. Each approved pass updates both in the same
change. Next surfaces after the board editor: schematic editor, library browser.

## Design Language & Consistency

Datum's UI follows the Ive/Jobs ethos: **the software should not ship with a
manual.** Good design is invisible — the interface is obvious, and its craft goes
unnoticed because nothing about it demands attention. The aspiration: let a novice
wield an expert's reach — *the design carries the competence, so the user doesn't
have to.* If a workflow needs explaining, the design failed, not the user.

**One action, one identity, everywhere.** A verb has exactly one icon, one label,
and one behavior wherever it appears — menu bar, marking menu, command palette,
toolbar. The context-menu glyph *is* the toolbar glyph *is* the File-menu glyph.
This is not policed; it falls out of the verb-registry projection (every surface
renders the same verb, so its icon and behavior are shared *by construction* — see
Modularity). Consistency the architecture makes hard to break.

**Entry points are calibrated, not redundant.** Multiple ways to reach an action
are a feature *only if* each is the natural reach in a distinct moment: **menu bar**
= discovery/learning; **marking menu** = in-place speed, eyes on the work; **command
palette** = search-by-name; **keyboard/gesture** = expert muscle memory; **toolbar**
= the few always-present tools. All fire the *same* verb, glyph, and result.
**Paths that behave differently, or exist for no distinct moment, are a defect** —
that is precisely how tools get confusing (too many roads to one place). Default to
*subtraction*: when unsure whether a path earns its place, remove it, don't add it.

**Cross-domain interaction parity.** The same verb-shape works the same way across
every editor mode. Drawing a **wire** in the schematic and a **trace** on the PCB
are one interaction — same tool slot, same gesture, same icon family, same
place→click→finish rhythm; only the domain object differs. place / move / delete /
properties behave identically in schematic, PCB, footprint, and symbol modes.
**Learn a move once; it works everywhere.** Enforced by the marking-menu invariants
(Delete always Cardinal-S, `▸ change` always the same slot) and by every mode
projecting the same verb families.

## Modularity & Extensibility

The design is deliberately **add/remove, not rewrite**: every addition or change to
the GUI is a *data entry* against one of a few extension points, never a bespoke new
surface. This is what keeps the current effortless add/remove workflow true as the
product grows — and it is a controlling constraint, not a nice-to-have.

**Extension points** (add a capability *once*; every surface projects it):
- **Verb registry (decision 017)** — the single capability catalog. Add a tool =
  add a verb; the menu bar, marking menus, command palette, CLI, and MCP all
  project from it. No surface hardcodes a tool list.
- **Menu model (data, not code)** — the menu bar and per-object marking menus are
  the data manifest `docs/gui/menu_model.json` (each entry bound to a real
  `datum.*` verb, a `gui_local` view action, or `not_built` = worklist), realizing
  the content in `DATUM_GUI_CONTEXT_MENU_CONTENT.md`. Adding / removing /
  reassigning a menu item is **editing one row**, and `scripts/check_menu_model.py`
  (in `run_drift_gates.sh`) guarantees every `verb` reference exists in the registry
  catalog and enforces the marking-menu invariants (cardinal N/E/S/W, destructive
  never on a diagonal). No vaporware menu items; the GUI reads the manifest. Icons
  are the same discipline: every menu icon is declared in `docs/gui/icon_set.json`
  (Tabler MIT base + custom EDA glyphs), validated by the same gate — the
  `to_author` entries are the glyph worklist, exactly parallel to `not_built` verbs.
  The menu model is **authored in a spreadsheet** — `docs/gui/menu_model.csv` — and
  built to the JSON via `scripts/menu_model_csv.py build` (round-trips both ways).
  Edit the grid, rebuild, gate validates. That is the human authoring surface;
  nesting (the 3-layer align) fits via the `wheel` + `slot` columns.
- **Design Book tokens (decision 015)** — visual change = token edit; nothing
  hardcodes a color or size. Restyle by changing a token; everything follows.
- **Typed `Operation` + one `commit()`** — add an operation variant + its builder;
  every surface (GUI / CLI / MCP) gains it through the same path. No private writers.
- **Self-contained object-type and mode definitions** — each object's wheel and each
  editor mode (schematic / PCB / footprint / symbol) is an independent definition;
  a new object type or editor mode is *added*, not woven through the shell.

**Rules that preserve it:**
- Add a tool → add a verb + a menu_model entry (+ its typed op if it mutates).
- Add / reorder a menu item → edit a menu_model row.
- Add an object type → add its per-object menu definition.
- Add an editor mode → add a document type + toolset; shell composition untouched.
- **If you find yourself editing a *surface* to add a capability, the extension
  point is missing — add it there first.**

The co-development loop (spec ⇄ prototype, one element per pass) is the same
principle at the process level: each pass adds or refines a single, self-contained
element. That is why it feels effortless — keep it that way.

## Governance

Tracked in `specs/spec_governance_manifest.json` (docs/gui enforced glob) and in
the `specs/PROGRESS.md` Active Frontier (step 1's first concrete deliverable).
The prototype under `docs/gui/prototypes/` is referenced here so it is not an
orphan. Update this spec + the prototype together; never let one drift.
