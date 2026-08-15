# Datum Native Terminal Product Specification

Status: active target contract

Authority: Product Mechanics 005 and 024 as amended by 027 and 028. Decision
027 controls terminal-emulation quality; decision 028 controls agent discovery
and interoperability.

## 1. Product acceptance statement

Datum's terminal is an embedded, fully fledged Linux terminal emulator. A user
must be able to replace a separate day-to-day project terminal with it without
losing shell correctness, interactive applications, code agents, text
interaction, session ergonomics, or safe access to Datum tooling.

The word **terminal** in this specification always means the PTY-backed foreign
shell surface. The viewport Command Console is a separate Datum command surface.
Neither may impersonate the other.

## 2. Architecture

### 2.1 TerminalCore

`TerminalCore` is a narrow Datum-owned adapter over a pinned mature emulator
library. The production integration target is `libghostty-vt`; a pinned
`alacritty_terminal` implementation is permitted only when a recorded bake-off
shows a blocking libghostty integration defect and the fallback closes the same
capability matrix or carries explicit owner-approved deltas.

The adapter owns no terminal semantics. It exposes:

- `feed_pty(bytes)` and terminal-generated reply bytes;
- `resize(columns, rows, pixel_width, pixel_height)`;
- immutable render-state/damage iteration;
- cursor, mode, title, cwd, bell, palette, hyperlink, and graphics state;
- scrollback/search and selection-coordinate operations;
- keyboard/mouse/paste/focus encoders where supplied by the core; and
- reset, teardown, and deterministic test snapshots.

Vendor structures never cross into engine or `gui-protocol`. Terminal state is
process-local consumer state in `gui-app`; the engine never persists it as design
truth.

### 2.2 Session transport

Each terminal tab or split binds one `TerminalSession` to one `TerminalCore`.
The PTY transport uses `portable-pty` unless a platform-specific production
defect is evidenced. It must preserve controlling-terminal setup, process groups,
signals, resize, exit status, inherited user credentials/environment, and
independent concurrent sessions.

### 2.3 Screen authority

The only input to terminal cells is PTY output interpreted by `TerminalCore`.
Datum may display title, cwd, lifecycle, bell, tabs, and controls as chrome
outside the cell rectangle. It must never prepend, append, overlay, or reserve
terminal rows for:

- activity spans or telemetry;
- GUI diagnostics and pan traces;
- command handoff summaries;
- session-open/rename/restart messages;
- engine notices or operation echoes; or
- instructions that belong in help, notifications, or the Command Console.

The screen rectangle has a dedicated hit target and clipping/scissor region.
The PTY row/column size is derived from that exact rectangle after chrome.

## 3. Capability matrix

Every row is required for epic closure. A dependency-provided capability still
needs Datum integration and acceptance evidence.

