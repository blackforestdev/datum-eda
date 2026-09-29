# GPU attribution study A1 approval packet

Status: proposed; no implementation, build or GPU execution is authorized yet.
Source: `424beb32e5f6ce0c9392203df922208494814c0e`. Issue:
`dat-gui-performance-implementation-vkq`, GPI-S4. Approval response:
`approve GPU attribution study a1`.

## Decision this study enables

Determine whether the measured R4 cost is sensitive to its measurement markers,
to repeatable Datum preparation, to its restoration and painter commands, or to
the backend image-operation sequence after those commands are removed. This is
one attribution study, not a renderer candidate, architecture redesign, tile
sweep, qualification campaign or search for a passing number.

The outcome must identify a measured, material cost before recommending further
optimization. Inconclusive evidence ends the study. A low-cost diagnostic control
is not a usable renderer, and a slow control is not a hardware impossibility
proof. Results cannot partition the historical native p95 into percentages.

## Fixed host and inputs

Use the same Intel P630, Vulkan, Mesa identity and X11 native surface as the valid
R4 result, 1280x800 physical pixels, scale1, Bgra8UnormSrgb, exact8, Fifo. Use the
same pinned CLI, F-DOA project, normalized board and source generation. Their
hashes and two exact archived input records are in `inputs.json`. No new fixture,
synthetic application scene, settings change or system tuning.

The two cases are real adjacent pointer transitions, chosen geometrically before
this study rather than by timing rank:

- H: (499,150) to (500,150), archived sequence409, unchanged zone hover.
- V: (700,269) to (700,270), archived sequence1048, unchanged zone hover.

Use the archived camera, selection, layout and hover identity. Derive the ordinary
R4 old/new damage union and tile plan; assert the warm graph, unchanged strong
revision and no world upload. Preserve 32-pixel tiles,256slots,512atlas. Record
exact tiles, commands and upload bytes; do not substitute a more favorable case
if setup is unsupported or overflows.

## Common reset and measurement boundary

Use one visible native X11 surface and the production format/usage/acquisition
contracts. Acquire before each sample; record acquisition duration separately.
No synthetic XTest stream or normal application event-loop benchmark is needed.

For each case build the full-reference old and new outputs using the existing
common painter. Every observation starts from the same old composed C and the
same retained prefix A. Outside timing, initialize T with the selected prefix
samples and R with their hardware-resolved pixels. This uses existing restoration
and hardware resolve with a diagnostic Store for initialization; timed operations
retain production Store/Discard choices. No new shader, resolve formula or
multisample partial copy. Complete initialization before measurement.

Controls that omit work therefore read initialized images. None is allowed to
benefit from undefined/discarded contents. The common reset is extra diagnostic
work, not a production optimization, and its cost is reported separately and
excluded from attributed intervals. It can affect cache state; conclusions are
conditional on this common reset, not native workload percentiles.

Preparation and pointer uploads complete before timed GPU graph submission.
Measure CPU wall time for preparation/plan, upload API calls, graph encoding,
submit and completion separately with a monotonic clock. Drain preparation writes
before graph timing for every condition. Never classify an excluded preparation
upload as free: retain its bytes and separate duration. The primary GPU observable
is the resident R4 restoration/painter/resolve/copy graph, not the historical
complete-frame span. There is one measured graph submission in flight; its wait
and sampling cadence are identical in every condition. Presentation follows the
normal surface contract; GPU completion and presentation are distinct events.

## Seven conditions

All conditions use the same resources, initial contents, target, source buffers,
formats, sample counts and immutable case inputs. N,D,M,P must produce the exact
new full-reference image. A,B,C must produce the same deliberate ablation image:
old C with whole selected tiles replaced by hardware-resolved prefix pixels.
They intentionally omit the suffix and are never scored as application output.

| ID | Exact change from resident R4 graph | Purpose |
|---|---|---|
| N | Full preparation and complete R4 graph; no GPU queries or marker passes | Completion-wall control for measurement perturbation |
| D | Same as N, with one encoder timestamp immediately before restoration and one after final C-to-target copy; no render-pass markers | Common GPU interval for contrasts |
| M | Same as D, plus the existing production leading marker, restore/suffix pass timestamp writes and final1x1 marker in their original order | Measure the combined production marker/query scheme against D; compare outer versus legacy inner intervals |
| P | Same timed GPU commands, uploads and output as D, but reuse the frozen prepared scene and damage plan for this exact old/new transition | Isolate repeatable preparation/plan construction CPU cost, without a new GPU algorithm |
| A | D with suffix draw calls omitted; restoration, suffix attachment load/resolve/discard, tile copies and final presentation copy unchanged | Conditional cost of painter commands: D versus A |
| B | A with restoration pass omitted; preinitialized T supplies the identical prefix samples | Conditional restoration cost: A versus B |
| C | B with atlas attachment/resolve pass omitted; preinitialized R supplies identical resolved pixels; retain all tile and presentation copies | Conditional atlas attachment/resolve cost: B versus C; C measures the remaining copy sequence |

