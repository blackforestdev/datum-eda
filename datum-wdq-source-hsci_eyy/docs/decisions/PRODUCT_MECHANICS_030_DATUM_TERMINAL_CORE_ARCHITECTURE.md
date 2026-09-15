# Product Mechanics 030: Datum-Owned Terminal Core Architecture

Status: ratified doctrine

## Decision

Datum implements terminal semantics in a new first-party Rust workspace crate,
`datum-terminal-core`. The crate has no external package dependencies: it uses
Rust `std` and Datum-authored source/data only. Linux PTY/session ownership stays
behind a bounded Datum module in `gui-app` using the inherited `libc` baseline
and operating-system interfaces. Renderer integration uses existing approved
Datum/wgpu/text substrate but terminal semantics remain renderer-independent.

There is no Zig replacement. Zig was only the reverted Ghostty build tool.

## Architecture laws

- **DTC-001 — pure streaming core.** `feed(bytes)` and `resize()` reduce input
  into deterministic private state and return replies, typed events, damage, and
  immutable render snapshots. They perform no GUI, clipboard, filesystem,
  process, network, MCP, or design mutation.
- **DTC-002 — one state authority.** The first-party core becomes the only
  production cell/state/history authority. Terminal state leaves
  `gui-protocol`; that crate retains chrome/session metadata only.
- **DTC-003 — typed reducer.** Streaming syntax produces typed actions. One
  reducer owns screen mutation. Parser, model, history, input, graphics,
  security, damage, and renderer are separate cohesive module families.
- **DTC-004 — standards-first clean room.** Normative specifications and
  black-box observations define behavior. No upstream terminal source, internal
  test, generated table, fixture, algorithm, or translated implementation is an
  input to Datum source.
- **DTC-005 — governed standards data.** Unicode property data is pinned
  normative standards data with exact provenance, retained Unicode-3.0 notice,
  deterministic Datum-owned generation, reviewed output, and no runtime or
  build download. It is not a code dependency.
- **DTC-006 — bounded by construction.** Parameters, payloads, replies,
  queues, history, graphics, decompression, search, reflow, and work per feed
  have explicit checked limits and deterministic exhaustion behavior.
- **DTC-007 — truthful identity.** Datum ships a Datum terminfo identity until
  exact xterm compatibility is proven. `TERM`, device attributes, and probes do
  not advertise unimplemented behavior.
- **DTC-008 — proof with every slice.** Each package lands normative snapshots,
  chunk invariance, hostile-input limits, deterministic mutation/replay,
  performance evidence, and source/dependency gates with its implementation.
- **DTC-009 — one-way cutover.** The provisional and new core may receive the
  same recorded bytes only in tests/debug comparison. There is no user-visible
  selector or runtime fallback. Production cutover atomically selects the new
  core and deletes the provisional parser, string grid, rival state, and shadow
  path. Source-control revert is the emergency rollback.
- **DTC-010 — source health.** Every new production or test file remains below
  decision 022 limits, with a design target of 300–500 production lines. No
  `include!`, continuation, forwarding, or facade-only decomposition is allowed.

## First-party crate boundaries

`datum-terminal-core` owns parser, actions/reducer, cells/screens, text and
Unicode policy, modes/replies/events, logical history/reflow, selection/copy,
bounded search/regex, input protocols, metadata/security, graphics/codecs,
damage, snapshots, limits, and deterministic proof utilities.

`gui-app` owns PTY/session transport, scheduling/backpressure, focus/IME, system
clipboard/security prompts, session registry, core adapter, and accessibility
platform bridge. `gui-render` owns only render projection and GPU resources.
The engine and design model never depend on terminal state.

## Required scratch-built codecs

Full kitty-graphics parity requires Datum-owned Base64, zlib framing, Adler-32,
DEFLATE, PNG parsing/filtering/CRC/color/interlace decode, and bounded image
storage/composition. Sixel is separately implemented from its normative grammar.
These are explicit late packages, not hidden utilities or dependency exceptions.

## Migration law

Existing PTY, focus, session, context, wake/batching, parser, and input behavior
is preserved as regression evidence. Existing terminal code is migrated or
replaced behind the new boundaries; near-limit legacy modules receive no new
feature growth. The old model is deleted only after overlap parity and new
normative proof pass.

## Conductor law

One conductor owns Frontier selection, synchronized claim/lease, issue state,
exclusive write paths, evidence, and advancement. At most one completion package
is a shared-worktree writer. Parallel agents may perform read-only standards
mapping, adversarial review, or isolated verification. A writer receives exact
paths, requirement IDs, line budgets, commands, and stop conditions. The
conductor advances only after independent verification and same-change evidence.

Dependency suggestions, standards-data license changes, unsupported protocol
deltas, security posture, performance budgets, golden replacement, capability
identity, and final release acceptance are owner boundaries.

## Completion

The architecture is complete only when the Datum-owned PTY and core are sole
production paths; decisions 027 FT-001..013 and terminal NT-CAP-01..18 have
code, deterministic evidence, compatibility results, and owner evidence; the
provisional core is deleted; named shells/TUIs/agents pass; dependency, offline,
source-health, security, performance, accessibility, visual, lifecycle, and
multi-session gates pass; and T4e receives final owner acceptance.
