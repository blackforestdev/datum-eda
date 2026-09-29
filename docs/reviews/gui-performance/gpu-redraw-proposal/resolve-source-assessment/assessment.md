# Remaining full-target work: evidence and architecture constraints

Status: source-only assessment within GPI-S4. No mechanism amendment,
implementation, build or runtime scope is approved here. The latest campaign
remains stopped after its valid p95 failure. Its report and archived raw evidence
are unchanged. `receipt.json` pins the extracted existing observation and sources.

## Existing draw-free observation

In the painter/hover native trial, host 1/device epoch 1/frame 2 has final attempt
`[1,3,10,10,1,1,1]`, workload phase 1/demand 5, and these GPU intervals:

| Interval | Milliseconds |
|---|---:|
| upload-leading | 0.018000 |
| suffix | 3.665917 |
| complete span | 3.728667 |

It has no restoration or prefix pass. The matching input record retains render
revision 10, preparation activity 10, extent 1280x800, scale 1, camera and empty
pointer/hover state. Activity advances from presentation attempt 2 to 3.

The production control flow makes this an empty-damage case: in
`session_prefix.rs`, a matching completed revision returns an empty `Pixels`.
In `gpu_frame.rs`, a nonempty accepted damage set emits a restoration pass;
an unavailable reusable prefix emits the full prefix/frame graph. The empty
accepted set enters the final suffix pass but makes zero calls to
`draw_frame_suffix`. The pass still loads/stores composed eight-sample B and
resolves the full target. Test fault branches are absent from this release.

Thus substantial suffix time can occur without suffix draw traversal. This is
stronger evidence than the active pass totals alone and does not require a new
observer or native diagnostic. It is only one startup frame: do not subtract
its duration from active samples, label it the active p95 floor, attribute an
exact percentage to resolve, or claim a hardware/Wayland limit. Attachment
handling and resolve remain combined. The active p95 of 4.330583 ms still fails.

## API constraints verified in installed wgpu 28

The receipt pins `wgpu-core-28.0.1` source hashes. `command/render.rs` requires
color and resolve views to have equal render extents and matching formats;
the source is multisampled and destination single-sampled. The ordinary resolve
attachment provides no per-damage destination rectangle. A painter scissor does
not restrict that resolve. `command/transfer.rs::validate_texture_copy_range`
rejects partial x/y copies of multisampled textures. Copying a rectangle out of
B directly to a smaller eight-sample texture is therefore not an available
shortcut in this implementation.

The same installed render validation permits bounded negative viewport origins.
That makes integer viewport translation a candidate for preserving the existing
vertex coordinates when drawing into smaller attachments. It is not evidence
of exact raster equivalence: framebuffer positions, damage predicates, original
clips, derivatives, helper invocations, edge tiles and sample placement still
need explicit treatment and proof. Do not assume unchanged vertex shaders alone
make tiled rendering exact.

A shader average of eight samples is also not an established substitute for
the existing resolve. Vulkan documents implementation-dependent sRGB transfer
behavior, clarified by maintenance10; the current adapter has not been qualified
for equivalence to a custom average. Keep hardware resolve as the reference.
See the [Vulkan render-pass specification](https://docs.vulkan.org/spec/latest/chapters/renderpass.html)
and [maintenance10 proposal](https://docs.vulkan.org/features/latest/features/proposals/VK_KHR_maintenance10.html).
These are API constraints, not measurements of the reference host.

## Candidate comparison and resulting design work

| Approach | Work removed | Limitation |
|---|---|---|
| Further glyph/draw culling | Some repeated suffix dispatch/vertex work | Leaves the demonstrated draw-free attachment/resolve work; insufficient basis for selecting the next correction alone. |
| Extra timestamps or another unchanged trial | None | Existing draw-free evidence already establishes the architectural concern; no new probe is justified for that question. |
| Retain a resolved image only | Repeated resolve on unchanged exposure | Active pointer updates still resolve the full target; not a complete answer to the failing workload. |
| Shader resolve into retained output | Could restrict active resolve work | Exact sRGB/rounding equivalence is unproven; not selected as a silent replacement. |
| Shared bounded eight-sample regional composition plus retained resolved output | Full-target warm B attachment and resolve work | Requires a concrete bounded layout, coordinate/fidelity proof, presentation method and resource amendment before execution. |

Advance the last architecture through source design. The shared RenderSession
must retain resolved composition validity alongside prefix/dependency state.
Editors continue submitting typed changes. A warm update reconstructs damaged
samples from prefix A, paints the original ordered suffix once per affected
pixel into bounded regional working storage, performs the existing hardware
resolve there, and updates retained single-sample output. The native backend
presents that complete output; it must not become the owner of damage history.
Empty exposure reuses resolved output without an eight-sample pass.

This would supersede the warm full-target B load/store/resolve and exposure
resolve paths. It must preserve the cold/full fallback, actual-pad invalidation,
8x fidelity and every painter's original clipping/order. A full-target single-
sample presentation operation may remain and must be included in complete GPU
timing. No performance result or chosen tile size follows from this assessment.

Before a concrete execution proposal, settle: deterministic bounded regional
layout without a tuning sweep; integer coordinate and fragment-position mapping;
sample-preserving A-to-regional restoration; how resolved pixels reach every
new swapchain image; overflow/refusal fallback; failed/stale presentation and
device/extent/format recovery; and an allocation ledger for all current,
submitted and retiring storage. Include fallback peak overlap, not just warm
steady-state bytes. Existing PM050 says no new image and one final resolve;
this direction requires an explicit owner-ratified amendment, not interpretation
of the previous approval. Resource caps and correctness thresholds stay fixed.

Do not expand measurement infrastructure to justify this design. Reuse the
successful 81 pixel checkpoints and six all-sample comparisons where unaffected;
eventual new proof must target changed regional boundaries, retained output and
resource lifetimes, then measure the full approved workload including
presentation. Existing evidence does not guarantee this architecture meets p95.

Complete DRM lifetime accounting/duty remains a separate qualification
obligation. It does not block this source design or a separately approved focused
experiment, but it still blocks final S4 qualification. CPU event-loop work
remains deferred. S4, S5 and the issue remain open.
