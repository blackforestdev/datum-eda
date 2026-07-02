## PR Lane

- [ ] `m4-feature` (functional milestone work)
- [ ] `decomp` (structural decomposition only)

## Lane State (Required)

- [ ] `active`
- [ ] `closed`
- [ ] `blocked-awaiting-contract`
- [ ] `switched`

## Required Policy Declarations

- [ ] I confirm this PR follows the selected lane and does not mix feature and decomposition work.
- [ ] If this is a `decomp` PR: this PR is structural-only (no intended behavior changes).
- [ ] If this PR touches a known monolith file: the file shows burn-down with structural extraction evidence (no compression-only line shaving).
- [ ] If lane state is `closed` or `blocked-awaiting-contract`: this PR includes one explicit next branch decision (`define-contract` or `switch-slice`) and does not continue same-lane exploratory audit work.
- [ ] If lane state is `blocked-awaiting-contract`: this PR includes a single `fallback_slice` target.
- [ ] I ran required local gates:
  - `bash scripts/run_drift_gates.sh`
  - `cargo test --workspace`

## Scope

Describe the exact write set and why it is needed.

## Risk

List any drift/scope risks and how they were mitigated.

## Verification

Summarize test and gate results.
