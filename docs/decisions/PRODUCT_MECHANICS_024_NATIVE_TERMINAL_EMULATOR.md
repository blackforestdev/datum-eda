# Product Mechanics 024: Native Terminal Emulator

Status: ratified doctrine

## Decision

Datum's embedded terminal is built up to a **pro-grade native terminal emulator**
(Ghostty/Alacritty-class): a real PTY-backed shell with a real cell-grid state
model, full color and text attributes, Unicode width, scrollback, and reflow —
in which the user runs code agents, shells, `bash`/`python`, builds, and the
Datum CLI across multiple tabs.

Datum **builds** this by owning the cell-grid **state model** on top of its
already-mature VT parser and xterm keyboard/mouse codec — it does **not** rewrite
the parser and does **not** write a terminal from scratch. The VT core is a
**swappable component** behind a stable render interface: the surrounding work
(the render interface, PTY layer, and keyboard-focus authority) is identical
whether the core is Datum's own grid or an adopted permissively-licensed engine,
so the build is reversible and the core is not a bet the redesign rides on.

The terminal is a **real shell and only a shell**. It is never a GUI write path,
never a command console, never a mutation bridge. This decision reaffirms and
raises the bar of decision 005; it is subordinate to the operation/commit model
and the GUI decisions it builds on (§ Relationship).

## Why This Is Required

The owner's product target is a first-class native terminal — run agents, open a
tab, run a script, full syntax colorization — held to Ghostty/Alacritty quality.
A five-layer survey of the current ~16,350-line subsystem
(`research/terminal-redesign/TERMINAL_CURRENT_STATE_MAP.md`) found the
implementation is ~60% of a real terminal already — a mature, tested VT parser
and a complete xterm keyboard+mouse codec — but the layers around them are the
weak ones: there is **no keyboard-focus authority** (routing fakes focus from
dock *visibility*, so workspace hotkeys leak into the PTY and typing can be
silently swallowed), the screen model is a ragged `Vec<String>` with run-length
style spans rather than a cell grid (no wide-char, no reflow, lossy color), and
the PTY is ~90 lines of hand-rolled `unsafe` libc.

A build-vs-adopt research pass
(`research/terminal-redesign/TERMINAL_REDESIGN_ARCHITECTURE.md`) established that
(a) the entire Rust VT-engine space is permissively licensed — no copyleft
blocker either way; (b) the pro-grade gap is *bounded and concentrated* in the
data model, width, and reflow; and (c) the correctness treadmill of terminal
emulation (Unicode width/grapheme, reflow) is real and permanent, so a build must
be disciplined by borrowing the field's free assets — the `unicode-width`/
`unicode-segmentation` crates and the `vttest`/`esctest2` conformance suites —
rather than hand-rolling them. Building the state model fits Datum's
own-your-core doctrine while the swappable-core architecture keeps `adopt` as a
safety net.

## Normative Rules

- **TE-001 (real shell, higher bar).** The embedded terminal remains a real
  PTY-backed shell (decision 005), now built to pro-grade emulator quality. It is
  not a fake command pane, a Datum-only console, an agent proxy, or a mutation
  bridge.
- **TE-002 (build the state model, keep the parser/codec).** Datum owns the
  `Grid<Cell>` state model; the existing VT parser and the `terminal_input.rs`
  keyboard/mouse codec are **kept** and re-targeted, not rewritten. Writing a new
  ANSI state machine from scratch is forbidden waste.
- **TE-003 (swappable core, adopt-agnostic interface).** The VT core sits behind
  a stable render interface (`Grid → (Quad, TextRun)`); the interface, the PTY
  layer, and the focus authority MUST be independent of whether the core is
  Datum's own grid or an adopted engine (`alacritty_terminal` today,
  `libghostty-vt` to re-evaluate later). A build decision must not foreclose
  swapping the core.
- **TE-004 (the three guardrails).** A build MUST: link `unicode-width` +
  `unicode-segmentation` rather than hand-roll Unicode width/grapheme; gate
  correctness on the `vttest` + `esctest2` conformance suites (pro-grade is a
  measured pass-rate, not a claim); and declare the graphics ceiling **out of
  scope** — no sixel, iTerm2, or kitty graphics protocols; no per-terminal width
  correction tables.
