# MM-C06 — disabled-action inventory (landed inert marking-menu shell)

**Scope:** the landed marking-menu shell (MM-C03 `3541d0ea`, MM-C04 `5ae409eb`, wedge form `42b3b525`).
**Sources (read-only, no build):** live menu model `docs/gui/menu_model.json` (last changed `098afc7c`) + shell code `crates/gui-render/src/marking_menu.rs` / `marking_menu/sub_wheel.rs` (inert render rule) + `crates/gui-protocol/src/gui_menu_model.rs` (bindings).

## Controlling fact — the entire shell is inert
Every action/leaf in all 12 marking-menu keys is **disabled**: selecting any leaf invokes **no verb**, authors **no typed Operation**, writes **no journal entry**, and mutates nothing. This holds *regardless of binding type* — even leaves carrying a real `datum.*` verb do **not** fire in this build. The shell deliberately ignores registry capability (`is_enabled`) and keeps every verb leaf inert (the MM-C03 inert-pilot rule). The **only** live behaviors are UI navigation (opening/closing a submenu sub-wheel via its parent / the center back affordance) and dismiss — neither touches the design.

Binding tags below: `[verb]` = a real `datum.*` verb exists (inert only because of the pilot; goes live with the verb-firing successor); `[submenu]` = navigable parent (sub-wheel opens; its terminal leaves are inert); `[NOT-BUILT]` = **no verb exists** — an authoring obligation that will not work even once the shell goes live.

## A. Per-key leaf inventory (every leaf = NOT working)

**pcb.component** — N Rotate `[verb]` · E Properties `[NOT-BUILT: component properties dialog]` · S Delete `[verb,destructive]` · W Flip `[verb]` · NE Move `[verb]` · SE Lock `[verb]` · SW Highlight Net `[NOT-BUILT: net.highlight view op]` · NW Select on Schematic `[NOT-BUILT: crossprobe.select]` · overflow: Set Reference `[verb]`, Set Value `[verb]`, Unlock `[verb]`, Align/Distribute `[verb]` · submenu Change Footprint → Set Package `[verb]`, Set Part `[verb]`

**pcb.track** — N Change Layer `[submenu + NOT-BUILT: change-layer inserts via]` · E Drag `[NOT-BUILT: interactive push/shove drag]` · S Delete `[verb,destructive]` · W Change Width `[verb]` · NE Route `[verb]` · SE Break `[NOT-BUILT: wire.break at point]` · SW Highlight Net `[NOT-BUILT: net.highlight]` · NW Properties `[verb]` · overflow: Delete Whole Net `[verb,destructive]`

**pcb.via** — N Change Size `[verb]` · E Properties `[verb]` · S Delete `[verb,destructive]` · W Change Net `[verb]`

**pcb.pad** — N Change Size `[verb]` · E Properties `[NOT-BUILT: pad properties dialog]` · S Delete `[verb,destructive]` · W Assign Net `[verb]` · NE Move `[NOT-BUILT: pad-local move]` · SE Change Layer `[NOT-BUILT: pad layer editor]` · SW Highlight Net `[NOT-BUILT: net.highlight]` · NW Lock `[NOT-BUILT: pad lock]`

**pcb.zone** — N Fill `[verb]` · E Properties `[verb]` · S Delete `[verb,destructive]` · W Edit Outline `[NOT-BUILT: interactive vertex edit]`

**pcb.net** — N Highlight `[NOT-BUILT: net.highlight view op]` · E Select Whole Net `[NOT-BUILT: net.select view op]` · S Hide Ratsnest `[NOT-BUILT: ratsnest toggle view op]` · W Assign Net Class `[NOT-BUILT: assign netclass from canvas]` *(every leaf of this key is NOT-BUILT)*

**pcb.empty** — N Route `[verb]` · E Place Component `[verb]` · S Add Zone `[verb]` · W Paste `[NOT-BUILT: clipboard model]` · NE Add Via `[verb]` · SE Add Text `[verb]` · SW Add Dimension `[verb]` · NW Fill All Zones `[verb]`

