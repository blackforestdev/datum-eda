# Deferred CPU investigation after S4 closure

Current scheduling disposition: this completed assessment is retained at its
recorded scope. The subsequent owner reset and
[resize investigation](resize-investigation.md) supersede its proposed
pointer/control packet and numerical-basis prerequisite for descriptive
investigation. Budgets remain undecided; no repeat audit, derivation or pointer
trial is a resize prerequisite. See `owner-direction.json` → `resize_reset`.


<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:CPU-AUDITED -->
Audit at 61be2e83, GUI source unchanged from corrected R4 c3949e53. No new runtime,
production edit, build, profile or measurement. This explicitly retains the CPU
part of the original repair; S4 closure did not complete or cancel it.

## Authority and scheduling

The implementation contract's GPU-redraw amendment explicitly says CPU event-loop
investigation follows the GPU solution and subsequent authorized selection. Its
S4 exit required explanation of material CPU regressions without opening CPU
optimization. S4 checklist C12 credited only absence of a demonstrated unexplained
matched production regression; it never established clean CPU acceptance.
PM045 GP-045-05 and the implementation contract GPI-S5 retain full applicable
CPU qualification. AM MET-02/03/05/06 and STAT-01 define total application CPU,
semantic-action denominators, event/preparation/encoding tails, diagnostic mode
separation and per-trial/relative reporting. PM051 preserves adjacent CPU limits
pending provenance review; it withdraws GPU 4/8 ms only.

The unresolved original issue is `dat-gui-pointer-layout-x1r` (pointer/zoom CPU
and presentation cost), related to the main `dat-gui-performance-implementation-vkq`.
Previously it was open in beads without its own scheduled Frontier step: CPU was
only mentioned as deferred prose. This audit repairs that scheduling omission.

Within GUI-PERFORMANCE-IMPLEMENTATION, GPI-CPU-AUDIT completes this evidence audit;
GPI-CPU-SCOPE is the owner decision on only the missing establishment packet below;
GPI-CPU-ESTABLISH records its eventual outcome, after explicit scope approval.
These steps follow completed GPI-S4 and remain ahead of final GPI-S5 disposition.
There is no CPU optimization step or automatic execution grant. S5's separate
method-plan disposition stays pending. GBB-P05 owns budget derivation and broader
baseline work, not this focused check of whether a current CPU issue exists.

## Evidence credited, with limits

Evidence paths below are relative to `docs/reviews/gui-performance/` unless noted.
`evidence.json` pins reviewed records, source and current per-trial CPU values.

| Evidence | What it establishes | What it does not establish now |
|---|---|---|
| Original pointer/zoom bead and historical commits | Repeated layout solves, menu allocations and world preparation had actual corrections. Historical debug/driver/UI-thread profiles and native output tests retain their scope | Old NVIDIA/private-X11, debug, synthetic handler and /tmp-only records are not a current P630 R4 total-process baseline. Neither old percentages nor experimental driver tuning are transferable |
| `implementation/S5/pass-consolidation/native-performance/assessment.json` (4eaa0087) | Three quiet pointer trials: 11.032%, 10.725%, 11.027% of one core, exact endpoints/normal exits; failures against then/current retained CPU criterion at that pin | Predates shared-session/R4 changes and current observer; not evidence today's ordinary pointer cost still exceeds 10% |
| `implementation/S5/clamp-matched-batch/result.json` (374b0a66) | Corrected native clamp recipe; historical quiet CPU ~1.442–1.540%, exceeding 1%, with zero camera work at bounds | Different candidate; cannot assert persistent polling or current clamp failure |
| `implementation/S5/terminal-idle-poll/result.json` | Atomic terminal wake fast path, deferred/no-proxy and drain/close behavior: 19 passing tests | Numerical effect and current native event-loop CPU were explicitly unqualified. Credit the fix; do not rediscover it or treat it as a performance pass |
| `implementation/S5/lazy-terminal-pipeline/native-startup-batch/assessment.json` (0709fc90 scope) | Three candidate trials/270 warm opens/closes satisfy bounded X11/1x CPU/output limits; matched baseline also passes | Supersedes the older open-overrun claim only at matching scope; not current R4/all-config acceptance or causal attribution. Keep original failures |
| `implementation/S5/repeated-device-recovery/native20-ba24e35a/assessment.json` | 20 actual recoveries, exact selected-window state/focus, normal exit; diagnostic intervals 275.743–774.777 ms exceed 100 ms | Diagnostics-on elapsed/CPU accounting must retain its exact recorded metric; not a clean current recovery cost or proof of an event-loop cause |
| `implementation/S5/window-endurance/native-60minute-4d645582/drift-analysis.json` | One producer hour, sampled tracked behavior and diagnostics-on first/last CPU analysis are reusable | Diagnostic trend evidence cannot establish current ordinary CPU, all instantaneous resource behavior or a leak root cause |
| `gpu-redraw-proposal/attribution-a1/corrected-result/report.md` | Fixed diagnostic preparation medians ~0.064–0.065 ms; preparation bypass contrast ~0.025 ms. Marker and end-to-end attribution inconclusive | No native event-loop distribution, scheduler attribution, ordinary total CPU, or justification for preparation optimization. A1 is finished, not reopened |
| `baseline-budget/pointer-analysis.json`, accepted report/receipt | Current R4 binary completes all input and exact endpoint/crosshair proof; mode-specific CPU table below | Semantic observer is present in quiet mode. No clean total CPU/action/tail acceptance or current event-loop diagnosis |
| `dat-gui-idle-sample-activity-lhb` | Earlier intermittent no-input activity preserved; three controlled repeats were quiet | Cannot discard original episodes, attribute them to user input, or assert a permanent current idle loop |

