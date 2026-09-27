# GPU redraw proposal r2: shared render ownership through presentation

Status: **owner-review proposal; no implementation or execution authorization**.
Issue: `dat-gui-performance-implementation-vkq`; step: GPI-S4-REASSESS.
This revision supersedes r1's proposed integration and proof boundaries, retaining
its full-sample image-copy mechanism. Owner clarification is recorded in
`owner-clarification-r2.json`; source anchors/hashes are in `reassessment-r2.json`.
The current production renderer remains unchanged. The prior build guard refused
before Cargo; its uncompiled partial attachment patch is historical, not a base
implementation to adopt. Independent review supplies recommendations, not owner
ratification. Neither proposal completion nor a successful experiment closes S4.

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S4-GPU-REASSESS-R2 -->

## Decision requested

Approve a **shared per-host RenderSession, owned by the shared Renderer**, which
accepts typed editor changes, owns derived content validity and frame composition,
and publishes successful-frame receipts through the existing native adapter.
Use an 8x multisample prefix image as its first incremental GPU strategy. Remove
the superseded app-owned projection/cache-validity paths and duplicate legacy
world encoding policy during adoption; keep necessary native lifecycle adapters.
This completes the missing connection between retained content and incremental
GPU execution. It is not a board-only cache bolted onto a pointer event handler.

Also approve the specific proof-scope amendment below: a bounded copy-inclusive
GPU experiment may report its directly observed timings without claiming complete
DRM duty or qualification. Add complete DRM client-lifetime method work to the
repair's scope before final duty acceptance. Neither an incomplete observer nor
r1's narrow observer boundary ends the architecture workstream. All implementation,
builds, tests and native runs remain proposals awaiting explicit owner approval.

## Observed ownership and the actual break

