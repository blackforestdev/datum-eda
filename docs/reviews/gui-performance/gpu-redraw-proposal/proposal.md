# GPU redraw proposal r1: retain the multisample scene prefix

Status: **proposal for GPI-S4-APPROVAL, not ratified or implemented**.
Issue: `dat-gui-performance-implementation-vkq`.
Inspected production revision: `997b618e8369e327f9d5ee2d30e8019c9c4c6ef9`.
Planning handoff: [handoff.json](handoff.json). Evidence and source hashes:
[review.json](review.json). No build, application test, benchmark, diagnostic
launch or system change was performed for this proposal. CPU event-loop work
is excluded. Documentation governance checks are separate from runtime proof.

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S4-GPU-DESIGN-R1 -->

## Recommendation and decision

Approve one bounded implementation of an **8x multisample prefix image cache**
inside the shared Renderer. Cache the existing ordered drawing up to, but not
including, the first interaction overlay. On an eligible pointer-only frame,
copy that image's samples into the normal multisample frame attachment, draw
the unchanged ordered suffix once, and resolve once to the acquired surface.
This removes all warm pointer-frame world and grid draws, rather than merely
retaining their vertex buffers or replaying them under smaller scissors.

This is a concrete correction candidate, not evidence that the P630 meets
4 ms p95 / 8 ms p99 / 25% duty. Existing evidence establishes expensive repeated
scene drawing; it does not isolate the cost of the proposed multisample copy.
The falsifiable feasibility condition is that the full copy, barriers, remaining
suffix and resolve fit those unchanged limits together. If they do not, stop
and return the measured boundary for owner review. Do not cycle through the
alternatives below automatically or describe the hardware as incapable.

Owner approval would authorize only the scope and staged proof below. Before
execution, record the exact response, ratify the image-retention mechanism in
a numbered decision through existing governance, reconcile its evidence route,
complete the required independent design delta review, and change Frontier
authorization. This document reserves no decision number.
It changes neither PM045 numerical requirements nor PM047/PM029 authority.

## Evidence and the architectural defect

| Evidence | Observation | What it does not establish |
|---|---|---|
| Current `gui-app/src/interaction_refresh.rs::refresh_interaction_overlay` and `gui-render/src/render/interaction_overlay.rs::refresh_interaction` | Pointer refresh retains PreparedScene/world and replaces board/schematic interaction vertices. | A new CPU scene-cache invalidation defect. That earlier defect is already corrected. |
| Current `render/gpu_frame.rs::render_submission_inner` | Every general pointer frame clears the full MSAA target, draws panels, grids and cached world bundles, then overlays/text, resolving once and discarding the MSAA contents. | Cached bundles save GPU rasterization. They retain commands; executing them still draws the scene. |
| S5 `scene-gpu-localization/result.json`, diagnostic base `62696261` | 1,276 samples; world interval mean 4.467 ms, p95 8.386 ms; underlay mean 2.193 ms; frame p95 12.780 ms. | Isolated shader time, current-HEAD timing, a copy/resolve lower bound, or permission to subtract independent percentiles. Markers include synchronization/scheduling/observer costs. |
| Retired `500387c1`, S5 `interaction-damage/native-performance/assessment.json` | All three GPU trials failed: p95 15.002–15.132 ms, p99 16.847–17.172 ms. Seven earlier focused correctness checks passed. | A successful partial-redraw optimization. It was reverted by `0de1d528`; keep the failure. |
| `4eaa0087`, S5 `pass-consolidation/native-performance/assessment.json` | One merged painter pass retained exact endpoints in six native runs; GPU p95 12.026–12.073 ms, p99 12.438–12.528 ms still failed. Endpoint render duty was 33.056–33.567%. | Full duty acceptance: endpoint clients do not prove intervening lifetimes/reset exclusion. Historical comparisons are unpaired. |
| `9f16eb69` triangle compaction | Removes demonstrated redundant degenerate vertices with bounded correctness evidence. | A later native GPU performance cure; no such result exists. |

The retired candidate reset and replayed the complete scene separately for up
to 32 disjoint damage regions, preserved the full MSAA composition and resolved
the complete output. Scissoring does not eliminate repeated command/vertex work.
The proposed cache instead executes **zero** prefix draws on a warm eligible
frame and copies samples once; it needs no damage rectangles, stale-overlay
erasure algorithm or per-region text/world replay. This distinction is directly
visible in the proposed command graph, without a performance experiment.

