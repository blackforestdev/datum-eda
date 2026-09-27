# Product Mechanics 048: Multisample scene-prefix retention

Status: ratified by the owner through approval of pinned GPU redraw proposal r1.
Issue: `dat-gui-performance-implementation-vkq`; Frontier: GPI-S4.

## Authority and scope

The owner replied `+ approve GPU redraw proposal r1`, recorded in
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r1.json`, then
`please proceed? what are you waiting for?` in response to reviewer delegation.
The immutable r1 proposal SHA256 is
`6b7ed1c58158b65dc9c635535362f78360a32a7427be9183b694d6363d816eee`.
Independent design delta review is recorded in
`docs/reviews/gui-performance/gpu-redraw-proposal/independent-review-r1.json`.
It finds no blocking rendering-mechanism defect. This decision ratifies only
that exact mechanism and its already-approved bounded implementation/proof.

## Mechanism

A shared Renderer may retain one exact 8x multisample image A of the composed
frame prefix ending immediately before schematic interaction. On a warm eligible
pointer frame, copy all samples to equal-format/extent/sample-count attachment
B, then draw the original ordered suffix once, resolving once. A is stored
without resolve; B loads copied samples and discards after its final resolve.
Initialize suffix pass state explicitly. Preserve legacy and dialog paths.
No 1x recreation, quality reduction, regional replay or alternate copy shader.

Explicit noninteraction composition and resource/host/device generations govern
reuse; only audited interaction refresh preserves composition identity.
Conservatively invalidate/fall back for nonpointer changes and uncertain frames.
Publish reusable state only after successful submission/presentation; preserve
pending damage on failure. Camera/layout/style/layer/model/extent/atlas/device
changes must not reuse stale prefix pixels. No scheduler change is authorized.

A is optional, actual-size, at most 45 MiB per host; key/metadata capacity is
at most 4 KiB. Count A and B separately under existing aggregate/local caps.
Admit the pair coherently as one attachment generation; at most one current
and one retiring generation. Track texture/view aliases once, retain all
submitted resources through completion, and preserve allowances across recovery.
Admission failure uses the existing correct full-frame path; it is no budget pass.

## Bounded proof and mandatory stop

The pinned proposal's four gates, exact cases, negative controls, fixed run
counts and first-failure rules are controlling. Gate1 implementation and focused
capability/fidelity/lifetime proof may proceed. Its independent question is
whether this mechanism preserves current output and ownership. It establishes
neither numerical GPU feasibility nor S4/S5 completion.

Independent finding IR-R1-01 confirms that current DRM endpoint observations
cannot establish complete client lifetimes, reset epochs and final drained
counters. This known Gate2 prerequisite remains unavailable. Stop before native
performance trials; do not repeat snapshots, promote endpoint duty, or expand
fd-lifecycle/system tracing work under this decision. Copy-inclusive GPU timing
cannot cure that separate accounting gap. Reuse the source evidence in the review.
The gap does not prohibit approved Gate1 work. IR-R1-02 requires explicit suffix
pipeline/binding state and exact-output cases for its first nonempty stage.

Preserve 4 ms p95, 8 ms p99, 25% duty and all other PM045/PM047 requirements.
No dependency or licensing change (PM029), protected prototype edit, CPU event-loop
work, backend/tuning experiment, replacement renderer or alternative campaign is
authorized. Missing/failed proof remains failed or unqualified. S5, broader
qualification, independent native replay and owner product acceptance remain open.
Rollback uses an isolated revert preserving historical evidence and unrelated work.