The historical layout/menu/world fixes and current shared renderer remain credited.
No demonstrated current CPU code defect is identified by this audit. That is not
proof CPU is low enough or the original concern resolved.

## Ordinary cost versus measurement cost

Current corrected R4, X11/1x, F-DOA single board, exact8/Fifo, 30-second pointer
workload; all values are saved cgroup CPU, not newly collected observations:

| Trial | Mode | CPU ms | Actual interval s | % of one core |
|---|---|---:|---:|---:|
| Q1 | GPU timing off, semantic observer on | 5131.972 | 30.087278 | 17.056950 |
| Q2 | GPU timing off, semantic observer on | 5202.786 | 30.085093 | 17.293568 |
| Q3 | GPU timing off, semantic observer on | 4952.783 | 30.080238 | 16.465239 |
| G2 | GPU query/poll/sample logging and semantic observer on | 6514.215 | 30.038891 | 21.685937 |
| G3 | GPU query/poll/sample logging and semantic observer on | 6260.805 | 30.035207 | 20.844887 |

G2−Q2 = 1311.429 ms / 4.392369 percentage points; G3−Q3 = 1308.022 ms /
4.379649 points. Two fixed-order descriptive contrasts, not a statistically
established universal tax. Archived G1 (23.703241%) has no paired quiet trial.

The launcher enters only the GUI child into the owned cgroup; descendants inherit
it. Input producer/capture/controller run outside it. Thus cgroup CPU includes
in-process observer/query/log work and child CPU, not the external harness CPU.
External work can still perturb scheduling; endpoint membership is not a separate
GUI/engine/per-thread attribution or proof every short-lived role was identified.

Ordinary application cost includes required event handling, shared preparation,
encoding, driver submission/presentation CPU and always-active correctness guards.
It must not be reduced by deleting accounting guards or offloading work to an
uncounted process. Optional semantic records, workload attribution, GPU queries,
polling and log serialization are measurement work. Boundary-mode differences
also include induced scheduling/work changes; simple subtraction does not uniquely
partition instruction costs. Q is **not** an uninstrumented lower bound or a
clean estimate of ordinary CPU; even its observer's sign/effect is not measured.

Source check: `baseline-budget/pointer_trial.py` supplies DATUM_INPUT_RECEIPT,
DATUM_WORKLOAD_MANIFEST and measurement shutdown in both modes.
`crates/gui-app/src/native_input_observation.rs::from_environment` supports absence
of the receipt only when the workload manifest/output diagnostic are absent.
The existing launcher and its receipt validator depend on those records. Removing
one environment variable is not a valid existing ordinary-mode experiment.
A clean control requires a bounded launcher/oracle adaptation, not renderer changes.
Without comparable delivered work, off/on differences cannot establish overhead.

No saved current ordinary-mode CPU duty, acknowledged CPU/action distribution,
per-event p95/p99, idle wake attribution or thread profile fills this gap. A1's
25 microseconds cannot fill it, nor may the measured ~1.31 seconds be subtracted
from a CPU/GPU threshold to manufacture acceptance.

## Threshold provenance audit

| Obligation | Provenance finding | Current authority |
|---|---|---|
| MET-03 idle/clamp 1%, T1 active 10%, Preferences/T0 5%, mixed15% | PM045 qualitatively places10% below owner-rejected32–35% resize. This is not a validated hardware/responsiveness derivation, especially across different workloads. Other duty choices lack complete derivations | Retained, not silently waived; historical exceedances remain at their pins. A future measured exceedance is a qualification observation, not by itself proof of an avoidable event-loop defect |
| MET-03 per-action ceilings | Several are duty/rate arithmetic; arithmetic does not validate the parent duty or semantic-action rate. 3600 requests,1280 integer changes and completed routes are distinct denominators | Retain exact semantic-action definition; report scheduled/received counts separately; no invented acknowledged actions |
| MET-05 CPU p95 4 ms/p99 8 ms and50 ms stall | Distinct from withdrawn GPU thresholds; documented validation is absent. A1 preparation-only medians do not validate them | Unchanged/unqualified; no replacement or withdrawal in this audit |
| Warm open50 ms/close20 ms and recovery100 ms | Scoped measurements exist, including corrections and failures; no complete supported-host/product derivation | Preserve values, applicable configurations and original evidence; broader qualification stays S5 |
| MET-06 repeatable5% regression and STAT-01 seven pairs | Protocol/review thresholds, not a product-cost justification or guaranteed statistical power | No current matched renderer regression established. Seven-pair formal claim not required merely to establish descriptive current cost |