| Stage and source | Current owner and behavior | Finding |
|---|---|---|
| `gui-app/app_native_events.rs:267–329` | Native pointer routing first handles terminal/menu/drag/pan/tool semantics, then hover; a boolean requests workspace redraw. | Correct editor/input responsibility. A pointer event alone is **not** proof of interaction-only damage: it can move a camera, divider, terminal selection or authoring preview. |
| `runtime_camera_pane.rs::update_hover`, `clear_interaction_overlay` | Resolve per-pane hover, update transient session cursor/hover, call `refresh_interaction_overlay`. | Preserve semantic routing, focus and hit authority. No evidence that this routine rebuilds unchanged world geometry on ordinary warm movement. |
| `interaction_refresh.rs:12–95` | Runtime owns several invalidation choices: clear retained scenes/history, clear prepared projection, or mutate only interaction vertices; all mark `scene_dirty`. | Classification exists implicitly across application helpers, but there is no shared render-change transaction delivered to the GPU compositor. `scene_dirty` is not a prefix-validity contract. |
| `runtime_state.rs:33–51`, `retained_scene_history.rs`, `retained_scene_lifecycle.rs` | Runtime holds board/schematic `RetainedScene`, separate bounded histories, `Option<PreparedScene>` and terminal render cache. History already has six-entry/64 MiB limits and charged pinned/retiring metadata. | These are derived rendering resources with working accounting, not authored state. Their policy is app-owned and must join the shared render-session authority, preserving existing limits and refusal behavior. |
| `runtime_terminal_render.rs:7–82`, `runtime_present.rs:67–113`, `main.rs::capture_visual_screenshot` | Several callers ensure/build/store prepared content; terminal snapshots are restored on failed preparation. Camera/grid setters modify the prepared projection after construction. | Multiple preparation entry points can bypass a new image-validity key unless they delegate to one owner. Snapshot consumption/rollback must survive migration. |
| `app_shell.rs:120–130`, `native_frame_coordinator.rs:406–541` | Workspace request increments host damage generation; coordinator coalesces tokens, captures host/device/attempt/damage and retires only successful presented damage. Native exposure can create a redraw even when app content is clean. | Existing shared **delivery** damage is valuable. It does not describe which composed pixels changed. Do not replace it with another scheduler or infer scene invalidation from an exposure token. |
| `runtime_present.rs:118–188`, `native_render_target.rs`, `native_surface_transaction.rs` | Shared target acquires after upload continuation, tracks submission, presents, and separately publishes hit/console state. | Necessary platform integration. The renderer currently receives target/submission callbacks but not a semantic presentation-completion receipt with which to publish a reusable scene image. |
| `render/gpu_frame.rs:77–551` | Shared GPU renderer retains/upload-checks buffers, prepares grids/uniforms/text, then always clears and draws panel/grid/world plus suffix and resolves. | **Confirmed GPU defect:** unchanged composed-world bundles are executed again on pointer-only frames. Geometry/command retention does not retain rasterized samples. |
| `gpu_surface_pass/world_bundles.rs:112–208` | Cached ordered bundles preserve buffers/bindings and are invalidated when encoded dependencies change. | Sound reusable command owner; keep it for cold/content/camera frames. Stop executing it on valid interaction-only frames. Do not call bundle reuse a saved GPU draw. |
| `gpu_frame.rs:239–397`, `coordinate_hit.rs::build_surface_passes` | General compositor has both ordered composed panes and raw single-board/companion branches guarded by empty `surface_passes`. | Mutually exclusive, **not double-drawn today**. They duplicate painter/viewport/camera policy and complicate a universal frame plan. Normalize valid compatibility input once, then retire raw branches; empty/no-scene layouts must remain truly empty. |
| `gpu_overlay.rs`, `global_preferences_window.rs:199,306–454` | Dialog-only content uses a shared specialized one-pass renderer, while the host keeps its own nullable prepared content and completion wiring. | Keep the dialog graph and modality. Adopt the same session/revision/receipt contract; allocate no prefix image to a dialog. This is not evidence that the dialog encoder is faulty. |
| `gpu_surface.rs`, `text_gpu/lifetime.rs`, `gpu_text.rs:100`, `native_queue_owner.rs` | Renderer owns admitted attachment identity and tracked submission holds; native queue owner arbitrates device/host work and retirement. Texture handle is dropped after creating a tracked view. | Extend these existing owners to a tracked image pair, not a second lifetime manager. GPU completion and presentation are different signals. |

The audit supports a missing shared incremental-composition policy, not a
hardware-impossibility result or a Wayland fault. Existing native recovery,
queue admission, text retention and terminal damage are not wholesale failures.
The old 32-region replay failure and merged-pass numerical failures remain at
their original source pins; no new runtime result is inferred from this trace.

## Proposed shared contract and responsibility boundary

`RenderSession` is a cohesive owner inside `datum-gui-render`, shared by Main,
owned dialogs and offscreen capture; not a new thread, process, toolkit or crate.
It owns active/history derived scene references, prepared projection, monotonic
content/composition/interaction revisions, accumulated render changes, image
validity, and the immutable snapshot used by one frame. Renderer resource owners
continue to own actual GPU allocations. Their identity and device/target epochs
are dependencies of the session's frame plan. Per-host sessions share immutable
document payloads through existing charged handles; moving ownership must not
duplicate world storage per host or pane.

Editors still own authored/session state, camera intent, gestures, selection and
terminal input. They submit immutable inputs plus changes; they do not choose
render passes, clear caches, mutate PreparedScene, or publish image validity.
The engine commit/journal and TerminalCore/PTYS remain authoritative.

Proposed internal API, to implement through the existing owners rather than
parallel APIs retained indefinitely:

- `update(inputs, changes) -> Revision`: take coherent typed inputs and accumulate
  changes. `Content` includes stable document/source revisions and retained-source
  handles; `Composition` includes ordered panes, physical geometry, camera/grid,
  filters/style/selection dependencies; `Interaction` includes per-pane hover,
  cursor/style and tool overlay; `Chrome/Text/Terminal` and `TargetLifecycle` are
  distinct causes. Missing/unknown or mixed changes conservatively invalidate.
  Callers supply semantics; shared code resolves dependencies and assigns revisions.
  Change tags are hints, never permission to ignore source/descriptor changes.
  Interaction/tool previews qualify only if the audited contract places all their
  changed pixels in the suffix; otherwise they invalidate the prefix.
