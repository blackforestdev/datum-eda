# Datum-Owned Terminal Core Implementation Plan

Status: authorized execution plan

Authority: Product Mechanics 022, 027, 029, and 030; the native
terminal and agent-interoperability specifications remain the product contract.

## 1. Outcome

Design, build, integrate, and prove a fully Datum-owned Linux PTY/session layer
and terminal core replacing every capability previously delegated to proposed
Ghostty/Zig and temporary portable-pty paths. The core is first-party Rust,
offline-buildable, dependency-free, bounded, clean-room, and production-real.
This plan does not reduce the full terminal target to a console widget.

## 2. Package DAG

Each package is a stable machine completion step within its owning Frontier
phase bead, with exact acceptance and evidence recorded there. A separate child
bead is created only when a package needs independent defect or re-entry
tracking; the Frontier completion contract remains the sequencing authority.
The conductor serializes writers even when prerequisites permit parallel
read-only work.

### Research and authority

- **DTC-P00 — provenance baseline (complete).** Pin exact standard versions/dates,
  clean-room procedure, requirement IDs, Unicode data/license/generation path,
  black-box witness policy, and inherited-substrate boundary.
- **DTC-P01 — architecture ratification (complete).** Ratify decision 030, the
  clean-room and Unicode standards-data posture, truthful capability identity,
  explicit unsupported-delta process, and the rule that security defaults,
  graphics file-transfer enablement, numerical budgets, golden replacement, and
  release acceptance remain actionable owner boundaries at their owning slice.
- **DTC-P02 — architecture/API preflight (complete).** Freeze first-party crate interfaces,
  module/file map, ownership, limits schema, source-health baseline, migration
  overlap matrix, and exact deletion targets.

### Datum-owned PTY track

- <!-- REQ:TERMINAL-T1-PTY:DTC-P03 --> **DTC-P03 — PTY boundary.** Extract request/event/reader/session-handle and
  Linux pty/spawn/job-control ownership without behavior change.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P04A --> **DTC-P04A — transport-budget owner decision.** Ratify the exact output,
  input, global-drain, and concurrent-session bounds below before P04 execution.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P04B --> **DTC-P04B — launch and I/O.** Arbitrary argv, cwd/env/credentials,
  descriptor hygiene, termios, partial nonblocking I/O, the ratified bounded
  queues/backpressure, inactive-session draining, and typed error surfaces.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05 --> **DTC-P05 — job control.** Foreground process groups, line-discipline control
  characters, pipelines, stopped/continued jobs, resize/SIGWINCH, exact exit,
  terminate/escalate/SIGHUP, and orphan-free teardown.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06 --> **DTC-P06 — session proof.** Concurrent input/output/resize/detach/reattach/
  exit/restart isolation, stress, performance, and dependency/source gates.

#### DTC-P04A owner packet

DTC-P04B introduces the transport's first normative memory, backpressure, and
event-loop work budgets. Decision 030 reserves those numerical budgets to the
project owner. Output remains lossless; saturation stops PTY reads and lets the
kernel backpressure the child. Input admission is nonblocking and atomic: a
request is accepted completely or rejected with typed terminal-local feedback.
Inactive and detached sessions are drained fairly into their own screen/core;
only the active session is visibly projected. These laws are not tuning hints.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L1 --> **P04-L1 — per-session output.** Owner-approved 2026-08-15: 256 output
  chunks of at most 16,384 bytes each, capped at 4,194,304 queued payload bytes,
  plus separately reserved exit and persistent-I/O-error state. At either limit the
  reader stops; bytes are never dropped, truncated, decoded, coalesced, or
  reordered. Lower limits stall sustained producers sooner; higher limits
  increase memory held by every hidden session. The larger burst allowance is
  for sustained build and code-agent output; it is not the logical scrollback or
  model-context limit governed by later terminal-core packages.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L2 --> **P04-L2 — per-session input.** Owner-approved 2026-08-15: at most 64 accepted
  requests and 4,194,304 aggregate pending bytes. The aggregate limit is also the maximum
  single paste/write request. A request that would exceed either limit accepts
  zero bytes, is not input-logged, and returns a typed retryable backpressure
  error. Accepted bytes retain FIFO order and are delivered exactly once. Larger
  payloads require a future bounded streaming-paste or file workflow rather than
  an unbounded in-memory request.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L3 --> **P04-L3 — GUI-turn work.** Recommended: preserve the existing global ceiling
  of 128 output events or 65,536 bytes per event-loop turn, spent work-conservingly
  round-robin across every live session from a persistent fairness cursor. Check
  each session's exit/error state before granting a noisy session another data
  quantum; remaining work schedules exactly one successor wake.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L4 --> **P04-L4 — aggregate session bound.** Recommended: at most 16 concurrent
  terminal sessions. Refuse the seventeenth before PTY allocation with visible
  terminal-local feedback. This permits twice the required eight-session release
  proof while bounding pending payload memory to 32 MiB across output and input.

