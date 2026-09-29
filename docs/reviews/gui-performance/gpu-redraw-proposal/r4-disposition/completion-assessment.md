# GPU completion assessment at the R4 disposition

The GPU repair is incomplete. R4 passed its bounded GPU proof, but its first
native launch crashed before the pointer workload. The approved first-failure
rule ended the campaign. One GPU batch, one candidate release, one matched
baseline release and one invalid native attempt were consumed. No replacement,
remaining trial, production correction or successor campaign followed. The
owner requested this stop and an end-to-end assessment; this document supplies
that assessment, not a new execution proposal. S4, S5 and the main issue remain
open. The current R4 production source is not a qualified native candidate.

The exact declarations, binary hashes, build logs, process result and failure
logs are preserved by `receipt.json` and the adjacent artifacts. The candidate
renderer source is `544dc61a`; campaign-method binding is `81614624`.

## Renderer and integration defects

The original architectural defect was real: pointer-only interaction could
execute the full composed scene despite retained CPU content. The shared
RenderSession now owns dependency invalidation, prepared content, presented
versus desired damage, image validity and completion. Editors submit typed
changes. Native integration still owns surface acquisition, presentation and
queue completion; those are necessary platform responsibilities.

Earlier repairs also removed coarse all-object-hover invalidation where only
pad material actually depended on hover, and replaced fragment-discard-only
suffix restriction with shared painter clips. The last valid prior native run
preserved exact output and visible crosshair, completed the pointer workload,
and had no world uploads across all 14 observed non-pad transitions. These are
supported improvements, not proof that every legacy path was defective.

R4 replaces the remaining full-target warm multisample attachment/resolve with
32-pixel tiles in a fixed atlas and a retained resolved image. Eligible warm
frames restore and repaint whole selected tiles, hardware-resolve the atlas,
copy changed cells into retained C, then copy all C to the acquired target.
They avoid full A-to-B restoration, full-size B rendering and full-target 8x
resolve. Empty damage copies C only. Genuine dependency changes, unsupported
conditions and capacity overflow retain complete cold/fallback rendering.
Source implementation and bounded GPU proof establish this mechanism; no valid
native performance result establishes its benefit yet.

**The unresolved defect is a missed R4 native integration consumer.**
`native_attachment_ledger.rs:46–50` has `current = [None; 3]` and indexes it once
per supplied allocation. R4 supplies five current images and can describe ten
current/submitted union entries (`gpu_surface.rs:55–73`). The fourth entry
panics. `native_render_target::finish` calls
`SurfaceTransaction::observe_attachment`, then
`QueueOwner::observe_attachments`, then this ledger unconditionally. It is not
conditional on timestamp or verbose diagnostics. Thus disabling measurements
would not repair this source mismatch. The three-slot assumption came from the
repair's earlier pair implementation, not the original legacy redraw audit.

The native log records this panic first, followed by in-use swapchain semaphore
destruction, a poisoned ledger and abort during cleanup. These follow-on errors
do not establish a separate driver root cause. No full frame was successfully
qualified. The startup screenshot is black. The owned process group was empty
at cleanup and the candidate binary was unchanged.

This is a production resource-accounting integration defect, with a separate
validation-coverage gap: the four-group proof exercised renderer image lifetimes;
the native test checked surface capability without submitting the image bundle
through the native ledger. Neither reached the failing bridge. The implementation
and proof missed that consumer. It is not the earlier 4,096-record observer
exhaustion, and it does not justify a new streaming observer.

Tracked defect: `dat-r4-native-attachment-capacity-uquy`. A future correction must
make the native consumer honor the shared bounded image-set contract, preserving
current/retiring membership, exact last-use serials, close and device-loss
semantics. Raising three to five alone would not cover the ten-entry union.
Dropping entries or disabling accounting would violate the contract. This is a
source-supported finite correction, not evidence requiring another renderer
architecture. No correction or retry is authorized by this stopped packet.

## Measured budget failures

| Candidate and result | Complete active p95 | Complete active p99 | Meaning |
|---|---:|---:|---|
| Prior valid painter/hover candidate `453cd9f2` | 4.330583 ms; fails 4 ms | 5.701750 ms; meets 8 ms | Valid numerical failure over 1,277 active frames |
| R4 `544dc61a` | Unavailable | Unavailable | Startup integration failure; zero active pointer workload |