**pcb.multiselect** *(the MM-C04/C05 proof subject)* — N Rotate `[verb]` · E Move Exactly `[NOT-BUILT: move-exactly with delta]` · S Delete `[verb,destructive]` · W Align `[submenu]` · NE Duplicate `[NOT-BUILT: duplicate selection]` · SE Flip `[verb]` · SW Lock `[verb]` · NW Distribute `[submenu]` · submenus Align/Distribute → all terminal leaves `[verb: datum.pcb.align_components]` (inert)

**schematic.symbol** — N Properties `[NOT-BUILT: symbol properties dialog]` · E Rotate `[verb]` · S Move `[verb]` · W Mirror `[verb]` · NE Edit Value `[verb]` · SE Edit Reference `[verb]` · SW Highlight Net `[NOT-BUILT: net.highlight]` · NW Select on PCB `[NOT-BUILT: crossprobe.select]` · overflow: Delete `[verb,destructive]`, Add Field `[verb]`, Assign Part `[verb]` · submenu Unit → Set Unit `[verb]`

**schematic.wire** — N Highlight Net `[NOT-BUILT: net.highlight view op]` · E Add Label `[verb]` · S Drag `[NOT-BUILT: interactive wire drag]` · W Add Junction `[verb]` · overflow: Delete `[verb,destructive]`

**schematic.label** — N Rename `[verb]` · E Change To `[submenu + NOT-BUILT: label type conversion]` · S Move `[NOT-BUILT: move-label verb]` · W Highlight Net `[NOT-BUILT: net.highlight]` · overflow: Delete `[verb,destructive]`

**schematic.empty** — N Place Symbol `[verb]` · E Place Wire `[verb]` · S Place Power `[NOT-BUILT: place power symbol]` · W Place Label `[verb]` · NE Place Bus `[verb]` · SE Place Text `[verb]` · SW Place No-Connect `[verb]` · NW Place Junction `[verb]`

## B. Remaining verb/tool authoring obligations (`NOT-BUILT` — no verb; will not work even once the shell goes live)
Deduplicated worklist with the leaves each blocks:
1. **net.highlight (view op)** — pcb.component·SW, pcb.track·SW, pcb.pad·SW, pcb.net·N, schematic.symbol·SW, schematic.wire·N, schematic.label·W
2. **net.select whole net (view op)** — pcb.net·E
3. **ratsnest toggle (view op)** — pcb.net·S
4. **assign netclass from canvas** — pcb.net·W
5. **crossprobe.select (cross-editor select)** — pcb.component·NW (Select on Schematic), schematic.symbol·NW (Select on PCB)
6. **clipboard model (paste / duplicate)** — pcb.empty·W (Paste), pcb.multiselect·NE (Duplicate)
7. **properties dialog verbs** — pcb.component·E, pcb.pad·E, schematic.symbol·N
8. **interactive drag (push/shove; wire drag)** — pcb.track·E, schematic.wire·S
9. **wire.break at point** — pcb.track·SE
10. **change-layer (inserts via)** — pcb.track·N (submenu parent)
11. **pad-local move** — pcb.pad·NE
12. **pad layer editor** — pcb.pad·SE
13. **pad lock** — pcb.pad·NW
14. **zone interactive vertex edit** — pcb.zone·W
15. **move-exactly with delta** — pcb.multiselect·E
16. **label type conversion** — schematic.label·E (submenu parent)
17. **move-label verb** — schematic.label·S
18. **place power symbol** — schematic.empty·S

## C. Successor dependency (stated truthfully, not part of C06)
The verb-bound leaves (`[verb]`) go live only with the verb-firing successor in the GUI-WRITE-PATH era — which is itself hard-gated on radial screen-reader node coverage (bead `dat-mm-radial-a11y-node-coverage-gtww`; the landed shell emits 0 accessibility nodes for the radial menu). Until then, **no marking-menu action works** — by design.
