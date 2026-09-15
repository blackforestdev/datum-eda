# Context Menu & Marking Menu Research — Datum GUI

Research record backing the **Context Menu System** section of
`docs/gui/DATUM_GUI_DESIGN_SPEC.md`. Two parts: (A) what belongs in EDA
right-click menus (content), and (B) the marking/radial-menu interaction model
and its HCI evidence (form). Compiled 2026-07-06.

---

## Part A — EDA right-click context menus (content)

Four reference tools sit on two paradigms:

- **Altium, KiCad** — *object-verb, select-first-ish*: click/hover an object, the
  menu is a large contextual command list. Rich, sometimes bloated.
- **Horizon EDA** — *radically non-modal, select-first*: the right-click menu is
  the **primary tool launcher**, filtered to *only* the tools valid for the
  selection. Deliberately short. The **spacebar menu** is the same tool set as a
  searchable/hotkey-annotated palette (one tool registry).
- **Eagle / Fusion** — *modal, verb-first*: pick a command then click objects.
  Right-click is a secondary, **per-object user-customizable** convenience.

**Recommended paradigm for Datum:** select-first, object-verb (KiCad + Horizon),
NOT Eagle modal. Borrow **Horizon's discipline** (filter to valid actions, keep it
short) + **KiCad's consistent generic block + predictable ordering**. Pair with a
Horizon-style command palette over the same action registry (context menu = palette
pre-filtered by selection). Every context command = emit a typed `Operation`.

**Ordering** (top→bottom, separators between groups): object-special (frequent-
first) → transform block (Move/Rotate/Mirror/Align) → grouped submenus
(`Select ▸`/`Net ▸`/`Convert ▸`) → clipboard/lifecycle → lock/visibility →
**Properties last (`E`)**. **Nesting ≤2** (only Altium goes deeper and pays in bloat).

**Empty-canvas menu is different** — space-level only (Paste, Select All, `Grid ▸`,
`View ▸`, `Place ▸`, global toggles). No object verbs.

**Multi-select** — menu = **intersection** of valid actions; singular labels →
"…Selected…"; Properties → multi-edit.

**Per-object action groups (distilled):**

| Object | Special (frequent-first) | Submenus | Always |
|---|---|---|---|
| Component | Edit/Update/Change Footprint, Fanout, Explode, Lock, Cross-probe | Select ▸ | Transform, Delete, Properties |
| Track/wire | Break/Slice, Drag, Fillet, Length-tune, Change Width, Assign Netclass | Net ▸ | Delete, Properties |
| Via | Change Size/Type, Assign Net | Net ▸ | Delete, Properties |
| Net | Highlight, Hide/Show ratsnest, Assign Netclass, Assign Color | — | Properties |
| Pad | Edit, Copy/Push Pad Properties, Assign Net | Net ▸ | Properties |
| Zone/pour | Fill/Unfill, Repour, Modify Border, Add Cutout, Pour-order | Zones ▸ | Delete, Properties |
| Empty | Paste, Select All, Place ▸ | Grid ▸, View ▸ | global toggles |

Notable near-context-menu-exclusive verbs: Break/Slice Track, Assign Netclass from
canvas, Fill Zone, cross-probe (Select on Schematic/PCB), Unfold from Bus.

---

## Part B — Marking / radial menus (form + HCI evidence)

A **marking menu** (Kurtenbach & Buxton, University of Toronto, CHI '91–'94) is a
radial menu whose selection **gesture is identical whether or not the menu is
visible**.

- **Two modes, one gesture ("mark ahead").** Press and *wait* (~250–300 ms) → the
  labeled radial appears (novice). Press and *immediately flick* toward the item →
  the command fires, menu never draws (expert). The novice drag *is* the expert
  flick, so every novice use rehearses the expert motion — **no cliff** between
  beginner and power user (unlike keyboard accelerators, a disjoint skill).
- **Compound marks (hierarchy).** A path through submenus becomes a single
  continuous multi-segment stroke (e.g. up-then-right = an inverted "L"), drawn
  blind by experts. This is the **exponential speedup** for nested selection.

**Hard limits (the study data):**
- **~8 items per level, reliable only at depth ≤2.** Beyond depth 2, 8-wide menus
  become error-prone even for experts; 12-wide is worse.
