# GPU correction r3: retain composed samples and restore pointer damage

Status: **proposal only; no new implementation, build or run authorization**.
Issue: `dat-gui-performance-implementation-vkq`; current step: GPI-S4.
Base: `6704c1e0`; failed experiment preserved by `7ae75ed3` in
[stopped-experiment-r2/receipt.json](stopped-experiment-r2/receipt.json).
R2's shared ownership implementation remains the foundation. This amendment
changes its incremental GPU strategy and closes demonstrated measurement gaps;
it does not restart the stopped r2 declaration.

## Evidence and decision

R2 removed repeated warm world/grid rasterization, but still copies the entire
8x prefix A to working B and paints the complete suffix. The first candidate run
passed all 1280 changed-position routes and exact startup/final pixels. Fourteen
active cold frames have incomplete upload timing. The 1241 captured warm spans
have median 10.216 ms, p95 15.939 ms and p99 18.310 ms. These are **partial warm
observations, not complete workload percentiles**. They justify removing full
image work; they establish neither relative improvement nor a hardware limit.
The between-pass interval includes copy, barriers and gaps, not isolated bandwidth.
No baseline, quiet or W-PAN run was consumed; the entire r2 sequence is stopped.

Approve one correction: keep B's successfully composed 8x samples between frames;
restore only the old/new interaction pixels from immutable A, then run the
existing suffix once with a shared pixel-damage predicate, resolving once.
Preserve the exact full-copy cold/fallback graph. Complete multi-submission GPU
measurement and semantic workload attribution before a new bounded experiment.
This is a changed mechanism requiring owner ratification; approval of r2 did not
approve shader restoration, retained B or another candidate campaign.

## Ownership and removed work

RenderSession remains the sole owner of derived content, revision dominance,
preparation, composition validity and matching completion. Editors submit typed
input; they do not calculate GPU damage or select a graph. Native acquisition,
presentation, host scheduling and queue retirement stay with existing adapters.

The shared interaction builder will emit conservative physical-pixel support
alongside the actual board/schematic overlay primitives. Keep separate thin
crosshair arms and hover-border strips: a single crosshair bounding box would
degenerate to the whole pane. Include old presented support and newest desired
support, pane clipping and a one-pixel outward guard around actual rasterized
geometry. Unknown/nonfinite bounds, capacity overflow, or any change outside the
narrow pointer contract selects the existing full graph. Use at most 32 rectangles
and fixed inline metadata; overlap is harmless for a union predicate. The union
must be computed from presented state, not the last received pointer event, so
coalesced input cannot leave trails. No new editor-side cache is introduced.

A warm eligible frame removes the full A-to-B transfer and all destination writes
and blending outside damaged pixels. Prefix/world/grid commands remain absent.
The ordered suffix is encoded **once**, not once per rectangle; suffix vertex
processing and some fragment work remain. A full-size 8x resolve, attachment
load/store and driver synchronization also remain. This proposal does not promise
that these remaining costs fit 4/8 ms. It gives a concrete command-graph reduction
to test under a fixed failure stop, with no parameter search.

Retire warm full-copy selection and final Discard for cache-eligible B. Do not
remove the cold/full copy primitive: it remains necessary to initialize B and
recover conservatively. Do not resurrect the retired 32-region whole-scene replay;
that candidate repeated world and text commands and failed its measured budget.

## Exact sample restoration and painter order

Native wgpu28 requires whole XY extents for multisample texture copies. A smaller
`copy_texture_to_texture` is therefore not a valid implementation. Instead, A gains
TEXTURE_BINDING usage and an alias view with the matching nonsRGB UNORM format.
B gains the same nonsRGB view for a restoration pass. Both retain their normal
sRGB views for the existing painters. No new image, dependency or backend is added.

Restore shader: integer pixel coordinate from fragment position;
`textureLoad(A_unorm, pixel, sample_index)` to B_unorm, with sample_index genuinely
used, 8x sample shading, full color write mask, no blend and no alpha-to-coverage.
For each covered pixel all eight corresponding samples are replaced. A and B
are distinct images. The restore pass draws one fullscreen triangle per integer damage scissor,
at most32 small restore draws; overlapping rectangles simply restore the same
samples twice without blending. It never invokes the per-sample restoration
shader across the whole viewport merely to discard most pixels. Only restoration
is repeated; the common suffix and world are not replayed per rectangle.
The fullscreen triangle, viewport and integer coordinates must preserve all
edge pixels; use no filtering, interpolation of sampled color or reprojection.
UNORM views preserve stored channel values without an sRGB decode/encode cycle.
Normalized conversion still requires explicit byte equality proof on the actual
backend; API plausibility is not an exactness result.

