# Agent operational procedures

Status: active task-specific reference. Read the applicable section before its
action; this document is not mandatory startup reading. CLAUDE.md controls doctrine,
PM059 controls delegated routing, and AGENTS.md is the compact execution entry.
This consolidation preserves procedures without creating a competing authority.

## Reading map

- Tracker/claims and ordinary roadmap responses: Issue tracker below and
  `docs/PROJECT_STATE_POLICY.md`; PM059 applies to delegated boundaries.
- Commits: Commit discipline below.
- Specification/research edits: Specification governance below and the owning
  evidence route; preserve complete route review and same-change reconciliation.
- Rust proof/formatting: corresponding sections below.
- Performance requirements: Numerical provenance below; no invented budget.

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
- **Claude owns every GUI visual-truth HTML file.** Every
  `docs/gui/prototypes/*.html` file is in the Claude file lane. Codex must never
  edit, format, rename, delete, regenerate, or apply even mechanical annotation,
  link, terminology, or disposition changes to those files. Codex may inspect
  and screenshot them. Required changes must be returned to Claude as a bounded
  reconciliation list naming the exact file, anchor/region, required outcome,
  preserved decisions, and proof expected. The owner-approved commit gate will
  require a Claude-session visual-truth marker for any staged change touching
  this lane; Codex must never set, copy, forward, or recommend bypassing that
  marker. The marker authorizes only the prototype lane, never other files or
  product decisions.
- **A tripped gate stays with the responsible lane.** If an agent's own
  out-of-lane edit makes evidence traceability, parity, golden, digest, or other
  governance checks fail, that agent must report the failure and hand the edit
  back to the owning lane. It must not refresh authority digests, update parity
  manifests/goldens, bless output, or otherwise absorb the failure to make the
  gate green. Refresh-authority records only an authorized, fully reviewed
  owning-lane change; it never launders a lane violation.
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

## Verification discipline

