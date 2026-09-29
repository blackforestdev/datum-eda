# R4: bounded regional composition and retained resolved output

Status: concrete proposal, not ratified or execution-authorized.
Issue: `dat-gui-performance-implementation-vkq`; completion step: GPI-S4.
Source baseline: `b546e3b4`. This replaces the warm r3 image graph, preserving
the shared RenderSession, exact8 painter, actual-pad dependency correction,
original numerical acceptance and all unfinished S4/S5 obligations.

## Reason and alternatives

The valid painter/hover trial has exact output and no active world redraws, but
p95 4.330583 ms fails 4 ms. The existing draw-free startup frame spent 3.665917 ms
in the suffix interval. `resolve-source-assessment/assessment.md` pins that
observation, its limitations and installed wgpu constraints. It establishes
cost without suffix draws; it does not establish an active percentile, a hardware
limit or a predicted speedup. Do not run another diagnostic to rediscover it.

Further glyph culling leaves full-target attachment/resolve work. Retaining only
resolved output helps exposure but leaves active pointer resolves. A custom
shader average risks changing sRGB/rounding behavior. Direct backend-specific
regional resolve would introduce a second unsafe resource/transition path.
Select a shared wgpu regional atlas with the existing hardware resolve instead.
This is renderer work, not an observer redesign or CPU event-loop optimization.

## Fixed layout and shared ownership

Use 32x32 pixel source tiles and a fixed 512x512 atlas (16x16 cells, 256 slots).
These are one proposed configuration, not sweep parameters. The layout limits
warm resolved area to 262,144 pixels and keeps the optional image payload within
45 MiB at the existing 1280x800 reference extent. It also makes all translations
multiples of 32, preserving pixel and 2x2-quad parity. This reduces a correctness
risk; it is not proof of exact raster/interpolation equivalence.

The shared owner derives touched tiles from the existing conservative union of
old successfully presented and new desired support. Deduplicate tile IDs and
sort by source row/column; assign atlas cells in that order. Keep at most 256
u32 source IDs in fixed storage, with checked arithmetic and no per-frame maps
or allocation-growing tile cache. Source tile coordinates derive from the ID
and ceil(width/32); atlas coordinates derive from slot index. Existing input
support remains bounded by 32 rectangles. Unknown support, rectangle overflow,
more than 256 tiles, unsupported format/target, or admission refusal uses the
full graph. Never truncate damage or omit a frame from timing.

Tiles are a conservative expansion of damage. Reconstruct and repaint each
entire selected tile, intersected with the target extent. Do not restore a whole
tile while keeping the old narrow damage predicate: that would erase unchanged
suffix content inside the tile. The atlas painter uses a tile-valid predicate,
not the original strip union. Source tiles are disjoint and each pixel receives
the original painter order exactly once. Editors continue submitting typed
changes; neither editors nor native backends own tile or retained-output state.

## Images and frame graph

Keep required full-size eight-sample B for cold/full fallback. Keep optional
full-size eight-sample prefix A. Add optional full-size single-sample composed C,
eight-sample atlas T and single-sample atlas R. T and R are 512x512, same format as
the target. A and T have matching UNORM alias views for sample restoration;
normal suffix drawing and hardware resolve retain the existing target format.
C/R copies preserve encoded bytes. No shader averages or color conversions.

**Cold with admitted regional resources:** render prefix into A, copy all samples
A to B, paint the full original suffix into B, and hardware-resolve into C.
Discard B after the pass. Copy all of C to the acquired presentation texture.
Commit A/C validity only through the existing successful matching completion.

**Warm nonempty damage:** one restoration pass writes each selected cell of T
from the matching A samples using textureLoad/sample_index through UNORM aliases,
with no blending. Clear T once at pass start so unused cells are initialized;
valid edge pixels read A, unused padding stays initialized. One suffix pass loads
T, traverses the common ordered painter per selected tile, resolves all T into R
once, and discards T. Copy each valid resolved cell rectangle R to its matching
position in C. Copy all C to the acquired target. There is no full-size B pass,
full-size A-to-B copy, full-target eight-sample resolve, or world redraw.

**Warm empty damage:** copy valid C to the acquired target only. Do not touch A,
B, T or R. Keep normal acquisition/presentation and completion semantics.

