# Editor Action Feedback & Command-Line Interaction — Research

**Status:** research artifact (working evidence, not doctrine). Feeds an owner
decision packet on the action-feedback / Command Console question. This document
ratifies nothing; it classifies the problem, records external evidence, and
narrows the candidate dispositions the owner must choose between.

**Companion visual study:** `docs/gui/prototypes/command-feedback-study.html`
(comparative prototypes; owner-review reference, not an accepted design).

---

## 1. Why this study exists — the inherited assumption

The question "where does editor action feedback go?" has never been studied on
its own. It has only ever been answered *by inheritance*:

- The terminal-redesign research (`research/terminal-redesign/
  TERMINAL_CURRENT_STATE_MAP.md`) correctly identified the real need — "Codex
  trace spam + engine messages need a home that isn't the shell screen" — and
  then **assumed the answer**: "the answer is the Command Console / console
  sink, not a new dock tab" (lines 176–186). That research was about the
  terminal; it validated the *need* but never studied the *surface*.
- `ConsoleLaneState`'s own doc comment encodes the same assumption: "The
  correct visible home for these echoes is the editor command console, which
  does not exist yet."
- Commit `419bcf6` (terminal production acceptance) then wrote CONSOLE-RO-01..05
  into `DATUM_GUI_DESIGN_SPEC.md` — turning the assumption into build-ready
  acceptance criteria for a visible console, still without any dedicated study
  of editor action feedback. (The commit changed no routing code; it changed
  only the plan.)

So the lane's contents were shaped by *where messages could be parked without
violating the PTY write fence*, not by *where a user needs to see them*.
"Already goes into `ConsoleLaneState`" is provenance, not evidence of belonging.

This document is the missing study.

## 2. What the code actually contains (inventory summary)

<!-- EVIDENCE:CONSOLE-CONTRACT-RECOVERY:CONSOLE-C01-INVENTORY -->

Full producer-by-producer inventory was taken 2026-08-22 (105 call sites of
`Runtime::log_review_event`, the single production route into the sink via
`terminal_narration::route_gui_narration` → `ConsoleLaneState::push_line`).

**The sink itself** (`crates/gui-protocol/src/workspace_layout.rs:95–118`):
`Vec<String>`, 240-line FIFO cap, no severity, no timestamp, no source, no
category, no dedupe/coalescing. Write-only today — `gui-render` never reads it.
Narration deliberately does not trigger a repaint (`main.rs:1626–1631`), a live
hazard the moment the lane is rendered.

**Producer distribution by narrated event class** (approximate, of 105 sites):

| Class | ~Sites | Examples |
|---|---|---|
| Terminal-subsystem errors/status | 55 | `"terminal session open failed: {err}"`, `"clipboard copy failed"`, `"terminal restart requested…"`, IME/mouse/focus encoding failures |
| View / filter / layer echoes | 20 | `"menu view.fit"`, `"fit board"`, `"layer {id} hidden"`, `"dim unrelated on"` |
| Tool change + tool gating refusals | 9 | `"tool SELECT"`, `"place text requires project backing"`, `"no board text selected"` |
| Authoring-handoff narration | 10 | `"queued authoring command {label}"`, `"editing selected board text height"` |
| Selection echoes | 6 | `"selected authored object {object_id}"`, `"selected check finding {fp}"` |
| Production-artifact events | 5 | `"focused production artifact {id}"`, `"ran production output command {cmd}"` |

**Observations that must inform the design:**

1. **Over half the lane is not editor action feedback at all** — it is
   terminal-subsystem plumbing status. A visible "console" that faithfully
   renders today's lane would be dominated by terminal lifecycle noise.
