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
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05A --> **DTC-P05A — lifecycle owner decision.** Ratify live-tab close behavior,
  teardown signals and deadlines, app-shutdown posture, owned-process boundary,
  and bounded Linux process-session discovery before job-control execution.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05B --> **DTC-P05B — job control.** Foreground process groups, line-discipline control
  characters, pipelines, stopped/continued jobs, resize/SIGWINCH, exact exit,
  ratified terminate/escalate/SIGHUP behavior, and orphan-free teardown.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05C --> **DTC-P05C — agent-session usability repair.** Restore terminal-screen focus
  entry under child mouse reporting, terminal-specific governed glyph coverage,
  and bounded shaped-text caching under continuously changing agent output.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05E --> **DTC-P05E — application focus and hit-ownership convergence.** Clip every
  editor-scene hit region to its visible viewport, prevent board/review targets
  from shadowing terminal chrome, and make one application-level focus authority
  govern terminal selection, mouse reporting, status projection, and Tab routing.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05F --> **DTC-P05F — Claude completion compatibility repair.** Distinguish
  Claude's kitty-keyboard push, pop, and query control sequences from the legacy
  CSI cursor-restore command; prove literal HT delivery and visible completion in
  an actual documented Claude completion context without claiming full kitty
  keyboard support. Depends on DTC-P05E.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05D --> **DTC-P05D — owner production retest.** Accept a fresh-build Claude session
  only after click focus, Tab completion, box/Powerline glyphs, sustained output,
  and post-output typing remain usable without a lockup. Depends on DTC-P05F.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05G --> **DTC-P05G — terminal cell-metric and owner-QA lifecycle convergence.**
  Make shaped terminal ink/advance, logical cell geometry, ANSI style spans,
  cursor geometry, hit testing, and PTY columns consume one measured metric;
  shape each styled row once so color changes cannot create gaps or drift, and
  make a naturally exited selected shell remove its tab without a second close gesture.
  Depends on DTC-P05D.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05H --> **DTC-P05H — ordered terminal session tab projection.**
  Project every owned terminal session into one stable left-to-right top-strip
  tab, append new sessions after existing sessions, keep the active session
  visually selected, and place the new-session affordance after the final tab.
  Depends on DTC-P05G.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P05I --> **DTC-P05I — lean terminal session chrome.**
  Remove the redundant persistent sessions menu, duplicate session labels, and
  duplicate new/rename/restart/close controls; reclaim its cell row while
  preserving the top tab strip, shortcuts, and contextual teardown safeguards.
  Depends on DTC-P05H.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06A --> **DTC-P06A — proof-budget owner decision.** Ratify the exact proof tiers,
  session load, latency, throughput, resource, storage, soak, platform, and evidence
  budgets before timed transport assertions are implemented.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06B --> **DTC-P06B — session isolation.** Prove concurrent input/output/resize,
  inactive-tab drain and reactivation, exit, termination, and verified restart
  isolation across eight real sessions. Datum has no detached-PTY mode.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06C --> **DTC-P06C — stress and resources.** Prove bounded queue saturation,
  persistent fairness, lifecycle churn, maximum-session refusal, and absence of
  cross-session bytes, owned-session survivors, or descriptor/thread growth.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06D --> **DTC-P06D — measured Linux proof.** Run the owner-ratified release-build
  latency, throughput, memory, resize, lifecycle, and long-session soak matrix
  with reproducible machine-readable evidence.
- <!-- REQ:TERMINAL-T1-PTY:DTC-P06E --> **DTC-P06E — transport closure.** Ratchet the Linux platform, offline,
  dependency-authority, source-health, isolation, stress, and performance gates;
  synchronize evidence and close the owned-transport slice.

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
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L3 --> **P04-L3 — GUI-turn work.** Owner-approved 2026-08-15: preserve the existing
  global ceiling of 128 output events or 65,536 bytes per event-loop turn, spent work-conservingly
  round-robin across every live session from a persistent fairness cursor. Check
  each session's exit/error state before granting a noisy session another data
  quantum; remaining work schedules exactly one successor wake.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P04A:P04-L4 --> **P04-L4 — aggregate session bound.** Owner-approved 2026-08-15: at most 16
  concurrent terminal sessions. Refuse the seventeenth before PTY allocation with visible
  terminal-local feedback. This permits twice the required eight-session release
  proof while bounding pending payload memory to 128 MiB across the approved
  4 MiB output and 4 MiB input allowances.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P04A-OWNER-APPROVED -->
The project owner approved P04-L1 at 256 chunks / 16 KiB / 4 MiB output,
P04-L2 at 64 requests / 4 MiB aggregate and single input, P04-L3 at the
existing 128-event / 64 KiB fair global GUI-turn drain, and P04-L4 at 16 live
sessions. These dispositions authorize DTC-P04B only under the lossless,
nonblocking, atomic-admission, fair-drain laws above.