**Fallback:** run the existing full painter into required B, resolve directly to
the target and invalidate optional retained composition. Suppress optional
reallocation for that target generation after admission refusal. Capacity
overflow may reconstruct A/C through the cold path if already admitted; it may
not run the removed full-target warm r3 restoration path as another optimization.
All fallback work remains inside the complete frame timing and resource totals.

## Coordinate and presentation contracts

Keep original prepared vertices, glyph instances and projection uniforms.
For source tile origin s and atlas cell origin a, set viewport origin a-s and
the original full-target viewport extent. This translates raster coordinates
without rewriting geometry. Intersect each painter's original clip in source
coordinates first, then translate that intersection into its atlas cell.
Preserve Console/text/menu/terminal order through one shared painter.

Audit every suffix shader use of framebuffer position: damage/clip predicates
must use the intended atlas/source coordinates explicitly. Do not translate
UVs or world coordinates. Sample before conditional discard where derivatives
require it. Even tile origins avoid shifted quad parity; exact samples, glyph
interpolation and raster precision must still pass the proof below. There is no
tolerance or golden refresh if translation changes output.

Extend the shared target contract to explicitly describe a full-size copyable
texture target, rather than inferring a whole texture from an arbitrary view.
Native integration requests COPY_DST only when SurfaceCapabilities permits it,
and supplies the acquired texture, identity, extent and format. The shared
renderer performs C-to-target copy. Offscreen session targets use the same
contract and COPY_DST usage. Unsupported targets use full fallback; if the
reference host lacks this capability, stop before the native experiment.
Do not add an alternative sampled presenter, platform whitelist or private Vulkan
path within this packet. Present every pixel of each newly acquired target;
swapchain contents are never retained-composition authority.

## Resource ledger and recovery

RGBA/BGRA8 logical payload, reference extent 1280x800:

| Allocation | Required/optional | MiB per generation |
|---|---|---:|
| B: full-size exact8 fallback | required | 31.25 |
| A: full-size exact8 prefix | optional | 31.25 |
| C: full-size resolved composition | optional | 3.90625 |
| T: 512x512 exact8 atlas | optional | 8 |
| R: 512x512 resolved atlas | optional | 1 |
| Total optional | | 44.15625 |
| Total image payload | | 75.40625 |
| Two coherent generations | | 150.8125 |

The current implementation's optional reservation is one B payload (31.25 MiB
here), even though prefix eligibility permits up to 45 MiB. This proposal
explicitly changes per-generation optional admission to an aggregate 45 MiB
ceiling for A+C+T+R. Reference optional payload increases 12.90625 MiB; the
two-generation image peak increases 25.8125 MiB over the existing pair. This is
an owner-visible mechanism/admission amendment, not an unchanged allocation.
The process GPU cap remains 512 MiB, inclusive of all other allocations.

For any extent, preflight 32*w*h + 4*w*h + 9 MiB of optional image payload with
checked arithmetic against 45 MiB and process admission. Unsupported extents
fall back. Reserve the entire optional bundle transactionally before creation;
failure drops partial unsubmitted work and cannot evict submitted resources.
Aliases count once. Retain at most one current and one retiring coherent bundle,
sharing the existing generation permits. The required B remains charged even
while warm frames do not use it. Never claim that unused B saves residency.

Retain the 4 KiB image-metadata limit; fixed source IDs consume 1 KiB. Prove all
remaining owned metadata fits, including handles/containers at the existing
accounting scope. No cap increase by implementation convenience. Charge immutable
per-frame mapping/translation uniforms, alignment padding and transfer staging
to existing budgets; a 256-byte-aligned 256-entry mapping is at most 64 KiB
before any separately charged header. Do not overwrite a submitted mapping.

Every submitted reference to A/B/C/T/R and mapping storage participates in the
existing resource observer and queue-completion retirement. Increase fixed
resource snapshot capacity to describe the actual five-image bundle and its
retiring usage, not a new observer format or thread. Include creation, submitted
use, optional eviction, fallback overlap, close and device-replacement peaks.

Mutating C before presentation does not validate it. Failed/uncertain encode,
submission or presentation, stale receipt, stronger scene/material change,
extent/scale/format/device change or eviction invalidates A/C reuse as applicable.
Keep old presented support until matching success; coalescing never advances it
early. Reconstruct full composition before reuse after uncertainty. Ordered queue
submissions may reuse atlas storage only through the existing admission contract;
resource replacement waits or refuses rather than releasing live holds.

## Supersession and bounded proof request

