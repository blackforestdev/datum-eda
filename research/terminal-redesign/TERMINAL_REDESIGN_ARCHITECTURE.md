# Datum Terminal — Pro-Grade Redesign: Recommendation & Architecture (research, 2026-07-12)

Companion to `TERMINAL_CURRENT_STATE_MAP.md`. Synthesizes a 3-agent build-vs-adopt
research pass into a recommendation and a concrete architecture. **Research only —
no code changed.** Input to a governed decision record / redesign spec.

Target set by owner: a **pro-grade native terminal** (Ghostty/Alacritty-class) built
into Datum — run code agents natively, multi-tab bash/python, full syntax
colorization, real emulator — plus the sanctioned Datum context "back-door."

---

## Recommendation: BUILD the state model, on three guardrails, behind an adopt-agnostic interface

**Build** — but precisely scoped as *"own the cell-grid state model,"* not *"write a
terminal from scratch."* This aligns with the owner's inclination, Datum's own-your-core
doctrine, and — critically — the evidence:

- **Datum is already ~60% of the way there.** The ~2,100-line VT/ANSI parser and the
  fully-tested xterm keyboard+mouse codec (`terminal_input.rs`) are exactly the parts
  Alacritty and wezterm factored into standalone crates because they're the stable
  foundation. The pro-grade gap is *concentrated and bounded*: a real `Grid<Cell>` data
  model, Unicode width, reflow, and scrollback/alt-screen separation.
- **The integration is unusually clean.** Datum's wgpu path is *already* two-pass (quads
  drawn before text) — exactly the shape a cell grid wants. `Quad` is already the bg-rect
  /underline primitive; `TextRun` already carries truecolor (`color:[f32;3]`). The
  parser just needs its **write target** swapped from `Vec<String>`+RLE to `Grid<Cell>`.
- **No copyleft anywhere** in the Rust VT space — licensing is a non-issue either way.

### The three guardrails that make "build" safe rather than a treadmill
1. **Don't hand-roll Unicode.** Link `unicode-width` + `unicode-segmentation` (MIT/Apache)
   for grapheme/width — the single deepest correctness pit. This is the #1 bug generator
   in every emulator and it *drifts yearly*; do not author width tables.
2. **Gate on the free conformance suites.** `vttest` (interactive) + `esctest2`
   (automated, one test class per escape sequence) are the real definition of
   "pro-grade" and cost nothing to borrow. Correctness becomes a measured pass-rate, not
   a claim — the highest-leverage thing to take from the incumbents without linking a line
   of their code.
3. **Declare the ceiling out of scope.** No sixel / iTerm2 / kitty graphics protocols, no
   per-terminal width correction tables. Floor = correct grid + full color/attributes +
   width + scrollback + reflow. The graphics long tail is a multi-year product commitment
   an EDA app should decline. (Ghostty: 3+ years, ~70% of it on *rendering*.)

### The strategic insight that de-risks the whole decision
**The high-leverage work is identical whether Datum builds or adopts the VT core.** The
wgpu renderer interface (`Grid → (Vec<Quad>, Vec<TextRun>)`), the `portable-pty` swap, the
keyboard-focus authority, and moving the grid out of `gui-protocol` are the *same code*
whether the grid behind them is Datum's own `Grid<Cell>` or `alacritty_terminal`'s
`RenderableContent`. So:

- Build is **reversible**: if the width/reflow treadmill proves too costly, swap
  `alacritty_terminal` (Apache-2.0, pure Rust, proven as a library by Zellij) in behind the
  *same* renderer interface, with the surrounding work already done and kept.
- Therefore the commitment order is: **do the shared/identical work first** (it's also
  where the P1 bug and the immediate user pain live), and treat "own vs adopt the state
  model" as a **contained, swappable core decision** — not a bet the whole redesign rides on.

Fallback if ever needed: **`alacritty_terminal`** today; **`libghostty-vt`** to
re-evaluate in 6–12 months once its C API stabilizes (best engine — reflow, Kitty
graphics — but Zig+FFI, unstable, `!Send`).

---

## Reuse / Replace / Swap / Move map (from the integration survey)

| Piece | Verdict | Note |
|---|---|---|
| VT parser state machine (`terminal_screen/*`: escape/SGR/OSC/CSI/DCS, charsets, tab stops, scroll regions) | **Keep, re-target writes** | Mature + tested. It writes directly into `TerminalLaneState`; keeping it means swapping its write target to `Grid<Cell>`. **This is the bulk of the real work.** |
| `terminal_input.rs` key/mouse→byte codec | **Keep unchanged** | Pure `input→Vec<u8>`; zero screen-model coupling. |
| `TerminalSessionRegistry` multi-tab + lifecycle | **Keep** | Only the per-slot screen's internal model changes; registry API untouched. |
| `TerminalEvent` mpsc + reader/wait threads | **Keep** | Byte transport is model-agnostic. |
| `Vec<String>` + RLE `styled_lines` model + `terminal_grid.rs`/`terminal_style.rs` span-shuffling | **Replace** | → `Grid<Cell>` (truecolor + attr flags + wide-char + scrollback ring + damage). ~400 lines of string/span juggling collapse into grid row-copy ops; RLE coalescing disappears (style is per-cell). |
| Lossy render map in `bottom_dock.rs` (`terminal_span_color`, 7.9px advance, `.take(180)`, stringly-typed colors) | **Replace** | → `TerminalGrid::emit → (Vec<Quad>, Vec<TextRun>)`. Stringly color table **deleted**; truecolor flows straight through `TextRun.color`. |
| libc PTY (`open_pty_pair`, `configure_child_pty`, ioctl resize) | **Swap** | → `portable-pty` (safe, cross-platform). Retain a thin `libc::killpg` for **process-group** signals (Ctrl-C to child pipelines) — the sole reason any libc stays. |
| Cell grid's home crate | **Move** | Out of `gui-protocol` (it never serializes / the engine never sees it — mis-filed as scene state) into `gui-app`. Shrink the boundary type to terminal **chrome** only (title/cwd/bell/tabs/cursor-style/mode-flags). |
| Context back-door (`terminal_context*.rs`, env + `.datum/` sidecars, activity spans) | **Keep unchanged** | Orthogonal — operates at spawn time + filesystem; never reads the cell model. Only its host call `Command → CommandBuilder` is retargeted under portable-pty. |