Approving these limits authorizes only DTC-P04B. It does not approve a dependency,
TERM identity change, P05 job-control behavior, P06 stress closure, later security
or performance budgets, a visual golden, or release acceptance. Any later budget
change requires explicit owner evidence; an agent may not silently tune it.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P04B-VERIFIED -->
DTC-P04B landed in `bd464f1b600b10e321bfe717c3537ee7dd3029ba`.
Datum's owned Linux transport now provides literal argv/cwd/environment launch,
descriptor and canonical-termios hygiene, lossless bounded nonblocking output,
atomic bounded input, typed launch/runtime failures, and work-conserving fair
draining across active and inactive sessions. Deterministic tests cover every
owner-approved P04-L1 through P04-L4 bound, partial/EINTR/EAGAIN/HUP/EIO paths,
closed-stdio spawning, control priority, persistent round-robin order, and exact
byte preservation. No dependency, TERM claim, or DTC-P05 job-control behavior
was introduced.

#### DTC-P05A owner packet

DTC-P05B must follow native Linux foreground-TTY semantics, but decisions 005
and 030 leave close UX, teardown security posture, numerical deadlines, and
bounded process ownership to the project owner. A code agent must not claim or
edit DTC-P05B until P05-O1 through P05-O6 are approved. The selector presents
these decisions one at a time.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O1 --> **P05-O1 — live-tab close UX.** Owner-approved 2026-08-15: closing an
  already-exited tab is immediate. Clicking a live tab's close control or pressing
  `Ctrl+Shift+W` arms terminal-local confirmation without sending a signal. The
  user confirms by clicking `Terminate`, repeating `Ctrl+Shift+W`, or typing
  `yes` and pressing Enter; Escape or `Cancel` disarms it. The tab stays visibly
  `Terminating` until Datum verifies that every process in its owned Linux
  terminal session is gone, with an explicit `Force Kill` affordance if graceful
  teardown stalls. Datum provides no detached-PTY mode and never leaves an
  unattached local PTY session running; deliberately detached tmux servers remain
  under tmux authority. `Ctrl+C` remains the native PTY interrupt byte and is
  never overloaded as close confirmation.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O2 --> **P05-O2 — graceful termination.** Owner-approved 2026-08-15: Terminate
  atomically enters `Terminating`, rejects new input/resize, sends SIGHUP to the controlling
  session leader and every verified process group still belonging to the owned
  Linux terminal session, pairs it with SIGCONT so stopped jobs can react, and
  continues draining final output for 2,000 ms.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O3 --> **P05-O3 — escalation.** Owner-approved 2026-08-15: after the HUP grace, send
  SIGTERM plus SIGCONT to every still-owned process group; after another 2,000 ms
  send SIGKILL, then allow 2,000 ms for reap and empty-session verification. Mark
  `Closed` only after the leader is reaped and no owned member remains; otherwise
  retain visible `TerminationFailed` state with the surviving identities.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O4 --> **P05-O4 — app shutdown.** Owner-approved 2026-08-15: tear down sessions concurrently
  under one 6,000 ms global deadline. Datum exits only after every owned session
  is verified empty. If the deadline expires with survivors, block the controlled
  application exit in visible `TerminationFailed`, durably report exact
  PID/PGID/session identities, and offer `Retry Termination` or `Cancel Shutdown`;
  never silently detach and provide no normal `Exit Anyway` path.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O5 --> **P05-O5 — ownership boundary.** Owner-approved 2026-08-15: Datum owns the original
  Linux terminal session and process groups that remain members of it. A process
  that deliberately daemonizes into another session is outside that ownership;
  remote SSH jobs and detached tmux servers follow their remote/server policy.
- <!-- OWNER:TERMINAL-T1-PTY:DTC-P05A:P05-O6 --> **P05-O6 — discovery bound.** Owner-approved 2026-08-15: inspect at most 4,096 Linux
  process members and 4,096 distinct process groups per terminal session per
  escalation scan. Exhaustion stops before signaling an incomplete set, leaves
  visible `TerminationFailed`, and never truncates then claims orphan-free
  closure.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O1-OWNER-APPROVED -->
