# M6 Opening Charter

This file records the recommended opening boundary for `M6`. Progress-state
authority remains `specs/PROGRESS.md`.

## Objective

Open `M6` as the strategy-layer milestone on top of the completed `M5`
routing-kernel substrate, but start with one narrow read-only slice that maps
an explicit routing objective to the existing accepted selector/profile
vocabulary without reopening routing semantics.

## Recommended First Slice

Recommended first `M6` slice:
- deterministic routing-objective recommendation/reporting on top of the
  existing `route-proposal` selector lane

Practical shape:
- consume only the completed `M5` selector/profile surfaces
- accept a bounded routing objective set that reuses existing selector profile
  names
- report one recommended selector profile, the deterministic mapping rule, and
  the current live selector result under that profile
- compare that same accepted objective/profile set without introducing new
  objectives or profiles
- remain read-only

## Non-Goals

The opening `M6` slice should explicitly avoid:
- new pathfinding or geometry generation
- new routing candidate families or selector scoring
- placement-kernel behavior
- free-form intent parsing
- AI-authored constraints or invented objectives

## Entry Criteria

Before coding the first `M6` slice:
1. `M5` is treated as closed for routing-kernel scope in `specs/PROGRESS.md`.
2. The first `M6` slice must reuse accepted `M5` selector/profile vocabulary.
3. The first slice must be read-only and deterministic.
4. Acceptance checks must be written up front.

## Acceptance Shape For The First Slice

The first `M6` slice should satisfy all of:
- deterministic objective-to-profile mapping
- explicit explanation of why that profile is recommended
- no new routing semantics
- focused CLI/MCP proof coverage

## Initial Contract

Initial contract selected on 2026-03-30:
- canonical surface:
  `project route-strategy-report <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--objective <objective>]`
- comparison surface:
  `project route-strategy-compare <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- decision-delta surface:
  `project route-strategy-delta <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- batch evaluation surface:
  `project route-strategy-batch-evaluate --requests <path>`
- saved batch result inspection surface:
  `project inspect-route-strategy-batch-result <path>`
- saved batch result validation surface:
  `project validate-route-strategy-batch-result <path>`
- accepted objective set:
  - `default`
  - `authored-copper-priority`
- output shape:
  - one recommended selector profile from the accepted profile vocabulary
  - one explicit deterministic mapping rule
  - one current selector status/candidate summary under that profile
  - one next-step recommendation pointing back to `project route-proposal`
  - one comparison report over the same accepted objective/profile set that
    includes per-entry availability, selected candidate family when present,
    concise distinction text, and one deterministic recommended profile
  - one decision-delta report over that same accepted set that includes
    compared profiles/objectives, outcome identity vs difference, per-profile
    selected candidate/policy when available, one bounded explicit delta
    classification, one recommendation summary, and one short material
    difference explanation
  - one versioned batch request manifest format that evaluates explicit route
    requests across one or more fixtures/projects by reusing the existing
    report/compare/delta surfaces
  - one aggregate batch summary with total evaluated requests, recommendation
    counts by profile, delta classification counts, same-vs-different outcome
    counts, and proposal-available vs no-proposal counts
  - one explicit saved batch result artifact format with:
    - `kind = native_route_strategy_batch_result_artifact`
    - `version = 1`
    - the original batch-evaluate per-request evidence and aggregate summary
  - one read-only inspection/validation workflow for saved batch result
    artifacts that reports artifact identity/version, distributions, per-request
    outcomes, malformed entries, version compatibility, required-field
    coverage, and summary/result count integrity
- paired MCP surface:
  - `route_strategy_report`
  - `route_strategy_compare`
  - `route_strategy_delta`
  - `route_strategy_batch_evaluate`
  - `inspect_route_strategy_batch_result`
  - `validate_route_strategy_batch_result`
