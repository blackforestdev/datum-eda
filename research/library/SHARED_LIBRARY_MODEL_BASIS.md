# Shared engineering library amendment basis

Date: 2026-11-16. Task: `dat-shared-library-spec-vvtz`.
Scope: owner-directed specification reconciliation; no runtime delivery claim.

## Owner requirements

The owner requests a unified editable library that reuses electrical symbols,
physical footprints and device-specific mappings; common asset identity,
authorship/origin/modification records and version history; vendor and community
contributions reviewed against factual evidence; GitHub as collaboration host;
checks connected to ERC/DRC and CAM publication; reliable fast searching without
individual browsing surveillance. Specification completion precedes integration.

The owner authorized specification changes through the active PM after review of
the isolated proposal. PM reserved decision 066; QA/spec transferred append-only
consumer sections and retained all preceding QA013 evidence. No statement here
ratifies a numerical standards requirement, source-content license or dependency.

## Existing product basis and delta

PM008/008A already define the reusable graph and pinned references. PM034 keeps
Git optional transport and Datum authority local; PM038 keeps formal governance
opt-in. PM009 owns check runs/findings. PM061 owns effective mask/paste geometry
and unknown-basis distinctions; PM062 preserves pending placement/pool choices.
The amendment links these authorities rather than replacing their settled rules.

Inspected implementation: `pool/mod.rs` defines distinct objects and a SQLite
PoolIndex; `api/mod.rs` initializes that index in memory; rebuild currently clears
and repopulates tables in `PoolIndex::rebuild_from_pool`; native pool changes use
`substrate/pool_journal_ops.rs` staging and journal machinery. These facts justify
explicit consistency/recovery requirements, not an assertion of corruption or
adequate large-library performance. `pool/symbol.rs` has optional source/review
fields while `pool/footprint.rs` lacks the same common provenance envelope.
The new common record and portable revision exchange are targets, not existing
features proven merely by local technical counters or source hashes.

## Reviewed baseline and boundaries

Reused prior library complete-route evidence in
`docs/reviews/workflow-delivery-rollout/library-surface-route-preflight.json` and
`surface-specification-combined-review.json`; current QA013 baseline is recorded
in its task and spec-reconciliation recovery. Amendment review must assess only
the new target clauses and affected claims while preserving original proof pins.
Existing accepted prototype states, historical authors and runtime readiness
remain unchanged. No prototype is edited or newly accepted by this amendment.

Remaining decision boundaries and exact implementation dependencies are stated in
the governed specification, not resolved by this evidence note. The numbered
decision and contract are the authority; this file records the basis only.