The project owner approved guarded terminate-only tab closure with no detached
Datum PTY sessions, including click, repeated `Ctrl+Shift+W`, and typed
`yes`+Enter confirmation paths. This evidence approves P05-O1 only.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O2-OWNER-APPROVED -->
The project owner approved the 2,000 ms SIGHUP-plus-SIGCONT graceful phase across
every verified process group in the owned Linux terminal session, with input and
resize closed and final output still draining. This evidence approves P05-O2 only.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O3-OWNER-APPROVED -->
The project owner approved the bounded six-second escalation ladder: 2,000 ms
SIGHUP grace, 2,000 ms SIGTERM-plus-SIGCONT grace, and 2,000 ms for SIGKILL,
reap, and empty-session verification. Unverified survivors remain visibly
`TerminationFailed`. This evidence approves P05-O3 only.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O4-OWNER-APPROVED -->
The project owner approved concurrent application-shutdown teardown under one
global 6,000 ms deadline. Controlled exit is blocked until every owned terminal
session is verified empty; survivors produce visible `TerminationFailed` with
exact identities and Retry/Cancel actions, never silent detach or Exit Anyway.
This evidence approves P05-O4 only.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O5-OWNER-APPROVED -->
The project owner approved terminal teardown ownership of every live process
whose Linux session ID remains the terminal's original session ID, across all
of its process groups. A process that deliberately creates a different session
is outside Datum ownership; detached tmux servers and remote SSH jobs remain
under their server's lifecycle authority. This evidence approves P05-O5 only.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05A:P05-O6-OWNER-APPROVED -->
The project owner approved at most 4,096 observed Linux process members and
4,096 distinct process groups per terminal session per escalation scan.
Exhaustion stops before signaling an incomplete set, leaves visible
`TerminationFailed`, and never truncates discovery then claims orphan-free
closure. This evidence approves P05-O6 and completes DTC-P05A.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05B-VERIFIED -->
DTC-P05B landed in `fdd64469bb53b038f0655252af06bb2286498c16`
with final queued-input cancellation in
`f7fc50f57fbd0e4bf6e1a1d9c2d8db1291865895`.
Datum now routes native line-discipline control bytes to the foreground job,
preserves stopped/continued pipelines and kernel `SIGWINCH`, reports exact
code/signal/core exit identity only after final output, and applies the
owner-ratified HUP/TERM/KILL lifecycle to the complete bounded Linux session.
Close confirmation, Force/Retry/Cancel, concurrent application shutdown,
fail-closed process discovery, deliberate new-session exclusion, and
reader/writer/master-FD completion barriers are regression- and
drift-guarded. No dependency or terminal capability identity changed.

#### DTC-P05C agent-session usability repair

The owner's production screenshot exposed three coupled defects that must be
repaired before transport stress proof: a mouse-aware child could consume the
terminal activation click before Datum changed keyboard focus; terminal cells
used an IBM Plex Mono UI face without box-drawing/Powerline coverage; and the
renderer retained every unique shaped text buffer forever, making animated
agent output progressively slower and larger. DTC-P05C restores the already
ratified terminal-entry law, applies the owner-approved decision 031
JetBrains Mono exception to terminal cells only, and bounds cache residency by
active render generations. The exact internally-versioned 2.305 snapshot was
already vendored; its
applicable copyright/OFL notice and hash-pinned provenance ship beside it. This
adds no package, downloaded asset, terminal dependency, or new interaction
policy.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05C-VERIFIED -->
DTC-P05C landed in `30b5d7ddcdc90ac0669b826cc5b22eab1cd7e4e9`.
Production focus-before-mouse-report routing, shared terminal geometry,
per-session bounded output batching, output-before-exit ordering, one-record
batch audit, terminal-only glyph shaping, and two-generation shaped-buffer
residency are regression- and drift-guarded. Decision 031 records the owner's
terminal typography exception and the exact adjacent OFL/provenance boundary.

#### DTC-P05E application focus and hit-ownership convergence

The DTC-P05D production retest was rejected on 2026-08-16 because clicking the
terminal did not make Datum consistently recognize it as the selected surface;
Tab could still cycle Board/Schematic panes. The dogfooded Claude session
recorded its code-reading diagnosis in
`research/TERMINAL_FOCUS_TAB_BUG_2026-08-16.md`. That evidence identifies a
second concrete focus-entry defect: editor overlay hit regions are visually
clipped but can extend beyond their pane, are inserted after terminal chrome,
and therefore can win reverse-order hit testing inside the terminal dock.

DTC-P05E has this bounded scope:

1. Add a positive-area rectangle-intersection primitive and clip every hit
   region emitted by the board/review scene to its owning scene viewport,
   dropping empty intersections. Visible clipping and hit ownership must agree.
2. Prove terminal screen and chrome hit targets cannot be shadowed by authored
   objects or review actions under adversarial camera pan/zoom, dock height,
   scale, or overlay placement.
3. Make selected application surface explicit and singular: editor selection
   retains its focused `PaneId`, terminal selection is equally representable,
   and transient overlays remain exclusive owners. `PaneContent` remains
   Board/Schematic because Terminal is a dock surface, not an editor document.
4. Make terminal hit entry, status projection, mouse-report eligibility,
   cursor/focus presentation, raw PTY routing, Tab/Shift+Tab handling, and focus
   exit consume that same application-level authority.
5. Add production-path regressions for editor focus -> terminal screen click ->
   terminal selection -> Tab/Shift+Tab bytes, including mouse-reporting and
   non-mouse-reporting children, plus outside-screen and editor-return cases.
6. Preserve the DTC-P05C output batching, glyph, cache, job-control, queue,
   lifecycle, and no-dependency boundaries unchanged.