**Make every proposed test justify the decision it will enable.** Follow
[CLAUDE.md — Verification must earn its cost](CLAUDE.md#verification-must-earn-its-cost).
State a brief rationale per cohesive batch, reuse adequate evidence across
handoffs, and keep raw output out of working context. No new approval workflow
or test ledger is required. Required acceptance and commit checks remain in force.

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


## Specification Governance (controlling)
Core principle: **no orphaned specs — in coverage OR in sequence.** Every active
specification, contract, or decision record must be (a) tracked and classified,
and (b) placed in the ordered roadmap so its position, dependencies, and next
action are derivable. A doc that exists but is not tracked is a governance
defect; a doc that is classified but not woven into the roadmap frontier is a
**roadmap orphan** — equally a defect, and the specific failure this project has
repeatedly paid course-correction time for. Writing a spec is not finished until
the next agent can find where it sits and what comes next.

When you CREATE or REFINE any spec / contract / decision record, in the SAME
change you MUST:
1. Add or update its status row in `specs/PROGRESS.md` (current vs target).
2. Classify it in `specs/spec_governance_manifest.json`
   (`class`: `governed` / `doctrine` / `pending` / `historical`), enforced by
   `check_spec_governance.py` (coverage + classification).
3. Register any inventory shapes it defines in
   `specs/spec_parity_manifest.json` (enforced by `check_spec_parity.py`).
4. If it ratifies mechanism, it must be a numbered decision record in
   `docs/decisions/`, not a loose doc or an `OPEN_QUESTION_RESOLUTIONS` entry.
5. Weave it into the **Active Frontier** at the top of `specs/PROGRESS.md`: give
   it an ordered position with its dependency (what must land first), what it
   unblocks, and its state. A spec describing future work that does not appear in
   the Active Frontier is not done being written.

**Research/prototype evidence is part of specification governance.** Before
drafting, modifying, reviewing, or implementing a specification or decision,
inspect its route in `specs/evidence_traceability_manifest.json` and read every
listed research source, GUI prototype, governed specification, and decision.
The route digest covers both evidence and consumers: changing either side makes
`scripts/check_evidence_traceability.py` fail until the entire route is reviewed
and its digest is deliberately refreshed in the same change. Every Markdown
artifact under `research/` and every `docs/gui/prototypes/*.html` file must have
exactly one owning route. Code/runtime incompleteness or drift never overrides a
ratified decision or controlling visual source. If sources conflict, preserve
the higher authority, record the conflict, and reconcile it through the normal
owner/decision transaction rather than inventing a replacement from memory.

**Roadmap wayfinding (the bullseye rule).** Decision 025 makes
`specs/active_frontier.json` the canonical structured roadmap. The Active
Frontier at the top of `specs/PROGRESS.md` is its generated human projection.
The single canonical answer to "what is the next logical development task?" is:

```bash
python3 scripts/project_status.py next
```

For an ordinary next-task query, return this command's stdout byte-for-byte as
the entire answer. It includes the sole selected completion substep; do not
append an inferred alternative from step order, dependency readiness, or prior
conversation.

**Fresh-invocation rule:** execute the applicable `project_status.py` command
from the current repository during every user turn that asks what is next or how
to complete a task—even when the same question was answered earlier in the
session. Memory, cached stdout, conversation summaries, prior tool results,
manual manifest inspection, `br ready`, and recent commits are never substitutes
for a fresh successful invocation. If the command cannot run or validation
fails, report that failure and stop; do not guess, reconstruct, or fall back to
remembered project state.

**Owner-boundary rule:** when `next` or `details` reports `Owner boundary:
INPUT REQUIRED`, immediately alert the project owner that autonomous work has
stopped and present the selector's complete ordered decision packet and response
format byte-for-byte. Do not claim the bead, edit
files, choose a candidate disposition, skip ahead, or silently treat owner
approval as implied. Resume only after the owner supplies the required choice;
the same governance transaction records its durable review evidence, completes
the decision step, selects the successor substep, and changes authorization to
match it.

For “how/steps to finish or complete the current task?” run:

```bash
python3 scripts/project_status.py details [<Frontier-key|issue-id>]
```

Omit the target for the current canonical task; pass a stable Frontier key or
issue ID when the user names a scheduled task. For an ordinary request for the
steps, return the command's stdout byte-for-byte as the entire answer: no
preface, summary, regrouping, renumbering, relabeling, inferred concurrency, or
postscript. If the user explicitly asks for analysis too, reproduce stdout
unchanged first and put analysis in a separate labeled section. If the plan
appears incomplete, repair the structured contract; never invent a rival plan.
The printed `Next completion step` is the sole subtask-selection authority;
dependency-ready alternatives are not additional next steps and must not be
offered as parallel or interchangeable choices.

Run it before proposing or claiming work; it validates the roadmap against
beads, governing documents, dependencies, authorization, and claim freshness.
Never infer the answer from `br ready`, recent commits, issue priority, or prose
outside the generated Frontier. Update the manifest and regenerate the projection
in the same change as any roadmap change. Other docs, including this file's
"Current Status" and PROGRESS's lower sections, must not restate rival ordering.
The selector's lifecycle, authorization, tracker status, assignee, and live-claim
fields are the complete operational answer; never append contrary state from
conversation history or an earlier tracker read.

Behavioral invariants are enforced by the PG-* proof gates
(`run_migration_proof_gates.sh`) and the write-fence gates
(`check_schematic_private_writers.py`, `check_daemon_write_parity.py`,
`check_resolver_raw_loads.py`, `check_spec_parity.py`). There is NO per-doc
enforcement ledger and no doc-string pinning gates: gates lock behavior in
code, not prose in docs.

Authority ordering (lower layers must not contradict higher ones; on conflict
the higher layer wins and the lower doc is the one to fix):
1. `CLAUDE.md` — operational instructions
2. `docs/DATUM_PRODUCT_MECHANICS.md` + `docs/decisions/` — product doctrine
3. `docs/contracts/` — domain tool contracts
4. `specs/PROGRESS.md` — status truth

### Numerical performance requirements must have an evidentiary basis

A measured baseline, an aspirational target, and a binding acceptance budget are
different things. Label them explicitly. Never invent a number to complete a
specification, present a proposed target as measured capability, or promote a
guideline into a development gate merely through repetition, review or approval.

Before a numerical performance threshold becomes blocking, its controlling
specification must cite its derivation, supporting representative evidence,
applicable hardware/backend/workload configurations, measurement limitations,
and review conditions. Establish the observed baseline with a pinned stable
candidate, verified input/output, complete metric boundaries, instrumentation
effects, sample counts and per-trial variability. Reuse adequate evidence. Budget
selection additionally needs a product rationale: responsiveness, supported-host
expectations and explicit headroom or improvement assumptions. Neither observed
performance alone nor owner approval supplies a missing technical derivation.
If a defensible figure cannot be established, leave it undecided and nonblocking
pending an explicit owner disposition; do not substitute another arbitrary value.

Review the provenance of an existing threshold before organizing a repair around
it. Missing provenance requires a specification correction and owner disposition,
not silent relaxation or an assumption that implementation must eventually meet
it. Preserve actual measurements and correctness obligations. Audit adjacent
CPU/GPU/resource thresholds separately; discovery of one unsupported figure
does not authorize changing all others.

Investigations require a bounded effort and a product decision point. Repeated
qualification failure must trigger review of both implementation and requirement
feasibility, including the value of further work relative to deferred product
functionality. Approval of successive bounded experiments must not become an
unlimited optimization campaign. Inconclusive evidence is an allowed outcome,
not automatic authorization for another experiment. Routine setup failures must
remain distinguishable from measured results; execution recovery remains within
the owner's actual authorized scope.

Record corrections in the controlling decision/specifications, roadmap and
active-agent handoff together. A historical numeric failure stays in the record,
but its interpretation must follow corrected authority; do not claim a withdrawn
threshold was met. See the
[GUI budget provenance incident](docs/reviews/gui-performance/baseline-budget/budget-provenance-incident.md)
for the motivating evidence, owner-reported impact and correction handoff.
