# Datum Terminal — Current-State Map (research, 2026-07-12)

Working notes grounding a terminal design/redesign. Assembled from a 5-agent
read-only survey of the ~16,350-line terminal subsystem in `crates/gui-app`,
`gui-protocol`, `gui-render`, plus governing docs/decisions. **Research only —
no code changed.** Not a spec; input to one.

---

## Headline

The terminal implementation and the ratified doctrine have diverged in one
specific, coherent way:

- **Doctrine** (decision 005 + `DATUM_GUI_DESIGN_SPEC` "three surfaces"): the
  bottom-dock terminal is a **real PTY shell** and *nothing else*. Editor-driving
  input belongs to a **separate, viewport-anchored Command Console** (type a verb,
  act on the hovered object) that **does not exist yet**. GUI actions must **never
  write to the PTY**; GUI edits must emit **typed `Operation`s through `commit()`**.
- **Implementation**: the terminal is a real PTY shell that is *also* being used as
  the GUI's **de-facto write path**, by synthesizing `datum-eda …` CLI strings and
  piping them into the PTY. It is doing exactly the double-duty doctrine forbids —
  because the Command Console and the typed GUI→engine write path don't exist yet,
  the terminal is standing in for both.

So "fix the terminal" is really **two separable problems** the current code
conflates:

- **(A) Fix the terminal *as a terminal*** — doctrine-aligned, unblocked: a real
  keyboard-focus authority (the P1 bug), keep the mature codec + parser, replace
  the lossy screen model/renderer, trim PTY/session cruft.
- **(B) Stop using the terminal as the write path** — extract CLI-string authoring
  (board-text edit) *out* of the PTY and onto typed Operations / the future Command
  Console. Entangled with the GUI write-path gap and the Command Console frontier
  item; **not** a terminal-internal fix.

---

## Layer map

### 1. Session / PTY lifecycle
- **Real Unix PTY + child shell**, hand-rolled on raw `libc` (`posix_openpt`/
  `grantpt`/`unlockpt`/`ptsname_r`, `setsid`+`TIOCSCTTY`, `dup2`) — no
  `portable-pty`/`nix`. `$SHELL` or `/bin/sh`, `TERM=xterm-256color`. (`terminal_session.rs`)
- **Multi-tab** via `TerminalSessionRegistry { sessions: Vec<Slot>, active_index }`,
  projected to `TerminalLaneState`/`TerminalTabState` for the renderer. Several
  multi-tab methods are `#[allow(dead_code)]` — registry supports more than the UI wires.
- **Two OS threads/session**: blocking reader → `mpsc<TerminalEvent>`; wait thread → `Exited`.
  Output pump lives in `Runtime` (outside these files). Resize = `ioctl(TIOCSWINSZ)` + screen resize.
- **Engine linkage is one-directional**: env vars (`DATUM_*`) + `.datum/` sidecar
  JSON + per-session `*.events.jsonl` provenance. The shell is a normal shell that
  knows where the project is; **no socket to engine/daemon**.
- **Cruft**: hand-rolled libc PTY (unsafe `pre_exec`, Linux-only — consider
  `portable-pty`); triple-file lifecycle writes per transition; two parallel state
  notions (`status: String` vs journaled `DatumToolSessionLifecycle`) nothing reads
  back; **event-log re-read+reparse on *every output chunk*** (O(file)/chunk) to
  detect command completion; `python3 -c` finish hook (fragile); "detach" is
  cosmetic (never backgrounds the process).

### 2. Emulator / screen
- **Mature, expensive-to-replace VT/ANSI parser** (~2,100 lines) + **~1,900 lines of
  tests**: SGR (incl. 256/truecolor parsed), full cursor/erase/scroll-region/IL-DL-SU-SD,
  DECSET/DECRST matrix, alt-screen, OSC 0/1/2/7, tab stops, DA/DSR/DECSCUSR/XTWINOPS,
  C1 8-bit, REP, RIS. **KEEP.**
- **Thin, lossy data model**: `TerminalLaneState.lines: Vec<String>` (ragged) +
  parallel RLE `styled_lines` with **stringly-typed colors** — **not a cell grid**.
  No wide/CJK column math, **no reflow on resize**, scrollback+screen conflated in
  one `MAX_TERMINAL_ROWS = 240` buffer.
- **Lossy renderer** (`gui-render/bottom_dock.rs`): draws via the shared Datum text
  engine (IBM Plex Mono, cosmic-text) — good — but **only honors fg/bg/bold/inverse**;
  italic/underline/strike/dim/blink **dropped**; **256/truecolor render as default**;
  **background never painted**; hardcoded 7.9px advance, truncated at 180 cols; three
  inconsistent cell metrics (7.9 / 8×16 / 0.72).
- **Real functional gap**: charset designation parsed then **discarded** → **no DEC
  line-drawing** → box-drawing TUIs break.