- `prepare(FrameRequest) -> FramePlan`: the request carries native exposure vs
  content demand and host/device/configuration/attempt identities from the existing
  platform receipt. Content revisions alone cannot distinguish an exposure that
  requires presentation. Return `Unchanged`, `Full`,
  `RebuildPrefixThenSuffix`, `ReusePrefixThenSuffix`, or existing `DialogOnly`,
  plus an opaque revision/host/device/attempt token and current hit snapshot.
  Revision dominance is explicit: content/composition/target invalidation wins
  over interaction reuse. A full change followed by pointer movement before a
  frame must never downgrade to interaction-only.
- `encode(plan, target_adapter) -> SubmittedFrame`: only the shared encoder chooses
  the GPU graph. Upload-only work returns deferred without consuming render damage
  or acquiring a surface. No editor branch chooses the prefix cache.
- `complete(token, outcome)`: shared receipt matching accepts only the matching
  successful presentation/offscreen completion, publishes prefix validity and
  presented hit/layout snapshots, and retires only the captured revisions.
  Failed/deferred/dropped attempts keep newer and pending changes; old host/device
  completions cannot validate a new image. Native submit/present errors and observer
  failures report through the same adapter; drop without completion fails closed.

`NativeFrameCoordinator` retains native wake/coalescing/fairness/recovery and its
host damage receipt. It carries/references the opaque render token; it does not
classify editor pixels. `SurfaceTransaction` retains acquisition/configure/present,
`QueueOwner` retains admission/completion, and the session retains content-validity
policy. Unsolicited exposure is a presentation request, not proof of content change.
Bridge both receipts in one shared native completion adapter used by Main and all
owned hosts; do not add unrelated per-window cache-commit calls.

TerminalCore row damage and snapshot acknowledgements remain their existing
contract. The render session owns only derived terminal layout/composition;
preparation failure restores the producer snapshot/damage as today. Retain a
snapshot/damage lease through later encoding/acquisition/presentation failures;
merge newer producer damage rather than overwrite it with an older snapshot. Presented
hits refer to the successfully presented snapshot, never speculative geometry.
Offscreen capture uses the same plan/encoder with explicit completion after its
submission/readback succeeds; it does not fabricate a native presentation or acknowledge native damage/publish
native displayed-hit state. A capture completion belongs to its own target token.

## Superseded paths and adoption endpoints

| Remove or replace in this correction | Replacement and preserved behavior |
|---|---|
| Runtime's independent `prepared_scene` nullable-cache policy, `scene_dirty` render-validity role, and direct projection mutation in `interaction_refresh.rs` | Session-owned prepared projection/revisions. Existing helpers become typed input adapters; remove direct cache writes after all callers migrate. Keep model/gesture changes and diagnostics explicitly separate. |
| Duplicated ensure/build/cache decisions in `runtime_terminal_render`, `runtime_present` and native capture | One session prepare/snapshot API, with terminal rollback and current/presented hit semantics preserved. Capture/native are target adapters, not separate content owners. |
| App-only derived `RetainedSceneHistory`, active board/schematic ownership and key validity policy | Move the existing accounted history implementation to the shared derived-content owner, with explicit source/view dependencies. Keep algorithms, six-entry/64 MiB caps, equal-size replacement checks, active/pinned accounting and refusal/retry rules. Do not optimize CPU selection/history algorithms. App retains authoritative source handles, not another shadow cache. |
| Public production mutation of prepared camera/grid fields after revision selection | Typed camera/grid update before immutable plan creation. Retain existing pane camera and grid math; selection/style changes that affect world pixels are composition/content changes. |
| `gpu_frame` raw board/companion underlay/world branches selected by empty composed passes; redundant legacy camera bindings if no consumers remain | A single normalized ordered scene descriptor feeds shared grids/world bundles. Compatibility construction maps valid legacy input once; zero-world/Revision/unadmitted schematic stays empty. Delete raw encoding branches and bindings only with consumer proof; never count them as currently duplicated GPU work. |
| Unconditional full-prefix execution for valid interaction-only frame plans | Shared plan selects stored 8x prefix copy and common suffix. Full rendering remains a required strategy for changes/refusal, using the same prefix/suffix encoders, not a competing legacy implementation. |
| Editor-facing `render_with_acquisition(prepared, retained, schematic_retained, ...)` composition authority | Session plan/receipt entry point after all native/capture consumers migrate; keep the narrow late-acquisition callback internally. No parallel editor API that bypasses revision validation. |
| Main/dialog-specific completion publication that a cache could bypass | One receipt bridge publishes matching session image/hit state after native presentation and handles failure/drop. Keep native host input/modality and platform calls. |