The earlier timing failure remains evidence about its pinned candidate. It
cannot be assigned to R4. The baseline was built with the original renderer,
existing matched observer and only the same final timestamp marker added;
it never ran. No paired speedup, instrumentation-overhead or CPU-regression
comparison is available. R4's complete span includes final presentation copies
by construction and GPU encoder proof, but there is no valid native span
population to assess. Tile-area arithmetic is not a performance result.

## Qualification completed and still missing

The single P630/Vulkan/exact8 proof passed in 16.62 seconds. It covered translated
samples versus the independent full painter, fractional geometry, first/last
atlas cells, seams/padding, wrong-origin/sample and painter negative controls;
Console/text/menu/terminal output; non-pad and real-pad changes; old damage,
coalescing/recovery and overflow; renderer five-image lifetime/admission/refusal;
and actual cold/warm/empty/fallback final timing markers, including 31/32 maximum
cold query use. Native X11 COPY_DST and 8x capability passed. Test-only sample
storage and scratch oracles were excluded from resource/performance claims.
Offline builds, default/visual Clippy and required policy checks passed.

Those successes do not cover the native ledger defect or establish end-to-end
resource correctness. The failed native attempt supplied no completed input
receipt/frame seal and no exact ready/final output. All eleven later trials
were cancelled under the immutable campaign's stop rule.

Beyond restoring that native bridge, completion still requires:

1. Correct native output, input lineage and bounded native image retirement on
   the corrected candidate, followed by valid pointer timing at p95 <=4 ms and
   p99 <=8 ms. A new authorized replay would reuse unaffected proof, not discard
   it or rerun an unchanged failed candidate. The interrupted R4 declaration
   cannot be resumed as though C-G1 had passed.
2. Complete DRM client/context lifetime accounting and engine duty <=25%.
   Existing source review found that request completion/GEM_BUSY idle and retained
   file descriptors do not establish final context-runtime accounting after
   destruction. The required observable accounting-finalization mechanism is
   unresolved. This gap does not prevent the focused pointer experiment but
   blocks final S4 acceptance. It is a separate method-development obligation,
   not simply one missing test run or an inference from own-queue timestamps.
3. Remaining camera, affected consumers/backends/scales, resource/observer
   conformance and overhead, endurance/recovery, distinct-reviewer native
   qualification, and the separate owner UX/product disposition required by
   the controlling S4/S5 contract. Preserve prior evidence only at its recorded
   pins and scopes. The bounded R4 packet could not close those obligations
   even if all twelve trials had passed. CPU optimization remains deferred;
   its separate event-loop issue was not investigated.

## Feasibility and overall path

Exact regional rasterization is demonstrated for the bounded tested cases;
reference-host copy support is demonstrated. The native ledger mismatch has a
concrete source explanation and bounded remedy, but the remedy is not implemented
or validated. Correcting it restores the ability to measure R4; it does not
itself meet the performance budget.

Whether R4 reaches the pointer budget remains unknown. Per-tile traversal,
copies and fallback frequency can offset the reduced resolve area. No hardware
floor, Wayland defect or impossibility has been established. If valid timing
still fails, the retained pass/copy evidence must determine whether another
renderer correction is justified; that is a conditional remaining task, not a
promised short final step.

Whether a sufficiently complete, low-perturbation DRM method is feasible on the
reference host also remains unknown. Source review has rejected specific proposed
barriers, not all possible methods. Exact deployed-driver reconciliation,
coverage, retirement/loss semantics and overhead remain to be established before
that method can qualify duty. Broader qualification on the corrected renderer
is also unproven, not presumed to pass.

The overall path is therefore native integration correction and bounded proof,
then pointer budget determination; separately, a viable complete-duty method;
then the remaining integrated qualification and owner disposition. R4 has
reduced uncertainty about regional pixel/sample correctness, but has not resolved
performance feasibility or the DRM method. The repair is not one final validation
away, and a defensible finish-date or completion percentage cannot be inferred
from the amount of implemented code. Work stops here at the requested disposition.