Terminal text selection and arbitrary-range copy are not falsely claimed by
this repair. The current whole-scrollback copy and focus-gated paste behavior
remain provisional; full selection/clipboard semantics stay in the scheduled
TerminalCore/input work and must be tracked separately from this focus defect.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05E-VERIFIED -->
DTC-P05E landed in `6bffa395f7539ef5394a445ee8ba95e74da7c2b5`.
Board/review hits are clipped to the same visible viewport used for rendering,
and adversarial camera tests prove they cannot shadow terminal screen geometry.
`ApplicationFocus` is now the single editor-pane, Terminal, or Overlay owner
consumed by click entry, mouse reporting, cursor/status projection, and key
routing. Production-path tests prove terminal clicks route Tab and Shift+Tab to
the PTY without editor pane cycling for mouse-aware and ordinary children.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05D-OWNER-REJECTED -->
The owner's first production rejection and the dogfooded Claude diagnosis
authorized DTC-P05E execution. The second production rejection on 2026-08-16
confirmed focus now enters the terminal but reported that completion still did
not work in Claude Code 2.1.233. The session audit log proves Datum delivered two
literal HT bytes and Claude rendered a response to the first; however, the same
Claude stream contains `CSI > 1 u`, `CSI < u`, and `CSI ? u`. The provisional
parser currently treats every CSI `u`, including those private/intermediate
forms, as legacy cursor restore. That can move the cursor to stale saved state
while Claude paints or accepts a completion even though the key reached the PTY.

#### DTC-P05F Claude completion compatibility repair

DTC-P05F is a bounded compatibility correction, not an early claim of the full
DTC-P15 kitty-keyboard protocol:

1. Restrict legacy CSI `u` cursor restore to its parameter-free grammar. Kitty
   keyboard push (`CSI > flags u`), pop (`CSI < number u`), and query
   (`CSI ? u`) must never mutate cursor or screen state.
2. Continue advertising no kitty-keyboard support in the provisional core: do
   not reply to the query, track a mode stack, or change key encoding here.
   Claude's requested flag `1` retains literal HT for Tab under the normative
   protocol, so Datum must keep sending exactly `0x09`.
3. Add chunk-invariant regressions using the observed Claude control stream to
   prove push/pop/query cannot restore a stale cursor, while parameter-free CSI
   `u` retains its existing restore behavior.
4. Add a production-path completion fixture with a real completion candidate:
   enter terminal focus, display a candidate, deliver one HT, and prove the
   child response is parsed at the intended cursor without pane cycling or
   cross-session leakage.
5. Re-run DTC-P05D with a documented Claude context that actually offers a
   completion: an `@` file suggestion, a `/` command menu, a generated prompt
   suggestion, or shell-mode history/path completion. Pressing Tab on an empty
   prompt without an active candidate is not a valid completion assertion.
6. Preserve the P04 transport budgets, DTC-P05B lifecycle barriers, DTC-P05C
   batching/font/cache repairs, DTC-P05E focus authority, dependency boundary,
   and the separately scheduled complete kitty-keyboard work unchanged.

The official kitty keyboard protocol specifies that enhancement flag `1`
disambiguates escape codes but explicitly retains legacy bytes for Enter, Tab,
and Backspace. The official Claude interactive reference documents Tab as an
accept action only where a suggestion or completion is active. These behavioral
references guide clean-room tests; no third-party source or dependency enters
Datum.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05F-VERIFIED -->
DTC-P05F landed in `9f3aee1d5a6ea5d57d5d5d5ac0468c8e49d5c879`.
Parameterized kitty-keyboard CSI `u` controls are now state-neutral in the
provisional parser, while parameter-free CSI `u` retains legacy cursor restore.
The observed Claude 2.1.233 push/query/pop stream is covered whole and across
arbitrary chunk boundaries, and the convergence guard rejects any return to the
broad cursor-restore match. A fresh locked/offline release binary was built for
the corrected hands-on completion-context retest.

<!-- OWNER:TERMINAL-T1-PTY:DTC-P05D:DTC-P05D -->
<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05D-OWNER-ACCEPTED -->
DTC-P05D was owner-approved on 2026-08-16 after hands-on testing confirmed that
the production terminal can be focused and Tab completion works in a Claude
session. The owner explicitly reported that several other terminal issues
remain outside this acceptance boundary. This approval therefore closes only
the DTC-P05D focus/completion gate; it neither resolves nor closes those
separately scoped defects.

#### DTC-P05G terminal cell-metric convergence

The owner's post-acceptance screenshot on 2026-08-16 exposed a distinct
renderer defect: the Claude input row contains expanding gaps between styled
fragments and its cursor is displaced from the visible end of the text. Datum
currently shapes terminal runs at 11 px while cursor, fragment origins, hit
testing, PTY columns, and screen width use a fixed 7.9 × 16 px logical cell.
Independent styled runs therefore restart on logical-cell boundaries that do
not equal the preceding shaped advance, and cursor error accumulates across the
row.

DTC-P05G has this bounded scope:

1. Establish one terminal font/cell metric consumed by terminal shaping,
   logical geometry, ANSI style placement, cursor placement, hit testing, and
   PTY resize authority.
2. Shape each terminal row in one rich-text buffer so SGR color changes retain
   their metadata without restarting glyph positions at fractional cell edges.
