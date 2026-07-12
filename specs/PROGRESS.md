# Specification Progress Tracker

> **Purpose**: Maps every requirement from the controlling specs to its
> implementation status. Updated when code changes.
>
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

---

## Active Frontier — Next Development Steps (canonical)

> **This is THE answer to "what is the next logical development step /
> specification?"** (CLAUDE.md → Specification Governance → Roadmap Wayfinding).
> Ordered, dependency-aware; every item links to its governing spec/decision.
> Detail lives in the sections below — this list is the single entry point.
> Update it in the SAME change as any spec creation or course-correction.

0. **Immutable source-health governance restoration — LANDED.** Decision 022
   restores repository-wide 700-line production/test
   tripwires, 350-line inline-test limits, exact zero-headroom legacy ceilings,
   merge-base downward ratchets, touched-monolith burn-down, automatic untracked
   child discovery, and logical `include!` measurement. The operational policy,
   comprehensive checker, hermetic checker tests, full clean-HEAD debt baseline,
   CODEOWNERS surface, and CI/base-SHA enforcement land as one governed recovery
   track. *Dependency:* audit of the July 2 governance retirement and July 9
   flag-ledger replacement complete. *Unblocks:* safe continuation of every
   feature frontier item without further silent monolith growth. *State:*
   **LANDED:** controlling decision + operational policy, 96-path/97-metric exact
   clean-HEAD debt ledger, tracked+untracked whole-repository discovery,
   recursive literal-`include!` measurement, merge-base immutability and
   touched-monolith extraction checks, 13 hermetic regressions, parity inventory,
   CI trusted-base wiring, PR declarations, and CODEOWNERS protection. Governing:
   decision 022 + `docs/SOURCE_HEALTH_POLICY.md`.
1. **Deepen the GUI product specification for the surfaces not yet designed.**
   The **board editor surface is defined and buildable** — its "how" is captured
   in `docs/gui/DATUM_GUI_DESIGN_SPEC.md`, the controlling visual prototype
   `docs/gui/prototypes/board-editor.html` (pro-audio/Bitwig-Ableton idiom, built
   from the Design Book tokens), the complete data-driven menu specification
   (`docs/gui/menu_model.json` + `.csv`, gated by `check_menu_model.py`), the
   context-menu system (`docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md` on
   `research/gui-context-menus/CONTEXT_MENU_RESEARCH.md`), and the deep-verb /
   tri-modal tooling model (`docs/gui/DATUM_GUI_PARAMETRIC_TOOLING.md`). **This
   does NOT gate step 2** — the board build has everything it needs. What remains
   to *define* is the **other surfaces** (schematic editor, library browser) and
   the still-open design decisions in the design spec (dock-vs-overlay,
   `open-decisions.html`). The **Datum visual identity** for symbols/footprints/
   silk/icons is now captured and largely locked in the **Datum Rendering Book**
   (`docs/gui/DATUM_RENDERING_BOOK.md`, owner-approved through the
   `docs/gui/prototypes/rendering-study.html` prototype; symbol standard locked to
   IEC rectangular, and IBM Plex is wired into the engine text registry).
   `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` remains the governed policy contract; the
   design spec + prototypes are what turn each surface into a buildable design.
   Governing: decision 019 + `DATUM_GUI_PRODUCT_SPEC.md` + decisions 014/015.
   State: **ongoing per-surface design work, parallel to the board build; scope
   shaped with the owner.** Gates only the schematic/library surfaces (step 6),
   not the board shell (steps 2–3).
2. **GUI Phase 1 build — application shell + board render fidelity (GO, buildable
   today).** Build the shell + board render on the `datum-test` fixture,
   read-only, with screenshot goldens + owner review; needs **no** write-path.
   **Precise executable spec: `docs/gui/DATUM_GUI_PHASE_1_SPEC.md`** (deliverables
   D1–D7, reuse map, acceptance gates, binding Do-NOT list) realizing
   `docs/gui/prototypes/board-editor.html`. Reads work via `run_cli_json`. This is
   the rail for the first GUI build — point a code agent here. **Conformance rail:**
   `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md` makes each prototype claim an actionable
   per-region checklist with an honest check disposition (ENFORCED / TO-ENFORCE /
   HUMAN) so the build is driven to match the prototype and drift is caught; it also
   carries the machine-layer gap register (checks to add) and the open-reconciliation
   register (owner calls) for this slice. It is the **pilot** for the
   spec-actionability discipline (every GUI-spec claim carries one honest check
   disposition) and the **template** for a future doc-by-doc pass that applies the
   same discipline to the marking-menu (step 3), command console (step 4), and
   schematic/library (step 6) surfaces as each reaches buildable definition
   (`DATUM_GUI_CONFORMANCE_SPEC.md` §7). **Human-review loop:**
   `docs/gui/reference/` (README + `board-editor.png`) is the HUMAN layer of that
   conformance rail — the committed reference image plus the region-by-region eyeball
   protocol against the build's chrome goldens
   (`crates/gui-render/testdata/golden/board/datum-test.scale-*.golden.png`); no check
   there pixel-diffs wgpu against the HTML prototype.
2b. **GUI Phase 2 — dual-pane + populated inspector (governed by
   `docs/gui/DATUM_GUI_PHASE_2_SPEC.md`).** The second GUI build phase: bring the
   `datum-test` shell to the *full* `board-editor.html` composition — a populated
   component inspector beside a split Board+Schematic view whose panes cross-probe
   the same selection. Sequenced in dependency order in the spec: **P2.0** populated
   single-pane component inspector (LANDED — the parity capture presets a component
   selection via the `--select <refdes>` launch flag, repointing the poor empty
   `--demo-known-good` target onto the prototype's single-pane composition,
   ENFORCED by `check_gui_visual_parity.py` against the shell golden) → **P2.1**
   split Board+Schematic dual-pane layout (**first slice LANDED** — the central
   viewport is a real two-pane split: Board pane A focused (accent frame + focus
   dot + active tools + the board world scene) | Schematic pane B unfocused (muted
   header, dimmed tools) over a labeled "Schematic (coming)" placeholder canvas,
   with per-pane headers, a divider gutter, and single-source focus driving
   context-follows-focus; `ShellLayout::viewport_panes()` + invariant tests +
   re-blessed shell/board goldens; pane B world geometry / focus-switch / cross-probe
   remain later slices) → **P2.2** schematic pane populated
   read-only from the engine (reuses `load_kicad_schematic_workspace_state`) →
   **P2.3** cross-probe (one selection identity projected into both panes via the
   existing `SelectionTarget`/`context_envelope::from_selection` substrate) →
   **P2.4** full Identity/Placement/Checks inspector sections. Each carries an
   honest check disposition (ENFORCED / TO-ENFORCE / HUMAN); P2.2–P2.4 remain
   **spec-only — build is a separately-authorized execution phase**. *Dependency:*
   Phase-1 shell + board fidelity (step 2) landed. *Unblocks:* the schematic /
   library authoring surfaces (step 6), which build on the read-only schematic pane
   and cross-probe substrate. *State:* **P2.0 populated single-pane inspector
   LANDED; P2.1 split-view first slice (two-pane LAYOUT + headers + focus +
   placeholder pane B) LANDED; P2.1 pane-tiling depth (dynamic single/split/close/
   nesting + Zoom + independent per-pane cameras + divider-drag resize) LANDED;
   P2.2 schematic render in pane B LANDED (multi-scene + per-element colour +
   interactive focused-pane camera + typed-object geometry + square grid);
   P2.3–P2.4 spec'd, build deferred to authorized execution** (cross-probe is the
   next slice, still DEFERRED).
   **Pane model — decision 021** (`docs/decisions/PRODUCT_MECHANICS_021_WORKSPACE_PANE_TILING.md`):
   the split view is the first implementation of a **recursive binary tile tree**,
   tile-first + View-menu-managed, with **Zoom/maximize** and deliberate **Float/detach**
   as bounded overlay modes. The hard-coded P2.1 fixed split becomes dynamic panes
   (single-pane default → split/close → nesting → divider-drag resize); layout is
   consumer/workspace state, never journaled. Distinct from decision-020 paper-space
   viewports. Reference: `docs/gui/prototypes/workspace-panes.html`. Near-term slice
   ordering under 021: **dynamic single/split + Zoom + divider-drag resize** (resolves
   "board-only / schematic-only / both" as a user choice and lets the owner set any
   split ratio) — **LANDED** → **P2.2 schematic render in pane B — LANDED**
   (multi-scene + symbol structure, then re-specced 2026-07-10 to match
   `schematic-editor.html` and completed 2026-07-10: P2.2c per-element colour /
   P2.2d interactive focused-pane camera / P2.2e typed-object geometry (bus, power,
   label kinds) / P2.2f square grid + frame removal; render fidelity reviewed
   complete-to-spec, shell golden re-blessed; see `DATUM_GUI_PHASE_2_SPEC.md` P2.2)
   → **P2.3 cross-probe — NEXT** (one selection identity projected into both panes).
   Governing: decision 019 + **decision 021 (pane tiling)** +
   `DATUM_GUI_PHASE_2_SPEC.md` on `DATUM_GUI_PHASE_1_SPEC` + `DATUM_GUI_CONFORMANCE_SPEC`.
