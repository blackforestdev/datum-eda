# Datum Native Terminal Product Specification

Status: active target contract

Authority: Product Mechanics 005 and 024 as amended by 027, 028, and 029.
Decision 027 controls terminal-emulation quality; decision 028 controls agent
discovery and interoperability; decision 029 controls implementation ownership
and dependency authority.

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

`TerminalCore` is Datum-owned source split into bounded, cohesive modules under
decision 022. It may not wrap, link, vendor, copy, download, or invoke a
third-party terminal implementation or fallback. It owns terminal semantics
and exposes:

- `feed_pty(bytes)` and terminal-generated reply bytes;
- `resize(columns, rows, pixel_width, pixel_height)`;
- immutable render-state/damage iteration;
- cursor, mode, title, cwd, bell, palette, hyperlink, and graphics state;
- scrollback/search and selection-coordinate operations;
- keyboard/mouse/paste/focus encoders where supplied by the core; and
- reset, teardown, and deterministic test snapshots.

Terminal-core structures never cross into engine or `gui-protocol`. Terminal state is
process-local consumer state in `gui-app`; the engine never persists it as design
truth.

The T1b Datum-core implementation closes through these ordered requirements.
T1b builds and proves the core; T2 owns production screen cutover,
renderer/input integration, parity, and deletion of the provisional
screen/parser:

- <!-- REQ:TERMINAL-T1-CORE:CORE-01 --> **CORE-01 — owned architecture and inventory.**
  Define the Datum-owned VT/state module boundary, protocol inventory, data
  ownership, source-health decomposition, and behavior-fixture provenance.
  Reject every external terminal implementation and fallback.
- <!-- REQ:TERMINAL-T1-CORE:CORE-02 --> **CORE-02 — closed core contract.**
  Define Datum-owned input, reply, render/damage, mode/event, selection,
  scrollback/search, and snapshot types. Terminal-private structures remain
  inside `gui-app` and never enter the engine or `gui-protocol`.
- <!-- REQ:TERMINAL-T1-CORE:CORE-03 --> **CORE-03 — semantic implementation.**
  Implement Datum-owned parsing, feed, resize, replies, cursor/modes,
  title/CWD/bell, palette, hyperlink/graphics state, selection coordinates,
  scrollback/search, input encoders, reset, and teardown without retaining the
  provisional screen as a rival core.
- <!-- REQ:TERMINAL-T1-CORE:CORE-04 --> **CORE-04 — deterministic corpus.**
  Drive the core with checked-in Datum-authored VT byte streams and assert
  deterministic snapshots for cells/styles, replies, damage, Unicode width,
  modes, alternate screen, scrollback, title/CWD/bell, hyperlinks, and governed
  graphics state.
- <!-- REQ:TERMINAL-T1-CORE:CORE-05 --> **CORE-05 — gate and T2 handoff.** Pass
  dependency-authority, module-boundary, snapshot, source-health, governance,
  and strict build gates; record the exact T2 production-cutover seam. Do not
  claim the provisional screen retired until T2 parity and deletion evidence
  land.

### 2.2 Session transport

Each terminal tab or split binds one `TerminalSession` to one `TerminalCore`.
Datum owns the Linux PTY transport over operating-system PTY interfaces. It must
preserve controlling-terminal setup, process groups, signals, resize, exit
status, inherited user credentials/environment, and independent concurrent
sessions without a third-party PTY implementation.

The T1a transport replacement closes through these ordered implementation
requirements:

- <!-- REQ:TERMINAL-T1-PTY:PTY-01 --> **PTY-01 — owned boundary and inventory.**
  Inventory the restored Linux PTY ownership and establish a Datum-owned
  transport boundary that exposes process/session operations without terminal
  cell semantics or third-party code.
- <!-- REQ:TERMINAL-T1-PTY:PTY-02 --> **PTY-02 — complete transport swap.** Move
  allocation, spawn, read, write, resize, cwd/environment, inherited credentials,
  and arbitrary executable/argv launch through the adapter while preserving Datum
  bootstrap context.
- <!-- REQ:TERMINAL-T1-PTY:PTY-03 --> **PTY-03 — process semantics.** Prove real
  pipelines, process groups, Ctrl-C, explicit termination, shell exit status, and
  child-exit reporting through production-path tests.
- <!-- REQ:TERMINAL-T1-PTY:PTY-04 --> **PTY-04 — session isolation.** Prove that
  concurrent tabs remain independent across input, output, resize, detach,
  reattach, exit, and teardown.
