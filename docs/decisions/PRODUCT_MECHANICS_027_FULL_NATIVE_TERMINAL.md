# Product Mechanics 027: Full Native Terminal Product

Status: ratified doctrine

## Decision

Datum ships a fully fledged native terminal, not a terminal-like console widget.
It must be suitable as a user's daily project terminal and must run arbitrary
interactive programs—including Codex, Claude Code, Cursor-compatible command-line
agents, local agents, shells, editors, multiplexers, debuggers, and build tools—
with the same behavioral expectations users bring to Ghostty, Konsole, and
Alacritty.

Datum owns the native product integration: terminal tabs, splits and detachable
views inside the Datum shell; focus and input arbitration; wgpu presentation;
project-context injection; and the secure bridge to `datum-eda` and Datum MCP.
Datum does **not** make terminal emulation a proprietary core competency. It
adopts and pins a mature permissively licensed VT/state core behind a narrow
`TerminalCore` adapter. `libghostty-vt` is the primary integration target because
it provides parsing, terminal state, Unicode behavior, scrollback, reflow, and
renderer state as an embeddable library. A pinned `alacritty_terminal` adapter is
the evidence-gated fallback if the libghostty API or build integration fails the
declared production gates. Continuing Datum's bespoke string/RLE screen into a
homegrown cell-grid/reflow engine is not an allowed fallback.

This decision amends Product Mechanics 024 where 024 required Datum to build and
own the cell-grid state model, bounded the protocol ceiling below the owner's
goal, or treated the current parser as sufficient evidence of terminal quality.
It preserves 024's real-shell posture, single focus authority, PTY separation,
render-adapter boundary, and prohibition on using the terminal as a GUI mutation
bridge.

## Product Boundary

"Native" means that the terminal is a first-class Datum surface with Datum-owned
interaction, visual integration, lifecycle, and project context. It does not mean
that Datum rewrites decades of terminal emulation. A mature embedded core is
analogous to using a font shaper or GPU API: the dependency supplies correctness;
Datum supplies the product.

Feature parity applies to terminal-session and terminal-surface capabilities.
Standalone application packaging, operating-system auto-update, global dropdown
windows, and desktop-wide window management are outside an embedded EDA surface.
Terminal semantics, interactive-program compatibility, text interaction,
protocols, accessibility, session management, profiles, tabs, splits, detachable
views, and performance are in scope.

## Normative Rules

- **FT-001 — foreign-shell screen authority.** The active PTY byte stream, as
  interpreted by `TerminalCore`, is the sole authority allowed to mutate terminal
  cells. Datum diagnostics, activity summaries, lifecycle messages, command
  echoes, and GUI notices never enter the terminal grid or consume terminal rows.
  They use terminal chrome, notifications, the Command Console, or logs.
- **FT-002 — mature core, pinned and replaceable.** Datum integrates a pinned
  mature core behind a closed adapter covering input bytes, resize, modes, damage,
  selection coordinates, render-state extraction, title/CWD/bell events, and
  replies to the PTY. No consumer depends directly on vendor-private structures.
- **FT-003 — real PTY and process tree.** Every session owns a real PTY, shell or
  requested executable, process group, cwd, environment, size, lifecycle, and
  independent terminal state. Job control, signals, long-running processes,
  pipelines, SSH, multiplexers, and alternate-screen programs must work.
- **FT-004 — complete text model.** The terminal supports Unicode grapheme
  clusters, combining marks, wide cells, emoji sequences, font fallback, complex
  shaping where the selected core exposes it, configurable fonts and sizes,
  device-scale changes, ligature policy, cursor styles, and lossless 24-bit color
  plus text attributes. Width is never inferred from Rust scalar count.
- **FT-005 — modern compatibility ceiling.** The target includes xterm/DEC
  compatibility, bracketed paste, focus and mouse reporting, application cursor
  and keypad modes, synchronized output, OSC 8 hyperlinks, controlled OSC 52
  clipboard, kitty keyboard protocol, and terminal graphics supported by the
  selected mature core, including kitty graphics and sixel where available.
  Unsupported protocol claims must be explicit, tested, and owner-approved; they
  may not disappear behind the word "polish."
- **FT-006 — daily-driver interaction.** Pointer and keyboard selection support
  character, word, line, block, and select-all scopes; copy/paste respects the
  system clipboard and Linux primary selection; scrollback is bounded and
  configurable; resize reflows logical lines; search supports next/previous,
  case sensitivity, regular expressions, and match visibility; hyperlinks and
  detected paths are actionable through explicit user gestures.
- **FT-007 — native session UX.** Datum supports multiple terminal tabs, splits,
  tab naming/title updates, restart, close/kill confirmation, attach/detach where
  the process model supports it, profiles, themes, font zoom, cwd-aware new
  sessions, bell/activity indication, and a terminal-only maximize/focus mode.
- **FT-008 — one input authority.** Keyboard, IME/preedit, composed/dead keys,
  paste, mouse protocols, selection gestures, and Datum shortcuts route through
  one focus authority. Opening or observing the terminal never steals input.
  Clicking terminal content deliberately focuses it; canvas focus returns keys to
  the editor. Application mouse capture and local selection have explicit modifier
  arbitration.
