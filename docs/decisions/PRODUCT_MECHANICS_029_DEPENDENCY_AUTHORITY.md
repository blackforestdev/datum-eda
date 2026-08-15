# Product Mechanics 029: Dependency Authority And Datum-Owned Terminal

Status: ratified doctrine

## Decision

The project owner is the sole authority that may introduce a new third-party
code dependency into Datum EDA. Agent execution authorization, roadmap
placement, an existing specification, a permissive license, technical merit,
or a general instruction to proceed is **not** dependency approval. A new
dependency requires an explicit, numbered owner decision naming the exact
component and accepted license obligations before any source, binary, build
helper, manifest entry, lock entry, or download path is added.

The Datum terminal has a stricter closed boundary: its terminal emulator,
VT/state machine, cell model, scrollback, reflow, protocol encoders, and PTY
session implementation are Datum-owned source in this repository. No external
terminal implementation may be linked, vendored, copied, downloaded at build
time, invoked as a hidden subprocess, or retained as an evidence-gated
fallback. Other terminals may be studied and named as behavioral compatibility
references only; their source code is not an implementation input.

This decision rejects and supersedes the third-party-core and third-party-PTY
mechanisms previously permitted by decisions 024 and 027 and the open-question
terminal-stack recommendation. The unauthorized dependency commits are retained
only in Git history and are reversed by auditable revert commits. They are not
approved precedent.

## Normative Rules

- **DA-001 — explicit owner authority.** Only an explicit numbered owner
  decision may approve a new third-party code dependency. Silence, prior art,
  license compatibility, `project_status.py`, and “proceed” do not imply it.
- **DA-002 — no terminal dependencies.** Datum's terminal implementation may
  use Rust's standard library, existing Datum workspace substrate, and Linux
  operating-system interfaces. It may not add a terminal-specific third-party
  crate, library, source tree, generated binding, downloaded build input, or
  subprocess implementation.
- **DA-003 — Datum-owned semantics.** `TerminalCore` is Datum source, not an
  adapter around vendor state. Datum owns parsing, state transitions, cells,
  Unicode-width policy, modes, replies, damage, history, reflow, selection,
  search, and terminal protocol behavior.
- **DA-004 — references are not dependencies.** Ghostty, Konsole, Alacritty,
  xterm, and similar products remain compatibility and UX benchmarks. No
  statement of parity or prior-art research authorizes their code.
- **DA-005 — no automatic fallback.** Failure or cost in the Datum-owned path
  creates an owner boundary. Agents must not select another library, crate,
  copied implementation, or embedded executable as a fallback.
- **DA-006 — locked existing baseline.** Existing direct external workspace
  dependencies are a frozen inherited baseline, not blanket approval for new
  dependencies. The dependency-policy gate rejects additions until the owner
  ratifies an exact exception in a later numbered decision.
- **DA-007 — same-change governance.** An approved exception must update the
  dependency policy, licensing inventory, governing specification, Frontier,
  tracker, and automated gates in the same change.

## Terminal Recovery Sequence

1. Remove the unauthorized libghostty-vt and portable-pty source, build,
   manifest, lock, and runtime integration paths.
2. Restore and harden Datum's Linux PTY/session implementation behind a
   Datum-owned transport boundary.
3. Build the Datum-owned `TerminalCore` in bounded modules under decision 022,
   using checked-in behavior fixtures authored for Datum.
4. Complete the renderer, input, daily-driver, protocol, accessibility, agent,
   and production-proof slices already required by decisions 027 and 028.

The full native-terminal quality target is unchanged. Ownership of the
implementation changes; acceptance rigor does not.

## Consequences

Datum accepts the engineering cost and schedule of owning terminal semantics.
The benefit is an implementation whose source, licensing, supply chain, and
product direction remain entirely under project-owner authority. No terminal
slice may reduce the product target to a console widget in response to that
cost.