- <!-- REQ:TERMINAL-T1-PTY:PTY-05 --> **PTY-05 — harden and lock.** Complete the
  Datum-owned PTY boundary, add drift guards that keep cell parsing out of
  transport, and pass terminal, dependency-authority, source-health,
  governance, and platform-aware integration gates.

PTY-01 freezes the recovery inventory below:

| Responsibility | Current owner after PTY-01 | PTY-02 destination |
|---|---|---|
| PTY allocation, slave setup, controlling TTY, raw resize | `terminal_process.rs` Linux PTY helpers | bounded Datum transport module |
| shell argv, cwd, environment and Datum discovery injection | `terminal_process.rs::spawn_terminal_process` | Datum transport launch request |
| master reader/writer and child wait threads | `terminal_process.rs` | Datum-owned session handles |
| input logging and terminal event publication | `TerminalSession::write_bytes` and `terminal_process.rs` | retained Datum wrappers around owned handles |
| process-group interrupt/terminate and exit code | `TerminalSession` plus the wait thread | Datum process/session adapter with platform proof |
| tab attachment, restart, dimensions and independent screen state | `TerminalSessionRegistry` | unchanged registry over the new transport session |
| VT parsing, cells, selection, rendering and chrome | `TerminalScreen`, protocol and renderer modules | explicitly outside transport; later TerminalCore/T2 work |

PTY-02 makes the table's destination column the production path. The bounded
Datum transport owns allocation, child, reader, writer, resize, process-group,
and teardown behavior. VT bytes still flow to `TerminalScreen` only through the
existing event consumer.

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

Every row is required for epic closure and must be implemented and evidenced by
Datum-owned terminal source.

| ID | Capability | Required outcome |
|---|---|---|
| NT-CAP-01 | Shell/process | Real login/non-login shell templates, arbitrary executable/argv, job control, signals, pipelines, long-running processes, SSH and tmux |
| NT-CAP-02 | VT state | xterm/DEC-compatible primary/alternate screen, margins, tabs, charsets, modes, attributes, cursor and device reports |
| NT-CAP-03 | Color/style | 16/256/truecolor foreground/background, underline styles/colors, bold/dim/italic/blink/inverse/conceal/strike/overline without lossy string mapping |
| NT-CAP-04 | Unicode/text | Grapheme clusters, combining marks, wide cells, emoji sequences, fallback fonts, shaping/BiDi where implemented, deterministic width policy |
| NT-CAP-05 | Rendering | GPU-backed clipped cell rendering, backgrounds, decorations, cursor styles/blink, damage-only updates, DPI/font-size changes, no fixed column truncation |
| NT-CAP-06 | Keyboard/IME | Printable/control/meta keys, dead/composed keys, IME preedit/commit, app cursor/keypad, kitty keyboard where supported, configurable terminal bindings |
| NT-CAP-07 | Mouse | X10/UTF-8/URXVT/SGR modes, motion/wheel, application capture, explicit modifier override for local selection and link interaction |
| NT-CAP-08 | Selection/clipboard | Character, word, logical-line, wrapped-line, block and select-all; clipboard and Linux primary selection; HTML/plain copy policy; bracketed paste |
| NT-CAP-09 | Scrollback/reflow | Configurable bounded history, primary-screen history, alternate-screen isolation, logical-line reflow, anchor preservation and jump-to-bottom behavior |
| NT-CAP-10 | Search | Forward/backward, case-sensitive/insensitive, literal/regex, wrap, all-match highlights, stable navigation while output arrives |
| NT-CAP-11 | Links/files | OSC 8 hyperlinks, detected URL/path hints, modifier-click/open/copy, cwd-relative paths, untrusted-target confirmation policy |
| NT-CAP-12 | Modern protocols | Focus reporting, synchronized output, OSC palettes/title/cwd, controlled OSC 52, kitty keyboard, and governed notifications/progress behavior |
| NT-CAP-13 | Graphics | Datum-owned kitty graphics and sixel slices; image lifetime, clipping, scroll/resize behavior, memory bounds and disabled-by-policy mode |
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
- A programmatic `run in terminal` or typed command handoff may open the dock
  and write its command, but it preserves the current keyboard owner. Follow-up
  typing begins only after the user deliberately clicks the terminal tab or
  cell rectangle; command injection is not an implicit focus gesture.
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
   than once per byte/chunk. Raw foreign-shell Enter must never infer a Datum
   mutation or schedule project/workspace reload; only an explicitly typed Datum
   mutation handoff may request synchronization. Deterministic debug and release
   probes record first-output latency, throughput, worst batch time, backlog
   completion, and immediate post-prompt input, followed by owner `ls -la` and
   sustained-output acceptance.