- **FT-009 — agent and tooling pipeline.** Code agents run as ordinary native PTY
  processes with their full-screen UI, colors, mouse, clipboard, signals, and
  authentication environment intact. Each session receives namespaced Datum
  bootstrap context: project root/id, model revision, session/context IDs,
  discovery path, canonical CLI, and an authenticated MCP endpoint or connection
  descriptor when available. Context can be refreshed without restarting the
  shell. The terminal grants no private mutation power: `datum-eda`, MCP, and
  typed engine operations remain the only design-authority paths.
  Product Mechanics 028 controls discovery and interoperability: a bootstrap
  environment variable or suggested launch prompt is not sufficient. Supported
  clients must receive native MCP registration, immutable pinned context,
  scoped authority, portable workflows, and production round-trip proof.
- **FT-010 — security boundary.** Paste, OSC 52, hyperlinks, file/URI opening,
  shell integration, environment injection, remote terminfo setup, and process
  termination have explicit trust and confirmation policies. Secrets are not
  copied into tracked context files or telemetry. Terminal output is untrusted
  display input and cannot invoke Datum GUI actions merely by emitting escapes.
- **FT-011 — accessibility.** Terminal text, cursor, selection, search matches,
  tabs, lifecycle, bell, and focus are exposed through an accessibility model.
  Color is never the sole state cue; high contrast, reduced motion, keyboard-only
  operation, and screen-reader traversal have acceptance evidence.
- **FT-012 — measured production quality.** "Pro-grade," "native," and
  "compatible" are release-gated claims. The checked-in matrix records exact
  results for `vttest`, `esctest2`, a Unicode/emoji/complex-text corpus, resize and
  scrollback stress, protocol/security cases, and real applications: bash, zsh,
  fish, SSH, tmux, less, Vim/Neovim, htop/btop, Python, Git, Cargo, Codex, Claude
  Code, and at least one local agent. Renderer throughput, input-to-present
  latency, memory per retained line, and sustained-output behavior have explicit
  budgets and reproducible fixtures.
- **FT-013 — no partial-product substitution.** A focus fix, PTY swap, parser test,
  cell grid, or command-console surface is not a completed native terminal. Each
  is only evidence for its named slice. The epic closes only when the full
  governed product matrix is verified on a production Datum build.

## Delivery Sequence

The implementation sequence is intentionally product-vertical:

1. **T0 shell truth:** remove every non-PTY writer/presentation from terminal
   cells; make terminal content itself focusable; land a real-shell visible-output
   canary. Preserve the valid TF-01 focus-owner extraction but do not claim the
   terminal usability defect repaired until this gate passes.
2. **T1 core and transport:** pin and integrate `libghostty-vt` through
   `TerminalCore`, retain an evidence-gated Alacritty fallback, and replace the
   unsafe PTY with `portable-pty`. Delete the bespoke terminal state/parser path
   after behavioral parity, rather than maintaining rival cores.
3. **T2 native renderer and input:** adapt core render state to Datum's wgpu/text
   pipeline; complete focus, IME, keyboard, mouse, clipboard, selection, fonts,
   resize, damage, and accessibility.
4. **T3 daily-driver surface and protocols:** finish scrollback/search,
   hyperlinks, graphics, tabs/splits/profiles/themes, shell/terminfo integration,
   lifecycle, security prompts, and detachable/maximized terminal UX.
5. **T4 Datum-agent integration and production proof:** implement the
   decision-028 launcher/discovery, standard MCP, pinned-context/authority, and
   portable-workflow slices; then verify the named agent/application matrix and
   meet conformance, latency, throughput, memory, security, visual, and
   accessibility gates.

No later stage may be represented as optional "polish" when it contains a
capability required by this decision.

## Consequences

Datum gains a terminal capable of hosting the same real work users perform in a
dedicated terminal while remaining coherent with the EDA shell. The project
stops paying the permanent correctness tax of a private emulator core and spends
its engineering effort on the Datum-specific adapter, interaction, renderer,
context, and verification layers.

The existing TF-01 commit remains valid evidence only for establishing a single
keyboard-focus owner. Its prior claim that it repaired the user-visible terminal
is superseded. TF-02 through TF-05 remain focus/input requirements under T0/T2;
they are not the definition of the terminal product.

## Course-correction completion anchors

<!-- REQ:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C01 -->
- **NTC-C01 — audit the product gap.** Reconcile the owner-observed unusable
  screen, the implementation's application-owned terminal rows, decisions
  005/024, the existing epic, and the full-terminal competitive capability bar.
<!-- EVIDENCE:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C01-AUDIT -->

<!-- REQ:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C02 -->
- **NTC-C02 — ratify the correction.** Land this numbered amendment and the
  governed `DATUM_NATIVE_TERMINAL_SPEC.md` product contract.
<!-- EVIDENCE:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C02-CONTRACT -->

<!-- REQ:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C03 -->
- **NTC-C03 — reconcile the execution graph.** Re-scope the existing terminal
  epic and children into T0–T4, preserve useful history, align hard dependencies,
  and make T0 shell truth the sole next execution slice after correction.
<!-- EVIDENCE:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C03-GRAPH -->

<!-- REQ:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C04 -->
- **NTC-C04 — repair claims and guidance.** Preserve TF-01 only as focus-owner
  extraction evidence and prohibit any focus/parser/grid slice from claiming the
  full terminal or visible usability without the T0 and T4 proof gates.
<!-- EVIDENCE:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C04-CLAIMS -->

<!-- REQ:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C05 -->
- **NTC-C05 — validate and hand off.** Regenerate the Frontier, pass project-
  state/governance/source-health gates, close the correction bead with evidence,
  and select T0 without auto-authorizing later phases.
<!-- EVIDENCE:FULL-NATIVE-TERMINAL-CONTRACT:NTC-C05-HANDOFF -->
