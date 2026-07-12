# AGENTS.md — Datum EDA agent operating guide

This file is the entry point for coding agents (Codex, and any AGENTS.md-aware
tool) working in this repository. It covers the **issue tracker** and the
**commit discipline** that are specific to how this project is run.

> **`CLAUDE.md` is the controlling operational doc.** Product doctrine, the
> attribution policy, spec governance, and the manual-first / one-mutation-path
> ethos live there and in `docs/`. Read it. This file does not restate it — it
> adds the tracker workflow and points back to it.

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
br update <id> --status in_progress                     # claim it
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

1. **Start:** `br ready` to see unblocked work; claim with `br update <id> --status in_progress`.
2. **Discover:** hit a bug or debt? `br q "..."` (or `br create`) and keep going.
   Wire a dependency if it blocks/relates to other work (`br dep add ...`).
3. **Finish:** `br close <id> --reason "..."`, citing the commit where it landed.
4. **Before committing:** `br sync --flush-only` so `issues.jsonl` reflects your changes.

### The tracker is intake, not the roadmap

`br` is the backlog/pool. The single canonical answer to "what is the next
development step" remains the **Active Frontier** at the top of
`specs/PROGRESS.md` (the bullseye rule in `CLAUDE.md`). When a tracked item
graduates into committed work — especially anything that ratifies mechanism or
touches a spec — it still gets its Frontier placement and the full spec
governance (`specs/PROGRESS.md` row, manifest classification, decision record if
it ratifies mechanism). Link the issue to the spec/decision it feeds.

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
- **Well-annotated messages.** Follow the repo's conventional style
  (`fix(viewport): …`, `docs(gui): …`, `chore(tracker): …`). Say what changed, why,
  and — where relevant — its place in the roadmap. Cite the `dat-…` issue ID when a
  commit advances or closes a tracked issue.
- **Destructive git** (force-push, history rewrite, `--no-verify`) requires the
  owner's explicit say-so first.

A typical end-of-unit sequence:

```bash
git status                                   # see what changed; confirm ownership
br sync --flush-only                         # if you touched the tracker
git add path/to/your/files .beads/issues.jsonl
git commit -m "fix(...): … (dat-<id>)"       # no attribution trailer
```