- **4-item (cardinal N/E/S/W) menus stay <~10% error even 4 levels deep** — the
  robust workhorse. **Off-axis (diagonal) items are more error-prone** than
  cardinals. So an 8-item wheel = 4 strong + 4 weaker slots.
- **Breadth vs. depth trades roughly evenly** (64 cmds as 4×4×4 ≈ 8×8) — pick
  hierarchy shape for *semantic clarity*, not raw performance.
- **Expert mark mode eliminates visual search** → eyes stay on the work.

**Zhao & Balakrishnan, UIST 2004 (*Simple vs. Compound Mark Hierarchic Marking
Menus*):** pure straight **compound marks degrade** in deep/wide hierarchies;
**"simple marks"** (a separate quick stroke per level, menu confirming each) scale
better when deep. **Implication for Datum:** at our depth-2 cap, the compound
(spawn-in-place) stroke is reliable; if a surface ever needs deeper nesting, switch
that level to simple marks rather than one long zig-zag.

**Spawn-in-place (Sketchbook Pro).** A submenu spawns as a new radial **at the
location of the wedge you flicked toward** (offset outward), not re-centered — so
the compound stroke flows outward and the gesture path stays constant → the whole
drill-down becomes one memorized flick-shape.

**Discoverability vs. speed** is reconciled by the **delayed popup** (Maya) — same
gesture, novice sees labels, expert flicks blind. Blender adds **keyboard
accelerators on each pie item** (a third eyes-free path) and mandates **frozen slot
geometry** (never reflow, never re-sort by recency) so muscle memory holds.
Sketchbook keeps a persistent on-canvas radial (the "Lagoon") as an ambient
reminder. All support **user-editable** menu contents so experts hoist their own
verbs onto cardinals.

**Reference-tool spectrum:** Maya (hierarchic marking menus, the canonical model)
→ Sketchbook Pro (radial marking menus under stylus/touch, spawn-in-place) →
Blender (pie menus: frozen positions + key accelerators, keep pies small) →
Fluxbox/Openbox (linear, deeply nestable, tear-off root menus — the coverage-first
counterpoint, and the model for our "More… → linear overflow").

---

## Part C — Synthesis adopted into Datum

Design decisions (see `docs/gui/DATUM_GUI_DESIGN_SPEC.md` → "Context Menu System"):
hybrid **marking menu + linear overflow**; select-first object-verb; per-object
marking menu (≤8 items, 4 cardinal, Delete never on a diagonal, ~280 ms delayed
popup, expert flick, frozen + user-editable geometry, keyboard accelerators);
**compound-mark spawn-in-place submenus** at ≤2 depth with 4-wide cardinal
sub-wheels; a **"More…"** wedge into a scannable linear overflow (magenta group
delineation, tear-off) for the long tail; every item emits a typed `Operation`
(AI "Propose…" variants live here). Progressive adoption — the flick is never
required; hold/menu-bar/palette/hotkeys keep the floor beginner-easy.

Prototype: `docs/gui/prototypes/context-menu-marking-menu.html`.

---

## Sources

- Buxton — Pie Menus overview: https://www.billbuxton.com/PieMenus.html
- Kurtenbach & Buxton — *The Limits of Expert Performance Using Hierarchic Marking
  Menus* (CHI '94): https://www.billbuxton.com/MMExpert.html
- Kurtenbach & Buxton — *User Learning and Performance with Marking Menus* (CHI '94):
  https://www.billbuxton.com/MMUserLearn.html
- Kurtenbach — *The Design and Evaluation of Marking Menus* (PhD thesis, U. Toronto /
  Autodesk Research):
  https://www.research.autodesk.com/app/uploads/2023/03/the-design-and-evaluation.pdf_recHpUp1v9dc1n2CJ.pdf
- Kurtenbach, Sellen & Buxton — *Articulatory and Cognitive Aspects of Marking
  Menus* (1993).
- Zhao & Balakrishnan — *Simple vs. Compound Mark Hierarchical Marking Menus*
  (UIST 2004): https://www.dgp.toronto.edu/~ravin/papers/uist2004_simplemm.pdf
- Autodesk Maya — Marking Menus (help); Sketchbook — Lagoon/marking menus; Blender —
  pie menu design (frozen positions, accelerators); Fluxbox menu manual.
- EDA: KiCad 8 PCB/Schematic docs; Altium PCB placement/editing + polygon docs;
  Horizon EDA board/tools/spacebar-menu docs; Autodesk Eagle context-menu
  customization.