2. **Refused actions are currently invisible.** Hard user-facing failures
   ("no board text selected", "place text requires project backing", "move
   requires a selected component target") land in a lane with no renderer. The
   user gets *zero* feedback for a refused action unless it also happens to
   write `ui.terminal.status`.
3. **Two parallel, inconsistent channels already exist.** Three sites mirror
   errors into both `terminal.status` (visible, tab strip) and the console lane
   (invisible); everything else picks one channel arbitrarily.
4. **Messages are untyped strings and chrome behavior is prefix-parsed** —
   `terminal_session_chrome.rs` derives actionable buttons by
   `starts_with`/`contains` on status text. Any severity/filter/announce design
   requires a typed record (exactly what `dat-notification-system-ztp`'s
   `Notice` sketch proposes).
5. **Vocabulary is inconsistent**: internal ids (`"menu view.fit"`,
   `"menu view.preset_board_schematic"`) vs prose (`"fit board"`); raw
   `ObjectId`s in some echoes, truncated suffixes elsewhere. Not
   presentation-ready.
6. **Diagnostics have no list surface.** DRC findings exist as a status-bar
   count and a per-selected-finding inspector card; nothing produces
   `HitTarget::CheckFinding` hit regions, so findings are effectively
   unreachable by mouse. The console lane is *not* the fix for this; a findings
   surface is its own gap.

## 3. The message taxonomy

Nine distinct feedback kinds, with Datum examples and their consequence class.
The single most important finding of the external survey (§4) is that **mature
tools give these different homes** — collapsing them into one scrolling lane is
the anti-pattern this study exists to prevent.

| # | Kind | Datum examples (from inventory) | Consequence if missed |
|---|---|---|---|
| 1 | **Immediate verb acknowledgement** | `"tool SELECT"`, `"fit board"`, `"layer F.Cu hidden"`, selection echoes | None — the canvas already shows the result; echo is confirmation + learnability |
| 2 | **Contextual prompt during a multi-step tool** | (future: route/place/wire step prompts, valid-key hints; today only implicit) | User stalls mid-tool; discoverability lost |
| 3 | **Transient success** | `"terminal text copied"`, `"confirmed terminal clipboard write"`, save/export confirmations | Mild doubt; recoverable by checking state |
| 4 | **Recoverable failure (refused action)** | `"no board text selected"`, `"place text requires project backing"`, `"move requires clicking a component first"` | User repeats the action confused — *the current worst gap* |
| 5 | **Persistent failure / degraded state** | `"terminal core start failed"`, daemon unreachable, `"shutdown blocked by terminal teardown"` | Work blocked without explanation |
| 6 | **Background progress** | status refresh, future check runs / CAM jobs / commits; terminal OSC 9;4 progress | Perceived hang; premature cancel |
| 7 | **Structured diagnostics** | ERC/DRC findings (severity, rule, target, cross-probe) | Design errors shipped; needs list + markers + exclusions, not prose lines |
| 8 | **History / audit** | what happened this session, in order; the journal already *is* the authoritative audit of committed ops | Can't reconstruct "what just happened"; agents/users lose shared context |
| 9 | **Interactive typed command** | the doctrine Command Console: type a verb, act on hover target ("typed twin of the marking menu") | A missing *input* modality, not feedback at all |

Kinds 1–8 are **output**; kind 9 is **input**. The conflation under repair is
exactly this: the spec's Command Console is an *input* surface (9) whose
passive state was drafted as the home for *all output* (1–8) because the sink
already existed.

## 4. External evidence

<!-- EVIDENCE:CONSOLE-CONTRACT-RECOVERY:CONSOLE-C02-RESEARCH -->

### 4.1 AutoCAD — the canonical editor command line

- **Anatomy:** single input row (command icon + active command + prompt with
  clickable bracketed options) + prompt-history above it. Docked (full-width,
  classically 3 lines) or floating (bottom-left, width-resizable, 0–50
  semi-transparent temporary history lines, `CLIPROMPTLINES` default 3). F2
  expands full scrollable session history in place; Ctrl+9 hides the window
  entirely. History/active-prompt/temporary-prompt/option are separately
  color-classified elements.
- **Single command stream:** every ribbon/toolbar button is a command macro
  (`^C^C_line`) *echoed through the same command line*; GUI clicks, typed
  input, scripts, and LISP reduce to one echoed transcript. Trainers teach
  "press F2 to see what happened." This is the audit/learnability property —
  identical in spirit to Datum's "five doorways, one vocabulary" law.
- **Coexisting preference, not chrome:** Dynamic Input (2006, `DYNMODE`) puts
  the same prompts/coordinates in an at-cursor tooltip; both surfaces stay
  live; some users hide the command window entirely, some disable Dynamic
  Input. The lesson: heads-up (at-cursor) and docked (line) presentation of
  the *same* prompt stream are user preferences over one model.
- **Prompt grammar:** per-step prompts (`Specify first point:`), options in
  brackets with capitalized accelerators, defaults in angle brackets, `>>` for
  nested transparent commands. Inquiry commands print into history and
  auto-open the expanded view when output exceeds the line.
- Modern input assist: autocomplete, autocorrect (learned), synonyms,
  mid-string search, adaptive frequency ordering — all per-feature toggleable.

### 4.2 EAGLE / Fusion Electronics — commands as the substrate

- Single-line command box in the editor toolbar; **every icon is a text
  command**; abbreviations (`MO` = `MOVE`); the status bar under the canvas
  shows the active command's short prompt.
- The grammar formalizes the mouse: `MOVE * *` — a click *is* a command
  argument. Scripts (`.scr`) are replayed command streams; ULPs compute and
  then *emit command strings* rather than mutating privately. Structurally
  identical to Datum's typed-operation/single-commit doctrine.
- The command line **survived the Fusion 360 integration** (Electronics
  workspace command-line reference; `.scr` still executes) — evidence that a
  professional EDA user base treats it as load-bearing, not legacy.

### 4.3 Altium Designer — HUD + Messages panel

- **Board Insight HUD:** translucent overlay *inside the viewport* (cursor,
  delta, layer, snap; hover mode expands to object + rule-violation detail).
  `Shift+H` toggle. In-command mode changes echo to **status bar + HUD
  simultaneously**. `Shift+F1` mid-command lists the currently valid shortcuts.
- **Messages panel:** application-global, persistent, severity-typed
  (Info/Warning/Error/Fatal by source), cross-probes on double-click (zoom +
  fade-all-but-offender), remediation from the diagnostic (place NoERC).
  Diagnostics are a *list you work through*, never a scrolling prose lane.

### 4.4 KiCad

- Status bar: cursor/zoom/relative-origin/grid/units zones; message panel shows
  transient selected-object info (no history — the outlier design, and a known
  weakness). DRC: modal dialog list + canvas markers; severity per-rule
  (error/warn/ignore) and per-violation exclusions are first-class persisted
  state. KiCad 8 added a notification bell (application-level popover).

### 4.5 Horizon EDA — the tool tip bar

- While a tool is active, a bar at the bottom of the canvas enumerates the
  tool's *live keymap* (`LMB: place arc center · RMB: cancel · e: flip arc`),
  pushed by the tool itself; it exists only during the tool and is the tool's
  only standing feedback channel. One interaction framework shared by all five
  editors — a shared backbone configured per editor, i.e. Datum's own law
  independently converged on.

### 4.6 Blender — the strongest cross-domain model

- **Status bar, three zones:** left = live keymap for the active/modal tool;
  middle = running-task progress (with cancel) + **Report messages**
  (color-coded, fade after seconds, click to open full log); right = stats.
- **Info editor:** persistent log where **every action is echoed as its Python
  call** — selectable, copyable, script-buildable. The durable archive behind
  the ephemeral report. This is *journal-as-UI*: the operator stream doubles as
  learning and automation surface. Datum already has the real thing — a typed,
  provenance-carrying journal — so an echo lane can be a *projection of the
  journal/operation stream*, not a parallel hand-written string log.
- **Adjust Last Operation:** post-hoc parameter panel (F9) that re-runs the
  operator — acknowledgement that *the richest feedback for a verb is an
  editable record of the verb itself*.

### 4.7 Notification systems (VS Code / JetBrains / GNOME / KDE)

- **VS Code three-tier:** toasts (bottom-right, max 3, auto-hide *unless they
  carry actions*) → status-bar bell + unread count → persistent notification
  center. Severity Info/Warning/Error; DND suppresses info/warn but never
  errors; background progress belongs in the status bar, "progress in a
  notification is a last resort." No per-message timeout API — the platform
  owns dismissal policy.
- **JetBrains:** surface chosen by required action — modal / inline banner /
  balloon; balloons are **sticky by default** and *timed (10 s)* only for
  miss-safe informational results; everything lands in the Event Log.
- **GNOME/KDE:** toasts for *events*, banners for *states*; one toast at a
  time; libadwaita default 5 s, timeout 0 for anything with a button;
  freedesktop urgency spec: critical never auto-expires.
- **Convergent rules:** (a) errors and anything actionable persist; only
  miss-safe info auto-dismisses (5–10 s); (b) **every transient message has a
  persistent home** — a toast is a *preview of a durable record*, so a missed
  toast is recoverable, never lost; (c) status bar = ambient channel (progress,
  unread count, ongoing state); (d) ongoing states use banners, events use
  toasts.

### 4.8 Accessibility (hard requirements for a wgpu-native shell)

- **WCAG 4.1.3 (AA):** every visually-presented status change must be
  programmatically announced without stealing focus. Native Linux path:
  AT-SPI announcement events (GTK 4.14 `gtk_accessible_announce`; AccessKit
  exposes the same on Linux). A custom-rendered shell must emit these
  explicitly — announcements are events, never inferred from pixels.
- Priority mapping: polite/MEDIUM for routine echoes, assertive/HIGH only for
  critical errors.
- **WCAG 2.2.1:** auto-dismiss is a time limit — users need a setting to
  extend/disable it, hover must pause it, and the persistent log is the
  information-is-never-lost escape hatch. Rule of thumb ≥ 3 s + 1 s per 3
  words.
- **Toast pitfalls (Scott O'Hara):** live announcements don't convey
  interactivity; never put functionality or critical info *only* in a toast;
  auto-dismissing toasts with action buttons are the worst combination.

## 5. Convergent findings

1. **Two-tier feedback is the field consensus.** An ephemeral, spatially-close
   channel (HUD / tip bar / feedback line / toast) backed by a persistent,
   inspectable history (Messages panel / Info editor / notification center).
   Ephemeral-only (KiCad's message panel) and persistent-only designs are the
   outliers, and are criticized as such.
2. **In-tool prompting is its own surface**, not a history entry: Horizon's tip
   bar, Blender's keymap zone, Altium's HUD + Shift+F1, AutoCAD's per-step
   prompt row. It exists only while a tool is active, shows *what the user can
   do next*, and never scrolls.
3. **The echoed command stream is a feature, not noise** — when it is *one*
   canonical vocabulary. AutoCAD/EAGLE echo every GUI action as its command;
   Blender echoes every action as executable Python. The value is learnability
   ("what command was that button?"), audit ("F2 shows what happened"), and
   scripting-by-copy. Datum's structural advantage: the typed operation journal
   already is this stream for committed mutations; consumer-state actions
   (view/tool/selection) would need a deliberate, *canonical-vocabulary* echo —
   which the current lane's inconsistent strings are not.
4. **Structured diagnostics never live in the prose lane.** Every tool with
   real check systems (Altium, KiCad) gives findings a severity-typed list with
   cross-probe and exclusions. Datum's DRC gap (no findings list, findings
   unreachable by mouse) is a separate surface to build, not console content.
5. **Terminal/plumbing status is not editor feedback.** No surveyed tool routes
   its subsystem lifecycle noise into the editor's command history. The ~55
   terminal-status producers belong to terminal chrome (where some already
   render) and, for severe/persistent cases, to the notification tier.
6. **The command line, where it exists, is a coexisting preference** —
   hideable, repositionable, with heads-up alternatives fed by the same model.
   No surveyed tool *requires* it; every command-driven tool keeps it because
   its users demand it.
7. **Severity is typed and first-class** end to end (message record → surface
   choice → announcement priority → persistence policy). A `Vec<String>` cannot
   carry this design.
8. **Accessibility forces the same architecture:** a typed message record with
   severity and an explicit announcement event, plus a persistent surface, is
   the *minimum* compliant design — not a nice-to-have.

## 6. Consequence classification → proper destination

The mapping the plan asked for. Destinations reference the candidate surfaces
of the visual study (`command-feedback-study.html`): **feedback line** (the
passive echo row, whatever its final form), **history** (its expandable
scrollback), **tool prompt** (in-tool hint surface), **toast/banner/log** (the
`dat-notification-system-ztp` notice tiers), **status bar** (ambient zone),
**findings surface** (future diagnostics list), **terminal chrome** (tab
strip/session chrome — exists).

| Producer class (from §2) | Kind (§3) | Proper destination | Notes |
|---|---|---|---|
| Tool change echoes (`"tool SELECT"`) | 1 | feedback line + history | Canonical verb vocabulary, not internal ids |
| View/filter/layer echoes (`"menu view.fit"`, `"layer … hidden"`) | 1 | feedback line + history | Normalize wording first; never toasts |
| Selection echoes | 1 | feedback line + history; detail already in Inspector | Status bar `Sel` zone stays the ambient readout |
| Tool gating refusals (`"no board text selected"`) | 4 | feedback line, error-styled + polite announcement; persists until next action | Must not be miss-able; today invisible |
| Multi-step tool prompts (future) | 2 | tool prompt surface (Horizon/Blender pattern) | Distinct from history; never scrolls |
| Transient success (copy/save confirmations) | 3 | feedback line (auto-fade) + history; toast only if out-of-view consequence | 5–10 s, hover-pauses, user-adjustable |
| Terminal lifecycle status/errors (~55 sites) | 5/3 | terminal chrome (exists); severe/persistent → banner + notices log | **Remove from editor feedback lane** |
| Persistent failure / degraded state (daemon down, teardown blocked) | 5 | sticky banner (state, auto-clears on resolution) + notices log | Never auto-dismiss; never toast-only |
| Background progress (refresh, checks, CAM, commits) | 6 | status bar zone (+ owning surface detail); completion/failure → notice | Never a toast per progress tick |
| ERC/DRC findings | 7 | findings surface (list + markers + cross-probe) + status-bar count | Separate build; not console content |
| Authoring handoff / production artifact narration | 1/8 | history (audit); feedback line for the acknowledgement | Long-term: project committed ops from the journal, not parallel strings |
| Session audit ("what just happened") | 8 | history (expandable), journal-backed for committed ops | Retention question for the owner (§7) |
| Typed verb input | 9 | the Command Console *input* row — **owner disposition** | The only genuinely open product question |

## 7. Owner disposition (resolved 2026-08-22)

The owner reviewed this research and the comparative prototype, then approved
the complete disposition below. Product Mechanics 033 is the resulting
authority; this research remains its evidence.

1. **Interactive Editor Command Line — RESOLVED by owner, 2026-08-22:** retire
   it from the target product contract. The Datum Console remains unilaterally
   output-only: it accepts no authored text, dispatches no verb, and emits no
   design operation. Datum Terminal remains the command-line input surface;
   manual GUI verbs remain tools, menus, shortcuts, and marking gestures over
   typed operation authority. Candidate B remains in the visual study only as
   rejected comparative evidence, not a future upgrade path.
2. **Visual model:** Candidate A is the Datum Console. Candidate C remains a
   separate, complementary notification tier; Candidate B is rejected.
3. **Routing:** the §6 matrix is ratified. In particular, terminal lifecycle
   producers leave editor feedback and route to terminal chrome, with severe or
   persistent facts also going to Notices.
4. **Placement:** the Console is a lower-left overlay in the focused viewport
   pane (P1/P3/P5). It reserves no canvas geometry; unfocused panes are silent.
5. **History:** expandable, deterministically bounded session history. Committed
   design operations project from the canonical journal instead of being copied
   into a rival string log.
6. **Sequencing:** Console implementation is the selected next task, but there
   is no hard technical dependency between it and the direct GUI write path.

## 8. Implications the next spec pass must carry (whatever is chosen)

- A **typed message record** (severity, source, category, timestamp, optional
  target/action) replaces `Vec<String>` before any surface renders it —
  convergent with `dat-notification-system-ztp`'s `Notice` sketch; publishers
  state facts, the shell decides presentation.
- **Canonical echo vocabulary**: one wording authority (ideally derived from
  the verb registry / journal provenance), no internal ids leaking to the UI.
- **Announcement events** (AT-SPI) emitted alongside every visible status
  change; polite for echoes, assertive only for critical errors.
- **Repaint on narration** once any surface is visible.
- **No auto-dismiss without a persistent home**, hover-pause, and a user
  duration setting (incl. off).
- The **terminal remains a foreign shell**: nothing here writes application
  text into terminal cells; terminal chrome keeps its own status line.

## 9. Sources

The product conclusions above were rechecked against current primary sources on
2026-08-22. Secondary historical material informed detail such as older EAGLE
lineage and AutoCAD preferences but does not control the recommendations.

- Autodesk AutoCAD: [command-window control and F2 history](https://help.autodesk.com/view/ACD/2022/ENU/?guid=GUID-363A3BFA-CAF2-469E-9F35-0BF64139811C),
  [command-window navigation and extended history](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-Core/files/GUID-06F6A6E9-C355-43F5-8706-49F0E5E552BF.htm),
  and [current command-window opening workflow](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-C9DB1661-36D0-4E5B-99A0-43A6ACA46110-htm.html).
- Autodesk Fusion Electronics: [command-line control, abbreviations, history,
  scripts, and mixed icon/text input](https://help.autodesk.com/view/fusion360/ENU/?guid=ECD-CMD-CTRL-CPT),
  [command catalog](https://help.autodesk.com/cloudhelp/ENU/Fusion-ECAD/files/ECD-CMD-LINE-CMDS.htm),
  and [Electronics shortcuts](https://help.autodesk.com/view/NINVFUS/ENU/?guid=GUID-F0491540-0324-470A-B651-2238D0EFAC30).
- Altium Designer: [Board Insight HUD](https://www.altium.com/documentation/altium-designer/pcb/board-insight-system),
  [Messages panel environment contract](https://www.altium.com/documentation/altium-designer/design-environment-elements),
  and [validation, severity, cross-probe, and remediation behavior](https://www.altium.com/documentation/altium-designer/schematic/design-validation).
- KiCad: [PCB Editor manual](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html)
  and [KiCad 8 release notes](https://www.kicad.org/blog/2024/02/Version-8.0.0-Released/).
- Horizon EDA: [shared tool model and active-tool bottom bar](https://docs.horizon-eda.org/en/stable/tools.html)
  and [Spacebar Menu](https://docs.horizon-eda.org/en/latest/spacebar-menu.html).
- Blender: [Status Bar keymap/progress/report contract](https://docs.blender.org/manual/en/latest/interface/window_system/status_bar.html)
  and [Info Editor operator/message history](https://docs.blender.org/manual/en/latest/editors/info_editor.html).
- Notification systems: [VS Code notifications and contextual-progress guidance](https://code.visualstudio.com/api/ux-guidelines/notifications),
  [GNOME feedback taxonomy](https://developer.gnome.org/hig/patterns/feedback.html),
  and [GNOME notification visibility/recoverability guidance](https://developer.gnome.org/hig/patterns/feedback/notifications.html).
- Accessibility: [WCAG 2.1 §4.1.3](https://www.w3.org/TR/WCAG21/#status-messages)
  and [W3C status-message explanation](https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html).

Internal evidence: `crates/gui-protocol/src/workspace_layout.rs` (sink),
`crates/gui-app/src/terminal_narration.rs` (route), the 105 `log_review_event`
sites across `crates/gui-app/src/`, `crates/gui-render/src/render/status_bar.rs`,
`crates/gui-render/src/terminal_tab_strip.rs`, commit `419bcf6`, beads
`dat-output-lane-t6v`, `dat-gui-write-path-qiu`, `dat-notification-system-ztp`,
`research/terminal-redesign/TERMINAL_CURRENT_STATE_MAP.md`,
`docs/gui/DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md`.
