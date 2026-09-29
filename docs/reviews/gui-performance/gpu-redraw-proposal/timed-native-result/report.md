# Timed GPU recovery result

The approved packet produced one correct, fully attributed native pointer result,
but the candidate failed both GPU timing budgets. The sealed campaign stopped
permanently after trial C-G1. Eleven unspent trials do not authorize replacements.
GPI-S4 and `dat-gui-performance-implementation-vkq` remain open.

| Check | Result |
| --- | --- |
| Source | `b67568ae`, one candidate and one matched baseline release build |
| Reference host | P630, Vulkan, exact8, Fifo,1280x800; unchanged fixture/reference |
| Readiness / final image | Zero changed pixels; visible crosshair |
| Pointer workload |3600requests,1280integer changes received and applied |
| Observer |2577/4096records; contiguous causal demands, no rejected/incomplete sample |
| GPU completeness |1269sealed frames,1265causally active; every submission validated |
| Active complete-span p95 |4.256333ms — FAIL, limit4ms |
| Active complete-span p99 |16.539917ms — FAIL, limit8ms |
| Maximum complete span |34.952251ms |
| Shutdown |Clean exit, no forced cleanup/survivors, binary unchanged |

`receipt.json` pins source, raw evidence and limitations. `result.json` is the
original native result. `native-raw.tar.gz` preserves full native logs, input and
pre-drain receipts, producer schedule, classified frames and images. Build JSON
streams are compressed without changing their contents; source paths verify that
the two artifacts came from the intended distinct source trees. No baseline or
quiet trial ran, so matched-pair ranges/medians and on/off effects are unavailable.
The native result is not evidence of an observer-free cost or relative speedup.

## Where the measured time remains

| Active graph | Count | Complete-span p95 | Complete-span p99 |
| --- | --- | --- | --- |
| Restore + suffix |1251|4.239250ms|4.595000ms|
| World upload + full prefix + suffix |14|34.952251ms|34.952251ms|

The restored group still fails the4ms p95 limit. Its restoration pass p95 is
0.225917ms and suffix pass p95 is3.858583ms. These independently ranked numbers
must not be added. `cost-analysis.json` retains the group distributions and the
20slowest complete records, including raw submission identities and intervals.
The14cold frames all exceed8ms; their range is14.915500–34.952251ms. Five restored
frames also exceed8ms. Keeping the cold frames in the workload is required.

Source inspection explains the graph boundaries, but not every hardware cost:

- `gpu_frame.rs` uses damaged sample restoration for1251frames. It does not
  rebuild the full prefix or make the full A-to-B copy on those warm frames.
- `gpu_frame_painter.rs` still submits the suffix painters in their established
  order. Their ordinary scissors cover the viewport or full frame. The shared
  `gpu_damage.rs` predicate rejects fragments outside damaged rectangles; it
  does not restrict draw submission/rasterization to those rectangles.
- The suffix timestamp also encloses loading/storing the8× working attachment
  and resolving the entire presentation target. Existing evidence cannot split
  painter, attachment and resolve costs inside that interval. Calling it a
  proven resolve bottleneck, driver defect or hardware limit would overstate it.
- Each of the14cold graphs has a `world` upload submission followed by `final`.
  `cold-demands.json` joins their causal tags to the immediately preceding hover
  transition, exactly one retained demand earlier in each case. Complete spans
  include upload/inter-submission intervals:8.055084–17.820501ms outside named
  passes in this group. These intervals cannot be called pure GPU busy time or
  silently removed from the controlling complete-span statistic.

## Remaining shared-renderer responsibility

`interaction_refresh.rs` already submits pointer intent through the shared
`RenderSession`. The residual coarse dependency is inside that shared owner:
`session_hover.rs::BoardHover::permits_pointer` rejects a changed board region;
`session_pointer.rs::update_pointer` calls `clear_content`; that method drops the
prepared projection, board, active source and both scene histories. Subsequent
preparation/upload redoes world work. This preserves legacy pad material/dimming
and hover-label behavior, but ties pointer emphasis to full retained content.
It is not a necessary X11/Wayland/swapchain integration requirement.

The next correction proposal must address both demonstrated paths: shared
painter damage that bounds repeated suffix work, and shared hover dependencies
that preserve immutable world data when actual material/text dependencies permit
it. Where hover really changes material, that appearance and painter order must
remain exact; treating every hover change as a ring-only update would be wrong.
The observed outline/zone/track transitions warrant dependency-specific review,
not an assumption that every legacy path is faulty.

Before another implementation/build/run packet, pin the replacement ownership,
precise superseded paths, expected removed work, correctness/fallback proof and
live/retiring allocation bounds. Resolve versus suffix drawing remains an
explicit attribution uncertainty. Preserve all8samples, damage union, painter
order and invalidation/recovery; introduce no speculative build sequence or CPU
event-loop change. This packet ends with this source assessment and authorizes
no further renderer change or native experiment.

## Scope of progress

The smaller observer correction succeeded: no streaming writer, larger record
cap or new evidence format was needed for this valid timing-enabled result.
The earlier missing-crosshair failure remains preserved and unexplained; this
run demonstrates current exact output, not a retrospective diagnosis.

Camera regression, resource/consumer reconciliation, endurance, independent
qualification and total GPU duty remain outstanding. The separate DRM method
still lacks a demonstrated accounting-finalization barrier; queue completion or
GEM_BUSY is insufficient. That gap did not block this focused timing experiment,
but it blocks final duty qualification. It is not remedied by these own-queue
spans, and no DRM execution was performed or authorized by this packet.
