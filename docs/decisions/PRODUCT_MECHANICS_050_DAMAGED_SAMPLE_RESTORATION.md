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

## R3 lineage corrective validation disposition

The owner explicitly replied `approve GPU r3 lineage corrective validation`.
The pinned packet and disposition are recorded in
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3-lineage.json`.
Execute its one named GPU case once; only on pass proceed to the new fixed
bounded native declaration. The earlier stopped campaign stays unchanged.
Preparation may advance while immutable submitted lineage and strict target
identity remain intact. No new renderer mechanism, DRM execution, CPU work,
numerical acceptance or S4/S5/issue closure is authorized.

## R3 crosshair evidence replay disposition

The owner replied `approve GPU r3 crosshair evidence replay`. The pinned packet
and exact disposition are recorded in
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3-crosshair.json`.
Authorize exactly one candidate diagnostic replay with the existing source-bound
binary and fixed new campaign identity/cap1. Preserve any output failure while
collecting bounded shutdown/input evidence. All prior campaigns remain stopped.
No new build, renderer variant, further native trial, DRM/CPU execution, numerical
acceptance or S4/S5/issue closure follows from this approval.

## Shared painter and hover correction disposition

The owner explicitly replied `approve GPU painter and hover correction`.
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-painter-hover.json`
pins the exact proposal at SHA256 `470653c6e688be4ddd403156d677b2212784a8e2cf1121708c647697b7da84de`.

Amend the warm suffix rule to bounded disjoint integer damage traversals inside
one render pass: each damaged pixel receives the original painter order exactly
once, with one final exact8 resolve. Every painter intersects its own clip with
the shared region. Keep the32-rectangle cap, shared shader predicate, full fallback
on normalization/admission refusal, immutable damage and submission lifetimes.

The shared renderer classifies actual pad-dependent material/text hover from
source-bound membership; non-pad hover does not invalidate retained world data.
Real pad-material transitions remain cold. Metadata admission, accounting and
retirement use existing document budgets; no new texture, query slot or raised cap.

Execute only the pinned proposal's bounded implementation and proof scope. The
previous timed campaign remains stopped. Preserve all original camera/resource,
complete DRM duty and independent qualification obligations. No CPU event-loop
optimization, DRM observer execution, dependency change or S4/S5 closure.

## Regional composition r4 owner disposition

The owner explicitly replied `approve GPU regional-composition proposal r4`.
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r4.json` pins
the approved proposal and exact response. Required independent design review
remains a prerequisite to implementation; this disposition is owner ratification,
not a claim that review or runtime proof has passed.

Amend the warm graph as specified in the pinned r4 packet: shared RenderSession
retains single-sample composed C, reconstructs conservative 32x32 source tiles
in a fixed 512x512 exact8 atlas, paints the common ordered suffix once per tile,
resolves the atlas once, and copies changed cells to C and all C to the acquired
copyable target. Deduplicate at most256 tiles; preserve checked refusal/full
fallback, exact sample/clip/painter semantics and completion-driven validity.
Remove the competing warm full-size B restoration and exposure-resolve paths.
Required B remains allocated and charged for the cold/full fallback graph.

The optional per-generation allowance changes from one required-image payload
to an aggregate45MiB for A+C+T+R. At1280x800, optional payload is44.15625MiB and
the two-generation five-image payload is150.8125MiB. Preserve the512MiB process
GPU cap,4KiB image-metadata cap, submitted references and one current/one retiring
coherent bundle. All mapping/staging and other resources remain separately
charged. No allocation or driver-residency reduction is implied by idle B.

Use the explicit shared full-texture COPY_DST presentation contract with
capability fallback. Include all atlas/composition/presentation copies before
the final timestamp marker on candidate and matched baseline. Preserve existing
3slots/32queries and input observation capacity; no new observer framework.

Execute only the pinned packet after its review prerequisite: focused offline
checks, one four-group GPU batch, conditional one candidate and one baseline
release build, and one fixed twelve-trial pointer campaign with first-failure
stops. All previous campaigns remain stopped. Exact output, performance,
resource, camera, endurance and independent qualification remain unproven at
their original scope. No CPU/DRM execution, dependency change, tolerance refresh,
parameter sweep, S4/S5 acceptance or issue closure is authorized.
