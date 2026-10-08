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
unrelated defects without making them new prerequisites.

Default to usable feature implementation; target roughly 80% of effort toward
that outcome, not a percentage of commit labels. Read the selected task's
`feature_delivery` contract. Use ordinary application workflows as delivery proof;
isolated fixtures supplement them. Do not yield merely after a document, test,
commit or lease update. Continue within the grant until delivery or a concrete
authority, safety, review/transport or ownership blocker; explain its exact remedy.
Full qualification remains open separately. Never invent an execution grant or PASS.

An owner-appointed manager may handle only PM059-delegated decisions. Route actual
review receipts, unchanged-scope continuation and objective acceptance to that
manager; new product rules, scope reductions, waivers, dependencies and nondelegated
HUMAN/UX decisions remain owner-only. No self-appointment, fabricated PASS or
automatic successor authorization.

Read-only audits/research are exempt from implementation claims and selector
success; disclose unrelated failures and continue. This grants no production edits,
acceptance or canonical-next claims. Tracked planning still needs authorization.

## Canonical task and ownership

Run fresh `python3 scripts/project_status.py next` for ordinary next-task queries
and fresh `python3 scripts/project_status.py details [key|issue]` for completion
queries. Return stdout byte-for-byte as the entire answer, with explicitly requested
analysis separately afterward. Never reconstruct from memory or `br ready`.
Report selector failure and stop the query. The printed next completion step is
sole authority; dependency readiness does not authorize alternate or parallel work.

Before scheduled editing, synchronize the structured Frontier lease and beads
status/assignee/export in one transaction. A tracker claim alone is invalid.
Respect scope, expiry and other sessions' dirty work. Expiry never silently
releases ownership; transfer explicitly.

If output says `Owner boundary: INPUT REQUIRED`, stop implementation and present
the ordered decision packet/response format unchanged. Route it to the owner or
the appointed manager only within PM059 delegation. Do not claim or advance until
the authorized disposition is durably recorded and state synchronized.

Detailed states, leases, completion contracts and transitions are owned by
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

Before spec/research/visual work, inspect the owning evidence route and review all
listed sources/consumers. Follow same-change classification, status, numbered
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