This agrees with `baseline-budget/threshold-audit.md` and PM051. Unsupported
provenance is not proof a limit is wrong and grants no numerical waiver. Budget
rationale, responsiveness/hardware evidence and any amendment remain GBB-P05 and
owner disposition. CPU4/8 values must never be confused with withdrawn GPU4/8.

## Only missing work to establish the current issue

**Question:** what CPU cost and idle work does the current ordinary application exhibit on the pinned pointer workload, after separating optional measurement
cost? Do not start with the premise that it does or that the event loop causes it.

1. Prepare one bounded **harness-only** ordinary-mode control, using unchanged
   production binary/fixture and current pinned environment. List every disabled
   diagnostic, retain required production guards, account GUI plus all engine
   descendants, and specify input/focus/readiness/final-output verification outside
   timed windows. Establish how comparable delivered work can be verified without
   the in-process semantic observer. Server input delivery alone does not prove
   application processing. If this cannot be demonstrated, stop with a method
   blocker; do not collect uninterpretable low CPU. No streaming observer redesign.
2. Only after approval, one finite current-cost packet: three ordinary-mode and
   three matched semantic-only pointer trials, alternating mode order, identical
   state and exact30-second input schedule; one60-second no-input window in each
   launch after settling. GPU timing stays disabled. Six launches maximum; no
   extra GPU mode, benchmark baseline candidate, wheel/zoom, clamp, recovery,
   backend sweep or formal comparative claim. Existing Q/G evidence supplies the
   timestamp-mode finding; no repeat to reaffirm it. The semantic repeats are
   justified only as contemporary controls for the previously absent ordinary mode.
3. Use process-tree total user+system CPU/actual elapsed, per-process contributions,
   raw counts and exact output at scope. Ordinary idle windows test unexplained
   continued cost without input. Per-event tails, precise CPU/action, task/driver
   attribution and full input-path equivalence are not inferred from process totals.
   Report all trials and descriptive off/on differences; no overhead subtraction,
   significance claim or favorable-run selection. Stop on incorrect input/output,
   incomplete process accounting, failed comparable-work proof or operational failure;
   preserve failures, no replacement or optimization automatically follows.
4. Disposition only: if current clean measurements do not reproduce the concern,
   record that bounded non-reproduction and leave zoom/other configurations to
   their actual qualification scope. If a repeatable cost concern remains with valid controls, report its measured
   scope without declaring a product defect. A separately approved attribution
   question needs a concrete unnecessary-work observation or an owner-prioritized
   responsiveness concern; only then could
   an event-loop/thread/driver profile be justified. If overhead dominates or
   controls are inconclusive, report that rather than assign a renderer cause.
   CPU threshold provenance remains a separate acceptance-policy question.

This is a residual scope, **not an approved runnable packet**: the clean-mode
oracle/launcher remains to be specified and reviewed before those six launches.
No new measurement is authorized here. Full DRM retirement accounting and native
fractional-line delivery do not block this pointer-only CPU establishment packet;
they still block their distinct S5 rows. Likewise this packet does not replace
S5's complete CPU/resource qualification or GBB-P05's representative baseline.

<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-CPU-SCOPE:CPU -->
Recommended next scope: prepare the bounded harness-only ordinary-mode control
and exact six-launch CPU establishment packet for review, with zero execution.
Alternatively defer CPU explicitly, keeping the original issue open. Neither
choice grants optimization or changes the existing S5 method disposition.

## Owner clarification: no invented CPU acceptance baseline

The owner explicitly cautioned against repeating the unsupported GPU p95/p99
budget error. Observed CPU distributions, acceptable CPU budgets and optimization
priorities are separate. This audit proposes **no new binding number**, headroom
percentage or required improvement. A historical/current overrun of a retained
criterion is reported at its actual scope; its unsupported derivation is disclosed
and does not establish an economically worthwhile optimization or a renderer bug.
The existing CPU requirements are not silently withdrawn by this audit.

The proposed three repetitions per mode and60-second idle windows reuse MET-01
recipe counts to bound descriptive observations. They do not establish statistical
power, a universal overhead, a supported-hardware baseline or acceptable CPU cost.
If the observations cannot answer their limited question, report inconclusive
and stop. No repeated sampling or new threshold follows. A binding replacement
requires explicit derivation, representative evidence, applicable configurations,
review conditions and owner approval under PM051/GBB-P05. Leave it undecided when
that evidence is insufficient.
