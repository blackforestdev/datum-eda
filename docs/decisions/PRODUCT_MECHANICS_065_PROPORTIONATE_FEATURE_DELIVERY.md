# Product Mechanics 065: Proportionate feature delivery

Status: owner-directed amendment under `dat-bounded-rust-feature-delivery-jbzc`.

The owner directed implementation of the delivery audit corrections, with actual
Rust feature work taking priority over administrative machinery. This amendment
narrows PM022's touched-file and protected-review requirements; it creates no
new acceptance scheme, waiver registry, execution grant or qualification claim.

## Bounded corrections in existing oversized modules

An independently reviewed, bounded defect correction recorded in the existing
task may leave a legacy module the same size or smaller without extracting
unrelated behavior. It must not grow any applicable measured dimension against
the actual base: physical, production, inline-test or textual-include expansion.
The task retains the defect, owned scope, affected behavior checks and actual
independent review. The size checker establishes measurements, not defect intent
or review truth; a green result alone does not authorize this path.

This supersedes SH-006's unconditional extraction requirement only for that
bounded correction. Real decomposition remains subject to SH-007: cohesive
ownership, behavior preservation and no forwarding, textual or cosmetic split.
Other changes to legacy debt retain the decomposition obligation. Normal source
budgets, no new oversized files or legacy entries, complete source discovery,
frozen ceilings and exact downward ratchets remain. Every smaller debt metric
ratchets in the same change; compliant modules leave the existing debt ledger.

## Owner-controlled landing and proportionate verification

Protected source-health policy changes require explicit owner authorization in
the existing task and actual independent review of the final change. Landing is
directly on main under the existing owned-staging rules. This supersedes PM022's
mandatory pull-request/remote approval mechanism; CODEOWNERS remains the ownership
map, not evidence that approval occurred. Repository files cannot establish remote
ruleset enforcement. Do not claim external protection or approval without observing
it, and do not weaken policy or bypass a failing gate merely because direct-main
landing is permitted. PM022 and this amendment remain immutable after landing;
later changes require a later numbered amendment.

Before landing, run the existing source-health checker with a trusted base against
the exact proposed candidate, using an isolated checkout with `--root` and
`--base-ref` when other sessions have unfinished changes. Preserve the complete
working-tree scan as shared-health diagnostics and the complete clean-checkout CI
gate; unrelated dirty work is attributed to its owner, not swept into the candidate.
No new staged-hook engine or proof ledger is required.

Run affected behavior checks for ordinary Rust corrections and retain independent
review. Source-health checker/policy changes require their focused regression
suite; unchanged checker regressions need not rerun for each product edit. Broad
qualification remains a separately designated milestone, never a substitute for
demonstrating the ordinary designer workflow.