3. Prove the cursor begins at the exact next cell after the visible prompt and
   remains aligned across representative long Claude input rows and the
   governed 1.0, 1.25, 1.5, and 2.0 surface-scale matrix.
4. Preserve the governed JetBrains Mono whole-run face, DTC-P05C cache bounds,
   DTC-P05E focus authority, DTC-P05F parser correction, P04 transport budgets,
   and the no-dependency boundary.
5. Treat a natural exit from the selected shell as terminal-close intent: after
   exact final output and presentation barriers complete, remove its tab without
   requiring a second CLOSE action or sending another signal. Preserve inactive
   exited tabs so their exact outcome remains reviewable.
6. Keep full grapheme, combining, wide-character, emoji, reflow, and immutable
   TerminalCore cell semantics in their already scheduled DTC-P07..P23 slices;
   this repair must not claim those broader capabilities.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05G-INITIAL-METRIC-REPAIR -->
The initial DTC-P05G implementation derived the governed JetBrains Mono render
size from the shared 7.9 px logical cell width. Real glyphon shaping proved
long ASCII advance and split-style contiguity, but owner QA on 2026-08-16
rejected closure because the focused full-cell cursor remained visually fused
to the preceding glyph. Pixel inspection of the supplied screenshot confirms
that the final slash ends in the preceding logical cell and the cursor occupies
the correct next cell; the remaining defect is cursor presentation rather than
PTY text or parser-column corruption. DTC-P05G is reopened to preserve the
logical cell while providing an unambiguous visual separation and a regression
covering the exact colored shell-prompt and trailing-slash case. Full Unicode
cell semantics remain explicitly scheduled rather than being overstated here.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05G-REOPENED-AUTOMATED -->
The first reopened attempt inset cursor paint by one pixel; the second reduced
JetBrains Mono ink to 12 px and added explicit tracking while preserving the
governed 7.9 px advance. Owner QA rejected both because the screenshot defect
persisted specifically at the green/blue/yellow/default prompt boundaries. The
root cause was independent shaping: each SGR fragment became a separate glyphon
buffer positioned at a fractional cell origin, so raster rounding restarted at
every color boundary even though total run-width tests were correct. Datum now
builds one rich-text buffer for the entire visible row. ANSI spans set glyph
colors inside that buffer and cannot restart glyph positioning. A real shaping
regression proves the plain and colored prompt have byte-for-byte identical
glyph start/end/x/advance values while retaining color overrides. Smaller ink,
explicit tracking, PTY bytes, parser columns, cursor reports, hit testing, and
PTY dimensions remain unchanged. A natural exit from the selected shell marks
the fully presented session for automatic tab removal; inactive exited sessions
remain reviewable. The convergence mutation gate requires whole-row rich
shaping, prompt/cursor, and natural-exit proofs. DTC-P05G remains open until a
fresh release binary passes owner visual acceptance.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05G-HIDPI-CURSOR-REPAIR -->
The third owner screenshot confirmed that whole-row shaping repaired the yellow
shell prompt delimiter but the magenta cursor still lagged several cells behind
the final `$ `. The captured runtime requested a 1280 × 768 logical window and
received a 1344 × 806 surface, establishing a 1.05 Wayland scale. Datum was
multiplying every terminal `TextRun` size by that surface scale even though the
terminal cursor, mouse mapping, PTY columns, and shared screen geometry retain
the fixed 7.9 × 16 device-pixel cell metric. Across the observed 72-cell prompt,
that five-percent double scaling accumulates roughly 28 pixels of cursor drift,
matching the rejected image. Terminal cell runs are now excluded from the
generic GUI text-size scale transform; ordinary GUI chrome continues to scale.
An exact replay of the colored bash prompt proves ANSI bytes leave the model
cursor after the `$` and trailing space, while a 1.0/1.25/1.5/2.0 prepared-scene
matrix proves shaped prompt width and cursor-grid x remain identical. The
convergence mutation gate rejects restoring terminal glyph-only HiDPI scaling.
DTC-P05G remains open for fresh-build owner visual acceptance of this repair.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05G-OWNER-ACCEPTED -->
Owner QA on 2026-08-16 accepted the fresh-build HiDPI correction and confirmed
that the terminal cursor issue is resolved. DTC-P05G therefore closes with
implementation revision `1877d5188680a622fe780629e477ad4502887d94` and its
governed parser, shaping, scale-matrix, mutation, and natural-exit proofs.

#### DTC-P05H ordered terminal session tab projection