2c. **Universal Editor-Interaction & Viewport Toolkit — SPEC LANDED, build staged
   (governed by decision 023 + `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`;
   the Layer-1 component of `docs/DATUM_SHARED_TOOLING_TAXONOMY.md`, the controlling
   four-domain catalogue of Datum's full shared editor tooling — Layer 0 substrate
   unified, Layer 1 = this, Layers 2–4 = selection-identity/property-inspector/
   one-Measure/geometry-solver-library/etc. as future per-capability specs).**
   The schematic grid rendered divergently from the board grid (weights thicken on
   zoom); investigation found the whole per-viewport interaction class (tool-mode,
   hover, selection, marquee, context menu, coordinate readout, cursor, snap,
   keybinding) is board-only or absent for the schematic, funneling through two
   board-only chokepoints — a structural per-editor fork of shared tooling, the Lean
   anti-pattern. Decision **023** ratifies one consumer-side backbone every surface
   *configures* via a `ViewportProfile` (grid/camera/coord-hit/snap/stroke-weight/
   hover/selection/tool-mode/context-menu/readout/layer-visibility); the governed
   spec fixes the weight-class table + the min-px floor bug + a unified 20px LOD knee,
   the two-tier snap resolver, and quantize-to-grid as align `reference: grid`.
   *Sequencing (spine):* **S0** init `gui-viewport` crate + StrokeWeightModel → **S1**
   unify GridEngine, screen-constant weight, both panes (**fixes the grid bug**) →
   **S2** CameraEngine routing collapse → **S3 CoordinateHit keystone** (per-pane
   hit-test; schematic hit regions) → {S4 hover, S5 selection+marquee, S6 tool-mode,
   **S7 context menu** (per-surface, verb-firing — the "local menu works across
   editors" ask), S8 readout, S9 layer-visibility} → S10 SnapEngine → S11 quantize
   verb. *Dependency:* P2.2 landed; S7 authoring verbs + S11 ride the write-path step.
   *Unblocks:* **P2.3 cross-probe rides on S3+S5** (schematic selection/hit-test);
   native authoring (step 6) reuses snap/commit; paper-space snap (step 7) reuses the
   SnapEngine. *State:* **decision 023 + governed spec + deep-research sections
   LANDED. Phase-B build in progress (schematic-first, each slice board-golden-safe +
   source-health-ratcheted): **S0–S3 LANDED status RESTORED after production
   correction.** S0 routes active authored primitives through explicit semantic
   weight policies and resolves minimum pixel floors from the live GPU projection.
   S1 emits only visible, overflow-safe, capped grid geometry and retains governed
   LOD hysteresis independently for every pane/content identity. S2 owns typed warm
   cameras through the shared CameraEngine and routes pointer versus focused commands
   without surface fallback. S3 uses the shared indexed EditorViewport with typed
   schematic hit metadata, deterministic query budgets, and one independently
   projected/rendered surface pass per visible pane. **S4 (hover + crosshair) — LANDED
   status RESTORED after production correction:** cursor/hover refresh only dedicated
   post-world interaction buffers while both retained scenes and `PreparedScene` stay warm;
   screen coordinates and hover ownership are typed; identifier-prefix inference is removed;
   CursorLeft, focus loss, terminal capture, and modal drags clear transient state; and a
   zero-retained-resolve/static-buffer regression covers pointer refresh. Hover/cursor policy
   and state construction now live in `gui-viewport`. Pointer previews target the containing
   pane while command/tool gestures target the focused pane, as clarified in UVT-004.
   The hit path is spatially indexed and covered by deterministic large-design and
   candidate-budget regressions. **S5 specification design is IN PROGRESS; build
   remains unauthorized:** owner-ratified gesture/selection/compound behavior is
   being captured in `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2; the broader
   attribute audit is durably tracked in
   `research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md` through
   `DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md`; and the researched delivery
   boundary is S5A selection+compound inspection → S5B persistent-group/universal-
   lock/typed-batch authority → later domain tools. Final review plus a numbered
   selection-identity decision are required before execution. The incomplete
   per-object/state selection glow contract is tracked by
   `dat-s5-selection-visual-contract-zid`, with its audit integrated through
   `research/selection-visual-language/SELECTION_VISUAL_LANGUAGE_RESEARCH.md` and
   `DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md`; owner build-out is in progress.
   The foundational construction is now locked: slight whole-owned-geometry
   brightening with semantic/material hue retained, plus `#CE5A92` internal glow
   and a crisp object-shaped 2-physical-pixel screen-space cue. Actual selection
   now also projects at identical full strength in active/inactive workspaces;
   pane frame/header/tool enablement alone communicates GUI mutation authority.
   Triple-click Global Net is one semantic selection subject whose complete
   visible resolved electrical projection glows across schematic/PCB, while
   connected parent symbol/footprint bodies remain related rather than selected.
   Related objects retain exact authored appearance; explicit relationship view
   may mildly dim unrelated context but never brightens/recolors/glows the related
   object or reuses the selection accent.
   Optional compound focus has no stronger canvas styling; Inspector identifies
   it and reference-requiring commands own a temporary marker.
   Locked objects are slightly neutral-greyed, retain normal selection, suppress
   handles, and use a selected/hovered anchor padlock only after the glyph is
   declared in `icon_set.json`, added to the Rendering Study contact sheet/style,
   and HUMAN-reviewed; dense compounds rely on Inspector locked counts.
   Proposal/diagnostic collision order is authored base → proposal ghost/dual
   stroke → selection cue → topmost semantic diagnostic marker; selecting a
   proposal/finding preserves its uncommitted/severity identity.
   The global bottom
   strip is now canonically named the **Application Status Bar**; its information
   role has completed focused proximity/attention research in
   `research/application-status-bar/APPLICATION_STATUS_BAR_RESEARCH.md` through
   `DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md`; retention and contents are
   reopened for owner review rather than assumed from the prototype. Then: S6 tool-mode →
   **S7 context menu** → S8 readout → S9 layer-vis →
   S10–S11 snap/quantize.** Governing: decision 023 +
   `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` on decisions 014/020/021/022.
3. **Marking-menu shell — read-only, rendered from `menu_model.json` (buildable
   today).** Build the radial marking-menu / context-menu surface realizing
   `docs/gui/prototypes/context-menu-marking-menu.html`, rendered *from* the
   `menu_model` manifest (same data the menu bar uses): cardinal/secondary/overflow
   layout, submenu wheels, icons from `icon_set.json`, per-object content per
   `docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md`. **Inert in this step** — items
   render and preview their gesture but do **not** invoke operations; `not_built`
   and mutating items are visibly disabled, exactly as Phase 1 handles the menu
   bar. This proves the interaction model and IA without the write path. **Ops
   wiring** (marking-menu items → journaled operations) is deferred to step 5.
   Governing: decision 019 + `DATUM_GUI_CONTEXT_MENU_CONTENT.md` +
   `DATUM_GUI_PARAMETRIC_TOOLING.md`.
4. **Command console — the editor command line (AutoCAD/Eagle command-echo).**
   The visible home for GUI-action narration, split by write-path dependency:
   **(a) read-only command-echo console (buildable now, on the Phase-1 shell):**
   the surface that displays GUI-action echoes (fit board, layer toggle, selection,
   view zoom, ...). The narration is **already decoupled** from the real PTY terminal
   into an invisible console sink (`ConsoleLaneState` on `WorkspaceUiState`, fed by
   `log_review_event` → `push_console_line`); this step gives that sink its visible
   surface. Needs **no** write-path. **(b) authoring command console (gated on the
   write-path, step 5):** typed mutating verbs entered at the command line, emitting
   journaled typed `Operation`s exactly as the marking menu and menu bar will. The
   integrated terminal stays a real shell that GUI actions never write to (decision
   005). Governing: decision 019 + decision 005 (embedded-terminal doctrine) +
   `docs/gui/DATUM_GUI_DESIGN_SPEC.md` § "Command Surfaces".
5. **GUI write-path enablement** — the four-item backend plumbing that lets the
   GUI author journaled operations directly (and wires the step-3 marking menu to
   real ops). Plan already written: `docs/gui/DATUM_GUI_WRITE_PATH_PLAN.md`
   (decisions 019 + 017), sequence P0 → W1 → W2 → W3. Needed once *authoring*
   surfaces are built (after the read-only Phase 1 + marking-menu shell), NOT
   before. State: **planned; not the immediate step.**
6. **Native authoring depth (queued):** schematic and PCB editor surfaces
   emitting typed operations over the write path. Contracts:
   `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`,
   `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`. Depends on steps 1–5.
   **Named engine dependency:** the **DFM Geometry Solver** (pad rounding · trace
   corner treatment · teardrops — author topology, derive manufacturable geometry
   rule-driven per net-class through `commit()`), governed by
   `docs/gui/DATUM_RENDER_FIDELITY_AND_DFM_GEOMETRY.md` under its two invariants
   (Law 1 render/CAM single-source fidelity, gated; Law 2 beauty-by-default). Sits
   on the landed routing kernel + the deferred `ImpedanceSpec` solver + the
   forthcoming Datum Rendering Book. Execution requires authorization.
7. **Documentation system (design landed, spec queued):** paper space / model space
   separation + **viewports** (decision 020) with the title block (Rendering Book §8) as
   the paper-space frame. The design foundation is locked — content foundation,
   Direction-B visual language, Grauwert typography, the dimensioned/proportional-scaling
   guideline, and the field-formula + doc-control template architecture
   (`research/documentation-system/`). **Next:** a spec pass for the
   `Sheet` / `Viewport` / annotation / `SheetSet` object model + v1 scope (schematic paper
   space vs the fab-drawing sheet template first). Pairs with native authoring (step 6) —
   it is how the authored model becomes documentation. Governing:
   `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` + Rendering Book §8.

**Also in flight (governance, parallel):** `specs/PROGRAM_SPEC.md` reconciled to
the product-mechanics model (2026-07-06); Tier E `GuiSupervisionSnapshot` strip
to be rescoped per the leverage audit (keep EDA counts as a status bar, drop
journal-tip/read-only provenance telemetry). **Shell visual-parity gate (landed):**
`scripts/check_gui_visual_parity.py` makes Phase-1 whole-shell composition parity a
real failing, same-engine regression gate against the owner-approved shell golden
`crates/gui-render/testdata/golden/shell/datum-shell.golden.png` (wired through
`check_gui_conformance.py` → `run_drift_gates.sh`), closing the paperwork defect
where visual parity was an unenforced HUMAN row; see
`docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md` §0.1/§2/§4 (G9). **Source-health
governance recovery (landed):** decision 022 supersedes the
permissive 1,500-line flag ledger with blocking repository-wide normal budgets,
zero-headroom legacy ceilings, merge-base downward ratchets, touched-monolith
burn-down, and logical-module measurement; see Active Frontier step 0. **GUI
Phase 2 spec (governed, spec-only):**
`docs/gui/DATUM_GUI_PHASE_2_SPEC.md` sequences the second GUI build phase —
populated component inspector + dual-pane Board+Schematic + cross-probe — extending
`DATUM_GUI_PHASE_1_SPEC` under the `DATUM_GUI_CONFORMANCE_SPEC` check-disposition
discipline; P2.0 (populated single-pane inspector via the `--select` capture
repoint) has landed and is ENFORCED by `check_gui_visual_parity.py`, the P2.1
split-view first slice (two-pane LAYOUT + per-pane headers + focus + placeholder
pane B) has landed and re-blessed the shell/board goldens, and P2.2–P2.4 are
specified with build deferred to authorized execution. Ordering lives in the
Active Frontier (step 2b); this row is status only.

---

Last updated: 2026-07-06 — GUI product-model recovery planning opened under
decision 019. `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`
ratifies the recovered desktop product model, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`
is the single governed GUI product spec, and
`docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` classifies the existing GUI code as
keep/adapt/replace/delete/missing before further implementation. Tier E GUI
supervision/status reflection remains implementation evidence, but it is no
longer the active product target. The active GUI target is now Phase 1
application shell + board render fidelity: the `datum-test` fixture with real
footprint/pad/track geometry and screenshot-golden review. Earlier
write-surface convergence remains COMPLETE (wave commits `dff7c2c..c567698`):
the engine native-write facade (`crates/engine/src/api/native_write/`, 11
families) authors every native operation batch, genesis is engine-owned, the
daemon reaches the substrate via `native.write`/`native.describe`, and the CLI
flat namespace was dissolved into `args/` + `commands/<family>/` + `context/` +
`main_tests/` with the exec layer deleted. (2026-07-02: governance apparatus
slimmed to behavioral gates plus spec-governance coverage/classification.)
(2026-07-09: GUI-action narration decoupled from the integrated PTY terminal —
`log_review_event` now feeds an invisible `ConsoleLaneState` sink via
`push_console_line` instead of the terminal display buffer, honoring the decision
005 doctrine that the terminal is a real shell GUI actions never write to. The
command console that will make this sink visible is sequenced as Frontier step 4.)

**Current-vs-target framing**:
- **Current implementation evidence**: the historical milestone tables below
  remain truthful records of the implementation slices that have landed.
- **Active target**: GUI product-model recovery planning and Phase 1 app
  shell + board render fidelity, without claiming editor readiness.
- **Not the North Star**: legacy M0-M7 milestone completion rows are retained as
  evidence, but they no longer define the next implementation priority.

**Active milestone**: GUI product-model recovery planning. Decision 019
supersedes the M7 route-review shell as the GUI planning target and anchors a
single product spec plus one code-leverage audit. The first implementation
phase is an application shell and board-render fidelity baseline that a human
can run, load, inspect, and review against screenshot goldens.
**Active product driver**: restore Datum as a human-driven desktop EDA
application while preserving the substrate rule that GUI edits emit typed
operations/proposals through engine commit/journal paths, not terminal-injected
CLI macros.
**Frozen**: M6 (strategy reporting layer landed; pending repeated evidence
runs from the checked-in baseline gate).
**Closed for scope**: M0–M5.
**Spec stubs awaiting implementation**: Standards Audit Batch 1 — see
section "Standards Audit Batch 1 — Spec Stubs Awaiting Implementation"
below.

Machine-checked inventory shapes live in `specs/SPEC_PARITY.md` (gated by
`scripts/check_spec_parity.py`, wired into `scripts/run_drift_gates.sh`).
Surfaces currently locked: `mcp_runtime_methods`, `cli_project_commands`,
`engine_text_modules`, `m7_text_visual_fixtures`, `workspace_crates`,
`daemon_dispatch_methods`, `engine_api_pub_fns`, `standards_check_surface`,
`pool_library_surface`, `erc_pin_taxonomy_surface`,
`schematic_connectivity_surface`, `zone_fill_surface`,
`gui_supervision_surface`.

---

## Spec Governance Coverage

Every specification that steers development is classified in
`specs/spec_governance_manifest.json` and enforced by
`scripts/check_spec_governance.py` (wired into `scripts/run_drift_gates.sh`).
The gate imposes two obligations: (1) **coverage** — no spec, contract, or
decision record may exist unclassified; (2) **classification honesty** — every
entry must be `governed`, `doctrine`, `pending`, or `historical`, and a
historical doc may not self-declare active. Behavioral enforcement and explicit
gaps live in proof gates, parity inventories, tests, and this tracker rather
than as required per-doc manifest blocks. This is the machine-checked backstop
for the CLAUDE.md "Specification Governance (controlling)" rule.

Verified enforcement of every previously-untracked spec (the manifest holds the
full per-reference detail; `progress_anchor` is the path/name below):

| Spec | Verified enforcement | Honest gap |
|------|----------------------|------------|
| `specs/STANDARDS_COMPLIANCE_SPEC.md` | `PG-STANDARDS-REPAIR-PROPOSALS` + `standards_check_surface` parity | §4 disposition table ungated; named IPC/IEEE numbers not mechanically substantiated |
| `docs/IPC_FOOTPRINT_SYSTEM.md` | pool/import footprint tests + engine IPC generator tests + CLI/proposal IPC generation tests + MCP taxonomy/dispatch tests | First engine-owned IPC-7351B two-terminal chip and SOIC generators, structured basis, journaled CLI generation, proposal generation, MCP surfaces, and LibraryGraph IPC basis diagnostics landed; SOT/QFN/BGA/etc. families, check-run findings, governed mask/paste-in-export, deviations, and import audit remain unbuilt/ungated |
| `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md` | `PG-STANDARDS-REPAIR-PROPOSALS` | IPC-7351B structured-basis generator doctrine-only |
| `docs/contracts/RULES_CHECKS_TOOL_CONTRACT.md` | `PG-CHECKRUN-GENERATED-EVIDENCE` + `standards_check_surface` parity | four-tool set + rules-shard migration + revision-keyed ERC ungated |
| `docs/SYMBOL_LIBRARY_IMPORT_SPEC.md` | pool-substrate engine/CLI tests + spec-governance coverage classification | no spec-specific normalization test; skeleton import only |
| `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md` | `check_pcb_layout_tool_matrix` + `check_mcp_public_taxonomy` + `check_daemon_write_parity` + `PG-ZONEFILL-SUBSTRATE` | richer general polygon pour solver remains bounded; interactive GUI editor remains pending |
| `docs/contracts/MANUFACTURING_OUTPUT_TOOL_CONTRACT.md` | `check_schematic_private_writers` + `PG-OUTPUT-JOB-RUN-REPLAY` | three-surface lean thesis + T0 unification + panelization ungated |
| `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` | `check_mcp_public_taxonomy` + `check_daemon_write_parity` + `mcp_runtime_methods` parity | DatumToolSession + generic commit() coverage + unified Proposal not-yet |
| `docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md` | `PG-UI-LAYOUT-INVARIANTS` + scale/layout unit tests + checked-in goldens | PNG golden image matrix in CI and broader populated-state coverage remain open |
| `specs/ERC_SPEC.md` | engine + CLI ERC tests + `check_erc_connectivity_parity` + `erc_pin_taxonomy_surface` parity | remaining target ERC result-shape and passive-only rule gaps are documented; finding revision keys are not complete across all library object revisions |
| `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` | engine + CLI connectivity tests + `check_erc_connectivity_parity` + `schematic_connectivity_surface` parity | target graph shape and future diagnostics beyond the shipped read surfaces remain partial |
| `specs/IMPORT_SPEC.md` | `check_import_query_determinism` (CI) + `PG-IDENTITY-SUBSTRATE` | feature matrices / round-trip ungated (import frozen) |
| `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` | spec-governance coverage + decision 019 authority | implementation not started; Phase 1 app shell / board-render fidelity acceptance and screenshot-golden matrix remain future work |
| `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` | spec-governance coverage + decision 019 authority | audit classifications steer follow-on implementation, but no code migration is complete until each keep/adapt/replace/delete item is tied to commits/tests |

Decision records `000..016` are `doctrine` — most are **enforced** by named
proof-gates/substrate gates (see the manifest's per-record `enforced_by`); the
genuinely thin ones carry an explicit gap.

Enumerated enforcement gaps (the honest worklist — these steer development but
no gate reconciles them with code):
1. **UI layout invariants** (contract + decision 014): CLOSED — now gated by
   `PG-UI-LAYOUT-INVARIANTS` (shell validity, column non-overlap, hit-region
   containment across the 1.0–2.0 scale matrix). Remaining: the PNG golden
   image matrix in CI and broader populated-state coverage.
2. **IPC-7351B footprint generator / structured basis** (010, IPC doc):
   two-terminal chip and SOIC engine generators, CLI/proposal/MCP authoring
   surfaces, and LibraryGraph validation diagnostics landed; SOT/QFN/BGA/etc.
   families, check/export consumption, deviations, and import audit remain
   open.
3. **STANDARDS_COMPLIANCE §4 disposition table**: ungated prose.
4. **ERC / connectivity specs**: CLOSED for pin-taxonomy and shipped
   diagnostic/code drift — `check_erc_connectivity_parity` plus spec-parity
   inventories now lock the canonical ten-type `LibraryPinElectricalType:v1`
   alias, ERC emitted code list, and shipped connectivity diagnostics.
   Remaining work is product depth: target ERC result-shape promotion,
   complete revision keys across library object revisions, passive-only net as
   its own finding if retained, and future connectivity diagnostics.
5. **Manual-editor GUI workflow (002)** and **workspace UX (007)**: substrate
   gated, UX not. Decision 019 opens the GUI product-model recovery plan; Phase
   1 shell/render-fidelity implementation and visual-regression gates remain
   open.
6. **`OPEN_QUESTION_RESOLUTIONS`**: unenforced ratifications log; consolidate
   into numbered decision records.

Closing these gaps (a dedicated gate/test per item) is the remaining governance
work; until then each is visible and gate-required to stay declared, not silent.

---

## Scope Integration / Substrate Readiness

This is the active tracking surface for the new scope. Items here should move
from target to current evidence only when there is an implementation anchor,
test/gate, and CLI/MCP/API surface where applicable. Imported-board fidelity can
continue only after the relevant substrate contracts are implemented or after an
explicit governance decision records why a fidelity slice is exempt.

### Native Write Facade — Write-Surface Convergence (2026-07-04, COMPLETE)

Evidence: wave commits `dff7c2c..c567698` (facade scaffold `22a1b0f`, Waves
1–4 `3d597cc`/`319ceef`/`f65314d`/`06a2f80`, daemon substrate reach `1c93ca6`,
legacy fence + MCP provenance `11f74bb`, CLI reorganization `57e2a07..c567698`).

| Item | Status | Notes |
|------|--------|-------|
| Engine native-write facade | [x] | `crates/engine/src/api/native_write/` (11 families over 4 waves): ALL CLI operation authoring moved engine-side; the CLI authors zero operation batches. Route-proposal domain logic lives in `crates/engine/src/board/route_proposal/`; fixture seeding goes through the facade (last raw `board.json` write gone). Remaining CLI `write_canonical_json` call sites are user-addressed artifact/report exports (route-proposal and forward-annotation artifacts), not project source shards. |
| Engine-owned genesis | [x] | `bootstrap_native_project` in `native_write/genesis.rs`; the `project new` CLI bootstrap bypass is closed. Decision 018 ratifies the genesis provenance boundary: genesis is deliberately not journaled, has no t=0 mutation-journal record, and any future visible genesis evidence must be a non-mutation sidecar. |
| Daemon substrate reach | [x] | `native.write` + `native.describe` (registry-driven, resolve-per-request, `tool`/`assistant` provenance). Legacy fence executed: 11 imported-board dispatch arms retired, the imported-KiCad converter session (4 legacy mutators + `Engine::save`, private `ImportedSessionUndoRecord` memo) is terminally frozen per decision 011. MCP provenance closed (`cli_commit_source` everywhere, `assistant` default). |
| CLI reorganization | [x] | Flat namespace dissolved into `args/`, `commands/<family>/`, `context/`, `main_tests/`; exec layer deleted (273 `run()` impls collapsed into single dispatch); `main.rs` at 194 lines; orphan-coverage gate added. |
| Tier A close-out | [x] | ComponentInstance bind/set/delete now have proposal twins (`datum.proposal.bind_component_instance`, `.set_component_instance`, `.delete_component_instance`) so assistant-provenance callers can create governed proposals instead of direct writes. Decision 018 closes the genesis t=0 owner decision. Public verb-registry migration remains complete with 17 public `datum.*` prefixes generated. |
| Tier B library foundation | [x] | `LibraryBinding` is now a Rust type carried by authored `ComponentInstance` shards, with resolver/commit validation and backward-compatible `part_ref` projection. IPC-7351B generation now has two engine-owned vertical slices: two-terminal chips and SOIC, both exposed through journaled CLI writes, proposal creation, MCP aliases, generated registry/catalog entries, and LibraryGraph basis/process-policy validation. A Datum-authored native baseline fixture under `crates/test-harness/testdata/library/native_authored_baseline_v1` proves governed native pool data for Unit, Symbol, Entity, Package, Footprint, Padstack, Part, and first-class PinPadMap without ratifying a bundled-library packaging decision. Public tool count was 337/337 after the SOIC library/proposal tools were added; Tier D raises it to 338 with `datum.pcb.align_components`. |
| Tier D PCB/manufacturing depth | [~] | The PCB 13-tool logical matrix is now machine-gated by `scripts/check_pcb_layout_tool_matrix.py`: it maps the 46 concrete `datum.pcb.*` verbs plus `datum.check.fill_zones`/`datum.check.run` back to the contract buckets, requires `datum.pcb.align_components` as one mode-parameterized batch tool, rejects per-mode align/distribute tools, and keeps `fill_zones` as derived generated evidence. Engine `build_align_board_packages` emits multiple `SetBoardPackagePosition` operations in one guarded `OperationBatch`; CLI/MCP parity tests prove locked-component skip reporting and one undoable batch. Native zone fill remains the bounded generated-evidence solver already shipped (`SetZoneFill`/`DeleteZoneFill`, Unfilled/Stale/Unsupported honesty); the full polygon-boolean/thermal/antipad pour solver remains future product depth. |
| Tier E GUI supervision/status surface | [~] | Native GUI board scenes now load through `ProjectResolver::resolve()` plus `DesignModel::materialized_source_shard_value(BoardRoot)` instead of raw promoted `board/board.json`, so journal-materialized board state is visible even when the promoted shard is stale. `ReviewWorkspaceState` now carries `datum_gui_supervision_snapshot_v1`, summarizing project identity, model revision, journal cursor/tip, source-shard health, scene object counts, check status, and production/proposal/artifact counts. The Outputs data lane renders a read-only `ENGINE SUPERVISION` block from that snapshot. This is supervision/status reflection only: GUI authoring remains terminal-mediated or future work, and the GUI does not construct or commit `OperationBatch` directly. |
| GUI product-model recovery planning | [~] | Decision 019 ratifies the recovered GUI product model and makes `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` plus `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` the governed planning surface. The current implementation is still the M7-derived review shell; the next implementation phase is Phase 1 application shell + board-render fidelity with the `datum-test` fixture, real footprint/pad/track geometry, screenshot goldens, and owner review. GUI editor actions must emit typed engine operations/proposals directly through the substrate; terminal command injection is explicitly disallowed as an editor implementation path. |
| GUI menu→mechanism bindings | [x] | `docs/gui/DATUM_GUI_MENU_BINDINGS.md` (governed) maps every product-spec menu command to its real backing verb/native-write builder/CLI, tagged live / engine-ready-GUI-blocked / not-built, from a four-source capability inventory (verb registry, native-write facade, CLI, daemon + tool surface, 2026-07-05). Records the File-menu semantics gap (no native save/open/close) and authoring gaps (schematic wire/junction edit, route-apply facade migration, forward-annotation action coverage). |
| GUI menu_model manifest + gate | [x] | `docs/gui/menu_model.json` — data-driven menu bar + per-object marking menus (149 entries), each bound to a real `datum.*` verb / `gui_local` / `not_built`. Gated by `scripts/check_menu_model.py` (wired into `run_drift_gates.sh`): every `verb` reference must exist in the registry catalog; marking-menu structural invariants enforced (cardinal N/E/S/W, secondary diagonals, destructive never on a diagonal). 101 verb-backed, 36 not-built (= the authoring buildout worklist). Realizes the design-spec Modularity principle (add/remove a row, not rewrite); full per-object content in `docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md`. |
| GUI write-path enablement (four-item plumbing) | [ ] | `docs/gui/DATUM_GUI_WRITE_PATH_PLAN.md` (governed) bounds the plumbing that unblocks every engine-ready/GUI-blocked authoring item: W1 register remaining native-write families with `native.write` (7→full set), W2 add a `Dispatch::NativeWrite` variant to the verb registry (decision 017), W3 expose verb/param-schema enumeration on the daemon (`specs/MCP_API_SPEC.md`), W4 give `gui-app` a daemon client + GUI action model (decision 019 Editor Authority). Sequenced P0 thin proof (rename via the 7 already-wired verbs) → W1 → W2 → W3. Planning only; execution unauthorized. |
| Rendering Book — house visual identity (symbols/footprints/silk/icons) | [~] | `docs/gui/DATUM_RENDERING_BOOK.md` (governed) extends decision 015 tokens into manufacturable content geometry. **Locked (owner-approved design pass 3, prototype `docs/gui/prototypes/rendering-study.html`):** board-editor canonical palette; schematic dark-working-default + vellum print/doc toggle; filled symbol bodies + stroke hierarchy + filled net-label pills + screen-only selection glow; rounded-rect pads (25% ratio) + pad-1 + courtyard + cyan dimension overlays; DFM defaults (acute→small inner fillets, right-angle→miter, teardrops); silk typeface = IBM Plex Sans Condensed as filled outline (screen==CAM). **Symbol standard locked: IEC rectangular** (Fork B closed 2026-07-08). **IBM Plex engine wiring done:** `crates/engine/src/text/registry.rs` vendors Sans Condensed Regular/Medium/SemiBold + Mono Regular/Medium, all intents resolve to IBM Plex, provenance recorded, manufacturing gerber goldens refreshed; engine/cli/gui-render/gui-app/gui-protocol/daemon suites green. **Open:** sheet borders/title block, full stackup palette, broader families (next passes). Captures the "Datum visual identity" item from Active-Frontier step 1. |
| Gerber text-export compaction (G36/G37 regions + G75 biarc arcs) | [ ] | `research/pcb-text-rendering/OUTLINE_TEXT_GERBER_COMPACTION_RESEARCH.md` (governed research). Root cause of huge outline-text gerbers (~190 KB for 4 chars) is the **export encoding** — scanline-trapezoid perimeter-stroking with no `G36/G37` regions and no `G75` arcs — **not** the flatten tolerance, which stays high-fidelity per the `outline.rs:270` doctrine (renderer never coarsened for fab reasons). Doctrine-faithful fix, renderer untouched: emit one `G36/G37` region per glyph contour with curved edges as `G75`+`G03/G02` arcs from **biarc-fitting the original glyph Béziers**; adaptive-segment fallback gated by fab export policy. Closes the arc/biarc deferral the Phase-2/3 text research left open. Design documented + tracked; **execution unauthorized** (a real engine build task). |
| Documentation system — paper space + viewports (decision 020) | [~] | `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` (ratified doctrine, 2026-07-08). Datum separates **model space** (authored schematic/PCB/3D/panel) from **paper space** (sheets carrying the title block + viewports + annotations). A **viewport** is a live, scaled, cropped, layer-filtered projection of a model asset (render==CAM, no mutation); paper-space objects (sheet/viewport/dimension/note) are typed ops through commit()/journal. **All documentation output** — schematic sheets, fab/assembly/drill/panelization/cover sheets — becomes *sheets with viewports + title block*; document-types are sheet templates. Enables composed cover sheets, multi-schematic pages, schematic **detail** viewports, and panelization scaled to fit Letter showing V-score/mousebite. Design foundation landed: title block (Rendering Book §8 LOCKED), field-formula/doc-control model + template architecture (`research/documentation-system/`), Grauwert typography (§5). **Next:** spec the `Sheet`/`Viewport`/annotation/`SheetSet` object model + v1 scope (schematic paper space vs fab-drawing template first). Design/decision documented + tracked; spec + execution future. |
| Render fidelity + DFM Geometry Solver (thesis + future engine) | [ ] | `docs/gui/DATUM_RENDER_FIDELITY_AND_DFM_GEOMETRY.md` (governed). Ratifies Law 1 (render/CAM single-geometry-source fidelity — one manufacturable geometry consumed by both the renderer and the Gerber/drill/paste/assembly exporter, enforced by a fidelity gate; presentation overlays fenced out) and Law 2 (engine-drives-visual / beauty-by-default). Names the DFM Geometry Solver — pad rounding · trace corner treatment (acute→chamfer, right-angle→miter/radius per net-class, incl. Douville–James impedance miter) · teardrops — as author-topology/derive-manufacturable-geometry, rule-driven through `commit()`, `capability = parameter of a small verb set`. Active-Frontier step-6 dependency on the routing kernel (landed) + `ImpedanceSpec` (deferred) + the forthcoming Rendering Book. Foundation for the Rendering Book; the fab "fingerprint" thesis (filled-outline silk typeface vs Eagle stroke-font; DFM-optimal copper). Design/planning only; execution unauthorized. |

### Single-Source Verb Registry (Decision 017)

`docs/decisions/PRODUCT_MECHANICS_017_VERB_REGISTRY.md` — the user-facing verb
surface (MCP tool name == GUI terminal command id) is declared once in the
leaf crate `crates/verb-registry` and projected into the checked-in
`mcp-server/datum_tool_catalog.json` (generated/gated by
`datum-verb-catalog --write|--check` in `scripts/run_drift_gates.sh`); the MCP
catalog merges generated families through `MIGRATED_PREFIXES` in
`mcp-server/tools_catalog_generated.py`, and argv templates are behaviorally
locked against the real CLI by `crates/cli/tests/verb_registry_roundtrip.rs`.

| Item | Status | Notes |
|------|--------|-------|
| Registry crate + generated catalog + drift gate + clap round-trip test | [x] | Landed; registry declares all 338 public verbs; seventeen public prefixes are registry-owned in the MCP catalog with their hand-written dicts deleted or filtered (`datum.artifact` 11, `datum.check` 10, `datum.component_instance` 3, `datum.context` 4, `datum.journal` 4, `datum.library` 55, `datum.manufacturing` 6, `datum.output_job` 5, `datum.pcb` 46, `datum.pool` 3, `datum.project` 3, `datum.proposal` 48, `datum.query` 48, `datum.replacement` 5, `datum.route` 21, `datum.schematic` 62, `datum.session` 4 — 338 tools); public tool set intentionally grew by the ComponentInstance proposal twins, IPC SOIC library/proposal tools, and Tier D `datum.pcb.align_components` |
| All public `datum.*` verbs registry-owned; hand-written MCP public catalog entries deleted | [x] | 17 public `datum.*` prefixes migrated via `MIGRATED_PREFIXES` (338 of 338 public tools generated); hidden compatibility tools remain fenced outside the generated public loader; taxonomy/count gates keep their invariants |
| CLI clap / daemon dispatch / GUI terminal catalog generated from the registry | [~] | GUI terminal catalog is now a registry projection (`datum-gui-protocol` renders the 50 `terminal` verbs; hand-written table and regex parity test deleted, templates locked byte-identical by gui-protocol tests + clap round-trip); CLI clap and daemon dispatch remain target projections |

### Next Production Goals — Current / Target Ledger

This ledger is the active parity surface for the next library/schematic
foundation goals. Claims here are intentionally partial unless code evidence,
tests, and public surfaces exist.

| Goal | Current Evidence | Target / Non-overclaim Boundary |
|------|------------------|---------------------------------|
| Canonical pin electrical type | [~] `schematic::PinElectricalType` is now the pool-owned `LibraryPinElectricalType` alias, so schematic pins and ERC consume the library taxonomy rather than a parallel schematic enum. The canonical roles are Input, Output, Bidirectional, Passive, PowerIn, PowerOut, OpenCollector, OpenEmitter, TriState, and NoConnect; focused ERC tests lock their current driver/conflict semantics. ERC pin-backed findings now carry `pin_evidence` with canonical pin type, taxonomy revision, symbol UUID/reference, library ID, unit selection, and available part/entity/gate bindings; CheckRun evidence projects that into an `erc_pin_taxonomy` evidence object. | One library-owned canonical taxonomy drives schematic materialization and ERC without a parallel schematic authority enum, and persisted findings expose enough revision/taxonomy context for stable audit. |
| Symbol pin style | [~] `SymbolPinAnchor` now carries orientation, optional length, and a bounded decoration enum; typed symbol pin-anchor authoring preserves unit-pin UUID and position. | Symbol style must cover the full decision-008/KiCad import vocabulary, including active-low/alternate-function style semantics where distinct from current decorations. |
| Symbol schema convergence | [~] Engine `Symbol` now has fields, default refdes prefix, style-profile assertions, standards basis, check state, provenance, drawings, and pin anchors. Rendering, checks, and importer/exporter parity for the richer schema remain partial. | Decision-008 symbol schema is the single engine-owned schema consumed consistently by CLI/MCP/GUI/import/export/check surfaces. |
| `PinPadMap` authority | [~] `pool/pin_pad_maps` are resolver-visible, validate part/footprint mapping refs, and now have typed CLI authoring via `create-pool-pin-pad-map` / `set-pool-pin-pad-map`; `Part.default_pin_pad_map` can be set during map creation. `PinPadMap.mappings` are gate-aware `pad -> {gate, pin}` bindings, so repeated unit pins across multi-gate entities remain distinct. Runtime component replacement, package-change compatibility signatures, and pad-net remapping now prefer a valid `Part.default_pin_pad_map` resolved through `Pool.pin_pad_maps`. Legacy `set-pool-part-pad-map*` commands now bridge legacy-shaped inputs into the part default first-class `PinPadMap` and do not write `Part.pad_map`; focused regressions prove missing or stale `default_pin_pad_map` rejects the legacy command without mutating `Part.pad_map`. `LibraryGraphValidationReport` now classifies legacy `Part.pad_map` rows as migratable, shadowed by an authoritative default `PinPadMap`, or blocked by malformed/ambiguous references. `Part.pad_map` remains read/import compatibility data for existing payloads, not a new write authority. | `PinPadMap` is the first-class binding authority; `Part.pad_map` is retired or strictly import/compatibility input behind migration policy. |
| Padstack depth | [~] `Padstack` now models aperture, drill, plated state, layer span, mask/paste policy, annular ring, thermal, and anti-pad fields with validation slices. | Padstack policy is fully consumed by footprint generation, board materialization, DRC/standards checks, and fabrication exports. |
| `LibraryGraph` ownership | [~] `eda_engine::pool::LibraryGraph` now owns pool object registration, duplicate/shadowing diagnostics, dependency diagnostics, validation tiers, validation summaries, and legacy `Part.pad_map` migration reports consumed by validation callers. Constructors, MCP projection, resolver materialization rules, pool layering, and complete dependency policy are still not a single graph-owned contract. | One engine-owned `LibraryGraph` owns dependency resolution, duplicate/shadowing policy, validation tiers, and materialization rules for CLI/MCP/GUI. |
| `ModelAttachment` status | [~] Part behavioural model attachments are typed, hash/provenance-backed, review-state capable, journal-authorable, and pool-model blobs are query/validate/gc visible. Package 3D `ModelRef` and cross-target governed `ModelAttachment` are not unified. | Model attachments are governed library data across Part/Package/Footprint targets with role/format/hash/provenance/review semantics and lifecycle policy. |
| Schematic place-symbol | [~] Native `place-symbol` routes through journaled schematic operations, has canonical MCP `datum.schematic.place_symbol`, has proposal creation, materializes pins from pool symbol UUID `lib_id`, and mints an authored symbol-first `ComponentInstance` when that pool symbol resolves to a unique entity/gate/compatible part. Binding status now distinguishes compatibility `lib_id`, unresolved entity/gate, ambiguous entity/gate, bound-without-part, ambiguous compatible parts, incompatible parts, and bound-with-part, with revisioned binding evidence for every resolved entity/gate binding. Sheet, sheet-definition, and sheet-instance mutation reports now expose `model_revision`, `created_ids`, and `modified_ids` from the journaled commit diff. Board-package attachment has a first CLI dry-run/apply handoff slice but is not yet proposal-first or GUI surfaced. | Place-symbol mints/binds the electrical `ComponentInstance`, pins resolved library revisions, and returns revisioned `OperationResult` evidence through CLI/MCP/GUI. |
| Schematic authoring contract normalization | [~] `Operation::PlaceSchematicMarker` is the first normalized schematic journal operation from the ten-tool contract: it parameterizes junction vs no-connect placement with `SchematicMarkerKind`, persists through the existing sheet shard maps, replays through `ProjectResolver`, and undoes with the matching delete inverse. Compatibility CLI verbs `place-junction` and `place-noconnect` now author that normalized operation while retaining stable user-facing names; focused substrate tests prove both marker kinds and replay/undo. | Continue collapsing compatibility-shaped schematic verbs into contract vocabulary: public `place-marker`/`datum.schematic.place_marker`, `DeleteObjects`, `SetSymbolField`, richer `CreateBus`, richer `DrawWire`, and uniform `OperationResult` / `model_revision` reporting. |
| ERC consumption | [~] ERC consumes the canonical library pin electrical taxonomy through `schematic::PinElectricalType = LibraryPinElectricalType`; richer roles such as `OpenCollector`, `OpenEmitter`, `TriState`, and `NoConnect` are preserved in ERC classification tests. Live/persisted CheckRun surfaces normalize object targets/fingerprints, carry model revision, and now include explicit pin-taxonomy evidence for pin-backed ERC findings. Findings are still not fully keyed to all library object revisions. | ERC findings are revision-keyed to the resolved `DesignModel` and consume canonical library pin electrical types through placed symbols/bindings. |
| Gates | [~] Spec-governance coverage/classification, spec parity, schematic private-writer, raw-load, MCP taxonomy, and PG-* migration proof gates are wired into `run_drift_gates.sh`; library-foundation invariants are enforced behaviorally by the PG-* proofs and write-fence gates rather than a per-doc marker ledger. | Gates prevent package/footprint collapse, lost PinPadMap/LibraryGraph authority behavior, and schematic private-writer regressions. |

| Substrate Area | Status | Current Evidence | Target / Readiness Definition |
|----------------|--------|------------------|-------------------------------|
| Native library foundation | [~] | Engine pool structs, resolver-visible pool shard discovery, generic pool-library journal operations, typed CLI library producers, MCP `datum.library.*` aliases, model blob handling, and first dependency validation slices exist. `LibraryBinding` is now a Rust type on `ComponentInstance`, projects backward-compatible `part_ref` data, and rejects stale/mismatched binding payloads in resolver and commit validation. `Footprint` is now a real engine pool type instead of a package-compatible validation alias, journaled footprint payloads validate against the `Footprint` struct, `project validate` reads `pool/footprints`, checks `Footprint.package` and footprint-pad `padstack` refs, and `PinPadMap.footprint` mappings validate against footprint pads when present. `Package` now supports body-only package records with package-family/code/mounting/body-dimension/terminal fields while legacy package `pads`/`courtyard`/`silkscreen` fields remain readable for import/materialization compatibility. Board-component pool materialization and engine runtime board pad regeneration now prefer first-class `Footprint` land-pattern pads through `Part.default_footprint`, `PinPadMap.footprint`, or a unique package-matching footprint before falling back to legacy `Package.pads`; regression coverage proves canonical footprint geometry wins when both shapes exist. Typed CLI/MCP authoring now creates first-class `Footprint` objects, sets `Footprint.pads` directly, and authors Footprint silkscreen lines/rectangles/circles/polygons while rejecting missing packages, missing padstacks, blank pad names, invalid geometry, and nonpositive widths/layer ids. Engine-owned IPC-7351B two-terminal chip and SOIC generators now emit real `Footprint` objects, generated `Padstack`s, structured `IpcFootprintBasis`, density-dependent toe/heel/side/courtyard values, and explicit mask/paste process policy from deterministic functions; `project generate-ipc7351b-two-terminal-chip`, `project generate-ipc7351b-soic`, `proposal generate-ipc7351b-two-terminal-chip`, and `proposal generate-ipc7351b-soic` cover journaled writes and proposal batches, MCP exposes matching `datum.library.generate_ipc7351b_*` and `datum.proposal.generate_ipc7351b_*` aliases, and `LibraryGraph` reports IPC basis, derived aperture, and mask/paste policy mismatches through validation diagnostics. Legacy `set-pool-package-pad`, `set-pool-package-courtyard-*`, and `add-pool-package-silkscreen-*` compatibility now require exactly one package-linked Footprint and write `Footprint.pads` / `Footprint.courtyard` / `Footprint.silkscreen`, leaving `Package.pads` / `Package.courtyard` / `Package.silkscreen` unchanged. The first `LibraryGraph` dependency diagnostic seam now lives in the engine and is projected by `project validate`. Typed CLI/MCP authoring now creates and updates first-class `PinPadMap` objects directly, with optional same-batch default binding to `Part.default_pin_pad_map`; runtime compatibility and component-pad net remapping now prefer that first-class map when valid, and legacy-named part pad-map commands bridge to it instead of writing `Part.pad_map`. A Datum-authored native baseline fixture now proves governed Unit/Symbol/Entity/Package/Footprint/Padstack/Part/PinPadMap data resolves and validates without import-derived content. The current implementation is not yet the full product target: broader IPC family coverage, check-run finding/deviation/export consumption, legacy `Part.pad_map` fallback retirement, footprint graphics/process-policy authoring beyond pads/courtyards/silkscreen lines/rectangles/circles/polygons, graph-owned resolver/commit validation tiers, importer migration away from legacy package geometry, and pool layering/materialization policy remain incomplete. | A governed native library graph is engine-owned and consumed by CLI/MCP/GUI: `Package` models component body/terminal data, `Footprint` models PCB land patterns, `Padstack` models copper/drill/mask/paste process policy, `PinPadMap` is first-class binding data, `Part` pins default library choices by revision, `ModelAttachment` carries hash/provenance/review state, and commit/resolver/validate tiers are explicit. This remains the next implementation axis before board-editor expansion, now with Tier B foundation slices landed. |
| ProjectResolver | [~] | First engine-owned scaffold landed in `crates/engine/src/substrate/mod.rs`: `ProjectResolver::resolve()` reads native project roots, assembles deterministic source-shard metadata, emits diagnostics, replays the accepted journal prefix, exposes materialized source-shard values, and is exposed through `project query <root> resolve-debug`. Core board query helpers for components, pads, routing objects/nets, net classes, dimensions, text, keepouts, outline, stackup, diagnostics, and routing substrate now read the resolver-materialized board state rather than only promoted `board.json`, with stale-promoted-file regressions for stackup, pads, route preflight, route corridor, route-path candidate, route-path candidate explain, board name summary, project name summary, rules query, and forward-annotation proposal component comparisons. All `command_project_route_path_candidate*` route solver surfaces now load resolver-materialized board state before running their solvers, including bounded via-count variants, authored-copper variants, authored-via-chain variants, orthogonal dogleg/two-bend variants, and orthogonal-graph multi-via variants; focused route-path candidate regressions prove the migrated surfaces still pass. Board materialization now preserves native `pad_expansion_setup` so standards/process-aperture DRC sees the authored pad mask/paste policy. Project manifest materialization now replays journaled project-name edits into summary readback, and rules-root materialization now replays journaled rules replacement into `project query design-rules`. Board-component mutation readbacks, board-pad mutation validation/readback, and package-materialization pre-reads now use resolver-materialized board state after journaled writes; board-pad stale-promoted regression coverage proves a pad created through the journal can still be queried and edited after promoted `board.json` is reverted. Schematic wire, junction, and no-connect query surfaces now read resolver-materialized sheet shards rather than promoted sheet JSON, with stale-promoted-sheet regressions proving journaled schematic connectivity remains queryable before deletion. Native `project validate` now validates resolver-materialized project, schematic, board, rules, sheet, and definition JSON shards rather than promoted files; focused coverage restores stale promoted schematic root, sheet, and board files after journaled sheet/wire/board-text edits and still reports a clean valid project. Initial fabrication read paths also moved: Gerber outline and Gerber copper export, validation, and semantic comparison now use resolver-materialized board state with stale-promoted-file regressions; copper CAM derives pads/tracks/zones/vias from one resolved board snapshot per command. Gerber soldermask, paste, silkscreen, and mechanical export, validation, and semantic comparison now load resolver-materialized board state; export has stale-promoted-file regressions for mask, paste, silkscreen, and mechanical layers. Gerber plan/set discovery now derives the planned artifact set from resolver-materialized stackup state with a stale-promoted-file plan regression. Native CSV drill, Excellon drill, and drill hole-class reporting now read one resolver-materialized board snapshot, with stale-promoted-file export regressions for CSV and Excellon drill. Manufacturing report/manifest/inspect/export/validate/compare wrappers now load resolver-materialized board state for wrapper metadata and default artifact naming; manifest has a stale-promoted-file regression. Native project summary now reads resolver-materialized project and board state for product identity. It does not yet own imported KiCad/Eagle resolution, dependency policy, remaining mutation routing, or the remaining fabrication/export read surfaces. | One resolver owns project roots, source discovery, dependency resolution, identity lookup, and deterministic diagnostics across native, imported, CLI, MCP, and GUI paths. |
| Source shards | [~] | First source-shard metadata exists for native project manifests, schematic roots/sheets, board roots, rules roots, pools, proposal metadata, output jobs, manufacturing plans, panel projections, output-job runs, artifact runs, check runs, ZoneFill records, and artifact metadata, including stable shard IDs, relative paths, schema versions, content hashes, authority class, and dirty-state field. Resolver-visible native pool shards now include `units`, `symbols`, `entities`, `parts`, `packages`, `footprints`, `padstacks`, and `pin_pad_maps`, with discovered root object kinds reflecting the pool subdirectory rather than the generic object fallback. Pool shards now require concrete `SourceShardTaxon` metadata for those eight pool families while preserving `SourceShardKind::Pool` for journal/replay matching; unknown `pool/*` shard families are rejected at the ownership boundary, and resolver, CLI `resolve-debug`, and MCP `datum.query.source_shards` regressions prove `PoolSymbol` taxonomy is exposed alongside dirty-state reporting. Authored production shards now carry concrete `SourceShardTaxon` metadata for `ManufacturingPlan`, `PanelProjection`, and `OutputJob`; resolver recovery and CLI `resolve-debug` regressions prove those taxons remain visible alongside `Missing` dirty-state when the promoted `.datum` files are recovered from the journal. Authored identity/relationship sidecars, sidecar metadata, and generated evidence now also derive concrete `SourceShardTaxon` metadata for `ComponentInstance`, `Relationship`, `VariantOverlay`, `ImportMap`, `ProposalMetadata`, `ForwardAnnotationReview`, `OutputJobRun`, `ArtifactRun`, `CheckRun`, `ZoneFill`, and `ArtifactMetadata` through the shared value/byte-backed source-shard builders; public `resolve-debug` coverage proves missing journal-recovered `ArtifactMetadata` and `ForwardAnnotationReview` shards expose those taxons alongside authority and dirty-state. `project validate` now checks declared pool directories for root `schema_version`, filename/payload UUID parity, and the first logical library reference graph across units, symbols, entity gates, packages, padstacks, parts, and pin-pad maps while leaving full footprint cross-reference validation out of scope. Journaled raw pool-library create/set now also rejects authored pool shards without `schema_version: 1`, deserializes `units`, `symbols`, `entities`, `parts`, `packages`, `footprints`, and `padstacks` through the engine's canonical pool structs before staging, validates the first `pin_pad_maps` envelope shape, and rejects malformed footprint geometry through the current package-compatible footprint schema. New native scaffolds now emit explicit `RulesRoot` `uuid` / `object_revision` identity while older rules roots without identity remain readable. Resolver and `resolve-debug` now distinguish authored design shards from generated evidence and sidecar metadata, with native authored shards locked as clean authored-design evidence. Generic `SourceShardKind::Unknown` and `SourceShardAuthority::Unknown` have been retired; source shards must now use a concrete family/authority, while `dirty_state=Unknown` remains only for unreadable promoted files. Dirty-state mutation tracking beyond clean resolved files, recovery semantics, richer semantic library editors, and future shard families are not complete. | Project state is decomposed into source shards with explicit ownership, load order, dirty-state tracking, and recovery semantics. |
| ObjectId / ObjectRevision / ModelRevision | [~] | Resolver discovers UUID-bearing JSON objects as `DomainObject`s, reads persisted `object_revision` when present, computes deterministic `ModelRevision` from sorted shard hashes and object revisions, and now has an explicit compare-and-swap guard path through `Operation::GuardObjectRevision { object_id, expected_object_revision }`. `commit_journaled()` validates object revision guards before staging, strips guard operations from the durable journal payload, and rejects stale guards without mutating or staging writes. Object revisions bump through the engine operation-application path for project/rules, board/package/layout/routing, schematic root/sheet/definition/instance objects, ComponentInstance, relationship/variant, pool/library, and production records as those typed operation families land. Public CLI guard emission now covers board component value/reference/move/part/package/layer/rotation/locked/delete, board layout/routing/pad/netclass/dimension helper paths, schematic connectivity/text/drawing helper paths, ComponentInstance set/delete, manufacturing plan/panel projection/output job set/delete, project name, whole-root and granular project-rule edits, generic batch-file proposals, and production proposal builders. Engine regressions cover stale model revision rejection, matching/stale object revision guards, object revision bumps, and guard stripping; CLI regressions prove guarded component edits do not persist guard operations. Remaining work is broader collaboration/diff semantics and extending guard emission as new authored operation families appear, not the first revision substrate. | Every mutable product object has stable identity, per-object revision, and model revision semantics suitable for diff, undo/redo, proposal/apply, collaboration, and artifact provenance. |
| ComponentInstance | [~] | Component mutation and replacement evidence exists for the imported-board write slice. `ProjectResolver` no longer materializes resolver-derived ComponentInstance joins from exact `reference` and `part`; ComponentInstances now enter the model only from authored shards or journaled operations, while diagnostics still report unmatched and ambiguous schematic-board relationships. Regressions forbid reference-only or part-only joins and prove legacy deterministic derived IDs cannot be mutated. Resolver diagnostics now report unmatched schematic symbols, unmatched board packages, and ambiguous compatibility joins, and ambiguous joins are refused instead of silently grouping multiple symbols/packages. First persisted authored identity now exists through `.datum/component_instances/<uuid>.json` shards: resolver classifies them as `AuthoredDesign`, includes them in `ModelRevision`, validates unsupported future source-shard schema versions, defaults legacy missing-version ComponentInstance shard payload envelopes to version 1, validates filename/embedded-id parity, validates revisioned symbol refs plus optional package refs against resolved objects, inserts `component_instance` domain objects, and lets persisted refs override ambiguous compatibility joins without emitting the legacy ambiguity diagnostic. `ComponentInstanceShard` now exposes an explicit schema-version constant used by journal staging and resolver payload validation, with regressions proving legacy missing-version promoted shards still resolve while unsupported future source-shard versions remain quarantined. OperationBatch create/set/delete now commits ComponentInstance shards through the journal and MCP exposes the matching read/write aliases. CLI and MCP bind/set surfaces now support multi-symbol ComponentInstance refs through repeated `--symbol` / `symbols`, preserve full existing symbol/package ref vectors when constructing inverse payloads, and have regressions proving multi-symbol refs survive bind, set, delete undo, and query. ComponentInstance shards now carry optional `part_ref` identity to a current native `pool/parts` object; resolver validation rejects missing, stale, or wrong-kind part refs; commit validation rejects stale part revisions before staging; CLI bind/set expose `--part`; MCP flat/canonical write tools forward the same field to the CLI bridge; and pool-backed `place-symbol` now authors a symbol-first ComponentInstance with `part_ref` before any board package exists. ComponentInstance shards also carry optional per-symbol and per-package role metadata with `{role, label}` records keyed by referenced object UUID; resolver and commit validation reject role keys outside the matching ref list, blank/invalid role identifiers, and invalid labels, while CLI/MCP bind/set expose `symbol_roles` and `package_roles` inputs and write default role metadata for new authored bindings. Engine commit validation also rejects duplicate ComponentInstance refs, symbol refs targeting the wrong schematic domain, and package refs targeting the wrong board domain when package refs are present. Forward-annotation audit/proposal matching now uses resolver `ComponentInstance` symbol/package refs before falling back to legacy reference matching for uncovered objects, and uses `part_ref` as the expected part identity when present; regressions prove refdes-only matches do not create update actions for already-bound objects and that mismatches target the ComponentInstance-bound package even when references differ. Forward-annotation artifact compare now treats exact `action_id` matches as applicable and same symbol/component UUID identity plus same action/reason as `drifted` when a reference rename changes the derived `action_id`; legacy reference/action drift matching has been removed. BOM/PnP export now emits `component_instance_uuid`, `component_instance_role`, and `component_instance_label`; compare/drift keys by ComponentInstance when present with package UUID fallback for board-only legacy rows; inspect/compare still accept legacy CSV headers; direct BOM/PnP export/validate/compare can filter expected rows by a selected `VariantOverlay`'s derived fitted population; manufacturing-set export/validate/compare pass `--variant` through to those BOM/PnP rows; and OutputJob create/update/run stores and honors variant context plus role/label columns for manufacturing-set BOM/PnP output and generated ArtifactMetadata. Resolver-derived ComponentInstance joins are retired; remaining ComponentInstance work is broader semantic coverage, not compatibility-join removal. | Component instances are first-class product objects spanning schematic, board, parts, packages, fields, placement, connectivity attachments, and import provenance. |
| Relationship / variant substrate | [~] | Engine substrate now defines `Relationship`, `RelationshipKind`, `RevisionedRef`, `AuthoredIntentRecord`, `DerivedRelationshipStatus`, `FittedState`, `VariantOverlay`, `RelationshipOverride`, and `DerivedVariantPopulation`. `ProjectResolver` discovers authored `.datum/relationships/*.json` and `.datum/variants/*.json` shards as `AuthoredDesign`, includes relationship/variant shards and objects in `ModelRevision`, derives resolver-only relationship statuses and variant population/applicability maps, exposes relationship/status and variant/population counts through `resolve-debug`, rejects relationship shards that try to persist derived status as source authority, rejects unsupported future relationship and VariantOverlay source-shard schema versions before payload insertion, defaults legacy missing-version relationship/variant shard payload envelopes to version 1, and rejects variant overlays that try to persist derived population. `CreateRelationship`, `SetRelationship`, `DeleteRelationship`, `CreateVariantOverlay`, `SetVariantOverlay`, and `DeleteVariantOverlay` now commit these records through `OperationBatch` + journal staging as one-record authored shards, with undo/redo and missing-shard replay coverage. `RelationshipShard` and `VariantOverlayShard` now expose explicit schema-version constants used by journal staging and resolver payload validation, with regressions proving legacy missing-version promoted shards still resolve while unsupported future source-shard versions remain quarantined. `project query <root> relationships` / `variants` and MCP `get_relationships` / `get_variants` expose authored records plus derived status/population through read-only resolver-backed query envelopes. Richer variant composition across option links/scopes and first-class UI review remain pending. | Authored relationship bindings, derived relationship status, sparse variants, and derived `NotApplicableForVariant` are separate model surfaces with check/proposal/GUI visibility. |
| Import Map `import_key` | [~] | Engine substrate now defines `ImportMapEntry`, passively collects embedded `import_key` metadata from UUID-bearing source objects, discovers `.datum/import_map/*.json` sidecar shards as `SidecarMetadata`, excludes ImportMap sidecars from `ModelRevision`, validates sidecar schema versions plus entries against resolved objects/source shards, reports missing-object and shard-mismatch diagnostics, and exposes `import_map_count` through `project query <root> resolve-debug`. `ImportMapShard` now exposes an explicit schema-version constant, legacy missing-version ImportMap sidecars default to version 1 for resolver compatibility, focused regressions prove legacy promoted sidecars still materialize while unsupported future source-shard versions remain quarantined, and ImportMap entries default legacy missing lifecycle `status` to `active`. CLI `project query <root> import-map` and MCP `get_import_map` now expose resolver-validated import-key mappings through read-only provenance envelopes, including lifecycle `status` provenance. The first importer-facing identity primitive now exists as `allocate_import_identity`: it reuses the resolved Datum `ObjectId` for an existing `import_key` and otherwise allocates a deterministic new Datum object ID, giving importers a substrate-owned path away from source-format UUIDs. KiCad standalone footprint package import now has an Import Map-aware path that reuses a mapped package identity, otherwise allocates deterministic identity from the same `import_key`, and reports `import_key`, `source_hash`, and `reused_existing_identity` metadata. KiCad board import now has an opt-in engine-level Import Map-aware path for board footprint, pad, routed segment, via, and zone identities: `import_board_document_with_import_map` reuses mapped Datum IDs by stable board import keys, otherwise allocates deterministic non-source Datum IDs in import-map mode, while legacy `import_board_document` preserves source UUID behavior. KiCad schematic import now has an Import Map-aware path for source-backed symbols, wires, junctions, labels, buses, bus entries, no-connect markers, schematic text, bounded drawing primitives, generated sheet definitions, generated sheet instances, and deterministic sheet ports: `import_schematic_document_with_import_map_identities` maps those stable schematic import keys to Datum object IDs while legacy `import_schematic_document` preserves source UUID behavior. Durable Import Map sidecar persistence is journal-owned through `Operation::CreateImportMapShard` / `DeleteImportMapShard`; focused coverage creates a shard through `commit_journaled()`, rejects unsafe relative paths through source-shard ownership validation, and proves undo removes the promoted sidecar. Native project-root importer wiring now exists for `datum-eda project import-kicad-footprint <root> --source <file> [--pool pool]`, `datum-eda project import-kicad-board <root> --source <file>`, `datum-eda project import-kicad-schematic <root> --source <file>`, and `datum-eda project import-eagle-library <root> --source <file> [--pool pool]`: footprint import journals pool package/padstack objects plus the package Import Map sidecar, board import journals native board packages, pads, tracks, vias, zones, required nets, and one board Import Map sidecar for supported footprint/pad/segment/via/zone identities against the native board root shard, schematic import journals native sheets when needed, supported symbols, wires, junctions, labels, hierarchical ports, buses, bus entries, no-connect markers, sheet definitions, sheet instances, schematic text, bounded drawing primitives, and one Import Map sidecar for supported schematic identities against native sheet, definition, and schematic-root shards, and Eagle library import journals native pool units, symbols, entities, parts, packages, and padstacks plus one Import Map sidecar for those pool identities. Eagle library ImportMap entries now use source-facing `source_object_ref` values such as `package:SOT23-5`, `deviceset:LMV321`, and `device:LMV321:package:SOT23-5` instead of UUID-only refs, while retaining `source_hash` and `source_path` provenance. Re-import coverage proves journal-recovered ImportMap entries are reused without duplicate board-object, schematic object, or Eagle library pool-object creation, and KiCad board/schematic plus Eagle library reimport now write full same-source lifecycle sidecars that preserve current keys as `active` while marking absent keys `missing_in_source`. Undo removes promoted import sidecars plus created native objects. Eagle board/schematic importers, remaining lifecycle transitions beyond current missing-source reconciliation, richer provenance fields for broader import families, and broader library/pool authoring operations remain pending. | Imported objects carry durable `import_key` mappings that preserve source fidelity without making source-format identity the internal product identity. |
| OperationBatch | [~] | Engine substrate now defines typed `OperationBatch`, `Operation`, provenance, revision guards, deterministic diff metadata, and inverse operation capture for native project-name, native project-rules replacement, granular project-rule create/set/delete, first pool/library package and padstack creation, generic pool-library object creation/replacement/deletion, Import Map shard creation/deletion, proposal metadata creation/update/deletion, native board-package, authored board-pad, authored board-track, authored board-via, authored board-zone, authored board-net, authored board-netclass, authored board-dimension, authored board-text, authored board-keepout, authored board-outline, authored board-stackup, authored board-name, authored production `ManufacturingPlan`, `PanelProjection`, and `OutputJob` creation/set/deletion, authored relationship/variant record creation/set/deletion, and authored schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol edits. First pool/library/import/proposal operation vocabulary now covers `AddProjectPoolRef`, `DeleteProjectPoolRef`, `CreatePoolPackage`, `DeletePoolPackage`, `CreatePoolPadstack`, `DeletePoolPadstack`, `CreatePoolLibraryObject`, `SetPoolLibraryObject`, `DeletePoolLibraryObject`, `CreateImportMapShard`, `DeleteImportMapShard`, `CreateProposalMetadata`, `SetProposalMetadata`, and `DeleteProposalMetadata`; the KiCad footprint project import path uses those operations so package/padstack pool files and the Import Map sidecar are journaled source shards rather than bootstrap-only direct writes, generic library object operations make native pool directories such as `symbols`, `units`, `entities`, `parts`, `footprints`, and `pin_pad_maps` replayable, replaceable, undoable, schema-version guarded, and typed for canonical non-footprint pool structs through the journal, and proposal create/review/apply status metadata now stages sidecar writes through the journal rather than direct overwrite. Native board-package operations now cover create, delete, part reference, package reference, value, reference, position, raw layer field, component side, rotation, and locked state through the substrate path; package position moves translate owned board pads by the authored move delta, and component side changes mirror owned authored pads plus persisted component_pads across the package origin, swap side-sensitive pad layer arrays, mirror pad rotation, and create/delete/package reassignment carry deterministic per-component materialization payloads for derived pads/models/graphics and object-revision accounting. Native board-pad operations now cover create, replace/edit, net assignment/clear, and delete using exact pad payloads. Native board-track operations now cover create, replace/edit, and delete using exact track payloads, and route-apply now batches all draw-track proposal actions into one atomic operation batch. Native board-via operations now cover create, replace/edit, and delete using exact via payloads. Native board-zone, board-net, board-netclass, board-dimension, board-text, and board-keepout operations now cover create/delete using exact payloads; board-net, board-netclass, board-dimension, board-text, and board-keepout also cover replace/edit. Project-name edits cover project-manifest field replacement and inverse capture against the project root object; project-rules replacement covers the shard-level `rules` array shape, while granular project-rule create/set/delete now uses `RulesRoot` identity, rule UUID identity, rules-root revision bumping, rule-payload `object_revision` normalization/bumping, native rule schema validation, inverse capture, and stale-promoted-rules replay coverage. Whole-root rules replacement also normalizes missing rule payload revisions to `0` and rejects malformed native rule payloads before staging. Board-outline, board-stackup, and board-name operations cover board-root field replacement and inverse capture against the board root object. Native schematic-wire, schematic-junction, schematic-noconnect, and schematic-bus-entry operations now cover create/delete; schematic-label, schematic-port, schematic-bus, schematic-text, schematic-drawing, and schematic-symbol operations now cover create/replace/delete against schematic sheet shards through the same operation vocabulary. Schematic bus-entry writes reject missing/cross-sheet bus references and optional missing/cross-sheet wire references at the engine operation boundary. Schematic symbol set operations also synchronize nested UUID-bearing field/pin object membership so promoted-tip replay keeps model revisions aligned. `BumpObjectRevision` remains as a narrow proof operation. Remaining richer semantic library editors, richer footprint cross-reference validation, rule reference-resolution semantics, check, artifact/generated-evidence, and broader proposal operation vocabulary remain pending. | Product edits are grouped as atomic, typed batches with validation, deterministic ordering, journal entries, revision updates, and surfaced results. |
| `commit()` / journal / recovery | [~] | `DesignModel::commit()` applies operation batches in memory, rejects stale model revisions, computes before/after `ModelRevision`, and records `TransactionRecord` metadata. `commit_journaled()` stages touched shard bytes from journal-materialized source state, captures inverse operations for project-rules, board-package, board-pad, board-track, board-via, board-zone, board-net, board-netclass, board-dimension, board-text, board-keepout, board-outline, board-stackup, board-name, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol writes, fsyncs staged data, appends canonical transaction JSONL to `.datum/journal/transactions.jsonl`, fsyncs the journal, atomically promotes staged shard files, and `ProjectResolver` replays persisted transactions on resolve. Replay now accepts production-only promoted histories by comparing final production shard state against the journal when transient replay would require already-deleted intermediate files. Replay skips duplicate transaction IDs, reports duplicate/parse/conflict/chain/after-revision/link diagnostics, preserves the valid prefix, handles already-promoted board, rules-root, and schematic-sheet shard tips, reconstructs missing/stale authored production shards for `ManufacturingPlan`, `PanelProjection`, and `OutputJob` create/delete records, accounts for create/delete/set object membership while validating replayed source shards, replays source-shard hashes during promoted-tip validation for source-changing operations, and append idempotency is tip-scoped so historical duplicate transaction IDs are refused. Materialized shard readback now skips replay when the promoted source hash already matches the accepted shard hash, preventing same-process double-application of source-changing rules operations while still replaying stale promoted files. Journal append now treats newline-terminated records as the durable prefix, truncates torn trailing JSONL fragments before append, fsyncs the repaired journal length, and still refuses complete malformed lines. `.datum/journal/cursor.json` now stores a durable applied-transaction cursor, resolver validates malformed/out-of-range/behind cursors, normalizes stale behind cursors to the journal tip, and `journal-list` plus canonical `datum-eda journal list` expose append-only undo/redo availability from the journal tip. First compensating undo/redo exists for invertible project-rules, board-package, board-pad, board-track, board-via, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol transactions through `DesignModel::commit_journal_undo`, `DesignModel::commit_journal_redo`, `project undo`, `project redo`, canonical `datum-eda journal undo`, and canonical `datum-eda journal redo`; undo records `undo_of`, redo records `redo_of`, and links survive reopen. Canonical `datum-eda journal show` reads full transaction records by UUID. MCP exposes both flat journal compatibility methods and canonical `datum.journal.list`, `datum.journal.show`, `datum.journal.undo`, and `datum.journal.redo` aliases. A normal transaction after undo now clears redo availability, `project redo` reports that the redo stack was cleared, and resolver replay rejects externally injected stale redo records after branch commits. `set-project-rules`, `create-project-rule`, `delete-project-rule`, `place-board-component`, `move-board-component`, `set-board-component-part`, `set-board-component-package`, `set-board-component-layer`, `delete-board-component`, `place-board-pad`, `edit-board-pad`, `set-board-pad-net`, `clear-board-pad-net`, `delete-board-pad`, `draw-board-track`, `edit-board-track`, `delete-board-track`, `place-board-via`, `edit-board-via`, `delete-board-via`, `place-board-zone`, `delete-board-zone`, `place-board-net`, `edit-board-net`, `delete-board-net`, `place-board-net-class`, `edit-board-net-class`, `delete-board-net-class`, `place-board-dimension`, `edit-board-dimension`, `delete-board-dimension`, `place-board-text`, `edit-board-text`, `delete-board-text`, `place-board-keepout`, `edit-board-keepout`, `delete-board-keepout`, `set-board-outline`, `set-board-stackup`, `set-board-name`, `add-default-top-stackup`, `draw-wire`, `delete-wire`, `place-junction`, `delete-junction`, `place-noconnect`, `delete-noconnect`, `place-label`, `rename-label`, `delete-label`, `place-port`, `edit-port`, `delete-port`, `create-bus`, `edit-bus-members`, `place-bus-entry`, `delete-bus-entry`, `place-schematic-text`, `edit-schematic-text`, `delete-schematic-text`, `place-drawing-line`, `place-drawing-rect`, `place-drawing-circle`, `place-drawing-arc`, `edit-drawing-line`, `edit-drawing-rect`, `edit-drawing-circle`, `edit-drawing-arc`, `delete-drawing`, all native schematic symbol mutation commands, direct `route-apply`, and route-proposal artifact apply now use the journaled substrate path; board-component mutation readbacks and package-materialization pre-reads resolve journal state before reporting or deriving replacement payloads; multi-track route apply and place/edit/delete track plus place/edit/delete via/zone/net/netclass/dimension/text/keepout plus edit net/netclass/dimension/text/keepout and set outline/stackup/name are locked by journal-list or undo/redo regressions, rules replacement/create/delete are locked by stale-promoted-rules query and undo regressions, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol create/delete/set are locked by journal-list and stale-promoted-sheet replay regressions, authored production record creation is locked by a missing-shard replay regression, delete target lookup for those schematic primitives resolves journal-materialized sheet state before committing deletion, label/port/bus/bus-entry/text/drawing/symbol queries plus schematic summary counts resolve journal-materialized sheet state, core board query helpers now resolve stale promoted board files through journal replay, Gerber outline/copper/mask/paste/silkscreen/mechanical export/validate/compare now read resolver-materialized board state, Gerber plan/set discovery resolves stale promoted stackup files through journal replay, native CSV/Excellon drill export/validate/compare plus drill hole-class reporting now read resolver-materialized board state, manufacturing report/manifest/inspect/export/validate/compare wrappers now resolve journal-materialized board state, and forward-annotation review decisions moved out of direct `project.json` rewrites into `.datum/forward_annotation_review/review.json` with legacy embedded review read fallback. The forward-annotation review-state sidecar remains an explicit tracked exception confined to `command_project_forward_annotation_review_state.rs`; the guard verifies its path and forbids it from writing `project.json` or using direct `std::fs::write`. The former GUI board-text private writer module has been deleted, selected GUI text edits route through terminal-prefilled journaled CLI commands, and the private-writer guard now covers both the forward-annotation review mutation modules and GUI board-text regression names. Full crash recovery mode UX, universal operation inverses, migration of remaining private write paths, and resolver-backed fabrication/export readers beyond those CAM layers remain pending. | Commits persist operation batches through a journal with crash recovery, idempotency, replay, and clear failure boundaries. |
| proposal / apply | [~] | Engine substrate now defines `Proposal`, `ProposalStatus`, `ProposalSource`, and `ProposalRef`; `ProjectResolver` discovers `.datum/proposals/*.json` as proposal metadata excluded from `ModelRevision`, rejects proposal metadata with unsupported future source-shard schema versions or filenames that do not match the embedded `proposal_id`, validates proposal payload schema versions, defaults legacy missing-version proposal payloads to version 1, rejects unsupported future proposal payload versions before they enter the model, `DesignModel` exposes proposals, and `apply_accepted_proposal()` requires `accepted` status plus current `expected_model_revision` before applying the embedded `OperationBatch` through `commit_journaled()`, then includes a journaled `SetProposalMetadata` status update in the same accepted model-change transaction, carrying that transaction id as the applied proposal id. `ProposalStatus` now includes `deferred`; `review_proposal_status()` centralizes draft-to-accepted/deferred/rejected transitions, refuses applied/non-draft proposal edits, and refuses to mark stale or missing-revision-guard drafts as accepted while still allowing stale drafts to be deferred or rejected. `ProposalApplyValidation` now provides shared typed blocker codes (`missing_acceptance`, `stale_model_revision`, `missing_revision_guard`) used by CLI validation and apply rejection. `create_draft_proposal_from_batch()` now provides generic arbitrary proposal authoring for typed `OperationBatch` payloads: it stamps or validates the prepared model revision guard, dry-runs the batch for operation validation/affected-object preview, commits draft proposal metadata through journaled `CreateProposalMetadata`, and never mutates authored design state. Producer-specific production proposal builders now cover OutputJob, ManufacturingPlan, and PanelProjection create/update/delete: canonical `datum-eda proposal create|update|delete-output-job`, `create|update|delete-manufacturing-plan`, and `create|update|delete-panel-projection` commands, flat MCP `*_proposal` tools, and canonical `datum.proposal.create|update|delete_*` aliases build the same typed operation batches as direct CRUD, preflight missing variants, missing manufacturing plans, and invalid panel board targets before draft persistence, leave authored production shards unchanged until review/apply, and then rely on the generic proposal gateway. Board-component replacement proposal builders now cover single and repeated multi-component package/part/value replacements through `datum-eda proposal create-board-component-replacement` and `create-board-component-replacements`; flat MCP `create_board_component_replacement(s)_proposal` and canonical `datum.proposal.create_board_component_replacement(s)` expose the same non-mutating proposal producers, with CLI coverage proving multi-component batches do not mutate until `accept-apply` and then apply through the proposal gateway. The commit gateway now blocks `Tool`/`Assistant` direct generated-evidence writes for `OutputJobRun`, `ArtifactRun`, `CheckRun`, `ArtifactMetadata`, and `ZoneFill` set/delete operations with `proposal_required_for_automated_generated_evidence_operation`, preserving CLI/test/internal generator paths and accepted proposal apply context while preventing AI/tool evidence mutation bypasses. Legacy `project ... --as-proposal` compatibility commands remain available. `project query <root> resolve-debug` exposes `proposal_count`; CLI `project query <root> proposals`, canonical `datum-eda proposal create <root> --batch <operation-batch.json> --rationale <text>`, `datum-eda proposal list <root>`, MCP `create_proposal`, `get_proposals`, `datum.proposal.create`, and `datum.proposal.list` expose generic proposal creation/query. CLI `project show-proposal`, `validate-proposal`, `defer-proposal`, `review-proposal`, and `apply-proposal`; canonical `datum-eda proposal show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, and `apply`; flat MCP `show_proposal`, `preview_proposal`, `validate_proposal`, `defer_proposal`, `reject_proposal`, `review_proposal`, `accept_apply_proposal`, and `apply_proposal`; and canonical MCP `datum.proposal.show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept_apply`, and `apply` now provide single-proposal inspection, explicit stale/prepared-against validation, draft deferral/rejection, review, and apply surfaces. `accept-apply` composes accepted review plus guarded apply without bypassing proposal policy. GUI production status now loads persisted proposal summaries plus best-effort validation state, and the Outputs dock renders proposal rows with terminal-command actions for proposal list/show, read-only preview, validate, defer, reject, and accept-apply so GUI review still routes through the same CLI proposal gateway while exposing the stored-proposal diff path needed by ghost-review consumers. `project route-apply` now wraps draw-track operations in a persisted accepted proposal and applies it through the proposal gateway. Exported route proposal artifacts now embed the engine `Proposal`, and artifact apply prefers that embedded proposal with legacy action replay retained for older artifacts. Forward-annotation exports/filtering now embed an engine `Proposal` for review-accepted self-sufficient value updates and board-component removals; artifact apply refuses executable self-sufficient artifacts that lack an embedded substrate proposal, and direct/reviewed self-sufficient forward-annotation applies now route through the proposal gateway while explicit-input add/package-resolution actions remain on the legacy path until their substrate operation mappings are complete. Richer direct GUI proposal authoring/apply surfaces and proposal builders for remaining production-adjacent operations remain pending. | All AI/tool-suggested changes can be proposed, inspected, checked, and applied through one substrate path with stable IDs and revision guards. |
| CheckRun / CheckFinding | [~] | ERC/DRC/check reports exist across older engine, CLI, and MCP slices. `project query <root> check-run`, native `project query <root> erc`, and native `project query <root> drc` now return read-only live `check_run_v1` views; the ERC/DRC queries are profile-scoped to `erc`/`drc`, model-revision keyed through `ProjectResolver`, preserve raw compatibility reports under `raw_report.erc` / `raw_report.drc`, normalize findings/fingerprints/coverage, honor existing waiver state, and do not persist `.datum/check_runs` evidence. Canonical `datum-eda check run <root>` persists the resolver-keyed `CheckRun` generated-evidence record under `.datum/check_runs/<check_run_id>.json` through journaled `Operation::SetCheckRun` rather than direct helper persistence. The engine helper for persisted CheckRun evidence now invokes the same semantic validator before writing promoted `.datum/check_runs` shards, while resolver discovery still quarantines invalid promoted evidence and rejects unsupported future source-shard schema versions before deserializing CheckRun payloads. CheckRun records carry explicit `schema_version: 1`, legacy missing-version read compatibility, unsupported payload-version rejection, deterministic run/finding IDs, model revision, summary, ordered findings, raw legacy report, and artifact-linked findings for resolver-discovered invalid generated artifacts. Native schematic check inputs now build sheets from `ProjectResolver` materialized source-shard values rather than promoted sheet files; a stale-promoted-sheet regression proves a journaled no-connect edit changes CheckRun ERC findings even when the promoted sheet omits the marker. Invalid artifact metadata raises check status to `error`, increments summary error counts, and emits deterministic `artifact_validation_invalid` findings with the artifact id, file payload, and persisted production projection proofs for generated Gerber copper / Excellon drill evidence. Resolver-discovered proposals can now link back to findings through semantic `Proposal.finding_fingerprints`; transitional matching still accepts older proposal metadata that stored finding IDs under that field. DRC violation IDs are now deterministic across the native DRC check module, and check-run regression coverage locks stable process-aperture standards findings for inherited copper apertures, missing pad mask expansion, and paste reduction. Resolver-derived unfilled zone fills now emit deterministic `zone_fill_unfilled` findings and raise check-run status to `error`. Persisted `CheckFinding` records now carry target-compatible fields for `fingerprint`, `domain`, `rule_id`, `status`, `primary_target`, `related_targets`, `message`, `evidence`, `waiver_refs`, and `deviation_refs`; the engine validator rejects legacy null `primary_target` values and requires every primary/related target to carry concrete `kind` plus nonblank `id`. ERC raw object UUIDs now project into normalized `object_uuid` `primary_target` / `related_targets` while remaining preserved in payload/evidence for compatibility. `fingerprint` is now a deterministic `sha256:` semantic hash over domain, rule id, primary target, and normalized evidence, while `finding_id` remains revision-keyed run identity. `WaiverTarget::Fingerprint` now exists in the shared waiver model, and check runs apply fingerprint-scoped native waivers by keeping waived findings visible, marking `status=waived`, filling `waiver_refs`, and recomputing summary severity counts from normalized findings. `project waive-finding <root> --fingerprint <sha256:...> --rationale <text>` and canonical `datum-eda check waive <root> --fingerprint <sha256:...> --rationale <text>` now commit fingerprint-scoped schematic waivers through `OperationBatch`, staged schematic-root source-shard writes, the append-only transaction journal, and journal undo/redo; `project accept-deviation` and canonical `datum-eda check accept-deviation` do the same for accepted deviations. MCP exposes both flat compatibility names (`waive_finding`, `accept_deviation`, `generate_standards_repair_proposals`) and canonical aliases (`datum.check.waive`, `datum.check.accept_deviation`, `datum.check.repair_standards`). `project generate-standards-repair-proposals` and `datum-eda check repair-standards` now persist the referenced CheckRun through the explicit journaled check-run path before creating opt-in draft `ProposalSource::Check` repair proposals, so check-authored proposal policy validates against durable generated evidence rather than a read-only live check view. Process-aperture DRC findings, including `pad_process_aperture_inherited_from_copper`, are grouped per pad into `SetBoardPad`; track-width DRC findings produce direct `SetBoardTrack` geometry proposals, and via hole/annular DRC findings produce direct per-via `SetBoardVia` geometry proposals while preserving netclass rules as the standard authority. End-to-end apply coverage now exists for process-aperture repairs, track-width repairs, and via-geometry repairs: each accepts/applies through the generic proposal gateway, reaches `Applied`, updates authored board geometry to the governing standard, and clears the repaired object's matching standards findings, including inherited aperture findings for repaired pads. Standards profiles, persisted check history UX beyond the latest deterministic record, additional repair families for clearance/silk/connectivity/topology, and proposal review/apply UX remain pending. | Checks produce first-class runs and findings with provenance, affected objects, waivers, revisions, severity policy, and artifact linkage. |
| ZoneFill | [~] | Engine substrate now defines `ZoneFillState` and `ZoneFill`; `ProjectResolver` derives native board zones as explicit `Unfilled` zone fills with source zone revision, model revision, empty islands, and no provenance. ZoneFill generated evidence can now persist under `.datum/zone_fills/<zone_id>.json`, resolves as `GeneratedEvidence`, and is marked `Stale` if its stored model/source-zone revision no longer matches the current resolved model. Resolver validation rejects invalid generated ZoneFill records before they can become renderable copper: payloads carry explicit `schema_version: 1`, legacy missing-version records default to version 1 for read compatibility, unsupported future schema versions are rejected, `Filled` records must carry provenance plus at least one closed, non-degenerate, non-self-intersecting island, and `Unfilled`/`Unsupported` records must not carry renderable islands. `project query <root> resolve-debug` exposes `zone_fill_count`, `project query <root> zone-fills` now returns a resolver-backed `zone_fills_query_v1` envelope, MCP exposes the same read surface as `get_zone_fills`, and `project query <root> check-run` emits deterministic `zone_fill_unfilled`, `zone_fill_stale`, or `zone_fill_unsupported` findings so authored zones are never silently treated as filled copper. Native copper Gerber export/validation/comparison now uses a ZoneFill-backed projection adapter: only `ZoneFill{Filled}.islands` become renderable copper regions, while `Unfilled`, `Stale`, and `Unsupported` states remain non-renderable hard-check states. Canonical `datum-eda check fill-zones <root> [--zone <uuid>] [--net <uuid>]`, compatibility `project fill-zones`, MCP `fill_zones`, and canonical MCP `datum.check.fill_zones` now commit journaled `SetZoneFill` generated-evidence operations with inverse delete/restore behavior for undo, redo, and replay; `SetZoneFill`/`DeleteZoneFill` payloads are validated against operation ids and schema version before model application and staging, and `fill-zones` captures `previous_zone_fill` from resolver/journal-materialized evidence so undo restores stale or missing-promoted prior fills rather than derived `Unfilled` placeholders. The engine-owned bounded solver emits `ZoneFill{Filled}` with one island equal to the authored polygon for closed, non-degenerate, no-thermal zones when intersecting same-layer authored board pads, tracks, and vias are on the same net; packages alone no longer block fill generation. It also supports rectangular foreign pad/via/track cutouts for rectangular zones by inflating each obstacle by the zone netclass clearance and emitting rectangular islands around the resulting no-copper rectangles. Multiple non-overlapping obstacles are grid-cut into rectangular islands, overlapping/touching inflated obstacle clearances are conservatively unioned into larger no-copper rectangles, edge-crossing inflated obstacles are clipped to the zone bounds before fill, non-orthogonal foreign track bounds are conservatively removed, and authored keepout bounding boxes are removed from copper rather than making the entire fill unsupported. Persisted component pads with unresolved net attribution are now scoped by geometry: unrelated component pads outside the requested zone no longer block fill generation, while unresolved component pads whose conservative bounds intersect the zone still produce explicit `Unsupported` evidence instead of fake copper. Thermals with same-net anchors, missing/nonpositive clearance basis for foreign copper, general clearance subtraction, antipads, and arbitrary obstacle-avoidance cases remain explicit `Unsupported` evidence with provenance rather than fake copper. Current resolver output without persisted generated evidence is still `Unfilled`, so authored zone boundaries are withheld from emitted copper, `unfilled_zone_count` / `unfilled_zone_ids` report the blocked zones, and supplied Gerber regions compare as extra until a supported fill producer supplies `Filled` islands. Remaining gaps are the general pour solver, GUI rendering, and richer persisted fill provenance. | Zone fill is represented as a product operation/result with deterministic generated geometry, invalidation, check integration, and artifact provenance. |
| OutputJob / artifacts | [~] | Engine substrate now defines shared `PanelProjection`, `PanelBoardInstance`, `ManufacturingPlan`, `OutputJob`, `OutputJobRun`, `ArtifactRun`, `OutputJobRunProvenance`, `OutputJobLogEntry`, `ArtifactMetadata`, `ArtifactFile`, `ArtifactProductionProjection`, `ArtifactKind`, and validation/status vocabulary; `ArtifactKind` now distinguishes `gerber_set` from full `manufacturing_set` output bundles. `ProjectResolver` discovers authored `.datum/panel_projections/*.json`, `.datum/manufacturing_plans/*.json`, and `.datum/output_jobs/*.json` shards as design authority and derived `.datum/output_job_runs/*.json`, `.datum/artifact_runs/*.json`, plus `.datum/artifacts/*.json` evidence shards without letting generated evidence change `ModelRevision` or create `DomainObject`s. Authored production records now have typed create/delete operations and journaled per-object shard staging, and `OutputJob` now also has a replacement `SetOutputJob` operation with inverse capture, object-revision bumping, stale-promoted replay coverage, and `project update-output-job` CLI editing for name/manufacturing-plan/variant links. `project delete-output-job` now commits `DeleteOutputJob` through the same journal path, removes the authored shard, and is locked by query/undo/redo regression coverage. `project create-panel-projection`, `project update-panel-projection`, `project delete-panel-projection`, `project create-manufacturing-plan`, `project update-manufacturing-plan`, `project delete-manufacturing-plan`, and `project create-gerber-output-job` create/edit/delete production records through the substrate path rather than direct JSON writes; delete guards refuse dangling ManufacturingPlan-to-PanelProjection and OutputJob-to-ManufacturingPlan references. Promoted production-journal replay now preserves the accepted journal tip revision when materialized production shards already match the journal, so later production deletes can append and undo/redo remains available after reopen. Authored production record payloads now carry explicit `schema_version: 1` with legacy missing-version read compatibility and unsupported-version rejection at staging/model validation plus resolver discovery. Generated evidence manifests now persist through engine substrate helpers (`persist_artifact_metadata`, `persist_output_job_run`, `persist_artifact_run`) rather than CLI-local JSON writers; `OutputJobRun`, `ArtifactRun`, and `ArtifactMetadata` payloads carry explicit `schema_version: 1` with legacy missing-version read compatibility, unsupported-version rejection, and promoted source-shard schema guards; run-evidence provenance rejects blank terminal session IDs, empty terminal context paths, empty project roots, and blank source revisions when provenance is present; artifact metadata rejects blank generator versions and absolute/parent-traversal artifact file paths, resolver rejects artifact/run manifests whose filename does not match the embedded id, and generated evidence shards are asserted as `GeneratedEvidence`. `project create-panel-projection` creates a production-only panel target from board instances without mutating board geometry; `project create-manufacturing-plan --panel-projection <uuid>` targets that panel; `project create-gerber-output-job --manufacturing-plan <uuid>` links Gerber output jobs to manufacturing intent; `project create-output-job --variant <uuid>` stores variant context; `project update-output-job --variant/--clear-variant` journals variant edits with undo coverage; and `project query <root> panel-projections`, `manufacturing-plans`, and `output-jobs` round-trip the links. `project export-gerber-set` writes linked artifact metadata plus a deterministic succeeded `OutputJobRun` log; when launched from the GUI PTY it records structured `gui_terminal` run provenance plus human-readable terminal session/context log entries. `project validate-gerber-set` persists artifact validation state. `project export-manufacturing-set` now writes a full manufacturing-set `ArtifactMetadata` record and succeeded `OutputJobRun` covering BOM, PnP, drill CSV, Excellon, and Gerber files, with the same structured GUI-terminal provenance when inherited. `project run-output-job` now honors stored OutputJob variant context for manufacturing-set BOM/PnP rows and generated ArtifactMetadata. Gerber-set and manufacturing-set `ArtifactMetadata.production_projections` now persist live-production projection proofs for generated Gerber copper and Excellon drill bytes, output-job logs record the same proof trail, artifact queries expose it, and validation updates preserve it. `project validate-manufacturing-set` updates that artifact validation state, and invalid manufacturing-set metadata feeds `project query <root> check-run` as an `artifact_validation_invalid` finding. Manufacturing-set report, manifest, inspect, validate, compare, and export now share the same T0 projection helper for expected artifact entries and Gerber plan context; export parity is locked by a regression that compares manifest filenames to exported artifact metadata and artifact view paths. MCP exposes panel projection, manufacturing plan, output-job query/authoring/update/delete, and manufacturing-set export/validation evidence methods over the CLI bridge, including journaled panel/manufacturing update/delete methods. GUI workspace load now queries `output-jobs`, `manufacturing-plans`, `panel-projections`, and persisted proposal metadata opportunistically for native projects, stores aggregate/per-job/per-plan/per-panel/per-proposal `ProductionStatus`, renders output-job count/artifact count/latest status in the side-panel chrome, exposes a read-only `OUTPUTS` dock tab listing panel projection summaries, manufacturing plan summaries, proposal summaries and terminal actions, job names/status/run counts/artifact counts/latest run IDs, capped artifact kind/file path/hash summaries, and production projection proof rows, and marks production refresh pending after terminal newline submission so subsequent PTY output can reload only `ProductionStatus` without parsing shell text, keeps pending refresh alive across unchanged early command output, and performs a final refresh attempt when the terminal command exits. First live-production equivalence metadata now exists for Gerber copper and Excellon drill export/validation: both surfaces emit a `production_projection` envelope with projection kind, projection contract, source model revision, byte count, and SHA-256 derived from the same resolver-materialized generation path used for the artifact bytes. Concrete panel production geometry editing, artifact selection/opening, richer artifact drill-down beyond capped summaries, and broader live-production projection families remain pending. | Outputs are modeled as jobs with inputs, revisions, produced artifacts, logs, status, reproducibility metadata, and CLI/MCP/GUI visibility. |
| PTY terminal | [~] | GUI terminal lane now opens a Unix PTY, starts `$SHELL` in the active project root with the slave as controlling terminal, routes terminal-focused printable keys, Enter, Backspace, Tab, arrows, Home/End, Escape, and paste text to the PTY master, sends Ctrl-C as SIGINT to the terminal process group, supports Ctrl+Shift+K process-group termination and Ctrl+Shift+R restart from the original project root, suppresses canvas hotkeys while the terminal is focused, propagates dock/window size to the PTY with `TIOCSWINSZ`, streams raw PTY bytes into a persistent first screen-row model, preserves partial prompts without requiring newline output, handles newline/carriage-return/backspace/delete basics, handles default 8-column horizontal tabs plus `CSI I` forward-tab and `CSI Z` backtab, decodes split UTF-8 safely, strips split CSI sequences, swallows OSC control strings with BEL or ST terminators, handles `ESC M` Reverse Index, tracks row/column cursor state, supports PTY-width autowrap with pending-wrap carriage-return cancellation and scroll-region bottom wrapping, supports `CSI K` erase-in-line modes 0/1/2, `CSI J` erase-display modes 0/1/2/3, `CSI nA` / `CSI nB` vertical cursor moves, `CSI nC` / `CSI nD` horizontal cursor moves, `CSI nE` / `CSI nF` next/previous line moves with column reset, `CSI d` vertical absolute positioning, `CSI e` vertical relative positioning, `CSI k` vertical backward positioning, `CSI n \`` horizontal absolute positioning, `CSI n a` horizontal relative positioning, `CSI L` insert-line, `CSI M` delete-line, `CSI S` scroll-up, `CSI T` scroll-down, `CSI G` absolute columns, `CSI row;col H` / `f` absolute cursor positioning, display-neutral multi-parameter SGR swallowing without byte leakage, maps plain Ctrl-C to PTY interrupt while Ctrl+Shift+C copies terminal scrollback text to the clipboard, surfaces the scrollback-copy and paste shortcuts directly in the terminal dock, reports terminal lifecycle status as running/exited/terminated, and no longer presents the lane as read-only or manually echoes submitted commands. Terminal-launched shells now receive canonical first-slice context variables `DATUM_PROJECT_ROOT`, `DATUM_PROJECT_ID`, `DATUM_MODEL_REVISION`, `DATUM_CONTEXT_ID`, `DATUM_SESSION_ID`, `DATUM_DISCOVERY`, and `DATUM_CLI=datum-eda`, while retaining legacy compatibility aliases `DATUM_SOURCE_REVISION`, `DATUM_TERMINAL_SESSION_ID`, `DATUM_TERMINAL_CONTEXT`, and `DATUM_LEGACY_CLI=eda`. `.datum/gui-terminal-context.json` carries a `datum_terminal_context_v1` discovery envelope derived from the active project/scene with canonical context/session/model fields, the current `ProductionStatus` snapshot, explicit terminal-launched `agent_commands` templates (`codex`, `claude`, `aider`, inspect context, refresh context), and explicit context, canonical check (`run`, `list`, `show`, `profiles`, `repair-standards`, `waive`, `accept-deviation`), canonical proposal (`list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, `apply`), production, journal, and resolver-query command templates; current Gerber/manufacturing export commands inherit those env vars and persist structured `OutputJobRun.provenance` plus ordered terminal-origin log entries. Terminal discovery now advertises canonical `datum-eda journal list|show|undo|redo` templates instead of legacy `project query` journal routes, and resolver query templates for schematic sheets/hierarchy plus import map, relationships, variants, and zone fills now use canonical `datum-eda query <family>` commands in both GUI-written and CLI-refreshed discovery envelopes. Focused coverage proves a shell command executes through the PTY, the PTY accepts resize calls, exit status is emitted, SIGTERM termination emits a signal exit, partial prompt bytes render, carriage-return rewrites a row by cursor column, default tab stops align printable output at eight-column boundaries, CSI forward-tab advances across repeated default stops, CSI backtab returns to prior stops without underflow, erase-in-line clears progress tails, row-addressed cursor movement rewrites prior terminal rows, CSI next/previous line moves reset to column zero and extend rows when needed, CSI vertical absolute/relative/backward positioning preserves the active column while moving/extending rows and saturating at the top, CSI horizontal absolute/relative positioning writes at the intended column and creates intervening blank cells, CSI insert/delete line shifts rows inside the active screen or scroll region without disturbing rows outside it, CSI scroll-up/down shifts rows inside the active screen or scroll region and clears introduced rows, erase-display clears stale terminal rows, terminal-width autowrap advances printable text to the next row, carriage return cancels a pending wrap at the last column, scroll-region bottom wrapping scrolls only the margin rows, Reverse Index moves the cursor up away from the top margin and scrolls only the active region down at the top margin, split ANSI color escapes do not leak bytes, OSC title/control payloads do not leak bytes even when split across PTY reads, split UTF-8 decodes once complete, Ctrl-C remains interrupt while Ctrl+Shift+C is a copy handoff, terminal scrollback copy text is stable and trims trailing blank rows, render coverage proves the terminal dock advertises copy/paste shortcuts, `+NEW`/`RENAME`/`RESTART`/`CLOSE` session controls, restarted-tab counts, and canonical journal list/undo/redo handoffs, registry coverage proves active terminal restart preserves the tab label while publishing previous-session lineage and a refreshed latest context, a real shell can read the injected Datum context including terminal agent launch templates, and the CLI helper converts GUI terminal context into structured output-job provenance plus ordered terminal-origin log entries. Full VT control-sequence screen rendering, pointer selection ranges, attach/detach persistence, and richer persistent session-history UX remain pending. | A PTY-backed terminal can run project-scoped jobs/commands with streamed output, cancellation, exit status, and artifact/job linkage. |
| CLI/MCP taxonomy `datum-eda` | [~] | Current CLI/MCP surfaces are inventoried in `specs/SPEC_PARITY.md`; runtime MCP catalog and CLI command counts are locked by parity gates. First canonical CLI taxonomy slice now exists as read-only `datum-eda context get|refresh|session-events`, returning the GUI terminal discovery envelope from `DATUM_DISCOVERY`, `DATUM_TERMINAL_CONTEXT`, `--path`, or `--project-root`, with optional `--session` mismatch rejection; `context refresh` persists the enriched envelope back to the resolved context file, while `context session-events` resolves `.datum/tool-sessions/<session>.events.jsonl` and returns parsed `datum_tool_session_events_v1` tool-session provenance without touching the project journal. Canonical query aliases now exist for resolver-backed native summary, component instances, schematic sheets/symbols/labels/ports/buses/bus entries/no-connects/hierarchy/nets/diagnostics, relationships, variants, import map, zone fills, panel projections, manufacturing plans, and output jobs; legacy imported-design reads remain under `query imported <file> <what>` plus the historical `query <file> <what>` compatibility parser. MCP already exposes the matching first query family through `datum.query.board_summary`, `components`, `netlist`, `schematic_summary`, `zone_fills`, `component_instances`, `relationships`, `variants`, `import_map`, `panel_projections`, `manufacturing_plans`, and `output_jobs` over compatibility dispatch methods. Canonical ComponentInstance write aliases now exist as `datum.component_instance.bind`, `set`, and `delete` over the journal-backed compatibility dispatch methods. Native library-object authoring now has CLI/MCP journal and query paths through `datum-eda project create-pool-library-object <root> --pool <pool> --kind <kind> --object <uuid> --from-json <file>`, typed `create-pool-unit`, typed `set-pool-unit-pin`, typed `create-pool-symbol`, typed `add-pool-symbol-line`, typed `add-pool-symbol-rect`, typed `add-pool-symbol-circle`, typed `add-pool-symbol-polygon`, typed `add-pool-symbol-arc`, typed `add-pool-symbol-text`, typed `create-pool-entity`, typed `create-pool-padstack`, typed `create-pool-package`, typed `set-pool-package-pad`, typed `set-pool-package-courtyard-rect`, typed `set-pool-package-courtyard-polygon`, typed `add-pool-package-silkscreen-line`, typed `add-pool-package-silkscreen-rect`, typed `add-pool-package-silkscreen-circle`, typed `add-pool-package-silkscreen-polygon`, typed `create-pool-part`, typed `set-pool-part-metadata`, typed `set-pool-part-parametric`, typed `set-pool-part-orderable-mpns`, typed `set-pool-part-pad-map-entry`, typed `set-pool-part-pad-map`, `set-pool-library-object`, `delete-pool-library-object`, `project query <root> pool-library-objects`, flat MCP `get_pool_library_objects` / `show_pool_library_object`, and canonical `datum.library.*` aliases, with public paths computed as `<pool>/<kind>/<uuid>.json` rather than user-supplied relative paths, typed unit pins rejecting blank names, duplicate pin IDs, and unsupported direction enums, typed symbols rejected unless their referenced unit is present in the resolved model, typed symbol lines rejecting zero-length endpoints and nonpositive stroke widths, typed symbol rectangles rejecting zero-area bounds and nonpositive stroke widths, typed symbol circles rejecting nonpositive radius or stroke width, typed symbol polygons preserving vertices, closed state, and stroke width while rejecting malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices, typed symbol arcs rejecting nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, typed symbol text rejecting blank text while preserving authored position and rotation, typed entities rejected unless their gate unit/symbol pair is consistent, typed padstacks validating circle/rect aperture dimensions before commit, typed packages rejected unless their initial pad references an existing padstack on a positive layer, typed package pads rejecting blank names, duplicate pad IDs, missing padstacks, and nonpositive layer ids, typed package courtyards rejecting zero-area rectangles, malformed polygon vertices, or polygons with fewer than three vertices while preserving the accepted closed polygon, typed package silkscreen lines rejecting zero-length endpoints and nonpositive stroke widths, typed package silkscreen rectangles rejecting zero-area bounds and nonpositive stroke widths, typed package silkscreen circles rejecting nonpositive radius or stroke width, typed package silkscreen polygons preserving vertices, closed state, and stroke width while rejecting malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices, typed parts rejected unless their referenced entity and package exist with a supported lifecycle, typed part metadata edits preserving omitted fields while rejecting empty no-op requests, unsupported lifecycle values, and out-of-range JEP106 manufacturer codes, typed part parametric edits rejecting malformed entries, blank keys, duplicate request keys, or unsupported merge modes, typed part orderable-MPN edits rejecting blank values, duplicate request MPNs, or unsupported merge modes, typed part tag edits rejecting blank values, duplicate request tags, or unsupported merge modes, and typed part pad-map authoring rejected unless the package pad, entity gate, and gate unit pin all resolve, with bulk `merge` / `replace` mode rejecting duplicate request pads before commit. Canonical check aliases now exist as `datum-eda check run`, `check repair-standards`, `check waive`, and `check accept-deviation`, with legacy imported-design checks retained as `check imported`; MCP exposes `datum.check.run`, `datum.check.repair_standards`, `datum.check.waive`, and `datum.check.accept_deviation` over the existing compatibility dispatch methods. Canonical proposal aliases now exist as `datum-eda proposal list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, and `apply`; MCP exposes `datum.proposal.list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept_apply`, and `apply` over canonical CLI-backed compatibility dispatch methods; `proposal preview` / `datum.proposal.preview` dry-runs a stored proposal batch on a cloned resolved model and returns `proposal_preview_v1` with classified `CommitDiff`, affected objects, validation state, and no shard writes for GUI ghost-review consumers. Canonical journal aliases now exist as `datum-eda journal list`, `show`, `undo`, and `redo`; MCP exposes `datum.journal.list`, `show`, `undo`, and `redo` over the existing compatibility dispatch methods. Canonical artifact aliases now exist as `datum-eda artifact generate`, `start-output-job-run`, `cancel-output-job-run`, `list`, `show`, `files`, `preview`, `compare`, `validate`, `export-manufacturing-set`, and `validate-manufacturing-set`; MCP exposes `datum.artifact.generate`, `list`, `show`, `files`, `preview`, `compare`, `validate`, `export_manufacturing_set`, and `validate_manufacturing_set` over compatibility dispatch methods, while flat lifecycle compatibility methods now bridge to the canonical artifact CLI aliases. MCP `tools/call` now wraps canonical `datum.*` success responses in the target `ok/schema/context/result` envelope, derives context fields from raw CLI payloads when present, preserves top-level raw fields for migration compatibility, and returns `ok:false` target envelopes for canonical tool-level failures. Broader `datum-eda query` coverage remains pending; first MCP boundary result normalization now exists for canonical `datum.context.*`, `datum.check.*`, `datum.artifact.*`, `datum.proposal.*`, `datum.journal.*`, `datum.component_instance.*`, and the first resolver-backed `datum.query.*` set, while deeper family-specific semantics beyond the current compatibility bridge remain pending. | CLI and MCP expose one coherent `datum-eda` product taxonomy aligned to substrate nouns, not milestone-era command accumulation. |
MCP route write-surface governance note: public route write-like aliases are now
classified by `scripts/check_mcp_public_taxonomy.py`. Route fixture generators
must declare `generated_fixture_only` boundaries and deterministic fixture-only
write policy, while `datum.route.apply`, `datum.route.apply_selected`, and
`datum.route.apply_proposal_artifact` must carry internal journal/proposal-backed
write-surface evidence in the catalog. The same route apply surfaces now expose
public CLI/MCP wording that says they apply through the proposal journal gateway,
not direct copper mutation, and tests lock that wording.

MCP proposal write-surface governance note: public `datum.proposal.*` aliases
are now fully inventoried as either read/dry-run surfaces or classified write
surfaces. Proposal metadata producers, review-state mutators, and apply gateways
must carry `x_public_write_surface_class` plus evidence metadata in the public
catalog. `scripts/check_mcp_public_taxonomy.py` and
`mcp-server/test_protocol_catalog.py` fail if a future public proposal alias
appears without read/write classification or if a write-capable proposal alias
lacks evidence.

MCP hidden-alias retirement governance note: every hidden compatibility alias
must now carry non-empty `x_canonical_replacements` metadata. The taxonomy gate
and protocol catalog tests reject hidden aliases whose replacements are neither
public catalog tools nor explicit `pending:<datum.* surface>` migration targets,
so compatibility retirement is tied to machine-readable migration paths rather
than prose-only criteria.

| Spec parity / governance | [~] | `specs/SPEC_PARITY.md` and drift gates lock selected current inventories; the parity gate now tracks 13 inventories, including `standards_check_surface` for shipped standards-aware DRC/check-repair markers, `pool_library_surface` for pool/library Operation/CLI/MCP markers, `schematic_connectivity_surface` for connectivity/diagnostic markers, `zone_fill_surface` for generated-evidence ownership of `.datum/zone_fills`, and `gui_supervision_surface` for the read-only GUI supervision snapshot/rendering markers. `scripts/run_migration_proof_gates.sh` runs the named PG-* proof gates from 000D against focused engine/CLI regressions and is invoked from `scripts/run_drift_gates.sh` (the former PG-HARNESS-WIRING self-check was retired 2026-07-02 — CI invokes the runner directly). `scripts/check_schematic_private_writers.py` is now wired into `scripts/run_drift_gates.sh` and fails if migrated schematic or production authoring modules reintroduce direct JSON writers instead of typed journaled `OperationBatch` commits, if generated-evidence manifest paths reintroduce CLI-local `.datum/artifacts` / `.datum/artifact_runs` / `.datum/output_job_runs` / `.datum/zone_fills` writers instead of substrate persistence helpers or journaled generated-evidence staging, if artifact/run CLI evidence users stop calling the expected `persist_artifact_metadata`, `persist_artifact_run`, or `persist_output_job_run` helpers, if the engine-owned generated-evidence helper grows additional raw `std::fs::write` call sites beyond its single temp-file atomic write, if `gui-app` re-imports board-text private writer helpers instead of terminal-prefilled journaled CLI commands, if retired GUI board-text private writer files reappear, or if forward-annotation review state escapes its explicit `.datum/forward_annotation_review/review.json` sidecar helper. The same guard now classifies non-journaled `project new` bootstrap writes, route-strategy fixture/artifact writes, ZoneFill generated-evidence staging through `zone_fill_journal_ops`, the single engine generated-evidence temp-file writer, and generated Gerber/drill/BOM/PnP export file writes as exact-count exceptions, so new source-writing `write_canonical_json` or `std::fs::write` call sites cannot hide among bootstrap/generated output paths without being deliberately classified. | Governance defines which specs are source of truth, how target/current claims are separated, and which gates prevent stale milestone evidence from driving active scope. |

MCP bridge note: flat compatibility methods and canonical aliases now invoke canonical CLI argv for the first resolver query family, proposals, journals, check waivers/deviations, artifact read/export/validation surfaces, OutputJob execution/lifecycle evidence, and producer-specific OutputJob/ManufacturingPlan/PanelProjection proposal builders. Remaining `project ...` bridge calls are direct production authoring compatibility commands rather than proposal authoring paths.

Proposal preview parity note: canonical `datum-eda proposal preview` now uses a
journal-aware dry-run path that stages proposal shard writes, updates preview
source hashes, cleans staging, and leaves promoted shards untouched. Focused
engine coverage asserts preview after-revision, deterministic predicted
transaction UUID, and accepted apply transaction UUID all match.

Replacement proposal migration note: native board-component replacement now has
a proposal-first CLI path. `datum-eda proposal
create-board-component-replacement` creates a draft proposal for one component
using guarded `SetBoardPackagePart`, `SetBoardPackagePackage`, and
`SetBoardPackageValue` operations, reusing the same component-package
materialization payload path as direct journaled package edits. Focused CLI
coverage proves creation leaves the board unchanged until `proposal
accept-apply` applies through the proposal gateway and marks the proposal
`Applied`. MCP exposes the same non-mutating path as
`datum.proposal.create_board_component_replacement`. Batch replacements and
legacy replacement-plan shaped selections now have equivalent proposal-first
producers through `datum-eda proposal create-board-component-replacements` and
`datum-eda proposal create-board-component-replacement-plan`, flat MCP
`create_board_component_replacements_proposal` /
`create_board_component_replacement_plan_proposal`, and canonical
`datum.proposal.create_board_component_replacements` /
`datum.proposal.create_board_component_replacement_plan`. The plan-shaped
bridge accepts repeated selections with `uuid`, optional `package_uuid`,
optional `part_uuid`, and optional `value`, then maps them to the same guarded
replacement operations without mutating board state before review/apply.
Legacy/imported replacement apply MCP aliases remain hidden until their
dispatch paths route through equivalent journal/proposal-backed semantics.

MCP library taxonomy note: raw native library-object authoring now has canonical aliases `datum.library.create_object`, `datum.library.set_object`, and `datum.library.delete_object` over `create_pool_library_object` / `set_pool_library_object` / `delete_pool_library_object`; typed semantic editor aliases now include `datum.library.create_unit` over `create_pool_unit`, `datum.library.set_unit_pin` over `set_pool_unit_pin`, `datum.library.create_symbol` over `create_pool_symbol`, `datum.library.add_symbol_line` over `add_pool_symbol_line`, `datum.library.add_symbol_rect` over `add_pool_symbol_rect`, `datum.library.add_symbol_circle` over `add_pool_symbol_circle`, `datum.library.add_symbol_polygon` over `add_pool_symbol_polygon`, `datum.library.add_symbol_arc` over `add_pool_symbol_arc`, `datum.library.add_symbol_text` over `add_pool_symbol_text`, `datum.library.set_symbol_pin_anchor` over `set_pool_symbol_pin_anchor`, `datum.library.create_entity` over `create_pool_entity`, `datum.library.create_padstack` over `create_pool_padstack`, `datum.library.create_package` over `create_pool_package`, `datum.library.create_footprint` over `create_pool_footprint`, `datum.library.set_footprint_pad` over `set_pool_footprint_pad`, `datum.library.set_footprint_courtyard_rect` over `set_pool_footprint_courtyard_rect`, `datum.library.set_footprint_courtyard_polygon` over `set_pool_footprint_courtyard_polygon`, `datum.library.add_footprint_silkscreen_line` over `add_pool_footprint_silkscreen_line`, `datum.library.add_footprint_silkscreen_rect` over `add_pool_footprint_silkscreen_rect`, `datum.library.add_footprint_silkscreen_circle` over `add_pool_footprint_silkscreen_circle`, `datum.library.add_footprint_silkscreen_polygon` over `add_pool_footprint_silkscreen_polygon`, `datum.library.set_package_pad` over `set_pool_package_pad`, `datum.library.set_package_courtyard_rect` over `set_pool_package_courtyard_rect`, `datum.library.set_package_courtyard_polygon` over `set_pool_package_courtyard_polygon`, `datum.library.add_package_silkscreen_line` over `add_pool_package_silkscreen_line`, `datum.library.add_package_silkscreen_rect` over `add_pool_package_silkscreen_rect`, `datum.library.add_package_silkscreen_circle` over `add_pool_package_silkscreen_circle`, `datum.library.add_package_silkscreen_polygon` over `add_pool_package_silkscreen_polygon`, `datum.library.add_package_silkscreen_arc` over `add_pool_package_silkscreen_arc`, `datum.library.add_package_silkscreen_text` over `add_pool_package_silkscreen_text`, `datum.library.add_package_model_3d` over `add_pool_package_model_3d`, `datum.library.create_part` over `create_pool_part`, `datum.library.set_part_metadata` over `set_pool_part_metadata`, `datum.library.set_part_pad_map_entry` over `set_pool_part_pad_map_entry`, and `datum.library.set_part_pad_map` over `set_pool_part_pad_map`. Resolver-backed native pool-library inspection now exists through `datum-eda project query <root> pool-library-objects`, flat MCP `get_pool_library_objects` / `show_pool_library_object`, and canonical `datum.library.list_objects` / `datum.library.show_object`; pool-model blob verification now exists through `datum-eda project query <root> pool-models`, flat MCP `get_pool_model_blobs`, and canonical `datum.library.pool_models`; flat MCP `gc_pool_model_blobs` exposes `datum-eda project gc-pool-models` dry-run/apply cleanup for orphaned regular hash-matching blobs. The aliases dispatch through the CLI path, inherit the CLI `pool` default for writes, keep public object paths computed as `<pool>/<kind>/<uuid>.json` rather than user-supplied relative paths, reject typed unit pins with blank names, duplicate pin IDs, or unsupported direction enums, reject typed symbols whose referenced unit is absent from the resolved model, reject typed symbol lines with zero-length endpoints or nonpositive stroke widths while preserving authored endpoints and stroke width, reject typed symbol rectangles with zero-area bounds or nonpositive stroke widths while preserving authored bounds and stroke width, reject typed symbol circles with nonpositive radius or stroke width while preserving authored center, radius, and stroke width, reject typed symbol polygons with malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices while preserving vertices, closed state, and stroke width, reject typed symbol arcs with nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, reject typed symbol text with blank text while preserving authored position and rotation, reject typed symbol pin anchors whose symbol unit or referenced unit pin is missing while preserving the authored unit-pin UUID and symbol-local position, place schematic symbols with pins when `--lib-id` is a pool symbol UUID with authored pin anchors while preserving arbitrary non-UUID lib IDs as unresolved compatibility identifiers, reject typed entities whose initial gate references a missing unit, missing symbol, or symbol/unit mismatch, reject invalid typed padstack aperture/drill dimensions before journal commit, reject typed packages whose initial pad references a missing padstack or nonpositive layer, reject typed first-class footprints whose package is missing, reject typed footprint pads with blank names, missing padstacks, or nonpositive layer ids, reject typed footprint courtyards with zero-area rectangles, malformed polygon vertices, or polygons with fewer than three vertices while preserving accepted closed polygons, reject typed footprint silkscreen lines with zero-length endpoints or nonpositive stroke widths, reject typed footprint silkscreen rectangles with zero-area bounds or nonpositive stroke widths, reject typed footprint silkscreen circles with nonpositive radius or stroke width, reject typed footprint silkscreen polygons with malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices while preserving vertices, closed state, and stroke width, reject typed package pads with blank names, duplicate pad IDs, missing padstacks, or nonpositive layer ids, reject typed package courtyards with zero-area rectangles, malformed polygon vertices, or polygons with fewer than three vertices while preserving accepted closed polygons, reject typed package silkscreen lines with zero-length endpoints or nonpositive stroke widths, reject typed package silkscreen rectangles with zero-area bounds or nonpositive stroke widths, reject typed package silkscreen circles with nonpositive radius or stroke width, reject typed package silkscreen polygons with malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices while preserving vertices, closed state, and stroke width, reject typed package silkscreen arcs with nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, reject typed package silkscreen text with blank text while preserving authored position and rotation, reject typed package 3D model paths that are blank, absolute, or traversal paths and malformed transform JSON while preserving the accepted model path and transform, reject typed parts whose referenced entity/package are missing or whose lifecycle is unsupported, reject typed part metadata edits with empty no-op requests, unsupported lifecycle values, or out-of-range JEP106 manufacturer codes, reject typed part parametric edits with malformed entries, blank keys, duplicate request keys, or unsupported merge modes, reject typed part orderable-MPN edits with blank values, duplicate request MPNs, or unsupported merge modes, reject typed part tag edits with blank values, duplicate request tags, or unsupported merge modes, and reject typed part pad-map authoring whose package pad, entity gate, or gate unit pin is missing, including bulk merge/replace requests with duplicate pads. Richer symbol graphics beyond lines/rectangles/circles/text/arcs/polygons, footprint graphics/process-policy authoring beyond pads/courtyards/silkscreen lines/rectangles/circles/polygons, and package silkscreen primitives beyond lines/rectangles/circles/text/arcs/polygons remain pending.

Footprint silkscreen extension note: direct CLI/MCP library authoring now also
includes `datum.library.add_footprint_silkscreen_rect`,
`datum.library.add_footprint_silkscreen_circle`, and
`datum.library.add_footprint_silkscreen_polygon` over the matching
`add_pool_footprint_silkscreen_*` compatibility methods. These surfaces reject
zero-area rectangles, nonpositive circle radii, malformed polygon vertices,
nonpositive widths, too few vertices, and closed polygons with fewer than three
vertices while preserving accepted vertices, closed/open state, and stroke
width. Footprint graphics/process-policy authoring beyond pads, courtyards, and
silkscreen lines/rectangles/circles/polygons remains pending.

MCP library proposal note: canonical `datum.proposal.create_pool_library_object`
now exposes the review path for raw AI/tool-authored pool objects, and
`datum.proposal.create_pool_unit`, `datum.proposal.create_pool_symbol`,
`datum.proposal.create_pool_entity`, and
`datum.proposal.create_pool_padstack`, and
`datum.proposal.create_pool_package`, and
`datum.proposal.create_pool_footprint`, and
`datum.proposal.set_pool_footprint_pad`,
`datum.proposal.set_pool_footprint_courtyard_rect`,
`datum.proposal.set_pool_footprint_courtyard_polygon`,
`datum.proposal.add_pool_footprint_silkscreen_line`, and
`datum.proposal.add_pool_footprint_silkscreen_rect`,
`datum.proposal.add_pool_footprint_silkscreen_circle`, and
`datum.proposal.add_pool_footprint_silkscreen_polygon`, and
`datum.proposal.set_pool_package_pad`,
`datum.proposal.set_pool_package_courtyard_rect`, and
`datum.proposal.set_pool_package_courtyard_polygon` expose the first typed
semantic unit, symbol, entity, padstack, package, package-pad, first-class
Footprint create, Footprint-pad, Footprint-courtyard, Footprint-silkscreen
line/rectangle/circle/polygon, and package-courtyard producers. They dispatch to
`datum-eda proposal create-pool-library-object`,
`datum-eda proposal create-pool-unit`, and
`datum-eda proposal create-pool-symbol`, and
`datum-eda proposal create-pool-entity`, and
`datum-eda proposal create-pool-padstack`, and
`datum-eda proposal create-pool-package`, and
`datum-eda proposal create-pool-footprint`, and
`datum-eda proposal set-pool-footprint-pad`, and
`datum-eda proposal add-pool-footprint-silkscreen-line`, and
`datum-eda proposal add-pool-footprint-silkscreen-rect`, and
`datum-eda proposal add-pool-footprint-silkscreen-circle`, and
`datum-eda proposal add-pool-footprint-silkscreen-polygon`, and
`datum-eda proposal set-pool-package-pad`, and
`datum-eda proposal set-pool-package-courtyard-rect`, and
`datum-eda proposal set-pool-package-courtyard-polygon`, write only proposal
metadata, and leave `datum.library.*` as direct journaled CLI surfaces for
manual/non-automated authors.

ProjectResolver mutation note: board routing/net, netclass/dimension, and
layout mutation validation plus post-commit readbacks now load
resolver-materialized board state rather than promoted `board.json`; focused
board-net, board-netclass, board-dimension, board-text, and board-keepout
regression coverage restores stale promoted board files after journaled
creation, then proves the authored objects remain queryable and editable
through journal replay.

Schematic resolver note: native schematic query, connectivity query,
connectivity mutation, text/drawing mutation, symbol mutation, and schematic
proposal entrypoints now load resolver-materialized project/schematic roots
before using materialized sheet helpers, so stale promoted schematic roots no
longer define the sheet map for authored schematic operations. Focused label,
wire, junction, port, bus, noconnect, text, drawing, and symbol tests pass after
the migration, including stale promoted sheet replay coverage. Schematic
hierarchy/net/diagnostic construction now materializes sheet definitions through
`ProjectResolver` instead of reading promoted definition files directly; focused
hierarchy coverage removes a promoted journal-created definition file and still
resolves the sheet-instance port link from the journal. Pool replay was
also tightened so `SetPoolLibraryObject` and pool model attachment operations
target the intended `relative_path` instead of broadcasting across all touched
pool shards; symbol pin materialization from typed pool unit/symbol shards and
engine pool replay coverage now pass.

Pool resolver note: native pool reference queries and pool library authoring
entrypoints now load resolver-materialized project manifests before inspecting
`manifest.pools`, so stale promoted `project.json` cannot cause duplicate pool
ref operations after a journaled pool introduction. Focused pool resolver
coverage restores a pre-pool project manifest after the first typed pool unit
create, creates a second unit, proves only one `AddProjectPoolRef` exists in
the journal, and verifies `project query pools` reads the materialized pool
reference. Pool-library mutation preimage reads now also use
`ProjectResolver::materialized_source_shard_value_by_relative_path` for
project-owned objects instead of reading promoted pool JSON directly; external
`--from-json` inputs remain raw file reads. Focused coverage removes a promoted
journal-created `pool/parts/<uuid>.json` file, edits part metadata, and proves
the mutation uses the journal-materialized previous object. Board-component pool
materialization now resolves project-local package and padstack shards through
`ProjectResolver` before falling back to existing promoted files for fixture and
external-pool compatibility; focused coverage creates a package/padstack through
journaled typed pool commands, removes the promoted `pool/packages/<uuid>.json`
and `pool/padstacks/<uuid>.json` files, places a board component, and proves
the persisted component pad geometry/drill data came from journal replay.

Route/default-stackup resolver note: route proposal export/apply and
`add-default-top-stackup` now load resolver-materialized project/board state
instead of stale promoted roots. Mutating route proposal artifacts still require
embedded accepted proposal metadata, while verified no-op policy artifacts
without draw-track actions apply as zero-action artifacts. Focused route
proposal, route apply, board stackup, format, check, and full drift gates pass.

Schematic sheet/root resolver note: schematic sheet mutations now load
resolver-materialized schematic root state before create/delete/rename sheet,
create definition, and create/move/delete/bind/unbind sheet-instance
operations, so journal-created sheet topology remains editable even when the
promoted schematic root is stale. Resolver replay now also recovers
journal-created schematic definition shards when stale promoted roots omit their
definition map entries, matching existing sheet-shard recovery semantics.
Definition replay now covers missing/unreadable promoted files plus
stale-promoted delete suppression. Focused sheet CLI tests, engine
schematic-sheet tests, schematic-definition replay tests, format, check, and
full drift gates pass.

Rules report resolver note: granular project-rule create/set/delete now reload
post-commit report state through the resolver-materialized rules root, matching
whole-root `set-project-rules` and query behavior. Focused coverage restores a
stale promoted `rules.json` before set/delete and asserts the mutation reports
still return resolver-derived `rule_count` and `rules_object_revision`; focused
rules tests, format, check, and full drift gates pass.

Forward-annotation/native-inspect resolver note: forward-annotation artifact
export, selected export, compare, filter, apply, review import, and review
replace now load resolver-materialized project identity/state before artifact
identity checks and report/export metadata. Native project inspect now reports
resolver-materialized schematic/board/rules state. Focused coverage restores a
stale promoted `project.json` after a journaled project-name edit and proves
forward-annotation export writes the resolved project name, and restores a
stale promoted project after reference-preserving UUID identity drift to prove
artifact compare reports `drifted` instead of `stale` when only the derived
forward-annotation `action_id` changes due to reference rename. Focused
coverage also restores a
stale promoted `board.json` after a journaled pad create and proves inspect
reports the materialized pad count; forward-annotation artifact tests, inspect
tests, format, check, and full drift gates pass.

Inventory resolver note: BOM/PnP export, validation, comparison, and
panel-PnP report surfaces now load resolver-materialized project/board state
for report metadata while row generation continues to use the resolver-backed
inventory model path. Focused BOM and PnP export coverage restores stale
promoted `board.json` after journaled component placement and proves exported
rows plus report counts come from materialized board state; BOM/PnP test
groups, format, check, and full drift gates pass.

Manufacturing wrapper resolver note: manufacturing-set inspect, validate, and
compare wrappers now have focused stale-promoted-board regressions. Each test
journals board edits, restores stale `board.json`, then proves wrapper metadata,
expected artifact sets, validation, and comparison results come from
resolver-materialized board state rather than promoted root files.

Generated-evidence helper visibility note: direct generated-evidence
persistence helpers (`persist_artifact_metadata`, `persist_output_job_run`,
`persist_artifact_run`, `persist_check_run`, and `persist_zone_fill`) are no
longer public substrate exports. They are internal `pub(super)` helpers with
explicit `#[allow(dead_code)]` where production command paths persist those
families through journaled generated-evidence operations. The private-writer
guard now forbids those helper names in `substrate/mod.rs`, requires internal
visibility, and rejects direct `persist_zone_fill` use in CLI command and test
files. The former CLI filled-zone fixture now seeds evidence through the real
`project fill-zones` journaled command path. Focused artifact metadata/replay,
ZoneFill DRC/fill/replay tests, format, check, private-writer guard, and full
drift gates pass.

Generated-evidence scope note: journal staging now rejects generated-evidence
`Set*` operations whose project/model revision scope does not match the current
resolved model. `OutputJobRun`, `ArtifactRun`, `CheckRun`, `ArtifactMetadata`,
and `ZoneFill` must be current before promotion; delete operations remain
available for removing stale generated evidence by id/schema. Focused engine
coverage rejects stale model revisions for all five evidence families and
wrong-project run evidence, while replay coverage still recovers accepted
historical evidence.

GUI terminal context note: terminal context snapshots now expose production
visibility at the top level for terminal-launched agents. The envelope derives
visible artifact IDs, output job IDs, artifact file paths, latest output-job
run/job/artifact IDs, and focused artifact/file from `ProductionStatus` while
continuing to embed the full production payload. Focused PTY terminal coverage
proves `.datum/gui-terminal-context.json` carries those fields from GUI
production state.

Generated-evidence delete replay note: journal replay now has focused
regressions proving later generated-evidence delete operations suppress stale
promoted `.datum` shards for `OutputJobRun`, `ArtifactRun`, `CheckRun`,
`ArtifactMetadata`, and `ZoneFill`. The ZoneFill case preserves the derived
authored-zone contract by resolving to `Unfilled` state after evidence deletion
instead of resurrecting stale generated copper islands.

ZoneFill schema note: generated ZoneFill evidence now carries explicit
`schema_version: 1` in persisted payloads, command/query output, journal replay,
and standards repair proposal payloads. Resolver validation rejects unsupported
future schema versions before they can become renderable copper, while legacy
promoted fills missing the field remain readable through the version-1 serde
default. Focused engine and CLI regressions cover versioned persistence, legacy
compatibility, unsupported-version rejection, replayed source-shard metadata,
and public command output.

ZoneFill standards repair note: standards repair generation now collects
`zone_fill_unfilled` and `zone_fill_stale` findings, recomputes the target zone
through the bounded solver, and creates check-source `SetZoneFill` repair
proposals only when the result is honestly `Filled`. Unsupported fills are
skipped rather than converted into fake copper. Focused regressions cover
unfilled proposal generation, stale proposal generation with previous evidence
preserved, unsupported skip behavior, and accept/apply clearing the repaired
zone's ZoneFill findings.

ZoneFill thermal-relief correction note: bounded fill generation now supports
thermal-relief zones when no same-net pad/via anchors intersect the fill, and
records provenance that thermal relief was requested but no spokes were needed.
If a same-net pad/via anchor intersects the thermal zone, the fill remains
`Unsupported` until real thermal spoke/isolation geometry exists, preserving the
Honesty Rule instead of emitting fake copper.

CheckRun coverage correction note: native CheckRun coverage now reports shipped
DRC `clearance_copper` and `silk_clearance_copper` rule families as normal
profile-aware coverage entries. They are `evaluated` for DRC/native/release
profiles and `filtered_by_profile` for profiles that intentionally exclude DRC,
rather than stale placeholder `not_implemented` coverage rows.

Copper-clearance standards repair note: standards repair generation now handles
simple `clearance_copper` findings for same-layer parallel horizontal/vertical
track pairs by creating a check-source `SetBoardTrack` draft proposal that
offsets one track to the governing netclass clearance. Non-parallel and
topology-aware reroutes remain deliberately skipped rather than fake-fixed.
Focused CLI coverage proves the proposal links back to the finding fingerprint,
accepts/applies through the generic proposal gateway, moves the selected track,
and clears the repaired track's `clearance_copper` finding in the follow-up
CheckRun.

Silkscreen standards repair note: standards repair generation now collects
`silk_clearance_copper` findings, identifies the board text object involved in
the DRC finding, and creates a check-source `SetBoardText` draft proposal that
moves the silkscreen text away from copper without mutating copper geometry.
Focused CLI coverage proves the proposal links back to the finding fingerprint
and remains gated by proposal acceptance. Apply-path coverage now accepts and
applies the generated silkscreen repair through the generic proposal gateway,
verifies the proposal reaches `Applied`, verifies the board-text position moves
away from copper, and verifies the repaired text's `silk_clearance_copper`
finding disappears from the follow-up CheckRun.

Peer aperture standards repair note: process-aperture repair generation now
handles `pad_process_aperture_inconsistent_with_peer_footprint` by copying the
unique majority same-package mask/paste policy onto the outlier pad through
`SetBoardPad`. Ambiguous equal-count peer policies are skipped rather than
guessed. Focused regressions cover proposal generation, check-authored
`SetBoardPad` payloads, accept/apply, and clearing the repaired pad's peer
aperture finding.

OutputJob replay note: real CLI `project run-output-job` execution now has
broader generated-evidence replay regressions. The tests create and run Gerber,
single-scope drill, aggregate BOM/PnP, and manufacturing-set OutputJobs, record
the authored `ModelRevision`, delete the promoted
`.datum/output_job_runs/<run>.json` shard, resolve the project, and prove the
OutputJobRun is recovered from the journal without mutating the authored
revision. The manufacturing-set path also proves generated child Gerber output
does not author a separate Gerber OutputJob.

Run-evidence schema note: generated `OutputJobRun` and `ArtifactRun` payloads
now carry explicit `schema_version: 1` in persisted payloads, command/list/show
output, and journal replay. Resolver validation rejects unsupported future
versions before inserting run evidence, while legacy promoted run evidence
missing the field remains readable through version-1 serde defaults. Focused
engine and CLI regressions cover versioned persistence, legacy compatibility,
unsupported-version rejection, replayed source-shard metadata, `project
run-output-job` output, and ad hoc `artifact generate/list/show` output.

Source-shard metadata hardening note: value-backed source-shard refs now use a
canonical builder for replay and staged-write metadata. The builder validates
kind/path ownership, rejects unsupported future schema versions, derives pool
taxon metadata from the canonical relative path, and preserves schema version,
authority, dirty-state, and content-hash metadata consistently across replayed
and immediately committed shards. Focused engine and CLI regressions prove
journal-created pool symbols keep `PoolSymbol` taxonomy plus schema version
after commit, and missing journal-recovered pool symbols surface through
`resolve-debug` as `AuthoredDesign` / `PoolSymbol` / `Missing`.
Authored production records now use the same taxon path for
`ManufacturingPlan`, `PanelProjection`, and `OutputJob`; focused resolver and
CLI regressions prove missing journal-recovered production shards expose their
concrete taxon with `AuthoredDesign` / `Missing` state, and public
`resolve-debug` coverage now proves unreadable promoted authored production
shards report `Unknown` while preserving concrete taxon and authority metadata.
MCP `datum.query.source_shards` parity now covers the same missing
journal-recovered pool-symbol case through the real CLI bridge.

Source-shard metadata hardening note: byte-backed promoted-shard refs now use
the same canonical ownership/taxon path as value-backed replay refs. The shared
byte builder rejects cross-authority paths before constructing a `SourceShardRef`
and derives concrete pool taxon metadata when applicable. Promoted ArtifactRun
and CheckRun discovery now route through the byte builder instead of hand-built
refs, keeping generated-evidence metadata construction aligned with production
and artifact promoted-shard readers.

ImportMap replay note: resolver journal replay now applies
`CreateImportMapShard` and `DeleteImportMapShard` to both source-shard metadata
and the derived `model.import_map` table. Focused coverage removes a promoted
sidecar after journaled create and proves the import-key table still recovers;
delete coverage restores a stale promoted `.datum/import_map/kicad.json` sidecar
after a journaled delete and proves the deleted sidecar cannot resurrect stale
import-key entries or `SourceShardKind::ImportMap` metadata.

KiCad board ImportMap note: board import-map mode now covers board footprints,
pads, routed segments, vias, and zones. `board_footprint_import_key`,
`board_pad_import_key`, `board_segment_import_key`, `board_via_import_key`, and
`board_zone_import_key` provide stable key contracts,
`import_board_document_with_import_map` reuses mapped Datum identities when
present, and otherwise allocates deterministic non-source UUIDs while legacy
`import_board_document` preserves source UUID behavior. Pad fallback identities
derive from the source footprint UUID and pad name, not the mapped package UUID,
so pad identity remains stable across package remaps. Focused engine coverage
proves mapped reuse and deterministic allocation for each family.
Project-root KiCad board reimport now writes a full same-source ImportMap
lifecycle sidecar when the latest source drops a previously mapped board object,
preserving current board keys as `active` while marking absent keys
`missing_in_source`. Board ImportMap entries now expose source-facing
`source_object_ref` values such as `board-footprint:<source_uuid>` and
`board-pad:<source_uuid>` instead of UUID-only refs while preserving
`source_hash` and `source_path`.

KiCad schematic ImportMap note: schematic import-map provenance now covers
source-backed symbols, wires, junctions, labels, buses, bus entries,
no-connect markers, schematic text, bounded drawing primitives, generated sheet
definitions, generated sheet instances, and deterministic sheet ports.
`schematic_*_import_key` contracts provide stable import keys,
`import_schematic_document_with_import_map_identities` reuses mapped Datum
identities when present and otherwise allocates deterministic non-source UUIDs,
while legacy `import_schematic_document` preserves source UUID behavior.
Project-root `project import-kicad-schematic` creates native sheets when
needed, journals supported symbols, wires, junctions, labels, hierarchical
ports, buses, bus entries, no-connect markers, sheet definitions, sheet
instances, schematic text, and bounded drawing primitives through typed
schematic operations, persists one `CreateImportMapShard` sidecar for the
supported schematic identities, and reuses journal-recovered ImportMap entries
on reimport. Project-root KiCad schematic reimport now writes a full same-source
ImportMap lifecycle sidecar when the latest source drops a previously mapped
schematic object, preserving current schematic keys as `active` while marking
absent keys `missing_in_source`. Schematic ImportMap entries now expose
source-facing `source_object_ref` values such as
`schematic-symbol:<source_uuid>` and `schematic-wire:<source_uuid>` instead of
UUID-only refs while preserving `source_hash` and `source_path`. Broader
schematic fidelity and Eagle schematic provenance remain out of scope for this
bounded slice.

Eagle library ImportMap note: project-root `project import-eagle-library`
imports an Eagle `.lbr` into a native project-local pool through journaled
`CreatePoolLibraryObject` operations for units, symbols, entities, parts,
packages, and padstacks, adds the pool reference when needed, persists one
`CreateImportMapShard` sidecar for those pool identities, verifies every
created object resolves after commit, reuses journal-recovered mappings on
reimport, writes a full same-source ImportMap lifecycle sidecar when the latest
library source drops a previously mapped pool object, preserving current pool
keys as `active` while marking absent keys `missing_in_source`, and supports
source-facing `source_object_ref` provenance for pool units, symbols,
devicesets, devices, packages, and padstacks instead of UUID-only refs. It also
supports undo through the same pool/import-map journal inverses. Eagle board
and schematic import remain outside this bounded slice.

Library validation note: `project validate` now checks standalone
`pin_pad_maps.mappings` entries against the referenced part, not only the
`pin_pad_maps[*].part` envelope. Mapping keys resolve to first-class footprint
pads when `PinPadMap.footprint` is present and fall back to package pads for
package-bound maps; each value must name a gate on the part entity and a pin on
that gate's unit.

Private-writer migration note: ComponentInstance, relationship/variant, and
production journal staging no longer own per-family `.datum/stage` file writes.
Their staging paths now compute the family-specific relative path/wrapper and
then route new-shard persistence through shared `journal::stage_new_shard_write`.
The private-writer guard now expects zero direct `std::fs::write` calls in those
family staging files, shrinking the engine source-stage direct-writer exception
surface to the journal owner.

Rules substrate note: native project rule payload validation now parses the
persisted `scope` field as the engine `RuleScope` AST and runs the existing
structural scope validator. Invalid scope expressions, including nil UUID
references, are rejected before journal staging and are reported by `project
validate` against `rules/rules.json`.

Production replay dirty-state note: `journal_replay_recovers_missing_production_shards`
now asserts that journal-recovered authored production records expose exact
`SourceShardRef` entries for `.datum/manufacturing_plans`, `.datum/panel_projections`,
and `.datum/output_jobs` with `SourceShardDirtyState::Missing`, locking the same
missing-promoted-file visibility for production source shards that generated
evidence recovery already reports.

Authored identity/relationship replay dirty-state note: ComponentInstance,
Relationship, and VariantOverlay journal replay now asserts that missing
promoted `.datum/component_instances`, `.datum/relationships`, and
`.datum/variants` shards recover with exact `SourceShardRef` paths and
`SourceShardDirtyState::Missing`, aligning non-generated identity/intent
sidecars with the production and generated-evidence dirty-state oracle.
Public `project query <root> resolve-debug` coverage now asserts the same
journal-recovered identity/relationship sidecars expose `AuthoredDesign`
authority, concrete `ComponentInstance` / `Relationship` / `VariantOverlay`
taxons, and `Missing` dirty state for terminal-launched agents and GUI refresh
surfaces.
GUI protocol `SourceShardStatusSummary` now counts those same missing
ComponentInstance, Relationship, and VariantOverlay sidecars as attention
items, preserving path, kind, taxon, authority, and dirty-state fields for
workspace health rendering and terminal context handoff.
CLI `datum-eda context refresh` now projects the same missing
ComponentInstance, Relationship, and VariantOverlay attention rows into the
terminal discovery envelope, preserving snake_case kind, taxon, authority, and
dirty-state fields for terminal-launched agents.
Unreadable promoted ComponentInstance, Relationship, and VariantOverlay shards
now recover through journal replay instead of aborting `resolve-debug`; focused
public CLI coverage replaces each promoted file with a directory and proves
path, kind, concrete taxon, `AuthoredDesign` authority, and `Unknown`
dirty-state metadata survive recovery.
Unreadable promoted ImportMap sidecars now recover through the same journal
replay path: focused `resolve-debug` coverage replaces
`.datum/import_map/kicad.json` with a directory and proves the recovered
`ImportMap` source shard remains visible as `SidecarMetadata` /
`ImportMap` / `Unknown`.
Unreadable promoted forward-annotation review sidecars now recover through
journal materialization as well: replay skips unreadable promoted
`.datum/forward_annotation_review/review.json` files, materialized shard access
reconstructs the review from the retained journal write, and focused public
`resolve-debug` coverage proves the recovered sidecar remains visible as
`SidecarMetadata` / `ForwardAnnotationReview` / `Unknown`.
ProposalMetadata sidecars now have journal replay parity with the other
sidecar families: resolver replay rebuilds missing or unreadable
`.datum/proposals/<uuid>.json` source shards from retained journal writes,
applies journal create/set/delete operations to the resolved proposal map,
marks stale promoted proposal files `Dirty`, suppresses stale files after
journaled deletes, and materialized shard access reconstructs recovered
proposal metadata. Engine regressions cover `Missing`, `Dirty`, `Unknown`, and
delete-suppression cases; focused public `resolve-debug` coverage proves an
unreadable proposal sidecar remains visible as `SidecarMetadata` /
`ProposalMetadata` / `Unknown`.
Unreadable promoted pool/library authored shards now have the same public
coverage: focused `resolve-debug` coverage replaces a journal-recovered
`pool/symbols/<uuid>.json` file with a directory and proves the recovered
source shard remains visible as `AuthoredDesign` / `PoolSymbol` / `Unknown`.
Pool directory discovery and materialized object refresh now skip unreadable
promoted pool entries instead of aborting before journal replay can rebuild the
shard. MCP `datum.query.source_shards` parity now covers both missing and
unreadable journal-recovered pool symbols, proving the normalized result
surface preserves `path`, `kind`, `taxon`, `authority`, and `dirty_state`.
Unreadable promoted schematic sheets now also recover through journal
materialization: referenced schematic shard discovery and ComponentInstance
join-key derivation skip unreadable promoted sheet files instead of aborting,
and focused public `resolve-debug` coverage proves a journal-created
`schematic/sheets/<uuid>.json` directory remains visible as
`SchematicSheet` / `AuthoredDesign` / `Unknown`.
Unreadable promoted schematic definitions now follow the same replay path:
missing or unreadable journal-created `schematic/definitions/<uuid>.json`
shards materialize from the accepted journal, stale promoted definition files
are suppressed after journaled deletes, and focused public `resolve-debug`
coverage proves recovered definition directories remain visible as
`SchematicDefinition` / `AuthoredDesign` / `Unknown`. Native MCP
`datum.query.source_shards` parity now covers the same unreadable
schematic-definition case so the canonical tool surface cannot drift behind
the CLI resolver diagnostic surface.
Unreadable promoted generated-evidence shards now have public `Unknown`
dirty-state coverage: focused `resolve-debug` and `context refresh` regressions
replace a journal-recovered ArtifactMetadata file with a directory and prove the
`unknown` bucket plus attention row preserve path, kind, taxon, authority, and
dirty-state fields for terminal-launched agents.
Generated-evidence replay now also tolerates unreadable promoted files during
later journal-prefix validation: the replay paths for OutputJobRun,
ArtifactRun, CheckRun, ArtifactMetadata, and ZoneFill skip unreadable promoted
values and let journal operations rebuild the materialized source shard; focused
public `resolve-debug` coverage proves a journal-recovered ArtifactMetadata
directory remains `GeneratedEvidence` / `ArtifactMetadata` / `Unknown` after a
subsequent journaled project-name transaction.
Unreadable promoted authored production shards now have matching public
`Unknown` coverage: focused `resolve-debug` regressions replace
journal-recovered ManufacturingPlan, PanelProjection, and OutputJob files with
directories and prove path, kind, concrete taxon, `AuthoredDesign` authority,
and `Unknown` dirty-state metadata survive recovery.

Production create idempotency note: `create-manufacturing-plan`,
`create-panel-projection`, and `create-gerber-output-job` now answer
already-exists checks from `ProjectResolver` instead of promoted shard
`Path::exists()` reads. Focused regressions remove the promoted production shard
after the first journaled create, rerun the same create command, and prove the
CLI returns the resolver-replayed object with `created: false` without appending
a duplicate journal transaction.

Production semantic-integrity note: production create/set operations now reject
dangling authored references before mutation. PanelProjection board instances
must target the project board, ManufacturingPlan and OutputJob board/panel
targets must resolve to the project board or an existing/current panel, optional
variants must exist, and OutputJobs cannot reference missing ManufacturingPlans.
Promoted ManufacturingPlan, PanelProjection, and OutputJob shard readers also
reject filename/payload UUID mismatches with resolver diagnostics and exclude
the bad shard from the model.

Production proposal-policy note: the engine commit gateway now rejects
automated direct production CRUD for `Tool`/`Assistant` provenance batches that
create/set/delete authored ManufacturingPlan, PanelProjection, or OutputJob
records. Accepted production proposals use an explicit internal
accepted-proposal commit context, so AI/agent-authored proposals can still be
reviewed and applied without allowing silent direct mutation.

Terminal attach/detach note: protocol terminal tabs now expose an explicit
`attached` flag separate from process `status`, session activation writes
`detached` and `attached` terminal lifecycle events for the old/new PTY
sessions, and the dock renders detached tabs distinctly while preserving the
running shell process. The dock now exposes a first-class `DETACH` control that
marks the active tab detached without terminating the PTY, blocks raw
keyboard/paste input until reattached, and lets a session-tab click reattach the
same shell.

Resolver raw-load governance note: `scripts/check_resolver_raw_loads.py` is now
wired into `scripts/run_drift_gates.sh` and fails if CLI project surfaces add any
raw `load_native_project(root)?` bypass.
The KiCad footprint import bootstrap exception has been retired: pool-ref
existence and priority now come from the resolver-materialized project manifest,
with stale-promoted `project.json` coverage proving a second import does not
journal a duplicate `AddProjectPoolRef`. The legacy forward-annotation review
fallback exception has also been retired: when no review sidecar exists, embedded
manifest review compatibility now reads the resolver-materialized project
manifest and focused coverage proves the fallback remains readable without
creating a sidecar. Forward-annotation review persistence itself is now
journal-owned through `Operation::SetForwardAnnotationReview` and
`Operation::DeleteForwardAnnotationReview`; the fixed
`.datum/forward_annotation_review/review.json` sidecar remains a compatibility
read model, but direct sidecar writes are retired and replay can recover the
sidecar from the journal. `project query <root> resolve-debug` now reports the
journal-recovered review sidecar as `SourceShardKind::ForwardAnnotationReview`,
`SidecarMetadata`, and `Missing` when the promoted sidecar file is absent,
keeping public source-shard health reporting aligned with the new sidecar family.
GUI protocol coverage also proves the resolver-backed `SourceShardStatusSummary`
counts that missing journal-recovered review sidecar as both `missing` and
attention-worthy for the project panel.
The former core-helper bootstrap exception has also been
retired: `load_native_project_with_resolved_board_and_model` now seeds
`LoadedNativeProject` from resolver-materialized manifest, schematic, board, and
rules shards instead of loading promoted roots first. The raw-load guard is now
zero-exception.

MCP schematic query taxonomy note: canonical `datum.query.sheets`, `symbols`, `symbol_fields`, `labels`, `ports`, `buses`, `bus_entries`, `noconnects`, `schematic_nets`, `connectivity_diagnostics`, and `design_rules` now expose the existing compatibility schematic query methods through the `datum.*` envelope. `datum.query.hierarchy` is now path-aware for native projects and dispatches to `datum-eda project query <path> hierarchy` when `path` is supplied, while preserving the legacy open-session fallback. This gives GUI panels and agents stable read-side schematic context, including connectivity-derived hierarchy links from persisted sheet-instance port bindings.

MCP proposal taxonomy note: producer-specific production proposal builders now have canonical aliases under `datum.proposal.*`: `create_panel_projection`, `update_panel_projection`, `delete_panel_projection`, `create_manufacturing_plan`, `update_manufacturing_plan`, `delete_manufacturing_plan`, `create_output_job`, `update_output_job`, and `delete_output_job`. The matching canonical CLI proposal aliases dispatch to the same proposal-gateway code path as the compatibility `project ... --as-proposal` commands. The MCP aliases dispatch to the same flat compatibility proposal builders and return the canonical target envelope.

MCP production authoring taxonomy note: public `datum.manufacturing.*`
create/update/delete aliases and `datum.output_job.create_gerber_set`,
`create`, `update`, and `delete` now dispatch to proposal builders and carry
`proposal_metadata_write` evidence metadata. `datum.output_job.run` remains an
execution surface. The public taxonomy guard and protocol catalog tests now fail
if those production authoring aliases regress to direct CRUD dispatch or lose
their proposal-write classification.

MCP PCB taxonomy note: canonical `datum.pcb.place_component`, `move_component`, `rotate_component`, `flip_component`, `align_components`, `delete_component`, `set_component_reference`, `set_component_value`, `set_component_part`, `set_component_package`, `lock_component`, `unlock_component`, `draw_track`, `edit_track`, `delete_track`, `place_via`, `edit_via`, `delete_via`, `place_zone`, `edit_zone`, `delete_zone`, `place_pad`, `edit_pad`, `delete_pad`, `set_pad_net`, `clear_pad_net`, `place_net`, `edit_net`, `delete_net`, `place_net_class`, `edit_net_class`, `delete_net_class`, `set_board_name`, `set_outline`, `set_stackup`, `add_default_top_stackup`, `place_keepout`, `edit_keepout`, `delete_keepout`, `place_dimension`, `edit_dimension`, `delete_dimension`, `place_text`, `edit_text`, and `delete_text` are now native-project scoped, require explicit `path` plus operation-specific arguments, and bridge to the matching `datum-eda project ...board-component...`, `...board-track`, `...board-via`, `...board-zone`, `...board-pad`, `...board-net...` / `...board-net-class`, board setup, `...board-keepout`, `...board-dimension`, and `...board-text` commands over journaled native project substrates. `align_components` is the one mode-parameterized align/distribute batch surface and is locked by `check_pcb_layout_tool_matrix`. Flat daemon-session writes `move_component`, `rotate_component`, `flip_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` remain hidden compatibility only; they are fenced by `NON_JOURNALED_DAEMON_WRITE_METHODS` and are not public `datum.*` tools.

MCP schematic taxonomy note: canonical `datum.schematic.create_sheet_definition`, `create_sheet_instance`, `delete_sheet_instance`, `move_sheet_instance`, `bind_sheet_instance_port`, `unbind_sheet_instance_port`, `draw_wire`, `delete_wire`, `place_junction`, `delete_junction`, `place_noconnect`, `delete_noconnect`, `place_label`, `rename_label`, `delete_label`, `place_port`, `edit_port`, `delete_port`, `create_bus`, `edit_bus_members`, `place_bus_entry`, `delete_bus_entry`, `place_text`, `edit_text`, `delete_text`, `place_drawing_line`, `place_drawing_rect`, `place_drawing_circle`, `place_drawing_arc`, `edit_drawing_line`, `edit_drawing_rect`, `edit_drawing_circle`, `edit_drawing_arc`, `delete_drawing`, `place_symbol`, `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`, `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override, and symbol field aliases are now native-project scoped, require explicit `path` plus sheet/object arguments, and bridge to the matching journaled `datum-eda project create-sheet-definition`, `create-sheet-instance`, `delete-sheet-instance`, `move-sheet-instance`, `bind-sheet-instance-port`, `unbind-sheet-instance-port`, `draw-wire`, `delete-wire`, `place-junction`, `delete-junction`, `place-noconnect`, `delete-noconnect`, `place-label`, `rename-label`, `delete-label`, `place-port`, `edit-port`, `delete-port`, `create-bus`, `edit-bus-members`, `place-bus-entry`, `delete-bus-entry`, `place-schematic-text`, `edit-schematic-text`, `delete-schematic-text`, `place-drawing-*`, `edit-drawing-*`, `delete-drawing`, `place-symbol`, `move-symbol`, `rotate-symbol`, `mirror-symbol`, `delete-symbol`, `set-symbol-reference`, `set-symbol-value`, `set-symbol-*`, `clear-symbol-*`, `set-pin-override`, `clear-pin-override`, `add-symbol-field`, `edit-symbol-field`, and `delete-symbol-field` commands. `create_sheet_definition` creates the definition shard and updates the schematic root definitions map in one undoable transaction; `create_sheet_instance`, `delete_sheet_instance`, and `move_sheet_instance` update the schematic root instances array through the same journaled substrate; `bind_sheet_instance_port` and `unbind_sheet_instance_port` persist parent-port bindings that make native hierarchy links real.

Terminal discovery note: production object create/update/delete templates now default to canonical proposal-first commands: `datum-eda proposal create|update|delete-output-job`, `create|update|delete-manufacturing-plan`, and `create|update|delete-panel-projection`. Agents launched from Datum discover draft-proposal authoring rather than direct production CRUD or compatibility `project ... --as-proposal` syntax.

Artifact execution note: `datum-eda artifact generate <root> --output-job <uuid>`, MCP `datum.artifact.generate` with `output_job`, and the flat MCP `run_output_job` compatibility method now execute an authored OutputJob through the canonical artifact family; production object CRUD remains on compatibility `project ...` verbs for now, but terminal discovery points those verbs at proposal creation rather than direct mutation.

Terminal discovery note: authored OutputJob execution is now advertised as `datum-eda artifact generate <root> --output-job <uuid>` and lifecycle evidence as `datum-eda artifact start-output-job-run|cancel-output-job-run` instead of the legacy `project run-output-job` / lifecycle compatibility verbs. The GUI terminal command catalog now carries structured `datum.artifact.start_output_job_run` and `datum.artifact.cancel_output_job_run` entries, and the Outputs dock exposes status-aware `START` / `CANCEL` lifecycle handoffs on OutputJob rows without adding a GUI private writer.

Terminal discovery manufacturing-set note: the GUI terminal command catalog and
`datum_terminal_context_v1.production_commands` now advertise canonical
`datum-eda artifact export-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
and
`datum-eda artifact validate-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
templates, matching the artifact CLI/MCP taxonomy instead of leaving those
production commands discoverable only through the written spec.

Terminal agent entry note: the visible GUI `AGENTS` dock entry now focuses the PTY terminal and prints shell launch guidance for installed agents (`codex`, `claude`, `aider`) using the injected `DATUM_DISCOVERY` context, rather than treating the legacy assistant lane as the canonical agent authority.

GUI agent/terminal convergence note: terminal activity summary selection now
keeps focus in the PTY terminal lane and appends a terminal-visible selection
note instead of activating or writing to the legacy assistant transcript. Closing
the active terminal tab now refreshes the surviving active session's
`DATUM_DISCOVERY` / `.datum/gui-terminal-context.json` compatibility alias so
the visible `AGENTS` launcher and terminal-launched agents do not inherit stale
context from a terminated tab. `scripts/check_gui_agent_terminal_convergence.py`
is wired into the full drift gate and locks the current convergence rules:
`AGENTS` remains a terminal prefill entry point, terminal activity selection
must not focus `DockTab::Assistant`, and terminal tab close must refresh the
surviving session context alias. Runtime context refresh now also updates the
in-memory `TerminalLaunchContext`, so newly spawned or restarted PTY sessions
inherit the latest selection/cursor/dock context rather than stale launch-time
state.

GUI terminal context durability note: GUI-written terminal discovery files
(`.datum/terminal-contexts/<session>.json`,
`.datum/gui-terminal-context.json`, and `.datum/tool-sessions/<session>.json`)
now use same-directory atomic temp-file writes with `sync_all` plus rename, and
lifecycle rewrites use the same path. Runtime context refresh failures are no
longer silent; they are surfaced in the terminal lane so agents are not left
with stale or partially-written `$DATUM_DISCOVERY` state without visible
diagnostics. Focused regressions repeatedly rewrite context JSON and lifecycle
state while proving the files remain parseable and temp files do not leak.

GUI terminal session context note: `$DATUM_DISCOVERY` now includes a
`terminal_sessions` summary derived from the protocol-backed terminal lane.
The envelope exposes the active session id, active label/status/attachment,
tab count, terminal dimensions, active activity summary, and per-tab session
lineage/event-log/activity metadata. Focused regression coverage drives the
summary from a real `TerminalSessionRegistry` through `ReviewWorkspaceState`
into `.datum/gui-terminal-context.json`, so terminal-launched agents see the
same active/detached/restarted session shape as the GUI dock.

GUI/CLI active context command note: terminal discovery now includes
`active_context_commands`, a small set of catalog-rendered commands for the
currently focused production/check context. When the focused artifact or latest
check-run id exists, agents receive concrete `datum-eda artifact
show|files|preview|validate`, `datum-eda check show`, and `datum-eda check
repair-standards` commands with session-specific IDs already bound. When the GUI
selection is a CheckFinding, the same active context now also exposes
fingerprint-bound `datum-eda check waive` and `datum-eda check accept-deviation`
commands with only the rationale left as a deliberate placeholder; when focus or
selection data is absent the fields remain `null` rather than exposing
placeholder commands. GUI-written terminal discovery now resolves
`accepted_transaction_tip` from the project journal and binds `journal_show_tip`
before a CLI refresh, while CLI `context get|refresh` rebuilds the same active
command fields from the persisted discovery envelope, resolver-derived context,
and preserved selection context, so refreshed terminal-launched agents do not
lose journal or finding-level handoffs. Regression coverage verifies focused,
selected-finding, refreshed, and empty-context cases.

Embedded assistant bridge retirement note: the GUI runtime no longer imports,
spawns, polls, or writes to the former embedded assistant bridge subprocess.
`crates/gui-app/src/assistant_bridge.rs` and `scripts/datum_assistant_bridge.py`
were removed, while the visible `AGENTS` entry now preloads terminal-launched
`codex`/`claude` workflows instead of creating a parallel assistant authority.
The compatibility `DockTab::Assistant` state is terminal-only: it renders the
terminal lane, does not accept text input, has no transcript renderer, and no
longer has protocol-backed `AssistantMessage`, `AssistantLaneState`, or
`WorkspaceUiState::assistant` payloads. The GUI convergence guard now fails if
retired bridge artifacts, runtime ownership markers, assistant transcript input,
assistant-lane rendering, or protocol assistant-state structs reappear. The
bottom dock now renders `AGENTS` as an inactive launcher button rather than a
second active command surface, and the terminal lane explicitly states that
agent launches prefill `codex`/`claude` shell commands into the PTY.

PTY terminal screen note: GUI terminal parsing now handles CSI insert-character (`CSI @`), delete-character (`CSI P`), and erase-character (`CSI X`) operations without moving the cursor, with regressions proving blank-cell insertion before overwrite, shifted-row deletes, and fixed-cell blanking before subsequent printable overwrites.

PTY terminal screen note: GUI terminal parsing now captures OSC `0`/`1`/`2`
shell title updates into `TerminalLaneState.title`, renders the active
PTY-provided session label in the terminal dock header, and continues
swallowing unsupported OSC payloads without visible byte leakage. Focused
regressions cover BEL-terminated, ST-terminated, icon-title, split OSC title
updates, and unsupported OSC commands that must not overwrite the active title.

PTY terminal context note: GUI terminal parsing now captures OSC `7`
current-directory updates from `file://host/path` URIs into
`TerminalLaneState.current_working_directory`, percent-decodes the path, renders
the same value in the terminal dock header as `CWD`, and projects it into
terminal context JSON as `terminal_sessions.active_working_directory` for
terminal-launched agents. Unsupported OSC 7 URI schemes are swallowed without
changing the tracked directory.

PTY terminal screen note: bare BEL (`0x07`) now increments
`TerminalLaneState.bell_count` without visible byte leakage, and the terminal
dock surfaces the current alert count in the session header. OSC BEL
terminators remain control-string terminators and do not increment the alert
counter.

PTY terminal screen note: GUI terminal parsing now supports `ESC M` Reverse Index, with regressions proving ordinary cursor-up behavior away from the top margin and scroll-region-only downward scrolling when invoked at the active top margin.

PTY terminal screen note: GUI terminal parsing now supports CSI next-line (`CSI E`) and previous-line (`CSI F`) cursor controls, with regressions proving default and explicit row counts reset the cursor to column zero and extend terminal rows when moving downward.

PTY terminal screen note: GUI terminal parsing now supports CSI vertical-position-absolute (`CSI d`), vertical-position-relative (`CSI e`), and vertical-position-backward (`CSI k`) controls, with regressions proving these preserve the current column, create target rows when moving downward, and saturate at the top row when moving backward.

PTY terminal screen note: GUI terminal parsing now supports CSI insert-line (`CSI L`) and delete-line (`CSI M`) controls, with regressions proving row shifts stay bounded to the active full-screen or scroll-region range and preserve rows outside that region.

PTY terminal screen note: GUI terminal parsing now supports CSI scroll-up (`CSI S`) and scroll-down (`CSI T`) controls, with regressions proving full-screen and scroll-region bounded shifts clear introduced rows while preserving rows outside the active region.

PTY terminal screen note: GUI terminal parsing now uses default eight-column horizontal tab stops and supports CSI forward-tab (`CSI I`) plus backtab (`CSI Z`), with regressions proving forward-tab alignment, repeated forward-tab counts, prior-stop rewrites, and saturation at column zero.

PTY terminal screen note: GUI terminal parsing now supports CSI repeat
preceding character (`CSI b`). The parser tracks the last printable character
and routes repeats through the same cursor, autowrap, and scroll-region writer
used for normal PTY bytes, with regressions proving repeat counts, no-op
behavior before any printable character, and terminal-width wrapping.

PTY terminal screen note: GUI terminal parsing now supports `ESC D` Index and
`ESC E` Next Line. Both controls share the same scroll-region-aware linefeed
path as printable wrapping; Index preserves the active column, while Next Line
resets to column zero. Focused regressions prove normal movement and bottom
scroll-region scrolling.

PTY terminal screen note: GUI terminal parsing now treats ASCII vertical tab
(`VT`, `0x0B`) and form feed (`FF`, `0x0C`) as line-feed controls that preserve
the active column, matching terminal emulator behavior for legacy line-advance
controls without leaking visible replacement bytes.

PTY terminal screen note: GUI terminal parsing now supports `ESC c` terminal
reset. The reset clears visible rows, cursor position, saved cursor,
alternate-screen state, scroll region, pending wrap, and repeat-character state,
with regressions proving post-reset writes do not restore stale state.

PTY terminal screen note: GUI terminal parsing now swallows VT charset
designation sequences such as `ESC ( B` and `ESC ) 0`. Focused regressions prove
whole and split charset-designation sequences do not leak designation bytes into
visible terminal rows. It also supports DEC screen alignment (`ESC #8`) by
filling the visible protocol grid with unstyled `E` cells while keeping
unsupported `ESC #` intermediate controls non-leaking.

PTY terminal screen note: GUI terminal parsing now swallows non-printing
ST-terminated control strings for DCS (`ESC P`), SOS (`ESC X`), PM (`ESC ^`),
and APC (`ESC _`), plus two-byte `ESC #` intermediate sequences. Focused
regressions prove whole and split control strings, split ST terminators, and
split `ESC #` sequences do not leak payload/final bytes into visible terminal
rows.

PTY terminal screen note: GUI terminal parsing now supports xterm private
alternate/cursor modes `CSI ?47h/l`, `CSI ?1047h/l`, and `CSI ?1048h/l` in
addition to the existing `CSI ?1049h/l` alternate-screen path. The parser also
tracks bracketed paste mode via `CSI ?2004h/l`; terminal clipboard paste wraps
payload bytes in `CSI 200~` / `CSI 201~` when the active PTY requests it, and
terminal reset clears that mode. Focused regressions prove legacy `47`, modern
`1047`, and full save/restore `1049` restore the main buffer, `1048`
saves/restores cursor position without buffer switching, split alternate-screen
private-mode sequences do not leak bytes into terminal rows, and bracketed paste
state is tracked without visible escape leakage.

PTY terminal screen note: GUI terminal parsing now tracks xterm focus-event
reporting mode (`CSI ?1004h/l`) in `TerminalLaneState.focus_event_reporting`
and surfaces the active mode in the terminal dock header. While active, GUI
window focus changes now send xterm focus-in/focus-out controls (`ESC [ I` /
`ESC [ O`) to the attached PTY without marking production refreshes or warning
for detached sessions. Focused regressions prove enable, disable, split
private-mode input, terminal reset behavior, and focus-report byte selection do
not leak bytes into terminal rows.

PTY terminal screen note: GUI terminal parsing now tracks xterm mouse reporting
mode negotiation (`CSI ?1000/1002/1003h/l`) and mouse coordinate encodings
(`CSI ?1005/1006/1015h/l`) in `TerminalLaneState.mouse_reporting_mode` and
`mouse_coordinate_encoding`, then surfaces the active mode in the terminal dock
header. GUI terminal clicks and wheel input now emit classic/default X10 mouse
reports (`ESC [ M Cb Cx Cy`) when an attached PTY requests mouse reporting
without an extended coordinate encoding, UTF-8 X10-shape reports with UTF-8
encoded coordinate codepoints when UTF-8 encoding is active, URXVT reports
(`CSI Cb ; Cx ; Cy M`) when URXVT encoding is active, and xterm SGR mouse
reports (`CSI < Cb ; Cx ; Cy M/m`) when SGR encoding is active, using
protocol-visible terminal geometry in all emitted paths. `button_event` mode
reports pointer motion while a terminal mouse button is held; SGR `any_event`
mode also reports terminal pointer motion without a held button. Ordinary dock
scrolling/click actions remain in effect when the PTY has not requested mouse
reporting. Focused regressions prove mode precedence, encoding precedence,
disable/reset behavior, split private-mode input, and SGR/X10/UTF-8/URXVT mouse
byte selection do not leak bytes.

PTY terminal screen note: GUI terminal parsing now answers normal Device Status
Report queries from the PTY. `CSI 5n` replies with terminal-ready status
(`ESC [ 0 n`), and `CSI 6n` replies with the current one-based cursor position
(`ESC [ row ; col R`) while private `CSI ?6n` replies with the DEC private
cursor-position form (`ESC [ ? row ; col R`) without leaking query bytes into
visible terminal rows.
The same response path now answers primary Device Attributes (`CSI c`) and
DECID (`ESC Z`) with `ESC [ ? 1 ; 2 c`, and secondary Device Attributes
(`CSI > c`) with `ESC [ > 0 ; 0 ; 0 c`. The response path is owned by the active
`TerminalSessionRegistry` slot so each PTY tab replies from its own screen
state.

PTY terminal geometry note: GUI terminal parsing now answers xterm text-area
size reports. `CSI 18t` replies with `ESC [ 8 ; rows ; columns t` using the
active `TerminalLaneState.rows` / `columns`, so shell applications can query the
same protocol-visible geometry that Datum applies to the PTY. `CSI 19t` reports
the same active row/column grid as full-screen character size because Datum's
embedded PTY surface does not expose a larger enclosing terminal desktop.

PTY terminal geometry note: GUI terminal parsing now answers xterm text-area
pixel and cell-size reports. `CSI 14t` derives total pixel size from the active
row/column grid using stable 16x8 terminal cells, `CSI 15t` returns the same
dimensions as root/screen pixel size because Datum's embedded PTY surface has no
larger enclosing terminal desktop, and `CSI 16t` reports that same cell size
directly, avoiding platform font metric dependencies while preserving
deterministic terminal-query behavior.

PTY terminal status note: GUI terminal parsing now answers xterm window-state
and position queries. `CSI 11t` replies with `ESC [ 1 t` for the active PTY
session, `CSI 13t` returns a deterministic `0,0` top-left embedded-terminal
position, and split query bytes remain buffered without visible leakage.

PTY terminal status note: GUI terminal parsing now answers xterm icon-label and
window-title status queries. `CSI 20t` and `CSI 21t` return ST-terminated OSC
responses derived from `TerminalLaneState.title`, while stripping control bytes
from the title payload before writing the response back to the PTY. `CSI 22t`
and `CSI 23t` now save and restore the protocol title without visible terminal
output, with split-sequence regressions proving title stack controls do not leak
bytes into terminal rows.

PTY terminal screen note: GUI terminal parsing now tracks DEC application
cursor-key mode (`CSI ?1h/l`) in
`TerminalLaneState.application_cursor_keys`; terminal key routing emits SS3
arrow/Home/End sequences (`ESC O A/B/C/D/H/F`) while active and normal CSI
sequences otherwise, and the terminal dock surfaces the active mode. Focused
regressions prove enable, disable, split private-mode input, reset clearing, and
input sequence selection.

PTY terminal screen note: GUI terminal parsing now tracks DEC application
keypad mode (`ESC =` / `ESC >`) in `TerminalLaneState.application_keypad`;
terminal key routing emits SS3 keypad sequences for physical numpad keys while
active and leaves non-numpad keys on the normal text path. The terminal dock
surfaces the active mode, and focused regressions prove enable, disable, split
escape input, reset clearing, and keypad sequence selection.

PTY terminal geometry note: GUI PTY resize now has protocol-visible geometry.
`TerminalLaneState.columns` / `rows` default to `80x24`, are updated whenever
the dock/window resizes the active PTY, survive active-session restart by
reapplying the stored size to the new PTY, and are surfaced in the terminal
dock header as `SIZE CxR`. Focused registry and render regressions prove
published geometry follows resize and restart behavior.

PTY terminal screen note: GUI terminal parsing now exposes PTY screen cursor
row/column/visibility/style through `TerminalLaneState`, supports DEC private
cursor visibility mode (`CSI ?25h/l`) plus DECSCUSR cursor shape controls
(`CSI Ps SP q`), and the dock renders the protocol cursor as block, underline,
or bar only when visible. Focused regressions prove hidden/visible cursor mode
and split cursor-shape controls do not leak escape bytes into terminal rows,
terminal reset clears cursor style, and renderer contracts honor the protocol
visibility/style fields.

PTY terminal session-history note: terminal tab protocol state now carries the
durable per-session event-log path, activity event count, and compact per-tab
activity summary, allowing GUI surfaces and terminal-launched agents to
correlate each tab with its persisted JSONL history without guessing
`.datum/tool-sessions` paths. Terminal tab activation updates the visible
activity lane from that tab's persisted summary, the dock renders an activity
badge for active tabs when no restart badge is present, and focused registry
coverage proves detach/attach lifecycle events advance the protocol history
count and summary while preserving existing context/session isolation.

PTY terminal screen note: GUI terminal parsing now accepts 8-bit C1 control
bytes for CSI, OSC, DCS/SOS/PM/APC control strings, ST, Index, Next Line, and
Reverse Index. These controls dispatch through the same screen-grid paths as
their ESC-prefixed equivalents, swallow payload bytes without visible leakage,
and preserve existing split UTF-8 decoding behavior.

PTY terminal screen note: GUI terminal parsing now supports DEC autowrap mode
`CSI ?7h/l`. Autowrap defaults enabled, can be disabled for right-margin
overwrite/clamp behavior, can be re-enabled for existing wrap semantics, is
honored by repeat-character writes, and is restored by terminal reset.

PTY terminal screen note: GUI terminal parsing now supports DEC origin mode
`CSI ?6h/l`. Cursor home and `CSI H/f` plus `CSI d` vertical absolute
positioning become relative to the active scroll-region top margin while enabled
and clamp to the bottom margin; disabling origin mode restores absolute screen
addressing.

PTY terminal screen note: GUI terminal parsing now applies combined DEC/xterm
private-mode sequences in parameter order instead of stopping at the first
matched mode. Focused regressions prove `CSI ?6;7h` enables both origin-mode
scroll-region addressing and autowrap, while `CSI ?6;7l` disables both absolute
origin remapping and autowrap right-margin wrapping.

PTY terminal screen note: GUI terminal parsing now preserves cursor position
for whole-line erase `CSI 2K`, matching terminal erase semantics used by prompt
redraws and full-line status rewrites. Focused coverage proves the next
printable character lands at the pre-erase cursor column instead of incorrectly
resetting to column zero.

PTY terminal screen note: GUI terminal parsing now preserves cursor row/column
for full-display erase `CSI 2J` and scrollback/full erase `CSI 3J` instead of
treating them as terminal reset. Focused regressions prove post-erase printable
bytes land at the preserved address on the cleared screen.

PTY terminal screen note: GUI terminal parsing now supports normal insert mode
`CSI 4h` and replace mode `CSI 4l`. Printable writes and `CSI b` repeat
characters share the same insert-aware cursor path, split mode sequences do not
leak bytes, and terminal reset clears insert mode back to replacement mode.

PTY terminal screen note: GUI terminal parsing now supports mutable horizontal
tab stops. `ESC H` sets a custom stop at the current column, `CSI g` clears the
current stop including default stops, `CSI 3g` clears all stops so HT/CHT become
no-ops until a new stop is set, and terminal reset restores default eight-column
stops. HT, `CSI I`, and `CSI Z` now all consult the same tab-stop state.

PTY terminal screen note: GUI terminal parsing now retains SGR foreground color
and bold state instead of only stripping `CSI ... m` sequences. The protocol
exposes styled terminal rows through `TerminalStyledLine` spans, the dock
renders styled spans as colored monospace runs, reset prevents style leakage to
later cells, and alternate-screen save/restore preserves styled rows.

PTY terminal screen note: known-width PTY sessions now apply terminal-grid
semantics for `CSI @` insert-character and `CSI P` delete-character. Inserted
cells shift only within the visible column width and discard overflow at the
right margin; deleted cells shift left and pad the right margin with blanks.
Known-width horizontal cursor controls now also clamp to the right margin for
`CSI C`, `CSI G`, `CSI \``, `CSI a`, and `CSI H/f` column addressing instead of
allowing cursor positions beyond the visible grid. PTY resize now also passes
row geometry into the screen parser, and known-height vertical cursor controls
clamp to the bottom margin for `CSI B`, `CSI E`, `CSI d`, `CSI e`, and
`CSI H/f` row addressing. Known-height scroll-region setup (`CSI top;bottom r`)
now clamps an oversized bottom margin to the PTY bottom row instead of growing
the screen model beyond the visible grid. Known-width erase-character (`CSI X`)
now also blanks visible cells beyond the current row text up to the right margin
instead of ignoring cells that have not yet been materialized as string content.
Known-width erase-in-line (`CSI K`) now materializes visible blank cells for
modes 0/1/2 across the bounded terminal row instead of truncating or clearing
only the row string backing store. Known-geometry erase-display (`CSI J`) now
materializes visible blank cells across bounded rows and columns for modes
0/1/2/3 while preserving cursor position. Known-height line operations
(`CSI L`, `CSI M`, `CSI S`, and `CSI T`) now use the visible PTY screen region
when no scroll region is active instead of deriving the active region only from
already-materialized backing rows.
Legacy no-width/no-height parser behavior remains unchanged for compatibility
tests.

PTY terminal UX note: terminal-focused keyboard scrollback now supports
`Shift+PageUp`, `Shift+PageDown`, `Shift+Home`, and `Shift+End` without sending
those navigation keys to the PTY. The dock renders the scrollback shortcut
beside copy/paste, while plain Home/End remain PTY control sequences.

PTY terminal input note: terminal key routing now emits xterm-compatible
navigation, editing, and function-key sequences for `Insert`, `Delete`,
`PageUp`, `PageDown`, and `F1`-`F12`. Shift-modified navigation remains reserved
for GUI scrollback controls, so native shell applications receive plain terminal
keys while Datum keeps explicit scrollback shortcuts. Alt/meta character input
now emits an ESC-prefixed byte stream like native terminal emulators, while
ordinary Ctrl character chords emit ASCII control bytes for shell/readline
apps, including `Ctrl+Space` as NUL. Ctrl+Alt character chords remain reserved
instead of being downgraded into ambiguous text bytes. `Shift+Tab` now emits
the xterm reverse-tab sequence (`CSI Z`) instead of a literal tab. Modified
arrow keys now emit xterm modifier-parameter sequences such as `CSI 1;5D` for
Ctrl+Left and `CSI 1;5H` for Ctrl+Home instead of collapsing to plain keys. Modified
Insert/Delete/PageUp/PageDown now use the same xterm tilde modifier form, such
as `CSI 6;5~` for Ctrl+PageDown. Modified `F1`-`F12` now also preserve xterm
modifier parameters, using CSI forms for modified `F1`-`F4` and tilde forms for
modified `F5`-`F12`.

PTY terminal screen note: GUI terminal parsing now retains SGR background color
and inverse-video state in protocol spans alongside foreground and bold. The
dock renders inverse/background spans with a visible color fallback while
preserving the richer metadata for future cell-background rendering.

PTY terminal screen note: GUI terminal parsing now retains extended SGR color
metadata for 256-color (`38;5;n` / `48;5;n`) and truecolor (`38;2;r;g;b` /
`48;2;r;g;b`) foreground/background spans. Malformed extended-color sequences
are ignored without clearing the active style.

PTY terminal screen note: GUI terminal parsing now retains SGR italic,
underline, and strikethrough metadata (`3`/`4`/`9`) in protocol
`TerminalStyleSpan`s and honors their reset controls (`23`/`24`/`29`) without
clearing unrelated color, bold, or inverse state.

PTY terminal screen note: GUI terminal parsing now retains SGR dim and conceal
metadata (`2`/`8`) in protocol `TerminalStyleSpan`s. SGR `22` clears both bold
and dim without touching conceal or colors, while SGR `28` clears conceal
without touching other active style metadata.

PTY terminal screen note: GUI terminal parsing now retains SGR blink metadata
(`5`/`6`) in protocol `TerminalStyleSpan`s. SGR `25` clears blink without
touching unrelated style metadata.

PTY terminal screen note: GUI terminal parsing now retains SGR overline metadata
(`53`) in protocol `TerminalStyleSpan`s. SGR `55` clears overline without
touching underline, strikethrough, or unrelated style metadata.

GUI source-shard health note: the project panel now renders resolver
source-shard attention as an explicit dirty/missing/unknown breakdown instead of
a single aggregate attention count, so users and terminal-launched agents can
distinguish stale promoted files from missing journal-recovered shards. The
protocol summary now also carries an `attention` list with shard relative path,
kind, authority, taxon, and dirty state for each non-clean shard; the GUI
renders the first attention rows in the project panel and terminal context JSON
exposes the same drilldown to agents.

GUI supervision/status note: native board scene loading now resolves the project
through `ProjectResolver` and asks the `DesignModel` for the materialized board
root shard, so the viewport reflects accepted journal state rather than only
the promoted `board/board.json` file. The protocol surface now includes
`datum_gui_supervision_snapshot_v1` on `ReviewWorkspaceState`, carrying model
revision, journal cursor/tip, source-shard summary, scene object counts, check
summary, and production/proposal/artifact counts. The Outputs dock renders that
snapshot as `ENGINE SUPERVISION`. This is a read-only supervision/status
surface; GUI edit affordances remain terminal-prefilled command handoffs until
a future direct editor path constructs and commits substrate operations with
its own gates.

Generated-evidence source-shard note: public `resolve-debug` coverage now proves
journal-recovered ZoneFill, OutputJobRun, ArtifactRun, and CheckRun evidence
retain `GeneratedEvidence` authority, concrete `ZoneFill` / `OutputJobRun` /
`ArtifactRun` / `CheckRun` taxons, and `Unknown` dirty-state metadata when
promoted `.datum/zone_fills/<uuid>.json`,
`.datum/output_job_runs/<uuid>.json`, `.datum/artifact_runs/<uuid>.json`, or
`.datum/check_runs/<uuid>.json` paths are unreadable, matching the existing
ArtifactMetadata unknown-state coverage.

GUI text edit note: selected board-text Inspector entries now prefill the PTY terminal with canonical `datum-eda project edit-board-text "$DATUM_PROJECT_ROOT" --text <uuid> ...` commands instead of opening assistant-only `/text` commands or mutating board JSON directly. Explicit center edits prefill the current value; edge/cycle controls prefill the next boolean/cycle/step value, with height steppers carrying paired `--height-nm` and `--stroke-width-nm` arguments so proportional scaling still routes through the journaled `SetBoardText` CLI path. The legacy assistant `/text` command parser and completions have been removed, and the old `gui-protocol` board-text private writer modules were deleted rather than left as dormant public helpers.

Open tracking rule:
- Add implementation evidence here before marking a substrate row complete.
- If a row is intentionally deferred, record the governance reason and the
  downstream fidelity work allowed despite the deferral.
- Do not promote legacy milestone completion to substrate readiness without a
  direct product-mechanics contract.

Source-shard taxonomy correction:
- Remaining hand-built resolver/replay refs for ImportMap, ProposalMetadata,
  ComponentInstance, Relationship, VariantOverlay, production records, ZoneFill,
  and schematic sheet payloads now derive concrete taxons through the central
  source-shard path taxonomy instead of constructing `taxon=None` refs.

Context envelope correction:
- `datum-eda context get|refresh|session-events|session-activity` no longer returns only the raw GUI discovery
  file. The CLI now enriches compatible `datum_terminal_context_v1` discovery
  payloads with the first `DatumContextEnvelope` fields: `actor_type`,
  `capabilities`, resolver-derived `project_id` / `project_name` /
  `model_revision` when a project root resolves, `accepted_transaction_tip`,
  `visible_artifact_ids`, `visible_output_job_ids`,
  `visible_artifact_file_paths`, latest output-job/run/artifact IDs,
  latest artifact/artifact-run IDs, focused artifact/file defaults,
  `source_shard_status`, `latest_check_run_id`, `latest_profile_id`,
  `profile_latest_check_runs`, `visible_check_run_ids`, `output_context`,
  `provenance_seed`, `expires_at`, `refresh_command`, and storage metadata.
  The GUI terminal discovery writer emits the same first-slice fields for
  terminal-launched shells, writes authoritative per-session context files under
  `.datum/terminal-contexts/<session>.json`, keeps
  `.datum/gui-terminal-context.json` as a latest-session compatibility alias,
  and exports `DATUM_DISCOVERY` / `DATUM_TERMINAL_CONTEXT` to the per-session
  file so concurrent terminals do not race through the singleton path. CLI
  project-root lookup now prefers the requested session file before falling
  back to the legacy latest alias. `datum-eda context get` returns the normalized
  envelope without writing it, while `datum-eda context refresh` persists the
  enriched envelope back to the resolved session context file without losing
  GUI-owned typed context fields; `context session-events` returns parsed
  `.datum/tool-sessions/<session>.events.jsonl` records as
  `datum_tool_session_events_v1`, including terminal command `origin`,
  `command_id`, optional `mcp_alias`, and `handoff_mode` fields that distinguish
  board-text prefill commands from executed terminal handoffs. `context session-events`
  and MCP `datum.context.session_events` also expose exact-match `event_kind`,
  `origin`, and `command_id` filters plus `limit` for the newest matching
  events after filters, with returned, matched, and raw event counts.
  `context session-activity` returns a compact
  `datum_tool_session_activity_summary_v1` aggregate over the same filtered
  window for agent orientation. MCP `datum.context.get|refresh|session_events|session_activity`
  continues to bridge through the canonical CLI while its catalog/test doubles
  document the richer envelope and event stream; focused MCP coverage now proves
  `datum.context.get` preserves artifact, OutputJob, proposal, journal, and CheckFinding `active_context_commands` through the
  agent-facing target envelope. GUI protocol now defines shared typed session/context
  structs, GUI-authored context files carry `selection_context`,
  `cursor_context`, and `projection_context` derived from `ReviewWorkspaceState`,
  GUI terminals persist `DatumToolSessionMetadata` under
  `.datum/tool-sessions/<session>.json`, and GUI selection, cursor/hover, dock,
  and frame-affecting session events rewrite the same per-session context file
  without minting a new session. Remaining context gaps are richer engine-owned
  session policy/expiry semantics and broader projection/cursor vocabulary
  beyond the current GUI snapshot.

CheckRun row correction:
- ComponentInstance operation correction: `OperationBatch` now covers
  `CreateComponentInstance`, `SetComponentInstance`, and
  `DeleteComponentInstance`; journaled commits stage/promote the authored shard,
  capture inverse operations, support undo/redo, and reconstruct missing
  promoted component-instance shards from accepted journal replay. CLI now
  exposes `project query <root> component-instances`,
  `project bind-component-instance`, `project set-component-instance`, and
  `project delete-component-instance`, with regression coverage proving the
  commands use the journal path and remain undoable. MCP now exposes
  `get_component_instances`, `bind_component_instance`,
  `set_component_instance`, `delete_component_instance`, and canonical
  `datum.query.component_instances` / `datum.component_instance.*` aliases over
  the same CLI bridge. BOM/PnP export now emits `component_instance_uuid`, and
  BOM/PnP compare keys matched/missing/extra/drift evidence by ComponentInstance
  when present, with package UUID fallback for board-only legacy rows.
  Multi-unit ComponentInstance refs now survive bind/set/delete undo and query.
  ComponentInstance shards now carry optional `part_ref` identity to a current
  native `pool/parts` object; resolver validation rejects stale/missing/non-part
  refs, commit validation rejects stale part revisions before staging, CLI
  bind/set expose `--part`, and MCP flat/canonical ComponentInstance write
  tools forward the same field to the CLI bridge. Forward-annotation
  audit/proposal matching now uses resolver ComponentInstance symbol/package
  refs before legacy reference fallback for uncovered objects, and treats
  `part_ref` as the expected part identity when present. Regressions prove
  refdes-only matches no longer create update actions for already-bound objects
  and mismatches target the ComponentInstance-bound package even when references
  differ. Forward-annotation artifact compare now classifies exact `action_id`
  matches as applicable and same symbol/component UUID identity plus same
  action/reason as `drifted` when a reference rename changes the derived
  `action_id`, without broadening filter/apply eligibility. The legacy
  reference/action fallback was removed, so same-ref/action replacements with
  different object UUID identity are stale rather than drifted.
  ComponentInstance shards now carry per-symbol and per-package `{role, label}`
  metadata keyed by referenced object UUID; resolver and commit validation reject
  role keys outside the selected refs, blank/invalid role identifiers, and
  invalid labels, while CLI/MCP bind/set can author `symbol_roles` and
  `package_roles` and default new bindings to stable role records. BOM/PnP
  assembly rows now project package role/label into `component_instance_role`
  and `component_instance_label`, compare reports role/label drift, inspect
  accepts both new and legacy CSV headers, and output-job manufacturing-set
  artifacts preserve the same columns through stored variant runs. Resolver
  synthesized exact-match joins are retired; authored ComponentInstances are
  the only source of stable BOM/PnP identity, forward-annotation `part_ref`
  authority, component-scoped variant propagation, resolver/debug counts, and
  ComponentInstance query rows, while package/symbol fitted-state entries
  remain valid.
  Automated cross-domain identity/intent writes are now proposal-required at the
  commit gateway: `Tool`/`Assistant` provenance batches that directly
  create/set/delete ComponentInstance, Relationship, or VariantOverlay shards
  are rejected with
  `proposal_required_for_automated_cross_domain_identity_operation`, keeping
  schematic/board/part bindings, design-intent relationships, and variant
  overlays on the reviewable proposal path. Automated pool/library writes are
  now covered by the same gateway policy: `Tool`/`Assistant` provenance batches
  that directly create/delete typed pool packages or padstacks, create/set/delete
  generic pool library objects, or attach/detach pool part models are rejected
  with `proposal_required_for_automated_library_operation`, so AI/tool-authored
  symbols, packages, padstacks, parts, pin maps, and model attachments cannot
  bypass proposal review before authored pool shards are staged. The MCP
  runtime now preserves that source boundary by launching CLI-backed library
  authoring methods with `DATUM_COMMIT_SOURCE=tool`, preventing canonical
  `datum.library.*` aliases from silently downgrading automation into
  `CommitSource::Cli`. Positive library proposal producers now exist for raw
  objects and typed semantic unit/symbol/entity/padstack/package/package-pad/package-courtyard paths: `datum-eda proposal
  create-pool-library-object` / `datum.proposal.create_pool_library_object` and
  `datum-eda proposal create-pool-unit` / `datum.proposal.create_pool_unit` and
  `datum-eda proposal create-pool-symbol` / `datum.proposal.create_pool_symbol`
  and `datum-eda proposal create-pool-entity` /
  `datum.proposal.create_pool_entity` and `datum-eda proposal
  create-pool-padstack` / `datum.proposal.create_pool_padstack` and
  `datum-eda proposal create-pool-package` /
  `datum.proposal.create_pool_package` and `datum-eda proposal
  set-pool-package-pad` / `datum.proposal.set_pool_package_pad` and
  `datum-eda proposal set-pool-footprint-courtyard-rect` /
  `datum.proposal.set_pool_footprint_courtyard_rect` and `datum-eda proposal
  set-pool-footprint-courtyard-polygon` /
  `datum.proposal.set_pool_footprint_courtyard_polygon` and
  `datum-eda proposal set-pool-package-courtyard-rect` /
  `datum.proposal.set_pool_package_courtyard_rect` and `datum-eda proposal
  set-pool-package-courtyard-polygon` /
  `datum.proposal.set_pool_package_courtyard_polygon` create non-mutating draft
  proposals, then `proposal accept-apply` writes the object through the generic
  proposal gateway.
- The deviation lifecycle is no longer pending for the current compatibility
  slice. `project accept-deviation` and MCP `accept_deviation` now author
  fingerprint-scoped accepted deviations through the native journal; CheckRun
  readback reports `status=accepted_deviation` and `deviation_refs`.
- DRC fingerprint lifecycle coverage now matches the ERC path for the current
  native-project slice. Focused CLI regressions waive and accept-deviate a
  `connectivity_unrouted_net` DRC finding through the journaled
  `project waive-finding` / `project accept-deviation` commands, prove domain
  readback is `drc`, and prove waiver undo/redo clears and restores
  `waiver_refs` on the normalized CheckRun finding.
- Canonical CLI now exposes `datum-eda check list` and
  `datum-eda check show --check-run <uuid>` over resolver-discovered persisted
  `.datum/check_runs` generated evidence. MCP now exposes matching
  compatibility tools `get_check_runs` / `show_check_run` plus canonical
  `datum.check.list` / `datum.check.show` aliases over the same CLI bridge.
  Canonical CLI also exposes `datum-eda check profiles <root>` and MCP exposes
  matching `get_check_profiles` / `datum.check.profiles`, now reporting the
  bounded supported profile set `native-combined`, `erc`, `drc`, `standards`,
  `manufacturing`, and `release`. `datum-eda check run <root> --profile <id>`
  and MCP `datum.check.run` with `profile=<id>` persist profile-keyed CheckRun
  evidence, with non-default profiles filtering the current deterministic
  findings by domain or standards-repair rule family; unsupported profile ids
  are rejected. The remaining CheckRun
  proposal-link gap has been narrowed: live and persisted `check_run_v1`
  payloads now preserve compatibility `proposal_refs` while adding structured
  `proposal_links` at run and finding level with proposal status/source,
  rationale, validation blockers, and canonical `datum-eda proposal ...`
  command templates including preview. ZoneFill standards-repair coverage now
  also proves generated `SetZoneFill` repair proposals link back into the
  CheckRun and finding `proposal_links` graph, so supported unfilled/stale
  copper-evidence repairs are visible to agents through the same proposal
  discovery path as process-aperture repairs. Canonical MCP
  `datum.check.repair_standards` now normalizes the repair output into
  `check_run_id`, `proposal_count`, and proposal readiness/affected-object
  fields while preserving the raw compatibility payload. Live and persisted CheckRun payloads now also carry
  deterministic `profile_basis` and `coverage` entries so evaluated,
  profile-filtered, and not-yet-implemented rule families are explicit instead
  of inferred from missing findings; canonical MCP normalization and GUI
  protocol preserve those fields. Standards profile basis rows, standards coverage rows, and generated standards findings now also carry typed v1 `standards_basis_detail` evidence with `basis_id`, registry-entry reference, selection scope, basis kind, disposition, and provenance while preserving flat `standards_basis` compatibility. The current process-aperture and ZoneFill honesty basis details are resolved from an engine-owned v1 check standards-basis registry seam instead of CLI-local fallback constructors, and unknown basis IDs are not silently coerced into a known basis family. Engine validation rejects malformed typed basis details before helper persistence or resolver acceptance. Finding target projection now maps common scalar payload identifiers such as `artifact_id`, `zone_id`, and `pad_id` to concrete domain target kinds while preserving `objects[]` / `object_uuids[]` compatibility for legacy DRC/ERC payloads. Live and persisted `CheckFinding` records now also carry
  inline `explanation` and nullable `suggested_next_action` fields so GUI and
  agents do not need a separate index-addressed explain tool. GUI workspace
  load now best-effort runs the current CheckRun into `ReviewWorkspaceState`,
  the Outputs dock renders a compact latest-run/finding action lane, and those
  actions prefill canonical terminal commands for check show/run, zone-fill
  refresh, standards repair generation, proposal show/accept-apply, waiver,
  and accepted deviation without adding any GUI private writer. Terminal
  discovery context now includes visible check-run ids, visible finding
  fingerprints, the compact check status snapshot, and a profile-specific
  `datum-eda check run "$DATUM_PROJECT_ROOT" --profile <profile>` template for
  in-Datum agents. The Outputs dock now renders standards-basis evidence for
  process-aperture findings, exposes direct profile discovery via
  `datum-eda check profiles "$DATUM_PROJECT_ROOT"`, exposes persisted check-run
  history via `datum-eda check list "$DATUM_PROJECT_ROOT"`, and exposes direct
  `standards` / `release` profile rerun actions through the terminal command
  catalog and matching MCP aliases. GUI-written and CLI-refreshed terminal
  contexts now also expose `active_context_commands.artifact_list` and
  `active_context_commands.proposal_list` as always-available
  `datum-eda artifact list <project-root>` and
  `datum-eda proposal list <project-root>` discovery commands when the project
  root is known. When a previous artifact and current/latest artifact are both
  known, contexts also expose `active_context_commands.artifact_compare` as
  `datum-eda artifact compare <project-root> --before <previous> --after
  <latest>`, so terminal-launched agents can compare generated evidence without
  rescanning artifact history. GUI-written, CLI-refreshed, and MCP-preserved
  context envelopes also expose `active_context_commands.source_shards` as
  `datum-eda project query <project-root> resolve-debug`, giving terminal-launched
  agents a direct resolver/source-shard diagnostic command that mirrors the
  canonical `datum.query.source_shards` read model. They also expose `active_context_commands.check_run`,
  `active_context_commands.check_list`,
  `active_context_commands.check_profiles`, and
  `active_context_commands.check_fill_zones` as always-available
  `datum-eda check run|list|profiles|fill-zones <project-root>` commands, so
  terminal-launched agents can discover generated evidence, run checks, inspect
  profiles, and refresh ZoneFill generated evidence before a latest run or
  selected finding exists. Remaining CheckRun UX gaps are
  deeper profile configuration and richer first-class GUI review/apply widgets;
  the canonical waive/deviation taxonomy now exists for the current
  native-project slice.
- ERC `object_uuids[]` are no longer only preserved as raw compatibility
  payload. Live native `project query <root> erc` now projects those UUIDs into
  normalized `object_uuid` `primary_target` / `related_targets` fields while
  still preserving the raw array in payload/evidence. Focused ERC regression
  coverage fails if an ERC finding falls back to `erc/unknown` when object UUID
  evidence is present.
- Engine-daemon `run_erc` and `run_drc` now return live non-persisted
  `check_run_v1` envelopes instead of top-level raw arrays/reports. The raw
  compatibility payloads remain nested under `raw_report.erc` and
  `raw_report.drc`, while normalized findings carry target objects,
  deterministic fingerprints, summaries, profile basis, and coverage. The
  lower in-memory engine API remains raw; canonical persisted generated
  evidence remains `datum-eda check run` / `datum.check.run`.
- Persisted check-history list UX now exposes `latest_check_run_id`,
  `latest_profile_id`, `profile_latest_check_runs[]`, and per-row
  `latest_for_profile` flags from `datum-eda check list`, so GUI surfaces and
  terminal-launched agents can select the current run per profile without
  duplicating resolver sort semantics. GUI-written terminal context and CLI
  `context get|refresh` now carry the same latest CheckRun profile summary into
  `datum_check_context_v1` / top-level terminal-agent envelopes.
- Engine-daemon and MCP `explain_violation` now accept a stable finding
  `fingerprint` and resolve it against the current live CheckRun finding set.
  Legacy positional `index` remains accepted for compatibility, but new callers
  can avoid index drift when explaining ERC/DRC findings.
- Canonical MCP `datum.check.explain_violation` is now public and dispatches to
  the same fingerprint-capable implementation, while hidden flat
  `explain_violation` now declares `datum.check.explain_violation` as its real
  canonical replacement instead of a pending migration target. The MCP taxonomy
  guard now locks 338 public tools, 525 registered tools, and 187 hidden
  compatibility aliases.
- Lower engine DRC fingerprints now use
  `datum-eda:drc-violation-fingerprint:v2` material that includes
  `standards_basis`, `rule_revision`, and `import_key` slots. Standards-backed
  DRC producers stamp the current process-aperture/geometry basis plus
  `rule_revision="v1"` before fingerprinting, and daemon DRC CheckRun views
  preserve those fields instead of flattening them to null.

Artifact / taxonomy row correction:
- Manufacturing-set T0 projection correction: report, manifest, inspect,
  validate, compare, and export now share
  `native_project_manufacturing_projection` for expected artifact entries and
  Gerber plan context. The older row wording that only listed
  report/manifest/inspect/validate/compare is superseded by this correction.
- Artifact aliases are no longer pending for the current generated-evidence
  evidence slice. Canonical CLI now exposes `datum-eda artifact list`,
  `artifact generate`, `artifact show`, `artifact files`, `artifact preview`,
  `artifact compare`, `artifact validate`, `artifact export-manufacturing-set`,
  and `artifact validate-manufacturing-set`.
- MCP now exposes `datum.artifact.list`,
  `datum.artifact.generate`, `datum.artifact.show`, `datum.artifact.files`,
  `datum.artifact.preview`, `datum.artifact.compare`, `datum.artifact.validate`,
  `datum.artifact.export_manufacturing_set`, and
  `datum.artifact.validate_manufacturing_set`; the daemon bridge routes these
  through the canonical artifact CLI family.
- `artifact preview` / `datum.artifact.preview` verifies a requested safe
  relative file against resolver-owned artifact metadata, reads it from either
  explicit `--artifact-dir` or persisted `ArtifactMetadata.output_dir`, checks
  the on-disk hash against the metadata hash, and returns
  `artifact_file_preview_v1` using real semantic readers for supported
  RS-274X Gerber, Excellon drill inspection, bounded Gerber/Excellon preview
  primitives in nanometer coordinates, and CSV BOM/PnP/drill summaries.
- `artifact generate` / `datum.artifact.generate` now also accepts `bom`,
  `pnp`, and `drill` scopes. `bom` and `pnp` persist independent one-file CSV
  artifact metadata records; `drill` persists an independent drill-family
  artifact covering drill CSV plus Excellon drill, including the Excellon
  production projection proof. Generic `artifact validate` now performs
  family-specific semantic validation for those finer scopes, updates their
  persisted `ArtifactMetadata.validation_state`, and therefore feeds invalid
  BOM/PnP/drill artifacts into the existing artifact check-finding path.
  `project create-output-job --include <gerber-set|manufacturing-set|bom|pnp|drill|all>[,<scope>...]`
  now authors deterministic OutputJob templates for one or more implemented
  artifact scopes through the journaled `CreateOutputJob` path, and generated
  BOM/PnP/drill artifacts attach to a matching authored job when the
  prefix/scope match or when the stored include list contains the generated
  scope. Stored OutputJob variant context is now passed into direct BOM/PnP
  artifact scopes rather than only manufacturing-set generation.
  Direct manufacturing-set export, validate, compare, manifest, and inspect
  now accept `--include <scope>[,<scope>...]`, `--output-job <uuid>`, and
  exact-name `--job <name>`; selected jobs provide default prefix, variant, and
  include scope, duplicate names are rejected as ambiguous, and direct
  manufacturing export writes only the selected artifact families.
  Generic BOM/PnP/drill `artifact generate` executions now persist succeeded
  `OutputJobRun` evidence for linked authored jobs. Unlinked ad hoc finer-scope
  artifact generation persists separate `ArtifactRun` generated evidence so
  artifact history remains visible without inventing an authored `OutputJob`.
  `datum-eda artifact list` now exposes top-level `latest_artifact_id`,
  `latest_artifact_run_id`, and `latest_output_job_run_id` pointers so GUI
  surfaces and terminal-launched agents can select current generated evidence
  without duplicating artifact/run sorting. Latest artifact selection is now
  explicitly keyed by artifact `model_revision` plus UUID tie-break rather than
  resolver map order, with focused coverage proving a newer low-UUID artifact
  wins over an older high-UUID artifact.
  The GUI terminal discovery envelope and CLI `context get|refresh` now carry
  `latest_artifact_id`, `latest_artifact_run_id`, and `latest_check_run_id`
  alongside the existing latest output-job fields so terminal-launched agents
  can navigate generated evidence and check history from one context file.
  GUI production visibility now prefers canonical artifact-list
  `latest_output_job_run_id` over the legacy aggregate `latest_run_id` fallback
  when no output-job row carries its own latest run summary, and reconstructs
  latest output-job/artifact IDs from matching `artifact_runs[]` evidence.
  GUI production refresh now uses canonical `latest_artifact_id` /
  `latest_output_job_run_id` navigation context before falling back to output-job
  row order, so the OUTPUTS dock focuses the current generated evidence instead
  of the first artifact listed under an older job.
  `artifact_generate_v1.generated[]` entries now expose normalized top-level
  `output_job_run`, `output_job_run_path`, `artifact_run`, and
  `artifact_run_path` fields across Gerber-set, manufacturing-set, BOM, PnP,
  and drill scopes, while preserving the family-specific nested report fields.
  Generic multi-scope `run-output-job` now suppresses per-artifact run
  persistence and records one aggregate `OutputJobRun` for the logical command;
  the aggregate run keeps `artifact_id` null for multi-artifact executions and
  records the generated scopes in its log. `OutputJobRun.run_sequence` is
  assigned monotonically per authored job, repeated identical runs create
  distinct generated-evidence records, and `project query <root> output-jobs`
  reports status, execution count, sequence-ordered latest run, and artifact
  linkage for those finer scopes.
  Artifact-only CLI generation now has replay coverage matching the engine
  oracle: the focused regression removes promoted `.datum/artifacts/<id>.json`
  and `.datum/artifact_runs/<run>.json` sidecars after `datum-eda artifact
  generate`, then proves `ProjectResolver` recovers both generated-evidence
  records from the journal while the authored `ModelRevision` remains stable.
  `datum-eda artifact list/show` expose resolver-discovered `ArtifactRun`
  history for ad hoc generated artifacts and linked `OutputJobRun` history for
  artifacts generated by authored jobs; direct Gerber/manufacturing export
  reports now also include the persisted `output_job_run_path` when they create
  an `OutputJobRun`. GUI production status now consumes
  artifact-list evidence, stores ad hoc `ArtifactRun` summaries, can focus the
  latest ad hoc generated artifact, and renders ad hoc artifact-run rows in the
  Outputs dock as clickable artifact evidence. GUI production status also maps
  OutputJob include scopes into explicit normalized job-family labels (`GERBER
  SET`, `MANUFACTURING SET`, `BOM`, `PNP`, `DRILL`), retains the latest run's
  artifact id, and renders those family/run/artifact details in the Outputs
  dock instead of presenting finer-scope jobs as indistinguishable generic
  artifact rows.
- The remaining artifact taxonomy gap is richer family-specific GUI actions.

GUI production artifact drill-down correction:
- GUI terminal-launched production commands now keep production refresh pending
  across unchanged early PTY output, perform a final refresh attempt when the
  terminal exits, and use a bounded event-loop retry so proposal/artifact/output
  rows can refresh even when the command emits no useful trailing output.
- GUI production status now consumes the canonical `datum-eda artifact files`
  contract for a focused artifact discovered from output-job artifacts, stores
  it as `ProductionStatus.focused_artifact`, and renders focused file/hash plus
  production-projection proof rows in the `OUTPUTS` dock.
- Artifact rows in the `OUTPUTS` dock are now clickable hit targets that focus
  the selected artifact from production summaries through a session command.
- Focused artifact file rows are now clickable hit targets that select a file
  proof and render a generated-file viewer block with path/hash. The first
  dedicated viewer classifies Gerber, Excellon/NC drill, drill CSV, BOM, and
  PnP artifact files and attaches matching production-projection proof rows
  when available. Generated artifact metadata now carries optional
  `output_dir`, GUI production summaries/details retain it, production refresh
  can load `artifact_file_preview_v1` for the focused file without guessing
  filesystem layout, and the Outputs dock renders verified preview kind,
  hash status, primitive count, semantic counts, and a lightweight geometric
  CAM viewport from bounded Gerber/Excellon primitives. The CAM viewport now
  has first stateful drill-down controls in `WorkspaceUiState`: click targets
  and Outputs-dock mouse-wheel handling zoom the artifact preview, reset returns
  it to fit view, and geometry/drill toggles gate rendered Gerber and Excellon
  primitive families. The preview body is now a hit-test target, and middle/
  right drag over it pans the generated-artifact viewport through normalized
  `pan_x_ppm` / `pan_y_ppm` state instead of moving the board camera. The
  artifact preview contract now carries bounded CSV columns and sample rows for
  BOM, PnP, and drill-table artifacts, and the Outputs dock renders those rows
  as a small table instead of only showing `row_count`. Focused generated
  artifacts now also expose clickable top-level `datum-eda artifact list`,
  focused-artifact `datum-eda artifact show`, `datum-eda artifact validate`,
  `datum-eda artifact files`, and focused-file `datum-eda artifact preview`
  terminal actions, and the artifact-run section exposes
  `datum-eda artifact compare "$DATUM_PROJECT_ROOT" --before <older> --after <newer>`
  for the latest two distinct generated artifacts, so output-job artifacts can
  be discovered, inspected, compared, and validated from the same drill-down surface as ad hoc artifact-run evidence. The
  remaining GUI artifact UX gap is full generated-artifact drill-down with
  richer layer/family controls, richer artifact-family-specific viewers, and
  independently modeled drill, BOM, and PnP artifact scopes; drill-down data is
  no longer limited to capped output-job aggregate summaries, row counts, or
  the first discovered artifact.

---

## Current Repo Health

Current repo health status (2026-07-02):
- `cargo test --workspace` passes (2,006 tests).
- 358 Python MCP tests pass.
- The full drift-gate suite (`scripts/run_drift_gates.sh`) is green.

Current M7 delivery rule (2026-04-16):
- opening `M7` work may not advance on a "low resolution but technically
  implemented" basis
- user-facing slices must be intentionally triggerable, externally observable,
  and supported by the minimum interaction/render substrate they depend on
- missing prerequisite work in selection, hit-testing, focus/relatedness,
  visibility, or render-state consistency is not "scope creep"; it is
  prerequisite completion for the slice already being claimed
- working note:
  `docs/gui/M7_DELIVERY_GATES.md`

### Drift RCA + Prevention (2026-03-25)

Drift cause (root):
- Prior alignment checks were primarily static text-presence checks.
- They did not verify factual claims against live repo state (for example
  daemon method count, git/CI status, and current MCP method catalog parity).
- `specs/MCP_API_SPEC.md` mixed historical deferral wording with current write
  support, creating internal contradiction.

Corrections landed:
- Added `scripts/check_progress_coverage.py` to enforce:
  - `PLAN.md` progress-block coverage for active milestones (`M0`-`M4`, `R1`)
  - `specs/PROGRESS.md` section presence and infrastructure-fact parity
  - MCP current-method parity (`dispatch.rs` ↔ `tools_catalog.py` ↔
    `specs/MCP_API_SPEC.md` list/headings)
  - stale deferral text rejection for write/save support
- Wired `check_progress_coverage.py` into CI (`.github/workflows/alignment.yml`).

---

## Legacy Implementation Evidence

The following milestone tables preserve historical/current implementation
evidence. They are not the active North Star for the new scope. Use them to
ground factual current-state claims, then promote only substrate-relevant facts
into "Scope Integration / Substrate Readiness" above.

---

## PROGRAM_SPEC.md — M0 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Pool types: Unit, Pin, Entity, Gate, Package, Pad, Padstack, Part, Symbol | [x] | `pool/mod.rs` |
| Pool serialization round-trip (100% of types) | [x] | serde derive + golden tests |
| Pool SQLite index: create, insert, keyword search, parametric search | [x] | `pool/mod.rs` PoolIndex |
| Deterministic serialization (byte-identical 3 runs) | [x] | `ir/serialization.rs` |
| Eagle .lbr import (20 libraries, 0 errors) | [x] | `import/eagle/mod.rs` |
| Eagle canonicalization round-trip | [x] | import → pool JSON → deserialize |
| Eagle deterministic re-import (identical UUIDs) | [x] | UUID v5 with eagle namespace |
| RuleScope IR compiles, serializes, leaf evaluator works | [x] | `rules/ast.rs`, `rules/eval.rs` |
| UUID v5 import identity | [x] | `ir/ids.rs` |
| .ids.json sidecar (write on import, restore on re-import) | [x] | `import/ids_sidecar.rs` |
| **No .lbr write-back** | [x] | import-only, confirmed |

**M0 overall**: [x] Complete (closed by architect)

---

## PROGRAM_SPEC.md — M1 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| KiCad .kicad_pcb import: DOA2526 + 4 designs, 0 errors | [~] | Skeleton parser exists; warns "full geometry not implemented" |
| KiCad .kicad_sch import: DOA2526 + 2 schematics, 0 errors | [~] | Skeleton parser exists; warns "full symbol connectivity not implemented" |
| Eagle .brd/.sch import: 3 designs, 0 errors | [ ] | Returns "not implemented" error |
| Schematic connectivity: hierarchy, labels, power symbols | [~] | Union-find solver in `connectivity/mod.rs`; imported KiCad child-sheet hierarchical linking and basic bus/member expansion now feed existing net/query/check surfaces for the current supported subset, while wider import fidelity and advanced bus syntax remain open |
| Board connectivity: net-to-pin resolution | [x] | `board/mod.rs` net_pins(), net_pad_points() |
| Airwire computation: matches KiCad on DOA2526 | [~] | Algorithm implemented; DOA2526 validation not confirmed |
| Board diagnostics: net_without_copper, via_only, partially_routed | [x] | `board/mod.rs` diagnostics() |
| Golden tests: 8+ designs with checked-in golden files | [ ] | `tests/corpus/` is empty |
| Deterministic import: byte-identical on 3 runs | [~] | Current KiCad import/query corpus is now gated by `scripts/check_import_query_determinism.py` over checked-in query/check fixtures and passes 3-run stability for the current supported subset; broader import fidelity and non-query/save-backed import determinism remain open |
| Import fidelity KiCad ≥ 90% | [ ] | No fidelity matrix exists |
| Import fidelity Eagle ≥ 85% | [ ] | Eagle design import not implemented |

### PROGRAM_SPEC.md — M1 Query API

| Method | Engine | Daemon | MCP | CLI |
|--------|--------|--------|-----|-----|
| get_netlist | [x] | [x] | [x] | [x] |
| get_components | [x] | [x] | [x] | [x] |
| get_nets / get_net_info | [x] | [x] | [x] | [x] |
| get_schematic_net_info | [x] | [x] | [x] | [x] |
| get_board_summary | [x] | [x] | [x] | [x] |
| get_schematic_summary | [x] | [x] | [x] | [x] |
| get_sheets | [x] | [x] | [x] | [x] |
| get_symbols | [x] | [x] | [x] | [x] |
| get_ports | [x] | [x] | [x] | [x] |
| get_labels | [x] | [x] | [x] | [x] |
| get_buses | [x] | [x] | [x] | [x] |
| get_bus_entries | [x] | [x] | [x] | [x] |
| get_noconnects | [x] | [x] | [x] | [x] |
| get_hierarchy | [x] | [x] | [x] | [x] |
| get_connectivity_diagnostics | [x] | [x] | [x] | [x] |
| get_unrouted | [x] | [x] | [x] | [x] |
| search_pool | [x] | [x] | [x] | [x] |

**M1 overall**: [~] Query-surface parity is now complete across engine/daemon/MCP/CLI for the current imported-design read slice; import fidelity, corpus, and broader golden coverage are still open

M1 imported-query reliability note (2026-03-30):
- Checked-in CLI query goldens now cover the current imported-design read
  surfaces for `simple-demo.kicad_sch`, `bus-demo.kicad_sch`,
  `hierarchy-mismatch-demo.kicad_sch`, `erc-coverage-demo.kicad_sch`,
  `simple-demo.kicad_pcb`, and `airwire-demo.kicad_pcb`:
  - `summary`
  - `netlist`
  - `nets`
  - `schematic-nets`
  - `sheets`
  - `symbols`
  - `buses`
  - `bus-entries`
  - `noconnects`
  - `labels`
  - `ports`
  - `hierarchy`
  - `diagnostics`
  - `unrouted`
- Those fixtures live under `crates/cli/testdata/golden/query` and are enforced
  by `main_tests_query_goldens`, with `UPDATE_GOLDENS=1` as the explicit
  regeneration path.
- Checked-in CLI check goldens now cover the imported schematic fixtures already
  used by the current M1/M2 corpus:
  - `simple-demo.kicad_sch`
  - `bus-demo.kicad_sch`
  - `analog-input-demo.kicad_sch`
  - `analog-input-bias-demo.kicad_sch`
  - `erc-coverage-demo.kicad_sch`
  - `hierarchy-mismatch-demo.kicad_sch`
- Those fixtures live under `crates/cli/testdata/golden/check` and are enforced
  by `main_tests_check_goldens`, again with `UPDATE_GOLDENS=1` for intentional
  regeneration.
- Imported KiCad schematic hierarchy now follows the current supported child-sheet
  subset:
  - `simple-demo.kicad_sch` now resolves its checked-in `sub.kicad_sch`
    child file during import
  - hierarchical net propagation now merges the parent sheet pin and
    hierarchical label with the child-sheet hierarchical label target
  - existing query/check surfaces now expose the resulting extra child sheet,
    hierarchy link, and propagated `SUB_IN` pin attachment without adding new
    endpoints
  - resolver diagnostics now report missing or multiply-mapped hierarchical
    child targets deterministically when the imported subset cannot link them
- Imported KiCad bus/member expansion now covers the current supported subset:
  - scalar member labels like `DATA[0]` resolve to deterministic scalar net
    names like `DATA0` on existing net/query/check surfaces
  - simple bus-range labels like `DATA[0..1]` populate imported bus members
  - imported `bus_entry` forms now associate to one bus and one wire when the
    current geometric subset is unambiguous
  - the checked-in `bus-demo.kicad_sch` fixture plus CLI goldens now exercise
    `schematic-nets`, `buses`, `bus-entries`, and `diagnostics` for this path
- The current KiCad import/query corpus now has a repo-native determinism gate:
  - manifest: `crates/test-harness/testdata/quality/import_query_determinism_manifest_v1.json`
  - gate: `python3 scripts/check_import_query_determinism.py`
  - CI: `.github/workflows/alignment.yml`
  - scope: repeated `query` and `check` runs across the checked-in KiCad board
    and schematic fixture corpus used by the current M1 goldens
  - current result: 36/36 cases stable across 3 repeated runs for the current
    supported subset

---

## PROGRAM_SPEC.md — M2 Exit Criteria

### ERC Rules (specs/ERC_SPEC.md §4)

| Check | Status | Notes |
|-------|--------|-------|
| output_to_output_conflict | [x] | `erc/mod.rs` |
| undriven_input | [x] | `erc/mod.rs` (undriven_input_pin + input_without_explicit_driver) |
| power_without_source | [x] | `erc/mod.rs` (power_in_without_source) |
| noconnect_connected | [x] | `erc/mod.rs` |
| unconnected_required_pin | [x] | `erc/mod.rs` (unconnected_component_pin) |
| passive_only_net | [~] | No distinct `passive_only_net` rule/code ships in `erc/mod.rs` (0 grep hits 2026-06-22). Passive pins are only consumed as a modifier that softens `input_without_explicit_driver` messaging; a standalone passive-only-net check is target work. |
| hierarchical_connectivity_mismatch | [x] | Implemented as sheet-level hierarchical-label vs port mismatch check in `erc/mod.rs` |

ERC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current ERC fixtures (2026-03-25)

### DRC Rules

| Check | Status | Notes |
|-------|--------|-------|
| clearance_copper | [x] | Implemented in `drc/mod.rs` (track-track same-layer clearance) |
| track_width | [x] | Implemented in `drc/mod.rs` (`track_width_below_min`) |
| via_hole | [x] | Implemented in `drc/mod.rs` (`via_hole_out_of_range`) |
| via_annular | [x] | Implemented in `drc/mod.rs` (`via_annular_below_min`) |
| connectivity | [x] | Implemented in `drc/mod.rs` (no-copper + unrouted-net checks) |
| unconnected_pins | [x] | Implemented in `drc/mod.rs` (`connectivity_unconnected_pin`) |
| silk_clearance | [x] | Implemented in `drc/mod.rs` (`silk_clearance_copper`) |

DRC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current DRC fixtures (2026-03-25)

### MCP Tools (specs/MCP_API_SPEC.md — M2)

| Tool | Status | Notes |
|------|--------|-------|
| open_project | [x] | Current implementation contract |
| close_project | [x] | Current implementation contract |
| search_pool | [x] | Current implementation contract |
| get_part | [x] | Current implementation contract |
| get_package | [x] | Current implementation contract |
| get_package_change_candidates | [x] | Current implementation contract |
| get_board_summary | [x] | Current implementation contract |
| get_schematic_summary | [x] | Current implementation contract |
| get_sheets | [x] | Current implementation contract |
| get_symbols | [x] | Current (all-sheets only) |
| get_ports | [x] | Current (all-sheets only) |
| get_labels | [x] | Current (all-sheets only) |
| get_buses | [x] | Current (all-sheets only) |
| get_bus_entries | [x] | Current implementation contract |
| get_noconnects | [x] | Current (all-sheets only) |
| get_symbol_fields | [x] | Current implementation contract |
| get_hierarchy | [x] | Current implementation contract |
| get_netlist | [x] | Current implementation contract |
| get_components | [x] | Current implementation contract |
| get_net_info | [x] | Current (full inventory, not single-net selector) |
| get_schematic_net_info | [x] | Current (full inventory, not single-net selector) |
| get_connectivity_diagnostics | [x] | Current implementation contract |
| get_design_rules | [x] | Current implementation contract |
| get_unrouted | [x] | Current implementation contract |
| get_check_report | [x] | Current implementation contract (not in M2 exit list but implemented) |
| run_erc | [x] | Current implementation contract |
| run_drc | [x] | Current implementation contract |
| explain_violation | [x] | Current implementation contract |

MCP tools: M2 slice 26/26 implemented. Current MCP runtime catalog: 181
methods (daemon-dispatched + CLI-bridged via `mcp-server/server_runtime.py`),
locked via `specs/SPEC_PARITY.md` → `mcp_runtime_methods`.

### CLI Commands (specs/PROGRAM_SPEC.md — M2)

> **Scope note.** The table below is the *M2 historical slice* — the eight
> commands M2 froze. It is **not** the current CLI surface. The present
> `tool project` surface is **256 commands**, locked via
> `specs/SPEC_PARITY.md` → `cli_project_commands`. For the authoritative,
> code-derived enumeration run
> `python3 scripts/check_spec_parity.py --print` and read the
> `[cli_project_commands]` block. Do not read this M2 table as today's reach.

| Command | Status | Notes |
|---------|--------|-------|
| tool import \<file\> | [x] | Eagle .lbr only in current slice |
| tool query \<design\> --nets | [x] | |
| tool query \<design\> --components | [x] | |
| tool query \<design\> --summary | [x] | |
| tool erc \<design\> | [x] | .kicad_sch only |
| tool drc \<design\> | [x] | Runs DRC, reports JSON/text, exits nonzero on violations |
| tool pool search \<query\> | [x] | |
| tool check \<design\> | [x] | Unified check report |
| CLI exit codes (0/1/2) | [x] | Checking commands now return 1 on violations and 2 only on execution errors |

### Other M2 Requirements

| Criterion | Status | Notes |
|-----------|--------|-------|
| MCP registration in active MCP host config | [x] | Configured `datum-eda` MCP entry with `EDA_ENGINE_SOCKET=/tmp/datum-eda-engine.sock` (2026-03-25) |
| Test corpus: 10+ designs with ERC/DRC golden files | [x] | `m2_quality` reports 11 unique designs (erc=5, drc=6) |
| Quality-rate harness (ERC/DRC FP/FN + corpus gate) | [x] | `m2_quality` implemented with checked-in manifest; current gate state is `pass=true` |
| ERC on DOA2526 < 3 seconds | [x] | `m2_perf` median ERC = 2ms (3-iteration run, 2026-03-25) |
| DRC on DOA2526 < 5 seconds | [x] | `m2_perf` median DRC = 18ms baseline, 20ms compare run (2026-03-25) |
| MCP transport (daemon ↔ MCP server) | [x] | Unix socket JSON-RPC implemented in daemon + Python client; live transport smoke is tracked separately in unrestricted CI |

**M2 overall**: [x] Complete for the current implementation slice — ERC/DRC checks, MCP/CLI parity, quality/performance harnesses, and local MCP host registration are all in place.

### M2 Pre-Freeze Closeout Checklist (Ordered)

This is the execution order to close meaningful M2 gaps before full integrated
program specification freeze.

| Order | Work Item | Exit Condition | Status |
|------:|-----------|----------------|--------|
| 1 | DRC foundation + first runnable checks | `run_drc` returns structured violations for at least connectivity + clearance on fixture boards | [x] |
| 2 | CLI DRC integration | `tool drc <board.kicad_pcb>` works with pass/fail exit behavior and JSON output | [x] |
| 3 | MCP/daemon DRC path | MCP `run_drc` round-trips through daemon with stable payload shape | [x] |
| 4 | Remaining high-value MCP query parity | `get_sheets`, `get_netlist`, `get_design_rules` implemented or explicitly deferred with gate note | [x] |
| 5 | ERC corpus hardening | Corpus-backed ERC goldens exist and cover required M2 codes in current implementation slice | [x] |
| 6 | DRC corpus hardening | Corpus-backed DRC goldens exist for implemented DRC checks | [x] |
| 7 | Performance gate harness | Reproducible timing harness for ERC/DRC on DOA2526 with recorded baseline | [x] |
| 8 | M2 gate reconciliation pass | `PROGRAM_SPEC.md`, `MCP_API_SPEC.md`, `ENGINE_SPEC.md`, `PLAN.md`, `PROGRESS.md` mutually consistent for remaining open items | [x] |

Item 4 defer notes (2026-03-25):
- `get_sheets`: implemented in engine/daemon/MCP.
- `get_design_rules`: implemented in engine/daemon/MCP using the current rule
  evaluator subset payload.
- `get_netlist`: implemented in engine/daemon/MCP using a canonical net
  inventory payload with board-vs-schematic field parity.

Item 5 current artifacts (2026-03-25):
- ERC golden fixtures added:
  - `crates/engine/testdata/golden/erc/simple-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-bias-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/erc-coverage-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/hierarchy-mismatch-demo.kicad_sch.json`
- Golden validation tests:
  - `api::tests::erc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_bias_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_hierarchy_mismatch_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_corpus_covers_required_m2_codes_for_current_implementation_slice`
- Golden contract is normalized to stable semantic fields (code/severity/message/
  net/component/pin/objects/waived) to avoid false churn from volatile UUIDs.
- Added implementation slice coverage for required ERC codes:
  - `output_to_output_conflict`
  - `undriven_input_pin`
  - `input_without_explicit_driver`
  - `power_in_without_source`
  - `unconnected_component_pin`
  - `undriven_power_net`
  - `noconnect_connected`
  - `hierarchical_connectivity_mismatch`
- Note: current implementation is sheet-level interface name consistency. Full
  instance-aware hierarchical net resolution remains tracked under schematic
  connectivity maturity (M1/M3+), not as an open M2 rule omission.

Item 6 current artifacts (2026-03-25):
- DRC golden fixtures added:
  - `crates/engine/testdata/golden/drc/simple-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/partial-route-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/airwire-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/clearance-violation-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/drc-coverage-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/silk-clearance-demo.kicad_pcb.json`
- Golden validation tests:
  - `api::tests::drc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_partial_route_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_airwire_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_clearance_violation_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_silk_clearance_demo_matches_checked_in_fixture`
- Golden contract is normalized to stable semantic fields (pass/fail, summary,
  code/rule/severity/message/location/objects) and excludes volatile violation IDs.
- Coverage for implemented DRC checks now includes:
  - connectivity (`connectivity_unrouted_net`)
  - clearance (`clearance_copper`)
  - connectivity unconnected pin (`connectivity_unconnected_pin`)
  - track width (`track_width_below_min`)
  - via hole (`via_hole_out_of_range`)
  - via annular ring (`via_annular_below_min`)
  - silk clearance (`silk_clearance_copper`)
- Remaining gap: broaden corpus and validation metrics against external reference
  behavior for quality-rate gates.
- Corpus-size gate is now met by checked-in golden fixtures:
  - total unique designs: 10
  - DRC additions in this pass include `airwire-demo.kicad_pcb`.
  - ERC additions in this pass include `analog-input-bias-demo.kicad_sch`.

Item 7 current artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_perf.rs`
- Baseline artifact:
  - `crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Commands:
  - Baseline write:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --write-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
  - Baseline compare:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --compare-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Current measured medians:
  - Baseline run: import_board=1200ms, import_schematic=1759ms, ERC=2ms, DRC=18ms
  - Compare run: import_board=1285ms, import_schematic=1838ms, ERC=2ms, DRC=20ms

Item 9 quality-rate gate artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_quality.rs`
- Manifest artifact:
  - `crates/test-harness/testdata/quality/m2_quality_manifest.json`
- Command:
  - `cargo run -p eda-test-harness --bin m2_quality -- --json`
- Current measured output:
  - `erc false_positive_rate_pct=0.0`
  - `erc false_negative_rate_pct=0.0`
  - `drc false_positive_rate_pct=0.0`
  - `drc false_negative_rate_pct=0.0`
  - `unique_designs=11`, `required_min_designs=10`
  - `pass=true`

Item 8 reconciliation notes (2026-03-25):
- Performance harness status synced across:
  - `specs/PROGRESS.md` M2 requirement table
  - `specs/PROGRESS.md` closeout checklist
  - `PLAN.md` M2 progress section
- Deferred MCP query language remains aligned across:
  - `specs/PROGRAM_SPEC.md`
  - `specs/MCP_API_SPEC.md`
- Stale wording corrected:
  - `Waiver matching in DRC` now states DRC exists and waiver matching is the remaining gap.

Checking follow-up note (2026-03-30):
- Native-project authored waivers are now honored by the existing `project query erc`
  and `project query check` load path.
- DRC-domain waiver matching is now honored inside `run_drc`; waived DRC
  violations remain visible and waived-only DRC runs do not fail.
- Native authored board state now exposes the same waiver-aware DRC report through
  `project query <dir> drc`.
- Native `project query <dir> check` now returns a combined report with distinct
  `erc` and `drc` sections while leaving `project query <dir> board-check`
  unchanged.
- Native-project structural validation is now exposed through
  `project validate <dir>`, covering required native files, schema-version
  compatibility, duplicate UUID consistency within authored object types, and
  non-dangling persisted schematic/board references.
- The same native-project validation contract is now available through MCP as
  `validate_project`, preserving the CLI report shape and valid/invalid result
  semantics.
- MCP tool registration now also uses one shared spec table for `tools/list`
  export and `tools/call` dispatch, with parity tests checking runtime and
  fake-daemon coverage for each registered tool.
- Checked-in native fixture projects are now exercised continuously through
  `scripts/check_native_project_fixtures.py`, driven by
  `crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json`
  and the existing `project validate` contract in CI.
- That manifest now covers both real route-strategy native fixtures and a
  dedicated checked-in invalid-case suite for duplicate UUID, missing sheet,
  and unsupported schema-version failures, and it currently exhausts all
  checked-in native project roots under `crates/test-harness/testdata/quality`.

### 2026-03-25 Contract Alignment Pass

| Item | Status | Notes |
|------|--------|-------|
| Integrated spec scaffold added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` |
| Integrated contract verification matrix added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §8 |
| Integrated M3/M4 boundary contracts drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§9-11 |
| Integrated M3/M4 acceptance tables drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§12-13 |
| M3 determinism evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_op_determinism.rs` and `crates/test-harness/src/bin/m3_replacement_op_determinism.rs` together pass for the full current save-backed M3 mutation slice: `move_component`, `delete_track`, `delete_via`, `delete_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` |
| M3 undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_undo_redo_roundtrip.rs` passes for current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, and `set_net_class` undo/redo slice |
| M3 replacement undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_replacement_undo_redo_roundtrip.rs` passes for current `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` undo/redo slice |
| M3 board round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_board_roundtrip_fidelity.rs` passes for unmodified KiCad-board identity plus current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, and `set_reference` save→reimport→save artifact stability |
| M3 sidecar round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs` passes for current `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` save→reimport→save slice: rules/net-class remain sidecar-only byte-stable, while part/package replacement flows verify semantic sidecar replay and reimported board state after intentional footprint/body materialization |
| M3 write-surface parity hook behavioral | [x] | `crates/test-harness/src/bin/m3_write_surface_parity.rs` passes for current engine/daemon/MCP/CLI `move_component`/`rotate_component`/`set_value`/`set_reference`/`assign_part`/`set_package`/`set_package_with_part`/`set_net_class`/`delete_component`/`delete_track`/`delete_via`/`set_design_rule`/`undo`/`redo`/`save` slice, including follow-up derived-state checks |
| M3 aggregate acceptance gate behavioral | [x] | `crates/test-harness/src/bin/m3_acceptance_gate.rs` passes and composes base determinism, replacement determinism, base undo/redo, replacement undo/redo, board round-trip fidelity, sidecar round-trip fidelity, and write-surface parity into one milestone checkpoint |
| PLAN progress ticks added for M0/M1/M2/R1 | [x] | `PLAN.md` now has dated progress blocks |
| MCP contract split (current vs target) | [x] | `specs/MCP_API_SPEC.md` |
| PROGRAM spec references MCP contract split | [x] | `specs/PROGRAM_SPEC.md` M2 status section |
| ENGINE spec API split (current vs target) | [x] | `specs/ENGINE_SPEC.md` §5 |
| MCP design doc hardened to target/current labels | [x] | `docs/MCP_DESIGN.md` |
| User workflows aligned to current MCP/CLI surfaces | [x] | `docs/USER_WORKFLOWS.md` |
| Engine design doc marked target-state vs live API | [x] | `docs/ENGINE_DESIGN.md` |

---

## PROGRAM_SPEC.md — R1 Research Gates

| Criterion | Status | Notes |
|-----------|--------|-------|
| R1-G0 Foundation Gate | [x] | Minimum history/context gate is now evidenced in `docs/R1_G0_FOUNDATION.md` and blocks downstream milestone completion claims unless maintained |
| R1-G0 tool lineage + format evolution map | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §1 covers KiCad/Eagle/Altium/PADS/OrCAD lineage, likely ingestion surfaces, and roadmap implications |
| R1-G0 migration pain taxonomy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §2 classifies migration failure modes using the current KiCad/Eagle design/library corpus and current interop workflow evidence |
| R1-G0 fidelity boundary policy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §3 defines exactness, approximation, preservation-as-metadata, and unsupported-loss boundaries for future interop claims |

**R1 overall**: [~] Minimal context gate is complete, but broader commercial-interop research exit criteria remain open (corpus, legal posture, prototypes, recommendation)

---

## PROGRAM_SPEC.md — M3 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| MoveComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#movecomponent`. |
| RotateComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#rotatecomponent`. |
| DeleteComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletecomponent`. |
| SetValue | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setvalue`. |
| SetReference | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setreference`. |
| AssignPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#assignpart`. |
| SetPackage | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackage`. |
| SetPackageWithPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackagewithpart`. |
| ReplaceComponents | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacecomponents`. |
| ApplyComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementplan`. |
| ApplyComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementplan`. |
| ScopedReplacementPlanManifest | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#scopedreplacementplanmanifest`. |
| Package-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#package-change-introspection`. |
| Part-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#part-change-introspection`. |
| Replacement-plan introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacement-plan-introspection`. |
| SetNetClass | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setnetclass`. |
| SetDesignRule | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setdesignrule`. |
| DeleteTrack | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletetrack`. |
| DeleteVia | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletevia`. |
| Undo/redo (100% undoable) | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#undoredo-100-undoable`. |
| Operation determinism | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#operation-determinism`. |
| KiCad write-back | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#kicad-write-back`. |
| Round-trip fidelity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#round-trip-fidelity`. |
| MCP write tools | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#mcp-write-tools`. |
| Derived data recomputation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#derived-data-recomputation`. |
| CLI modify command | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#cli-modify-command`. |

**M3 overall**: [x] M3 is complete for the implemented imported-board write slice; criterion-level details and evidence anchors are consolidated in `specs/progress/m3_details.md`.

---

## PROGRAM_SPEC.md — M4 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Native format | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#native-format`. |
| Schematic operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-operations`. |
| Board operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#board-operations`. |
| Schematic query parity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-query-parity`. |
| Forward annotation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#forward-annotation`. |
| Gerber export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-export`. |
| Drill export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#drill-export`. |
| BOM export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#bom-export`. |
| PnP export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#pnp-export`. |
| Gerber comparison | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-comparison`. |

**M4 overall**: [x] Closed for scope; details remain in `specs/progress/m4_details.md`.

## M5 Opening Charter

Status: [~] Routing-kernel scope complete; closure review pending
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m5_opening.md`
- Recommended M5 focus: deterministic layout-kernel groundwork from persisted
  native board state, opening with one narrow routing/constraint slice rather
  than broad placement/routing ambition.
- Governance narrowing (2026-03-30):
  - for milestone closure, `M5` is now interpreted as the deterministic
    persisted-state routing-kernel milestone recorded in the opening charter
    and current frontier below
  - placement-kernel and placement/routing co-optimization work are deferred
    to a later reopened milestone/slice and are not required for `M5` closure
  - `M6` may open from the completed routing-kernel substrate once closure
    review accepts this narrowed milestone scope
- M5 must not inherit M4 parity work by default; new slices require an
  explicit layout-kernel contract and acceptance criteria.
- MCP implementation parity remains deferred unless explicitly reopened, but
  deferred parity tracking for newer native M4/M5 slices must stay current in
  `specs/MCP_API_SPEC.md`.
- A narrow MCP parity exception is now live for the policy-selected
  authored-copper-graph proposal export surface; broader M5 MCP query/apply
  parity remains deferred.
- That narrow exception now covers the full route-proposal artifact lifecycle
  for the policy-selected authored-copper-graph family: export, inspect, and
  apply.
- Current M5 checkpoint chain (routing-kernel read/query lane):
  - `project query <dir> routing-substrate`
  - `project query <dir> route-preflight --net <uuid>`
  - `project query <dir> route-corridor --net <uuid>`
  - canonical route query surface: `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - canonical route explanation surface: `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - contract-specific `route-path-candidate-*` and
    `route-path-candidate-*-explain` commands are now compatibility wrappers
    around those bounded generic surfaces
  - the remaining legacy wrapper implementations now also dispatch through the
    same shared generic candidate/policy executor path as the canonical
    surfaces, reducing wrapper-only branching ahead of removal
- Current M5 routing expansion (persisted-via reuse lane):
  - fixed-arity via reuse was proven through bounded ordinal slices and is no
    longer the preferred growth model
  - current generalized contract is `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - current adjacent observability surface is `project query <dir>
    route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - a same-layer synthesis slice now also exists via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-graph`, with the
    paired explanation surface under `route-path-candidate-explain`
  - a first bounded cross-layer graph-search lane now also exists via
    `project query <dir> route-path-candidate --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`, with the paired explanation
    surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers `project
    query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`, with the paired
    explanation surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers the
    remaining authored-via graph sequence via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`,
    with the paired explanation surface under `route-path-candidate-explain`
  - the shared orthogonal graph selector now uses an explicit deterministic
    cost rule across that whole family: bend count ascending, then segment
    count ascending, then point-sequence coordinate ascending
  - the same orthogonal graph query and explanation reports now expose that
    selected path cost directly as bend/segment/point counts on the returned
    path or per-segment path data
  - the same orthogonal graph route proposal artifact lane now preserves the
    selected bend count in exported actions and exposes it again through
    artifact inspection/apply reporting
  - `export-route-path-proposal` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same-layer orthogonal-graph `route-path-candidate` and
    `route-path-candidate-explain` reports now also return `segment_evidence`
    so direct query output matches the artifact lane vocabulary
  - the same-layer orthogonal-graph candidate and explain surfaces now also
    share one internal graph-search spine, keeping the public reports
    unchanged while consolidating the preflight/search/segment-evidence path
  - the one-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal via-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - the two-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal pair-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - orthogonal-graph artifact apply now reports whether stale proposals
    drifted because candidate availability changed, the deterministic ranked
    winner changed, or same-rank geometry changed
  - `inspect-route-proposal-artifact` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same lane now also has `project revalidate-route-proposal-artifact
    <dir> --artifact <path>` so callers can read that drift classification
    and live/recorded path summaries without applying
  - that revalidation report now also carries segment-level orthogonal-graph
    evidence so stale proposals can show which layer-side segment changed and
    how its bend/point/track-action facts differ live
- Current M5 existing-copper readback lane:
  - deterministic authored-copper graph path queries now exist in
    increasingly filtered/readback-focused forms recorded in
    `specs/progress/m5_opening.md`
  - that same generalized query surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - that same generalized explanation surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - accepted bounded policies are `plain`, `zone_aware`, `obstacle_aware`,
    `zone_obstacle_aware`, `zone_obstacle_topology_aware`, and
    `zone_obstacle_topology_layer_balance_aware`
  - the plus-one-gap bridge is now reached canonically through `project query
    <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap`
  - a first write-capable bridge now exists for that accepted query contract:
    `project export-route-path-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-plus-one-gap --out <path>`,
    `project inspect-route-proposal-artifact <path>`, and
    `project apply-route-proposal-artifact <dir> --artifact <path>`
  - the same route-proposal artifact lane now also covers the accepted
    completed write-capable family, now canonically via `project
    export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy
    <policy>] --out <path>`, spanning the accepted single-layer, via-path, and
    authored-copper-graph policy-selected contracts
  - a native direct convenience apply surface now also exists for that same
    accepted single-layer path contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded single-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded two-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-two-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded three-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-three-via`
  - the same direct convenience apply lane now also covers the remaining
    bounded ordinal via contracts via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-four-via|route-path-candidate-five-via|route-path-candidate-six-via`
  - that same direct convenience apply lane now also covers the accepted
    deterministic authored-via-chain contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-authored-via-chain`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal dogleg contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-dogleg`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal two-bend contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-two-bend`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal graph contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph`
  - that same direct convenience apply lane now also covers the accepted
    one-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`
  - that same direct convenience apply lane now also covers the accepted
    two-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`
  - that same direct convenience apply lane now also covers the remaining
    authored-via orthogonal graph sequence via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`
  - a bounded convenience export surface now also exists for the completed
    write-capable route family via `project export-route-path-proposal <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    <accepted_candidate> [--policy <policy>] --out <path>`
  - `project route-apply` now parses `--candidate` from a bounded accepted
    value set instead of a free-form string, and enforces `--policy` only for
    `--candidate authored-copper-graph`
  - a native direct convenience apply surface now also exists for that same
    policy-selected family via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-graph --policy <policy>`
  - the bounded convenience export surface is now exposed through MCP as
    `export_route_path_proposal`
  - the bounded direct route-apply surface is now exposed through MCP as
    `route_apply`
  - the matching generic artifact follow-up surfaces are now exposed through
    MCP as `inspect_route_proposal_artifact`,
    `revalidate_route_proposal_artifact`, and
    `apply_route_proposal_artifact`
  - `specs/PROGRESS.md` tracks only the checkpoint/frontier; detailed per-slice
    history stays in `specs/progress/m5_opening.md`
- Current M5 frontier:
  - deterministic persisted-state layout-kernel/routing queries continue under
    explicit contract selection from `specs/progress/m5_opening.md`
  - a bounded native route selector now exists via `project route-proposal
    <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`,
    selecting the first successful family from the explicit accepted candidate
    order recorded in `specs/progress/m5_opening.md`
  - that same selector now also feeds a selected-proposal write lane via
    `project export-route-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --out <path>` and
    `project route-apply-selected <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also has a bounded explanation surface via
    `project route-proposal-explain <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also accepts a bounded deterministic profile set
    via `--profile default|authored-copper-priority`; the default order is
    unchanged, while `authored-copper-priority` prepends the accepted
    authored-copper-graph policy family ahead of the existing default family
    order without introducing scoring
  - a narrow MCP parity reopening now covers the selector lane via
    `route_proposal`, `route_proposal_explain`, `export_route_proposal`, and
    `route_apply_selected`
  - the first artifact/export/apply write lane now covers:
    the accepted plus-one-gap bridge, the accepted single-layer
    `route-path-candidate` contract, and the accepted bounded single-via
    `route-path-candidate-via` contract, and the accepted bounded two-via
    `route-path-candidate-two-via` contract, and the remaining bounded
    `three`/`four`/`five`/`six`-via plus `authored-via-chain` contracts, and
    the accepted same-layer orthogonal dogleg contract, and the accepted
    same-layer orthogonal two-bend contract, and
    the accepted zone-aware and zone-obstacle-aware existing-copper graph
    reuse contracts, and the accepted topology-aware plus layer-balance-aware
    topology-aware zone-obstacle-aware existing-copper graph reuse contracts,
    and the accepted obstacle-aware existing-copper graph reuse contract, and
    the policy-selected authored-copper graph family over the accepted bounded
    policy set
  - new slices must still avoid broad autorouting semantics, invented
    constraints, and untracked MCP drift
  - closure interpretation:
    - this frontier is the intended `M5` closure target under the narrowed
      routing-kernel milestone definition above
    - remaining placement-kernel work is explicitly out of scope for `M5`
      closure

## M6 Opening Charter

Status: [~] Frozen pending evidence
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m6_opening.md`
- Recommended M6 focus: read-only deterministic strategy reporting layered on
  top of the completed M5 routing-kernel substrate.
- Current M6 checkpoint chain:
  - `project route-strategy-report <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--objective <objective>]`
  - `project route-strategy-compare <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project route-strategy-delta <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project write-route-strategy-curated-fixture-suite --out-dir <path> [--manifest <path>]`
  - `project capture-route-strategy-curated-baseline --out-dir <path> [--manifest <path>] [--result <path>]`
  - `project route-strategy-batch-evaluate --requests <path>`
  - `project inspect-route-strategy-batch-result <path>`
  - `project validate-route-strategy-batch-result <path>`
  - `project compare-route-strategy-batch-result <before> <after>`
  - `project gate-route-strategy-batch-result <before> <after> [--policy <policy>]`
  - `project summarize-route-strategy-batch-results [--dir <path> | --artifact <path> ...] [--baseline <path> --policy <policy>]`
  - accepted objective set currently reuses the selector profile vocabulary:
    - `default`
    - `authored-copper-priority`
  - the report maps the requested objective to one existing selector profile,
    explains that mapping, and includes the current live selector outcome under
    that profile without reopening M5 routing semantics
  - the comparison report evaluates that same accepted objective/profile set,
    reports the current live selector outcome for each entry, and recommends
    one profile under a deterministic baseline-preserving comparison rule
  - the decision-delta report reduces that same accepted objective/profile set
    to one bounded explicit delta classification plus one short material
    difference summary for the user
  - the batch evaluation surface runs the existing report/compare/delta
    surfaces over a versioned explicit request manifest and returns both
    per-request evidence and aggregate summary counts
  - one curated fixture-suite writer now materializes a deterministic native
    fixture set plus a compatible batch-request manifest for repeated evidence
    runs over real persisted projects
  - one curated baseline-capture surface now materializes that fixture suite,
    runs the existing batch evaluator, and saves one reusable versioned
    batch-result artifact with deterministic default paths
  - one checked-in repo baseline asset set now lives under
    `crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1`
    and the alignment CI lane regenerates and strict-gates a fresh run against
    that baseline through `scripts/check_route_strategy_evidence.py`
  - the current curated suite covers:
    - same-outcome baseline route selection
    - profile divergence between `default` and
      `authored-copper-priority`
    - no-proposal-under-any-profile
    - one cross-layer routable same-outcome case
  - the batch-evaluate JSON output is now also the explicit saved result
    artifact format with `kind = native_route_strategy_batch_result_artifact`
    and `version = 1`
  - saved batch result artifacts can now be inspected and structurally
    validated through read-only surfaces that report summary/distribution
    details, per-request outcomes, malformed entries, version compatibility,
    required-field coverage, and deterministic summary/result integrity checks
  - saved batch result artifacts can now also be compared without live
    re-evaluation by `request_id`, reporting aggregate count deltas,
    added/removed/common request ids, common-request recommendation/delta/live
    outcome changes, and one bounded summary classification
  - saved batch result artifact comparisons can now also be evaluated as a
    read-only CI/review gate under the explicit accepted policy set
    `strict_identical|allow_aggregate_only|fail_on_recommendation_change`,
    reporting pass/fail reasons and count facts while returning CLI exit code
  - the current M6 implementation frontier is intentionally frozen pending
    repeated evidence from the checked-in baseline gate and curated fixture
    suite; new semantics are not the default next step

## M7 Opening Charter

Status: [~] Opened narrowly as a read-only route-proposal review layer
- Authority for the opening charter and entry criteria:
  `specs/progress/m7_opening.md`
- Concrete workspace/contract/spike definition:
  `specs/M7_FRONTEND_SPEC.md`
- Active imported-board fidelity execution plan:
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- Active board-review fidelity diagnosis:
  `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- Active M7 focus: one narrow visual review layer on top of the closed M5
  routing-kernel substrate and the frozen M6 strategy-reporting/evidence
  stack.
- Entry conditions satisfied before opening:
  - daemon result serialization no longer unwrap-panic on response encoding in
    `crates/engine-daemon/src/dispatch.rs`; serialization failures now return
    structured internal JSON-RPC errors instead of crashing the daemon
  - one remaining imported-connectivity runtime unwrap in
    `crates/engine/src/connectivity/mod.rs` was removed from the hierarchical
    link resolver path
  - `mcp-server/server_runtime.py` now generates the plain daemon-backed MCP
    request/call wrappers from one shared `DAEMON_CLIENT_METHOD_SPECS` table
    instead of duplicating those wrappers by hand across request builders and
    call helpers
  - `mcp-server/daemon_client_request_tests.py` now sanity-checks that the
    generated daemon client wrappers are installed and preserve required fixed
    parameter shapes such as `rotate_component`
  - governance/docs no longer treat `M5` as the active execution window; M5 is
    closed and M6 is frozen pending evidence
- Implemented opening slice:
  - new native CLI review surface:
    `project review-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - the same command also supports saved-artifact review with:
    `project review-route-proposal --artifact <path>`
  - the review payload reuses the existing selected route-proposal and route
    proposal artifact data structures, exposing deterministic proposal actions,
    segment evidence, selected contract metadata, and source identity for one
    frontend-consumable review object
  - the same review surface is exposed through MCP as
    `review_route_proposal`
  - apply/export remain in the existing machine-native CLI/MCP surfaces; M7 is
    review-only in this opening slice

- Next accepted frontend slice for `M7`:
  - one fixed route-proposal review workspace defined in
    `specs/M7_FRONTEND_SPEC.md`
  - one deterministic `board_review_scene_v1` contract for the opening board
    review context
  - one locked viewport-centered three-column shell with bottom dock strip
  - one fixed panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`
  - one single-selection read-only interaction model with a separate active
    review target
  - one fixed panel set plus integrated terminal and AI supporting lanes, all
    remaining subordinate to engine authority
  - one explicit authored/proposed/diagnostic visual-state model
  - one adopted interim frontend layout system: Taffy solves GUI shell/panel
    logical-pixel rectangles behind the retained `wgpu` renderer while engine
    and protocol crates remain layout-solver free

- Accepted post-spike correction track inside opening `M7`:
  - one bounded imported-board fidelity program defined in
    `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
  - scope split:
    - import fidelity for KiCad PCB truth preservation
    - scene-contract fidelity for explicit authored / unrouted / proposed /
      diagnostic lanes
    - renderer fidelity for PCB-native semantic readability
  - sequencing rule:
    - execute after the architecture spike proves the
      `gui-app` / `gui-protocol` / `gui-render` boundary
    - execute before broadening imported-board review claims or generalizing
      the opening `M7` workflow beyond the current bounded review surface
  - current correction-track read:
    - Stage 3 unrouted-lane work is functionally landed on the canonical
      half-routed fixture
    - previously recorded Stage 1 / Stage 2 blockers around outline ownership,
      pad rotation, roundrect semantics, outline layer-id carriage, and outline
      visibility gating have materially landed in code; the fidelity docs are
      now being reconciled to that newer repo state
    - `M7-INT-001` closed its first slice (2026-06-09): authored-object
      selection ownership, switch-clears-prior, and hover-preview-only
      behavior are regression-locked in
      `crates/gui-render/tests/selection_ownership.rs`
    - `M7-REN-006` closed (2026-06-09): the render-stack policy now has a
      single code encoding (`RenderStage` declaration order, shared
      `POST_COPPER_STAGES` walk), copper-layer appearance is constructed
      material-first, the bounded exception set is documented at the
      retained-geometry pass header, and contract regression tests lock the
      declared stage ladder
    - `M7-REN-003` closed (2026-06-12): proposed overlays verified
      copper-like at world-true width in selected and non-selected states;
      the diagnostic per-vertex marker violation ("generic path nodes" over
      proposed copper) was removed and regression-locked
    - `M7-REN-004` closed (2026-06-12): filled-zone copper is a declared
      derived shade of the layer material so pad/teardrop/pour boundaries
      read correctly (teardrop tangency criterion verified on DOA2526), and
      dim-unrelated readability is verified on the canonical fixture with
      regression locks in `render_contract_tests.rs`
    - active next implementation frontier is `M7-REG-001..003` fixture-backed
      import/scene/visual regression coverage (owner-ordered 2026-06-11)
    - the renderer semantic contract note now lives in
      `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  - standards amendment for the opening slice:
    - opening `M7` remains review-focused and does not expand into a full IPC
      authoring/validation milestone
    - standards-relevant imported observables already exposed in the review
      surface must preserve source truth
    - bounded import-audit diagnostics are in-scope where they report delta
      without mutating imported geometry

- Acceptance checks met for the opening slice:
  - deterministic repeated output on unchanged persisted state
  - no pool re-resolution or live import-session dependence
  - no invented routing geometry or inferred constraints beyond persisted facts
  - engine + CLI test coverage for sorted/deterministic extraction

- Acceptance checks required for the next frontend slice:
  - deterministic repeated `board_review_scene_v1` output on unchanged
    persisted state
  - stable authored and review companion identities on unchanged state
  - no frontend-owned semantic geometry or review inference beyond explicit
    machine contracts
  - explicit authored vs proposed vs diagnostic visual separation in the
    opening workspace
  - terminal and AI supporting lanes consume explicit selection/review context
    without becoming parallel design truth
  - the opening route review starts from the first proposal action in
    deterministic review order

- Acceptance checks required for the imported-board fidelity track:
  - supported KiCad PCB layer identities do not silently collapse to fallback
    copper layers
  - supported imported pads preserve the physical dimensions and shape
    semantics required for board review
  - unsupported imported-board cases fail explicitly or remain clearly bounded
    instead of silently producing materially wrong board meaning
  - the accepted fixture set can visually distinguish authored copper,
    unrouted connectivity, proposed overlay geometry, and board-context
    primitives reliably enough for a PCB user to trust the review
  - fixture-backed tests plus screenshot or image-based review cover the
    canonical half-routed board and the supporting imported-board edge cases

---

## Standards Audit Batch 1 — Spec Stubs Awaiting Implementation

Spec edits landed in commit `db98eff` (2026-04-17) per the apply order in
`docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`. This section tracks each stub
against its implementation status. Status semantics:
- `[x]` spec/doc text landed in the named anchor
- `[ ]` implementation work not started (no Rust type, no engine op, no
  pool storage, no MCP runtime entry, no importer/exporter, etc.)
- A row may carry both: `[x]` spec landed + `[ ]` implementation pending —
  shown as two columns

### Pass 0 — Standards Compliance Disposition Refresh

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Domain 1 disposition refresh (STEP/IDF/IDX/EDMD/DXF prerequisites; Gerber X3 / IPC-2581C / IPC-D-356 / ODB++ contracts) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.1 | [x] | [—] N/A (disposition text only) |
| Domain 2 disposition refresh (IBIS/Touchstone/SPICE attachment; encrypted-content policy) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.2 | [x] | [—] N/A (disposition text only) |

### Pass 1 — Engine Schema Bedrock

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelAttachment`, `ModelProvenance`, `ModelFormatMetadata` (D2-1) | `specs/ENGINE_SPEC.md` §1.1a | [x] | [~] (engine schema is model-backed and validated through typed part behavioural-model authoring; attach command extracts file hash/provenance and promotes source files into `pool/models`; detach removes attachment edges; richer file parsing remains pending) |
| `ModelFormat` enum, typed `Transform3D`, expanded `ModelRef`, `Package.body_height_nm` / `body_height_mounted_nm` (D1-2) | `specs/ENGINE_SPEC.md` §1.1a + §1.2 | [x] | [~] (`ModelFormat`, deterministic typed `Transform3D`, expanded `ModelRef`, and package body-height fields are model-backed; `add-pool-package-model-3d` and `set-pool-package-body-heights` journal typed authoring; model provenance remains an empty slot and richer 3D import/export use remains pending) |
| `StackupLayer` material fields (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`) (D1-3) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number material fields, legacy board JSON deserializes with unset material metadata, and `set-board-stackup` plus `datum.pcb.set_stackup` accept backward-compatible material-aware layer tuples; richer KiCad stackup-material extraction and impedance-solving use remain pending) |
| `Net.controlled_impedance: Option<ImpedanceSpec>` and `ImpedanceSpec` (D1-4) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number impedance target/tolerance fields, legacy board-net JSON deserializes with no impedance target, and `place-board-net` / `edit-board-net` plus canonical MCP `datum.pcb.place_net` / `edit_net` can author, update, and clear per-net controlled-impedance metadata; impedance solving/export use remains pending) |
| `Part` extensions (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, `last_supply_chain_check`) plus `ThermalSpec`, `PackagingKind`, `PackagingOption`, `SupplyOffer` (D2-2) | `specs/ENGINE_SPEC.md` §1.2 | [x] | [~] (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, and `last_supply_chain_check` are model-backed and journal-authorable through typed part commands; timestamp is currently serialized as a string rather than a chrono `DateTime<Utc>`) |
| `AttachModel` / `DetachModel` operations with `inverse()` reversibility (D2-4) | `specs/ENGINE_SPEC.md` §3 | [x] | [~] (`attach-pool-part-model` promotes a content-addressed model file and journals typed `AttachPoolPartModel` operations over the part attachment list; `detach-pool-part-model` journals typed `DetachPoolPartModel` operations by attachment/model UUID; undo/redo restores exact attachment arrays while retaining shared content-addressed blobs; richer blob lifecycle policy remains pending) |

### Pass 2 — Pool and Native Persistence

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `pool/models/{ibis,spice,touchstone,ami,thermal}/` directory; `models` and `part_model_attachments` SQL index tables (D2-3) | `docs/POOL_ARCHITECTURE.md` §2 | [x] | [~] (`attach-pool-part-model` writes content-addressed model files under `pool/models/<role>/`; `project query pool-models`, flat MCP `get_pool_model_blobs`, and canonical `datum.library.pool_models` enumerate blobs, recompute SHA-256, derive deterministic model UUIDs, report part attachment references, and expose `referenced` / `orphaned` lifecycle flags; `project gc-pool-models` provides dry-run/apply cleanup for orphaned regular hash-matching blobs; `project validate` now fails missing provenance-backed blobs, filename/hash mismatches, and bad deterministic model UUIDs; SQL/index tables, richer GC policy, and bundle handling remain pending) |
| `pool/models/` in native project layout; new "Pool Model Files" schema (D2-9) | `specs/NATIVE_FORMAT_SPEC.md` §4 + §6.x | [x] | [~] (native projects can materialize, query, validate, and conservatively garbage-collect orphaned regular `pool/models/<role>/<sha256>.<ext>` blobs via attach/query/validate/gc commands with hash and attachment-integrity verification; migration, AMI bundle hashing, richer orphan lifecycle policy, and full bundle schema remain pending) |

### Pass 3 — MCP API Stubs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| M7+ Export Tools: `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline` (D1-5) | `specs/MCP_API_SPEC.md` "M7+ Export Tools" | [x] | [ ] (not in `tools_catalog_data.py`; would need exporter implementations) |
| Component Modelling Tools: `attach_ibis`, `attach_touchstone`, `attach_spice`, `validate_*`, `extract_*`, `export_spice_netlist`, `lookup_part_*`, `refresh_supply_chain`, `find_alternate_parts`, `query_packaging_options`, `normalize_manufacturer`, `infer_diffpair_from_pinnames` (D2-5) | `specs/MCP_API_SPEC.md` "Component Modelling Tools (M7+)" | [x] | [ ] (not in `tools_catalog_data.py`) |
| Encrypted Content Handling Policy (D2-6) | `specs/MCP_API_SPEC.md` top-level | [x] | [—] N/A (policy framing) |

### Pass 4 — Import Spec

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| IPC-2581 Import (Future — Post-M7) rationale + feature matrix (D1-7) | `specs/IMPORT_SPEC.md` §5 | [x] | [ ] (importer deferred) |
| KiCad and Eagle import matrices: SPICE/IBIS/Touchstone rows promoted Deferred → Best-effort (M7+) with `Part.behavioural_models` mapping note (D2-8) | `specs/IMPORT_SPEC.md` §3 + §4 | [x] | [ ] (importer deferred) |

### Pass 5 — Architecture and Scope Docs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Behavioural model attachment subsection (D2-7) | `docs/LIBRARY_ARCHITECTURE.md` | [x] | [—] N/A (architecture text) |
| Interop scope re-organisation — Hard/Should/On-demand/Out-of-scope per Domain 1 (D1-1) | `docs/INTEROP_SCOPE.md` §Future (M5+) | [x] | [—] N/A (scope text) |
| Behavioural model attachment & export scope buckets (D2-10) | `docs/INTEROP_SCOPE.md` new section | [x] | [—] N/A (scope text) |

### Deferred — Held For A Later Batch

| # | Spec target | Why deferred |
|---|-------------|--------------|
| D1-6 | `specs/NATIVE_FORMAT_SPEC.md` §12 or `docs/POOL_ARCHITECTURE.md` | `.gitignore` / `.gitattributes` conventions — descriptive, no contract impact |
| D1-8 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` §10 | "Datum's Open-Stack Position" appendix — marketing position |
| D2-11 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` | "Behavioural Model Stack — Open-Stack Position" appendix — marketing position |

**Standards Audit Batch 1 overall**: [x] spec/doc/policy stubs landed; [~]
implementation is mixed. Model schema, part metadata, stackup materials,
controlled-impedance metadata, package/body/model geometry, part model
attach/detach, and first pool-model blob query/verification are partial.
SQL indexes, AMI bundle handling, blob lifecycle GC, migration, richer parsing,
MCP query aliases, and importer/exporter handlers remain pending.

---

## ENGINE_SPEC.md — Core Types

### §1.1 Geometry Primitives

| Type | Status |
|------|--------|
| Point | [x] |
| Rect | [x] |
| Polygon | [x] |
| Arc | [x] |
| LayerId | [x] |

### §1.1a Shared Enums

| Type | Status | Notes |
|------|--------|-------|
| PinDirection (10 variants) | [x] | In pool/mod.rs as PinElectricalType on SymbolPin |
| Lifecycle | [x] | In pool/mod.rs |
| StackupLayerType | [x] | In board/mod.rs |
| PortDirection | [x] | In schematic/mod.rs |
| Primitive | [x] | In pool/mod.rs (Symbol graphics) |

### §1.2 Pool Types

| Type | Status |
|------|--------|
| Pin | [x] |
| Unit | [x] |
| Gate | [x] |
| Entity | [x] |
| Pad | [x] |
| Package | [x] |
| PadMapEntry | [x] |
| Part | [x] |

### §1.3 Board Types

| Type | Status |
|------|--------|
| Board | [x] |
| PlacedPackage | [x] |
| Track | [x] |
| Via | [x] |
| Zone | [x] |
| Net | [x] |
| NetClass | [x] |
| Stackup / StackupLayer | [x] |
| Keepout | [x] |
| Dimension | [x] |
| BoardText | [x] |

### §1.4 Schematic Types

| Type | Status |
|------|--------|
| Schematic | [x] |
| Sheet | [x] |
| PlacedSymbol | [x] |
| SheetDefinition | [x] |
| SheetInstance | [x] |
| SchematicWire | [x] |
| Junction | [x] |
| NetLabel / LabelKind | [x] |
| Bus | [x] |
| BusEntry | [x] |
| HierarchicalPort | [x] |
| NoConnectMarker | [x] |
| Variant | [x] |

### §1.5 Rule Types

| Type | Status |
|------|--------|
| Rule | [x] |
| RuleType (7 variants) | [x] |
| RuleParams (7 variants) | [x] |
| RuleScope (leaf nodes) | [x] |
| RuleScope (combinators) | [x] parse, [—] eval deferred to M6 |

### §2 Object Invariants

| Invariant | Status | Notes |
|-----------|--------|-------|
| All authored objects have non-nil UUID | [x] | Enforced by constructors |
| No duplicate UUIDs within type | [x] | `project validate` checks persisted UUID-key consistency and duplicate authored UUIDs across native project object types |
| Non-dangling references | [x] | `project validate` checks required native files plus persisted schematic/board cross-file references and board-side object references |
| Integer coordinates (no float) | [x] | i64 throughout |
| Connectivity recomputed from authored data | [x] | |

### §3 Operations

| Item | Status |
|------|--------|
| Operation trait | [ ] |
| OpDiff | [ ] |
| Transaction | [ ] |
| Undo/redo semantics | [ ] |
| Derived data invalidation | [ ] |

### §5 Engine API

| Method | Status | Notes |
|--------|--------|-------|
| new() | [x] | |
| has_open_project() | [x] | |
| import() | [x] | KiCad skeleton + Eagle .lbr |
| save() | [~] | Writes unmodified imported KiCad boards byte-identically, persists current `delete_track`/`delete_via`/`delete_component` removals, rewrites current `move_component`/`rotate_component` footprint placement and component `Value`/`Reference` properties, rewrites package-backed footprint bodies for current `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` slices, and writes rule/part-assignment/package-assignment/net-class sidecars for current `set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` slice |
| save_to_original() | [~] | Current M3 helper for imported-design write-back to original file |
| search_pool() | [x] | |
| import_eagle_library() | [x] | |
| get_board_summary() | [x] | |
| get_components() | [x] | |
| get_package_change_candidates() | [x] | Current engine API supports component-scoped package compatibility introspection |
| get_part_change_candidates() | [x] | Current engine API supports component-scoped part compatibility introspection |
| get_component_replacement_plan() | [x] | Current engine API supports unified component replacement planning introspection |
| get_scoped_component_replacement_plan() | [x] | Current engine API supports scoped policy-driven replacement preview introspection |
| edit_scoped_component_replacement_plan() | [x] | Current engine API supports scoped replacement preview post-processing via exclusions and explicit compatible target overrides |
| replace_components() | [x] | Current M3 board write slice supports batched explicit component replacement as one transaction / one undo step |
| apply_component_replacement_plan() | [x] | Current M3 board write slice resolves package/part selections from the unified replacement plan and applies them as one transaction |
| apply_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies from the unified replacement plan and applies them as one transaction |
| apply_scoped_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies over a scoped component filter and applies them as one transaction |
| apply_scoped_component_replacement_plan() | [x] | Current M3 board write slice validates and applies a previously previewed scoped replacement plan without re-resolving policy |
| get_net_info() | [x] | Returns all nets, not single-net |
| get_stackup() | [x] | |
| get_unrouted() | [x] | |
| get_schematic_summary() | [x] | |
| get_sheets() | [x] | |
| get_symbols() | [x] | |
| get_ports() | [x] | |
| get_labels() | [x] | |
| get_buses() | [x] | |
| get_noconnects() | [x] | |
| get_hierarchy() | [x] | |
| get_schematic_net_info() | [x] | Returns all nets, not single-net |
| get_connectivity_diagnostics() | [x] | |
| get_check_report() | [x] | |
| delete_component() | [x] | Current M3 board write slice |
| delete_track() | [x] | Current M3 board write slice |
| delete_via() | [x] | Current M3 board write slice |
| move_component() | [x] | Current M3 board write slice |
| rotate_component() | [x] | Current M3 board write slice |
| set_value() | [x] | Current M3 board write slice |
| set_reference() | [x] | Current M3 board write slice |
| assign_part() | [x] | Current M3 board write slice |
| set_package() | [x] | Current M3 board write slice |
| set_package_with_part() | [x] | Current M3 board write slice |
| set_net_class() | [x] | Current M3 board write slice |
| set_design_rule() | [x] | Current M3 board write slice |
| run_erc_prechecks() | [x] | |
| run_drc() | [x] | Engine API method implemented (connectivity + clearance checks currently) |
| execute() / execute_batch() | [ ] | |
| undo() / redo() | [~] | Current engine API supports transaction reversal for `delete_component`, `delete_track`, `delete_via`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, and `set_design_rule` |

---

## CHECKING_ARCHITECTURE_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| CheckDomain enum (ERC, DRC) | [x] | Implemented as check report domain field |
| CheckSeverity (Error, Warning, Info) | [x] | ErcSeverity in erc/mod.rs |
| CheckSummary | [x] | In api/mod.rs |
| CheckWaiver (domain, target, rationale) | [x] | In schematic/mod.rs |
| WaiverTarget variants | [x] | Object, RuleObject, and RuleObjects matching exercised across ERC/DRC tests |
| Waiver matching in ERC | [x] | Includes authored native-project waivers in existing project ERC/check flows |
| Waiver matching in DRC | [x] | `run_drc` now applies authored DRC waivers and keeps waived findings visible |
| Cross-domain checks excluded from M2 | [x] | Not implemented (correct) |

---

## ERC_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| PinElectricalType enum (10 variants) | [x] | In schematic/mod.rs SymbolPin |
| NetSemanticClass enum | [~] | Partial; power/signal inference exists |
| M2 rule: output_to_output_conflict | [x] | |
| M2 rule: undriven_input | [x] | |
| M2 rule: power_without_source | [x] | |
| M2 rule: noconnect_connected | [x] | |
| M2 rule: unconnected_required_pin | [x] | |
| M2 rule: passive_only_net | [~] | No distinct rule/code ships; passive pins only modulate `input_without_explicit_driver` severity (verified 0 grep hits in `erc/` 2026-06-22). Standalone check is target work. |
| M2 rule: hierarchical_connectivity_mismatch | [x] | Sheet-level hierarchical label/port mismatch check implemented |
| Compatibility matrix (pin-pair) | [~] | Driving analysis exists; full matrix not formalized |
| ErcReport struct | [x] | Via CheckReport |
| ErcViolation struct | [x] | ErcFinding |
| Waiver integration | [x] | |
| Configurable severity | [x] | ErcConfig with BTreeMap overrides |

---

## IMPORT_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| NAMESPACE_KICAD UUID | [x] | ir/ids.rs |
| NAMESPACE_EAGLE UUID | [x] | ir/ids.rs |
| import_uuid() function | [x] | |
| KiCad object path construction | [~] | Skeleton parser; not all object types mapped |
| Eagle object path construction | [x] | Full library mapping |
| .ids.json schema | [x] | import/ids_sidecar.rs |
| .ids.json precedence rules (3 cases) | [x] | restore_or_merge_mappings() |
| KiCad board feature matrix (Required items) | [~] | Skeleton only; geometry not fully parsed |
| KiCad schematic feature matrix (Required items) | [~] | Skeleton only |
| Eagle board feature matrix | [ ] | Design import not implemented |
| Eagle schematic feature matrix | [ ] | Design import not implemented |
| Eagle library feature matrix | [x] | Symbols, packages, devicesets |

---

## SCHEMATIC_CONNECTIVITY_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| Sheet-local wire connectivity | [x] | Union-find in connectivity/mod.rs |
| Junction semantics | [x] | |
| Local labels | [x] | |
| Global labels | [x] | |
| Hierarchical labels/ports | [~] | Basic support; full hierarchy links partial |
| Power symbols | [x] | |
| Bus/member expansion | [x] | Basic imported KiCad subset now supports deterministic `NAME[n]` scalar member normalization, `NAME[a..b]` bus-range expansion, and geometric bus-entry association; advanced syntax edge cases remain deferred |
| Net naming and identity | [x] | |
| Connectivity diagnostics | [x] | |
| Deterministic graph output | [x] | Sorted by UUID |

---

## SCHEMATIC_EDITOR_SPEC.md — M4 Operations

Status: [x] Closed for scoped M4 slice
- Verified by code audit in `crates/cli/src/cli_args.rs`,
  `crates/cli/src/command_exec.rs`, and dedicated CLI test shards:
  `main_tests_project_symbol*.rs`, `main_tests_project_label.rs`,
  `main_tests_project_wire.rs`, `main_tests_project_junction.rs`,
  `main_tests_project_port.rs`, `main_tests_project_bus.rs`,
  and `main_tests_project_noconnect.rs`.
- Implemented schematic operation families include symbol
  place/move/rotate/mirror/delete, symbol field add/edit/delete,
  label place/rename/delete, wire draw/delete, junction place/delete,
  hierarchical port place/edit/delete, bus create/edit/place-entry/delete-entry,
  and no-connect place/delete.
- Remaining editor expansion items such as power-symbol placement,
  sheet-instance lifecycle operations, and deterministic annotate flow
  completion are deferred beyond scoped M4 closure.

---

## NATIVE_FORMAT_SPEC.md

Status: [x] Closed for scoped M4 slice
- Native project scaffold, deterministic file layout, and first native
  read/query/check surfaces are implemented in the current M4 slice.
- Remaining native-format contract areas (full schema coverage, migration
  completeness, and richer manufacturing/output semantics remain open beyond
  scoped M4 closure.

---

## Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Rust workspace (8 crates) | [x] | engine, cli, engine-daemon, test-harness, gui-protocol, gui-render, gui-app, verb-registry (locked via `specs/SPEC_PARITY.md` → `workspace_crates`) |
| Engine compiles without GUI deps | [x] | |
| Test harness (golden file utilities) | [x] | test-harness crate |
| Test corpus (real designs) | [ ] | tests/corpus/ empty |
| Daemon JSON-RPC dispatch | [x] | 39 methods in `dispatch.rs`, with the 11 retired imported-board writer arms removed (Retired tombstones point at datum.pcb.*/datum.proposal.*) and the 4 terminally frozen imported-session writers fenced as hidden compatibility, with coverage in daemon tests |
| Daemon socket transport | [x] | `main()` parses `--socket` and serves Unix socket; live smoke is environment-gated because sandboxed local runs deny socket IPC |
| MCP Python server (tool host) | [x] | Tool definitions + stdio dispatch |
| MCP→daemon transport | [x] | `EngineDaemonClient.call()` uses Unix socket JSON-RPC; behavioral parity remains covered separately from live socket smoke |
| Git repository initialized | [x] | `main` branch with GitHub remote configured |
| CI pipeline | [x] | `.github/workflows/alignment.yml` runs alignment and file-size budget checks |