The unchanged 14-consumer map remains authoritative: MAIN, BOARD, SCHEMATIC, REVISION, GLOBAL, PROJECT, NEW, LAYERS, NAV, INSPECTOR, MENU, CONSOLE, TERMINAL, TERM_OVERLAY.
These enter through their existing host (not fourteen separate image caches);
resolved-schematic qualification exclusions remain explicit.

Retain `gpu_overlay`'s dialog-only graph, board/schematic geometry producers,
world bundles, terminal/text owners, native queue/recovery/surface adapters, and
editor semantic routing. They implement distinct responsibilities rather than a
second incremental compositor. Adopt the session contract in all current native
hosts and offscreen entry points; only eligible composed Main frames use the
image cache. Future editors submit these same contracts. Source-health extraction
must move actual ownership and ratchet touched debt; no forwarding-only migration.

## Exact GPU work, output and resource contract

Keep r1's prefix boundary after panel/underlay/ordered grids/world and immediately
before schematic interaction. Keep the entire original suffix: schematic
interaction, viewport overlay, board interaction, console, terminal background
images, workspace text, terminal foreground images, menu card, menu text. Initialize
pipeline, binding and scissor state explicitly in a new suffix pass.

| Plan | Shared encoder work |
|---|---|
| Warm interaction-only with matching dependencies | Full equal-format/extent 8x A-to-B texture copy; B `Load`; suffix once; one final resolve; B `Discard`. **Zero panel/grid/world draws or world bundle executions**; no unchanged world uploads/allocations. |
| Eligible changed/cold prefix | Original ordered prefix into A, clear/store without resolve; full A-to-B copy; identical suffix/resolve. Added store/copy/pass break is a measured cold/camera cost, not free. |
| Mixed/nonpointer change initially, admission refusal, unsupported configuration | Correct full draw using the same encoders; invalidate cached prefix. No separate per-editor GPU fallback or silent omission. |
| No content change and no native exposure | No new GPU work under existing scheduling. Exposure redraws current content using a valid plan; it does not blindly clear content generations. |

The warm path removes repeated prefix vertex processing, fragment shading,
blending and bundle execution. It still performs a full multisample copy, suffix
work (including text), resolve and presentation. This does not promise zero GPU
work or guaranteed 4/8 ms timing. Retaining buffers alone already works and is
not the proposed performance gain. No repeated region replay, tolerance change,
1x approximation, 4x fallback, gamma/filtering shader or lower input delivery.

A/B share exact format, extent, sample positions/count, mip/layer and device.
A uses RENDER_ATTACHMENT|COPY_SRC; B RENDER_ATTACHMENT|COPY_DST. Pinned native
wgpu28 permits full equal-sample copies; actual device support and sample-exact
output remain proof requirements. One final 8x resolve preserves coverage/blend
ordering; stored samples followed by identical suffix is an argument to verify,
not a visual acceptance result.