Amend PM050 explicitly: replace retained full-size composed B and warm A-to-B
restoration with retained C/regional T; exposure resolves become C copies; one
hardware atlas resolve replaces the full-target warm resolve. Remove production
warm r3 B restoration/reuse and its competing completion identity. Keep full
fallback, immutable preparation, actual-pad membership and shared painter owners.
No editor-specific implementation, new dependency, licensing or prototype change.

Requested execution scope, only after explicit owner approval and required
independent design review/numbered-decision reconciliation:

1. Implement this one layout and graph in the shared owners. Add focused offline
   controls for tile deduplication/bounds/overflow, source-to-atlas clips, optional
   bundle arithmetic/refusal and completion invalidation. These decide whether
   the graph can proceed to GPU proof. Run scoped default/visual lint and required
   governance checks through existing guarded compilation. Review all 14 preserved
   non-pad transition fixtures for tile capacity before any timing experiment.
2. Run one four-group GPU batch, at most 300 seconds, P630/Vulkan/exact8. Group 1:
   fractional old/new crosshairs across 32-pixel boundaries, first/last atlas cells,
   empty/duplicate/full-capacity/overflow cases and exact per-sample tile comparison
   to the full painter; reject wrong-origin/sample and duplicate-paint controls.
   Group 2: Console/text/menu/terminal clipping and occlusion across tile seams,
   with exact resolved C and presented output. Group 3: all 14 native non-pad
   transitions, real pad enter/change/leave, coalescing and cold/empty/warm/recovery
   transitions; verify complete output and no warm world redraw. Group 4: normal
   production usages, metadata/admission bounds, current/retiring/submitted image
   and mapping lifetimes, refusal/eviction/close/device loss. Diagnostic sample
   readback opt-in stays test-only and excluded from resource/performance claims.
   Reuse unaffected previous proofs; this batch targets the changed graph.
3. On full pass only, build one release candidate and one measurement-matched
   baseline. Reuse original baseline source/render patch; its only added method
   change is a final bounded timestamp marker so copy-to-presentation work cannot
   fall outside the frame span. Add the same marker to candidate and baseline.
   No extra observer slots/queries/record capacity or streaming infrastructure.
   Worst cold frame uses 20 continuation queries +2 leading +2 prefix +3 scene
   markers +2 suffix +2 trailing =31 of the existing32; warm uses at most28.
   The final marker follows every atlas-to-C and C-to-target copy before query
   resolution. Report those inter-pass intervals within the complete span;
   do not mislabel them as suffix pass duration. Verify ordering offline first.
4. Run one new fixed twelve-trial pointer campaign, retaining the current method:
   C-G1,B-G1,B-Q1,C-Q1,B-G2,C-G2,C-Q2,B-Q2,C-G3,B-G3,B-Q3,C-Q3; G=GPU timing,
   Q=quiet. Same40s readiness,5s warmup,30s120Hz400x240 input,5s still,150s trial
   cap and controlled drain. All input/frame lineage, exact output, capacity and
   cleanup checks remain mandatory. Candidate complete-span p95<=4ms/p99<=8ms;
   preserve every active sample including overflow/full fallback. No paired or
   overhead claim unless its required trials complete.
5. First offline design-contract failure prevents GPU entry. First GPU API,
   exact-output/sample, negative-control, capacity or resource failure stops the
   batch and later release/native execution. First invalid native result or
   candidate numerical failure stops the campaign. Preserve evidence and end
   with source assessment; no replacement, changed tile size, tolerance refresh,
   observer expansion or alternative mechanism under this approval.

This requests one candidate architecture with a falsifiable result, not a promise
that it will meet p95. Repeated suffix traversal can offset resolve savings; the
complete native span, not pixel-count arithmetic, decides performance.

Camera, remaining resource/observer-overhead, endurance and independent final
qualification remain mandatory before closure. Full DRM lifetime/duty<=25%
qualification remains a separate scope: its current gap does not block this
focused experiment but blocks final S4 acceptance. No DRM observer execution or
CPU event-loop optimization is included. A passing pointer packet does not close
S4, S5 or the issue.

Owner response: `approve GPU regional-composition proposal r4`.

The previous approved sample-readback packet ends after its descriptive report
and source assessment, and the native first-failure rule stopped that campaign.
This new mechanism, admission allowance, builds and bounded runs therefore need
explicit owner approval. Existing unused trials are not execution authority.
