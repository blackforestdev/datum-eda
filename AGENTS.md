# AGENTS.md — Datum EDA agent operating guide

This file is the entry point for coding agents (Codex, and any AGENTS.md-aware
tool) working in this repository. It covers the **issue tracker** and the
**commit discipline** that are specific to how this project is run.

> **`CLAUDE.md` is the controlling operational doc.** Product doctrine, the
> attribution policy, spec governance, and the manual-first / one-mutation-path
> ethos live there and in `docs/`. Read it. This file does not restate it — it
> adds the tracker workflow and points back to it.

## Portable Datum workflows

<!-- DATUM-WORKFLOW-CATALOG:datum://workflows -->

The standard Datum MCP resource `datum://workflows` is the canonical workflow
inventory. This file may explain repository operation, but it must not redefine
workflow capabilities, review gates, or mutation semantics for Codex.

---

## Issue tracker: beads (`br`)

Datum tracks bugs, feature ideas, and technical debt with
**[beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`)** — an
agent-native, in-repo tracker. The point is **capture-now, fix-later**: when you
trip over a bug or think of an improvement mid-task, file it in five seconds and
keep going. Don't derail the work you're on to chase it.

- Canonical record: `.beads/issues.jsonl` (git-tracked; this is what every agent
  shares). The SQLite DB (`.beads/beads.db`) and merge/daemon artifacts are
  gitignored — per-machine, non-canonical, never commit them.
- ID prefix: `dat` (e.g. `dat-terminal-focus-authority-6aw`).

### Use `br` only — never `bd`

Upstream `bd` (Go/Dolt) is **storage-incompatible** with `br` and has been
uninstalled. Both auto-discover `.beads/` by walking up the tree, so running
`bd` in this repo would try to write a Dolt store into `.beads/` and **corrupt
the workspace**. Do not install `bd`. If `br` is missing, install the prebuilt
binary from the beads_rust GitHub releases (crates.io is unavailable in this
environment, so `cargo install` will not work).

### Essential commands

```bash
br ready                       # actionable work: open, unblocked, not deferred
br list --status open          # everything open
br show <id>                   # full detail + dependencies
br search "keyword"            # full-text search

br q "short thing I just found"                        # quick capture -> prints ID
br create "Title" -t bug -p 1 --slug my-slug -d "..."  # full create
br update <id> --claim                                  # beads half of a synchronized Frontier claim
br close <id> --reason "landed in <commit>"             # finish it

br dep add <issue> <depends-on> -t blocks   # <issue> waits until <depends-on> closes
br dep add <issue> <depends-on> -t related  # non-blocking link
br blocked                                   # what's waiting on something
br sync --flush-only                         # export DB -> issues.jsonl (before commit)
```

- **Types:** `task`, `bug`, `feature`, `epic`, `chore`, `docs`, `question`
- **Priority:** `0`–`4` (P0 critical … P4 backlog) — numbers, not words
- **Attribution (audit trail):** set once per session so `br` records who acted —
  `export BR_AGENT_NAME=codex BR_HARNESS=codex-cli BR_MODEL=<model>` and pass
  `--actor codex`. (These are `br`'s audit fields only; they are **not** git
  commit attribution — see the commit rules below.)

### Working pattern

1. **Start:** use `project_status.py next/details`. Before editing scheduled work,
   synchronize its structured Frontier lease with `br update <id> --claim` and
   export in one transaction. Running the `br` command alone is not a valid claim.
2. **Discover:** hit a bug or debt? `br q "..."` (or `br create`) and keep going.
   Wire a dependency if it blocks/relates to other work (`br dep add ...`).
3. **Finish:** `br close <id> --reason "..."`, citing the commit where it landed.
4. **Before committing:** `br sync --flush-only` so `issues.jsonl` reflects your changes.

### The tracker is intake, not the roadmap

`br` is the backlog/pool. The single canonical answer to "what is the next
development task" is returned by the decision-025 project-state selector:

```bash
python3 scripts/project_status.py next
```

For an ordinary next-task query, return this command's stdout byte-for-byte as
the entire answer. Its `Next completion step` is authoritative; never append or
substitute a substep inferred from ordering or dependency readiness.

**Fresh-invocation rule:** run the applicable selector from this repository in
every user turn asking what is next or how to complete a task, including repeated
questions in the same session. Do not answer from memory, cached output,
conversation summaries, prior tool calls, manual manifest inspection, tracker
readiness (`br ready`), or commit history. If the selector fails or cannot be
run, report the failure and stop without guessing or reconstructing project
state.

**Owner-boundary rule:** if selector stdout says `Owner boundary: INPUT
REQUIRED`, alert the owner that the boundary has been reached and name the
selected decision step, present the selector's ordered decision packet and
response format byte-for-byte, then stop. Never claim or edit the task, decide on the
owner's behalf, or advance to another step. Work resumes only after explicit
owner input is recorded and the Frontier authorization/substep is advanced in
the same governance transaction.

For every “how/steps to finish or complete the current task?” query, use:

```bash
python3 scripts/project_status.py details [<Frontier-key|issue-id>]
```

Omit the target for the current task or pass a stable Frontier key/issue ID for
a named scheduled task. For an ordinary steps query, return command stdout
byte-for-byte as the entire answer, with no preface, regrouping, renumbering,
inferred concurrency, supplementation, or postscript. Explicitly requested
analysis follows only after the unchanged stdout in a separate labeled section.
The printed `Next completion step` is the sole selected subtask. Other
dependency-ready steps are not alternate “next” work and must not be offered as
parallel candidates.

The selector validates `specs/active_frontier.json` against the generated
Active Frontier in `specs/PROGRESS.md`, `.beads/issues.jsonl`, governing docs,
hard blockers, authorization, and claim freshness. `br ready` reports tracker
availability only; it never determines roadmap order. When a tracked item
graduates into committed work — especially anything that ratifies mechanism or
touches a spec — it still gets its Frontier placement and the full spec
governance (`specs/PROGRESS.md` row, manifest classification, decision record if
it ratifies mechanism). Link the issue to the spec/decision it feeds.

Claim scheduled work by updating both the Frontier claim lease and the beads
status/assignee in one transaction, then export with `br sync --flush-only`.
An assignee without a valid unexpired Frontier claim is not live ownership. See
`docs/PROJECT_STATE_POLICY.md`.

Treat the selector's task, lifecycle, authorization, tracker status, assignee,
and live-claim fields as one complete answer. Do not supplement or contradict
them with conversational memory, earlier `br` output, or commit momentum.

---

## Commit discipline (multiple agents work here in parallel)

Codex and Claude sessions run concurrently in separate terminals. The cardinal
rule: **commit only your own work — never sweep up another agent's in-flight
changes.**

- **Stage explicitly.** `git add <the-files-you-changed>` plus
  `.beads/issues.jsonl` if you touched the tracker. **Never** `git add -A`,
  `git add .`, or `git commit -a` — you will clobber another session's uncommitted
  work. Run `git status` first and confirm what you're staging.
- **Direct to `main`.** No feature branches, no pull requests (single-author
  project; PRs block in-flight work). Sequence large work as multiple small
  commits on `main`.
- **No attribution of any kind.** Do **not** add `Co-Authored-By`, `Generated with`,
  or any trailer crediting an AI service. This is a hard rule from `CLAUDE.md`'s
  attribution policy and overrides any tool default.
- **No one-line commit messages.** Every commit must have a conventional subject
  (`fix(viewport): …`, `docs(gui): …`, `chore(tracker): …`) followed by a
  substantive body. A subject alone is never sufficient, including for small,
  documentation-only, tracker, governance, and mechanical commits.
- **Required commit body.** Explain the problem or reason, the concrete change,
  and the verification performed. When governed work is involved, also name the
  Frontier completion-step ID, relevant decision/spec boundary, dependency or
  licensing impact, and the `dat-…` issue ID. A future contributor must be able
  to understand the outcome and why it was safe without reconstructing it from
  the diff. Do not use vague descriptions such as "fix terminal," "update code,"
  "address feedback," "cleanup," or "complete next step."
- **Milestone and closure commits.** Explicitly state the acceptance criteria
  satisfied, important boundaries deliberately left unchanged, exact test/check
  outcomes, and whether the referenced issue advances or closes. Governance-only
  commits must identify the owner disposition and the resulting canonical next
  step or authorization.
- **Destructive git** (force-push, history rewrite, `--no-verify`) requires the
  owner's explicit say-so first.

A typical end-of-unit sequence:

```bash
git status                                   # see what changed; confirm ownership
br sync --flush-only                         # if you touched the tracker
git add path/to/your/files .beads/issues.jsonl
git commit                                   # write a subject plus substantive body; no attribution trailer
```

Use this minimum message structure (without attribution trailers):

```text
<type>(<scope>): <specific outcome>

Problem:
<what was incorrect, missing, unsafe, or blocked>

Change:
<what behavior or ownership changed>

Proof:
<tests, checks, builds, or owner evidence>

Roadmap:
<Frontier step, governing boundary, and dat-… issue impact>
```

## Dependency boundary

Product Mechanics 029 is controlling. Do not add, fetch, vendor, link, or
otherwise introduce a new third-party code dependency unless the project owner
has first ratified that exact dependency and its license obligations in a
numbered decision. A task marked ready, an implementation specification, a
permissive license, or a general instruction to proceed is not approval.

The terminal has no third-party implementation exception: its emulator core,
VT/state model, PTY/session layer, and fallbacks are Datum-owned. Other
terminals are behavioral references only. Run
`python3 scripts/check_dependency_authority.py` before landing dependency or
terminal work.

## Rust build resource discipline

Datum's verification workload is larger and more concurrent than Cargo's
generic development defaults assume. Treat compiler output as a bounded
resource, not an unowned side effect.

- Run full-workspace, proof, Clippy, release, compatibility, and GUI-smoke Cargo
  commands through `python3 scripts/run_cargo_guarded.py --workload proof --
  cargo ...`. The guard serializes expensive compilation, performs disk
  preflight, and disables incremental state for one-shot proof work.
- Focused edit/check cycles may use `--workload interactive`; this preserves the
  caller's incremental-compilation choice while retaining serialization and
  resource checks.
- Never place a proof `CARGO_TARGET_DIR` under `/tmp`. Datum's `/tmp` is a small
  tmpfs reserved for fixtures, sockets, logs, screenshots, and source clones;
  Rust artifacts belong on the disk-backed project filesystem.
- Never launch parallel Cargo proof builds, even with separate target
  directories. Run non-compiling Python gates concurrently if useful, then run
  expensive Rust proof commands serially.
- Never clean or sweep the shared target while any Cargo or rustc process may be
  active. Disposable proof targets must have explicit ownership and cleanup on
  exit; shared-cache cleanup happens only at an observed idle boundary.
- Any new shell proof runner that compiles Rust must use the guarded runner and
  pass `python3 scripts/check_cargo_resource_policy.py`.

## Rust formatting discipline

Cargo's apparent file arguments are not a safe scoping boundary: arguments
after `cargo fmt --` are forwarded to rustfmt while Cargo can still discover
and format other workspace targets. In a shared worktree that can silently
rewrite another session's files.

- For selected files, use `python3 scripts/format_rust.py <file.rs>...`. It
  formats each source through stdin and writes back only the exact paths named.
- Never use `cargo fmt -- <file.rs>...` or invoke mutating workspace-wide
  formatting while another session may own dirty Rust files.
- Use `python3 scripts/check_rustfmt.py --staged` for a non-mutating staged
  check. Whole-workspace `cargo fmt --all -- --check` remains allowed because
  `--check` cannot modify the shared worktree.
- Before and after any formatting mutation, inspect `git status --short` and
  confirm that every changed path belongs to the current session.