Session-owned explicit revisions plus retained allocation identities, ordered
pane descriptors, camera/grid/style/layer inputs, physical target and host/device
epochs form validity. No hash-only or pointer+length key. Equal-length replacement,
cloned then changed projection, visibility/order, extent/DPI, atlas/terminal/menu
changes and mixed pending changes must invalidate conservatively. Cache validity
is published only on successful matching completion; acquisition failure,
upload-only continuation, submit/present/observer error, device loss, close or
uncertain exposure state cannot publish a stale image. Keep pending damage and
surviving pane cameras through recovery.

The prefix remains optional, at most 45 MiB per eligible host, actual size; image
key/metadata at most 4 KiB including owned lists. This small cap covers **image
validity metadata**, not the existing session/history/text data, which retain all
original separately accounted caps. For 4-byte pixels, each image is
`width * height * 4 * 8` checked bytes: 31.25 MiB at 1280x800, 45 MiB at 1536x960.
A+B are 62.5/90 MiB; old+new pairs are 125/180 MiB. Pair admission reserves both
allocations under existing 512 MiB process and local limits; the two images share
one generation allowance, maximum current plus retiring. Track texture/view
aliases once and each image separately. A failure must roll back all new holds;
optional A cannot starve required B. Release optional resources safely before
rejecting required rendering; no increased cap or hidden history.

Keep real texture handles through copy and submission completion. Shared lifetime
records track creating/prepared/submitted/retiring ownership and preserve allowance
across device replacement. GPU completion releases submission holds; presentation
publishes content validity. Neither signal substitutes for the other or proves
compositor display timing. At 1280x800, logical copy+resolve traffic is about
97.66 MiB/frame before suffix/driver effects; this is arithmetic, not measured
bandwidth, a hardware limit or expected runtime.

## DRM scope amendment and separate claims

