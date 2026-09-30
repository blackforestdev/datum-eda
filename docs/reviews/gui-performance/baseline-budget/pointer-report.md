# Reference baseline: measured pointer subset and budget disposition

<!-- EVIDENCE:GUI-PERFORMANCE-BASELINE:GBB-P02-POINTER -->
The frozen five-trial packet completed without setup, correctness, measurement
or resource-capacity failure. Two new GPU-timed trials and three quiet trials
reuse one archived GPU trial. No trial was discarded or replaced, no renderer
was changed or rebuilt, and no numerical stop used the withdrawn 4/8 ms figures.
The full representative baseline is **incomplete**, not accepted by this subset.

## Observed distributions

Candidate/source: corrected R4 `c3949e53ec7d68e70535892262024fa9f340c00f`;
SHA256 `5c1c8f35746b55397f95280799be3b7bc4f28984b76f94514aaac6ef4933816b`.
Intel P630, Vulkan, Mesa 25.0.7-2+deb13u1, X11/Xwayland, 1280×800 physical pixels,
scale 1, Bgra8UnormSrgb, Fifo, exact 8x, F-DOA single-board view. Workload is the
unchanged 30-second warm pointer rectangle, 120 requests/second, preceded/followed
by five-second warmup/still intervals. Device-live drain follows the final image.

| Trial | Timing samples | GPU p95 ms | GPU p99 ms | GPU max ms | Active cgroup CPU ms |
|---|---:|---:|---:|---:|---:|
| G1 (archived) | 1275 | 6.700167 | 10.205084 | 12.937750 | 7129.721 |
| Q1 | unavailable | unavailable | unavailable | unavailable | 5131.972 |
| G2 | 1280 | 5.724167 | 7.292250 | 8.715000 | 6514.215 |
| Q2 | unavailable | unavailable | unavailable | unavailable | 5202.786 |
| G3 | 1280 | 5.760250 | 6.443500 | 10.395917 | 6260.805 |
| Q3 | unavailable | unavailable | unavailable | unavailable | 4952.783 |

GPU percentiles use nearest rank on each trial's causally active complete frame
spans, including upload continuations and final presentation copies. Archived
raw samples reproduce the original values exactly. Its historical
`valid_budget_failure` label is preserved; PM051 withdraws that blocking
interpretation. No quantiles are pooled. Across the three trials p95 ranges
5.724167–6.700167 ms and p99 6.443500–10.205084 ms. The lower new values are
variation on an unchanged executable, not an optimization or a favorable-run
selection. G1 is separated in time and has no matched quiet observation.

## Measured instrumentation effects

G2 minus Q2 consumed 1311.429 ms more cgroup CPU; G3 minus Q3 consumed
1308.022 ms more. Normalizing by each observed interval gives 4.392369 and
4.379649 percentage points of one core. Intervals were 30.038891/30.085093 seconds
and 30.035207/30.080238 seconds, respectively; preserve these boundary differences.
The first quiet trial is a repeatability observation, not a fabricated partner
for archived G1. Two fixed-order pairs do not establish statistical significance
or eliminate order/background effects.

The changed mode enables GPU queries, polling and sample logging together.
Both modes retain the causal semantic observer. Thus this measures the mode's
incremental CPU effect, including any induced scheduling/work differences. It
neither isolates marker GPU cost nor measures total instrumentation overhead.
Quiet GPU tails are unavailable, not zero. Do not subtract CPU milliseconds or
A1's inconsistent marker contrasts from GPU timings. These instrumented CPU
observations are not a clean CPU-duty/action acceptance test; adjacent CPU
criteria retain their authority and remain unqualified here.

## Correctness and stability

All five new trials received and completed all 1280 integer pointer changes
from 3600 requested positions. Independent archived readiness and final images
matched with zero differing pixels; final capture state agreed with the applied
pointer and no later semantic transition changed it. The crosshair was visually
inspected in a quiet and GPU trial and matches the independent reference.
Camera, scale, focus, hover and single-board initialization were retained.
GPU receipt validation included complete drained frame/submission identity,
causal phase classification and trailing-copy boundaries. Quiet trials confirmed
GPU measurement was disabled. Every child exited normally, binaries and authored
fixture remained unchanged, and no process remained in its owned cgroup.

This supports repeated completion of this warm pointer workload, not endurance,
all-frame displayed-output correctness, other editor families or full resource
qualification. No newly demonstrated renderer defect emerged. Existing renderer
fixes and remaining correctness/lifetime obligations retain their prior scope.

## Why a full baseline or replacement budget is not established

<!-- EVIDENCE:GUI-PERFORMANCE-BASELINE:GBB-P03-UNDECIDED -->
The admission matrix in `collection-protocol.md` records seven remaining family
gaps. The unchanged observer enforces one native host and a 5/30/5 schedule;
it lacks the necessary wheel delta, per-pane state, multi-host lifecycle and
PTY-byte/output oracles for the other recipes. Earlier archives supply recipes
and some correctness evidence, but use other candidate/method pins and cannot
be pooled into this candidate's distributions. Extending those methods is a
separate substantive scope, not routine relaunch repair. This packet stops here;
no observer redesign, extra sampling or hardware sweep follows automatically.

Total observer effects, displayed responsiveness, supported hardware expectations
and complete DRM/resource lifetimes remain unresolved. Pointer GPU tails alone
cannot derive an acceptable product latency, a hardware floor, an improvement
requirement or justified headroom. Replacement budgets therefore remain
**undecided**. No replacement number is proposed for owner ratification.
The adjacent-threshold audit remains unchanged: arithmetic or an observed maximum
does not establish why a CPU, duty or resource cap is acceptable. Existing caps
are not silently relaxed, and observed results are not invalidated by withdrawal
of the unsupported GPU limits.

## Overall completion path

The renderer remains the corrected R4 implementation. Historical A1 attribution
and the new pointer observations provide bounded evidence, not a reason to resume
architecture experiments. The bounded collection and its budget assessment are
complete; representative baseline and broader GPU qualification are incomplete.
Their exact missing methods are now explicit. Functionality remains unblocked
under its own correctness and authorization gates; UVT-S5A-BUILD is still the
canonical functionality lane. Residual shared-renderer qualification remains
on GPI-S4-RECONCILE. Do not label S4/S5 or the GPU issue resolved.

GBB-P04 asks the owner to disposition this limited assessment with budgets left
undecided, or request an explicit revised scope. It cannot approve nonexistent
replacement numbers. Accepting the assessment does not accept the missing
representative baseline or complete runtime qualification.

Raw results, images, source/configuration pins and CPU observations are preserved
in `pointer-raw.tar.gz`; `pointer-state.json` records all attempts;
`pointer-analysis.json` contains unrounded per-trial values and comparisons;
`pointer-receipt.json` records checks and hashes. `pointer-final.png` is measured
output, not a new reference golden. Original historical files remain untouched.