Approving these limits authorizes only DTC-P04B. It does not approve a dependency,
TERM identity change, P05 job-control behavior, P06 stress closure, later security
or performance budgets, a visual golden, or release acceptance. Any later budget
change requires explicit owner evidence; an agent may not silently tune it.

### Core foundation

- <!-- REQ:TERMINAL-T1-CORE:DTC-P07 --> **DTC-P07 — closed types and limits.** First-party crate; cell/cluster/style/
  color, logical coordinates, cursor/margins/modes/tabs/saved state, replies,
  events, damage, snapshots, and complete checked limit types.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P08 --> **DTC-P08 — streaming parser.** Incremental UTF-8 and ECMA-48 state machine;
  bounded params/intermediates/subparams/control strings; typed actions;
  cancellation, chunk invariance, malformed/oversized recovery.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P09 --> **DTC-P09 — screen model and reducer.** Primary/alternate buffers, edit/
  erase/scroll, delayed wrap, margins, protected cells, save/restore, reset,
  and cell-continuation invariants.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P10 --> **DTC-P10 — DEC/xterm semantics.** SGR/color/underline, charsets/DEC graphics,
  tabs, reports/replies, private modes, palette/title/CWD/bell, synchronized
  output, and exact supported-query behavior.

### Text, history, and interaction semantics

- <!-- REQ:TERMINAL-T1-CORE:DTC-P11 --> **DTC-P11 — Unicode data and text.** Pinned generated property tables,
  extended graphemes, emoji/ZWJ, deterministic width tailoring, combining and
  wide-cell placement, original text preservation, font/shaping boundary, and
  declared BiDi behavior.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P12 --> **DTC-P12 — history and reflow.** Logical lines, hard/soft breaks, bounded
  storage, alternate isolation, stable viewport/cursor/selection/search/link/
  graphics anchors, deterministic trim, and resize reflow.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P13 --> **DTC-P13 — selection and copy.** Grapheme, word, logical/wrapped line, block,
  all scopes; stable endpoints; trailing blank/tab/wrap/newline extraction.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P14 --> **DTC-P14 — search.** Incremental literal/case search plus Datum-owned bounded
  Thompson-NFA regex, stable matches under output/trim/reflow, prompt navigation,
  no backtracking/ReDoS.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P15 --> **DTC-P15 — input protocols.** Legacy and modified keys, application cursor/
  keypad, kitty negotiation stack, mouse families, focus, paste, IME commit
  contract, replies, local override, and coordinate clipping.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P16 --> **DTC-P16 — metadata security.** OSC 8/52/133, links, clipboard requests,
  palette/title/CWD, notifications/progress, URI/paste policy, session scoping,
  rate limits, and proof that escapes cannot invoke Datum operations.

### Extended graphics parity

- <!-- REQ:TERMINAL-T1-CORE:DTC-P17 --> **DTC-P17 — owned binary codecs.** Base64, checksums, Adler-32, zlib/DEFLATE,
  and PNG parse/filter/color/interlace decode with bombs, overflow, malformed
  input, and deterministic resource limits.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P18 --> **DTC-P18 — sixel.** Grammar, raster/repeat/newline, RGB/HLS registers,
  transparency, palette, clipping, scrolling/history placement, teardown, and
  bounds.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P19 --> **DTC-P19 — kitty graphics.** APC grammar, transfer/chunk/query/reply,
  images/placements/z-order/crop/scale/offset/cursor/virtual placement,
  animation/composition/deletion, safe transport posture, history/reflow, and
  bounds.

### Projection, proof, and cutover

- <!-- REQ:TERMINAL-T1-CORE:DTC-P20 --> **DTC-P20 — damage and render snapshot.** Dirty cell/row/scroll/cursor/palette/
  graphics events, immutable iteration, deterministic snapshot schema, and no
  renderer ownership in core.
- <!-- REQ:TERMINAL-T1-CORE:DTC-P21 --> **DTC-P21 — core proof gate.** Datum-authored normative corpus, chunk
  invariance, generational mutation/replay/shrink, hostile streams, reset,
  resource and performance proof for P07–P20.
- **DTC-P22 — session adapter.** PTY bytes to core, replies to PTY, bounded
  scheduling/backpressure, every-session draining, resize, lifecycle, and
  context association.
- **DTC-P23 — GPU renderer.** Backgrounds, glyph clusters, fallback/shaping,
  every text decoration/color, cursor, clipping, damage-only updates, images,
  DPI/font change, and visual proof through renderer-owned types.
