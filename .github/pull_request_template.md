## PR Lane

- [ ] `feature` (functional product work)
- [ ] `decomp` (structural decomposition only)
- [ ] `governance` (governed mechanism/specification work)

## Lane State (Required)

- [ ] `active`
- [ ] `closed`
- [ ] `blocked-awaiting-contract`
- [ ] `switched`

## Required Policy Declarations

- [ ] I confirm this PR follows the selected lane and does not mix feature and decomposition work.
- [ ] If this is a `decomp` PR: this PR is structural-only (no intended behavior changes).
- [ ] If this PR touches legacy source debt: every applicable limit ratchets to
      the exact lower count and the diff contains real module ownership
      extraction (no `include!`, forwarding, continuation, or compression-only split).
- [ ] No new source file exceeds its decision-022 normal budget and no new
      legacy-ledger entry was added.
- [ ] If roadmap or tracker state changed: `active_frontier.json`, its generated
      `PROGRESS.md` projection, beads export, governing docs, and claim/closure
      evidence were synchronized under decision 025.
- [ ] If lane state is `closed` or `blocked-awaiting-contract`: this PR includes one explicit next branch decision (`define-contract` or `switch-slice`) and does not continue same-lane exploratory audit work.
- [ ] If lane state is `blocked-awaiting-contract`: this PR includes a single `fallback_slice` target.
- [ ] I ran focused checks for the changed behavior and affected safety boundaries,
      and source-health checks for changed source. Checker regression suites run
      when those checkers change; state checks run when structured state changes.
- [ ] Routine structural verification uses `bash scripts/run_drift_gates.sh --focused`.
      Exhaustive `--full`, workspace tests and end-to-end/statistical campaigns run
      only for a designated major milestone, as specified in `CLAUDE.md`. Any focused
      results below are identified by their actual scope and do not claim qualification.

## Scope

Describe the exact write set and why it is needed.

## Risk

List any drift/scope risks and how they were mitigated.

## Verification

Summarize test and gate results.
