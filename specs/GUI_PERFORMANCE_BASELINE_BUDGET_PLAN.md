# Reference performance baseline and budget derivation

Authority: PM051 and exact owner direction in
`docs/reviews/gui-performance/baseline-budget/owner-direction.json`.
Issue `dat-gui-baseline-budgets-j9w8`; Frontier GUI-PERFORMANCE-BASELINE.
Status: bounded pointer assessment complete; representative baseline incomplete.
GBB-P04 owner disposition pending; no established replacement acceptance budget.
Replacement GPU p95/p99: **undecided**. Functionality follows its own Frontier.

## Evidence already available

`docs/reviews/gui-performance/baseline-budget/baseline-inventory.json` pins the
preserved R4 production candidate and reusable observations. Its single valid
native W-POINTER trial has 1275 active GPU samples, p95 6.700166871 ms, p99
10.205083645 ms and max 12.937750395 ms, with exact endpoint pixels and verified
3600 requests/1280 integer changes. This is valid historical measurement, not a
representative multi-trial baseline or demonstrated end-to-end responsiveness.
The former4/8 ms exceedance is no longer itself a product defect.

Choose corrected R4 provisionally for its current architecture and existing
correctness proof, not because it is fastest. Candidate source
`c3949e53ec7d68e70535892262024fa9f340c00f`, preserved production binary SHA256
`5c1c8f35746b55397f95280799be3b7bc4f28984b76f94514aaac6ef4933816b`.
Reference: Intel P630, Vulkan, Mesa25.0.7-2+deb13u1, X11/Xwayland,1280×800, scale1,
Bgra8UnormSrgb, exact 8×, Fifo, F-DOA. P01 must verify stability/source/configuration
suitability before expanding claims; 30 seconds without failure is not endurance.
Do not switch to the historical 4.330583 ms candidate to improve reported numbers.
If this candidate fails a substantive prerequisite, preserve the failure and
return an incomplete baseline; selecting another candidate needs a new explicit
rationale, not a favorable replacement run.

A1 conformance, fixture verification and instrumentation observations are reusable
at their diagnostic scope. Its native COPY_SRC/common reset/serialized readback
and two fixed transitions do not supply production workload percentiles or a
universal marker correction. Its inconsistent marker/completion contrasts leave
native instrumentation effects unresolved. Prior wrong-layout data remain
invalid for intended-workload attribution. Failed attempts stay in the inventory.

## Completion steps

<!-- REQ:GUI-PERFORMANCE-BASELINE:GBB-P01 -->
**GBB-P01 — Freeze the gap-only collection protocol.** Start from the inventory,
verify candidate and input/output pins, and admit a finite workload/metric matrix.
Representative initial scope covers idle, pointer, pan, zoom, Preferences
scroll/controls, pane transitions, native window lifecycle and mixed admitted
interaction. Group related actions into the existing recipes; do not replace
real native interactions with favorable synthetic scenes. Record why each is
representative, including rates, durations, warm/cold distinctions and consumer
coverage. Unadmitted schematic/T2/other hardware and separately deferred resize
remain excluded and explicit. Do not call one pointer trial representative of
all functionality.

For each cell identify reusable evidence, precisely missing evidence, method
validity, instrumentation needed, delivered-output oracle, and fixed attempts.
Use MET/GPU/ACC/STAT methods where adequate; annotate changed timing boundaries
rather than merging unlike distributions. Preserve final copies and every frame
submission/upload in complete timing. Pair instrumented and uninstrumented modes
under identical delivered input/output and configuration; report measured effects
and variation without universal subtraction. If a metric's observer cannot be
validated within the packet, leave that metric inconclusive rather than building
an open-ended observer or alternate method campaign.

Collection envelope: one pinned candidate, one reference host/configuration,
at most eight named workload families above, at most three declared native
trials per family per instrumented/quiet mode (48 attempts maximum across the
packet), with original recipe durations and at most 300 seconds per process.
These are collection bounds, not performance budgets. Reuse a trial only if its
pins, boundary, recipe and validity satisfy the declared cell; subtract reusable
trials from the needed count. Do not run all 48 by default. Three per mode reuses
MET-01's descriptive repeatability convention; it is not a power calculation
or justification for a binding tail threshold. No same-mode replacement attempts
beyond the frozen count; incomplete coverage remains incomplete. Separate cold
start/endurance/DRM campaigns are not silently added. A required unresolved method
or material correctness/resource failure stops the affected cell/packet under
its frozen policy; routine pre-sampling setup errors are recorded and repaired.