The same QA pass exposed a separate session-chrome defect: the session registry
appends new sessions in correct creation order, but the top dock strip ignores
that ordered projection and redraws only the active title in the first tab
rectangle. Each new session therefore appears to replace the far-left tab.
DTC-P05H renders the protocol `terminal.tabs` sequence directly, assigns every
tab its own session activation hit target, preserves registry order, and seats
the `+` affordance after the last projected tab. A renderer regression must
create at least three sessions and prove strictly increasing, non-overlapping x
positions in creation order with the new-session target to their right.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05H-AUTOMATED -->
The renderer now consumes the ordered `terminal.tabs` projection instead of
redrawing only the active title at the strip's fixed left origin. Each session
receives its own `TerminalSessionTab` hit target, the x origin advances after
every projected tab, and the new-session target follows the final tab. The
three-session production-scene regression
`new_terminal_tabs_append_left_to_right_and_plus_follows_last_tab` proves the
creation order, strictly increasing non-overlapping rectangles, and final `+`
placement. The convergence guard and its mutation suite reject a reversed or
fixed-origin loop, a missing new-session target, or removal of the production
proof. Fresh-build owner QA remains the completion boundary for DTC-P05H.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05H-OWNER-ACCEPTED -->
Owner QA on 2026-08-17 confirmed that new terminal tabs are indexed correctly
and append to the right. DTC-P05H therefore closes with implementation revision
`0c1e153` and its ordered prepared-scene and convergence-mutation proofs.

#### DTC-P05I lean terminal session chrome

The owner identified the persistent `SESSIONS / +NEW / RENAME / RESTART /
CLOSE / shell 1..N` row as redundant with the top tab strip and keyboard
controls. DTC-P05I removes that entire band from shared terminal geometry so
the reclaimed height becomes a real PTY cell row. The top strip remains the
sole visible session index and new-session affordance; Ctrl+Shift+T,
Ctrl+Shift+R, and Ctrl+Shift+W retain new, restart, and guarded-close access.
Normal running state renders no persistent lifecycle buttons. Armed close,
termination failure, force-kill, application-shutdown retry, and cancel remain
contextual safeguards in the compact header rather than consuming a permanent
row. The retired inline rename editor and its protocol/input/hit-target state
are deleted; shell applications may continue to provide standard OSC titles.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05I-AUTOMATED -->
Automated evidence: the prepared dock contract rejects every retired persistent
menu label and requires exactly one top-strip target per session plus the final
new-session affordance; shared geometry proves the removed band returns a real
cell row at the default dock height. GUI app, renderer, protocol, and viewport
tests pass together (323, 123, 96, and 37 unit tests, plus their integration
suites), as do strict Clippy, dependency authority, convergence mutations,
spec governance, Frontier validation, and diff hygiene. Owner visual acceptance
of the lean row remains the DTC-P05I completion boundary.

Owner QA on 2026-08-17 rejected the first DTC-P05I build on two session-creation
regressions: the displayed Ctrl+Shift+T shortcut was not connected to creation,
and closing an earlier tab allowed the live-session count to reuse an existing
default label. The repair gives the registry a monotonic session ordinal that
is independent of removal and routes one shared, nonrepeating Ctrl+Shift+T
predicate through both terminal-focus and editor-focus dispatch. Regression
proof now requires `shell 2` followed by `shell 3` after `shell 1` is removed
and rejects shortcut or count-derived-label drift. Fresh owner QA remains the
completion boundary.

The next owner QA pass found that both `+` and Ctrl+Shift+T updated the session
projection but left the already-open dock's prepared frame valid. The visible
tab therefore waited for an unrelated shell-output wake, presenting as a
variable multi-second spawn delay even though the real PTY spawn path completes
in tens of milliseconds. The shared successful-spawn boundary now invalidates
the frame immediately after synchronizing tabs; failure status does the same.
The convergence guard rejects a creation path that publishes tabs without the
matching frame invalidation.

Subsequent owner QA confirmed two remaining launch-path defects: the pending
tab was visible but did not become active, and the selected shell remained slow
to become ready on the production project. The registry now gives the pending
tab explicit active authority immediately, parks the prior shell projection,
and rejects input until the new PTY is installed so no keystroke can leak to the
old session. Readiness no longer persists the retired `Attached` pseudo-event:
background tabs are continuously owned under P05-O1, so creation and tab
switching do not rewrite lifecycle state. Pre-exec bootstrap now durably writes
only the per-session discovery document passed to the child; after spawn it
publishes the complete PID-bearing context, latest alias, and session metadata
once. This reduces the additional-tab path from nine durable file
synchronizations to four while preserving context-before-exec and
PID-context-before-output ordering. The guard rejects restored attach writes or
redundant pre-spawn alias publication, and the full GUI-app suite passes 329
tests. Fresh owner QA remains the completion boundary.

Owner QA then isolated two remaining approximately two-second pauses: the
prompt arrived late after creation, and selecting an already-running tab by
pointer was equally slow. The activation handler was still rebuilding and
durably rewriting all three context records before swapping the visible
projection. Tab activation is now memory-only with respect to session context;
context refresh remains an explicit workspace/context operation rather than a
navigation side effect. For new sessions, the per-session discovery document
still forms the first durable pre-exec barrier, but the independent
PID-bearing context, latest alias, and tool-session metadata writes now execute
concurrently behind the second barrier. Atomic temporary names include a
process-wide nonce so concurrent tabs cannot collide. The full GUI-app suite
passes 330 tests, and the convergence guard rejects restored activation I/O or
serial PID-context publication. Fresh owner latency QA remains required.