<!-- EVIDENCE:TERMINAL-T0-SHELL-TRUTH:T0-C03A-OWNER-ACCEPTED -->
Owner acceptance on 2026-08-14 confirmed that the final raw-Enter repair removes
the post-command hang and restores responsive successive command entry.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C04 -->
5. **T0-C04 — regression boundary.** Prove workspace shortcuts, Datum telemetry,
   session lifecycle, and diagnostic paths cannot write to or displace terminal
   cells, while cell/canvas clicks transfer focus through the one authority.
<!-- REQ:TERMINAL-T0-SHELL-TRUTH:T0-C05 -->
<!-- OWNER:TERMINAL-T0-SHELL-TRUTH:T0-C05:T0-ACCEPT -->
6. **T0-C05 — owner acceptance.** The owner runs `ls` and a unique `printf`
   payload in the production Datum terminal and confirms that only the real shell
   screen is visible, input is usable, and focus exits back to the editor.
<!-- EVIDENCE:TERMINAL-T0-SHELL-TRUTH:T0-C05-OWNER-ACCEPTED -->
Owner acceptance on 2026-08-14 confirmed a fresh build displayed normal `ls`
output and one unique `printf` payload, returned a usable shell prompt without
Datum-owned terminal rows, and returned keyboard focus to the Board editor.

TF-01 may remain historical evidence for focus-owner extraction, but no issue or
Frontier row may claim the user-visible terminal defect repaired before this gate.

### 7.2 T1 input-model collapse

The rolling foreign shell has one input ingress. The legacy dock line editor may
remain only as explicitly scoped terminal chrome (for example, tab rename); it
must never become a second shell input buffer.

<!-- REQ:TERMINAL-T1-INPUT:TI-01 -->
1. **TI-01 — inventory and boundary.** Inventory every terminal input/cursor
   state field and line-edit call site. Classify each as PTY/TerminalCore input,
   terminal-chrome editing, or dead legacy state, and make that classification
   explicit in production structure and regression coverage.

The TI-01 production inventory is closed as follows:

| State or call-site family | Classification | Production boundary |
|---|---|---|
| `terminal_input` encoders, `terminal_accepts_raw_input`, `write_foreign_shell_bytes`, PTY replies | PTY/TerminalCore input | Encoded bytes go directly to the attached PTY; no GUI text buffer participates. |
| `screen_cursor_row`, `screen_cursor_col`, visibility/style and terminal modes | PTY/TerminalCore projection | Written only from interpreted terminal state; these fields are not text-entry cursors. |
| `rename_session_id`, `rename_input`, `rename_cursor`, and `*_terminal_rename_*` edit calls | terminal-chrome editing | The tab-label editor is explicitly named and must never call the foreign-shell writer. |
| terminal scrollback copy | read-only terminal observation | Clipboard export reads the PTY-derived grid and is not an input model. |
| generic `terminal.input` / `terminal.cursor`, generic `*_dock_input` calls, line completion/history, and non-rename buffered submit | dead legacy state | Generic fields/names are prohibited by the convergence guard; remaining no-op/unreachable branches are deleted by TI-02/TI-03. |

`check_gui_agent_terminal_convergence.py` enforces the named protocol fields,
rejects generic terminal input/cursor state and dock-editor call sites, and proves
that the chrome rename insertion boundary cannot call the foreign-shell writer.
<!-- REQ:TERMINAL-T1-INPUT:TI-02 -->
2. **TI-02 — one shell input model.** Attached sessions route keyboard, composed
   text/IME, paste, and terminal protocols only to the PTY/TerminalCore path.
   Detached sessions are read-only and expose an explicit reattach affordance;
   typing while detached must not accumulate in a hidden buffer. Chrome-local
   rename editing remains isolated and can never submit bytes to the shell.
<!-- REQ:TERMINAL-T1-INPUT:TI-03 -->
3. **TI-03 — deletion and proof.** Delete the rival shell line-editor paths and
   dead state, then prove attached exact-once delivery, detached zero-byte
   behavior plus reattach recovery, and rename isolation through production-path
   tests and the terminal convergence guard.

### 7.3 Compatibility matrix

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

### 7.4 Performance matrix

Release evidence records input-to-present latency, sustained PTY throughput,
frame pacing during output, damage upload volume, memory per scrollback line,
resize/reflow time, startup time, and multi-session scaling. Budgets are fixed
before final verification and run against checked-in deterministic fixtures.

### 7.5 Visual-parity failure disposition

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
- **T1 Core + transport:** Datum-owned terminal core and PTY, rival provisional
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