Pin exact executables/configuration, source changes if instrumentation is needed,
accounting/storage limits, commands and stop/invalidity rules before execution.
Existing resource caps apply; no renderer strategy or optimization changes.
Select GBB-P02 and execution authority in a synchronized claim transaction when
the protocol is concrete and reviewable under this owner-authorized scope.
No new owner approval loop is introduced for ordinary in-scope setup repair.

<!-- REQ:GUI-PERFORMANCE-BASELINE:GBB-P02 -->
**GBB-P02 — Collect only missing evidence and publish the observed baseline.**
Execute the frozen finite protocol, preserving all attempted trials and failures.
Before the first timed trial verify input paths and expected state/images using
independent evidence, and verify instrumentation conformance and complete timing.
Include accepted/coalesced/lost input, eligible/complete GPU frames, output checks,
missing observations, cold/warm population, explicit exclusions and display
observation limitations. A matching endpoint alone is not continuous responsiveness.

Per trial report p95/p99 using nearest rank, N, max and configuration/method hashes.
Preserve archived percentile values and name their original estimator; do not
silently relabel them nearest-rank. Recompute from retained raw samples only if
needed for a comparable cell, showing both versions without new sampling.
If there are no valid samples, report unavailable, not 0. Show all per-trial values
and their ranges/variability; do not pool to hide outliers or omit slow valid runs.
Describe instrumentation-on/off effects with matched output, clock resolution and
uncertainty. Report CPU, GPU, presentation and observer costs separately. Missing
complete DRM client lifetimes preclude duty claims, not fabricated low duty.
A partial or inconclusive baseline is an allowed truthful result, with each gap
and its implication. It does not automatically authorize another collection.

<!-- REQ:GUI-PERFORMANCE-BASELINE:GBB-P03 -->
**GBB-P03 — Derive candidate budgets or record undecided.** Complete the adjacent
threshold audit and reconcile distributions with observed responsiveness, supported
hardware expectations and workload/configuration scope. For every proposed
threshold cite derivation, exact evidence, applicable configurations and review
conditions. Explain the amount of headroom or required improvement, tradeoffs,
uncertainty and feasibility; never use baseline plus an unexplained percentage.
A supported-hardware policy is owner product input, not inferred from one P630.
When responsiveness or hardware evidence is inadequate for a number, explicitly
leave it undecided. Distinguish admission guards from latency budgets and alert
triggers. No change to retained adjacent thresholds until owner disposition.

<!-- REQ:GUI-PERFORMANCE-BASELINE:GBB-P04 -->
<!-- OWNER:GUI-PERFORMANCE-BASELINE:GBB-P04:GBB-P04 -->
**GBB-P04 — Owner disposition of the exact rationale.** Present the pinned
baseline, all trial/invalidity records, audit, configurations, rationale and review
conditions. The owner may approve specified replacement budgets as binding,
request revision, or accept an explicitly undecided disposition. Independent
review is not ratification. Update the numbered decision, matrix, roadmap and
handoff together; no threshold becomes blocking before this exact approval.
An undecided disposition closes an assessment only, not performance qualification.
No automatic implementation or measurement successor follows.

## Preserved correctness and handoff

All exact 8×/fidelity, painter/input/hit/focus/selection, mutation authority,
invalidation/recovery and live/retiring resource correctness obligations remain.
Other numerical caps retain their authority while audited. Functionality is not
blocked globally on baseline/budget or blanket S4/S5 completion, but an affected
feature must resolve its actual correctness blockers and prove its changes.
Existing broader qualification is neither waived nor relabeled as complete.

See `docs/reviews/gui-performance/baseline-budget/handoff.md` for claim release,
canonical functionality selection, evidence locations and forbidden inferences.

## Bounded collection disposition

The concrete `docs/reviews/gui-performance/baseline-budget/collection-protocol.md`
freezes the complete admission matrix, including unavailable methods. Its five
new pointer trials completed, with one archived GPU trial reused. The measured
findings and undecided budget rationale are in `pointer-report.md` beside it.
GBB-P02 completion means that frozen partial packet ended, not that the full
representative baseline was obtained. Seven other workload families and total
observer/display/hardware evidence remain unqualified. GBB-P04 may disposition
this limited assessment or request revised scope; it cannot ratify a replacement
number because none is proposed. No implicit further collection is authorized.