The next fresh-build QA pass confirmed tab switching was immediate but prompt
readiness still waited several seconds. The remaining blocker was forced
`sync_all` on generated discovery/session snapshots. Those snapshots are
reconstructible runtime metadata, not Datum's canonical design state, so shell
launch now requires atomic same-directory replacement and parseable visibility
without waiting for storage-firmware crash persistence. The pre-exec discovery
and post-spawn PID-bearing ordering remain unchanged, as do journal fsyncs for
canonical design mutation. A real PTY fixture reaches its prompt in roughly 40
ms locally, all 330 GUI-app tests pass on repeat, and the convergence guard
rejects a restored forced flush. Fresh owner prompt-latency QA remains the
completion boundary.

Fresh QA disproved storage flushing as the complete explanation: prompt
readiness still exceeded two seconds on the production workspace after that
repair. Each terminal-context serialization was also refreshing the accepted
transaction tip by running a full `ProjectResolver::resolve()` over the
project, and session creation serialized that context twice. Terminal launch
now carries the accepted transaction tip already held by the authoritative
workspace supervision state; it no longer performs hidden project resolution
while preparing a shell. Context contents remain unchanged, while the
convergence guard rejects reintroducing resolver work into the launch path.
All 330 GUI-app tests and strict Clippy pass. Fresh owner prompt-latency QA
remains the completion boundary.

Owner QA then confirmed prompt readiness is immediate and accepted the launch
latency repair. The same pass identified the remaining persistent `SHELL
SESSION / RUNNING / title / CWD / SIZE / modes` line as redundant diagnostic
chrome. Normal terminal state now keeps those values only in the functional
session/protocol model and renders no metadata row; close, teardown failure,
and blocked application shutdown remain visible and actionable in the compact
header. Reducing the header from 34 to 18 pixels returns one additional cell
row (eight rather than seven at the default dock height). Fresh visual QA of
the single-line header remains the DTC-P05I completion boundary.

The next owner review found that the remaining `PROJECT TERMINAL` and
copy/scroll/paste hint band still duplicated the tab strip and permanently
withheld a row. The persistent header is now deleted end to end, including its
shared geometry field and sizing budget. The terminal screen begins directly
at the lane interior and receives nine rows at the default dock height.
Actionable close, teardown, and blocked-shutdown state renders only while
active as transient safety chrome over the first screen row, so it remains
visible without changing PTY geometry. Fresh visual QA of the header-free
terminal remains the DTC-P05I completion boundary.

DTC-P05I also restores a complete pointer close path after retiring the old
session-control row. Every projected terminal tab owns a right-edge `×` hit
target carrying that tab's session identity. Clicking an inactive tab's close
target activates that exact session before applying the existing lifecycle
law: an already-complete session is removed immediately, while a live session
only arms guarded confirmation and remains visible until verified teardown.
The close target is non-entry chrome and cannot steal terminal keyboard focus.
Fresh owner QA of the tab close affordance remains the completion boundary.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P05I-OWNER-ACCEPTED -->
Owner acceptance on 2026-08-18 closes DTC-P05I after repeated fresh-build QA of
the complete terminal round. The accepted surface includes terminal click
focus and Tab routing, prompt and cursor alignment, color and governed glyph
rendering, responsive input after output, ordered and draggable session tabs,
prompt new-session activation, guarded per-tab close, clipboard and visible
selection, reclaimed terminal rows, and smooth dock-boundary resizing. The
owner explicitly classifies this terminal round as usable. Project/layer
sidebar resizing, scrolling, overflow affordances, and project-panel cleanup
are separate viewport work and do not extend DTC-P05I.

#### DTC-P06A owner packet

DTC-P06 is transport/session proof, not terminal-core or application
compatibility proof. Decision 030 reserves numerical performance and resource
budgets to the project owner, so code agents must stop before timed assertions.
The selector presents these decisions one at a time. P05-O1 already forbids a
detached Datum PTY: P06 therefore proves inactive/background tabs remain owned,
drain independently, and reactivate without contamination; it must never
reintroduce detach/reattach APIs. Logical history/reflow remains DTC-P12 and
shell/TUI/agent compatibility remains DTC-P28 through DTC-P30.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O1 --> **P06-O1 — proof tiers and duration.** Owner-approved 2026-08-18:
  release builds are normative. Run a bounded ten-minute eight-session proof in
  normal CI; a scheduled 24-hour single-session agent-style soak plus a
  four-hour proof at the governed maximum of sixteen sessions; and 1,000
  spawn/exit/restart cycles. Require three consecutive clean scheduled runs
  before release evidence is accepted. Record the exact revision, seed, kernel,
  libc, CPU, RAM, display backend, and raw samples. This duration is intended to
  expose slow leaks and lifecycle drift from multi-million-token code-agent
  sessions; it does not cap transcript or model-context length.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O1-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O1 proof tiers and durations