## Exact pass graph and fidelity

The current graph is one general color pass: clear; panel; applicable underlay;
composed grids and world (or legacy board/schematic fallback); schematic
interaction; viewport overlay; board interaction; console geometry; terminal
background images; workspace text; terminal foreground images; menu card; menu
text; one resolve. Original PASS-01's larger graph is historical implementation
description; its painter, contributing-work and resolve obligations still apply.
The fallback is guarded by empty composed surface passes: it is not a duplicate
world draw to remove.

Define the prefix boundary immediately **before schematic interaction** in the
current graph, after all applicable board/schematic world and underlay work.
Keep every later stage in the suffix, including apparently static viewport
selection geometry, console, terminal images and both text owners. Do not move
text beneath interaction, move a menu beneath workspace text, or reorder panes.
Initially only the composed general-frame path is cache-eligible. Legacy paths
and all three dialog-only hosts keep their existing graph.

| Frame | Commands, load/store and resolve |
|---|---|
| Unchanged prefix / pointer-only suffix change | Full-size equal-format 8x texture copy from immutable prefix A to frame attachment B; one suffix pass on B with `Load`, final `Discard`, and one resolve to the new surface target. |
| Cold or changed prefix, eligible configuration | One prefix pass on A with existing clear color, exact existing prefix commands, `Store`, no resolve; full-size A-to-B copy; the same suffix pass with `Load`, `Discard`, one resolve. |
| Noneligible configuration or cache admission refused | Existing full-frame clear/draw/resolve path on B; no cache-valid publication. This is correct fallback, never GPU budget acceptance for a failed candidate. |
| No visible change | Existing scheduler rules: no new copy, pass or submission. Native exposure still presents valid current content through the ordinary host path. |

A and B have identical extent, sample count, format and mip/layer configuration.
A uses `RENDER_ATTACHMENT | COPY_SRC`; B uses
`RENDER_ATTACHMENT | COPY_DST`. Retain actual tracked texture handles as well
as views; current `SurfaceAttachments` keeps only the view after allocation.
No CPU readback, resolve-to-1x-and-expand, sampling shader, filtering, gamma
conversion, blending or viewport reprojection is involved in the copy.
Copy all samples, then continue the identical suffix at the identical sample
locations. The prefix pass break stores the same color samples that the original
single pass would feed to the suffix; the copy preserves those samples.
This is the fidelity argument to test, not a substitute for exact GPU equality.

Pinned installed wgpu 28.0.0 `api/command_encoder.rs` explicitly requires whole
multisample texture copies. wgpu-core 28.0.1 `command/transfer.rs` enforces whole
XY extents, equal sample counts, compatible formats, usages and device ownership;
`device/resource.rs` requires supported sample count/usages and render-attachment
usage. wgpu-hal 28.0.1's Vulkan path emits `cmd_copy_image`. These sources make
this a concrete existing-API design; actual combined usage/support and output on
the pinned P630 remain the first future proof gate. No dependency modification
or new API package is proposed. Web API portability is not inferred from the
native implementation. Local file hashes, including Cargo.lock, are in the review.

## Shared identity, invalidation and recovery

One Renderer-owned prefix entry per eligible native host, never one per pane.
Its immutable key contains host/device/attachment generation, physical extent,
format/sample count, prepared noninteraction composition generation, ordered
pane identities and geometry, cameras/projections, retained world identities,
visibility/draw-order/style dependencies and all prefix underlay/grid inputs.
Use explicit monotonic generation/owned resource identity, not pointer+length or
a hash alone as authority. A newly built PreparedScene starts a new generation;
only its narrowly audited interaction refresh preserves that generation. Camera
or other setters outside that refresh must invalidate it. Additional key fields
fence resource/device replacement even when logical composition is unchanged.