Warm graph: restore pass loads/stores B through its UNORM view; suffix pass loads
B through its original view, paints the common suffix once under the same damage
predicate, stores B and resolves to the current surface once. Cold graph: existing
prefix draw/store A; full sample copy A-to-B; unrestricted suffix/store B/resolve.
No valid damage but native exposure: resolve retained B without replaying suffix.
Noneligible/refused configuration uses the existing full painter with no retained
composition validity. Never expand resolved 1x pixels back into 8x samples.

Add one shared fixed-size damage uniform and WGSL predicate to every suffix
pipeline: screen geometry, workspace/foreground/menu glyphs and terminal images.
Prefix and full/dialog rendering use an explicit unrestricted setting. Existing
scissors still intersect this predicate. Preserve the current painter order:
schematic interaction, viewport/board interaction, terminal background, workspace
text, opaque Console, foreground text, terminal foreground, menu card and text.
At damaged pixels every suffix stage must run, including unchanged text that
covers a removed cursor. Outside damage no suffix stage may write B.

Keep shader derivatives and sampling behavior unchanged. Screen/glyph shaders
can reject before their ordinary work where legal; terminal textureSample must
remain in uniform control flow, evaluating before the final conditional discard.
Do not substitute a different sampler, explicit LOD, gamma formula or blend mode.
Use one shared predicate definition rather than independent per-editor masks.

## Invalidation, hover, failure and resources

A's existing preparation/strong revision/target/pair key remains necessary.
B adds a matching successfully presented composition revision and old interaction
support. Both must match before partial restore. Pending content/composition wins
over pointer changes. Any uncertain encode/submit/present/observer completion,
foreign or late receipt, eviction, device/configuration/extent change invalidates
B reuse. Even if A remains valid, reconstruct all of B after a failed attempt:
physical B may already contain an unpresented composition. Submission holds keep
both image aliases and damage-uniform generations alive independently of validity.

Keep existing board-hover invalidation. `session_hover.rs` and `session_pointer.rs`
correctly reject suffix-only treatment when pad material or hovered labels change;
`retained.rs` bakes pad emphasis into world content. Moving these effects above all
world geometry would change occlusion. R3 does not hide or relabel these frames:
full cold rendering and all their uploads count toward the workload budget. A
future material indirection/reordering is not included in this amendment. A cold
budget failure stops this candidate and returns the evidence; it cannot be worked
around by selecting a hover-free path or omitting slow frames.

A remains <=45 MiB per host; A+B logical bytes and current-plus-retiring pair
limits remain unchanged. Views alias existing allocations and are counted once.
Damage metadata remains within the existing 4 KiB image-key allowance; GPU uniform
storage and each in-flight generation are separately charged under existing caps.
Do not rewrite a uniform buffer still used by queued work: immutable admitted
per-submission storage or the existing fenced generation owner is required.
Optional texture usage, views, bind groups and pipelines must be supported/admitted
before selecting the strategy; refusal falls back and is not performance success.
No cap increase, stencil attachment, hidden image history or global tuning.

## Measurement correction required before another trial

Replace `cold_world.measurement_frame`'s incomplete-only reporting with a shared
bounded frame-measurement transaction. Preserve one frame ID through world, glyph
and terminal upload continuations and final rendering. Timestamp **every** GPU
submission from a leading marker before its upload/copy work through a trailing
marker after it, using the existing supported pass timestamp API and charged 1x1
marker. Track submission IDs, host/device epoch and immutable render-attempt
lineage. Final rendering records restore/copy, suffix and resolve boundaries.
Resolve query data asynchronously. Report first-start/last-end span and own-pass
sum separately as MET-04 requires; identify intervening gaps/other-host work.
Do not reset the span at acquisition or after an upload. Missing boundaries,
overflow, lost/cancelled lineage or unsupported/reset timestamps fail the run.
Keep the existing three slots and32 queries per slot: at most five continuation
upload submissions per frame (four queries each for leading/trailing marker
passes), plus six final-submission queries and three optional scene queries =29.
Final queue ordering is leading marker command, optional flush_frame_uploads
command, then prefix/restore and suffix/resolve commands. Its leading marker
must precede the mixed atlas/uniform/vertex upload batch currently submitted by
gpu_frame.rs before the render command buffer; rendering-pass queries alone
omit that work. Exposure-only and full fallback fit within the same upper bound.
Audit every upload route, including dialog final uploads, for this ordering;
implicit queue writes outside measured boundaries are not accepted.
The remaining slot capacity is not an invitation to grow the batch. More work
reports unsupported/incomplete and stops this proof rather than dropping a
submission. Pass sums include actual marker/render passes; separately label the
marker-to-marker upload intervals, which include transfer/barrier/gap work.
No query-ring growth or synchronous per-frame polling to force completion.

Give the bounded runner an explicit workload epoch and frame/submission manifest,
recording active input, still/drain and close demand identities through existing
observer contracts. Arrival time of a log line is not the identity of GPU work.
A late receipt for active work belongs to that work; an explicitly identified
close-only frame is separate. Untagged or ambiguous work invalidates the run.
Preserve complete raw receipts for startup, active, still, close and teardown;
never retroactively reclassify the failed r2 run as valid. Input delivery/cadence
and reference output controls remain, including independent PID/epoch checks.