- **TE-005 (one keyboard-focus authority).** Keyboard routing MUST flow through a
  single focus authority (`FocusTarget = Viewport(pane) | Terminal | TextField`),
  not emergent match-guard order over dock visibility. Dock **visibility**,
  keyboard **focus**, and OS **window** focus are three separate axes; terminal
  focus-report sequences (`\x1b[I`/`\x1b[O`) track terminal focus, not window
  focus. This authority MUST subsume/compose the pane-focus notion of decision
  021, and detached-session focus MUST give feedback, not silently swallow keys.
- **TE-006 (GUI never writes to the PTY).** GUI actions MUST NOT synthesize CLI
  strings into the terminal as an action mechanism (reaffirms decision 005 and
  the `ConsoleLaneState` doctrine). The existing board-text CLI-string-into-PTY
  path is a pre-existing violation to be removed onto the typed-Operation write
  path / Command Console — a **separate** write-path track, not part of this
  terminal build.
- **TE-007 (the cell grid is a gui-app concern).** The `Grid<Cell>` model lives
  in `gui-app` beside the parser, not in `gui-protocol` — it never serializes and
  the engine never sees it; it is neither engine truth nor a scene projection.
  The gui-app↔gui-render boundary carries render primitives (`Quad`, `TextRun`)
  plus a small terminal-**chrome** type, not the cell content.
- **TE-008 (PTY layer).** The hand-rolled libc PTY is replaced by `portable-pty`,
  retaining only a thin `killpg` for process-group signals (Ctrl-C to child
  pipelines). The context back-door (env + `.datum/` sidecars) is orthogonal and
  preserved; only its host call is retargeted.
- **TE-009 (phased, shippable, conformance-gated).** Delivery is phased and each
  phase is shippable: Phase 0 foundation (focus authority + `portable-pty` +
  render interface) → Phase 1 cell grid + lossless color → Phase 2 width +
  scrollback separation → Phase 3 reflow → Phase 4 damage tracking + DEC/OSC
  polish. Graphics is deferred out of scope. Tracked as the
  `terminal-emulator` epic in the issue tracker; conformance pass-rate is the
  definition of "pro-grade."

## Relationship to Existing Decisions

This decision builds on and is subordinate to:

- **The operation/commit model + Lean ethos (`CLAUDE.md`, PM-000 series):** one
  mutation path, no private write paths. TE-006 applies it — the terminal is not
  a write path.
- **Decision 005 (embedded terminal):** TE-001/005/006 reaffirm the real-shell /
  no-mutation-bridge posture and raise the quality bar to pro-grade. It does not
  amend 005.
- **Decision 019 (GUI product-model recovery):** the terminal build is a named
  part of the GUI surface work; Phase 0's focus authority unblocks a trustworthy
  terminal surface.
- **Decision 021 (workspace pane tiling):** TE-005's focus authority MUST subsume
  or compose 021's pane-focus, which the dock currently ignores.
- **Decision 023 (universal viewport tooling):** the terminal is **not** a
  drawing surface and carries no `ViewportProfile`; but keyboard-focus authority
  spans both viewport panes and the terminal, so TE-005 is the input-routing
  counterpart that must reconcile with 023's per-surface interaction backbone
  rather than fork a second focus notion.
- **Decision 013 (vacated supervision surface):** the reframed
  GUI-message/diagnostics need (formerly a proposed "Output tab") respects 013 —
  no standalone supervision/Output tab; the sanctioned homes are the Command
  Console + `ConsoleLaneState` sink and files-in-workdir + viewers.

It does not amend any of them; on conflict the higher decision wins and this
document is the one to fix.

## Consequences

Datum gains a real, native, pro-grade terminal that runs agents and shells with
full colorization — reusing the ~60% it already built (parser + codec) and
paying only the bounded cost of the data model, width, and reflow, disciplined by
free conformance suites and the Unicode crates. The keyboard-focus authority
fixes the terminal being un-typeable and hotkeys leaking into the PTY as a
by-product of unification. Because the VT core is swappable behind a stable
interface, the build is reversible — `alacritty_terminal` can be dropped in later
with the surrounding work kept. The cost is the multi-month Phase 1–3 model
rewrite plus a permanent Unicode-width maintenance tax; the graphics long tail is
explicitly declined as outside an EDA app's remit. The one job this decision
*removes* from the terminal — being the GUI's CLI-string write path — is
re-homed on the typed write-path/Command Console track, restoring the
one-mutation-path law.
