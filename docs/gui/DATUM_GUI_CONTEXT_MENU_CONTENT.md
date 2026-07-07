# Datum GUI — Context-Menu Content (per object type)

> **Status**: Governed active design spec.
> **Authority**: Realizes the per-object *content* of the marking-menu system in
> `docs/gui/DATUM_GUI_DESIGN_SPEC.md` → "Context Menu System". Research basis:
> `research/gui-context-menus/CONTEXT_MENU_RESEARCH.md` (+ the 2026-07-06 per-object
> pass over KiCad / Altium / Horizon).
> **Scope**: What each object type's right-click marking menu contains — the
> Cardinal-4, secondary (diagonals), linear overflow, and nested sub-wheels — with
> a suggested engine-op for each leaf.

## How to read

- Radial = 8 slots: **N NE E SE S SW W NW**. **Cardinal 4** = N/E/S/W (highest
  frequency, only reliable blind targets). **Secondary** = the 4 diagonals.
  **Overflow** = linear "More…" list (long tail). **▸** = nested sub-wheel.
- Flags: **⚠** destructive/irreversible · **⚡** tool-start (arms an interactive
  session, commits one typed op on finish) · **▪** immediate (single typed op) ·
  **⌖** cross-probe (also selects the twin on the other domain) · **★** near-unique
  to the context menu (binds to the object/point under the cursor).
- Every leaf → a typed `Operation` through the one `commit()` (or a proposal); the
  menu is a projection of the verb registry filtered to the selection.

## Cross-cutting rules (apply to every wheel)

1. **Destructive isolation.** Delete and other ⚠ verbs never sit on a diagonal.
   **Cardinal-S is the sanctioned "safe destructive" home** (a deliberate,
   muscle-memory-stable direction); everything else ⚠ goes to Overflow or a
   confirmed submenu leaf. (Exception: No-Connect delete — place↔remove is
   symmetric.)
2. **Reserve slots for the recurring nested-wheel verbs** so the gesture is the
   same across object types: **Change Layer ▸ / Change Width ▸ / Change Size ▸ /
   Assign Net Class ▸ / Align ▸** carry a variable child list; give them consistent
   slots (e.g. the primary "▸ change" always on W or N).
3. **Net-scoped verbs are one op family, many entry objects** (wire / junction /
   label / pin / bus / pad / via / track all resolve to a net): `net.highlight`,
   `net.assign_netclass`, `net.select`, `crossprobe.highlight_net`.
4. **Cross-probe is one op** — `crossprobe.select(object_id, target)` — reached as
   "Select on Schematic" / "Select on PCB" twins.
5. **Empty-canvas wheel = tool/mode launcher** (`tool.*`); **multi-select wheel =
   batch launcher** (`selection.*`) computed as the *intersection* of the
   selection's per-type menus (composition-gated items grey in place, never error).

---

# PCB objects

### Component / Footprint
- **N** Rotate ▪ `pcb.rotate_component` · **E** Properties ▪★ `pcb.edit_component` ·
  **S** Delete ⚠ `pcb.delete_component` · **W** Flip to other side ▪ `pcb.flip_component`
- Secondary: Drag (keep tracks) ⚡ · Move Exactly ▪ · Duplicate ▪ · Select on Schematic ⌖★ · Highlight Net ▪★ · Lock/Unlock ▪
- Overflow: Position Relative To… · Create Array ⚡ · Hide Designator/Value ▪ · Update Footprint from Library ▪ · Edit in Footprint Editor ⌖ · Export to Library ▪ · ⚠ Explode to primitives · ⚠ Reset to Library · ⚠ Swap Component
- Submenus: **Assign Net Class ▸** · **Change Footprint ▸** (this / by value / by lib link / update all) · **Positioning ▸**

### Track / Trace
- **N** Change Layer ▸★ (inserts via) · **E** Drag ⚡★ (push/shove) · **S** Delete ⚠ (segment; whole-track = Backspace) · **W** Change Width ▸★
- Secondary: Route/Re-route ⚡ · Break Track ⚡★ · Select Whole Net ▪★ · Highlight Net ▪★ · Properties ▪ · Tune Length ⚡
- Overflow: Slice Tracks ⚡★ · Fillet · Chamfer · Convert to Arc · Track→Zone · Cleanup · Lock ▪ · ⚠ Delete Whole Net
- Submenus: **Change Layer ▸** (stack) · **Change Width ▸** (class default · presets · Custom) · **Assign Net Class ▸**