- **Redesign**: keep parser; replace `Vec<String>`+RLE model with a real cell grid
  (fixed dims, wide-char aware, separate scrollback ring); make the render
  style/color mapping **total** so what the parser captures actually reaches screen.

### 3. Input routing & focus — the P1
- **No focus authority.** ~25 near-identical `WindowEvent::KeyboardInput` arms in one
  giant `main.rs` match; **arm order = routing priority**; each gated on three ad-hoc
  booleans. "Focus" is reconstructed per-arm from `active_dock_tab` (**visibility**),
  `terminal_rename_session_id` (rename mode), `active_attached()` (PTY attach).
- **Visibility-as-focus**: merely *opening* the terminal to glance at output hijacks
  the whole keyboard — every workspace hotkey (`s/b/v/m/x/r/f/t/z/c/[/]`, pane-Tab) is
  suppressed by its `is_none()` gate and every char goes to the PTY. → **"hotkeys leak
  into PTY."** No visible-but-unfocused state exists.
- **Can't-type-when-detached**: raw path requires `active_attached()`; when detached,
  keys fall through to the vestigial line-editor arms which return `false` → **keys
  silently swallowed, zero feedback** (only Ctrl+C special-cases a message).
- **Dual input model — CONFIRMED but the line-editor is now vestigial**: it only
  survives as the **session-rename text field** (`complete_dock_input` is a dead
  no-op; `submit_dock_input` only submits a rename). Not two peer command models —
  a live PTY path + a half-dead shadow input kept for renaming.
- **Focus-report bug**: `\x1b[I`/`\x1b[O` emitted from **OS window** focus
  (`WindowEvent::Focused`), not terminal/dock focus — wrong signal source.
- **Fragile invariant**: a hand-maintained `consumes_release()` allowlist must stay in
  lockstep with the release arms, or e.g. Enter-release leaks into rename-submit.
- **`terminal_input.rs` is the crown jewel**: a complete, heavily-tested **xterm
  keyboard+mouse codec** (SGR/urxvt/utf8/x10 mouse, app-cursor/keypad, bracketed
  paste, F-keys, modifiers). Pure, focus-agnostic — **KEEP wholesale**; a new router
  just calls `terminal_key_action`.
