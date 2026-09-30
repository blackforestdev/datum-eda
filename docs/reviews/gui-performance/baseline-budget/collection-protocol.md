# GBB-P01: bounded reference baseline collection

<!-- EVIDENCE:GUI-PERFORMANCE-BASELINE:GBB-P01-PROTOCOL -->
This protocol follows PM051 and the baseline plan. It admits only methods whose
input/output and timing boundaries can be verified now. It freezes the complete
workload inventory, including unavailable cells; it does not label the admitted
pointer subset representative of all GUI functionality. No renderer changes,
rebuild, observer redesign, additional hardware or optimization sweep.

## Candidate and reusable evidence

Use the exact corrected R4 binary, source and host configuration recorded in
`baseline-inventory.json`; its SHA256 was reverified before this protocol. CLI,
fixture, independent archived reference image and all eight original pointer
method files also match the R4 declaration. Source at c3949e53 is unchanged by
this collection. Preserve original files and the stopped historical campaign.

R4-C-G1 has 1275 causally active GPU frames out of 1278 complete frames. Recomputed
nearest-rank p95/p99/max from archived raw ticks agree exactly with the original
6.700166871139526 / 10.205083644767761 / 12.937750394828797 ms. Credit it as one
of three GPU-mode trials. It is separated in time from the new batch and has no
matched quiet partner. Do not report a three-pair contemporary comparison.

## Workload/method admission matrix

| Family | Reusable evidence | New collection in this packet | Remaining limitation |
|---|---|---|---|
| Pointer | R4-C-G1 complete GPU/input/output proof; independent archived readiness/final image | Two GPU-mode and three quiet-mode trials | Only warm single-board pointer; two contemporary pairs |
| Idle | Earlier idle-host batch supplies recipes and static reference evidence | None | Current causal schedule requires 30 seconds, unlike the 60-second recipe; old performance observations use another candidate |
| Pan | Existing native camera recipes and shared camera math | None | Current producer/oracle is pointer-only; held-pan delivery and camera oracle not yet reconciled to this candidate |
| Zoom | Existing wheel/anchor semantics | None | Current receipt does not preserve fractional wheel deltas; no verified complete delta oracle in this method |
| Preferences scroll/controls | Archived 30-cycle control proof and independent images | None | Main-only causal observer rejects another host; old controls evidence explicitly excludes calibrated numeric scroll state/cost |
| Pane transitions | Archived pane sequence and images | None | Snapshot captures main camera/final focus, not full per-pane camera/lifecycle state |
| Native window lifecycle | Archived window recipes, host images and accounting limitations | None | Causal manifest prohibits another native host/device epoch; removing it would lose the current causal timing proof |
| Mixed interaction | Existing specified bounded PTY recipe | None | Current pointer oracle does not validate PTY byte conservation/terminal output; no false mixed-workload claim |

These are concrete method capability gaps, not measured renderer failures. No
unavailable family receives fabricated samples, a zero cost or acceptance credit.
Historical observations remain valid at their original scope but are not pooled
into this candidate's distributions. This bounded packet ends with partial
baseline findings; it does not automatically launch method expansion.

## Exact controls and fixed observations

Run the derived `pointer_trial.py` with `pointer-declaration.json` and index 0–4.
The derivative preserves the original R4 native method, independent image oracle,
3600 requested positions at 120 Hz on the same 400×240 rectangle, readiness,
5-second warmup / 30-second active / 5-second tail, causal demand/submission
validation, exact final crosshair state, controlled device-live drain and cleanup.
It changes only declaration/output locations and the withdrawn numerical stop.
An outer 300-second process deadline and bounded evidence storage apply.

Fixed order: Q1, G2, Q2, G3, Q3. G2/Q2 and G3/Q3 are the two contemporary paired
comparisons; Q1 is the third quiet repeat and is not falsely paired with archived
G1. Report order effects as an uncontrolled limitation. Do not randomize after
seeing results. Five new attempted trials maximum; first substantive input,
output, timing, capacity, resource or capability failure stops the batch. Preserve
all attempts. Routine pre-sampling prerequisite failures are recorded and repaired
before collection; no replacement sampling beyond the declared slots.

Both modes retain the same causal semantic observer. G enables GPU timestamps;
Q disables them. Therefore the contrast estimates incremental timestamp/query,
polling and GPU-record logging effects in the observed application, not total
semantic-observer overhead or an uninstrumented GPU duration. Neither A1's
inconsistent marker contrasts nor this comparison warrants overhead subtraction.
Quiet mode has no GPU percentiles. Use external cgroup CPU totals over actual
recorded active boundaries, with delivered inputs and elapsed durations alongside
them; do not label handler wall time CPU or GPU time observed-display latency.
CPU observations describe instrumentation effects, not a CPU optimization study.

## Prerequisites and interpretation

Before each launch verify binary/CLI/method/reference/fixture hashes, clean
source pins, X11 display authorization, 1280×800 physical extent, scale 1, P630,
Vulkan, Bgra8UnormSrgb, Fifo, exact 8x, writable owned cgroup and no concurrent
compiler. Inherit the established display connection only; clear DATUM overrides,
Wayland and injected libraries. Use isolated XDG paths and no persistent tuning.
The launcher records complete arguments and explicit environment overrides.
No candidate rebuild is allowed by this packet.

All final presentation copies and upload continuations must remain covered by
accepted frame/submission boundaries. Reject missing, duplicate or incomplete
records. Preserve phase-boundary frames separately; classify by causal demand,
not log arrival. Never discard a slow valid sample. Compare readiness/final
images to independent archived evidence, inspect the captured crosshair, and
require received/applied integer pointer changes and final state to agree.

Report each GPU trial's N, nearest-rank p95/p99/max and the across-trial range;
keep archived/new trials identifiable. Report all quiet and GPU CPU observations,
paired differences and limitations without significance claims. No GPU-duty or
complete physical-memory claim without full DRM/lifetime accounting. Endpoint
pixels do not demonstrate continuous responsiveness. Replacement GPU budgets
remain undecided because this limited workload coverage and displayed-response/
hardware evidence cannot justify a product threshold. Owner review of that
rationale, rather than another automatic experiment, ends this bounded work.