- **DTC-P24 — native input and accessibility.** Focus, IME preedit/commit,
  keyboard/mouse/paste, selection/clipboard/search/links and the Linux AT-SPI
  text/caret/selection/event bridge with real screen-reader evidence.
- **DTC-P25 — bounded shadow comparison.** Feed recorded bytes to old and new
  cores only in tests/debug; compare the declared overlap; use normative proof
  for new behavior; no production selector or fallback.
- **DTC-P26 — atomic cutover.** Select the Datum core as sole production
  authority; delete provisional `TerminalScreen`, string/style grid authority
  from `gui-protocol`, lossy renderer, shadow path, and `TERM` overclaim.

### Product completion

- **DTC-P27 — truthful shell identity and daily-driver UX.** Datum terminfo,
  shell integration, tabs/splits/profiles/themes/fonts, search/history, links,
  graphics/security prompts, lifecycle/detach/maximize.
- **DTC-P28 — compatibility proof.** Bash/zsh/fish, SSH, tmux, less, Vim/Neovim,
  htop/btop, Python, Git, Cargo and standards probes on the production build.
- **DTC-P29 — agent pipeline proof.** Codex, Claude Code, Cursor-compatible CLI,
  local-agent TUI, discovery, authenticated MCP, pinned context/authority,
  proposal/review/apply/refresh/resume, and one mutation path.
- **DTC-P30 — production acceptance.** Offline/dependency, source health,
  security, performance, memory, visual, accessibility, lifecycle, eight-session
  scaling, complete FT/NT-CAP evidence matrix, and final owner hands-on review.

## 3. Proof attached to every package

- Stable requirement IDs and primary authority references.
- Datum-authored inputs and expected state/replies/events/damage/resources.
- Whole-stream, one-byte, and deterministic arbitrary-chunk equivalence.
- No panic, unbounded allocation/work, reply amplification, hidden I/O, or
  terminal-to-Datum mutation.
- Seeded std-only mutation generator, minimized replay artifact, and bounded CI
  run; longer local/scheduled soak uses the same format.
- Strict build/clippy/tests, dependency authority, offline Cargo, source health,
  and package-specific performance/resource evidence.
- Independent verifier review before conductor advancement.

## 4. Source-health and ownership

New production files target 300–500 lines and never exceed 700 pre-test lines;
dedicated tests never exceed 700 physical lines. Existing near-limit files
receive no feature growth. Extraction owns real behavior and shrinks its source;
no `include!`, forwarding, continuation, registration, or test-dumping split.

The first-party crate is decomposed by parser, action/reducer, model, text,
history, selection/search, input, metadata/security, graphics/codecs,
damage/snapshot, limits, and proof utilities. PTY and renderer remain separate.

## 5. Conductor transaction

For each package the conductor:

1. Runs the fresh selector and validates prerequisites.
2. Atomically synchronizes the Frontier lease and beads claim.
3. Assigns one writer exact files/IDs/budgets/tests/stop conditions.
4. Permits other agents only read-only research, adversarial review, or isolated
   verification while the writer is active.
5. Verifies diff ownership, dependency/offline state, source health, behavior,
   security, and evidence.
6. Lands one bounded commit, records typed evidence, closes/advances the package,
   regenerates the Frontier, and releases the lease in the same transaction.

Any dependency proposal, standards-data license change, unsupported protocol
delta, security-policy choice, numeric performance budget, golden replacement,
TERM identity change, or release acceptance stops at an actionable owner packet.

## 6. Goal acceptance

The execution goal is achieved only when DTC-P00 through DTC-P30 are complete,
the owned PTY/core are sole production paths, provisional state is deleted,
FT-001..013 and NT-CAP-01..18 contain no unknown/unverified row, named shells/
TUIs/agents pass, all governed gates pass, and final owner acceptance closes
T4e and the terminal epic.

## 7. Planning evidence and authority boundaries

DTC-P00 is evidenced by `DATUM_TERMINAL_CORE_RESEARCH.md`; DTC-P01 by decision
030 and the owner's direction to build a fully first-party replacement; DTC-P02
by this bounded package DAG, decision-022 module limits, migration/cutover law,
and the existing-code inventory recorded in the research baseline. These three
planning packages authorize the conductor to establish the execution goal and
select DTC-P03. They do not pre-approve a later boundary named in section 5.

When one of those boundaries is reached, the canonical completion step must
present the exact question, recommended safe default, consequences, and response
syntax. Work stops before claim or edit until the owner answers. No general
"proceed" instruction can be reused as approval for an unpresented dependency,
license, security, capability, performance, golden, or release decision.