### Via
- **N** Change Size ▸★ · **E** Properties ▪ · **S** Delete ⚠ · **W** Change Layer Span ▸★ (blind/buried)
- Secondary: Move Exactly ▪ · Highlight Net ▪★ · Select Whole Net ▪★ · Lock ▪
- Overflow: ⚠ Change Net · Duplicate · Add Stitching/Array ⚡ · Assign Net Class ▸
- Submenus: **Change Size ▸** (class default · presets · Custom dia/drill) · **Change Layer Span ▸** · **Via Type ▸** (through/blind-buried/micro)

### Pad *(part of a footprint; no standalone delete on the board)*
- **N** Route from Pad ⚡★ · **E** Properties ▪ · **S** Highlight Net ▪★ · **W** Select Whole Net ▪★
- Secondary: Select on Schematic (pin) ⌖★ · Copy Pad Settings ▪ · Push to Identical Pads ▪ · Edit in Footprint Editor ⌖
- Overflow: Change Pad Shape ▸ · Mask/Paste Expansion ▪ · ⚠ Remove from Net
- Submenus: **Assign Net ▸** (net search) · **Change Pad Shape ▸**

### Net *(whole-net semantics on a selected copper item)*
- **N** Highlight Net ▪★ · **E** Select Whole Net ▪★ · **S** Hide/Show Ratsnest ▪★ · **W** Assign Net Class ▸★
- Secondary: Route Net ⚡ · Net Color ▸ ▪ · Tune Length ⚡ · Select on Schematic ⌖★
- Overflow: ⚠ Unroute Net · ⚠ Rename Net · ⚠ Delete Net · Net Length Report ▪
- Submenus: **Assign Net Class ▸** · **Net Color ▸** (palette · use schematic color · clear)

### Copper Zone / Pour
- **N** Fill Zone ▪★ · **E** Properties ▪ · **S** Delete ⚠ · **W** Edit Outline ⚡★
- Secondary: Unfill ▪ · Change Net ▸★ · Change Layer ▸★ · Duplicate onto Layer ▸ ▪ · Add Cutout ⚡★
- Overflow: Zone Priority ▪ · Merge Zones ▪ · Zone→Polygon ▪ · Move Exactly ▪ · Zone Manager ▪
- Submenus: **Change Net ▸** · **Change Layer ▸** · **Duplicate onto Layer ▸**

### Keepout / Rule Area
- **N** Properties ▪★ · **E** Edit Outline ⚡★ · **S** Delete ⚠ · **W** Change Layers ▸★
- Secondary: Move Exactly ▪ · Duplicate ▪ · Add Cutout ⚡ · Overflow: Convert to Zone ▪ · Lock ▪
- Submenu: **Change Layers ▸** (multi-layer checklist)

### Board Text / Silkscreen
- **N** Rotate ▪ · **E** Edit Text/Properties ▪★ · **S** Delete ⚠ · **W** Change Layer ▸★
- Secondary: Mirror/Flip ▪ · Move Exactly ▪ · Duplicate ▪ · Justify ▸ ▪
- Overflow: Convert to Graphics ▪ · Set Thickness ▪ · Create Array ⚡ · Font ▸
- Submenus: **Change Layer ▸** (F/B.SilkS, F/B.Fab, User.*) · **Justify ▸** (L/C/R × T/M/B) · **Font ▸**

### Dimension
- **N** Properties ▪ · **E** Edit/Recompute ▪★ · **S** Delete ⚠ · **W** Change Layer ▸★
- Secondary: Move Exactly ▪ · Override Text ▪ · Change Units ▸ ▪
- Submenus: **Change Layer ▸** · **Change Units ▸** (mm/mil/in/auto) · **Precision ▸**

### Board Outline / Edge.Cuts
- **N** Edit Outline ⚡★ · **E** Properties ▪ · **S** ⚠ Delete Segment (can open the outline — high-consequence) · **W** Fillet/Chamfer ▸★
- Secondary: Add Vertex ⚡ · Convert to Arc ▪ · Move Exactly ▪ · Overflow: Convert to Polygon ▪ · Set Line Width ▪ · Import DXF ⚡

### Empty Canvas (no selection) — *tool launcher*
- **N** Route ⚡ · **E** Place Footprint ⚡ · **S** Add Zone ⚡ · **W** Paste ▪
- Secondary: Add Via ⚡ · Add Text ⚡ · Add Dimension ⚡ · Fill All Zones ▪
- Overflow: Add Keepout ⚡ · Import DXF ⚡ · ⚠ Unfill All · Grid Origin ▪ · Zoom to Fit ▪ · Select All ▪
- Submenus: **Add… ▸** · **Grid ▸**