P must replay the identical preparation-upload list, including bytes, resources,
offsets and ordering. It changes only derivation of immutable CPU results. It
must not skip uploads, change cache admission, alter shaders or batch commands.
Before timing, assert D/P equality of upload hashes and ordered graph descriptors
(draw/instance counts, binds, clips, copies and attachment operations). If this
cannot be achieved through existing shared preparation/encoding owners, stop
before GPU entry; do not build a second renderer or reinterpret the control.
P measures preparation cost, not the separate CPU event-loop issue.

A/B/C are operation controls, not proposals to remove necessary work from a
usable frame. In particular, D-minus-A includes both useful and potentially
redundant painter work and content-dependent backend effects. A-minus-B measures
this implementation's restoration sequence, not a universal restoration cost.
C includes backend execution/transitions for Datum's chosen copy fragmentation;
it is not minimum memory bandwidth. Report conditional contrasts and residuals,
not an additive division into unavoidable versus avoidable milliseconds.

## Timing validity and marker effects

D,M,P,A,B,C use the same diagnostic two-query encoder bracket. M additionally
retains the production pass/marker queries; keep total queries <=32 and one
fixed query/readback slot. Resolve/map after the measured end, outside its GPU
interval; measure their wall overhead separately. N has no query writes,
resolve/map or1x1 markers. All conditions retain a completion fence/wait, so
submit-to-completion wall times are comparable but include driver and host wait
cost. No subtraction turns wall time into GPU busy time.

At device creation require TIMESTAMP_QUERY and TIMESTAMP_QUERY_INSIDE_ENCODERS,
and the existing exact8 surface capabilities. Installed wgpu-hal28 maps encoder
timestamps to Vulkan BOTTOM_OF_PIPE. However, wgpu explicitly warns that encoder
timestamps are not general ordering barriers. Do not insert an unsafe Vulkan
barrier or claim interior stages are isolated merely from timestamp placement.
Use only whole-submission resident-graph intervals, verify source/feature identity,
and interpret them alongside completion-wall controls. N versus D estimates the
net query/readback scheme's completion-wall perturbation; M versus D estimates
the incremental production marker/pass-query scheme. This does not isolate
individual markers or establish a zero-overhead clock.

Before timing, run exactly eight conformance cases twice (16 observations):
empty encoder; empty render pass; known4MiB buffer copy; dependent two-copy chain;
known exact8 draw plus hardware resolve; two ordered drawing passes; production
1x1 marker only; known4MiB copy with queries disabled. Check expected bytes,
finite positive timestamp period, enabled features, complete bounded query
indices and nonreversed available timestamps. Do not require empty work to be
zero or more work always to be slower. Offline validator negatives reject missing
queries, duplicate samples and reordered/mismatched case IDs. Those intentional
offline rejections do not spend GPU observations.

Conformance does not prove absence of timestamp perturbation. If N/D/M completion
comparisons contradict a GPU-only attribution or reveal unstable perturbation,
mark the affected attribution inconclusive. Do not repair the timing system or
launch an alternate measurement technique within this packet.

## Exact build and run limits

After approval, implement only a diagnostic feature and ignored optimized test
entry in gui-app, shared renderer diagnostic helpers, and a bounded writer/runner.
Suggested ownership: `render/gpu_attribution.rs` and one control helper;
`gui-app/src/gpu_attribution.rs` registered from native GPU tests. Small feature,
visibility and module-registration changes may expose existing owners. Ordinary
production behavior is unchanged with the feature absent. No new dependency,
shader, observer thread, normal-renderer strategy or native event-loop change.

- One optimized diagnostic test build through the guarded offline Cargo runner;
  no candidate or baseline release build. At most two guarded scoped check/Clippy
  invocations for diagnostic/default configurations. No build variants or retries
  after a compiler/check failure; preserve the failure and stop. Pure offline
  controls execute from that one test binary before the GPU launch.
- One diagnostic GPU process, one device/window, maximum300seconds inclusive of
  setup, controls, sampling, output checks and cleanup. No replacement process.
  No other Cargo/compiler process may be active during the study.