| Change or failure | Required result |
|---|---|
| Pointer enter/move/leave, hover, FullViewport/Local/None crosshair | Apply current hit/input semantics; reuse prefix only if key matches; copy fresh A into B and draw current suffix. No old crosshair trails. |
| Camera, grid/LOD, layer visibility/order, equal-length model replacement, style, shell/pane layout, selection affecting prefix | Invalidate A before reuse and render complete new prefix. Do not reset surviving pane cameras. |
| Text/atlas, terminal, menu, console or other nonpointer change | Initially fall back to full current frame and invalidate A conservatively. This avoids broadening the optimization to mixed activity without proof. Existing shaping and atlas invalidation still apply. |
| Extent, DPI, target format/sample count, host identity, device replacement | Retire old image ownership through shared lifetime accounting; rebuild exact current configuration. No old-size cache display. |
| Upload-only continuation, failed preparation/acquisition, lost/outdated, timeout, suspension or known occlusion | Preserve pending input/damage under existing host policy; invalidate cache eligibility conservatively. Restore uses current state; no new retry loop. |
| Failed submit/present, observer error, asynchronous device loss | Never publish a reusable cache generation from an uncertain frame; invalidate it. Old completions cannot validate a new host/device. |
| Close, eviction or pressure | Drop renderer ownership; submission references retain resources until completion/teardown. Retain no authoritative document/terminal data in this derived cache. |

Build a candidate key locally, encode using one coherent snapshot, and publish
it only on the existing successful submission/presentation path. Native hosts
must report presentation failure to this owner. No acquired surface is retained
across configuration; uploads finish before acquisition as today. The current
coordinator's one outstanding frame per host and bounded recovery remain the
synchronization authority. No separate queue, thread or event-loop policy change.

## Resource cost and remaining work

For 4-byte color formats, let `P = width * height * 4 * 8` with checked arithmetic.
The cache adds one P-byte image to the current P-byte frame attachment. Set an
initial optional cache cap of **45 MiB per host**, actual-size allocation only,
and at most **4 KiB CPU key/metadata capacity per entry** (including owned lists;
oversized keys use fallback). No image history or pooling. Treat the pair as one
coherent attachment generation, with one current plus one retiring generation
maximum, and count every allocation separately. Two images are not two temporal
generations, and packaging them together must not hide their bytes.

| Physical extent | Existing B | Added A | Current pair | One old plus one new pair |
|---|---:|---:|---:|---:|
| 1280x800 | 31.25 MiB | 31.25 MiB | 62.5 MiB | 125 MiB |
| 1536x960 | 45 MiB | 45 MiB | 90 MiB | 180 MiB |

These are allocation arithmetic examples, not measured residency or a new
fixture/extent pin. Required reference extent comes from the preserved native
recipe and must be verified before execution. Admit A and B transactionally
against the existing 512 MiB process GPU cap and all relevant local/RSS/staging
caps, counting other hosts, atlases, buffers and in-flight resources. The 45 MiB
cache cap is a maximum, not a reservation or replacement for aggregate admission.
Larger physical configurations use correct full-frame fallback pending separately
approved scope; they are not silently scaled or reduced to 4x. Release optional
A before refusing required frame content when safe; never lose pending damage.
The generation allowance must survive Renderer/device replacement as today.

A warm pointer frame removes panel/grid/world vertex and fragment execution,
but retains hit/input work, changed interaction uploads, suffix draws, full copy,
full resolve, acquisition and presentation. Text/terminal/menu suffix execution
is once per frame, not zero. At 1280x800 the copy reads and writes 31.25 MiB;
resolve additionally reads up to 31.25 MiB and writes 3.90625 MiB of logical color
payload. That is approximately 97.66 MiB/frame before suffix traffic, compression,
cache effects and driver synchronization. At 60 frames/s it is 5.72 GiB/s of
logical traffic, **not a measured memory-bandwidth requirement or timing bound**.
Cold/changed-prefix frames add a store, copy and pass break; CPU/GPU regressions
there must be assessed explicitly rather than hidden in warm pointer averages.

## Alternatives considered