### Multi-Selection — *intersection / batch launcher*
- **N** Rotate ▪ · **E** Move Exactly ▪ · **S** Delete ⚠ · **W** Align/Distribute ▸★
- Secondary: Duplicate ▪ · Flip ▪ · Lock ▪ · Group/Ungroup ▪
- Overflow: Assign Net Class ▸ (copper) · Change Layer ▸ (homogeneous) · Create Array ⚡ · Select Similar ▸
- Submenus: **Align ▸** [3-layer, icon-driven] → **Horizontal ▸** (Left/Center/Right) · **Vertical ▸** (Top/Center/Bottom) · **Distribute ▸** (Even spacing / Even gaps / To pitch) · **Change Layer ▸** · **Assign Net Class ▸** · **Filter ▸** (by type)

---

# Schematic objects

### Symbol / Component *(the richest menu)*
- **N** Properties ▪ `symbol.edit_properties` · **E** Rotate ▪ · **S** Move ⚡ · **W** Mirror/Flip ▪
- Secondary: Edit Value ▪ · Edit Reference ▪ · Drag (keep wires) ⚡ · Mirror Vertical ▪
- Overflow: Rotate CW · Edit/Assign Footprint ▪ · Show Datasheet · Autoplace Fields ▪ · Highlight Pins' Nets ▪★ · Select on PCB ⌖★ · Align to Grid ▪ · Copy/Cut/Duplicate ▪ · ⚠ Change Symbol · ⚠ Update from Library · ⚠ Delete · ⚠ Reset Fields
- Submenus: **Edit Field ▸** (Reference/Value/Footprint/Datasheet/user/Add…) · **Unit / Convert ▸** (multi-unit; De Morgan) · **Edit Library Symbol ▸** ⚠

### Pin *(on a placed symbol)* / *(library-editor pin — separate object)*
- Canvas pin — **N** Pin Function ▸ · **E** Highlight Net ▪★ · **S** Place No-Connect ▪ · **W** Unfold Wire from Pin ⚡★
- Library pin — **N** Properties ▪ · **E** Electrical Type ▸ · **S** Move ⚡ · **W** Graphic Style ▸
  - **Electrical Type ▸**: Input/Output/Bidir/Tri-State/Passive/Free/Unspecified/Power-In/Power-Out/OC/OE/NC
  - **Graphic Style ▸**: Line/Inverted/Clock/Inv-Clock/Input-Low/Clock-Low/Output-Low/Falling-Clock/Non-Logic

### Wire / Net Segment
- **N** Highlight Net ▪★ · **E** Add Label to Wire ▪ · **S** Drag ⚡ · **W** Add Junction ▪
- Secondary: Assign Netclass ▪★ · Break Wire ⚡★ · Slice ⚠ · Properties ▪
- Overflow: Add Global/Hierarchical Label ▪ · Net Class Directive ▪ · Select Whole Net ▪★ · Cross-probe Net to PCB ⌖★ · Align to Grid · ⚠ Delete Segment · ⚠ Delete Connection
- Submenu: **Change To ▸** (Label/Global/Hier/Directive/Text) when the endpoint carries a label

### Junction
- **N** Highlight Net ▪★ · **E** Properties ▪ · **S** Move ⚡ · **W** Add Label ▪ · Overflow: Assign Netclass · Tie Nets ⚠ · ⚠ Delete Junction (splits net)

### Net Label — Local / Global / Hierarchical / Power
- **N** Properties (net name) ▪ · **E** Change To ▸ · **S** Move ⚡ · **W** Highlight Net ▪★
- Secondary: Rotate ▪ · Assign Netclass ▪★ · Mirror/Flip side ▪ · Edit Text (rename net) ▪
- Overflow: Cross-probe to PCB ⌖★ · Font/Justify ▪ · ⚠ Delete (unnames/splits net)
- Submenu: **Change To ▸** — Label · Global · Hierarchical · Directive · ⚠ Text (drops net semantics). *This is the schematic-defining conversion wheel.*
- Power subtype: N Properties = net name (+3V3/GND); Overflow adds Change Power Symbol, Edit Value = net name.