| ID | Capability | Required outcome |
|---|---|---|
| NT-CAP-01 | Shell/process | Real login/non-login shell templates, arbitrary executable/argv, job control, signals, pipelines, long-running processes, SSH and tmux |
| NT-CAP-02 | VT state | xterm/DEC-compatible primary/alternate screen, margins, tabs, charsets, modes, attributes, cursor and device reports |
| NT-CAP-03 | Color/style | 16/256/truecolor foreground/background, underline styles/colors, bold/dim/italic/blink/inverse/conceal/strike/overline without lossy string mapping |
| NT-CAP-04 | Unicode/text | Grapheme clusters, combining marks, wide cells, emoji sequences, fallback fonts, shaping/BiDi where core-supported, deterministic width policy |
| NT-CAP-05 | Rendering | GPU-backed clipped cell rendering, backgrounds, decorations, cursor styles/blink, damage-only updates, DPI/font-size changes, no fixed column truncation |
| NT-CAP-06 | Keyboard/IME | Printable/control/meta keys, dead/composed keys, IME preedit/commit, app cursor/keypad, kitty keyboard where supported, configurable terminal bindings |
| NT-CAP-07 | Mouse | X10/UTF-8/URXVT/SGR modes, motion/wheel, application capture, explicit modifier override for local selection and link interaction |
| NT-CAP-08 | Selection/clipboard | Character, word, logical-line, wrapped-line, block and select-all; clipboard and Linux primary selection; HTML/plain copy policy; bracketed paste |
| NT-CAP-09 | Scrollback/reflow | Configurable bounded history, primary-screen history, alternate-screen isolation, logical-line reflow, anchor preservation and jump-to-bottom behavior |
| NT-CAP-10 | Search | Forward/backward, case-sensitive/insensitive, literal/regex, wrap, all-match highlights, stable navigation while output arrives |
| NT-CAP-11 | Links/files | OSC 8 hyperlinks, detected URL/path hints, modifier-click/open/copy, cwd-relative paths, untrusted-target confirmation policy |
| NT-CAP-12 | Modern protocols | Focus reporting, synchronized output, OSC palettes/title/cwd, controlled OSC 52, kitty keyboard, notifications/progress where core-supported |
| NT-CAP-13 | Graphics | Kitty graphics and sixel when provided by the selected core; image lifetime, clipping, scroll/resize behavior, memory bounds and disabled-by-policy mode |
| NT-CAP-14 | Sessions | Tabs, splits, rename/title, new-in-cwd, restart, close/kill confirmation, attach/detach, maximized terminal, process-exit state |
| NT-CAP-15 | Profiles/appearance | Shell/argv/cwd/env templates, font/fallback/size, theme/palette, cursor, scrollback, bell and protocol-security settings |
| NT-CAP-16 | Accessibility | Keyboard-only control, screen-reader text/cursor/selection/search exposure, high contrast, non-color cues, reduced motion |
| NT-CAP-17 | Agent/tool integration | Codex, Claude Code, Cursor/local agent CLIs and arbitrary TUIs run unmodified; each supported adapter natively discovers `datum-eda`, standard authenticated Datum MCP, pinned context, scoped authority, and portable workflows per decision 028 |
| NT-CAP-18 | Security | Untrusted escape isolation; explicit OSC 52, URI, paste and process-close policies; no secrets in tracked context/telemetry; no escape-driven GUI mutation |

## 4. Datum CLI, MCP, and agent pipeline

### 4.1 Ordinary processes, first-class context

Agents and tools are launched as ordinary child processes of the user's shell.
Datum does not proxy their terminal protocol, synthesize their UI, or restrict
them to a special command vocabulary. Full-screen agent interfaces must receive
the same PTY, mode, Unicode, color, mouse, clipboard, resize, and signal behavior
as any other TUI.

Each session receives at minimum:

- `DATUM_PROJECT_ROOT`, `DATUM_PROJECT_ID`, and `DATUM_MODEL_REVISION`;
- `DATUM_SESSION_ID`, `DATUM_CONTEXT_ID`, and `DATUM_DISCOVERY`;
- `DATUM_CLI=datum-eda`;
- `DATUM_MCP_ENDPOINT` or an equivalent connection descriptor when MCP is
  available; and
- an explicit refresh command that obtains current selection/model context.

The discovery document contains capabilities and connection metadata, not user
credentials or copied secrets. Long-running agents can refresh context after the
model revision changes. Environment metadata and a launch prompt alone do not
constitute agent integration: `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`
controls native adapter registration, standard MCP primitives, live versus
pinned context, scoped authority, workflow parity, and proof.

### 4.2 One mutation path remains absolute

Direct pipeline means low-friction access, not private authority. Terminal tools
use `datum-eda`, MCP, proposals, and typed engine operations. Datum GUI actions
never generate shell strings and write them into the PTY. Terminal escape output
never invokes an engine operation.

## 5. Interaction contract

- Opening the terminal leaves editor keyboard focus unchanged.
- Clicking the terminal cell rectangle gives it keyboard focus.
- Clicking a tab/control performs that action and gives focus only when the
  resulting behavior expects terminal typing.
