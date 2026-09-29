# Product Mechanics 050: Retained composition and damaged-sample restoration

Status: ratified by explicit owner approval of pinned GPU correction r3.
Issue: `dat-gui-performance-implementation-vkq`; Frontier: GPI-S4.

## Authority and scope

The owner replied `approve GPU damage-restoration proposal r3`.
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3.json`
records the exact response and immutable proposal-r3.md SHA256
`716389b40bfafa5ebcbcf628af835d29309bf96c33b1091248871df6df6ac7b4`.
That proposal controls implementation, proof bounds and stopping conditions.
Independent review is advisory; the owner response supplies ratification.
This amends PM049's incremental graph and bounded measurement scope, preserving
its shared rendering ownership, original budgets and all225requirements.

## Retained composition

The shared RenderSession owns old presented/new desired pointer support and
retained composed B validity alongside prefix A. Editors submit typed changes.
Use at most32 conservative clipped integer rectangles; unknown, overflow or
stronger changes require the full graph. Coalesced input uses presented support.

For valid warm pointer frames, restore corresponding samples from A through
matching nonsRGB UNORM alias views into B using multisampled textureLoad and
sample_index. Restore only integer damage scissors, no blending/filtering/gamma
round-trip. Paint the common ordered suffix once with one shared damage-union
predicate across screen, glyph and terminal pipelines. Store B and resolve once.
Preserve shader derivative/sampling behavior and exact8x painter order.
Cold frames retain full prefix draw/copy/suffix; refusal retains full fallback.
Exposure-only frames may resolve valid composed B without suffix replay.

Only matching successful completion validates retained composition. Failed or
uncertain encoding/submission/presentation invalidates B even if its pixels were
already written. Preserve full reconstruction, stale-receipt rejection, pending
damage and live/retiring submission holds. Material-hover changes remain cold.

No new image or raised cap: A<=45MiB, image metadata<=4KiB, current plus retiring
coherent pair, process GPU cap512MiB. Alias views count once. Damage uniforms and
queued generations are charged and cannot be overwritten while submitted.

## Measurement and proof

Instrument every world/glyph/terminal continuation and final mixed upload before
rendering. One frame identity spans submissions; retain host/device/attempt and
submission lineage. Report first-start/last-end span separately from pass sums
and transfer intervals. Existing3slots/32queries remain: at most5continuations,
then final leading marker before mixed upload plus final graph and scene markers.
Missing boundaries, overflow, cancelled/lost lineage or invalid timestamps fail.
Use explicit workload/frame attribution; log arrival is not work identity.

Run the proposal's offline controls, one focused exact-sample/output/resource
batch and one measurement-conformance batch before any performance experiment.
Then one pinned candidate, at most12pointer runs and4conditional pan runs with
fixed input, original thresholds, first-failure stop and no replacements. No
parameter sweep, tolerance/golden refresh or restart of the stopped r2 sequence.

Complete DRM duty remains unavailable until the separately reviewed observer
method and execution scope are approved and proven. This decision authorizes no
ptrace, retained-fd observer, global tracing, system tuning or dependency change.
CPU event-loop optimization stays deferred. S4/S5 and the GPU issue remain open
until all required correctness, performance and resource evidence exists.

## Corrective validation disposition

After the r3 stable-hover reuse failure, the owner explicitly approved
`approve GPU r3 corrective validation`. The source-pinned scope and response
are in `gpu-redraw-proposal/owner-approval-r3-corrective.json` under
`docs/reviews/gui-performance/`. The four-case correction validation preserves
the first-failure stop, earlier evidence and all original numerical/DRM/CPU
boundaries. It ratifies no different rendering mechanism.