### Bus
- **N** Unfold from Bus ▸★ · **E** Add Bus Label ▪ · **S** Drag ⚡ · **W** Highlight Bus/Members ▪★
- Secondary: Properties ▪ · Add Bus Entry ▪ · Break Bus ▪ · Edit Bus Definition ▪
- Overflow: Assign Netclass · Slice ⚠ · Align · ⚠ Delete Bus
- Submenu: **Unfold from Bus ▸** (dynamic member list `D0…Dn` + Add member) — auto-generates entry + wire + member label

### Bus Entry
- **N** Properties ▪ · **E** Flip Direction ▪ · **S** Move ⚡ · **W** Highlight Net ▪★ · Overflow: Add Label · Align · ⚠ Delete

### No-Connect Marker *(the delete-on-cardinal exception)*
- **N** Move ⚡ · **E** Properties ▪ · **S** Delete ▪ (symmetric with place) · **W** — (or Highlight Pin)

### Hierarchical Sheet *(navigation-heavy)*
- **N** Enter Sheet ▪ · **E** Properties ▪ · **S** Move ⚡ · **W** Import Sheet Pin ▪
- Secondary: Add Sheet Pin ▪ · Resize ⚡ · Drag ⚡ · Rename ▪
- Overflow: Highlight Sheet Nets ⌖★ · Annotate This Sheet ▪ · Edit Page Number ▪ · Reveal in Navigator ▪ · ⚠ Change Sheet File · ⚠ Delete Sheet
- Submenu: **Sheet Actions ▸** (Enter/Leave/Import All Pins/Cleanup Unused)

### Hierarchical / Sheet Port (Sheet Pin)
- **N** Properties ▪ · **E** Edit Shape ▸ (I/O) · **S** Move along edge ⚡ · **W** Highlight Net ▪★
- Overflow: Sync to Sheet Label ▪ · Add Wire from Port ⚡ · Assign Netclass · ⚠ Delete
- Submenu: **Edit Shape ▸** (Input/Output/Bidir/Tri-State/Passive)

### Text / Graphic
- **N** Properties ▪ · **E** Edit Text ▪ · **S** Move ⚡ · **W** Change To ▸ (text→label)
- Secondary: Rotate ▪ · Edit Text & Graphics Properties (global) ▪ · Mirror ▪ · Line Style/Fill ▸
- Overflow: Duplicate · Align · Order ▸ (front/back) · ⚠ Delete
- Submenus: **Change To ▸** (Label/Global/Hier/Directive) · **Line Style ▸**

### Empty Canvas (no selection) — *tool launcher*
- **N** Place Symbol ⚡ · **E** Place Wire ⚡ · **S** Place Power Symbol ⚡ · **W** Place Label ⚡
- Secondary: Place Global Label ⚡ · Place Hierarchical Label ⚡ · Place Sheet ⚡ · Place Bus ⚡
- Overflow: Net Class Directive · No-Connect · Junction · Text/Graphic · Paste ▪ · ⚠ Annotate Schematic · Run ERC ▪ · Zoom/Grid ▪ · Enter/Leave Sheet ▪ · Schematic Setup ▪
- Submenu: **Place ▸** (full palette mirror)

### Multi-Selection — *intersection / batch launcher*
- **N** Move ⚡ · **E** Rotate ▪ · **S** Mirror H/V ▪ · **W** Duplicate ▪
- Secondary: Align ▸★ · Assign Netclass (if nets) ▪ · Copy/Cut ▪ · Cross-probe to PCB ⌖★
- Overflow: Distribute ▸ · Align to Grid · Group/Ungroup · Edit Common Field (homogeneous) ▪ · Highlight All Nets ▪ · ⚠ Change/Update Symbols · ⚠ Delete
- Submenus: **Align ▸** [3-layer, icon-driven] → **Horizontal ▸** (Left/Center/Right) · **Vertical ▸** (Top/Center/Bottom) · **Distribute ▸** (Even spacing / Even gaps / To pitch) · **Edit Field ▸** (homogeneous)

---

## Notes for implementation

- The **conversion sub-wheels** (`Change To ▸`, `Change Layer ▸`, `Electrical Type ▸`,
  `Change Width/Size ▸`, `Assign Net Class ▸`, `Align ▸`) each map to a single
  *parameterized* engine op (`change_type(t)`, `set_layer(l)`, …) — the exact
  "capability = a parameter of a small verb set" lean-tooling ethos. They are the
  natural home for the depth-2 cardinal sub-wheel.
- Menu items should be **generated from the verb registry** (like the menu bar) so
  each leaf references a real `datum.*` verb; gaps here (ops that don't exist yet)
  are the buildout worklist for the authoring surfaces.
