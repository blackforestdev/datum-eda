# MM-C01 readiness reconciliation + MM-C02 owner-authorization queue

**Step:** GUI-MARKING-MENU / MM-C01 (planning). **By:** dev-agent-01 (claim-holder, front opened 29ef2a58). **Visual lane:** frontend-dev-01. **Date:** 2026-10-26.
**Deliverable:** reconcile the exact inert-shell readiness, live visual authority and full native scenario matrix, and produce the MM-C02 owner-authorization queue. **Bound:** inert shell only; no implementation (MM-C03), no owner authorization (MM-C02 is the gate), no operation invocation, no prototype edit/bless, no write-path, no resolving the owner questions.

## §0 — Selection status (RESOLVED)
MM is properly selected: front opened in 29ef2a58 under the owner's recorded "full authority to begin any work / remove roadblocks" grant; feature_delivery contract recorded (inert-shell-scoped, two gesture specifics softened, PM-approved); state=in_progress, authorization=planning, claim=dev-agent-01, MM-C01 in_progress; both gates green; next=MM-C01. The earlier selector blocker ("STOP: unselected work") no longer applies. INSP-C06 preserved + carried as Tap B in the owner packet.

## §1 — Authority order (controlling) — UNCHANGED, verified
WHEEL CONTENT (live, test-pinned): (1) docs/gui/menu_model.json/.csv + check_menu_model.py; (2) DATUM_GUI_CONTEXT_MENU_CONTENT.md; (3) icon_set.json.
INTERACTION/GEOMETRY/RATIONALE: (4) DATUM_GUI_DESIGN_SPEC.md §Context-Menu-System — EXCEPT the L477 per-object cardinal TABLE (drifted → Finding F1, record-only); (5) DATUM_GUI_PARAMETRIC_TOOLING.md; (6) PM023; (7) prototypes = HISTORICAL STUDY, illustrative not golden, Claude-visual-lane-owned read-only.

**Finding F1 (record-only, do NOT fix in MM-C01):** DESIGN_SPEC L477 cardinal table drifted from live menu_model.json. Verified live pcb.component: cardinal N Rotate / E Properties(not_built) / S Delete(destructive) / W Flip; Move secondary NE; Align/Distribute overflow (datum.pcb.align_components); component sub-wheel = Change Footprint. Spec-text reconciliation routes through the owning evidence route (digest reconciliation), out of MM-C01 scope.

## §2 — In-scope native surfaces & wheels
Contextual object wheel (per type; cardinal-4 + diagonal secondary; ≤8 slots); empty-canvas wheel (space-level, no object verbs); multi-selection wheel (intersection; "…Selected…"); overflow "More…" leaf → linear scannable list — **IN-SCOPE and governed** (completion plan L26 "preserve … overflow", L37/L81 "overflow navigation"): renders as a DISABLED, truthful, navigable list, NOT a dead-end. The **tear-off / float-as-palette** interaction is governed-SILENT → spec-silent default = LATER-delivery, not inert-shell (flagged, not silently excluded). Sub-wheel spawn-in-place offset-outward, parent faded+blurred, nesting ≤2. (All inert in this deliverable — leaves disabled, no verb fires.)