- **Redesign constraint**: one `FocusTarget` authority (`Viewport(pane) | Terminal |
  TextField(rename)`) replacing the three booleans; make **dock visibility ≠ keyboard
  focus ≠ OS window focus** three separate axes; **reconcile with the existing
  pane-focus system (decision 021)** which the dock currently knows nothing about;
  decide deliberately whether **keyboard-focus or pointer-hover** owns Space/pan
  (today it's implicit pointer ownership that leaks state); define detached-focus as a
  real state with feedback.

### 4. Context system & purpose (the "what is it FOR")
- The terminal carries a heavy **context-projection layer**: it serializes resolved
  engine truth (journal tip, visible/latest proposals, check-run review state,
  production artifacts, selection/cursor) into `.datum/gui-terminal-context.json`
  (`TerminalContextEnvelope`, contract `datum_terminal_context_v1`) every refresh, and
  pre-renders a catalog of `datum-eda …` CLI command strings.
- Purpose today (four overlapping roles): **engine-state viewer**, **AI-agent
  bootstrapper** (hands `claude`/`codex`/`aider` a discoverable context + activity),
  **command console / authoring surface**, **proposal/check reviewer**.
- **Activity spans** = a reconstructed audit timeline folded from the session
  `*.events.jsonl` (the "ACTIVITY SPANS" / "selected terminal activity span" lines in
  the screenshots). **Active context** = focus-derived command bindings for the
  selected object.
- **The doctrine-violating fact**: every write flows out as a **CLI string executed in
  a subprocess shell**. Board-text edit → `datum-eda project edit-board-text …` →
  `write_terminal_bytes(command.as_bytes())` (`runtime_board_text_edit.rs:121`). **Zero**
  `native.write`/daemon/typed-Operation usage in the subsystem. The context system
  exists precisely to make those CLI strings correct — it is a **CLI-string bridge
  substituting for the missing typed GUI→engine write path.**

### 5. Product/spec intent
- **`PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`** (ratified): terminal = **real
  PTY-backed shell**, explicitly *not* "a fake command pane, command palette, agent
  proxy, Datum-only console, or hidden mutation bridge." For VCS/scripts/builds/CAM
  inspection/AI agents on the user's own credentials.
- **`DATUM_GUI_DESIGN_SPEC` "Command Surfaces — three, not one"**: (1) **Command
  Console** — viewport-anchored lower-left verb input, "typed twin of the marking
  menu," drives the *editor*, **not a shell**, **does not exist yet**; (2) **Native
  Terminal** — the real PTY, multi-tab; (3) **AI** = agent-in-terminal + inline ghost
  overlay, no separate tab.
- **Dock**: bottom dock is **terminal alone, multi-tab**; 32px collapsed strip;
  height is session state; solved by Taffy (`UI_LAYOUT_SYSTEM_CONTRACT`,
  `PHASE_1_SPEC`). `DockTab` enum has a **single `Terminal` variant** — so "which dock
  tab" is already vestigial.
- **No Output tab**: `DATUM_GUI_DESIGN_SPEC` explicitly vacates a dedicated Output/
  supervision tab (the vacated decision-013 "meta-supervision" misfire). CAM/export
  results are **files-in-workdir + gerber/drill viewer / paperspace**, not a lane.
- **"Never write to the PTY"** is quoted doctrine: `ConsoleLaneState` doc-comment
  ("a real shell that GUI actions must never write to"; GUI-action echoes land in an
  **invisible model-only sink** until the Command Console is built); `PHASE_1` Do-NOT
  "do not synthesize CLI strings into the terminal as an action mechanism (decision 019)."
- **Active Frontier**: terminal is a **landed read-only supporting dock**, not a
  frontier feature. The **Command Console** is the frontier item (≈ step 4),
  downstream of the GUI write path (`DATUM_GUI_WRITE_PATH_PLAN.md`). Interactive
  terminal-authoring is out of scope until the write path lands.
- **Stale docs**: `WORKSPACE_MODEL.md` / `INTERACTION_MODEL.md` still mention a
  bottom-dock **Assistant** lane; superseded by the design spec (no Assistant, no Output tab).

---

## How this reconciles with the tracker issues filed this session

- **`dat-terminal-focus-authority` (P1)** — **confirmed & sharpened.** Fix = one
  `FocusTarget` authority, collapse the ~25 guard arms into one router, reconcile with
  pane-focus (021), split visibility/keyboard-focus/window-focus axes, rewire
  `\x1b[I/O` to terminal focus, define detached-focus feedback. Doctrine-aligned,
  unblocked. This is problem (A)'s core.
- **`dat-terminal-dual-input-model`** — **confirmed, reframed.** It is *not* two peer
  models; the line-editor decayed into a rename-only shadow input. Cleanup = fold
  rename into an explicit focused text-field/modal and delete the shadow input on
  `ui.terminal.input`. Naturally subsumed by the focus-authority work.
- **`dat-pan-trace-terminal-pollution`** — **confirmed as a doctrine violation**, not
  just UX noise: GUI diagnostics written into the PTY screen breach "never write to
  the terminal." Strong keep.
- **`dat-output-lane` — COLLIDES WITH DOCTRINE; must be reframed.** I proposed an
  "app-owned Output dock tab." Doctrine explicitly vacated a standalone Output/
  supervision tab (013). The sanctioned homes are: (a) the **Command Console**
  (viewport-anchored) for GUI-action echoes + the existing model-only
  `ConsoleLaneState` sink, and (b) files-in-workdir + viewers for CAM output. The
  *legitimate* need behind the issue (Codex trace spam + engine messages need a home
  that isn't the shell screen) is real, but the answer is the Command Console /
  console sink, **not** a new dock tab. Rewrite the issue accordingly.
- **`dat-notification-system`** — still valid (user-facing notices ≠ console echoes),
  but must be designed to respect: not a supervision lane; toasts/banners are viewport
  chrome, and the persistent log is the console sink, not an Output tab.
- **`dat-pan-invocation-decision`** — unchanged; independent viewport decision.

## The real structural insight for the redesign
The board-text CLI-string-into-PTY path and the "output lane" instinct are the same
mistake from two directions: **treating the shell as the app's I/O bus.** Doctrine's
answer already exists on paper — the **Command Console** (in) + **typed Operations /
`commit()`** (write) + **files-and-viewers** (out) — it's just unbuilt, so the terminal
absorbed all three jobs. A clean redesign *subtracts* those three jobs from the
terminal and lets it be a real shell, while the Command Console + write path (frontier
items) take them over.

## Open design forks (need owner input before a redesign spec)
1. **Scope**: do we tackle only (A) fix-terminal-as-terminal now, or also open (B)
   de-CLI-string the write path — which pulls in the Command Console + write-path
   frontier items and is a much larger commitment?
2. **PTY backend**: keep the hand-rolled libc PTY, or adopt `portable-pty` (deletes
   ~70 lines of unsafe, gains portability, adds a dependency)?
3. **Emulator data model**: how far to go on the cell-grid rewrite (wide-char, reflow,
   scrollback ring, total color) vs. minimal fixes — scaled to whether Datum wants a
   pro-grade terminal or a competent supporting lane.
4. **Focus model**: does keyboard-focus or pointer-hover own the Space/pan gesture?
   (This also touches `dat-pan-invocation-decision`.)
5. **Governance**: the focus-authority fix ratifies input-routing mechanism and
   touches decision 021 — likely wants its own numbered decision record + Frontier
   placement, not just a bug-fix commit.