The preserved `independent-replay/drm-client-method.json` documents the real gap:
sequential fdinfo snapshots and outer app-device hooks miss clients that open and
close inside loader/driver calls, earlier exits, reset epochs and final undrained
work. Same endpoints cannot prove that nothing happened between them.
The [kernel DRM usage specification](https://docs.kernel.org/gpu/drm-usage-stats.html)
defines client IDs for deduplicating shared fds; it does not make an endpoint scan
a complete lifecycle observer. The [ptrace interface](https://man7.org/linux/man-pages/man2/ptrace.2.html)
provides syscall/clone/exec/exit observation, but stopping threads can perturb work.
These are method candidates/constraints, not a validated measurement solution.

| Intended result | Does the missing complete DRM method block it? |
|---|---|
| Source architecture proposal and focused correctness/resource-lifetime controls | No. They establish contracts and owned-resource behavior, not total device duty. Current phase still runs none. |
| A narrowly labeled, owner-approved experiment measuring copy-inclusive **own-queue GPU timestamps**, exact output and work counters | Not inherently. Its timer must include copy/barriers/suffix/resolve, every required sample, and calibrated observer effects. Incomplete DRM observations remain explicitly incomplete; no total-duty, full tier, hardware-limit or overall improvement claim. **R1 procedurally prohibited this; r2 requests that exact scope amendment.** |
| Focused experiment claimed to demonstrate complete engine duty <=25%, or full CPU/GPU/resource compliance | Yes. It needs complete accounting for that claim, regardless of its short duration. |
| S4 correctness/performance exit and final S5 qualification | Yes for the required duty/lifetime claims. A favorable own-queue distribution cannot close them. S5 also retains broader configurations/endurance/independent replay and owner acceptance. |

Requested method scope addition: an isolated, Datum-owned **workload-process
DRM fd-lifecycle observer and conformance work**, spanning pre-spawn through normal
drain and teardown. It must identify processes by PID/start time, client/device
and lifecycle epochs; cover open/dup/inheritance/replacement/close/close_range,
exec and exit across threads/children; deduplicate aliases; retain final readable
counters for earlier clients after their queued work is drained; detect reset,
lost events, missing final readings and observer failure. A pre-close read alone
is insufficient. Unknown completeness always yields unavailable, never zero.

This expands r1 beyond app endpoints/existing-observer-only changes to observation
of the workload's fd lifecycle. It does **not** authorize global tracing, system
or driver tuning, CPU event-loop optimization, dependency changes or attaching to
unrelated processes. No observer implementation is selected or ratified here:
source review must select a feasible mechanism with exact privileges, race/drain
coverage and overhead disposition before an observer execution packet. Syscall
supervision is a concrete candidate to assess, not permission to adopt ptrace or
change process lifetimes by retaining fds. Reusing/draining app-owned clients may
simplify steady frames, but must prove coverage of internal clients rather than
assume it. High-overhead lifecycle conformance cannot be used as low-overhead
performance evidence without the required calibration.

Thus the overall repair includes the missing method. Architecture implementation
and a separately scoped useful experiment need not wait for final qualification;
qualification must wait. If the observer needs additional authority, return its
specific reviewed method/privilege amendment, not a generic stop of GPU repair.

## Concrete delivery and future proof packet

After explicit owner approval, deliver one architecture correction in reviewable
units: (1) typed session/snapshot/receipt adoption across native and capture with
existing graph; (2) normalize and remove superseded preparation/world branches;
(3) attach the 8x prefix strategy and tracked pair to that single owner. These
are ordered parts of one design, not alternative optimization trials. The
reference/full strategy shares painter code; rollback restores the prior source
and preserves authored state, evidence and unrelated changes.

The proposed minimum proof is decision-driven, not another open-ended campaign:

1. **Contract/adoption/correctness batch:** all current consumers reach the shared
   owner; retained-history behavior and caps survive the move; mixed update order,
   old receipts and equal-length replacement fail closed. Exact current-vs-candidate
   8x pixels cover pointer enter/move/leave, crosshair styles, thin/overlapping and
   fractional geometry, multiple panes, hover, terminal image/text layers and menu
   order; changes/recovery/pressure/retirement preserve output and lifetime. Stale
   key, missing copy, reduced samples and premature resolve are targeted negatives.
   Failure stops the affected proof; no tolerance or golden refresh. This proves
   adoption/fidelity, not performance or platform universality.
2. **Optional focused experiment scope:** only after that batch and complete
   copy-inclusive timer/input/output conformance, one pinned candidate, three
   baseline/candidate W-POINTER pairs in each quiet/GPU mode, maximum 12 runs,
   5 s warmup + 30 s schedule + identical idle/drain; alternate AB/BA and stop at
   first invalid/correctness failure or GPU p95 >4 ms / p99 >8 ms. No replacement
   runs or producer rehearsal. One W-PAN pair per mode (maximum four runs) checks
   cold/camera cost; unexplained >5% regression blocks adoption. Three pairs are
   descriptive, not seven-pair formal inference. Label DRM duty unavailable and
   **no S4 qualification** while its method remains incomplete. These are useful
   measured command-graph results, not permission to waive CPU/resource limits.
3. **Complete method and qualification:** review the workload-fd observer mechanism
   before implementing/running it; offline/lifecycle negatives and a separately
   bounded overhead/drain calibration must earn that execution scope. Then reuse
   valid unchanged evidence and run only missing required qualification. Complete
   duty, correctness, affected resources and all original S4 obligations must pass
   before S4 closes. Preserve all 225 mapped requirements and S5 exclusions.

The experiment timer retains r1's diagnostic-only tracked 1x1 marker attachment
before a warm copy, through final suffix end; cold span starts at prefix. Report
copy-inclusive span separately from pass sums/logical copy bytes. Never report
suffix-only time as full frame. Native launch/storage prerequisites are checked
before spending proof work; the prior 5.8-vs-6.0 GiB `/tmp` refusal is not an
architectural finding or a reason to stop this planning work. No run is performed
by this proposal. Owner approves exact implementation/proof scope; independent
review remains advisory, and the GPU issue stays open until required results exist.