**The single genuinely new contract:** a font-derived `(cell_w, cell_h)` + wide-char
width policy (advance = `2*cell_w` for East-Asian-Wide + spacer cell). The measuring
machinery already exists (`geometry.rs:1250` shapes Mono at real metrics); measure one
Mono glyph advance per size → `cell_w`.

The render interface (lives in `gui-render`):
```
TerminalGrid → fn emit(&self, origin, cell_w, cell_h, damage)
                 -> (Vec<Quad> /* bg + underline/strike + cursor */, Vec<TextRun> /* glyphs */)
```
Renderer consumes `Quad`/`TextRun` exactly as today; only the producer changes. Two-pass:
bg-rect run per bg-color run → glyph atlas-quad per non-blank cell → cursor overlay last;
damage set drives which rows re-emit.

---

## Phased plan (dependency-ordered; each phase shippable)

**Phase 0 — Shared foundation (build/adopt-agnostic; fixes the live pain):**
- The **keyboard-focus authority** (`dat-terminal-focus-authority`, P1): one `FocusTarget`
  (`Viewport(pane) | Terminal | TextField`), collapse the ~25 ordered guard arms into one
  router, reconcile with pane-focus (decision 021), split visibility ≠ keyboard-focus ≠
  window-focus, rewire `\x1b[I/O` to terminal focus, define detached-focus feedback. Also
  folds in `dat-terminal-dual-input-model` (the "second model" is a decayed rename field).
- **`portable-pty` swap** (retain thin `killpg`).
- Stand up the **`Grid → (Quad, TextRun)` render interface** and the font-measured cell
  metric — even over the current model first, to lock the interface.
- Route **Codex's pan-trace out of the PTF screen** (`dat-pan-trace-terminal-pollution`) —
  a doctrine fix regardless of the rest.

**Phase 1 — The cell grid (the core build):**
- `Grid<Cell { grapheme, fg: Rgb, bg: Rgb, flags }>` with per-cell truecolor + all SGR
  attributes; retarget the parser's writes; delete the RLE span machinery.
- Lossless render: paint bg rects, underline/strike/overline, 256/truecolor — everything
  the parser already captures now reaches the screen.

**Phase 2 — Width + scrollback correctness:**
- `unicode-width` + `unicode-segmentation`; cursor advances by cell width; wide cells =
  lead + spacer; combining marks width 0. Gate on esctest2 + a curated CJK/emoji corpus.
- Separate visible screen / bounded scrollback ring / alt-screen (DEC 1049 leaves
  scrollback untouched). Removes `MAX_TERMINAL_ROWS=240` conflation.

**Phase 3 — Reflow (highest-risk single feature):**
- Record wrapped-vs-hard line continuation; rewrap on column change, preserving cursor,
  wide-cell boundaries, scroll region, scrollback. Only possible after Phases 1–2.

**Phase 4 — Damage tracking + polish:**
- Per-row/per-cell dirty flags → renderer re-uploads only changed rows (perf at scale).
- Round out DEC-mode breadth (DECSCUSR cursor style, synchronized output 2026, focus 1004),
  OSC 8 hyperlinks / OSC 52 clipboard *if wanted* (security caveats). Charset/DEC
  line-drawing (the currently-discarded charset designation).

**Explicitly deferred / out of scope:** sixel, iTerm2, kitty graphics; kitty text-sizing;
per-terminal width correction tables.

**Conformance gate across all phases:** `vttest` + `esctest2` pass-rate is the definition
of done for "pro-grade," tracked as it climbs.

---

## Governance / next steps
- This ratifies mechanism (input-routing authority + a terminal-core architecture) and
  touches decision 021 → it wants a **numbered decision record** + **Active Frontier**
  placement, not a loose bug-fix. Phase 0's focus authority is the natural first executable
  spec for Codex.
- **Tracker reconciliation needed:** the emulator work is now scoped *pro-grade* (was a
  vague "supporting lane"); `dat-terminal-focus-authority` becomes Phase 0 spine;
  `dat-output-lane` must be rewritten to respect doctrine (no Output tab — Command Console
  + console sink); the pro-grade emulator build should be its own tracked epic with the
  phase list as children.
- Doctrine alignment confirmed: the owner's vision (real shell + run agents + context
  back-door) *is* decision 005 held to a higher bar; the only doctrine conflict is the
  pre-existing GUI-writes-CLI-strings-into-PTY path (board-text), which is a **separate**
  write-path track, not part of this terminal build.