| Alternative | Fidelity / invalidation | Removed and remaining work / resources | Disposition |
|---|---|---|---|
| Current merged full draw | Existing exact reference; no image key | No extra image, but all prefix rasterization remains | Rollback/reference; existing failed GPU budgets preserved. |
| Retired per-region redraw | Bounded exact controls passed; complex old/new coverage and recovery | Replays prefix per region, full MSAA storage/resolve | Do not resurrect or rerun unchanged. |
| Single stencil damage mask, one complete scene traversal | Can preserve samples/order if all pipelines/bundles honor identical mask and clips | Avoids repeated traversals; still all world vertex work, mask attachment and full resolve; early fragment savings are backend-dependent | Plausible alternative, less direct removal of measured world work. Not selected or automatically authorized. |
| **Full 8x prefix copy** | Sample-preserving copy, explicit prefix generation, unchanged suffix order | Zero warm prefix draws; extra P bytes; full P copy and resolve remain | Recommended bounded candidate: minimal change to drawing semantics, explicit risk. |
| Sampled multisample regional restore / tiled rendering | Needs sample-index, color-space, coverage and tile-seam proof; old/new region management | Could reduce copy traffic; extra shaders/targets, possible extra passes and suffix replay | Reserve for a new proposal only if evidence identifies full copy as the limiting cost. Native partial MSAA copies are not supported by the pinned path. |
| Resolved 1x background plus overlay | Resolving before overlapping 8x overlay loses sample covariance; exact output cannot be assumed | Smaller cache/copy but different blend/coverage semantics | Reject for this exact-8x proposal. |
| Lower AA, fewer required updates, backend/driver tuning, replacement framework | Changes protected quality/delivery/authority | May move cost rather than remove required work | Outside authorization and not a solution under current requirements. |

