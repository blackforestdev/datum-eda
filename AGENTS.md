# AGENTS.md — Datum EDA agent entry

`CLAUDE.md` is controlling. Read it at startup; this file adds tracker and shared
workspace execution rules. Load `docs/AGENT_OPERATIONAL_PROCEDURES.md` sections
only before the applicable action. Do not preload the documentation inventory.

## Portable Datum workflows

<!-- DATUM-WORKFLOW-CATALOG:datum://workflows -->

`datum://workflows` is the canonical inventory. This guide does not redefine
workflow capabilities, review gates or mutation semantics.

## Delivery and authority

Deliver complete designer behavior through build, test, fix, document and required
independent review (PM059). Continue routine work under its existing grant; do not
stop after administrative updates. Reuse adequate research/proof and capture
unrelated defects without making them new prerequisites. Select product execution
by default; planning detours identify the blocked workflow and finite exit. Keep
one task record and combine routine evidence/state updates with delivery; PM059
retains actual independent review without a receipt set for every ordinary step.

Default to usable feature implementation; CLAUDE.md requires at least 80% of
effort on actual Rust product code, not a percentage of commit labels. Read the selected milestone's
`feature_delivery` contract. Use ordinary application workflows as delivery proof;
isolated fixtures supplement them. Do not yield merely after a document, test,
commit or lease update. Continue within the grant until delivery or a concrete
authority, safety, review/transport or ownership blocker; explain its exact remedy.
Full qualification remains open separately. Never invent an execution grant or PASS.

A verified standing project-manager appointment routes covered project decisions
to that manager across current and future tasks. Do not request a new owner grant
for each phase, acceptance or successor decision already covered by that appointment.
Unclassified decisions go to the manager for classification before disposition
only when its appointment includes that capability; otherwise they go to the
owner. Explicit owner_only decisions and uncovered authority also go to the owner.
Permission never supplies proof, a human observation or independent PASS. PM059
controls scope, revocation and retained boundaries; no self-appointment or automatic
successor selection is authorized.

Read-only audits/research are exempt from implementation claims and selector
success; disclose unrelated failures and continue. This grants no production edits,
acceptance or canonical-next claims. Tracked planning still needs authorization.

## Canonical task and ownership

Run fresh `python3 scripts/project_status.py next` for roadmap next-milestone queries
and fresh `python3 scripts/project_status.py details [key|issue]` for completion
queries. Use the compact default output and give a concise plain-language answer:
the selected task, next step and any blocker or required decision. Fetch `--full`
or `--json` only when the task needs that detail; paste raw output only when the
user explicitly requests it. Never reconstruct from memory or `br ready`.
Report selector failure and stop the query. The printed next completion step is
sole milestone-selection authority; dependency readiness grants no work. Routine
assignments use the verified manager's recorded PM059 grant, never tracker priority.

For routine work, record the manager's bounded authorization, implementer and
file scope in one task before editing; update its assignee/status and export.
No duplicate Frontier lease or completion plan is required. Scheduled milestone
execution retains its structured Frontier/beads claim. Respect all scopes and
other sessions' dirty work; transfer ownership explicitly. A tracker claim alone
is never execution authority.

If output reports `INPUT REQUIRED`, stop the affected milestone decision and present
the decision concisely. Read the full packet before requesting a disposition;
preserve request IDs, option meanings and exact response tokens. Route it to the owner or
the appointed manager only within PM059 delegation. Do not claim or advance until
the authorized disposition is durably recorded and state synchronized. The PM
continues separately authorized routine work; owner absence is not a project hold.

Detailed query/response, boundary, state, lease and completion procedures are owned by
`docs/PROJECT_STATE_POLICY.md`; ordinary command recipes are in the procedures.

## Tracker

Use beads_rust `br`, never incompatible `bd` (do not install it). Canonical data:
`.beads/issues.jsonl`; SQLite/daemon files are ignored and never committed.
Set `BR_AGENT_NAME=codex BR_HARNESS=codex-cli BR_MODEL=<actual model>` and use
`--actor codex` for tracker audit only, never git attribution.

Capture incidental defects with `br q`/`br create`, link actual dependencies and
continue. Close with landing evidence; `br sync --flush-only` before committing
tracker changes. Tracker intake is not roadmap priority. Load the tracker section
of the procedures before unfamiliar mutations.

## Shared-worktree commits and protected files

- Inspect `git status`; stage only owned paths/hunks. Never `git add -A`,
  `git add .` or `git commit -a`. Do not sweep another session's changes.
- Direct to main; no branches/PRs. Each conventional subject needs a substantive
  body explaining problem, change, proof and roadmap/decision/issue impact.
  Governance/closure commits identify disposition, outcomes and retained boundaries.
- No AI attribution in commits, PRs, comments or documents.
- Force-push, history rewrite and `--no-verify` require explicit owner authority.
- Claude owns all `docs/gui/prototypes/*.html`. Codex never edits, reformats,
  renames, deletes or regenerates them. Inspect/screenshots are allowed. Send exact
  bounded reconciliation to Claude; never set/copy/forward or suggest bypassing
  the lane marker. A tripped gate stays with its responsible lane; do not bless
  unauthorized output by refreshing digests/parity/goldens.

Before spec/research/visual work, inspect the complete owning evidence route.
Reuse a verified unchanged baseline and review changed artifacts and affected
claims; reread the full route when the baseline is missing or meaning changes
across the route. Follow same-change classification, status, numbered
decision, parity and digest obligations in the procedures; code is not product
or visual authority.

## Verification, build and formatting

Make each verification batch justify its decision; reuse adequate evidence and
retain required acceptance/gates. New sessions alone do not justify reruns.

Run proof/full-workspace/Clippy/release/compatibility/smoke Cargo through
`python3 scripts/run_cargo_guarded.py --workload proof -- cargo ...`. Interactive
cycles may use `--workload interactive`. Disk-backed targets only; no parallel
proof builds or cache cleanup while Cargo/rustc is active. New compiling shell
runners must use the guard and pass `check_cargo_resource_policy.py`.

For selected Rust files use `python3 scripts/format_rust.py <paths>`, inspecting
status before/after. Never `cargo fmt -- <paths>` or mutating workspace-wide
formatting in a shared tree. `check_rustfmt.py --staged` and whole-workspace
`cargo fmt --all -- --check` are nonmutating checks.

PM029 forbids new third-party code without exact prior owner/license ratification;
terminal implementation has no exception. Run dependency authority checks before
landing dependency or terminal work. Source-health and visual ownership remain
blocking; detailed command/body examples live in the procedures.

## Session hygiene and context

Externalize state continuously (claims, logs, records, checkpoints); the context
window holds only the active working set. Checkpoint at every lane boundary into
one durable recovery file (state, exact next command), report clear-safe to the
project manager, and let the owner compact or clear the session; never carry a
finished lane's context into the next lane. No silent idles: send the manager a
one-line status (completed artifact, blocker, or what is awaited) before ending
any idle turn. Messages to heavy sessions reference file paths, never inline bulk
content. Delegate read-heavy, conclusion-cheap work (broad searches, long-log
digestion, census sweeps) to subagents that return conclusions so bulk content
stays out of the parent window; lane-critical judgment and implementation stay in
the owning session, and a subagent's own cost must be weighed against the context
it spares the parent. Use bounded task lists and field-filtered records (procedure
recipes); load history only for a specific unanswered question. Reuse verified
unchanged instructions, while refreshing current status and ownership. Recovery
files point to current task records; replace obsolete
next actions when work lands rather than keeping a second active-task queue.
