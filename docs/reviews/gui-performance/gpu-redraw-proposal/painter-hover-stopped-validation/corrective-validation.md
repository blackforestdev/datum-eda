# Painter/hover sample-readback correction

Status: proposed; no replay approval, patch application or post-failure build.
Implementation: `7e7c644f`. The single approved GPU batch stopped in group 1.
See `receipt.json` and the complete `gpu-batch.log`.

The new sample oracle is defective. Working image B intentionally has
`RENDER_ATTACHMENT | COPY_DST`; restoration reads A and writes B. Creating its
UNORM alias does not add `TEXTURE_BINDING`. The new compute readback tried to
sample that alias, so wgpu rejected the bind group before sample comparison.
The cold and first fractional-crosshair pixel comparisons passed. Nothing later
in group 1, none of groups 2–4, and no native timing trial ran. This does not
establish a new renderer, platform or hardware defect.

## Concrete correction

Apply only the accompanying `sample-readback-correction.patch` against the
implementation commit. It adds a `cfg(all(test, feature = "visual"))` field and
explicit pre-allocation opt-in for B's `TEXTURE_BINDING` usage. Only groups 1 and
3 enable it, because they inspect all eight samples. The default remains off,
including groups 2 and 4, which retain the production allocation usages for
painter/resource checks. Native and release builds contain neither the flag
nor its setter. The patch adds no texture, buffer, observer, dependency or device
feature; it leaves the renderer's algorithm, masks, formats, sample count,
per-pixel painter order, admission caps and retirement mechanism unchanged.

This is preferable to changing production B usage for a test, relaxing the
all-eight-samples requirement, or constructing another sample-copy pipeline.
The existing test readback buffers remain diagnostic scratch, excluded from
candidate resource/performance claims. No resource qualification is claimed
from the opt-in groups. Group 4 retains normal allocation usages. The P630 log
already reports the required texture-binding format support; the issue is the
resource's declared usage, not absent adapter capability.

The patch has passed `git apply --check`; it is a review artifact only.
Before any GPU replay, inspect the default/opt-in allocation paths and both call
sites, run scoped default/visual lint, compile the test executable through the
Cargo guard, list the exact test, and seal its source/binary/fixture hashes.
Check fixed readback dimensions, dispatch bounds, UNORM alias and eight-sample
indexing against the shader. No new test infrastructure or native observer work.

## Requested execution amendment

1. Apply that exact test-only correction and complete the offline preflight above.
   Ordinary offline compile/lint corrections stay within that correction.
2. Execute the same four-group test exactly once, serially, with the same pinned
   P630/Vulkan/exact8 fixtures and 300-second batch bound. Group 1's two earlier
   pixel checks repeat because its diagnostic allocation usage changes; groups
   2–4 were never reached. Preserve the original failure. First API, sample,
   pixel, graph, negative-control, lifetime or capacity failure stops the batch
   and all later GPU/build/native execution for source reassessment. No retries.
3. Only if all four groups pass, continue the already approved one candidate
   release build and new fixed twelve-trial pointer campaign, reusing the pinned
   baseline executable. No baseline rebuild or observer change. Original trial
   order, cadence, readiness/drain deadlines, exact-output checks and candidate
   p95 <=4ms/p99 <=8ms stop rules remain unchanged. Neither a release build nor a
   trial from that allowance has been consumed.
4. End after the descriptive result and source assessment. Keep camera,
   remaining resources, endurance, independent qualification and complete DRM
   lifetime accounting open. No CPU optimization, DRM execution, new trial
   variant, acceptance or issue closure is authorized by this amendment.

Owner response: `approve GPU sample-readback corrective validation`.

The prior approved `painter-hover-correction/proposal.md`, execution item 3,
requires: “First GPU correctness/resource failure stops execution for source
reassessment.” This explicit stop rule requires renewed approval for the GPU
replay, even though the diagnosed defect is confined to the test harness.