Qt Quick's [layer documentation](https://doc.qt.io/qt-6/qml-qtquick-item.html#memory-and-performance)
describes retaining a subtree in an offscreen texture and warns of memory and
rendering overhead. It supports considering image reuse and accounting for its
cost, not predicting a Datum speedup. The [WebGPU stencil specification](https://www.w3.org/TR/webgpu/#depth-stencil-state)
and [WGSL multisample loads](https://www.w3.org/TR/WGSL/#textureload) support the
alternative mechanisms only; neither establishes P630 performance or exact
cross-renderer equivalence. No reference code or dependency is adopted.

## Proposed production touch points and rollback

| Owner | Bounded change after approval |
|---|---|
| `render/gpu_frame.rs` | Extract cohesive prefix/suffix encoding owners; select reference or prefix-copy graph without duplicating painter policy. Preserve merged suffix and exact current command ordering. |
| `render/gpu_surface.rs`, `renderer_state.rs`, `gpu_init.rs`, shared resource-lifetime owner | Track copy-capable texture handles; optional prefix image/key; combined generation admission, submission holds, teardown/recovery. Include texture usages in replacement identity. |
| `render/prepared_scene_access.rs`, `frame_preparation.rs`, `interaction_overlay.rs` and camera setters | Explicit composition generation; audit every mutation that can change prefix pixels. Do not add pointer-triggered scene rebuilding. |
| `gui-app/src/runtime_present.rs`, shared auxiliary-host failure/recovery adapters | Successful-frame publication/failure invalidation only; no event-loop optimization. Dialog path does not allocate a prefix cache. |
| Existing GPU measurement/resource observers and their proof harness | Account copy command, bytes, image generations and prefix/suffix execution; include the copy in complete-frame timing. Preserve zero unchanged world upload accounting. |

Before editing any oversized Rust module, follow decision 022/source-health
ownership extraction and downward ratchets; the claim here authorizes none of
those production edits. A source-health extraction must stay with the relevant
encoding/resource owner, not create a forwarding-only split. Any broader required
change returns for review. No dependency, shader quality, prototype, persisted
model or settings change. Rollback is an ordinary isolated revert of this slice
and its optional cache owners, restoring the existing one-pass reference path;
preserve evidence, restored damage, authored state and unrelated work.

## Smallest decision-enabling future proof and stop rules

This is the **requested future execution scope**, not a command to execute now.
One mechanism, one pinned candidate, serial GPU/build work, no parameter sweep.
Pin HEAD/diff, binary, Cargo.lock/toolchain, F-DOA fixture/resolved identity/order,
actual extent/backend/P630/device/driver/environment, input recipe and all method
sources before running. Reuse prior evidence only at its original valid scope.

1. **Capability, fidelity and ownership gate.** One cohesive focused batch through
   the production owners: actual 8x full copy and retained-prefix output versus
   the reference; sample-sensitive thin strokes/holes/overlap, fractional positions,
   crosshair styles/enter/leave, multiple panes, hover, text, both terminal image
   layers and menu order. Compare exact RGBA, no tolerance/golden refresh. Cover
   equal-length source replacement, camera/layer/style/layout/atlas/extent/device
   changes, acquisition/submit/present failure, close while pending, admission
   refusal and old/new generation retirement. Deliberately stale-key, missing-copy,
   reduced-sample and early-resolve controls must fail their target assertions.
   Required decision: can the mechanism preserve the existing image and lifetime
   contract? On capability failure, pixel difference, missing invalidation or
   incomplete resource accounting, stop before native performance measurement.
   No alternate format, backend, shader copy or mask is silently substituted.
2. **Whole-frame measurement gate.** The copy precedes the suffix pass and would
   be missed by the present first-pass-start timer. Add a diagnostic-only opening
   timestamped marker pass before the copy, using the already-supported pass-query
   feature and a tracked diagnostic-only 1x1 RGBA8 attachment (4 logical bytes,
   with actual allocation/driver overhead separately accounted), then time
   through the final suffix end;
   on cold frames start at the prefix pass. The marker clears/discards only its
   tiny target; it must not load/store or initialize a full-size frame attachment.
   Separate marker/pass sum, copy-inclusive
   span, logical copy bytes and complete DRM duty. Do not report suffix-only time
   as whole-frame time. Conformance must reject omitted-copy boundaries, missing
   queries and epoch mismatch; measure marker overhead with matched quiet output.
   Reuse unchanged query-ring controls. Stop if required MET/GPU/ACC observation,
   including complete client lifetimes and overhead, remains unavailable. Existing
   endpoint-only duty receipts cannot be promoted into a pass. If method repair
   exceeds these existing-observer touch points, return its exact gap for review.
3. **One fixed native comparison.** After gates 1–2, request three matched
   baseline/candidate W-POINTER pairs in each existing quiet and GPU diagnostic
   mode (12 runs maximum, no replacement runs): 5 s warmup, 30 s fixed input,
   ordinary final-state/idle tail and bounded live-device drain. Alternate AB/BA.
   These same runs supply overhead and absolute results; do not add a producer
   rehearsal. Three pairs are descriptive, never STAT-01 formal relative inference.
   No formal seven-pair improvement campaign is requested. Verify all scheduled,
   received and semantically acknowledged actions, exact final state/pixels, all
   affected resource caps and CPU cost; never demand 120 distinct coordinates/s
   when the pinned schedule repeats integer positions. Candidate must meet GPU
   p95 <=4 ms, p99 <=8 ms and duty <=25% in every valid trial with the complete
   method. A first failed/invalid trial stops the sequence; retain it and the
   incomplete outcome. Success requires all three candidate trials per mode.
4. **Affected nonpointer regression and disposition.** Reuse the focused gate's
   counters to identify the added cold/camera rebuild work. Request one matched
   W-PAN baseline/candidate pair per quiet/GPU mode (4 additional runs maximum,
   same warmup/duration), descriptive only, to detect obvious regression from
   the extra store/copy/pass break. Existing cold-start distributions remain
   unqualified until S5; no cold budget pass from this small batch. Any material
   unexplained CPU/GPU regression (>5%), missing output, or inability to stay in
   affected caps blocks adoption and returns for review. These observations
   cannot establish formal relative equivalence. Stop at this bounded report;
   S4 closure still needs all original obligations and explicit governance review.

No rerun solely to improve a number. A demonstrated implementation defect may
be described and corrected only within the approved scope; a changed candidate
requires a new reviewed proof disposition rather than silently restarting this
fixed campaign. Preparation/method failure ends the runtime attempt, not an
open-ended troubleshooting campaign. The owner can revise the proposal instead
of granting any of these runs. No system tracing/tuning, CPU scheduler work,
new dependency, native Wayland expansion or endurance campaign is in this request.

S5 still owns complete backend/scale/consumer qualification, all 225 original
rows, native UX/product acceptance, endurance and distinct-reviewer native
replay. Existing failed and partial evidence stays intact. Resize, temporal,
T2 and unadmitted schematic exclusions remain explicitly unqualified. A positive
bounded S4 result cannot be presented as complete GUI performance acceptance.