## §3 — Visual contract (frontend-dev-01, visual lane)
Source: .../8e619a1b-.../scratchpad/marking-menu/ (MARKING_MENU_VISUAL_CONTRACT.md + correction banner + prototype-base.png, prototype-subwheel.png, g1-gap-evidence.png). Correction banner SUPERSEDES body: build cardinals from menu_model.json (not DESIGN_SPEC L477); Align full tree lives IN the wheel on pcb.multiselect (frontend's "tool-inspector" reading WITHDRAWN); live Align sub-wheels are 3-wide. Invariants: contiguous semi-transparent wedges over blurred board; hub=object ref, ◀ when nested; active wedge solid --acc #CE5A92; FROZEN geometry (constant slots, greyed-in-place, no reflow); Delete stays cardinal-S (check_menu_model.py gate); ▸=sub-wheel; dual-mode hold/flick; every item has a keyboard accel.
Build-to-SPEC (don't copy prototype): D1 build 4-cardinal+4-diagonal (not prototype even-6); D2 icon-in-wheel unpinned (→G3); D3 the "More…" linear-overflow's VISUAL REFERENCE is a separate mock (not drawn in the radial prototype) — this is a visual-SOURCE note, NOT a scope exclusion: the overflow list is in-scope per §2 (governed L26/L37/L81), built from that separate mock.

## §4 — MM-C02 OWNER-AUTHORIZATION QUEUE (researched options + recommendations; owner resolves, I do NOT)
Five items: G1, G2, G3, G6 + F1 awareness. (Governed/retired: pan→PM023; G5→PARAMETRIC_TOOLING; G4→terminology. See §4-governed.)

### G1 — Edge / narrow / off-screen placement  [THE visual-lane gap]
Problem: sub-wheels spawn offset-outward; near a viewport edge / in a narrow pane, the wheel or a sub-wheel spills off-canvas. Zero governed clamp/reposition contract (grep DESIGN_SPEC+CONTENT for edge/narrow/clamp/reposition → 0 hits). g1-gap-evidence.png shows the spill (labeled "NOT a proposed contract"). frontend: don't invent clamp behavior.
Production practice: Blender pie menus clamp the whole pie inward to stay fully on-screen, keeping slot DIRECTIONS constant relative to the (shifted) hub so blind-gesture memory survives; Maya marking menus render at cursor with minimal clamping (relies on gesture memory); Sketchbook Pro (the prototype's model) spawns sub-wheels offset-outward with no clamp.
Options:
  (a) Clamp the whole wheel inward so it fits on-screen; slot directions stay constant relative to the hub (blind-gesture memory preserved). Sub-wheels that would spill spawn inward (mirror the offset) or clamp.
  (b) Open at cursor without clamping (prototype/Maya model); accept reduced reachability near edges.
  (c) Refuse to open within an edge margin; nudge the open point inward.
Recommendation: **(a)** — it is the only option that preserves the FROZEN-geometry / blind-gesture invariant (constant slot directions) AND guarantees reachability; it matches the field-dominant Blender model. This is a RECOMMENDATION, not a build decision (frontend pushback applied): **placement geometry is OWNER-GATED at MM-C02**. MM-C03 renders whichever option the owner ratifies — defaulting to spec-silent offset-outward if the owner leaves it unresolved — NOT a hardcoded clamp-inward. **Explicit MM-C02→MM-C03 dependency:** the ratified placement rule gates the C03 render geometry; the inert shell does not pre-build clamp-inward. Owner question: approve clamp-inward (a), or pick (b)/(c)?

### G2 — Delayed-display (~280ms hold) + expert-flick + mark-trail timing
Problem: the dual-mode (hold→labeled wheel draws; immediate flick→command fires, nothing draws) + ~280ms threshold + mark-trail timing are prose-only, "still to prove" (DESIGN_SPEC L495); the static prototype doesn't demonstrate them.
Inert-shell bearing: the shell has NO firing path (inert), so the expert-flick-fires leg is moot for the shell. Only "the wheel renders on invocation" is in inert-shell scope.
Options:
  (a) Inert shell renders the wheel on invocation; the ~280ms hold threshold, flick-fires-nothing-draws and mark-trail are a LATER verb-firing-delivery concern (deferred out of the inert shell).
  (b) Build the 280ms hold threshold + mark-trail timing now in the inert shell.
Recommendation: **(a)**, with frontend's nuance applied: the ~280ms hold threshold is DISPLAY-ADJACENT — it governs whether/when the wheel DRAWS, not only whether a verb fires — so the deferral is "dual-mode timing/discrimination + mark-trail defer," not purely a firing concern. The inert shell renders-on-invocation; the hold-to-draw discrimination, expert-flick and mark-trail timing defer to the later verb-firing delivery. Owner question: confirm the inert shell renders-on-invocation, and that the dual-mode timing/discrimination + mark-trail defer to the verb-firing delivery.

### G3 — Icon-in-wheel treatment (icon vs label vs both)
Problem: prototype wedges are label+key only; DESIGN_SPEC says "one verb = one icon everywhere" — whether the wheel renders icon, label, or both (and wedge sizing) is unpinned.
Production practice: Blender pie = icon+label; Maya = label (icon optional); the "one verb = one icon everywhere" principle argues for showing the icon.
Constraint: icon_set.json has `to_author` glyphs not yet drawn — some verbs lack an icon today.
Options: (a) label+key only (prototype); (b) icon+label (honors one-verb-one-icon + keeps blind-learning labels); (c) icon-only (compact, fails blind-learning).
Recommendation: **(b) icon+label**, degrading to label+key for verbs whose icon is still `to_author`. Owner question: pin icon+label (b) as the wheel treatment; note icon_set.json completeness gates full icon coverage (a glyph-authoring follow-up, not inert-shell-blocking). **Open sub-point (frontend note — unpinned even if (b) wins): WEDGE SIZING.** The icon size within a wedge is not pinned by the reference visual; choosing icon+label does NOT settle wedge layout / icon-sizing, which stays an owner/visual-lane sub-decision.

### G6 — Sub-wheel WIDTH (4-wide cardinal-only vs 3-wide live)  [genuine design-rule conflict]
Problem: DESIGN_SPEC L429 "any sub-wheel is 4-wide (cardinal-only)" is contradicted by BOTH PARAMETRIC_TOOLING L56-73 AND the test-pinned live data — live Align sub-wheels are 3-wide (Align→{Horizontal,Vertical,Distribute}; {Left,Center,Right}).
What the data PROVES (frontend pushback applied): the live + PARAMETRIC evidence establishes **3-wide for the Align sub-wheels specifically** — NOT a general "any N-wide" rule. So the owner decision is between: **(i) generalize** L429 to N-wide (sub-wheels content-sized; 3-wide where Align needs it), same drift-relaxation class as F1; or **(ii) pin per-content** — correct L429's "4-wide cardinal-only" to the proven 3-wide Align case and leave other sub-wheels' widths to their own content. Recommendation: **(i) generalize**, since forcing 4-wide-cardinal-only contradicts the only concrete sub-wheel we have (3-wide Align) — but (i) vs (ii) is the owner's call, not established. The DESIGN_SPEC L429 text reconciliation routes through the owning evidence route (not edited in MM-C01). Owner question: (i) generalize to N-wide, or (ii) pin per-content correcting L429 to the proven 3-wide?

### F1 (awareness, 5th item) — canonical cardinal assignment
DESIGN_SPEC L477 per-object cardinal PROPOSALS (owner-tunable, still-to-prove) differ from the test-pinned live menu_model.json. NOT a spec bug to silently fix — the owner ratifies which cardinals are canonical (finishes the still-to-prove tuning). The inert shell builds cardinals from the live menu_model.json (per the authority order); F1 asks the owner to ratify that live set (or retune it). Spec-text reconciliation stays record-only via the owning route.

### §4-governed (cite authority; dropped from the owner queue)
- Pan / right-button: GOVERNED by PM023 (right reserved for context menu; pan=Space+primary; middle=3D rotate). MM's "no pan fallthrough / no editor-fork" is satisfied — cite PM023. Residual (not stale): the Space+drag trackpad-viability issue dat-dwt-touchpad-nag stays open on its own track, OUT of MM-C01 scope.
- G5 (Align placement): GOVERNED by PARAMETRIC_TOOLING L56-73 (+ live) — Align is a 3-layer in-wheel tree on pcb.multiselect by design; component overflow entry correct by design. Retired.
- G4 (Align depth "≤2 vs 3-layer"): terminology — "≤2 levels" (entry + 2 sub-wheel levels) == "3-layer". Document the counting convention. All Align layers live IN the wheel (frontend's tool-inspector reading withdrawn).

## §5 — Native handlers / entry surfaces / source scope (MM selected → audit's consumers folded in)
- crates/gui-app/src/editor/runtime_selection_menu.rs: open_marking_menu_at_cursor, open_marking_menu_for_keyboard, handle_marking_selection_key.
- crates/gui-render/src/marking_menu.rs, marking_menu/selection_list.rs, render/selection_menu_hover.rs, render/marking_shell_tests.rs.
- **FOLDED IN (audit, post-selection): crates/gui-app/src/editor/runtime_primary_pointer.rs (marking_menu_active() gating + dismiss) and runtime_hit_actions.rs (HitTarget::MarkingMenuItem / MarkingMenuSurface routing).**
- crates/gui-app/src/editor/runtime_primary_button.rs, application/app_native_events.rs, application/runtime_menu_actions.rs.
- Data/gate: menu_model.json/.csv, icon_set.json, check_menu_model.py; gui_menu_model.rs + action_registry (~5 current consumers to reconcile against the full contextual/empty-canvas/multi-selection matrix).
- Right-button ownership + shared pane targeting; no pan fallthrough, no editor-specific fork (PM023).

## §6 — Eight foundation answers (argue against source, not assert)
Units/precision · Numeric entry · Selection identity · Grid/snap · Undo/cancel · Persistence/recovery · Library/connectivity · Settings scope. For an INERT shell that invokes NO typed op, most reduce to "unchanged / N-A for a read-only shell" WITH reasons (selection identity + grid/snap come from the shared backbone unchanged; no persistence/undo because no mutation). To be argued line-by-line in the MM-C02 packet, not asserted.

## §7 — Proof plan (owner authorizes execution against this; proportional to an INERT shell — concrete beats long)

**Acceptance shape per downstream step (inspector-C0x precedent, scaled DOWN):**
- **MM-C03 (implement the authorized inert shell):** renders the in-scope surfaces from the test-pinned menu_model.json — contextual object wheel (per type), empty-canvas wheel, multi-selection wheel, AND the overflow "More…" leaf + its linear overflow list (governed in-scope per completion plan L26/L37/L81: the list renders DISABLED, truthful and navigable — no dead-end; tear-off/float deferred to later delivery) — with the ratified visual contract (contiguous wedges over blurred board; FROZEN geometry; Delete cardinal-S; ▸ sub-wheels spawn-in-place depth-2; icon treatment per the owner's G3 ruling (recommendation: icon+label); cardinals per live menu_model / the owner's F1 ratification; sub-wheel width per the owner's G6 ruling). Every leaf DISABLED (inert), each showing its truthful `not_built` reason from the manifest. Right-button ownership; no pan fallthrough; shared pane targeting. NO operation invocation, NO write-path, NO enabled leaf. **Acceptance = renders-per-contract + content-matches-menu_model, nothing fires.**
- **MM-C04 (prove usable shell behavior + nonmutation):** the nine dimensions below, via SAME-ENGINE goldens (production renderer, NOT an HTML-vs-wgpu diff) + the menu_model/icon gate (check_menu_model.py) + layout/token/conformance/resource gates. Nonmutation proven STRUCTURALLY — the inert shell's hit targets reach no `.commit`/journal/native.write (the inspector's `unreachable!()` write-path pattern is the model).
- **MM-C05 (independent replay):** a DISTINCT reviewer (not the implementer) replays the full contextual/empty-canvas/multi-selection matrix incl. failure/stale/mixed states, accounts for every finding — the inspector-C05 precedent.
- **MM-C06 (owner acceptance):** owner accepts the inert shell; the finished verb-firing menu is a separate later delivery contract.

**Nine proof dimensions (MM-C04), concrete inert-shell shape:**
1. **Normal** — each in-scope context's wheel renders with correct menu_model content, labels/icons, disabled leaves (golden per context × size).
2. **Invalid** — disabled-leaf activation attempts stay unavailable with truthful `not_built` reasons; no verb fires.
3. **Cancel** — dismiss (no invocation); normal editor use continues; no residual state.
4. **Scope** — correct wheel per object type / selection intersection ("…Selected…") / empty canvas.
5. **Precision** — N/A: an inert shell takes no numeric entry (argued, not exercised).
6. **Undo/redo** — N/A: no mutation → no journal entry (argued; corroborated by the structural no-write proof).
7. **Save/reopen** — clean close/reopen; byte-identical authored source + no history change (inspector-style).
8. **Accessibility (non-color)** — disabled/unavailable state legible without color (reason text + non-color indicators); keyboard-reach path present.
9. **Failure/recovery** — right-button never falls through to pan; stale/mixed/incompatible subjects refuse truthfully; recover to normal editor use.

## §8 — Scope boundaries
No implementation (MM-C03), no owner authorization (MM-C02 is the gate — I stop there), no operation invocation, no prototype edit/bless, no write-path, no resolving G1/G2/G3/G6/F1. Keep/drop G-set framing is a readiness RECOMMENDATION, never governed fact.

## Appendix A — Closed questions (owner-visible, NOT open decisions)
**Trace-guard softenings — what the inert-shell contract deliberately does NOT pin (so the owner sees the gaps left working-level):**
- "by its governed invocation gesture" (NOT "press/hold") — the G2 press/hold threshold stays working-level, owner-tunable at the finished menu, not asserted by the inert-shell contract.
- Dismissal stated without pinning keys ("(release/escape)" dropped) — the dismissal mechanism stays working-level.

**Governed / retired — cite authority, NOT owner questions:**
- **Pan / right-button** → PM023 (right-click/right-drag reserved exclusively for the context menu; pan = Space+primary; middle-drag = 3D rotate). MM's "no pan fallthrough / no editor-fork" is satisfied by citation. Residual `dat-dwt-touchpad-nag` (Space+drag trackpad viability on KDE Wayland) stays open on its OWN track, out of MM scope.
- **G5 (Align placement)** → PARAMETRIC_TOOLING L56-73 + live menu_model: Align is a 3-layer in-wheel tree (Align→axis→position) on pcb.multiselect by design; the component's flat overflow entry is correct by design. Retired.
- **G4 (Align depth "≤2 vs 3-layer")** → terminology: "≤2 sub-wheel levels" == "3-layer" counting the entry; all Align layers live IN the wheel (frontend's earlier "tool-inspector" reading withdrawn). Document the counting convention.