## Bounded proof and explicit authorization requested

After owner approval, implement the graph and measurement correction as one pinned
candidate in small reviewed commits. Reuse unchanged session/adoption/resource
proof at its original scope. No production or proof execution is authorized by
this proposal itself. Required source-health and repository gates still apply.

1. Source and offline controls: old/new union covers all emitted interaction
   pixels; coalesced/mixed updates, overflow and failures force the right graph;
   every suffix pipeline uses one predicate; multi-submission identities cannot
   omit, duplicate or misattribute work. Verify the manifest consumer with late
   active receipts and explicitly close-only work. These controls enable the
   decision to spend GPU correctness proof, not performance acceptance.
2. One focused P630 exact-output batch: compare full-reference and incremental
   outputs at every step of enter/move/leave, all crosshair styles, pane crossing,
   fractional thin geometry, hover/material changes, Console/text/menu/terminal
   overlap, mixed updates, failure/recovery and resource pressure. Explicitly
   read all eight samples for 256 channel values, alpha/BGRA order and asymmetric
   per-sample patterns through UNORM restoration. Negatives must detect omitted
   old damage, omitted suffix predicate, wrong sample, stale B,
   reduced samples and premature resolve. First mismatch stops; no tolerance,
   golden update or format/algorithm sweep. Source/API controls reject sRGB-view
   restoration even if its round-trip happens to produce the same bytes on this
   host; do not invent a nonvacuous pixel mismatch. This decides fidelity/admissibility.
3. One focused measurement-conformance batch covers world/glyph/terminal multi-
   submission work, final mixed-upload batch before rendering, warm restore and
   final resolve, cancellation/overflow and late
   log receipt attribution. Missing work or invalid bounds stops. Complete spans
   must include upload and restore; an omitted final-upload boundary must fail.
   Pass sums never substitute. This decides
   whether the experiment can be interpreted. No producer rehearsal.
4. Only after those gates, one newly declared candidate: same r2 reference scene,
   extent, input path, 5s warmup/30s120Hz input/5s still plus controlled drain,
   three baseline/candidate pairs in each quiet/GPU mode, maximum12 pointer runs;
   alternating order, candidate GPU first. Stop the entire declaration at first
   invalidity, pixel failure, p95>4ms or p99>8ms, with no replacements. Baseline
   has the same observers and is explicitly instrumented. One W-PAN pair per
   mode (maximum4 additional runs) only if pointer gates pass; unexplained >5%
   regression blocks adoption. Recheck source-bound binary hashes; avoid the
   archived shared-Cargo fingerprint collision. No new parameter variants.

This proof scope remains descriptive and nonqualifying until complete DRM duty
accounting is available. It does not reset any r2 failure or waive any of the
225 requirements, CPU/resource limits, exact8x fidelity or independent replay.

## DRM completion and authority still outstanding

Complete DRM client lifetimes still block S4 duty<=25% and final qualification;
they do not inherently block the explicitly limited own-queue experiment above.
The syscall-supervision draft is **not execution-ready**: fd-table races and
transfers, blocking-call supervision, submission-to-fence coverage and observer
lifetime/denominator effects need resolution. It must not be presented as an
approved observer. Source research continues within the repair, followed by a
separately reviewed exact mechanism/privilege/conformance/calibration packet.
No ptrace, retained-fd aliases, global tracing, tooling/dependency addition or
native observer run is authorized here. An incomplete method cannot close the
GPU issue, and the overall repair remains open if this candidate meets timings.

Requested owner response: `approve GPU damage-restoration proposal r3` approves
only the concrete renderer and measurement implementation/proof scope above,
with numbered-decision and same-change governance before execution. Independent
review is advisory and cannot supply this ratification. CPU optimization remains
deferred. S4/S5 and the GPU issue remain unresolved until their required proof.

## Primary source basis

- Installed wgpu-core28.0.1 `device/resource.rs`: multisample usage/sample support
  and only matching sRGB/nonsRGB view-format aliases; `command/transfer.rs`: full
  multisample copy validation. Existing lockfile/dependencies remain unchanged.
- [WebGPU copy validation](https://gpuweb.github.io/gpuweb/#abstract-opdef-validating-gputexelcopytextureinfo):
  whole-subresource restriction for multisampled texture copies.
- [WGSL sample_index](https://www.w3.org/TR/WGSL/#builtin-values): per-sample
  invocation and multisampled textureLoad provide the restoration mechanism.
- [Vulkan image operations](https://github.khronos.org/Vulkan-Site/spec/latest/chapters/images.html):
  format conversion distinguishes UNORM reads from sRGB decoding. Exact backend
  results remain a proof obligation, not inferred from this specification.