- `Escape` first exits transient terminal-owned modes such as search/selection;
  otherwise it returns focus to the editor without closing the terminal.
- Clicking a viewport returns focus to that viewport.
- Terminal focus, OS-window focus, pane focus, and pointer hover are independent.
- When the child requests mouse reporting, an explicit configured modifier
  temporarily gives the pointer to local selection/link behavior.
- IME composition is positioned at the terminal cursor and never routed as
  partial PTY bytes.
- Copy does not send `SIGINT`; plain `Ctrl+C` does when no local copy binding is
  active. Paste supports bracketed-paste mode and warns or confirms according to
  the configured multiline-paste policy.

### 5.1 User-visible operational feedback

Feedback placement follows consequence, not implementation convenience:

- A detached terminal displays a persistent terminal-local `Detached — input
  disabled` state with a reattach affordance. The content surface is read-only;
  typing produces no PTY bytes and never falls back to a hidden line editor.
- An explicit successful copy may display a brief terminal-local confirmation
  and accessibility announcement. Routine copy success may otherwise remain
  silent according to profile policy.
- Clipboard-read/write failure, rejected paste, and paste-security refusal use
  visible warning/error feedback through the notification backbone. A blocking
  confirmation remains local to the initiating terminal.
- PTY write failure displays persistent terminal-local error chrome with
  retry/restart actions. It also publishes structured diagnostic detail to the
  Notices/log surfaces when available.
- Routine narration and historical diagnostics may flow to `ConsoleLaneState`
  for the read-only Command Console. Interaction-blocking state and failed user
  actions may not exist only in that presently invisible sink.

No item above writes into, overlays, or reserves a row in the terminal cell
rectangle. T0-C02 owns only truthful cell geometry; detached lifecycle belongs
to the input/session slices, clipboard behavior to interaction/notification
integration, and PTY failure handling to transport/session lifecycle.

## 6. Security contract

Terminal output is hostile input. The adapter/core boundary must prevent escape
sequences from escaping the terminal surface. OSC 52 is disabled or confirmation-
gated by default for clipboard writes; hyperlink/path opening is user-initiated;
graphics allocations and scrollback are bounded; paste preserves visible user
agency; and closing a live process tree distinguishes detach, SIGHUP, and force
termination.

Datum context files are outside design authority and never journaled. Sensitive
MCP authentication uses process environment or protected runtime descriptors,
not git-tracked project files.

## 7. Verification and release gates

### 7.1 T0 shell-truth gate

The first executable gate launches a deterministic real shell, sends unique
commands through the same focus/input/PTY path used by the product, pumps output
through the terminal core, and proves from renderer-facing state that:

<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C01 -->
1. **T0-C01 — foreign-shell screen authority.** Remove every application-owned
   row and non-PTY grid writer. Route activity, diagnostics, lifecycle messages,
   and GUI command echoes to chrome, the Command Console, notifications, or logs.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C02 -->
2. **T0-C02 — truthful viewport geometry.** Give the terminal cell rectangle its
   own hit target and derive PTY rows/columns from that exact visible rectangle;
   application summaries consume zero cell rows.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C03 -->
3. **T0-C03 — real-shell canary.** Launch a deterministic real shell through the
   production path and prove prompt and command output are visible in order and
   every typed byte reaches the child exactly once.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C03A -->
4. **T0-C03A — responsive output delivery.** Before interaction-regression
   closure, PTY output must wake the waiting GUI without a user-generated event;
   PTY bursts must coalesce to at most one pending event-loop wake so stale wake
   events cannot queue ahead of subsequent keyboard input. GUI-thread drain work
   must be bounded and explicitly rescheduled while backlog remains; activity,
   style, and render projection must be synchronized once per bounded batch rather
   than once per byte/chunk. Deterministic debug and release probes record
   first-output latency, throughput, worst batch time, and backlog completion,
   followed by owner `ls -la` and sustained-output acceptance.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C04 -->
