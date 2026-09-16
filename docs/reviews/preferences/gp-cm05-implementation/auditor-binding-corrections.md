# GP-CM05E auditor binding corrections

The owner supplied two reproducible validator findings against c243eb5e:
reported measurements could disagree with their linked captures, and a case
command plus matching log could be replaced with `true` without executing the
required fixture recipe. The original 31 regressions did not detect either.
This correction remains within the existing claimed GP-CM05E implementation.

## Reproduction and correction

Two new regression methods were first run against the unfixed code. They failed
with six missed-refusal assertions: five contradictory capture variants and the
matching no-op command/log. The retained run is
`/tmp/datum-gp-cm05-audit-red.log` (2 tests, 6 failures, 6.818 seconds).

Measurement evidence now has a closed, typed capture schema. Each capture must
select the exact measurement/candidate/environment, trial and phase, record a
zero exit status without timeout, and agree with every reported observation.
Timing/RSS, daemon RSS, presentation acknowledgement, storage state/counts and
lifecycle resources are all verified before their existing limits apply.
Measured operations, warm-ups and cold opens use the same binding. A passing
outer trial or copied trial log cannot override failed or contradictory evidence.
Old incomplete captures are refused; production capture cannot be reconstructed
by copying claimed report values into a new success artifact. Only the synthetic
test fixture builder generates matching artificial captures for regressions.

Case argv is derived from the verified candidate recipe and the complete frozen
fixture: `python3 <recipe.path> --fixture-json <canonical fixture JSON>`.
The validator compares both the case command and its captured log with that
invocation. All fixture parameters, including variant, subcase, surface and
native coordinates, are passed as data. No-op commands, alternative recipes,
shell wrappers, dropped/changed parameters and extra bypass flags are refused.

The contract documents both bindings and registers MeasurementCaptureV2.
The owning route reconciliation preserves every other source/consumer and the
11 active / 45 reserved / eight individual Units seed boundary. No prototype,
product runtime, dependency, installed trust or owner receipt is changed.

## Verification

The four targeted regression methods passed after correction (92.075 seconds),
including all three sample kinds, all three phases, execution identity changes,
and substituted commands with internally matching logs. Retained output:
`/tmp/datum-gp-cm05-audit-green.log`.

All 36 tests in the complete regression suite passed in 452.519 seconds;
the run is retained at `/tmp/datum-gp-cm05-audit-full.log`.
Matrix, Preferences boundary, source health, spec governance, parity,
traceability, progress, dependency authority and Cargo-resource checks passed.
Project-state validation passed for all 57 items with the existing live claim.

These are evidence-validator corrections, not new product proof or a finding
that final owner acceptance could be bypassed. GP-CM05E remains unfinished and
GP-CM05A remains pending. No successor task is selected or started.