- After the16 conformance observations, run14 untimed correctness observations
  (seven conditions for each case). First mismatch stops. For each case, then
  four warmup observations per condition (56 total), followed by five fixed
  blocks of eight cycles of all seven conditions (560 measured observations).
  H finishes before V; comparisons remain within each case. Total maximum646
  observations. No extra tail collection or repetitions.
- Within each case/block, cycle r=0..7 starts the order N,D,M,P,A,B,C rotated by
  `(8*block+r)%7`; reverse that order when `(block+r)%2==1`. Block is0..4.
  Fixed order counterbalances position; it does not create independent hosts.
- Cap total queue submissions, including reset/readback/setup, at2200; diagnostic
  CPU storage at64MiB, raw numeric output at16MiB, and total tracked GPU ownership
  at the unchanged512MiB cap. Keep one case's resources and one sample in flight;
  no streaming observer. Reuse one bounded image-readback buffer. Missing
  accounting, capacity overflow or inability to fit stops before sampling.
- Validate exact target bytes after every observation, outside timing. Do not
  retain646 images: store hashes and scalar receipts; preserve the two reference
  images and first failure images if any. Keep source/binary/input hashes and
  all measured/warmup records, reset costs and acquisition/wait/readback times.

Offline review before launch must seal source and executable hashes, confirm
normal builds contain no diagnostic entry, and compare D/P operation manifests.
One scoped idle cache invalidation may precede the sole optimized build to avoid
the documented shared-baseline cache collision; verify fresh compilation and
binary identity before launch. No shared cache sweep or baseline rebuild.

## Predeclared analysis and stopping decision

For each case and condition report all five block medians, their range, overall
median and maximum for GPU interval where present and for each CPU/completion
phase. Report within-block paired median differences for D/N completion, M/D
GPU and completion, D/P preparation, D/A GPU, A/B GPU and B/C GPU. Never subtract
p95 values or add marginal effects as if operations had no interactions.

A contrast is a material diagnostic signal for a case only when all five block
differences have the same sign and their median magnitude exceeds both0.25ms
and5% of the control median. Otherwise report no resolved material difference;
that is not proof of equivalence. These are descriptive selection rules, not
confidence intervals, formal speedup claims or MET/STAT acceptance. Both cases
must agree for a general recommendation; disagreement remains case-specific.

| Outcome | Allowed interpretation and disposition |
|---|---|
| M materially exceeds D, or D materially exceeds N in completion time | Measurement scheme affects cost. Report magnitude at this scope; do not attribute historical tails to rendering or subtract overhead to pass4ms. No renderer redesign recommended from this result. |
| P lowers preparation time with identical GPU work; GPU/completion graph remains comparable | A measured amount of Datum preparation can be bypassed for frozen inputs. It is an upper opportunity bound, not a safe production optimization or a remedy for GPU execution limits. |
| A materially lowers D | Painter commands/content contribute materially. Useful rendering and redundant dispatch remain mixed; this supports a narrowly scoped command audit, not an invented hardware floor or approved optimization. |
| B materially lowers A | The current per-tile restoration sequence is costly. It does not prove exact8 restoration is inherently that costly or that a faster replacement exists. |
| B or C alone has median>=4ms in both cases with stable marker controls | R4's tested image-operation graph consumes the budget even with painter work removed. Preparation-only optimization cannot fix that graph; this supports stopping work on it, not declaring all strategies infeasible. |
| B/C are low, while full graph remains high and differences are reproducible | Fixed image operations alone do not explain the deficit. Implementation-controlled restoration/painter structure is a candidate for further investigation, subject to an explicit value decision. No new renderer implementation is authorized. |
| Missing capability, failed correctness/conformance, unstable marker effects, irreproducible contrasts, or historical native tail not reproduced | Stop and report the affected question inconclusive. No alternate host, backend, tile size, shader, extra sample batch or successor study. |

The study always ends after the fixed batch and report, even with a clear signal.
It does not run until4ms passes. Report whether another optimization has a
measured opportunity; separately state whether its likely implementation and
qualification effort is justified. There is no automatic recommendation to spend
more merely because a contrast is measurable.

## Completion boundary and approval

Required report: measured backend-operation controls; measured marker effects;
preparation and painter/restoration contrasts; correctness and accounting checks;
limitations from fixed cases, common initialization and serialized execution;
and an explicit proceed-with-a-narrow-question, stop, or inconclusive conclusion.
It must not promise a universal hardware lower bound or assign a percentage of
historical native p95 to one cause. No formal4/8ms, duty, endurance or S4/S5 pass.

Production R4 and both prior valid candidates remain preserved. Complete DRM
lifetime/duty qualification, CPU optimization and all broader qualification are
outside this study. No execution occurs until the owner approves this pinned
packet. Approval permits only the limits above; an inconclusive result ends it.