5. **T0-C04 — regression boundary.** Prove workspace shortcuts, Datum telemetry,
   session lifecycle, and diagnostic paths cannot write to or displace terminal
   cells, while cell/canvas clicks transfer focus through the one authority.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C05 -->
<!-- OWNER:TERMINAL-T0-SHELL-TRUTH:T0-C05:T0-ACCEPT -->
6. **T0-C05 — owner acceptance.** The owner runs `ls` and a unique `printf`
   payload in the production Datum terminal and confirms that only the real shell
   screen is visible, input is usable, and focus exits back to the editor.

TF-01 may remain historical evidence for focus-owner extraction, but no issue or
Frontier row may claim the user-visible terminal defect repaired before this gate.

### 7.2 Compatibility matrix

A checked-in machine-readable matrix must name the command, fixture/version,
expected behavior, result artifact, and last verified core revision for:

- `vttest` and `esctest2`, with explicit pass/known-delta counts;
- Unicode grapheme, emoji, combining, wide-cell, shaping and resize corpora;
- bash, zsh, fish, SSH, tmux, less, Vim/Neovim, htop/btop, Python, Git and Cargo;
- Codex, Claude Code, one Cursor-compatible CLI if available, and one local-agent
  TUI;
- OSC 8, OSC 52 security, synchronized output, mouse protocols, kitty keyboard,
  sixel and kitty graphics; and
- selection, clipboard, search, hyperlinks, tabs, splits, profiles, accessibility
  and lifecycle behavior.

Known upstream deltas are allowed only when documented, bounded, and owner-
approved. "Not tested" never counts as parity.

### 7.3 Performance matrix

Release evidence records input-to-present latency, sustained PTY throughput,
frame pacing during output, damage upload volume, memory per scrollback line,
resize/reflow time, startup time, and multi-session scaling. Budgets are fixed
before final verification and run against checked-in deterministic fixtures.

### 7.4 Visual-parity failure disposition

A failing committed visual-parity gate is a product defect even when it predates
the current slice. The defect record must preserve the exact baseline commit,
reproduction command, expected image, actual image, diff image, changed-pixel
ratio, and suspected rendering boundary. Implementers do not use `--bless` to
unblock unrelated work. They either restore the intended output or present the
artifacts for explicit owner review of an intentional replacement. Only that
review authorizes a new golden.

### 7.5 Human acceptance

The owner verifies at least: shell prompt/typing, long output, colors, Vim or
Neovim, tmux, an interactive code agent, copy/paste/selection, search, resize,
tabs/splits, link opening, font zoom, focus handoff, and process close/restart.
Screenshots alone cannot prove terminal correctness, but visual review is
required for clipping, cursor, attributes, selection, search, graphics,
accessibility, and Datum-shell integration.

## 8. Delivery slices

- **T0 Shell truth:** exclusive PTY cell authority, correct content hit target,
  visible-output canary, honest TF claims.
- **T1 Core + transport:** pinned mature core adapter, portable PTY, rival legacy
  state/input removal, independent sessions.
- **T2 Renderer + interaction:** complete attributes/fonts/Unicode/damage,
  focus/input/IME/mouse, selection/clipboard, accessibility foundation.
- **T3 Daily-driver surface:** scrollback/reflow/search/links/graphics,
  tabs/splits/profiles/themes, lifecycle and security UX.
- **T4a Launcher/discovery:** client-native adapters, protected ephemeral
  configuration, version/health checks, lifecycle, and explicit launch.
- **T4b MCP interoperability:** standard stdio broker, optional secured
  loopback transport, tools, resources, templates, subscriptions, and prompts.
- **T4c Context/authority:** immutable pinned context, revision fences, scoped
  capabilities, credentials, audit, and structured stale-state handling.
- **T4d Workflow parity:** canonical portable workflow inventory, checked client
  projections, OSC metadata boundary, and named agent round-trip evidence.
- **T4e Production verification:** full terminal, agent, conformance,
  performance, security, accessibility, and owner acceptance matrix.

Every slice is a bounded tracked execution unit. Only T4e verification closes
the native-terminal epic.