exactly as stated above. No later P06 numerical threshold is implied.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O2 --> **P06-O2 — session workload and correctness.** Owner-approved 2026-08-18:
  the ten-minute CI tier runs eight simultaneous real PTYs, with each session
  carrying at least 8 MiB of exact output and 1 MiB of exact input. Scheduled
  runs rotate agent-style burst, status-update, full-screen, resize, exit,
  restart, saturation, and idle roles; the long tiers carry at least 128 MiB of
  output and 8 MiB of bidirectional input per exercised session, with at least
  1 GiB aggregate output. Every stream uses a unique seeded identity and must
  prove zero lost, duplicated, reordered, stale, or cross-session bytes. Keep
  the already-ratified maximum of sixteen live sessions and refusal of the
  seventeenth.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O2-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O2 workload and byte-correctness
floor exactly as stated above. No latency or throughput threshold is implied.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O3 --> **P06-O3 — responsiveness.** Owner-approved 2026-08-18: on the reference
  release-build Linux workstation, idle input-to-first-output latency is p95 at most 25 ms,
  p99 at most 50 ms, and absolute maximum 100 ms. With sustained output from
  peer sessions it is p95 at most 50 ms, p99 at most 100 ms, and absolute
  maximum 250 ms; the first command after a burst is visible within the same
  100 ms p99 and 250 ms maximum. GUI drain work is p95 at most 8 ms, p99 at
  most 16 ms, and maximum 33 ms, with no main-thread unresponsive interval
  exceeding 100 ms. Shared CI records these timings but gates deterministic
  work and correctness rather than flaky wall-clock measurements.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O3-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O3 responsiveness thresholds
exactly as stated above. No throughput or memory threshold is implied.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O4 --> **P06-O4 — throughput and backlog recovery.** Owner-approved 2026-08-18:
  the raw owned-PTY consumer sustains at least 20 MiB/s for one session and at least
  40 MiB/s aggregate across eight producers for 60 seconds. The current
  production registry/provisional-screen path sustains at least 1 MiB/s for one
  session and 4 MiB/s aggregate across eight sessions. After a producer stops,
  a full approved 4 MiB output backlog falls below 64 KiB within 2 seconds and
  reaches zero within 5 seconds; no runnable session waits more than 100 ms for
  a drain quantum. The provisional-screen floors expire at the P26 owned-core
  cutover and are anti-regression limits, not TerminalCore performance targets.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O4-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O4 throughput and backlog-recovery
thresholds exactly as stated above. No memory or storage threshold is implied.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O5 --> **P06-O5 — memory and bounded backlog.** Owner-approved 2026-08-18:
  preserve the existing hard 4 MiB output plus 4 MiB input ceiling per session, the sixteen-
  session limit, and therefore a 128 MiB maximum aggregate queued payload. At
  sixteen sessions, Datum-process RSS peaks no higher than the warm baseline
  plus 192 MiB. After warm-up, RSS growth is at most 1 MiB/hour for the active
  24-hour session and at most 256 KiB/hour per idle session. Within 30 seconds
  after every terminal tab closes, RSS returns to the warm baseline plus 16 MiB.
  Disk event-log retention is measured separately and remains P06-O9 rather
  than being hidden inside the RSS claim.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O5-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O5 memory and bounded-backlog
thresholds exactly as stated above. These are current release acceptance
budgets, not a permanent cap on transcript or model context; future 2M-token
and larger agent workloads may expand them through a new measured owner
decision without weakening losslessness or boundedness.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O6 --> **P06-O6 — descriptors, workers, and process cleanup.** Owner-approved
  2026-08-18: Datum-process file-descriptor growth is at most four descriptors per live
  terminal session plus eight global, and worker-thread growth is at most four
  threads per live session plus four global. Within one second after ordinary
  verified close/reap, both counts return to their warm baseline plus two.
  Across the approved 1,000 spawn/exit/restart cycles there is zero upward
  descriptor or thread slope, zero zombies, and zero surviving process whose
  Linux session remains Datum-owned.

<!-- EVIDENCE:TERMINAL-T1-PTY:DTC-P06A:P06-O6-OWNER-APPROVED -->
Owner approval on 2026-08-18 ratifies the P06-O6 descriptor, worker-thread,
zombie, and owned-process cleanup thresholds exactly as stated above.

- <!-- OWNER:TERMINAL-T1-PTY:DTC-P06A:P06-O7 --> **P06-O7 — resize and isolation.** Recommended: perform 10,000 total
  resize requests during the sixteen-session mixed workload, with at least 500
  requests per session and concurrent terminal I/O. Resize ioctl/request
  completion is p95 at most 2 ms, p99 at most 5 ms, and maximum 20 ms; the final
  kernel winsize is exact within 100 ms. SIGWINCH reaches only the current
  foreground process group. Input, output, resize, exit, termination, restart,
  tab activation, and inactive-session draining exhibit zero cross-session
  contamination. Logical reflow correctness remains DTC-P14.

Approving P06-O7 approves only resize and cross-session isolation thresholds.
It does not yet approve platform, storage, or
landing thresholds; those remain subsequent P06A owner decisions.
No dependency, TERM identity, TerminalCore behavior, visual golden, or final
release acceptance is authorized.

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
